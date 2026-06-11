// Scanning report module for MCP SandboxScan

use serde::{Deserialize, Serialize};

use crate::scan::prompt_sink::PromptSink;
use crate::taint::flow::FlowMatch;
use crate::taint::source::TaintSource;
use crate::mcp::transcript::McpTranscript;
use crate::sandbox::exec_evidence::ExecutionEvidence;
use crate::monitor::event::MonitorEvent;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub exec: ExecutionEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_transcript: Option<McpTranscript>,
    #[serde(default)]
    pub events: Vec<MonitorEvent>,
    pub sources: Vec<TaintSource>,
    pub sinks: Vec<PromptSink>,
    pub flows: Vec<FlowMatch>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub num_sources: usize,
    pub num_sinks: usize,
    pub num_flows: usize,
    pub has_external_to_prompt_flow: bool,
}


