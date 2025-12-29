use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, bail, Context};
use clap::Parser;

// absolute path to the sandboxscan data directory
use crate::scan::dynamic_scan::run_dynamic_scan;

// define CLI arguments
#[derive(Parser, Debug)]
#[command(name = "mcp-sandboxscan")]
#[command(about = "MCP-SandboxScan: WASM sandbox + dynamic taint-style flow detection", long_about = None)]
pub struct Args {
    /// path to target WASm module
    #[arg(long)]
    pub wasm: PathBuf,

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

    if !args.wasm.exists() {
        // return the error with reason
        bail!("WASM not found: {}", args.wasm.display());
    }
    if let Some(d) = &args.data_dir {
        if !d.exists() {
            bail!("data_dir not found: {}", d.display());
        }
    }

    let env: HashMap<String, String> = args.env.into_iter().collect();

    let report = run_dynamic_scan(
        &args.wasm,
        args.data_dir.as_ref().map(|v| v.as_path()),
        &env,
        args.max_output_size,
    ).with_context(|| format!("Failed to run dynamic scan on {}", args.wasm.display()))?;

    // JSON output
    let json = serde_json::to_string_pretty(&report)
        .context("Failed to serialize report to JSON")?;

    println!("{}", json);

    Ok(())
}