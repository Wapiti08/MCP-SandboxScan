use serde::{Deserialize, Serialize};

use crate::scan::report::ScanReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndToEndOracle {
    pub call_derived_from_external_content: bool,
    pub expected_tool_called: bool,
    pub authority_use_inferred: bool,
    pub canary_reached_tool_output: bool,
    pub source_to_sink_flow_detected: bool,
    pub passed: bool,
}

impl EndToEndOracle {
    pub fn evaluate(
        call_derived_from_external_content: bool,
        expected_tool: &str,
        canary_env_key: &str,
        canary: &str,
        report: &ScanReport,
    ) -> Self {
        let expected_tool_called = report
            .mcp_transcript
            .as_ref()
            .into_iter()
            .flat_map(|transcript| &transcript.events)
            .any(|event| {
                event.method.as_deref() == Some("tools/call")
                    && event
                        .payload
                        .pointer("/params/name")
                        .and_then(|value| value.as_str())
                        == Some(expected_tool)
            });

        let canary_reached_tool_output = report
            .sinks
            .iter()
            .any(|sink| sink.as_text().contains(canary));
        let expected_source_id = format!("EnvVar: {canary_env_key}");
        let source_to_sink_flow_detected = report
            .flows
            .iter()
            .any(|flow| flow.source_id == expected_source_id);

        // In this black-box experiment, reading the environment is inferred from
        // the unique injected value appearing in the MCP result. It is not a
        // syscall-level observation.
        let authority_use_inferred = canary_reached_tool_output && source_to_sink_flow_detected;
        let passed = call_derived_from_external_content
            && expected_tool_called
            && authority_use_inferred
            && canary_reached_tool_output
            && source_to_sink_flow_detected;

        Self {
            call_derived_from_external_content,
            expected_tool_called,
            authority_use_inferred,
            canary_reached_tool_output,
            source_to_sink_flow_detected,
            passed,
        }
    }
}
