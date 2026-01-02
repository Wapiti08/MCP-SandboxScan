use serde_json::Value;
use crate::scan::prompt_sink::PromptSink;

pub fn extract_tool_return_sinks(stdout: &str) -> Vec<PromptSink> {
    let mut out = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let v: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // avoid picking up prompt/messages again 
        if v.get("prompt").is_some() || v.get("messages").is_some() {
            continue;
        }

        collect_string_leaves(&v, "$".to_string(), &mut out);
    }
    // sort by as_text for deterministic order
    out.sort_by(|a, b| a.as_text().cmp(b.as_text()));
    out
}

fn collect_string_leaves(v: &Value, path: String, out: &mut Vec<PromptSink>) {
    // dfs traversal to find string leaves - depends on Value type
    match v {
        Value::String(s) => {
            if is_candidate_leaf(&path, s) {
                out.push(PromptSink::ToolReturnLeaf { 
                    path, 
                    value: s.to_string(), 
                });
            }
        } 
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                collect_string_leaves(item, format!("{}[{}]", path, i), out);
            }
        }
        Value::Object(map) => {
            for (k, item) in map.iter() {
                collect_string_leaves(item, format!("{}.{}", path, k), out);
            }
        }
        _ => {}
    }
}

fn is_candidate_leaf(path: &str, s: &str) -> bool {
    let s = s.trim();
    if s.len() < 12 {
        return false;
    }

    // filter out metadata-ish paths
    let lower = path.to_ascii_lowercase();
    let meta = ["id", "uuid", "timestamp", "status", "code", "version"];
    if meta.iter().any(|m| lower.ends_with(&format!(".{}", m))) {
        return false;
    }

    true
}