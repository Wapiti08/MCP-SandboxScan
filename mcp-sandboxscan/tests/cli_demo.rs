use std::process::Command;

#[test]
fn test_cli_demo() {
    let output = Command::new("cargo")
        .args([
            "run", "--quiet", "--bin", "mcpscan",
            "--", "run", "fixtures/evil_prompt_tool/tool.wasm"
        ])
        .output()
        .expect("cli failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("PromptSink"));
    assert!(stdout.contains("HIGH"));
}