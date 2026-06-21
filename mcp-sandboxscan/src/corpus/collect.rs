use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::classify::wasm_class_from_language;
use super::filter::{CollectFilterStats, apply_collect_filter};
use super::model::{CorpusFile, RepoEntry};

const QUERIES: &[&str] = &[
    "mcp+server+stars:>10",
    "model+context+protocol+server+stars:>5",
    "topic:mcp+topic:server",
    "topic:mcp-server",
    "mcp-server+language:typescript+stars:>5",
    "mcp-server+language:go+stars:>5",
    "mcp-server+language:python+stars:>5",
    "mcp-server+language:rust+stars:>5",
    "model+context+protocol+in:name+stars:>3",
    "mcp-server-tool+stars:>3",
];

pub struct CollectOptions {
    pub limit_per_query: usize,
    /// Stop once at least this many repos remain after filtering.
    pub target_repos: Option<usize>,
    /// Keep only repos that look like dedicated, resolvable MCP servers.
    pub strict: bool,
    /// Preserve resolve/scan fields from an existing corpus file.
    pub merge_from: Option<std::path::PathBuf>,
}

pub struct CollectResult {
    pub corpus: CorpusFile,
    pub filter_stats: CollectFilterStats,
}

/// Collect repos via GitHub REST search (curl). Does not require `gh` CLI.
pub fn collect_github(opts: &CollectOptions) -> Result<CollectResult> {
    let mut seen = std::collections::HashSet::new();
    let mut repos = Vec::new();

    if let Some(path) = &opts.merge_from {
        if path.exists() {
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("read merge corpus {}", path.display()))?;
            let existing: CorpusFile = serde_json::from_str(&raw)?;
            for repo in existing.repos {
                seen.insert(repo.id.clone());
                repos.push(repo);
            }
        }
    }

    let mut filter_stats = CollectFilterStats {
        raw: 0,
        kept: repos.len(),
        rejected: 0,
        reasons: std::collections::HashMap::new(),
    };

    'queries: for query in QUERIES {
        if let Some(target) = opts.target_repos {
            if repos.len() >= target {
                break 'queries;
            }
        }

        let batch = github_search_curl(query, opts.limit_per_query)?;
        filter_stats.raw += batch.len();

        for row in batch {
            if !seen.insert(row.id.clone()) {
                continue;
            }
            if let Some(reason) = if opts.strict {
                super::filter::reject_reason_strict(&row.id, &row.topics, row.language.as_deref())
            } else {
                super::filter::reject_reason(&row.id, &row.topics)
            } {
                filter_stats.record_reject(reason.0);
                continue;
            }
            filter_stats.kept += 1;
            repos.push(row);
            if opts
                .target_repos
                .is_some_and(|target| repos.len() >= target)
            {
                break 'queries;
            }
        }
    }

    repos.sort_by(|a, b| b.stars.cmp(&a.stars).then_with(|| a.id.cmp(&b.id)));

    Ok(CollectResult {
        corpus: CorpusFile {
            collected_at: chrono_like_now(),
            queries: QUERIES.iter().map(|q| q.to_string()).collect(),
            repos,
        },
        filter_stats,
    })
}

fn chrono_like_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[derive(serde::Deserialize)]
struct GhSearchResponse {
    items: Vec<GhRepoItem>,
}

#[derive(serde::Deserialize)]
struct GhRepoItem {
    full_name: String,
    html_url: String,
    clone_url: String,
    stargazers_count: u64,
    language: Option<String>,
    topics: Option<Vec<String>>,
}

fn github_search_curl(query: &str, limit: usize) -> Result<Vec<RepoEntry>> {
    let mut all = Vec::new();
    let mut page = 1usize;

    while all.len() < limit {
        let per_page = (limit - all.len()).min(100).max(1);
        let url = format!(
            "https://api.github.com/search/repositories?q={query}&sort=stars&order=desc&per_page={per_page}&page={page}"
        );

        let output = Command::new("curl")
            .args([
                "-fsSL",
                "-H",
                "Accept: application/vnd.github+json",
                "-H",
                "User-Agent: mcp-sandboxscan-corpus",
                &url,
            ])
            .output()
            .with_context(|| format!("run curl for GitHub search query `{query}` page {page}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if page > 1 && all.is_empty() {
                bail!("GitHub search failed for query `{query}`: {stderr}");
            }
            break;
        }

        let parsed: GhSearchResponse =
            serde_json::from_slice(&output.stdout).context("parse GitHub search JSON")?;

        if parsed.items.is_empty() {
            break;
        }

        let batch: Vec<RepoEntry> = parsed
            .items
            .into_iter()
            .map(|item| {
                let wasm_class = wasm_class_from_language(item.language.as_deref()).to_string();
                RepoEntry {
                    id: item.full_name,
                    url: item.html_url,
                    clone_url: item.clone_url,
                    stars: item.stargazers_count,
                    language: item.language,
                    topics: item.topics.unwrap_or_default(),
                    wasm_class,
                    resolved: false,
                    scan_status: "pending".to_string(),
                    ecosystem: String::new(),
                    dep_count: 0,
                    tier: String::new(),
                    resolve_error: None,
                    subject_toml: None,
                }
            })
            .collect();

        let fetched = batch.len();
        all.extend(batch);
        if fetched < per_page {
            break;
        }
        page += 1;
    }

    Ok(all)
}

/// Offline seed corpus for development when GitHub/curl is unavailable.
pub fn seed_corpus() -> CollectResult {
    let seeds = [
        (
            "modelcontextprotocol/servers",
            "TypeScript",
            "https://github.com/modelcontextprotocol/servers",
        ),
        (
            "microsoft/playwright-mcp",
            "TypeScript",
            "https://github.com/microsoft/playwright-mcp",
        ),
        (
            "rust-mcp-stack/rust-mcp-filesystem",
            "Rust",
            "https://github.com/rust-mcp-stack/rust-mcp-filesystem",
        ),
    ];

    let repos = seeds
        .into_iter()
        .map(|(id, lang, url)| RepoEntry {
            id: id.to_string(),
            url: url.to_string(),
            clone_url: format!("{url}.git"),
            stars: 0,
            language: Some(lang.to_string()),
            topics: vec!["mcp".to_string(), "mcp-server".to_string()],
            wasm_class: wasm_class_from_language(Some(lang)).to_string(),
            resolved: false,
            scan_status: "pending".to_string(),
            ecosystem: String::new(),
            dep_count: 0,
            tier: String::new(),
            resolve_error: None,
            subject_toml: None,
        })
        .collect();

    let (repos, filter_stats) = apply_collect_filter(repos);

    CollectResult {
        corpus: CorpusFile {
            collected_at: chrono_like_now(),
            queries: vec!["seed".to_string()],
            repos,
        },
        filter_stats,
    }
}

pub fn write_corpus_file(corpus: &CorpusFile, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(corpus)?)?;
    Ok(())
}
