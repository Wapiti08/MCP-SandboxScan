use serde_json::json;
use std::collections::HashMap;

use crate::mcp::transcript::{McpDirection, McpTranscript};
use crate::monitor::event::{MonitorEvent, MonitorEventKind};

/*
transfer JSON-RPC transcript to structural events
*/

pub fn monitor_events_from_transcript(transcript: &McpTranscript) -> Vec<MonitorEvent> {
    let mut events = Vec::new();

    // Correlate response id -> original request method/tool name.
    let mut request_methods: HashMap<u64, String> = HashMap::new();
    let mut request_tools: HashMap<u64, String> = HashMap::new();

    for event in &transcript.events {
        let id = event.payload.get("id").and_then(|v| v.as_u64());

        if matches!(event.direction, McpDirection::ClientToServer) {
            if let Some(method) = event.payload.get("method").and_then(|v| v.as_str()) {
                if let Some(id) = id {
                    request_methods.insert(id, method.to_string());
                }

                let kind = match method {
                    "initialize" => Some(MonitorEventKind::McpInitialize),
                    "tools/list" => Some(MonitorEventKind::McpToolsList),
                    "tools/call" => Some(MonitorEventKind::McpToolCall),
                    "resources/read" => Some(MonitorEventKind::McpResourceRead),
                    "prompts/get" => Some(MonitorEventKind::McpPromptGet),
                    _ => None,
                };

                if method == "tools/call" {
                    // refer to structure of tools/call: params.name
                    if let Some(id) = id {
                        if let Some(tool_name) = event
                            .payload
                            .pointer("/params/name")
                            .and_then(|v| v.as_str())
                        {
                            request_tools.insert(id, tool_name.to_string());
                        }
                    }
                }

                if let Some(kind) = kind {
                    events.push(MonitorEvent {
                        kind,
                        actor: "mcp-client".to_string(),
                        target: event
                            .payload
                            .get("method")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        evidence: event.payload.clone(),
                    });
                }
            }
        }
        // server response events
        if matches!(event.direction, McpDirection::ServerToClient) {
            if let Some(id) = id {
                if request_methods.get(&id).map(String::as_str) == Some("tools/call") {
                    events.push(MonitorEvent {
                        kind: MonitorEventKind::McpToolResult,
                        actor: "mcp-server".to_string(),
                        target: request_tools.get(&id).cloned(),
                        evidence: json!({
                            "response": event.payload,
                            "tool_name": request_tools.get(&id),
                        }),
                    });
                }
            }
        }
    }
    events
}
