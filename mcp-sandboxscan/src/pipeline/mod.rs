pub mod fixtures;

pub use fixtures::{
    ensure_fastmcp_examples, ensure_go_build, ensure_python_fastmcp_venv, ensure_python_venv,
    ensure_rust_mcp_filesystem_repo,
};

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::adapter::go_wasi::GoWasiAdapter;
use crate::adapter::native_mcp::NativeMcpAdapter;
use crate::adapter::python_wasi::PythonWasiAdapter;
use crate::adapter::rust_wasi::RustWasiAdapter;
use crate::adapter::{AdaptationStatus, Adapter, BuildArtifact};
use crate::scan::dynamic::{run_dynamic_scan, run_python_dynamic_scan};
use crate::scan::native_mcp::run_native_mcp_scan;
use crate::scan::report::ScanReport;
use crate::subject::{Capability, Language, SubjectManifest};

pub struct SubjectScanResult {
    pub report: ScanReport,
    pub adaptation_status: AdaptationStatus,
}

pub fn scan_subject(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> Result<SubjectScanResult> {
    let adapter = select_adapter(subject)?;

    let adaptation = adapter
        .adapt(subject)
        .with_context(|| format!("failed to adapt subject {}", subject.name))?;

    let report = match adaptation.status {
        AdaptationStatus::NativeOnly => {
            let Some(ref artifact) = adaptation.artifact else {
                bail!(
                    "subject {} did not produce a native MCP artifact",
                    adaptation.subject_name
                );
            };
            run_native_mcp_scan(subject, artifact, env, data_dir).with_context(|| {
                format!(
                    "failed to scan native MCP subject {}",
                    adaptation.subject_name
                )
            })?
        }
        AdaptationStatus::DirectWasm | AdaptationStatus::WasmWithShim => match adaptation.artifact {
            Some(BuildArtifact::Wasm { wasm_path }) => run_dynamic_scan(
                &wasm_path,
                data_dir,
                env,
                max_output_bytes,
            )
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
        },
        _ => bail!(
            "subject {} cannot be scanned: status={:?}, blockers={:?}",
            adaptation.subject_name,
            adaptation.status,
            adaptation.blockers
        ),
    };

    Ok(SubjectScanResult {
        report,
        adaptation_status: adaptation.status,
    })
}

fn select_adapter(subject: &SubjectManifest) -> Result<Box<dyn Adapter>> {
    if subject.capabilities.contains(&Capability::McpProtocol) {
        return Ok(Box::new(NativeMcpAdapter));
    }

    match subject.language {
        Language::Rust => Ok(Box::new(RustWasiAdapter)),
        Language::Python => Ok(Box::new(PythonWasiAdapter)),
        Language::Go => Ok(Box::new(GoWasiAdapter)),
        _ => bail!("no adapter implemented for language {:?}", subject.language),
    }
}

#[cfg(test)]
mod tests;
