use std::collections::HashMap;
use std::path::Path;

use crate::pipeline::{SubjectScanResult, scan_subject};
use crate::scan::prompt_sink::PromptSink;
use crate::subject::SubjectManifest;

pub fn load_subject(case_study: &str) -> SubjectManifest {
    let raw = std::fs::read_to_string(case_study).expect("read subject manifest");
    toml::from_str(&raw).expect("parse subject manifest")
}

pub fn scan_case(
    case_study: &str,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
) -> SubjectScanResult {
    let subject = load_subject(case_study);
    scan_subject(&subject, env, data_dir, max_output_bytes).expect("scan subject")
}

pub fn assert_basic_mcp_driver_result(result: &SubjectScanResult) {
    assert_eq!(result.report.summary.num_sinks, 1);
    assert_eq!(
        result.report.mcp_transcript.as_ref().unwrap().events.len(),
        5
    );
    assert!(matches!(
        result.report.sinks[0],
        PromptSink::McpToolResultText { .. }
    ));
}
