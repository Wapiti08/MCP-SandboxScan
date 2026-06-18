pub mod case_study;
pub mod fixtures;
pub mod limits;

#[allow(unused_imports)]
pub use fixtures::{
    ensure_fastmcp_examples, ensure_go_build, ensure_go_sdk_examples, ensure_npm_install,
    ensure_python_fastmcp_venv, ensure_python_venv, ensure_rust_mcp_filesystem_repo,
    ensure_typescript_sdk_examples,
};

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::adapter::go_wasi::GoWasiAdapter;
use crate::adapter::native_mcp::NativeMcpAdapter;
use crate::adapter::python_wasi::PythonWasiAdapter;
use crate::adapter::rust_wasi::RustWasiAdapter;
use crate::adapter::typescript_wasi::TypeScriptWasiAdapter;
use crate::adapter::{AdaptationStatus, Adapter, BuildArtifact};
use crate::scan::dynamic::{run_dynamic_scan, run_python_dynamic_scan};
use crate::scan::native_mcp::run_native_mcp_scan;
use crate::scan::report::ScanReport;
use crate::subject::{Capability, Language, SubjectManifest};

pub use limits::ScanLimits;

fn build_ts_wasi_stdin_payload(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
) -> Option<Vec<u8>> {
    if subject.language != Language::TypeScript && subject.language != Language::JavaScript {
        return None;
    }

    let mut files = serde_json::Map::new();
    if let Some(dir) = data_dir {
        let secret = dir.join("secret.txt");
        if secret.exists() {
            if let Ok(content) = std::fs::read_to_string(&secret) {
                files.insert(
                    "/data/secret.txt".to_string(),
                    serde_json::Value::String(content),
                );
            }
        }
    }

    let payload = serde_json::json!({
        "env": env,
        "files": files,
    });
    Some(payload.to_string().into_bytes())
}

pub struct SubjectScanResult {
    pub report: ScanReport,
    pub adaptation_status: AdaptationStatus,
    pub timing: SubjectScanTiming,
}

#[derive(Debug, Clone, Default)]
pub struct SubjectScanTiming {
    pub build_ms: u128,
    pub scan_ms: u128,
    pub total_ms: u128,
}

pub fn scan_subject(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> Result<SubjectScanResult> {
    scan_subject_with_limits(subject, env, data_dir, max_output_bytes, ScanLimits::default())
}

pub fn scan_subject_with_limits(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
    limits: ScanLimits,
) -> Result<SubjectScanResult> {
    let total_start = std::time::Instant::now();
    let adapter = select_adapter(subject, &limits)?;

    let build_start = std::time::Instant::now();
    let adaptation = adapter
        .adapt(subject)
        .with_context(|| format!("failed to adapt subject {}", subject.name))?;
    let build_ms = build_start.elapsed().as_millis();

    let scan_start = std::time::Instant::now();
    let report = match adaptation.status {
        AdaptationStatus::NativeOnly => {
            let Some(ref artifact) = adaptation.artifact else {
                bail!(
                    "subject {} did not produce a native MCP artifact",
                    adaptation.subject_name
                );
            };
            run_native_mcp_scan(subject, artifact, env, data_dir, limits.mcp_timeout).with_context(|| {
                format!(
                    "failed to scan native MCP subject {}",
                    adaptation.subject_name
                )
            })?
        }
        AdaptationStatus::DirectWasm | AdaptationStatus::WasmWithShim => {
            match adaptation.artifact {
                Some(BuildArtifact::Wasm { wasm_path }) => {
                    let stdin_input = build_ts_wasi_stdin_payload(subject, env, data_dir);
                    run_dynamic_scan(&wasm_path, data_dir, env, stdin_input, max_output_bytes)
                }
                .with_context(|| format!("failed to scan wasm artifact {}", wasm_path.display()))?,
                Some(BuildArtifact::PythonWasm {
                    interpreter_wasm,
                    work_dir,
                    argv,
                }) => run_python_dynamic_scan(
                    &interpreter_wasm,
                    &work_dir,
                    &argv,
                    data_dir,
                    env,
                    max_output_bytes,
                )
                .with_context(|| {
                    format!(
                        "failed to scan Python wasm artifact {}",
                        interpreter_wasm.display()
                    )
                })?,
                _ => bail!(
                    "subject {} did not produce a wasm artifact",
                    adaptation.subject_name
                ),
            }
        }
        _ => bail!(
            "subject {} cannot be scanned: status={:?}, blockers={:?}",
            adaptation.subject_name,
            adaptation.status,
            adaptation.blockers
        ),
    };
    let scan_ms = scan_start.elapsed().as_millis();

    Ok(SubjectScanResult {
        report,
        adaptation_status: adaptation.status,
        timing: SubjectScanTiming {
            build_ms,
            scan_ms,
            total_ms: total_start.elapsed().as_millis(),
        },
    })
}

fn select_adapter(subject: &SubjectManifest, limits: &ScanLimits) -> Result<Box<dyn Adapter>> {
    if subject.capabilities.contains(&Capability::McpProtocol) {
        return Ok(Box::new(NativeMcpAdapter {
            build_timeout: limits.build_timeout,
        }));
    }

    match subject.language {
        Language::Rust => Ok(Box::new(RustWasiAdapter)),
        Language::Python => Ok(Box::new(PythonWasiAdapter)),
        Language::Go => Ok(Box::new(GoWasiAdapter)),
        Language::TypeScript | Language::JavaScript => Ok(Box::new(TypeScriptWasiAdapter)),
        _ => bail!("no adapter implemented for language {:?}", subject.language),
    }
}

#[cfg(test)]
mod tests;
