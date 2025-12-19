use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptSink {
    StdoutPrompt {line: String},
    JsonPrompt {key: String, value: String},
}

impl PromptSink {
    // &self is PromptSink - extract text which can be analyzed
    pub fn as_text(&self) -> &str {
        match self {
            PromptSink::StdoutPrompt { line } => line,
            PromptSink::JsonPrompt { value, .. } => value,
        }
    }
}


/// Extract prompt sinks from stdout string
pub fn extract_prompt_sinks(stdout: &str) -> Vec<PromptSink> {
    let mut sinks = vec![];

    for line in stdout.lines() {
        // trim line
        let trimmed = line.trim();
        // 1) Explicit Prompt
        if trimmed.contains("PROMPT:") {
            sinks.push(PromptSink::StdoutPrompt {
                line: trimmed.to_string(),
            });
            continue;
        }
        // 2) JSON line with prompt/messages
        if let Ok(v) = serde_json::from_str::<Value>(traimmed){
            // prompt
            if let Some(p) = v.get("prompt").and_then(|x| x.as_str()) {
                sinks.push(PromptSink::JsonPrompt {
                    key: "prompt".to_string(),
                    value: p.to_string(),
                });
            }

            // messages: [{role, content}, ...]
            if let Some(arr) = v.get("messages").and_then(|x| x.as_array()) {
                let mut combined = String::new();
                for m in arr {
                    if Some(c) = m.get("content").and_then(|x| x.as_str()) {
                        combined.push_str(c);
                        combined.push('\n');
                    }
                }
                if !combined.trim().is_empty() {
                    sinks.push(PromptSink::JsonPrompt {
                        key: "messages".to_string(),
                        value: combined.trim().to_string(),
                    });
                }
            }
        }
    }
    sinks
}

