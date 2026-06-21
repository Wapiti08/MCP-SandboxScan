use serde_json::{Map, Value, json};

use crate::mcp::driver::McpCallPlan;

#[derive(Debug, Clone)]
pub struct ExplorationConfig {
    pub enabled: bool,
    pub max_tools: usize,
    pub env_canary: String,
    pub input_canary: String,
    pub file_canary_path: Option<String>,
}

impl ExplorationConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_tools: 1,
            env_canary: String::new(),
            input_canary: String::new(),
            file_canary_path: None,
        }
    }
}

impl Default for ExplorationConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Debug, Clone)]
struct ToolCandidate {
    name: String,
    score: i32,
    input_schema: Value,
}

pub fn build_exploration_plans(
    tools_response: &Value,
    config: &ExplorationConfig,
) -> Vec<McpCallPlan> {
    if !config.enabled || config.max_tools == 0 {
        return Vec::new();
    }

    let Some(tools) = tools_response
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
    else {
        return Vec::new();
    };

    let mut candidates: Vec<_> = tools
        .iter()
        .filter_map(|tool| {
            let name = tool.get("name")?.as_str()?.to_string();
            let description = tool
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or_default();
            let input_schema = tool
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if is_potentially_destructive(&name, description) {
                return None;
            }
            Some(ToolCandidate {
                score: score_tool(&name, description),
                name,
                input_schema,
            })
        })
        .filter(|candidate| candidate.score > 0)
        .collect();

    candidates.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    candidates
        .into_iter()
        .take(config.max_tools)
        .map(|tool| McpCallPlan {
            tool_name: tool.name,
            arguments: synthesize_arguments(&tool.input_schema, config),
        })
        .collect()
}

fn score_tool(name: &str, description: &str) -> i32 {
    let text = format!("{} {}", name, description).to_lowercase();
    let mut score = 0;
    for keyword in [
        "config",
        "setting",
        "env",
        "environment",
        "status",
        "debug",
        "info",
        "whoami",
        "get",
        "show",
        "read",
        "list",
        "file",
        "workspace",
        "project",
        "secret",
        "token",
    ] {
        if text.contains(keyword) {
            score += 10;
        }
    }
    score
}

fn is_potentially_destructive(name: &str, description: &str) -> bool {
    let text = format!("{} {}", name, description).to_lowercase();
    [
        "delete", "remove", "write", "create", "update", "insert", "drop", "apply", "execute",
        "exec", "run", "launch", "start", "open", "send", "post", "comment", "merge", "deploy",
        "install",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

fn synthesize_arguments(schema: &Value, config: &ExplorationConfig) -> Value {
    if schema.get("type").and_then(|t| t.as_str()) != Some("object") {
        return json!({});
    }

    let mut out = Map::new();
    let required = schema
        .get("required")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    let properties = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .cloned()
        .unwrap_or_default();

    for name in required.iter().filter_map(|v| v.as_str()) {
        let prop_schema = properties.get(name).unwrap_or(&Value::Null);
        out.insert(
            name.to_string(),
            synthesize_value(name, prop_schema, config),
        );
    }

    Value::Object(out)
}

fn synthesize_value(name: &str, schema: &Value, config: &ExplorationConfig) -> Value {
    if let Some(values) = schema.get("enum").and_then(|e| e.as_array()) {
        if let Some(first) = values.first() {
            return first.clone();
        }
    }

    let ty = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string");
    match ty {
        "boolean" => Value::Bool(true),
        "integer" => json!(1),
        "number" => json!(1.0),
        "array" => Value::Array(Vec::new()),
        "object" => synthesize_arguments(schema, config),
        _ => Value::String(string_value_for_field(name, config)),
    }
}

fn string_value_for_field(name: &str, config: &ExplorationConfig) -> String {
    let lower = name.to_lowercase();
    if lower.contains("path")
        || lower.contains("file")
        || lower.contains("filename")
        || lower.contains("directory")
        || lower.contains("dir")
        || lower.contains("workspace")
    {
        if let Some(path) = &config.file_canary_path {
            return path.clone();
        }
    }
    config.input_canary.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_config_tools_and_fills_required_args() {
        let tools = json!({
            "result": {
                "tools": [
                    {"name": "delete_file", "description": "delete a file", "inputSchema": {"type": "object"}},
                    {"name": "get_config", "description": "show environment", "inputSchema": {
                        "type": "object",
                        "required": ["path"],
                        "properties": {"path": {"type": "string"}}
                    }}
                ]
            }
        });
        let plans = build_exploration_plans(
            &tools,
            &ExplorationConfig {
                enabled: true,
                max_tools: 1,
                env_canary: "ENV_CANARY".into(),
                input_canary: "INPUT_CANARY".into(),
                file_canary_path: Some("/tmp/file-canary.txt".into()),
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].tool_name, "get_config");
        assert_eq!(plans[0].arguments["path"], "/tmp/file-canary.txt");
    }
}
