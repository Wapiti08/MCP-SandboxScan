use std::collections::HashMap;

use crate::adapter::AdaptationStatus;
use crate::pipeline::scan_subject;
use crate::subject::SubjectManifest;
use crate::taint::source::TaintSource;

#[test]
fn scans_ts_wasi_benign_subject() {
    let raw = std::fs::read_to_string("case_studies/ts-benign/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let result = scan_subject(&subject, &HashMap::new(), None, 4096).expect("scan subject");
    assert_eq!(result.adaptation_status, AdaptationStatus::DirectWasm);
    assert_eq!(result.report.summary.num_flows, 0);
    assert!(result.report.sinks[0].as_text().contains("static benign result"));
}

#[test]
fn scans_ts_wasi_env_leak_subject() {
    let raw = std::fs::read_to_string("case_studies/ts-env-leak/subject.toml")
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
    assert!(result
        .report
        .flows
        .iter()
        .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
}

#[test]
fn scans_ts_wasi_file_exfil_subject() {
    let raw = std::fs::read_to_string("case_studies/ts-file-exfil/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let tempdir = std::env::temp_dir().join(format!(
        "mcp-sandboxscan-ts-wasi-data-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tempdir).expect("create temp data dir");
    std::fs::write(tempdir.join("secret.txt"), "top-secret-0123456789").expect("write secret.txt");

    let result = scan_subject(&subject, &HashMap::new(), Some(&tempdir), 4096).expect("scan subject");
    std::fs::remove_dir_all(&tempdir).ok();

    assert_eq!(result.adaptation_status, AdaptationStatus::DirectWasm);
    assert!(result.report.summary.has_external_to_prompt_flow);
    assert!(result
        .report
        .flows
        .iter()
        .any(|flow| flow.source_id.contains("secret.txt")));
}

#[test]
fn scans_ts_wasi_c2_beacon_subject() {
    let raw = std::fs::read_to_string("case_studies/ts-c2-beacon/subject.toml")
        .expect("read subject manifest");
    let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

    let mut env = HashMap::new();
    env.insert("DEMO_SECRET".to_string(), "BEACON_TOKEN_0123456789".to_string());
    let result = scan_subject(&subject, &env, None, 4096).expect("scan subject");

    assert_eq!(result.adaptation_status, AdaptationStatus::DirectWasm);
    assert!(
        result
            .report
            .sources
            .iter()
            .any(|src| matches!(src, TaintSource::NetworkConnect { .. })),
        "expected NetworkConnect taint source from HTTP_FETCH intent"
    );
    assert!(result
        .report
        .sinks
        .iter()
        .any(|s| s.as_text().contains("beacon")));
}

