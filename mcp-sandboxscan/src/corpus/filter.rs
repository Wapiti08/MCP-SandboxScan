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
        assert_eq!(
            reject_reason("modelcontextprotocol/servers", &[]),
            None
        );
    }

    #[test]
    fn rejects_without_mcp_signal() {
        assert_eq!(
            reject_reason("someone/random-tool", &[]),
            Some(RejectReason("no-mcp-signal"))
        );
    }
}
