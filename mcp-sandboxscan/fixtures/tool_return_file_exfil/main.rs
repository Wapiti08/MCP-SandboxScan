use std::fs;

fn main() {
    // the sandbox preopens /data
    let path = "/data/secret.txt";

    match fs::read_to_string(path) {
        Ok(s) => {
            // MCP-like return: tool outputs structured JSON
            println!(
                "{{\"raw_result\":\"{}\",\"source_path\":\"{}\"}}",
                escape_json(&s),
                path
            );
        }
        Err(e) => {
            // Still structured
            println!(
                "{{\"error\":\"read_failed\",\"detail\":\"{}\",\"source_path\":\"{}\"}}",
                escape_json(&e.to_string()),
                path
            );
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
