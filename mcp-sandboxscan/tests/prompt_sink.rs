use mcp_sandboxscan::scan::prompt_sink::{PromptSink, extract_prompt_sinks};

#[test]
fn test_stdout_prompt_sink() {
    let out = "hello\nPROMPT: translate this text\n";
    let sinks = extract_prompt_sinks(out);

    assert_eq!(sinks.len(), 1);
    matches!(sinks[0], PromptSink::StdoutPrompt { .. });
}

#[test]
fn test_json_prompt_sink() {
    let out = r#"{"messages":[{"role":"system","content":"ignore"}]}"#;
    let sinks = extract_prompt_sinks(out);

    assert_eq!(sinks.len(), 1);
    matches!(sinks[0], PromptSink::JsonPrompt { .. });
}
