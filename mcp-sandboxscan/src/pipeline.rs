use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

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

pub fn ensure_fastmcp_examples(manifest_dir: &Path) {
    let marker = manifest_dir.join("external/fastmcp/examples/simple_echo.py");
    if marker.exists() {
        return;
    }

    let script = manifest_dir.join("scripts/fetch-fastmcp-examples.sh");
    assert!(
        script.exists(),
        "missing fetch script: {}",
        script.display()
    );

    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(manifest_dir)
        .status()
        .expect("run fetch-fastmcp-examples.sh");
    assert!(status.success(), "fetch-fastmcp-examples.sh failed");
    assert!(marker.exists(), "fastmcp examples missing after fetch");
}

pub fn ensure_python_venv(subject: &SubjectManifest, module: &str) {
    let venv_python = subject.source_dir.join(".venv/bin/python");
    if venv_python.exists() {
        let check = format!("import {module}");
        let ok = std::process::Command::new(&venv_python)
            .args(["-c", &check])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if ok {
            return;
        }
    }

    let Some(build) = &subject.build else {
        return;
    };

    let status = std::process::Command::new(&build.command)
        .args(&build.args)
        .current_dir(&subject.source_dir)
        .status()
        .expect("build python native MCP venv");
    assert!(status.success(), "python native MCP venv build failed");
}

pub fn ensure_python_fastmcp_venv(subject: &SubjectManifest) {
    ensure_python_venv(subject, "fastmcp");
}

pub fn ensure_rust_mcp_filesystem_repo(manifest_dir: &Path) {
    let server_dir = manifest_dir.join("external/rust-mcp-filesystem");
    if server_dir.join("Cargo.toml").exists() {
        return;
    }

    std::fs::create_dir_all(manifest_dir.join("external")).expect("create external dir");
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "https://github.com/rust-mcp-stack/rust-mcp-filesystem",
            &server_dir.to_string_lossy(),
        ])
        .status()
        .expect("clone rust-mcp-filesystem");
    assert!(status.success(), "git clone rust-mcp-filesystem failed");
}

fn select_adapter(subject: &SubjectManifest) -> Result<Box<dyn Adapter>> {
    if subject.capabilities.contains(&Capability::McpProtocol) {
        return Ok(Box::new(NativeMcpAdapter));
    }

    match subject.language {
        Language::Rust => Ok(Box::new(RustWasiAdapter)),
        Language::Python => Ok(Box::new(PythonWasiAdapter)),
        _ => bail!("no adapter implemented for language {:?}", subject.language),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::monitor::event::MonitorEventKind;
    use crate::taint::source::TaintSource;

    #[test]
    fn scans_rust_c2_beacon_subject() {
        let raw = std::fs::read_to_string("case_studies/rust-c2-beacon/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "BEACON_TOKEN_0123456789".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert!(
            result
                .report
                .sources
                .iter()
                .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
            "expected NetworkConnect taint source"
        );
        assert!(
            result.report.events.iter().any(|event| {
                event.kind == MonitorEventKind::NetworkConnectDenied
                    || event.kind == MonitorEventKind::NetworkConnectAttempt
            }),
            "expected network monitor events"
        );
        assert!(
            result.report.sources.iter().any(|src| {
                matches!(
                    src,
                    TaintSource::NetworkConnect { protocol, .. }
                        if protocol == "http-intent"
                )
            }),
            "expected stdout HTTP_FETCH intent source"
        );
    }

    #[test]
    fn scans_rust_mcp_c2_beacon_subject() {
        let raw = std::fs::read_to_string("case_studies/rust-mcp-c2-beacon/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "BEACON_TOKEN_0123456789".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert!(
            result
                .report
                .sources
                .iter()
                .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
            "expected NetworkConnect taint source from egress proxy"
        );
        assert!(result.report.events.iter().any(|event| {
            event.kind == MonitorEventKind::NetworkConnectDenied
                || event.kind == MonitorEventKind::NetworkConnectAttempt
        }));
        assert!(
            result.report.sinks[0].as_text().contains("beacon"),
            "expected beacon tool result text"
        );
        assert!(result.report.mcp_transcript.is_some());
    }

    #[test]
    fn scans_rust_env_leak_subject() {
        let raw = std::fs::read_to_string("case_studies/rust-env-leak/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert!(result.report.summary.has_external_to_prompt_flow);
        assert!(result.report.summary.num_flows > 0);
        assert!(result
            .report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
    }

    #[test]
    #[ignore = "requires CPython WASI runtime"]
    fn scans_python_env_leak_subject() {
        let raw = std::fs::read_to_string("case_studies/python-env-leak/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert!(result.report.summary.has_external_to_prompt_flow);
        assert!(result.report.summary.num_flows > 0);
        assert!(result
            .report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
    }

    #[test]
    fn scans_python_mcp_server_fetch_subject() {
        let raw = std::fs::read_to_string("case_studies/python-mcp-server-fetch/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
        super::ensure_python_venv(&subject, "mcp_server_fetch");

        let result = scan_subject(&subject, &HashMap::new(), None, 8192).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert!(
            result
                .report
                .sources
                .iter()
                .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
            "expected NetworkConnect taint source from fetch egress"
        );
        assert!(result.report.events.iter().any(|event| {
            event.kind == MonitorEventKind::NetworkConnectDenied
                || event.kind == MonitorEventKind::NetworkConnectAttempt
        }));
        assert!(result.report.mcp_transcript.is_some());
        assert_eq!(result.report.summary.num_sinks, 1);
        assert!(
            result.report.sinks[0].as_text().contains("c2.evil.example")
                || result.report.sinks[0].as_text().contains("Failed to fetch"),
            "expected fetch tool result mentioning blocked target"
        );
    }

    #[test]
    fn scans_python_fastmcp_upstream_echo_subject() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        super::ensure_fastmcp_examples(&manifest_dir);

        let raw =
            std::fs::read_to_string("case_studies/python-fastmcp-upstream-echo/subject.toml")
                .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
        super::ensure_python_fastmcp_venv(&subject);

        let result = scan_subject(&subject, &HashMap::new(), None, 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert_eq!(result.report.summary.num_sinks, 1);
        assert_eq!(result.report.summary.num_flows, 0);
        assert!(result.report.mcp_transcript.is_some());
        assert_eq!(result.report.mcp_transcript.as_ref().unwrap().events.len(), 5);
        assert!(result.report.sinks[0]
            .as_text()
            .contains("hello from upstream fastmcp"));
    }

    #[test]
    fn scans_python_fastmcp_echo_subject() {
        let raw = std::fs::read_to_string("case_studies/python-fastmcp-echo/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
        super::ensure_python_fastmcp_venv(&subject);

        let result = scan_subject(&subject, &HashMap::new(), None, 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert_eq!(result.report.summary.num_sinks, 1);
        assert_eq!(result.report.summary.num_flows, 0);
        assert!(result.report.mcp_transcript.is_some());
        assert_eq!(result.report.mcp_transcript.as_ref().unwrap().events.len(), 5);
        assert!(result.report.sinks[0]
            .as_text()
            .contains("hello from sandboxscan"));
    }

    #[test]
    fn scans_python_fastmcp_env_leak_subject() {
        let raw = std::fs::read_to_string("case_studies/python-fastmcp-env-leak/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
        super::ensure_python_fastmcp_venv(&subject);

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert!(result.report.summary.has_external_to_prompt_flow);
        assert!(result
            .report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
    }

    #[test]
    fn scans_python_fastmcp_c2_beacon_subject() {
        let raw = std::fs::read_to_string("case_studies/python-fastmcp-c2-beacon/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
        super::ensure_python_fastmcp_venv(&subject);

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "BEACON_TOKEN_0123456789".to_string(),
        );

        let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert!(
            result
                .report
                .sources
                .iter()
                .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
            "expected NetworkConnect taint source from egress proxy"
        );
        assert!(result.report.events.iter().any(|event| {
            event.kind == MonitorEventKind::NetworkConnectDenied
                || event.kind == MonitorEventKind::NetworkConnectAttempt
        }));
    }

    #[test]
    fn scans_rust_mcp_filesystem_subject() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        super::ensure_rust_mcp_filesystem_repo(&manifest_dir);

        let raw = std::fs::read_to_string("case_studies/rust-mcp-filesystem/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let data_dir = manifest_dir.join("data");
        std::fs::create_dir_all(&data_dir).expect("create data dir");

        let result =
            scan_subject(&subject, &HashMap::new(), Some(&data_dir), 4096).expect("scan subject");

        assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
        assert_eq!(result.report.summary.num_sinks, 1);
        assert_eq!(result.report.summary.num_flows, 0);
        assert!(result.report.mcp_transcript.is_some());
        assert_eq!(result.report.mcp_transcript.as_ref().unwrap().events.len(), 5);

        let text = result.report.sinks[0].as_text();
        let data_dir_text = data_dir.to_string_lossy().into_owned();
        assert!(text.contains("Allowed directories") || text.contains(&data_dir_text));
    }
}
