use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use mcp_sandboxscan::corpus::{
    CollectOptions, CorpusFile, ResolveOptions, ScanOptions, assign_tiers,
    build_semantic_corpus_report, build_semantic_cross_validation_report, collect_github,
    enrich_corpus_report_from_path, prune_corpus, resolve_corpus, run_corpus_scan, seed_corpus,
    verify_suspicious_cases, write_corpus_file, write_corpus_report, write_semantic_corpus_report,
    write_semantic_cross_validation_report,
};
use mcp_sandboxscan::mcp::explore::ExplorationConfig;
use mcp_sandboxscan::pipeline::ScanLimits;

#[derive(Parser)]
#[command(
    name = "corpus",
    about = "Collect, resolve, and scan real MCP server repos"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Collect repos into corpus/repos.json (curl + GitHub API; no `gh` required)
    Collect {
        #[arg(long, default_value = "corpus/repos.json")]
        out: PathBuf,
        /// Max repos fetched per GitHub search query (paginated, up to 100/page)
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Stop once the corpus has at least this many repos after filtering
        #[arg(long, default_value_t = 100)]
        target: usize,
        /// Merge with existing repos.json (keeps resolve/scan status for known repos)
        #[arg(long)]
        merge: bool,
        /// Only keep dedicated MCP server repos (recommended)
        #[arg(long, default_value_t = true)]
        strict: bool,
        /// Disable strict dedicated-server filtering
        #[arg(long)]
        no_strict: bool,
        /// Offline seed list (no network)
        #[arg(long, conflicts_with_all = ["limit", "target", "merge", "strict", "no_strict"])]
        seed: bool,
    },
    /// Remove failed/unresolved repos from corpus/repos.json
    Prune {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
        /// Drop repos with resolve_error
        #[arg(long, default_value_t = true)]
        errors: bool,
        /// Drop repos that were never resolved (pending)
        #[arg(long)]
        pending: bool,
    },
    /// git clone + generate subject.toml manifests
    Resolve {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
        #[arg(long, default_value_t = 100)]
        max: usize,
        #[arg(long)]
        force: bool,
    },
    /// Dynamic scan of resolved manifests
    Scan {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
        #[arg(long)]
        out_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 512_000)]
        max_output_size: usize,
        /// Max seconds for per-repo build (pip/npm/go install)
        #[arg(long, default_value_t = 300)]
        build_timeout_secs: u64,
        /// Max seconds waiting for MCP initialize + tools/call
        #[arg(long, default_value_t = 60)]
        mcp_timeout_secs: u64,
        /// Dynamic exploration depth: shallow | targeted
        #[arg(long, default_value = "shallow")]
        explore_depth: String,
        /// Maximum tools to call in targeted exploration mode
        #[arg(long, default_value_t = 8)]
        max_tool_calls: usize,
    },
    /// Assign tier1/tier2 labels to repos.json (no network)
    Tier {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
    },
    /// List suspicious cases for manual review
    Verify {
        #[arg(long)]
        cases_dir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Enrich an existing corpus_summary.json with tool semantic profiles
    Enrich {
        #[arg(long)]
        summary: PathBuf,
    },
    /// Semantic-only extraction over the full corpus without rerunning tool calls
    Semantic {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
        /// Existing dynamic scan summary to reuse as high-confidence tools/list evidence
        #[arg(long)]
        scan_summary: Option<PathBuf>,
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
}

fn load_corpus(path: &PathBuf) -> Result<CorpusFile> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if raw.trim().is_empty() {
        bail!(
            "corpus file is empty: {} (run `corpus collect` or `corpus collect --seed`)",
            path.display()
        );
    }
    Ok(serde_json::from_str(&raw)?)
}

fn save_corpus(path: &PathBuf, corpus: &CorpusFile) -> Result<()> {
    write_corpus_file(corpus, path)
}

fn corpus_dir_from(path: &PathBuf) -> PathBuf {
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("corpus"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    match cli.cmd {
        Cmd::Collect {
            out,
            limit,
            target,
            merge,
            strict,
            no_strict,
            seed,
        } => {
            let result = if seed {
                seed_corpus()
            } else {
                collect_github(&CollectOptions {
                    limit_per_query: limit,
                    target_repos: Some(target),
                    strict: strict && !no_strict,
                    merge_from: if merge || out.exists() {
                        Some(out.clone())
                    } else {
                        None
                    },
                })?
            };
            write_corpus_file(&result.corpus, &out)?;
            println!(
                "wrote {} repos -> {} (target {}, filtered {} of {} raw)",
                result.corpus.repos.len(),
                out.display(),
                target,
                result.filter_stats.rejected,
                result.filter_stats.raw
            );
            if !result.filter_stats.reasons.is_empty() {
                let mut reasons: Vec<_> = result.filter_stats.reasons.iter().collect();
                reasons.sort_by(|a, b| b.1.cmp(a.1));
                println!("filter rejections:");
                for (reason, count) in reasons.iter().take(10) {
                    println!("  {count:4}  {reason}");
                }
            }
        }
        Cmd::Resolve { corpus, max, force } => {
            let mut file = load_corpus(&corpus)?;
            resolve_corpus(
                &mut file,
                &ResolveOptions {
                    corpus_dir: corpus_dir_from(&corpus),
                    max_repos: Some(max),
                    skip_existing: !force,
                },
            )?;
            save_corpus(&corpus, &file)?;
            let resolved = file.repos.iter().filter(|r| r.resolved).count();
            let skipped = file
                .repos
                .iter()
                .filter(|r| r.resolve_error.is_some())
                .count();
            println!("resolved {resolved}, skipped/failed {skipped}");
        }
        Cmd::Scan {
            corpus,
            out_dir,
            max_output_size,
            build_timeout_secs,
            mcp_timeout_secs,
            explore_depth,
            max_tool_calls,
        } => {
            let mut file = load_corpus(&corpus)?;
            let out = out_dir.unwrap_or_else(|| {
                PathBuf::from("reports").join(format!("corpus-{}", mcp_sandboxscan::eval::run_id()))
            });
            let report = run_corpus_scan(
                &mut file,
                &ScanOptions {
                    manifest_dir: manifest_dir.clone(),
                    out_dir: out.clone(),
                    env: HashMap::new(),
                    max_output_bytes: max_output_size,
                    limits: ScanLimits {
                        build_timeout: Some(std::time::Duration::from_secs(build_timeout_secs)),
                        mcp_timeout: Some(std::time::Duration::from_secs(mcp_timeout_secs)),
                        exploration: exploration_from_args(&explore_depth, max_tool_calls)?,
                    },
                },
            )?;
            write_corpus_report(&report, &out)?;
            save_corpus(&corpus, &file)?;
            println!("corpus scan -> {}", out.display());
            println!(
                "support={:.1}% suspicious={:.1}%",
                report.scan_success_rate * 100.0,
                report.suspicious_rate * 100.0
            );
            println!(
                "tier1 support={:.1}% ({}/{})",
                report.tier1.scan_success_rate * 100.0,
                report.tier1.scanned,
                report.tier1.resolved
            );
        }
        Cmd::Tier { corpus } => {
            let mut file = load_corpus(&corpus)?;
            assign_tiers(&mut file.repos);
            save_corpus(&corpus, &file)?;
            let tier1 = file.repos.iter().filter(|r| r.tier == "tier1").count();
            println!("labeled {} tier1 / {} total repos", tier1, file.repos.len());
        }
        Cmd::Prune {
            corpus,
            errors,
            pending,
        } => {
            let mut file = load_corpus(&corpus)?;
            let stats = prune_corpus(&mut file, errors, pending);
            save_corpus(&corpus, &file)?;
            println!(
                "pruned {} -> {} repos (removed {} errors, {} pending)",
                stats.before, stats.after, stats.removed_errors, stats.removed_unresolved
            );
        }
        Cmd::Verify { cases_dir, out } => {
            let rows = verify_suspicious_cases(&cases_dir)?;
            let json = serde_json::to_string_pretty(&rows)?;
            if let Some(path) = out {
                fs::write(&path, &json)?;
                println!(
                    "wrote {} suspicious cases -> {}",
                    rows.len(),
                    path.display()
                );
            } else {
                println!("{json}");
            }
        }
        Cmd::Enrich { summary } => {
            let report = enrich_corpus_report_from_path(&summary)?;
            let out_dir = summary
                .parent()
                .with_context(|| format!("missing parent directory for {}", summary.display()))?;
            write_corpus_report(&report, out_dir)?;
            println!(
                "enriched {} repos with tool metadata ({} tools) -> {}",
                report.semantic.all.repos_with_metadata,
                report.semantic.all.total_tools,
                summary.display()
            );
        }
        Cmd::Semantic {
            corpus,
            scan_summary,
            out_dir,
        } => {
            let file = load_corpus(&corpus)?;
            let scan_report = match scan_summary.as_ref() {
                Some(path) => Some(enrich_corpus_report_from_path(path)?),
                None => None,
            };
            let report = build_semantic_corpus_report(&file, scan_report.as_ref(), &manifest_dir);
            let out = out_dir
                .or_else(|| {
                    scan_summary
                        .as_ref()
                        .and_then(|p| p.parent().map(PathBuf::from))
                })
                .unwrap_or_else(|| PathBuf::from("reports").join("semantic-corpus"));
            write_semantic_corpus_report(&report, &out)?;
            if let Some(scan_report) = scan_report.as_ref() {
                let cross = build_semantic_cross_validation_report(scan_report, &out)?;
                write_semantic_cross_validation_report(&cross, &out)?;
            }
            println!(
                "semantic corpus -> {} (tool metadata repos {}/{}, tools {})",
                out.display(),
                report.repos_with_tool_metadata,
                report.total_repos,
                report.total_tools
            );
        }
    }
    Ok(())
}

fn exploration_from_args(depth: &str, max_tool_calls: usize) -> Result<ExplorationConfig> {
    match depth {
        "shallow" => Ok(ExplorationConfig::disabled()),
        "targeted" => Ok(ExplorationConfig {
            enabled: true,
            max_tools: max_tool_calls,
            env_canary: String::new(),
            input_canary: String::new(),
            file_canary_path: None,
        }),
        other => bail!("unknown --explore-depth `{other}` (expected shallow or targeted)"),
    }
}
