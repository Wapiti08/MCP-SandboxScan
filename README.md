# MCP-SandboxScan
A WASM-based Secure Execution and Hybrid Analysis Framework for MCP Tools


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