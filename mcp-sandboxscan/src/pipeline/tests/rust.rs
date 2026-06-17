use std::collections::HashMap;
use std::path::PathBuf;

use crate::adapter::AdaptationStatus;
use crate::monitor::event::MonitorEventKind;
use crate::pipeline::fixtures::ensure_rust_mcp_filesystem_repo;
use crate::pipeline::scan_subject;
use crate::subject::SubjectManifest;
use crate::taint::source::TaintSource;

#[test]
fn scans_rust_c2_beacon_subject() {
    let raw = std::fs::read_to_string("case_studies/rust-c2-beacon/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "BEACON_TOKEN_0123456789".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert!(
        result
            .report
            .sources
            .iter()
            .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
        "expected NetworkConnect taint source"
    );
    assert!(
        result.report.events.iter().any(|event| {
            event.kind == MonitorEventKind::NetworkConnectDenied
                || event.kind == MonitorEventKind::NetworkConnectAttempt
        }),
        "expected network monitor events"
    );
    assert!(
        result.report.sources.iter().any(|src| {
            matches!(
                src,
                TaintSource::NetworkConnect { protocol, .. } if protocol == "http-intent"
            )
        }),
        "expected stdout HTTP_FETCH intent source"
    );
}

#[test]
fn scans_rust_mcp_c2_beacon_subject() {
    let raw = std::fs::read_to_string("case_studies/rust-mcp-c2-beacon/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

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
    assert!(
        result.report.sinks[0].as_text().contains("beacon"),
        "expected beacon tool result text"
    );
    assert!(result.report.mcp_transcript.is_some());
}

#[test]
fn scans_rust_env_leak_subject() {
    let raw = std::fs::read_to_string("case_studies/rust-env-leak/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "SEKRET_0123456789abcdef".to_string(),
    );

    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

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
fn scans_rust_mcp_filesystem_subject() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ensure_rust_mcp_filesystem_repo(&manifest_dir);

    let raw = std::fs::read_to_string("case_studies/rust-mcp-filesystem/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let data_dir = manifest_dir.join("data");
    std::fs::create_dir_all(&data_dir).expect("create data dir");

    let result =
        scan_subject(&subject, &HashMap::new(), Some(&data_dir), 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::NativeOnly);
    assert_eq!(result.report.summary.num_sinks, 1);
    assert_eq!(result.report.summary.num_flows, 0);
    assert!(result.report.mcp_transcript.is_some());
    assert_eq!(
        result.report.mcp_transcript.as_ref().unwrap().events.len(),
        5
    );

    let text = result.report.sinks[0].as_text();
    let data_dir_text = data_dir.to_string_lossy().into_owned();
    assert!(text.contains("Allowed directories") || text.contains(&data_dir_text));
}
