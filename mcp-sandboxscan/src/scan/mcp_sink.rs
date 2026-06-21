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
    extract_result_sinks(tool_result, "$", &mut sinks);
    sinks
}

fn extract_result_sinks(value: &Value, path: &str, sinks: &mut Vec<PromptSink>) {
    if let Some(results) = value.get("exploration_results").and_then(|v| v.as_array()) {
        for (idx, item) in results.iter().enumerate() {
            if let Some(result) = item.get("result") {
                extract_result_sinks(
                    result,
                    &format!("{path}.exploration_results[{idx}].result"),
                    sinks,
                );
            }
        }
        return;
    }

    if let Some(content) = value.get("content").and_then(|v| v.as_array()) {
        for (idx, item) in content.iter().enumerate() {
            if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    sinks.push(PromptSink::McpToolResultText {
                        path: format!("{path}.content[{idx}].text"),
                        value: text.to_string(),
                    });
                }
            }
        }
    }

    if let Some(structured) = value.get("structuredContent") {
        extract_string_leaves(structured, &format!("{path}.structuredContent"), sinks);
    }
}

fn extract_string_leaves(value: &Value, path: &str, sinks: &mut Vec<PromptSink>) {
    match value {
        Value::String(text) => sinks.push(PromptSink::ToolReturnLeaf {
            path: path.to_string(),
            value: text.clone(),
        }),
        Value::Array(items) => {
            for (idx, item) in items.iter().enumerate() {
                extract_string_leaves(item, &format!("{path}[{idx}]"), sinks);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                extract_string_leaves(item, &format!("{path}.{key}"), sinks);
            }
        }
        _ => {}
    }
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

    #[test]
    fn extracts_structured_content_leaves() {
        let payload = serde_json::json!({
            "structuredContent": {
                "status": "ok SECRET",
                "nested": {"message": "hello"}
            }
        });

        let sinks = extract_mcp_tool_result_sinks(&payload);

        assert_eq!(sinks.len(), 2);
        assert!(sinks.iter().any(|sink| matches!(
            sink,
            PromptSink::ToolReturnLeaf { path, value }
                if path == "$.structuredContent.status" && value == "ok SECRET"
        )));
    }

    #[test]
    fn extracts_exploration_result_sinks() {
        let payload = serde_json::json!({
            "exploration_results": [
                {
                    "tool": "status",
                    "result": {
                        "content": [{"type": "text", "text": "CANARY"}]
                    }
                }
            ]
        });

        let sinks = extract_mcp_tool_result_sinks(&payload);

        assert_eq!(sinks.len(), 1);
        assert!(matches!(
            &sinks[0],
            PromptSink::McpToolResultText { path, value }
                if path == "$.exploration_results[0].result.content[0].text" && value == "CANARY"
        ));
    }
}
