# MCP-SandboxScan
A WASM-based Secure Execution and Hybrid Analysis Framework for MCP Tools (paper coming soon)


## What it detects (MVP)
- Prompt sinks: stdout `PROMPT:` lines, JSON `prompt/messages`
- External sources: env vars, file contents under `/data`, HTTP fetch intents in output
- External-to-prompt flows: string-level snippet matches between sources and sinks

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