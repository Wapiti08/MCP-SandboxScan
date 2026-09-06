use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use mcp_sandboxscan::targeted_egress::{TargetedRunReport, load_plan, prepare_case, run_case};

#[derive(Parser)]
#[command(name = "targeted-egress")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Prepare {
        #[arg(long)]
        plan: PathBuf,

        #[arg(long, default_value = "/workspace")]
        workspace: PathBuf,

        #[arg(long, default_value = "/subjects")]
        subjects_root: PathBuf,
    },

    Run {
        #[arg(long)]
        plan: PathBuf,

        #[arg(long, default_value = "/workspace")]
        workspace: PathBuf,

        #[arg(long, default_value = "/subjects")]
        subjects_root: PathBuf,

        #[arg(long, default_value = "/results")]
        results_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Prepare {
            plan,
            workspace,
            subjects_root,
        } => {
            let plan = load_plan(&plan)?;

            for case in &plan.cases {
                println!("preparing {}", case.repo_id);
                prepare_case(case, &workspace, &subjects_root)?;
            }
        }

        Command::Run {
            plan,
            workspace,
            subjects_root,
            results_dir,
        } => {
            let plan = load_plan(&plan)?;
            let mut reports = Vec::new();

            for case in &plan.cases {
                println!("running {}", case.repo_id);
                reports.push(run_case(case, &workspace, &subjects_root, &results_dir)?);
            }

            let report = TargetedRunReport { cases: reports };
            let json = serde_json::to_string_pretty(&report)?;

            fs::create_dir_all(&results_dir)?;
            let output = results_dir.join("targeted-egress-summary.json");
            fs::write(&output, format!("{json}\n"))
                .with_context(|| format!("write {}", output.display()))?;

            println!("report: {}", output.display());
        }
    }

    Ok(())
}
