use std::path::Path;
use mcp_sandboxscan::scan::dynamic_scan::run_dynamic_scan;

#[test]
fn test_evil_prompt_detected() {
    let report = run_dynamic_scan(
        Path::new("fixtures/evil_prompt_tool/tool.wasm"),
        None,
        &Default::default(),
        4096,
        ).expect("scan failed");
    
    assert_eq!(report.sinks.len(), 1);
    assert!(report.summary.has_external_to_prompt_flow);
    assert!(report.summary.num_flows > 0);
}