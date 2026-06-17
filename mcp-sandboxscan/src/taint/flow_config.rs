use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    pub min_len: usize,
    pub prefix_lens: Vec<usize>,
    pub suffix_lens: Vec<usize>,
    pub mid_window_len: usize,
    pub mid_window_enabled_threshold: usize,
    pub enable_separator_normalization: bool,
    pub enable_sink_decoders: bool,
    pub enable_rot13: bool,
    pub min_decoder_token_len: usize,
}

impl FlowConfig {
    pub fn raw_only() -> Self {
        Self {
            min_len: 4,
            prefix_lens: vec![16, 24, 32],
            suffix_lens: vec![16, 24, 32],
            mid_window_len: 24,
            mid_window_enabled_threshold: 48,
            enable_separator_normalization: false,
            enable_sink_decoders: false,
            enable_rot13: false,
            min_decoder_token_len: 8,
        }
    }

    pub fn default_matcher() -> Self {
        Self {
            min_len: 4,
            prefix_lens: vec![16, 24, 32],
            suffix_lens: vec![16, 24, 32],
            mid_window_len: 24,
            mid_window_enabled_threshold: 48,
            enable_separator_normalization: true,
            enable_sink_decoders: true,
            enable_rot13: true,
            min_decoder_token_len: 8,
        }
    }
}

pub fn make_snippets(secret: &str, config: &FlowConfig) -> Vec<String> {
    let secret = secret.trim();
    if secret.is_empty() {
        return vec![];
    }

    let bytes = secret.as_bytes();
    let mut out = vec![];

    if bytes.len() >= config.min_len {
        out.push(secret.to_string());
    }

    for &l in &config.prefix_lens {
        if bytes.len() >= l {
            out.push(String::from_utf8_lossy(&bytes[..l]).to_string());
        }
    }

    for &l in &config.suffix_lens {
        if bytes.len() >= l {
            let start = bytes.len().saturating_sub(l);
            out.push(String::from_utf8_lossy(&bytes[start..]).to_string());
        }
    }

    if bytes.len() > config.mid_window_enabled_threshold {
        let mid = bytes.len() / 2;
        let start = mid.saturating_sub(config.mid_window_len / 2);
        let end = (start + config.mid_window_len).min(bytes.len());
        out.push(String::from_utf8_lossy(&bytes[start..end]).to_string());
    }

    out.into_iter()
        .filter(|x| x.len() >= config.min_len)
        .collect()
}

pub fn detect_secret_in_sink(
    secret: &str,
    sink_text: &str,
    config: &FlowConfig,
) -> (bool, Vec<String>) {
    let snippets = make_snippets(secret, config);
    let mut strategies = Vec::new();

    let candidates = sink_candidates(sink_text, config);
    for snip in &snippets {
        if snip.len() < config.min_len {
            continue;
        }
        for (candidate, strategy) in &candidates {
            if candidate.contains(snip.as_str()) {
                if !strategies.contains(strategy) {
                    strategies.push(strategy.clone());
                }
            }
        }
    }

    (!strategies.is_empty(), strategies)
}

fn sink_candidates(sink_text: &str, config: &FlowConfig) -> Vec<(String, String)> {
    let mut out = vec![(sink_text.to_string(), "exact-substring".to_string())];

    if config.enable_separator_normalization {
        let normalized: String = sink_text.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if normalized != sink_text {
            out.push((normalized, "separator-normalized".to_string()));
        }
    }

    if !config.enable_sink_decoders {
        return out;
    }

    for (text, _) in out.clone() {
        if config.enable_rot13 {
            let decoded = rot13(&text);
            if decoded.len() >= config.min_decoder_token_len {
                out.push((decoded, "rot13-decoded".to_string()));
            }
        }

        if let Some(decoded) = try_hex_decode_tokens(&text, config.min_decoder_token_len) {
            out.push((decoded, "hex-decoded".to_string()));
        }

        if let Some(decoded) = try_base64_decode_tokens(&text, config.min_decoder_token_len) {
            out.push((decoded, "base64-decoded".to_string()));
        }
    }

    out
}

fn rot13(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
            'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
            other => other,
        })
        .collect()
}

fn try_hex_decode_tokens(text: &str, min_len: usize) -> Option<String> {
    let hex: String = text
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    if hex.len() < min_len * 2 || hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    if bytes.len() < min_len {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn try_base64_decode_tokens(text: &str, min_len: usize) -> Option<String> {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(compact.as_bytes())
        .ok()?;
    if decoded.len() < min_len {
        return None;
    }
    Some(String::from_utf8_lossy(&decoded).into_owned())
}

pub fn apply_transform(secret: &str, transform: &str) -> String {
    match transform {
        "plain" => format!("leak={secret}"),
        "prefix+suffix" => {
            let bytes = secret.as_bytes();
            let prefix = String::from_utf8_lossy(&bytes[..16.min(bytes.len())]);
            let suffix = String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(16)..]);
            format!("{prefix}...{suffix}")
        }
        "suffix-only" => {
            let bytes = secret.as_bytes();
            String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(16)..]).into_owned()
        }
        "rot13" => rot13(secret),
        "hex" => secret.as_bytes().iter().map(|b| format!("{b:02x}")).collect(),
        "base64" => base64::engine::general_purpose::STANDARD.encode(secret),
        "chunked" => secret
            .as_bytes()
            .chunks(4)
            .map(|chunk| String::from_utf8_lossy(chunk))
            .collect::<Vec<_>>()
            .join("-"),
        other => panic!("unknown transform: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_detects_rot13() {
        let secret = "SEKRET_0123456789abcdef0123456789abcdef";
        let sink = apply_transform(secret, "rot13");
        let (detected, strategies) =
            detect_secret_in_sink(secret, &sink, &FlowConfig::default_matcher());
        assert!(detected);
        assert!(strategies.iter().any(|s| s == "rot13-decoded"));
    }

    #[test]
    fn raw_only_misses_rot13() {
        let secret = "SEKRET_0123456789abcdef0123456789abcdef";
        let sink = apply_transform(secret, "rot13");
        let (detected, _) = detect_secret_in_sink(secret, &sink, &FlowConfig::raw_only());
        assert!(!detected);
    }
}
