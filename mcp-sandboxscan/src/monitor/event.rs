use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::scan::prompt_sink::PromptSink;
use crate::taint::flow::FlowMatch;
use crate::taint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub kind: MonitorEventKind,
    pub actor: String,
    pub target: Option<String>,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MonitorEventKind {
    CapabilityGranted,
    EnvRead,
    FileRead,
    FileWrite,
    NetworkRequest,
    ProcessSpawn,
    McpInitialize,
    McpToolsList,
    McpToolCall,
    McpToolResult,
    McpResourceRead,
    McpPromptGet,
    SinkObserved,
    FlowDetected,
}

pub fn sink_events(sinks: &[PromptSink]) -> Vec<MonitorEvent> {
    sinks
    .iter()
    .map(|sink| MonitorEvent {
        kind: MonitorEventKind::SinkObserved,
        actor: "scanner".to_string(),
        target: Some(match sink {
            PromptSink::StdoutPrompt { .. } => "stdout-prompt",
            PromptSink::JsonPrompt { .. } => "json-prompt",
            PromptSink::ToolReturnLeaf { .. } => "tool-return",
            PromptSink::McpToolResultText { .. } => "mcp-tool-result",
        }.to_string()),
        evidence: json!({
            "sink": sink,
            "text_len": sink.as_text().len()
        }),
    })
    .collect()
}

pub fn flow_events(flows: &[FlowMatch]) -> Vec<MonitorEvent> {
    flows
        .iter()
        .map(|flow| MonitorEvent {
            kind: MonitorEventKind::FlowDetected,
            actor: "scanner".to_string(),
            target: Some(flow.sink_type.clone()),
            evidence: json!(flow),
        })
        .collect()
}

pub fn source_inventory_events(sources: &[TaintSource]) -> Vec<MonitorEvent> {
    sources
        .iter()
        .map(|source| match source {
            TaintSource::EnvVar { key, value } => MonitorEvent {
                kind: MonitorEventKind::EnvRead,
                actor: "scanner".to_string(),
                target: Some(key.clone()),
                evidence: json!({
                    "source_id": source.short_id(),
                    "value_len": value.len()
                }),
            },
            TaintSource::FileRead { path, content } => MonitorEvent {
                kind: MonitorEventKind::FileRead,
                actor: "scanner".to_string(),
                target: Some(path.clone()),
                evidence: json!({
                    "source_id": source.short_id(),
                    "content_len": content.len()
                }),
            },
            TaintSource::HttpFetch { url, content } => MonitorEvent {
                kind: MonitorEventKind::NetworkRequest,
                actor: "scanner".to_string(),
                target: Some(url.clone()),
                evidence: json!({
                    "source_id": source.short_id(),
                    "content_len": content.len()
                }),
            },
        })
        .collect()
}
