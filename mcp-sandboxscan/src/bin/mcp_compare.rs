use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

use mcp_sandboxscan::scan::compare::{compare_subject, compare_subjects, discover_case_studies};

#[derive(Parser, Debug)]
#[command(name = "mcp_compare")]
#[command(about = "Compare stdout-only vs protocol-aware sink extraction for case studies")]
struct Args {
    #[arg(long, conflicts_with_all = ["study", "all"])]
    subject: Option<PathBuf>,

    #[arg(long, num_args = 1.., conflicts_with_all = ["subject", "all"])]
    study: Vec<PathBuf>,

    /// Run comparison across every case_studies/*/subject.toml
    #[arg(long, conflicts_with_all = ["subject", "study"])]
    all: bool,

    #[arg(long)]
    data_dir: Option<PathBuf>,

    #[arg(long, value_parser = parse_key_val)]
    env: Vec<(String, String)>,

    #[arg(long, default_value_t = 512_000)]
    max_output_size: usize,
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

    if args.subject.is_none() && args.study.is_empty() && !args.all {
        bail!("expected one of --subject, --study, or --all");
    }

    if let Some(d) = &args.data_dir {
        if !d.exists() {
            bail!("data_dir not found: {}", d.display());
        }
    }

    let env: HashMap<String, String> = args.env.into_iter().collect();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let json = if args.all {
        let paths = discover_case_studies(&manifest_dir)?;
        let matrix = compare_subjects(
            &manifest_dir,
            &paths,
            &env,
            args.data_dir.as_deref(),
            args.max_output_size,
        );
        serde_json::to_string_pretty(&matrix)?
    } else if !args.study.is_empty() {
        for subject_path in &args.study {
            if !subject_path.exists() {
                bail!("study subject not found: {}", subject_path.display());
            }
        }
        let matrix = compare_subjects(
            &manifest_dir,
            &args.study,
            &env,
            args.data_dir.as_deref(),
            args.max_output_size,
        );
        serde_json::to_string_pretty(&matrix)?
    } else if let Some(subject_path) = &args.subject {
        if !subject_path.exists() {
            bail!("subject not found: {}", subject_path.display());
        }
        let row = compare_subject(
            &manifest_dir,
            subject_path,
            &env,
            args.data_dir.as_deref(),
            args.max_output_size,
        );
        if let Some(error) = &row.error {
            bail!("{error}");
        }
        serde_json::to_string_pretty(&row)?
    } else {
        unreachable!();
    };

    println!("{json}");
    Ok(())
}
