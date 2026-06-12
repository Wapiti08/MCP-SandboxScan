use std::collections::HashMap;
use std::path::PathBuf;

use crate::pipeline::{
    ensure_fastmcp_examples, ensure_python_fastmcp_venv, ensure_python_venv,
};

use super::helpers::{assert_basic_mcp_driver_result, scan_case};

#[test]
fn driver_calls_mcp_server_fetch() {
    let subject_path = "case_studies/python-mcp-server-fetch/subject.toml";
    let subject = super::helpers::load_subject(subject_path);
    ensure_python_venv(&subject, "mcp_server_fetch");

    let result = scan_case(subject_path, &HashMap::new(), None, 8192);
    assert_basic_mcp_driver_result(&result);
}

#[test]
fn driver_calls_upstream_fastmcp_echo() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ensure_fastmcp_examples(&manifest_dir);

    let subject_path = "case_studies/python-fastmcp-upstream-echo/subject.toml";
    let subject = super::helpers::load_subject(subject_path);
    ensure_python_fastmcp_venv(&subject);

    let result = scan_case(subject_path, &HashMap::new(), None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.sinks[0]
        .as_text()
        .contains("hello from upstream fastmcp"));
}

#[test]
fn driver_calls_fixture_fastmcp_echo() {
    let subject_path = "case_studies/python-fastmcp-echo/subject.toml";
    let subject = super::helpers::load_subject(subject_path);
    ensure_python_fastmcp_venv(&subject);

    let result = scan_case(subject_path, &HashMap::new(), None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.sinks[0]
        .as_text()
        .contains("hello from sandboxscan"));
}
