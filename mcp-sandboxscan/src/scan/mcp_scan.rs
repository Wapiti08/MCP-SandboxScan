use crate::mcp::driver::McpDriverResult;
use crate::scan::mcp_sink::extract_mcp_tool_result_sinks;
use crate::scan::report::{ScanReport, Summary};
use crate::taint::flow::detect_flows;
use crate::taint::source::TaintSource;



pub fn scan_mcp_driver_result(
    driver_result: McpDriverResult,
    sources: Vec<TaintSource>,
) -> ScanReport {
    let sinks = extract_mcp_tool_result_sinks(&driver_result.tool_result_payload);
    let flows = detect_flows(&sources, &sinks);
    let summary = Summary {
        num_sources: sources.len(),
        num_sinks: sinks.len(),
        num_flows: flows.len(),
        has_external_to_prompt_flow: !flows.is_empty(),
    };
    ScanReport {
        exec: driver_result.exec,
        mcp_transcript: Some(driver_result.transcript),
        sources,
        sinks,
        flows,
        summary,
    }
}