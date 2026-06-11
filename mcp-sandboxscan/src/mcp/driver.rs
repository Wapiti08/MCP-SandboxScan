use anyhow::Result;
use serde_json::Value;
/*
The interface to call and run real MCP
*/

use super::transcript::McpTranscript;
use crate::sandbox::exec_evidence::ExecutionEvidence;


pub struct McpCallPlan {
    pub tool_name: String,
    pub arguments: Value,
}

pub struct McpDriverResult {
    pub exec: ExecutionEvidence,
    pub transcript: McpTranscript,
    pub tool_result_payload: Value,
}


pub trait McpDriver {
    fn call_tool(&self, plan: &McpCallPlan) -> Result<McpDriverResult>;
}