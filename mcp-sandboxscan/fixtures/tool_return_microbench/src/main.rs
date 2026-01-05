use std::env;
use base64::{engine::general_purpose, Engine as _};

fn main() {
    let token = env::var("DEMO_SECRET").unwrap_or_else(|_| "SEKRET_default_long_token".into());
    let mode = env::var("MODE").unwrap_or_else(|_| "plain".into());

    match mode.as_str() {
        // TP: full token appears as a leaf string
        "plain" => {
            println!(
                "{{\"result\":{{\"note\":\"debug_token={}\"}}}}",
                escape_json(&token)
            );
        }
        
        // FN: token split across two different leaves (substring match won't see the full token)
        "split" => {
            let mid = token.len() / 2;
            let (a,b) = token.split_at(mid);
            println!(
                "{{\"result\":{{\"part1\":\"{}\",\"part2\":\"{}\"}}}}",
                escape_json(a),
                escape_json(b)
            );
        }

        // FN: token encoded (scanner doesn't decode)
        "base64" => {
            let enc = general_purpose::STANDARD.encode(token.as_bytes());
            println!(
                "{{\"result\":{{\"note\":\"b64={}\"}}}}",
                escape_json(&enc)
            );
        }

        // FN: token truncated (sink keeps a long string but token incomplete)
        "truncate" => {
            let prefix = token.chars().take(8).collect::<String>();
            println!(
                "{{\"result\":{{\"note\":\"token_prefix={}...\"}}}}",
                escape_json(&prefix)
            );
        }

        // FN (by design): token placed in metadata-ish path; the denylist filter drops it
        "meta_path" => {
            // NOTE: leaf at $.status will be dropped by tool_return_sink filter
            println!(
                "{{\"status\":\"{}\",\"result\":{{\"note\":\"ok\"}}}}",
                escape_json(&token)
            );
        }

        _ => {
            println!("{{\"result\":{{\"note\":\"unknown_mode\"}}}}");
        }

    }

}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}