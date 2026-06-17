use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::adapter::{AdaptationReport, AdaptationStatus, Adapter, BuildArtifact};
use crate::subject::{Capability, Language, SubjectManifest};

pub struct GoWasiAdapter;

impl Adapter for GoWasiAdapter {
    fn name(&self) -> &'static str {
        "go-wasi"
    }

    fn adapt(&self, subject: &SubjectManifest) -> Result<AdaptationReport> {
        if subject.capabilities.contains(&Capability::McpProtocol) {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Unsupported,
                artifact: None,
                notes: vec![],
                blockers: vec![
                    "GoWasiAdapter does not support mcp-protocol subjects; use NativeMcpAdapter"
                        .to_string(),
                ],
            });
        }

        if subject.language != Language::Go {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Unsupported,
                artifact: None,
                notes: vec![],
                blockers: vec!["GoWasiAdapter only supports Go subjects".to_string()],
            });
        }

        let Some(build) = &subject.build else {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Failed,
                artifact: None,
                notes: vec![],
                blockers: vec!["missing build spec".to_string()],
            });
        };

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
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Failed,
                artifact: None,
                notes: vec![],
                blockers: vec![format!("build command exited with status {status}")],
            });
        }

        let wasm_path = infer_wasm_output_path(&subject.source_dir, &build.args);

        Ok(AdaptationReport {
            subject_name: subject.name.clone(),
            language: subject.language.clone(),
            status: AdaptationStatus::DirectWasm,
            artifact: Some(BuildArtifact::Wasm { wasm_path }),
            notes: vec!["compiled with Go WASI build command".to_string()],
            blockers: vec![],
        })
    }
}

fn infer_wasm_output_path(source_dir: &Path, args: &[String]) -> PathBuf {
    let output = args
        .windows(2)
        .find_map(|pair| (pair[0] == "-o").then(|| PathBuf::from(&pair[1])))
        .unwrap_or_else(|| PathBuf::from("tool.wasm"));

    if output.is_absolute() {
        output
    } else {
        source_dir.join(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapts_go_env_leak_to_wasm() {
        let raw = std::fs::read_to_string("case_studies/go-env-leak/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let adapter = GoWasiAdapter;
        let report = adapter.adapt(&subject).expect("adapt subject");
        assert!(matches!(report.status, AdaptationStatus::DirectWasm));
        let Some(BuildArtifact::Wasm { wasm_path }) = report.artifact else {
            panic!("expected wasm artifact");
        };
        assert!(
            wasm_path.exists(),
            "wasm not found: {}",
            wasm_path.display()
        );
    }
}
