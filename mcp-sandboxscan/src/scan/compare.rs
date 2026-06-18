use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::pipeline::case_study::{default_env_for_subject, resolve_data_dir};
use crate::pipeline::scan_subject;
use crate::scan::prompt_sink::extract_prompt_sinks;
use crate::scan::tool_return_sink::extract_tool_return_sinks;
use crate::subject::{Capability, SubjectManifest};
use crate::taint::flow::detect_flows;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkCompareRow {
    pub subject: String,
    pub subject_toml: String,
    pub has_mcp_protocol: bool,
    pub exec_stdout_len: usize,
    pub stdout_sink_count: usize,
    pub full_stdout_sink_count: usize,
    pub mcp_sink_count: usize,
    pub stdout_flow_count: usize,
    pub full_stdout_flow_count: usize,
    pub mcp_flow_count: usize,
    /// True when protocol-aware analysis finds flows that prompt-only stdout parsing misses.
    pub protocol_wins: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkCompareMatrix {
    pub rows: Vec<SinkCompareRow>,
}

pub fn compare_subject(
    manifest_dir: &Path,
    subject_path: &Path,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> SinkCompareRow {
    match compare_subject_inner(manifest_dir, subject_path, env, data_dir, max_output_bytes) {
        Ok(row) => row,
        Err(err) => failed_row(subject_path, err),
    }
}

fn compare_subject_inner(
    manifest_dir: &Path,
    subject_path: &Path,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> Result<SinkCompareRow> {
    let raw = std::fs::read_to_string(subject_path)
        .with_context(|| format!("failed to read {}", subject_path.display()))?;
    let subject: SubjectManifest = toml::from_str(&raw)
        .with_context(|| format!("failed to parse {}", subject_path.display()))?;

    let scan_env = default_env_for_subject(&subject, env);
    let effective_data_dir = resolve_data_dir(manifest_dir, &subject, data_dir)?;

    let result = scan_subject(
        &subject,
        &scan_env,
        effective_data_dir.as_deref(),
        max_output_bytes,
    )
    .with_context(|| format!("failed to scan subject {}", subject.name))?;

    Ok(compare_from_report(
        &subject,
        subject_path,
        &result.report,
    ))
}

pub fn compare_from_report(
    subject: &SubjectManifest,
    subject_path: &Path,
    report: &crate::scan::report::ScanReport,
) -> SinkCompareRow {
    let has_mcp_protocol = subject.capabilities.contains(&Capability::McpProtocol);

    let stdout_only_sinks = extract_prompt_sinks(&report.exec.stdout);
    let mut full_stdout_sinks = stdout_only_sinks.clone();
    full_stdout_sinks.extend(extract_tool_return_sinks(&report.exec.stdout));

    let stdout_flows = detect_flows(&report.sources, &stdout_only_sinks);
    let full_stdout_flows = detect_flows(&report.sources, &full_stdout_sinks);

    let mcp_sink_count = report
        .sinks
        .iter()
        .filter(|sink| {
            matches!(
                sink,
                crate::scan::prompt_sink::PromptSink::McpToolResultText { .. }
            )
        })
        .count();

    let mcp_flow_count = if has_mcp_protocol {
        report.summary.num_flows
    } else {
        full_stdout_flows.len()
    };

    let stdout_flow_count = stdout_flows.len();
    let full_stdout_flow_count = full_stdout_flows.len();

    SinkCompareRow {
        subject: subject.name.clone(),
        subject_toml: subject_path.to_string_lossy().into_owned(),
        has_mcp_protocol,
        exec_stdout_len: report.exec.stdout.len(),
        stdout_sink_count: stdout_only_sinks.len(),
        full_stdout_sink_count: full_stdout_sinks.len(),
        mcp_sink_count,
        stdout_flow_count,
        full_stdout_flow_count,
        mcp_flow_count,
        protocol_wins: mcp_flow_count > stdout_flow_count,
        error: None,
    }
}

pub fn compare_subjects(
    manifest_dir: &Path,
    subject_paths: &[PathBuf],
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> SinkCompareMatrix {
    let rows = subject_paths
        .iter()
        .map(|path| compare_subject(manifest_dir, path, env, data_dir, max_output_bytes))
        .collect();
    SinkCompareMatrix { rows }
}

pub fn discover_case_studies(manifest_dir: &Path) -> Result<Vec<PathBuf>> {
    let root = manifest_dir.join("case_studies");
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("failed to read {}", root.display()))?
    {
        let entry = entry?;
        let subject_toml = entry.path().join("subject.toml");
        if subject_toml.is_file() {
            paths.push(subject_toml);
        }
    }
    paths.sort();
    Ok(paths)
}

fn failed_row(subject_path: &Path, err: anyhow::Error) -> SinkCompareRow {
    let subject = subject_path
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    SinkCompareRow {
        subject,
        subject_toml: subject_path.to_string_lossy().into_owned(),
        has_mcp_protocol: false,
        exec_stdout_len: 0,
        stdout_sink_count: 0,
        full_stdout_sink_count: 0,
        mcp_sink_count: 0,
        stdout_flow_count: 0,
        full_stdout_flow_count: 0,
        mcp_flow_count: 0,
        protocol_wins: false,
        error: Some(err.to_string()),
    }
}
