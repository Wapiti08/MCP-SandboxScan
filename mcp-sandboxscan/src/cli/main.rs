use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, bail, Context};
use clap::{Parser, Subcommand};

// absolute path to the sandboxscan data directory
use crate::scan::dynamic_scan::run_dynamic_scan;

// define CLI arguments
#[derive(Parser, Debug)]
#[command(name = "mcp-sandboxscan")]
#[command(about = "MCP-SandboxScan: WASM sandbox + dynamic taint-style flow detection", long_about = None)]
pub struct Args {
    /// path to target WASm module
    #[arg(long)]

}