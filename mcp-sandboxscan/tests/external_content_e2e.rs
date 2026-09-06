use std::path::PathBuf;

use mcp_sandboxscan::attack::{load_scenario, run_external_content_experiment};

#[test]
fn github_issue_triggers_environment_canary_flow_to_tool_output() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenario_path =
        manifest_dir.join("fixtures/external_content_e2e/github_issue_env_leak.json");
    let scenario = load_scenario(&scenario_path).expect("load external-content scenario");
    let report = run_external_content_experiment(
        &scenario,
        scenario_path.parent().expect("scenario directory"),
    )
    .expect("run external-content experiment");

    assert!(report.oracle.call_derived_from_external_content);
    assert!(report.oracle.expected_tool_called);
    assert!(report.oracle.authority_use_inferred);
    assert!(report.oracle.canary_reached_tool_output);
    assert!(report.oracle.source_to_sink_flow_detected);
    assert!(report.oracle.passed);
    assert_eq!(report.trigger.tool_name, "get_environment_status");
}
