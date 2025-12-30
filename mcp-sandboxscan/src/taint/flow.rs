// extract snippet from every source and match substrings in sink
use serde::{Deserialize, Serialize};
use crate::taint::source::TaintSource;
use crate::scan::prompt_sink::PromptSink;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMatch {
    pub source_id: String,
    pub sink_type: String,
    pub snippet: String,
    pub confidence: String, // MVP: "high"/"medium"
}

fn make_snipppets(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.is_empty() {
        return vec![];
    }

    // extract snippets of several lengths
    let bytes = s.as_bytes();
    let mut out = vec![];

    // short strings also participate match
    if bytes.len() >= 4 {
        out.push(s.to_string());
    }

    let lens = [16usize, 24, 32];
    // reverse reference to extract real lengths
    // keep values in lens
    for &l in &lens {
        if bytes.len() >= l {
            out.push(String::from_utf8_lossy(&bytes[..l]).to_string());
        }
    }

    // another middle snippet
    if bytes.len() > 48 {
        let mid = bytes.len() / 2;
        let start = mid.saturating_sub(12);
        let end = (start + 24).min(bytes.len());
        out.push(String::from_utf8_lossy(&bytes[start..end]).to_string());
    }
    // remove empty duplicates
    out.into_iter().filter(|x| !x.trim().is_empty()).collect()

}

pub fn detect_flows(sources: &[TaintSource], sinks: &[PromptSink]) -> Vec<FlowMatch> {
    let mut flows = vec![];
    let min_len = 4;
    for src in sources {
        let content = src.content();
        let snippets = make_snipppets(content);

        for sink in sinks {
            let sink_text = sink.as_text();

            for snip in &snippets {
            // any matched snippet -> flow
            if snip.len() >= min_len && sink_text.contains(snip.as_str()) {
                flows.push(FlowMatch {
                    source_id: src.short_id(),
                    sink_type: match sink {
                        PromptSink::StdoutPrompt { .. } => "StdoutPrompt".to_string(),
                        PromptSink::JsonPrompt { .. } => "JsonPrompt".to_string(),
                    },
                    snippet: snip.clone(),
                    confidence: "high".to_string(), // MVP
                });
                break;
                }
            }
        }
    }
    // deduplicate flows by source_id + sink_type
    flows.sort_by(|a, b| (a.source_id.clone(), a.sink_type.clone()).cmp(&(b.source_id.clone(), b.sink_type.clone())));
    flows.dedup_by(|a, b| a.source_id == b.source_id && a.sink_type == b.sink_type);

    flows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::prompt_sink::PromptSink;
    use crate::taint::source::TaintSource;

    #[test]
    fn detects_external_to_prompt_flow() {
        let sources = vec![
            TaintSource::EnvVar {
                key: "API_KEY".to_string(),
                value: "SECRET_TOKEN".to_string(),
            }
        ];

        let sinks = vec![
            PromptSink::StdoutPrompt{
                line: "PROMPT: use SECRET_TOKEN to authenticate".to_string(),
            }
        ];

        let flows = detect_flows(&sources, &sinks);
        assert_eq!(flows.len(), 1);
        assert!(flows[0].snippet.contains("SECRET"));
    }

    #[test]
    fn no_false_positive_when_not_present() {
        let sources = vec![
            TaintSource::EnvVar {
                key: "API_KEY".to_string(),
                value: "SECRET_TOKEN".to_string(),
            }
        ];

        let sinks = vec![
            PromptSink::StdoutPrompt {
                line: "PROMPT: hello world".to_string(),
            }
        ];

        let flows = detect_flows(&sources, &sinks);
        assert!(flows.is_empty());
    }
}