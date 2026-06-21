use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use mcp_sandboxscan::eval::suite::resolve_suite;
use mcp_sandboxscan::eval::{SuiteId, run_bench, write_bench_report};

#[derive(Parser, Debug)]
#[command(name = "bench")]
#[command(about = "Run labeled evaluation suites with support and detection metrics")]
struct Args {
    /// Suite: full (all case_studies), wasi-core (rust+python 6-pack), small-ts
    #[arg(long, default_value = "full")]
    suite: String,

    /// Output directory (default: reports/<run-id>/)
    #[arg(long)]
    out_dir: Option<PathBuf>,

    #[arg(long)]
    data_dir: Option<PathBuf>,

    #[arg(long, value_parser = parse_key_val)]
    env: Vec<(String, String)>,

    #[arg(long, default_value_t = 512_000)]
    max_output_size: usize,

    /// Compare protocol-aware vs stdout-only sink extraction per case
    #[arg(long, default_value_t = true)]
    compare: bool,

    /// Write per-case ScanReport JSON under <out-dir>/cases/
    #[arg(long, default_value_t = true)]
    write_case_reports: bool,
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| "Expected KEY=VALUE".to_string())?;
    if k.trim().is_empty() {
        return Err("Empty KEY".to_string());
    }
    Ok((k.trim().to_string(), v.to_string()))
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(d) = &args.data_dir {
        if !d.exists() {
            bail!("data_dir not found: {}", d.display());
        }
    }

    let suite = SuiteId::parse(&args.suite)?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let subject_paths = resolve_suite(&manifest_dir, suite)?;

    let env: HashMap<String, String> = args.env.into_iter().collect();

    let out_dir = args.out_dir.unwrap_or_else(|| {
        let run_id = mcp_sandboxscan::eval::run_id();
        PathBuf::from("reports").join(run_id)
    });

    let report = run_bench(
        &manifest_dir,
        suite,
        &subject_paths,
        &env,
        args.data_dir.as_deref(),
        args.max_output_size,
        args.compare,
        args.write_case_reports,
        Some(&out_dir),
    )
    .context("bench run failed")?;

    write_bench_report(&report, &out_dir).context("write bench report failed")?;

    println!(
        "Wrote bench report to {} (suite={}, scanned={}/{})",
        out_dir.display(),
        report.suite,
        report.support.scanned_cases,
        report.support.total_cases
    );

    if let Some(p) = report.detection.precision {
        println!(
            "Detection: precision={p:.3} recall={:.3} f1={:.3}",
            report.detection.recall.unwrap_or(0.0),
            report.detection.f1.unwrap_or(0.0)
        );
    }

    Ok(())
}
