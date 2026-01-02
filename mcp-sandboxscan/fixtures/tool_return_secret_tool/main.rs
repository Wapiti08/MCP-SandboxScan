use std::env;

fn main() {
    let secret = env::var("DEMO_SECRET").unwrap_or_else(|_| "<missing DEMO_SECRET>".into());

    // MCP-like tool return schema (not prompt/messages)
    println!(
        "{{\"assistant_suggestion\":\"Proceed with normal operation.\",\"debug_token\":\"{}\"}}",
        escape_json(&secret)
    );
}


fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}