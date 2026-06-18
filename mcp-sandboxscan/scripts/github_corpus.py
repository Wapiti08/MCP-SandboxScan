#!/usr/bin/env python3
"""Collect MCP-related GitHub repos into corpus/repos.json.

Prefers `gh` when installed; falls back to GitHub REST API via curl.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import urllib.parse
from datetime import datetime, timezone
from pathlib import Path

QUERIES = [
    "mcp server stars:>10",
    '"model context protocol" server stars:>5',
    "topic:mcp topic:server",
]

CURL_QUERIES = [
    "mcp+server+stars:>10",
    "model+context+protocol+server+stars:>5",
    "topic:mcp+topic:server",
]

# Keep in sync with src/corpus/filter.rs
BLOCK_ID_SUBSTR = [
    "awesome-mcp",
    "awesome",
    "curated",
    "python-sdk",
    "typescript-sdk",
    "go-sdk",
    "csharp-sdk",
    "kotlin-sdk",
    "specification",
    "registry",
    "inspector",
    "documentation",
    "client-sdk",
    "/docs-",
    "-docs",
    "hacktoberfest",
    "learning",
    "tutorial",
    "course",
    "interview",
    "roadmap",
    "cheatsheet",
    "awesome-list",
]

LIB_REPO_NAMES = [
    "fastmcp",
    "python-sdk",
    "typescript-sdk",
    "go-sdk",
    "csharp-sdk",
    "kotlin-sdk",
    "mcp-go",
    "mcp-python",
    "mcp-typescript",
    "sdk",
]

BLOCK_TOPICS = [
    "awesome-list",
    "awesome",
    "curated-list",
    "documentation",
    "tutorial",
]


def reject_reason(repo_id: str, topics: list[str]) -> str | None:
    lower = repo_id.lower()
    name = lower.split("/", 1)[1] if "/" in lower else lower

    for pat in BLOCK_ID_SUBSTR:
        if pat in lower:
            return pat

    for lib in LIB_REPO_NAMES:
        if name == lib or name.endswith(f"-{lib}"):
            return "sdk-library"

    if name.endswith("-sdk") and "server" not in name:
        return "sdk-suffix"

    topics_lc = [t.lower() for t in topics]
    for topic in topics_lc:
        for pat in BLOCK_TOPICS:
            if topic == pat or pat in topic:
                return "topic-blocklist"

    has_mcp_in_id = "mcp" in lower or "modelcontextprotocol" in lower
    has_mcp_topic = any("mcp" in t for t in topics_lc)
    if not has_mcp_in_id and not has_mcp_topic:
        return "no-mcp-signal"

    return None


def apply_filter(repos: list[dict]) -> tuple[list[dict], dict[str, int]]:
    kept: list[dict] = []
    reasons: dict[str, int] = {}
    for row in repos:
        reason = reject_reason(row["id"], row.get("topics") or [])
        if reason:
            reasons[reason] = reasons.get(reason, 0) + 1
            continue
        kept.append(row)
    return kept, reasons


def wasm_class(lang: str | None) -> str:
    l = (lang or "").lower()
    if l in ("rust", "go"):
        return "wasm-ready"
    if l == "python":
        return "wasm-needs-runtime"
    if l in ("javascript", "typescript"):
        return "wasm-hard"
    return "unknown"


def gh_search(query: str, limit: int) -> list[dict]:
    cmd = [
        "gh",
        "search",
        "repos",
        query,
        "--limit",
        str(limit),
        "--json",
        "nameWithOwner,url,stargazersCount,primaryLanguage,repositoryTopics",
    ]
    out = subprocess.check_output(cmd, text=True)
    rows = json.loads(out)
    repos = []
    for row in rows:
        full = row["nameWithOwner"]
        lang = (row.get("primaryLanguage") or {}).get("name")
        repos.append(
            {
                "id": full,
                "url": row["url"],
                "clone_url": f"https://github.com/{full}.git",
                "stars": row.get("stargazersCount", 0),
                "language": lang,
                "topics": row.get("repositoryTopics") or [],
                "wasm_class": wasm_class(lang),
                "resolved": False,
                "scan_status": "pending",
            }
        )
    return repos


def curl_search(query: str, limit: int) -> list[dict]:
    per_page = max(1, min(limit, 100))
    url = (
        "https://api.github.com/search/repositories?q="
        + urllib.parse.quote(query)
        + f"&sort=stars&order=desc&per_page={per_page}"
    )
    cmd = [
        "curl",
        "-fsSL",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: mcp-sandboxscan-corpus",
        url,
    ]
    out = subprocess.check_output(cmd, text=True)
    data = json.loads(out)
    repos = []
    for item in data.get("items", []):
        lang = item.get("language")
        full = item["full_name"]
        repos.append(
            {
                "id": full,
                "url": item["html_url"],
                "clone_url": item["clone_url"],
                "stars": item.get("stargazers_count", 0),
                "language": lang,
                "topics": item.get("topics") or [],
                "wasm_class": wasm_class(lang),
                "resolved": False,
                "scan_status": "pending",
            }
        )
    return repos


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=50, help="per query seed")
    ap.add_argument("--out", type=Path, default=Path("corpus/repos.json"))
    args = ap.parse_args()

    seen: set[str] = set()
    repos: list[dict] = []
    use_gh = shutil.which("gh") is not None

    queries = QUERIES if use_gh else CURL_QUERIES
    search_fn = gh_search if use_gh else curl_search

    for q in queries:
        try:
            batch = search_fn(q, args.limit)
        except (subprocess.CalledProcessError, FileNotFoundError) as err:
            print(f"search failed for {q!r}: {err}", file=sys.stderr)
            if not use_gh:
                print("install curl or use: cargo run --bin corpus -- collect --seed", file=sys.stderr)
            return 1
        for row in batch:
            if row["id"] in seen:
                continue
            seen.add(row["id"])
            repos.append(row)

    raw = len(repos)
    repos, reject_reasons = apply_filter(repos)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "collected_at": datetime.now(timezone.utc).isoformat(),
        "queries": list(queries),
        "repos": sorted(repos, key=lambda r: (-r["stars"], r["id"])),
    }
    args.out.write_text(json.dumps(payload, indent=2) + "\n")
    backend = "gh" if use_gh else "curl"
    print(f"wrote {len(repos)} repos -> {args.out} (via {backend})")
    if reject_reasons:
        print(f"filtered {raw - len(repos)} of {raw} raw repos")
        for reason, count in sorted(reject_reasons.items(), key=lambda x: -x[1])[:10]:
            print(f"  {count:4d}  {reason}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
