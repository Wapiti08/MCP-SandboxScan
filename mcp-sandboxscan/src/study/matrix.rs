use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
    subject_paths: &[PathBuf],
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> StudyMatrix {
    let cases: Vec<StudyCaseResult> = subject_paths
        .iter()
        .map(|subject_path| scan_subject_for_matrix(subject_path, env, data_dir, max_output_bytes))
        .collect();
    let summary = StudySummary::from_cases(&cases);

    StudyMatrix { cases, summary }
}

fn scan_subject_for_matrix(
    subject_path: &Path,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> StudyCaseResult {
    match load_subject(subject_path).and_then(|subject| {
        let report = scan_subject(&subject, env, data_dir, max_output_bytes)?;
        Ok((subject, report))
    }) {
        Ok((subject, report)) => StudyCaseResult {
            subject_name: subject.name,
            subject_path: subject_path.to_path_buf(),
            language: subject.language,
            wasm_status: WasmPortabilityStatus::DirectWasm,
            num_sources: report.summary.num_sources,
            num_sinks: report.summary.num_sinks,
            num_flows: report.summary.num_flows,
            has_external_to_prompt_flow: report.summary.has_external_to_prompt_flow,
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

        let matrix = run_subject_matrix(&subject_paths, &env, None, 4096);

        assert_eq!(matrix.summary.total_cases, 1);
        assert_eq!(matrix.summary.scanned_cases, 1);
        assert_eq!(matrix.summary.failed_cases, 0);
        assert_eq!(matrix.summary.detected_cases, 1);
        assert!(matrix.cases[0].has_external_to_prompt_flow);
        assert!(matrix.cases[0].num_flows > 0);
    }
}
