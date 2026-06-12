use std::collections::HashMap;

use crate::pipeline::fixtures::ensure_go_build;

use super::helpers::{assert_basic_mcp_driver_result, load_subject, scan_case};

#[test]
fn driver_calls_go_mcp_echo() {
    let subject_path = "case_studies/go-mcp-echo/subject.toml";
    let subject = load_subject(subject_path);
    ensure_go_build(&subject, "server");

    let result = scan_case(subject_path, &HashMap::new(), None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.sinks[0]
        .as_text()
        .contains("hello from sandboxscan"));
}

#[test]
fn driver_calls_go_mcp_env_leak() {
    let subject_path = "case_studies/go-mcp-env-leak/subject.toml";
    let subject = load_subject(subject_path);
    ensure_go_build(&subject, "server");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "SEKRET_0123456789abcdef".to_string(),
    );

    let result = scan_case(subject_path, &env, None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.summary.has_external_to_prompt_flow);
    assert!(result
        .report
        .flows
        .iter()
        .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
}

#[test]
fn driver_calls_go_mcp_c2_beacon() {
    let subject_path = "case_studies/go-mcp-c2-beacon/subject.toml";
    let subject = load_subject(subject_path);
    ensure_go_build(&subject, "server");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "BEACON_TOKEN_0123456789".to_string(),
    );

    let result = scan_case(subject_path, &env, None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.sinks[0].as_text().contains("beacon"));
}
