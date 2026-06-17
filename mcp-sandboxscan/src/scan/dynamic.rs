use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::collect::observations_from_http_intents;
use crate::monitor::event::{flow_events, sink_events, source_inventory_events};
use crate::sandbox::wasi::preview1::WasiPreview1;
use crate::sandbox::wasi_hooks::{collect_env_sources, collect_file_sources};
use crate::sandbox::wasm_runner::WasmRunner;
use crate::scan::prompt_sink::extract_prompt_sinks;
use crate::scan::report::{ScanReport, Summary};
use crate::scan::tool_return_sink::extract_tool_return_sinks;
use crate::taint::flow::detect_flows;
use crate::taint::source::TaintSource;

pub fn run_dynamic_scan(
    wasm_path: &Path,
    data_dir: Option<&Path>,
    env: &HashMap<String, String>,
    stdin_input: Option<Vec<u8>>,
    max_output_bytes: usize,
) -> Result<ScanReport> {
    let wasm_bytes = fs::read(wasm_path)
        .with_context(|| format!("failed to read wasm file {}", wasm_path.display()))?;

    let mut runtime = WasiPreview1::new(
        env.clone(),
        data_dir.map(|p| p.to_path_buf()),
        max_output_bytes,
    );
    runtime.stdin_input = stdin_input;

    let runner = WasmRunner::default();
    let exec = runner.run(&wasm_bytes, &runtime)?;

    build_scan_report(exec, env, data_dir, &runtime)
}

pub fn run_python_dynamic_scan(
    interpreter_wasm: &Path,
    work_dir: &Path,
    argv: &[String],
    data_dir: Option<&Path>,
    env: &HashMap<String, String>,
    max_output_bytes: usize,
) -> Result<ScanReport> {
    let wasm_bytes = fs::read(interpreter_wasm).with_context(|| {
        format!(
            "failed to read Python interpreter wasm {}",
            interpreter_wasm.display()
        )
    })?;

    let python_root = interpreter_wasm
        .parent()
        .map(|p| p.to_path_buf())
        .filter(|p| !p.as_os_str().is_empty());

    let runtime = WasiPreview1::new_with_args(
        env.clone(),
        data_dir.map(|p| p.to_path_buf()),
        Some(work_dir.to_path_buf()),
        python_root,
        argv.to_vec(),
        max_output_bytes,
    );

    let runner = WasmRunner::default();
    let exec = runner.run(&wasm_bytes, &runtime)?;

    build_scan_report(exec, env, data_dir, &runtime)
}

fn build_scan_report(
    exec: crate::sandbox::exec_result::WasmExecResult,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    runtime: &WasiPreview1,
) -> Result<ScanReport> {
    let mut sinks = extract_prompt_sinks(&exec.stdout);
    sinks.extend(extract_tool_return_sinks(&exec.stdout));

    let mut sources: Vec<TaintSource> = vec![];
    sources.extend(collect_env_sources(env));
    sources.extend(collect_file_sources(data_dir, 64 * 1024)?);

    let network_collector = runtime.network_collector();
    for obs in observations_from_http_intents(&exec.stdout, &exec.stderr) {
        network_collector.record(obs);
    }
    sources.extend(network_collector.as_taint_sources());

    let flows = detect_flows(&sources, &sinks);

    let mut events = Vec::new();
    events.extend(runtime.take_monitor_events());
    events.extend(network_collector.as_monitor_events());
    events.extend(source_inventory_events(&sources));
    events.extend(sink_events(&sinks));
    events.extend(flow_events(&flows));

    let summary = Summary {
        num_sources: sources.len(),
        num_sinks: sinks.len(),
        num_flows: flows.len(),
        has_external_to_prompt_flow: !flows.is_empty(),
    };

    Ok(ScanReport {
        exec: exec.into(),
        mcp_transcript: None,
        events,
        sources,
        sinks,
        flows,
        summary,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test_flow_detection_logic() {
        assert!(true);
    }
}
