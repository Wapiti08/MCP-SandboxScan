use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;

use mcp_sandboxscan::taint::flow_config::{
    FlowConfig, apply_transform, detect_secret_in_sink,
};

#[derive(Parser, Debug)]
#[command(name = "evasion")]
#[command(about = "Document evasion limits of substring-based flow detection")]
struct Args {
    #[arg(long, default_value = "SEKRET_0123456789abcdef0123456789abcdef")]
    secret: String,

    #[arg(long, default_value = "reports/evasion.json")]
    output: PathBuf,
}

#[derive(Debug, Serialize)]
struct EvasionRow {
    transform: &'static str,
    raw_only_detected: bool,
    default_detected: bool,
    raw_only_strategies: Vec<String>,
    default_strategies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EvasionReport {
    secret: String,
    raw_only_config: FlowConfig,
    default_config: FlowConfig,
    rows: Vec<EvasionRow>,
}

const TRANSFORMS: &[&str] = &[
    "plain",
    "prefix+suffix",
    "rot13",
    "hex",
    "base64",
    "chunked",
    "suffix-only",
];

fn main() -> Result<()> {
    let args = Args::parse();
    let raw_only_config = FlowConfig::raw_only();
    let default_config = FlowConfig::default_matcher();

    let rows = TRANSFORMS
        .iter()
        .map(|transform| {
            let sink = apply_transform(&args.secret, transform);
            let (raw_only_detected, raw_only_strategies) =
                detect_secret_in_sink(&args.secret, &sink, &raw_only_config);
            let (default_detected, default_strategies) =
                detect_secret_in_sink(&args.secret, &sink, &default_config);
            EvasionRow {
                transform,
                raw_only_detected,
                default_detected,
                raw_only_strategies,
                default_strategies,
            }
        })
        .collect();

    let report = EvasionReport {
        secret: args.secret,
        raw_only_config,
        default_config,
        rows,
    };

    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }

    let json = serde_json::to_string_pretty(&report).context("failed to serialize report")?;
    std::fs::write(&args.output, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", args.output.display()))?;

    println!("{json}");
    Ok(())
}
