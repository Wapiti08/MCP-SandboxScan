// Scanning report module for MCP SandboxScan

use serde::{Deserialize, Serialize};

use crate::sandbox::exec_result::WasmExecResult;
use crate::scan::prompt_sink::PromptSink;
use crate::taint::flow::FlowMatch;
use crate::taint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub exec: WasmExecResult,

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


