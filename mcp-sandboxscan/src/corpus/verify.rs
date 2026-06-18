use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::scan::report::ScanReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRow {
    pub repo_id: String,
    pub has_flow: bool,
    pub num_flows: usize,
    pub flow_snippets: Vec<String>,
    pub sink_preview: String,
    pub manual_label: Option<String>,
}

/// Extract a human-review packet from a per-case ScanReport JSON.
pub fn verify_packet(repo_id: &str, report_json: &Path) -> Result<VerifyRow> {
    let raw = fs::read_to_string(report_json)
        .with_context(|| format!("read {}", report_json.display()))?;
    let report: ScanReport = serde_json::from_str(&raw)?;

    Ok(VerifyRow {
        repo_id: repo_id.to_string(),
        has_flow: report.summary.has_external_to_prompt_flow,
        num_flows: report.summary.num_flows,
        flow_snippets: report.flows.iter().map(|f| f.snippet.clone()).collect(),
        sink_preview: report
            .sinks
            .first()
            .map(|s| s.as_text().chars().take(240).collect())
            .unwrap_or_default(),
        manual_label: None,
    })
}

pub fn verify_suspicious_cases(cases_dir: &Path) -> Result<Vec<VerifyRow>> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(cases_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let repo_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .replace("__", "/");
        let packet = verify_packet(&repo_id, &path)?;
        if packet.has_flow {
            rows.push(packet);
        }
    }
    rows.sort_by(|a, b| a.repo_id.cmp(&b.repo_id));
    Ok(rows)
}
