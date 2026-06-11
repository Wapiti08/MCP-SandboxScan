# MCP-SandboxScan
A WASM-based Secure Execution and Hybrid Analysis Framework for MCP Tools (paper coming soon)

## Features

- WASM/WASI execution for sandbox-compatible MCP-like tools.
- Source collection from environment variables, files under `/data`, and HTTP fetch intents.
- LLM-facing sink extraction from stdout `PROMPT:` lines, JSON `prompt/messages`, tool-return JSON leaves, and MCP `tools/call` text results.
- String-level external-to-sink flow detection.
- Subject-based case study pipeline via `subject.toml`.
- Multi-case study matrix generation with `--study`.
- Real MCP stdio protocol smoke testing with `initialize`, `notifications/initialized`, and `tools/call`.
- Two-layer evidence model:
  - execution evidence: backend, stdout/stderr, exit code, duration
  - MCP protocol evidence: transcript of client/server JSON-RPC messages


## Structure (core modules under src)

- cli: entrypoint of command, parse parameters and running mode

    - main.rs: entrypoint of scanner

- subject: describe tested MCP server

    - language.rs   # Rust/Go/Python/TypeScript
    - capability.rs # fs/ env/ network / subprocess/ database
    - manifest.rs  # 

- adapter: multi-language adapter, try to convert subject to executable artifact

    - rust_wasi.rs: Rust -> wasm32-wasip1
    - go_wasi.rs: Go -> WASI/TinyGo
    - python_wasi.rs: Python + runtime/shim
    - typescript_wasi.rs: Node/JS runtime or javy/componentize
    - unsupported.rs: the reasons for not supported

- mcp: This layer speaks or represents the MCP protocol.

    It handles initialize, notifications/initialized, tools/list, tools/call, resources/read, prompts/get


- sandbox: execute artifacts and collect stdout/stderr/exit code/etc

    - exec_result.rs: execution result (stdout/stderr/exit/sources)
    - wasi_hooks.rs:  source collection of env/file/http intent
    - wasm_runner.rs: wasm sandbox

- scan: extract sink from execution results and generate check report
    
    - dynamic_scan.rs: orchestrator from execute to scan to report
    - prompt_sink.rs: prompt/messages/stdout sink
    - report.rs 

- taint: taint analysis layer

    - flow.rs    source to sink at string-level
    - source.rs   define source

- attack: attack simulation layer, define and run attack scenarios

    - scenario.rs: prompt injection / env leak / file exfiltration
    - runner.rs: run attack for subject

- study: experiment design for evaluation

## Dependency
```
cargo add cap_std
# build tool.wasm
rustup target add wasm32-wasip1 || true

```

## Demo
```
## give one example for evil_prompt_tool
cd fixtures/evil_prompt_tool
## compile main.rs -> tool.wasm
rustc main.rs -O --target wasm32-wasip1 -o tool.wasm

## go to root folder
cd ../..
cargo run --bin demo -- fixtures/evil_prompt_tool/tool.wasm

## for other demos, after compiling tool.wasm, go through the same process
cargo run --bin demo -- fixtures/benign_tool/tool.wasm
cargo run --bin demo -- fixtures/fs_violation_tool/tool.wasm

## ========= dynamic and static comparison =========
chmod +x demo/run_static_vs_dynamic.sh 
DATA_DIR="$(pwd)/data" ./demo/run_static_vs_dynamic.sh


## ========== microbench ============
cd fixtures/tool_return_microbench
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1

# cp generate wasm to the current folder
cp target/wasm32-wasip1/release/tool_return_microbench.wasm tool.wasm
# run the following command to trigger desired behavior - for plain mode
DEMO_SECRET="SEKRET_0123456789abcdef0123456789abcdef" \
MODE="plain" \
cargo run --bin demo -- fixtures/tool_return_microbench/tool.wasm

## to run all benchmark
chmod +x demo/run_microbench.sh
./demo/run_microbench.sh

```

## For Unit Test (files under tests)
```
cargo test xxx
```

## Run
```bash
cargo run --bin mcp-sandboxscan -- \
  --wasm ./fixtures/evil_prompt_tool/tool.wasm \
  --env USER_INPUT=hello \
  --env API_KEY=secret
```

## Real Rust MCP Server

The current real MCP smoke test uses `rust-mcp-filesystem` under:

```text
mcp-sandboxscan/external/rust-mcp-filesystem
```

If the external server is not present yet:

```bash
cd mcp-sandboxscan
mkdir -p external
git clone https://github.com/rust-mcp-stack/rust-mcp-filesystem external/rust-mcp-filesystem
```

Build the real Rust MCP server:

```bash
cd mcp-sandboxscan/external/rust-mcp-filesystem
cargo build --release
```

Run the real MCP stdio smoke test and print the JSON report:

```bash
cd mcp-sandboxscan
cargo test --lib mcp::native_stdio::tests::native_stdio_driver_calls_real_rust_mcp_filesystem -- --nocapture
```

This test runs:

```text
rust-mcp-filesystem
  -> initialize
  -> notifications/initialized
  -> tools/call list_allowed_directories
  -> MCP tool result sink extraction
  -> ScanReport JSON output
```

Expected report shape:

```json
{
  "exec": {
    "backend": "native-stdio",
    "exit_code": null
  },
  "mcp_transcript": {
    "events": []
  },
  "sinks": [
    {
      "type": "McpToolResultText"
    }
  ],
  "summary": {
    "num_sinks": 1,
    "num_flows": 0,
    "has_external_to_prompt_flow": false
  }
}
```

Note: this path is native MCP protocol testing, not WASM sandbox execution. It validates MCP-level monitoring and report generation for a real Rust MCP server.
