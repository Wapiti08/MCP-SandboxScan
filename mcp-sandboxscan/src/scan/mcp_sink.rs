use serde_json::Value;

use crate::scan::prompt_sink::PromptSink;

/*
extract LLM-facing text values as sinks.
*/

pub fn extract_mcp_tool_result_sinks(tool_result: &Value) -> Vec<PromptSink> {
    /*
    the response shape is like:

    {
    "content": [
        {
        "type": "text",
        "text": "..."
        }
         ],
    "isError": false
    }
    */

    let mut sinks = Vec::new();

    if let Some(content) = tool_result.get("content").and_then(|v| v.as_array()) {
        for (idx, item) in content.iter().enumerate() {
            if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    sinks.push(PromptSink::McpToolResultText {
                        path: format!("$.content[{idx}].text"),
                        value: text.to_string(),
                    });
                }
            }
        }
    }
    sinks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mcp_tool_result_text() {
        let payload = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "secret output"
                }
            ],
            "isError": false
        });

        let sinks = extract_mcp_tool_result_sinks(&payload);

        assert_eq!(sinks.len(), 1);
        match &sinks[0] {
            PromptSink::McpToolResultText { path, value } => {
                assert_eq!(path, "$.content[0].text");
                assert_eq!(value, "secret output");
            }
            _ => panic!("expected McpToolResultText"),
        }
    }
}
