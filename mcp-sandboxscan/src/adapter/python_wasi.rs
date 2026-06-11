use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::adapter::{AdaptationReport, AdaptationStatus, Adapter, BuildArtifact};
use crate::subject::{Language, SubjectManifest};

pub struct PythonWasiAdapter;

impl Adapter for PythonWasiAdapter {
    fn name(&self) -> &'static str {
        "python-wasi"
    }

    fn adapt(&self, subject: &SubjectManifest) -> Result<AdaptationReport> {
        if subject.language != Language::Python {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Unsupported,
                artifact: None,
                notes: vec![],
                blockers: vec!["PythonWasiAdapter only supports Python subjects".to_string()],
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

        if build.command != "cpython-wasi-bundle" {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Failed,
                artifact: None,
                notes: vec![],
                blockers: vec![format!(
                    "unsupported Python build command `{}` (expected `cpython-wasi-bundle`)",
                    build.command
                )],
            });
        }

        let entry = subject
            .entrypoint
            .as_deref()
            .unwrap_or("main.py")
            .to_string();
        let bundle_dir = infer_bundle_dir(&subject.source_dir, &build.args);
        std::fs::create_dir_all(&bundle_dir).with_context(|| {
            format!(
                "failed to create Python bundle dir {}",
                bundle_dir.display()
            )
        })?;

        let source_script = subject.source_dir.join(&entry);
        let bundled_script = bundle_dir.join(&entry);
        std::fs::copy(&source_script, &bundled_script).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source_script.display(),
                bundled_script.display()
            )
        })?;

        let interpreter_wasm = resolve_python_wasm()?;
        if !interpreter_wasm.exists() {
            return Ok(AdaptationReport {
                subject_name: subject.name.clone(),
                language: subject.language.clone(),
                status: AdaptationStatus::Failed,
                artifact: None,
                notes: vec![],
                blockers: vec![format!(
                    "CPython WASI interpreter not found at {}",
                    interpreter_wasm.display()
                )],
            });
        }

        let guest_script = format!("/work/{entry}");
        Ok(AdaptationReport {
            subject_name: subject.name.clone(),
            language: subject.language.clone(),
            status: AdaptationStatus::WasmWithShim,
            artifact: Some(BuildArtifact::PythonWasm {
                interpreter_wasm,
                work_dir: bundle_dir,
                argv: vec!["python".to_string(), guest_script],
            }),
            notes: vec!["bundled with CPython WASI interpreter".to_string()],
            blockers: vec![],
        })
    }
}

fn infer_bundle_dir(source_dir: &Path, args: &[String]) -> PathBuf {
    let output = args
        .windows(2)
        .find_map(|pair| (pair[0] == "-o").then(|| PathBuf::from(&pair[1])))
        .unwrap_or_else(|| PathBuf::from("bundle"));

    if output.is_absolute() {
        output
    } else {
        source_dir.join(output)
    }
}

fn resolve_python_wasm() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("MCP_SANDBOXSCAN_PYTHON_WASM") {
        return Ok(PathBuf::from(path));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join("external/cpython-wasi/python.wasm"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapts_python_env_leak_to_wasm() {
        let raw = std::fs::read_to_string("case_studies/python-env-leak/subject.toml")
            .expect("read subject.toml");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject.toml");
        let adapter = PythonWasiAdapter;
        let report = adapter.adapt(&subject).expect("adapt subject");
        assert!(matches!(report.status, AdaptationStatus::WasmWithShim));

        let Some(BuildArtifact::PythonWasm {
            interpreter_wasm,
            work_dir,
            argv,
        }) = report.artifact
        else {
            panic!("expected PythonWasm artifact");
        };

        assert!(work_dir.join("main.py").exists(), "bundled main.py missing");
        assert_eq!(argv, vec!["python", "/work/main.py"]);

        if !interpreter_wasm.exists() {
            eprintln!(
                "skipping interpreter existence check: {} not found (run scripts/fetch-cpython-wasi.sh)",
                interpreter_wasm.display()
            );
        }
    }
}
