use std::collections::HashMap;
use std::path::PathBuf;

use crate::pipeline::case_study::{default_env_for_subject, resolve_data_dir};
use crate::pipeline::scan_subject;
use crate::study::run_subject_matrix;
use crate::subject::SubjectManifest;
use anyhow::{Context, Result, bail};
use clap::Parser;

// absolute path to the sandboxscan data directory
use crate::scan::dynamic::run_dynamic_scan;

// define CLI arguments
#[derive(Parser, Debug)]
#[command(name = "mcp-sandboxscan")]
#[command(about = "MCP-SandboxScan: WASM sandbox + dynamic taint-style flow detection", long_about = None)]
pub struct Args {
    /// path to target WASM module
    #[arg(long, conflicts_with_all = ["subject", "study"])]
    pub wasm: Option<PathBuf>,

    /// Path to subject.toml describing a case study subject.
    #[arg(long, conflicts_with_all = ["wasm", "study"])]
    pub subject: Option<PathBuf>,

    /// Paths to subject.toml files for a multi-case study matrix.
    #[arg(long, num_args = 1.., conflicts_with_all = ["wasm", "subject"])]
    pub study: Vec<PathBuf>,

    /// Optional directory to preopen as /data inside WASI
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Provide env vars as KEY=VALUE (repeatable)
    #[arg(long, value_parser = parse_key_val)]
    pub env: Vec<(String, String)>,

    /// max bytes of stdout/stderr to keep
    #[arg(long, default_value_t = 512_000)]
    pub max_output_size: usize,
}

// result defines error from user input parsing
fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| "Expected KEY=VALUE".to_string())?;
    if k.trim().is_empty() {
        return Err("Empty KEY".to_string());
    }
    Ok((k.trim().to_string(), v.to_string()))
}

pub fn entry() -> Result<()> {
    let args = Args::parse();

    if args.wasm.is_none() && args.subject.is_none() && args.study.is_empty() {
        bail!("expected one of --wasm, --subject, or --study");
    }

    if let Some(d) = &args.data_dir {
        if !d.exists() {
            bail!("data_dir not found: {}", d.display());
        }
    }

    let env: HashMap<String, String> = args.env.into_iter().collect();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let json = if !args.study.is_empty() {
        for subject_path in &args.study {
            if !subject_path.exists() {
                bail!("study subject not found: {}", subject_path.display());
            }
        }

        let matrix = run_subject_matrix(
            &manifest_dir,
            &args.study,
            &env,
            args.data_dir.as_ref().map(|v| v.as_path()),
            args.max_output_size,
        );

        serde_json::to_string_pretty(&matrix).context("Failed to serialize study matrix to JSON")?
    } else if let Some(subject_path) = &args.subject {
        if !subject_path.exists() {
            bail!("subject not found: {}", subject_path.display());
        }

        let raw = std::fs::read_to_string(subject_path)
            .with_context(|| format!("Failed to read subject {}", subject_path.display()))?;

        let subject: SubjectManifest = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse subject {}", subject_path.display()))?;

        let scan_env = default_env_for_subject(&subject, &env);
        let effective_data_dir =
            resolve_data_dir(&manifest_dir, &subject, args.data_dir.as_ref().map(|v| v.as_path()))?;

        let result = scan_subject(
            &subject,
            &scan_env,
            effective_data_dir.as_deref(),
            args.max_output_size,
        )
        .with_context(|| format!("Failed to scan subject {}", subject.name))?;

        serde_json::to_string_pretty(&result.report)
            .context("Failed to serialize scan report to JSON")?
    } else if let Some(wasm_path) = &args.wasm {
        if !wasm_path.exists() {
            bail!("WASM not found: {}", wasm_path.display());
        }

        let report = run_dynamic_scan(
            wasm_path,
            args.data_dir.as_ref().map(|v| v.as_path()),
            &env,
            None,
            args.max_output_size,
        )
        .with_context(|| format!("Failed to run dynamic scan on {}", wasm_path.display()))?;

        serde_json::to_string_pretty(&report).context("Failed to serialize scan report to JSON")?
    } else {
        unreachable!("validated that one mode is present");
    };

    println!("{}", json);

    Ok(())
}
