use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::pipeline::case_study::{default_env_for_subject, resolve_data_dir};
use crate::pipeline::scan_subject;
use crate::subject::{Language, SubjectManifest};

use super::portability::WasmPortabilityStatus;
use super::summary::StudySummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyMatrix {
    pub cases: Vec<StudyCaseResult>,
    pub summary: StudySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyCaseResult {
    pub subject_name: String,
    pub subject_path: PathBuf,
    pub language: Language,
    pub wasm_status: WasmPortabilityStatus,
    pub num_sources: usize,
    pub num_sinks: usize,
    pub num_flows: usize,
    pub has_external_to_prompt_flow: bool,
    pub error: Option<String>,
}

pub fn run_subject_matrix(
    manifest_dir: &Path,
    subject_paths: &[PathBuf],
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> StudyMatrix {
    let cases: Vec<StudyCaseResult> = subject_paths
        .iter()
        .map(|subject_path| {
            scan_subject_for_matrix(manifest_dir, subject_path, env, data_dir, max_output_bytes)
        })
        .collect();
    let summary = StudySummary::from_cases(&cases);

    StudyMatrix { cases, summary }
}

fn scan_subject_for_matrix(
    manifest_dir: &Path,
    subject_path: &Path,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> StudyCaseResult {
    match load_subject(subject_path).and_then(|subject| {
        let scan_env = default_env_for_subject(&subject, env);
        let effective_data_dir = resolve_data_dir(manifest_dir, &subject, data_dir)?;
        let result = scan_subject(
            &subject,
            &scan_env,
            effective_data_dir.as_deref(),
            max_output_bytes,
        )?;
        Ok((subject, result))
    }) {
        Ok((subject, result)) => StudyCaseResult {
            subject_name: subject.name,
            subject_path: subject_path.to_path_buf(),
            language: subject.language,
            wasm_status: result.adaptation_status.into(),
            num_sources: result.report.summary.num_sources,
            num_sinks: result.report.summary.num_sinks,
            num_flows: result.report.summary.num_flows,
            has_external_to_prompt_flow: result.report.summary.has_external_to_prompt_flow,
            error: None,
        },
        Err(err) => StudyCaseResult {
            subject_name: subject_path
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string(),
            subject_path: subject_path.to_path_buf(),
            language: Language::Unknown,
            wasm_status: WasmPortabilityStatus::Failed,
            num_sources: 0,
            num_sinks: 0,
            num_flows: 0,
            has_external_to_prompt_flow: false,
            error: Some(err.to_string()),
        },
    }
}

fn load_subject(subject_path: &Path) -> Result<SubjectManifest> {
    let raw = std::fs::read_to_string(subject_path)
        .with_context(|| format!("failed to read subject {}", subject_path.display()))?;
    toml::from_str(&raw)
        .with_context(|| format!("failed to parse subject {}", subject_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_runs_rust_env_leak_subject() {
        let subject_paths = vec![PathBuf::from("case_studies/rust-env-leak/subject.toml")];
        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let matrix = run_subject_matrix(&manifest_dir, &subject_paths, &env, None, 4096);

        assert_eq!(matrix.summary.total_cases, 1);
        assert_eq!(matrix.summary.scanned_cases, 1);
        assert_eq!(matrix.summary.failed_cases, 0);
        assert_eq!(matrix.summary.detected_cases, 1);
        assert!(matrix.cases[0].has_external_to_prompt_flow);
        assert!(matrix.cases[0].num_flows > 0);
    }

    fn env_with_demo_secret() -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );
        env
    }

    fn six_case_subject_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("case_studies/rust-benign/subject.toml"),
            PathBuf::from("case_studies/rust-env-leak/subject.toml"),
            PathBuf::from("case_studies/rust-file-exfil/subject.toml"),
            PathBuf::from("case_studies/python-benign/subject.toml"),
            PathBuf::from("case_studies/python-env-leak/subject.toml"),
            PathBuf::from("case_studies/python-file-exfil/subject.toml"),
        ]
    }

    #[test]
    #[ignore = "requires CPython WASI runtime"]
    fn matrix_runs_rust_python_six_case_study() {
        let data_dir = std::env::temp_dir().join(format!(
            "mcp-sandboxscan-matrix-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("secret.txt"), "top-secret\n").unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let matrix = run_subject_matrix(
            &manifest_dir,
            &six_case_subject_paths(),
            &env_with_demo_secret(),
            Some(&data_dir),
            4096,
        );
        let _ = std::fs::remove_dir_all(&data_dir);

        assert_eq!(matrix.summary.total_cases, 6);
        assert_eq!(matrix.summary.scanned_cases, 6);
        assert_eq!(matrix.summary.failed_cases, 0);
        assert_eq!(matrix.summary.detected_cases, 2);
        assert_eq!(matrix.summary.total_flows, 2);

        let pairs = [
            ("rust-benign", "python-benign"),
            ("rust-env-leak", "python-env-leak"),
            ("rust-file-exfil", "python-file-exfil"),
        ];

        for (rust_name, python_name) in pairs {
            let rust_case = matrix
                .cases
                .iter()
                .find(|case| case.subject_name == rust_name)
                .unwrap_or_else(|| panic!("missing {rust_name}"));
            let python_case = matrix
                .cases
                .iter()
                .find(|case| case.subject_name == python_name)
                .unwrap_or_else(|| panic!("missing {python_name}"));

            assert_eq!(
                rust_case.has_external_to_prompt_flow, python_case.has_external_to_prompt_flow,
                "{rust_name} vs {python_name}"
            );
            assert_eq!(
                rust_case.num_flows, python_case.num_flows,
                "{rust_name} vs {python_name}"
            );
            assert_eq!(
                rust_case.num_sinks, python_case.num_sinks,
                "{rust_name} vs {python_name}"
            );
            assert_eq!(
                rust_case.wasm_status,
                WasmPortabilityStatus::DirectWasm,
                "{rust_name}"
            );
            assert_eq!(
                python_case.wasm_status,
                WasmPortabilityStatus::WasmWithShim,
                "{python_name}"
            );
        }
    }

    fn go_wasi_subject_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("case_studies/go-benign/subject.toml"),
            PathBuf::from("case_studies/go-env-leak/subject.toml"),
            PathBuf::from("case_studies/go-file-exfil/subject.toml"),
        ]
    }

    #[test]
    #[ignore = "requires Go toolchain for wasip1 cross-compile"]
    fn matrix_runs_rust_go_wasi_parity() {
        let data_dir = std::env::temp_dir().join(format!(
            "mcp-sandboxscan-go-matrix-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("secret.txt"), "top-secret\n").unwrap();

        let rust_paths = vec![
            PathBuf::from("case_studies/rust-benign/subject.toml"),
            PathBuf::from("case_studies/rust-env-leak/subject.toml"),
            PathBuf::from("case_studies/rust-file-exfil/subject.toml"),
        ];
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let rust_matrix = run_subject_matrix(
            &manifest_dir,
            &rust_paths,
            &env_with_demo_secret(),
            Some(&data_dir),
            4096,
        );
        let go_matrix = run_subject_matrix(
            &manifest_dir,
            &go_wasi_subject_paths(),
            &env_with_demo_secret(),
            Some(&data_dir),
            4096,
        );
        let _ = std::fs::remove_dir_all(&data_dir);

        assert_eq!(go_matrix.summary.failed_cases, 0);

        let pairs = [
            ("rust-benign", "go-benign"),
            ("rust-env-leak", "go-env-leak"),
            ("rust-file-exfil", "go-file-exfil"),
        ];

        for (rust_name, go_name) in pairs {
            let rust_case = rust_matrix
                .cases
                .iter()
                .find(|case| case.subject_name == rust_name)
                .unwrap_or_else(|| panic!("missing {rust_name}"));
            let go_case = go_matrix
                .cases
                .iter()
                .find(|case| case.subject_name == go_name)
                .unwrap_or_else(|| panic!("missing {go_name}"));

            assert_eq!(
                rust_case.has_external_to_prompt_flow, go_case.has_external_to_prompt_flow,
                "{rust_name} vs {go_name}"
            );
            assert_eq!(
                rust_case.num_flows, go_case.num_flows,
                "{rust_name} vs {go_name}"
            );
            assert_eq!(
                rust_case.num_sinks, go_case.num_sinks,
                "{rust_name} vs {go_name}"
            );
            assert_eq!(
                go_case.wasm_status,
                WasmPortabilityStatus::DirectWasm,
                "{go_name}"
            );
        }
    }
}
