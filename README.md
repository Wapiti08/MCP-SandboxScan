# MCP-SandboxScan
A WASM-based Secure Execution and Hybrid Analysis Framework for MCP Tools


## What it detects (MVP)
- Prompt sinks: stdout `PROMPT:` lines, JSON `prompt/messages`
- External sources: env vars, file contents under `/data`, HTTP fetch intents in output
- External-to-prompt flows: string-level snippet matches between sources and sinks

## Dependency
```
cargo add cap_std
```

## Run
```bash
cargo run -- \
  --wasm ./examples/tool.wasm \
  --data-dir ./data \
  --env API_KEY=secret \
  --env MODEL=gpt-4
```
## For Unit Test (files under tests)
```
cargo test xxx
```

## Structure

- main.rs: entrypoint of scanner

- sandbox

    - exec_result.rs: execution result (stdout/stderr/exit/sources)
    - wasi_hooks.rs:  source collection of env/file/http intent
    - wasm_runner.rs: wasm sandbox

- scan:
    
    - dynamic_scan.rs: orchestrator from execute to scan to report
    - prompt_sink.rs: prompt/messages/stdout sink
    - report.rs 

- taint:

    - flow.rs    source to sink at string-level
    - source.rs   define source