use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;

use mcp_sandboxscan::attack::{load_scenario, run_external_content_experiment};

#[derive(Parser, Debug)]
#[command(name = "external-content-e2e")]
#[command(about = "Replay a controlled external-content-to-MCP-tool-output attack chain")]
struct Args {
    /// JSON scenario containing the external content and controlled server spec.
    #[arg(long)]
    scenario: Option<PathBuf>,

    /// Optionally write the structured report to a file instead of stdout.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenario_path = args.scenario.unwrap_or_else(|| {
        manifest_dir.join("fixtures/external_content_e2e/github_issue_env_leak.json")
    });
    let scenario_directory = scenario_path.parent().unwrap_or_else(|| Path::new("."));
    let scenario = load_scenario(&scenario_path)?;
    let report = run_external_content_experiment(&scenario, scenario_directory)?;
    let json = serde_json::to_string_pretty(&report).context("failed to serialize report")?;

    if let Some(output) = args.output {
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
        }
        std::fs::write(&output, format!("{json}\n"))
            .with_context(|| format!("failed to write {}", output.display()))?;
        println!(
            "Wrote external-content experiment report to {}",
            output.display()
        );
    } else {
        println!("{json}");
    }

    if !report.oracle.passed {
        bail!("external-content end-to-end oracle failed");
    }
    Ok(())
}
