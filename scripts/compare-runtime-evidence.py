#!/usr/bin/env python3
"""Recompute paired runtime evidence from checked-in corpus reports."""

import json
from pathlib import Path


REPORTS = Path(__file__).resolve().parents[1] / "mcp-sandboxscan" / "reports"
PAIRS = (
    ("Original", "corpus-run-1781854580", "corpus-targeted-scan-ok-1781854580-v2"),
    ("Supplementary", "budget-shallow-final", "budget-targeted-final"),
)


def successful_cases(directory):
    summary = json.loads((directory / "corpus_summary.json").read_text())
    cases = {case["repo_id"]: case for case in summary["cases"]}
    if len(cases) != len(summary["cases"]):
        raise ValueError(f"Duplicate repository IDs in {directory}")
    return {repo: case for repo, case in cases.items() if case["scan_ok"]}


def counts(directory, cases, common):
    flow_repos = witnesses = 0
    for repo in sorted(common):
        case = cases[repo]
        report = json.loads(
            (directory / "cases" / Path(case["report_path"]).name).read_text()
        )
        count = len(report["flows"])
        if not (
            count == report["summary"]["num_flows"] == case["num_flows"]
            and bool(count) == case["has_flow"]
        ):
            raise ValueError(f"Inconsistent flow counts: {directory.name}: {repo}")
        flow_repos += bool(count)
        witnesses += count
    return flow_repos, witnesses


def main():
    print("| Experiment | Common successful repos | Shallow flow repos / witnesses | "
          "Targeted flow repos / witnesses | Net increase repos / witnesses |")
    print("|---|---:|---:|---:|---:|")
    for label, shallow, targeted in PAIRS:
        directories = [REPORTS / name for name in (shallow, targeted)]
        cases = [successful_cases(directory) for directory in directories]
        common = cases[0].keys() & cases[1].keys()
        if not common:
            raise ValueError(f"No common successful repositories: {label}")
        before, after = [
            counts(directory, records, common)
            for directory, records in zip(directories, cases)
        ]
        delta = [new - old for old, new in zip(before, after)]
        print(f"| {label} | {len(common)} | {before[0]} / {before[1]} | "
              f"{after[0]} / {after[1]} | {delta[0]:+d} / {delta[1]:+d} |")


if __name__ == "__main__":
    main()
