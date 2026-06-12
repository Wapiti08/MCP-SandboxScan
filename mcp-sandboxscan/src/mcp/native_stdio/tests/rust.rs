use std::collections::HashMap;
use std::path::PathBuf;

use crate::pipeline::ensure_rust_mcp_filesystem_repo;

use super::helpers::{assert_basic_mcp_driver_result, scan_case};

#[test]
fn driver_calls_mcp_c2_beacon() {
    let subject_path = "case_studies/rust-mcp-c2-beacon/subject.toml";
    let mut env = HashMap::new();
    env.insert(
        "DEMO_SECRET".to_string(),
        "BEACON_TOKEN_0123456789".to_string(),
    );

    let result = scan_case(subject_path, &env, None, 4096);
    assert_basic_mcp_driver_result(&result);
    assert!(result.report.sinks[0].as_text().contains("beacon"));
}

#[test]
fn driver_calls_mcp_filesystem() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ensure_rust_mcp_filesystem_repo(&manifest_dir);

    let data_dir = manifest_dir.join("data");
    std::fs::create_dir_all(&data_dir).expect("create allowed data dir");

    let result = scan_case(
        "case_studies/rust-mcp-filesystem/subject.toml",
        &HashMap::new(),
        Some(&data_dir),
        4096,
    );
    assert_basic_mcp_driver_result(&result);
    assert_eq!(result.report.summary.num_flows, 0);
    assert!(result.report.sinks[0]
        .as_text()
        .contains("Allowed directories"));
}
