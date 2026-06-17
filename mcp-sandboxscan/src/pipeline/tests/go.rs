use std::collections::HashMap;
use std::path::PathBuf;

use crate::adapter::AdaptationStatus;
use crate::monitor::event::MonitorEventKind;
use crate::pipeline::fixtures::{ensure_go_build, ensure_go_sdk_examples};
use crate::pipeline::scan_subject;
use crate::subject::SubjectManifest;
use crate::taint::source::TaintSource;

#[test]
fn scans_go_env_leak_subject() {
    let raw = std::fs::read_to_string("case_studies/go-env-leak/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "SEKRET_0123456789abcdef".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::DirectWasm);
    assert!(result.report.summary.has_external_to_prompt_flow);
    assert!(result.report.summary.num_flows > 0);
    assert!(
        result
            .report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET")
    );
}

#[test]
fn scans_go_c2_beacon_subject() {
    let raw = std::fs::read_to_string("case_studies/go-c2-beacon/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "BEACON_TOKEN_0123456789".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::DirectWasm);
    assert!(
        result
            .report
            .sources
            .iter()
            .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
        "expected NetworkConnect taint source"
    );
    assert!(result.report.sources.iter().any(|src| {
        matches!(
            src,
            TaintSource::NetworkConnect { protocol, .. } if protocol == "http-intent"
        )
    }));
}

#[test]
fn scans_go_mcp_echo_subject() {
    let raw = std::fs::read_to_string("case_studies/go-mcp-echo/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
    ensure_go_build(&subject, "server");

    let result = scan_subject(&subject, &HashMap::new(), None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
    assert_eq!(result.report.summary.num_sinks, 1);
    assert_eq!(result.report.summary.num_flows, 0);
    assert!(result.report.mcp_transcript.is_some());
    assert!(
        result.report.sinks[0]
            .as_text()
            .contains("hello from sandboxscan")
    );
}

#[test]
fn scans_go_mcp_env_leak_subject() {
    let raw = std::fs::read_to_string("case_studies/go-mcp-env-leak/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
    ensure_go_build(&subject, "server");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "SEKRET_0123456789abcdef".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
    assert!(result.report.summary.has_external_to_prompt_flow);
    assert!(
        result
            .report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET")
    );
}

#[test]
fn scans_go_mcp_c2_beacon_subject() {
    let raw = std::fs::read_to_string("case_studies/go-mcp-c2-beacon/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
    ensure_go_build(&subject, "server");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "BEACON_TOKEN_0123456789".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
    assert!(
        result
            .report
            .sources
            .iter()
            .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
        "expected NetworkConnect taint source from egress proxy"
    );
    assert!(result.report.events.iter().any(|event| {
        event.kind == MonitorEventKind::NetworkConnectDenied
            || event.kind == MonitorEventKind::NetworkConnectAttempt
    }));
    assert!(result.report.sinks[0].as_text().contains("beacon"));
}

#[test]
fn scans_go_mcp_upstream_hello_subject() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ensure_go_sdk_examples(&manifest_dir);

    let raw = std::fs::read_to_string("case_studies/go-mcp-upstream-hello/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");
    ensure_go_build(&subject, "upstream-hello-server");

    let result = scan_subject(&subject, &HashMap::new(), None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
    assert_eq!(result.report.summary.num_sinks, 1);
    assert_eq!(result.report.summary.num_flows, 0);
    assert!(result.report.mcp_transcript.is_some());
    assert_eq!(
        result.report.mcp_transcript.as_ref().unwrap().events.len(),
        5
    );
    assert!(
        result.report.sinks[0]
            .as_text()
            .contains("Hi hello from upstream go-sdk")
    );
}
