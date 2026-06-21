use super::model::RepoEntry;

/// Curated subset: standalone MCP server repos suitable for end-to-end dynamic scan.
/// Everything else is `tier2` (full corpus).
pub fn classify_tier(repo: &RepoEntry) -> &'static str {
    if is_tier1(repo) { "tier1" } else { "tier2" }
}

pub fn tier1_exclude_reason(repo: &RepoEntry) -> Option<&'static str> {
    if TIER1_BLOCKLIST.iter().any(|id| *id == repo.id) {
        return Some("blocklist");
    }

    let name = repo_name(repo);

    if MONOREPO_OR_APP_NAMES.iter().any(|n| name.as_str() == *n) {
        return Some("monorepo-or-app");
    }

    if name.contains("chrome")
        || name.contains("windows")
        || name.contains("xcode")
        || name.contains("blender")
        || name.contains("whatsapp")
    {
        return Some("platform-specific");
    }

    if name.ends_with("-hub") || name == "harbor" || name == "mcptools" {
        return Some("meta-or-tooling");
    }

    if repo.stars < 20 {
        return Some("low-stars");
    }

    if repo.dep_count > 120 {
        return Some("high-deps");
    }

    if !looks_like_dedicated_mcp_server(repo, &name) {
        return Some("not-dedicated-mcp");
    }

    None
}

pub fn assign_tiers(repos: &mut [RepoEntry]) {
    for repo in repos {
        repo.tier = classify_tier(repo).to_string();
    }
}

fn is_tier1(repo: &RepoEntry) -> bool {
    tier1_exclude_reason(repo).is_none()
}

fn repo_name(repo: &RepoEntry) -> String {
    repo.id
        .split('/')
        .nth(1)
        .unwrap_or(repo.id.as_str())
        .to_lowercase()
}

fn looks_like_dedicated_mcp_server(repo: &RepoEntry, name: &str) -> bool {
    if name.contains("mcp-server")
        || name.ends_with("-mcp")
        || name.contains("mcp-")
        || name.contains("mcp_server")
    {
        return true;
    }
    if name.ends_with("mcp") && name.len() > 3 {
        return true;
    }

    if repo.id == "modelcontextprotocol/servers" {
        return true;
    }

    if repo.id == "microsoft/playwright-mcp" || repo.id == "GLips/Figma-Context-MCP" {
        return true;
    }

    let topics: Vec<String> = repo.topics.iter().map(|t| t.to_lowercase()).collect();
    topics
        .iter()
        .any(|t| t == "mcp-server" || t.contains("mcp-server"))
        && (name.contains("mcp") || name.contains("server"))
}

/// Known non-server or unsuitable repos — never Tier-1 regardless of name.
const TIER1_BLOCKLIST: &[&str] = &[
    "activepieces/activepieces",
    "webiny/webiny-js",
    "opensumi/core",
    "nanbingxyz/5ire",
    "casdoor/casdoor",
    "stacklok/toolhive",
    "lharries/whatsapp-mcp",
    "clubpay/ronykit",
    "silkweave/silkweave",
    "ravitemer/mcp-hub",
    "f/mcptools",
    "av/harbor",
    "googleapis/mcp-toolbox",
    "hangwin/mcp-chrome",
    "CursorTouch/Windows-MCP",
    "getsentry/XcodeBuildMCP",
    "seehiong/blender-mcp-n8n",
    "zcaceres/markdownify-mcp",
    "idosal/git-mcp",
    "firecrawl/firecrawl-mcp-server",
    "brightdata/brightdata-mcp",
    "benborla/mcp-server-mysql",
    "containers/kubernetes-mcp-server",
    "datagouv/datagouv-mcp",
    "nixopus/nixopus",
    "kocierik/mcp-nomad",
    "its-dart/dart-mcp-server",
    "jhgaylor/node-candidate-mcp-server",
    "flaviodelgrosso/fastify-mcp-server",
    "bsmi021/mcp-chain-of-draft-server",
    "BrowserMCP/mcp",
];

const MONOREPO_OR_APP_NAMES: &[&str] = &[
    "activepieces",
    "webiny-js",
    "core",
    "5ire",
    "casdoor",
    "toolhive",
    "harbor",
    "silkweave",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(id: &str, stars: u64, dep_count: u32, topics: &[&str]) -> RepoEntry {
        RepoEntry {
            id: id.to_string(),
            url: format!("https://github.com/{id}"),
            clone_url: format!("https://github.com/{id}.git"),
            stars,
            language: Some("TypeScript".to_string()),
            topics: topics.iter().map(|s| s.to_string()).collect(),
            wasm_class: "wasm-hard".to_string(),
            resolved: true,
            scan_status: "resolved".into(),
            ecosystem: "npm".into(),
            dep_count,
            tier: String::new(),
            resolve_error: None,
            subject_toml: Some(format!("corpus/manifests/{}.toml", id.replace('/', "__"))),
        }
    }

    #[test]
    fn tier1_includes_playwright_and_github() {
        assert_eq!(
            classify_tier(&repo("microsoft/playwright-mcp", 34000, 30, &["mcp"])),
            "tier1"
        );
        assert_eq!(
            classify_tier(&repo("github/github-mcp-server", 20000, 40, &[])),
            "tier1"
        );
    }

    #[test]
    fn tier1_excludes_monorepos() {
        assert_eq!(
            classify_tier(&repo("activepieces/activepieces", 50000, 200, &["mcp"])),
            "tier2"
        );
        assert_eq!(
            tier1_exclude_reason(&repo("webiny/webiny-js", 5000, 300, &[])),
            Some("blocklist")
        );
    }

    #[test]
    fn tier1_excludes_platform_specific() {
        assert_eq!(
            classify_tier(&repo("hangwin/mcp-chrome", 1000, 20, &[])),
            "tier2"
        );
    }

    #[test]
    fn tier1_includes_aashari_servers() {
        assert_eq!(
            classify_tier(&repo(
                "aashari/boilerplate-mcp-server",
                500,
                25,
                &["mcp-server"]
            )),
            "tier1"
        );
    }
}
