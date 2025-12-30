use std::{collections::HashMap, path::{Path, PathBuf}};

use anyhow::Result;

use mcp_sandboxscan::scan::dynamic_scan::run_dynamic_scan;

fn main() -> Result<()> {
    let wasm = std::env::args().nth(1)
    .map(PathBuf::from)
    .unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/evil_prompt_tool/tool.wasm")
    });

    if !wasm.exists() {
        anyhow::bail!("wasm not found: {}", wasm.display());
    }
    let data_dir: Option<PathBuf> = None;

    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "shh-secret".to_string());
    env.insert("USER_INPUT".to_string(), "hello".to_string());
    env.insert("FILE_TO_READ".to_string(), "secret.txt".to_string());

    let report = run_dynamic_scan(
        &wasm,
        data_dir.as_deref(),
        &env,
        4096,
    )?;

    println!("== Exec ==");
    println!("exit_code: {}", report.exec.exit_code);
    println!("duration_ms: {}", report.exec.duration_ms);
    println!("stdout:\n{}", report.exec.stdout);
    println!("stderr:\n{}", report.exec.stderr);

    println!("\n== Summary ==");
    println!("sources: {}", report.summary.num_sources);
    println!("sinks: {}", report.summary.num_sinks);
    println!("flows: {}", report.summary.num_flows);
    println!(
        "has_external_to_prompt_flow: {}",
        report.summary.has_external_to_prompt_flow
    );

    println!("\n== Flows (top) ==");
    for (i, f) in report.flows.iter().take(10).enumerate() {
        println!("{}. {:?}", i + 1, f);
    }

    Ok(())
}
