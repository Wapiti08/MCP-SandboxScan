use std::process::Command;

use anyhow::{Context, Result};

use crate::adapter::{AdaptationReport, AdaptationStatus, Adapter, BuildArtifact};
use crate::subject::{Capability, SubjectManifest};

pub struct NativeMcpAdapter;

impl Adapter for NativeMcpAdapter {
    fn name(&self) -> &'static str {
        "native-mcp"
    }

    fn adapt(&self, subject: &SubjectManifest) -> Result<AdaptationReport> {
        if !subject.capabilities.contains(&Capability::McpProtocol) {
            return Ok(failed(
                subject,
                AdaptationStatus::Unsupported,
                "missing mcp-protocol capability",
            ));
        }

        let Some(run) = &subject.run else {
            return Ok(failed(
                subject,
                AdaptationStatus::Failed,
                "missing [run] spec",
            ));
        };

        if subject.mcp.is_none() {
            return Ok(failed(
                subject,
                AdaptationStatus::Failed,
                "missing [mcp] tool spec",
            ));
        }

        if let Some(build) = &subject.build {
            let status = Command::new(&build.command)
                .args(&build.args)
                .current_dir(&subject.source_dir)
                .status()
                .with_context(|| {
                    format!(
                        "failed to run build command `{}` in {}",
                        build.command,
                        subject.source_dir.display()
                    )
                })?;

            if !status.success() {
                return Ok(failed(
                    subject,
                    AdaptationStatus::Failed,
                    &format!("build command exited with status {status}"),
                ));
            }
        }

        Ok(AdaptationReport {
            subject_name: subject.name.clone(),
            language: subject.language.clone(),
            status: AdaptationStatus::NativeOnly,
            artifact: Some(BuildArtifact::NativeCommand {
                command: run.command.clone(),
                args: run.args.clone(),
            }),
            notes: vec!["native stdio MCP server".to_string()],
            blockers: vec![],
        })
    }
}

fn failed(subject: &SubjectManifest, status: AdaptationStatus, reason: &str) -> AdaptationReport {
    AdaptationReport {
        subject_name: subject.name.clone(),
        language: subject.language.clone(),
        status,
        artifact: None,
        notes: vec![],
        blockers: vec![reason.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapts_python_mcp_server_fetch_manifest() {
        let raw = std::fs::read_to_string("case_studies/python-mcp-server-fetch/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let report = NativeMcpAdapter.adapt(&subject).expect("adapt subject");

        assert!(matches!(report.status, AdaptationStatus::NativeOnly));
        let Some(BuildArtifact::NativeCommand { command, args }) = report.artifact else {
            panic!("expected NativeCommand artifact");
        };
        assert_eq!(command, ".venv/bin/python");
        assert_eq!(args, vec!["-u", "-m", "mcp_server_fetch"]);
    }

    #[test]
    fn adapts_python_fastmcp_echo_manifest() {
        let raw = std::fs::read_to_string("case_studies/python-fastmcp-echo/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let report = NativeMcpAdapter.adapt(&subject).expect("adapt subject");

        assert!(matches!(report.status, AdaptationStatus::NativeOnly));
        let Some(BuildArtifact::NativeCommand { command, args }) = report.artifact else {
            panic!("expected NativeCommand artifact");
        };
        assert_eq!(command, ".venv/bin/python");
        assert_eq!(args, vec!["-u", "server.py"]);
    }

    #[test]
    fn adapts_rust_mcp_c2_beacon_manifest() {
        let raw = std::fs::read_to_string("case_studies/rust-mcp-c2-beacon/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let report = NativeMcpAdapter.adapt(&subject).expect("adapt subject");

        assert!(matches!(report.status, AdaptationStatus::NativeOnly));
        let Some(BuildArtifact::NativeCommand { command, args }) = report.artifact else {
            panic!("expected NativeCommand artifact");
        };
        assert_eq!(command, "target/release/rust-mcp-c2-beacon");
        assert!(args.is_empty());
    }

    #[test]
    fn adapts_rust_mcp_filesystem_manifest() {
        let raw = std::fs::read_to_string("case_studies/rust-mcp-filesystem/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let report = NativeMcpAdapter.adapt(&subject).expect("adapt subject");

        assert!(matches!(report.status, AdaptationStatus::NativeOnly));
        let Some(BuildArtifact::NativeCommand { command, args }) = report.artifact else {
            panic!("expected NativeCommand artifact");
        };
        assert_eq!(command, "target/release/rust-mcp-filesystem");
        assert!(args.is_empty());
    }
}
