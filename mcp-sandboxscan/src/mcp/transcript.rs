use serde::{Deserialize, Serialize};
use serde_json::Value;

/*
store MCP-level evidence -- json response format
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTranscript {
    pub events: Vec<McpEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEvent {
    pub direction: McpDirection,
    pub method: Option<String>,
    // json format
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpDirection {
    ClientToServer,
    ServerToClient,
}
