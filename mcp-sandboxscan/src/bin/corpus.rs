use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use mcp_sandboxscan::corpus::{
    collect_github, resolve_corpus, run_corpus_scan, seed_corpus, verify_suspicious_cases,
    write_corpus_file, write_corpus_report, CollectOptions, CorpusFile, ResolveOptions,
    ScanOptions,
};
use mcp_sandboxscan::pipeline::ScanLimits;

#[derive(Parser)]
#[command(name = "corpus", about = "Collect, resolve, and scan real MCP server repos")]
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
        #[arg(long, default_value_t = 30)]
        limit: usize,
        /// Offline seed list (no network)
        #[arg(long, conflicts_with = "limit")]
        seed: bool,
    },
    /// git clone + generate subject.toml manifests
    Resolve {
        #[arg(long, default_value = "corpus/repos.json")]
        corpus: PathBuf,
        #[arg(long, default_value_t = 20)]
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
    },
    /// List suspicious cases for manual review
    Verify {
        #[arg(long)]
        cases_dir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn load_corpus(path: &PathBuf) -> Result<CorpusFile> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if raw.trim().is_empty() {
        bail!("corpus file is empty: {} (run `corpus collect` or `corpus collect --seed`)", path.display());
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
        Cmd::Collect { out, limit, seed } => {
            let result = if seed {
                seed_corpus()
            } else {
                collect_github(&CollectOptions {
                    limit_per_query: limit,
                })?
            };
            write_corpus_file(&result.corpus, &out)?;
            println!(
                "wrote {} repos -> {} (filtered {} of {} raw)",
                result.corpus.repos.len(),
                out.display(),
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
        Cmd::Resolve {
            corpus,
            max,
            force,
        } => {
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
            let skipped = file.repos.iter().filter(|r| r.resolve_error.is_some()).count();
            println!("resolved {resolved}, skipped/failed {skipped}");
        }
        Cmd::Scan {
            corpus,
            out_dir,
            max_output_size,
            build_timeout_secs,
            mcp_timeout_secs,
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
        }
        Cmd::Verify { cases_dir, out } => {
            let rows = verify_suspicious_cases(&cases_dir)?;
            let json = serde_json::to_string_pretty(&rows)?;
            if let Some(path) = out {
                fs::write(&path, &json)?;
                println!("wrote {} suspicious cases -> {}", rows.len(), path.display());
            } else {
                println!("{json}");
            }
        }
    }
    Ok(())
}
