use std::collections::HashMap;

/// Why a repository was excluded during corpus collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectReason(pub &'static str);

/// Returns `Some(reason)` when the repo should **not** be collected.
pub fn reject_reason(repo_id: &str, topics: &[String]) -> Option<RejectReason> {
    let lower = repo_id.to_lowercase();
    let name = lower.split('/').nth(1).unwrap_or(lower.as_str());

    // Curated lists, docs, registries, and inspector tooling — not runnable servers.
    const BLOCK_ID_SUBSTR: &[&str] = &[
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
    ];

    for pat in BLOCK_ID_SUBSTR {
        if lower.contains(pat) {
            return Some(RejectReason(pat));
        }
    }

    // Standalone SDK / library repos (no server entrypoint expected).
    const LIB_REPO_NAMES: &[&str] = &[
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
    ];

    for lib in LIB_REPO_NAMES {
        if name == *lib || name.ends_with(&format!("-{lib}")) {
            return Some(RejectReason("sdk-library"));
        }
    }

    // Name ends with `-sdk` but not `*-mcp-server`.
    if name.ends_with("-sdk") && !name.contains("server") {
        return Some(RejectReason("sdk-suffix"));
    }

    let topics_lc: Vec<String> = topics.iter().map(|t| t.to_lowercase()).collect();

    const BLOCK_TOPICS: &[&str] = &[
        "awesome-list",
        "awesome",
        "curated-list",
        "documentation",
        "tutorial",
    ];

    for topic in &topics_lc {
        for pat in BLOCK_TOPICS {
            if topic == *pat || topic.contains(pat) {
                return Some(RejectReason("topic-blocklist"));
            }
        }
    }

    // Must look MCP-related at all.
    let has_mcp_in_id = lower.contains("mcp")
        || lower.contains("modelcontextprotocol")
        || lower.contains("model-context-protocol");
    let has_mcp_topic = topics_lc.iter().any(|t| t.contains("mcp"));
    if !has_mcp_in_id && !has_mcp_topic {
        return Some(RejectReason("no-mcp-signal"));
    }

    None
}

/// Stricter filter: likely resolvable dedicated MCP server repos only.
pub fn reject_reason_strict(
    repo_id: &str,
    topics: &[String],
    language: Option<&str>,
) -> Option<RejectReason> {
    if let Some(reason) = reject_reason(repo_id, topics) {
        return Some(reason);
    }

    if let Some(lang) = language {
        match lang.to_lowercase().as_str() {
            "java" | "c" | "c++" | "c#" | "shell" | "kotlin" | "swift" | "ruby" | "php"
            | "dart" | "scala" => {
                return Some(RejectReason("unsupported-language"));
            }
            _ => {}
        }
    }

    let lower = repo_id.to_lowercase();
    let name = lower.split('/').nth(1).unwrap_or(lower.as_str());

    if name == "mcp" {
        return Some(RejectReason("umbrella-mcp-repo"));
    }

    const BLOCK_NAME_SUBSTR: &[&str] = &[
        "headroom",
        "mcp-use",
        "mockserver",
        "jscpd",
        "hexstrike",
        "-client",
        "client-sdk",
        "monorepo",
        "notebooklm-mcp-cli",
        "codebase-memory",
        "mcp-for-beginners",
        "-beginners",
        "gpt-researcher",
        "openmetadata",
        "funasr",
        "maxkb",
        "trigger.dev",
        "nuclear",
        "trendradar",
        "scrapling",
        "nginx-ui",
        "xiaozhi-esp32",
    ];

    for pat in BLOCK_NAME_SUBSTR {
        if name.contains(pat) {
            return Some(RejectReason("not-mcp-server"));
        }
    }

    if name.ends_with("-cli") && !name.contains("mcp-server") {
        return Some(RejectReason("cli-tool"));
    }

    if name == "sandbox" || (name.ends_with("-sandbox") && !name.contains("mcp-server")) {
        return Some(RejectReason("sandbox-repo"));
    }

    if !looks_like_dedicated_server(repo_id, name, topics) {
        return Some(RejectReason("not-dedicated-server"));
    }

    // Avoid giant platform repos that only mention MCP in docs/topics.
    if !name.contains("mcp") {
        let has_mcp_server_topic = topics.iter().any(|t| t.to_lowercase() == "mcp-server");
        if !has_mcp_server_topic {
            return Some(RejectReason("no-mcp-in-name"));
        }
    }

    None
}

fn looks_like_dedicated_server(repo_id: &str, name: &str, topics: &[String]) -> bool {
    if repo_id == "modelcontextprotocol/servers" {
        return true;
    }

    if name.contains("mcp-server")
        || name.ends_with("-mcp")
        || name.contains("mcp-")
        || (name.ends_with("mcp") && name.len() > 3)
    {
        return true;
    }

    topics.iter().any(|t| {
        let tl = t.to_lowercase();
        tl == "mcp-server" || tl.contains("mcp-server")
    })
}

#[derive(Debug, Clone, Default)]
pub struct CollectFilterStats {
    pub raw: usize,
    pub kept: usize,
    pub rejected: usize,
    pub reasons: HashMap<String, usize>,
}

impl CollectFilterStats {
    pub fn record_reject(&mut self, reason: &str) {
        self.rejected += 1;
        *self.reasons.entry(reason.to_string()).or_default() += 1;
    }
}

pub fn apply_collect_filter(
    repos: Vec<super::model::RepoEntry>,
) -> (Vec<super::model::RepoEntry>, CollectFilterStats) {
    let mut stats = CollectFilterStats {
        raw: repos.len(),
        ..Default::default()
    };
    let mut kept = Vec::new();

    for repo in repos {
        if let Some(RejectReason(reason)) = reject_reason(&repo.id, &repo.topics) {
            stats.record_reject(reason);
            continue;
        }
        stats.kept += 1;
        kept.push(repo);
    }

    (kept, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_awesome_list() {
        assert_eq!(
            reject_reason("punkpeye/awesome-mcp-servers", &[]),
            Some(RejectReason("awesome-mcp"))
        );
    }

    #[test]
    fn rejects_python_sdk() {
        assert_eq!(
            reject_reason("modelcontextprotocol/python-sdk", &[]),
            Some(RejectReason("python-sdk"))
        );
    }

    #[test]
    fn rejects_fastmcp_library() {
        assert_eq!(
            reject_reason("jlowin/fastmcp", &[]),
            Some(RejectReason("sdk-library"))
        );
    }

    #[test]
    fn keeps_playwright_mcp() {
        assert_eq!(reject_reason("microsoft/playwright-mcp", &[]), None);
    }

    #[test]
    fn keeps_mcp_servers_monorepo() {
        assert_eq!(reject_reason("modelcontextprotocol/servers", &[]), None);
    }

    #[test]
    fn rejects_without_mcp_signal() {
        assert_eq!(
            reject_reason("someone/random-tool", &[]),
            Some(RejectReason("no-mcp-signal"))
        );
    }

    #[test]
    fn strict_rejects_headroom() {
        assert_eq!(
            reject_reason_strict("chopratejas/headroom", &[], Some("Python")),
            Some(RejectReason("not-mcp-server"))
        );
    }

    #[test]
    fn strict_keeps_playwright() {
        assert_eq!(
            reject_reason_strict("microsoft/playwright-mcp", &[], Some("TypeScript")),
            None
        );
    }

    #[test]
    fn strict_rejects_java() {
        assert_eq!(
            reject_reason_strict("LaurieWired/GhidraMCP", &[], Some("Java")),
            Some(RejectReason("unsupported-language"))
        );
    }
}
