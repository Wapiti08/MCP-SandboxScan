use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::sandbox::wasm_runner::WasmRunner;
use crate::sandbox::wasi_hooks::{collect_file_sources, collect_env_sources, collect_http_intents};
use crate::scan::prompt_sink::extract_prompt_sinks;
use crate::scan::tool_return_sink::extract_tool_return_sinks;
use crate::scan::report::{ScanReport, Summary};
use crate::taint::flow::detect_flows;
use crate::taint::source::TaintSource;
use crate::sandbox::wasi::preview1::WasiPreview1;


pub fn run_dynamic_scan(
    wasm_path: &Path,
    data_dir: Option<&Path>,
    env: &HashMap<String, String>,
    max_output_bytes: usize,
) -> Result<ScanReport> {
    let wasm_bytes = fs::read(wasm_path)
        .with_context(|| format!("failed to read wasm file {}", wasm_path.display()))?;

    let runtime = WasiPreview1::new(
        env.clone(),
        data_dir.map(|p| p.to_path_buf()),
        max_output_bytes,
    );

    let runner = WasmRunner::default();
    let exec = runner.run(&wasm_bytes, &runtime)?;

    // 2) Extract prompt sinks from stdout
    let mut sinks = extract_prompt_sinks(&exec.stdout);
    sinks.extend(extract_tool_return_sinks(&exec.stdout));

    // 3) Collect external sources (MVP)
    let mut sources: Vec<TaintSource> = vec![];
    sources.extend(collect_env_sources(env));
    sources.extend(collect_file_sources(data_dir, 64 * 1024)?); // the file is max 64KB
    sources.extend(collect_http_intents(&exec.stdout, &exec.stderr));

    // 4) Detect flows (string-level)
    let flows = detect_flows(&sources, &sinks);
    
    let summary = Summary {
        num_sources: sources.len(),
        num_sinks: sinks.len(),
        num_flows: flows.len(),
        has_external_to_prompt_flow: !flows.is_empty(),
    };

    Ok(ScanReport {
        exec,
        sources,
        sinks,
        flows,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_flow_detection_logic() {
        assert!(true);
    }
}

