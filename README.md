# MCP-SandboxScan
A dynamic security analysis framework for MCP tools and servers, combining WASM/WASI sandboxed execution, native MCP protocol monitoring, and source-to-sink data-flow detection.

MCP-SandboxScan executes or interacts with MCP implementations, collects runtime and protocol evidence, and detects potentially unsafe flows from external inputs—including environment variables, files, and network responses—to LLM-visible outputs.

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

Core scanner (always required):

```bash
rustc --version          # stable Rust toolchain
cargo --version
rustup target add wasm32-wasip1   # Rust WASI case studies and fixtures
```

| Ecosystem | Required for | Install / verify |
|-----------|--------------|------------------|
| **Rust** | WASI tools, `rust-mcp-filesystem` | `rustup target add wasm32-wasip1` |
| **Go** | WASI + native MCP (`go-sdk`) | `go version` — ≥ 1.21 (wasip1); ≥ 1.23 (go-sdk fixtures) |
| **Python** | WASI + PyPI MCP (`fastmcp`, etc.) | `python3 --version` — ≥ 3.10; `python3 -m venv` must work |
| **Node.js / npm** | TypeScript native MCP + Javy | `node --version` — ≥ 18; `npm --version` |
| **Javy** | TypeScript WASI (`javy compile`) | see below |
| **TinyGo** | optional Go WASI backend | `./scripts/check-tinygo.sh` |

### Go

```bash
go version   # >= 1.21 for wasip1; >= 1.23 for go-sdk MCP fixtures
```

WASI builds default to `GOOS=wasip1 GOARCH=wasm go build -o tool.wasm .`. TinyGo is optional:

```bash
cd mcp-sandboxscan
./scripts/check-tinygo.sh
```

### Python

```bash
python3 --version                    # >= 3.10 recommended
python3 -m venv /tmp/venv-check      # ensure venv module works
rm -rf /tmp/venv-check
```

WASI subjects need a CPython `python.wasm` runtime (fetched once):

```bash
cd mcp-sandboxscan
./scripts/fetch-cpython-wasi.sh
# or: export MCP_SANDBOXSCAN_PYTHON_WASM=/path/to/python.wasm
```

PyPI MCP subjects run `pip install` into per-fixture `.venv` on first build; no global `pip install` required.

### TypeScript / npm

Native MCP fixtures (`ts-mcp-*`) need Node and npm. Dependencies install into `fixtures/<name>/node_modules` on first run.

```bash
node --version   # >= 18 recommended
npm --version
```

Upstream typescript-sdk examples (optional):

```bash
cd mcp-sandboxscan
./scripts/fetch-typescript-sdk-examples.sh
```

### Javy (TypeScript WASI)

TypeScript WASI case studies compile JS to `tool.wasm` with Javy:

```bash
brew install javy
javy --version   # e.g. javy-v3.x
```

Or use the repo check script:

```bash
cd mcp-sandboxscan
./scripts/check-javy.sh
```

Build command used by subjects:

```bash
javy compile -o tool.wasm main.js
```

Notes:

- Javy does not expose `process.env` or WASI file APIs to JS. For TypeScript WASI subjects in this repo,
  the scanner injects a small JSON object via **stdin** so the JS tool can simulate reading env vars and
  `/data/secret.txt`.
- The injected JSON shape is:

```json
{"env":{"DEMO_SECRET":"..."}, "files":{"/data/secret.txt":"..."}}
```

- TypeScript WASI case studies live under:
  - `case_studies/ts-benign`
  - `case_studies/ts-env-leak`
  - `case_studies/ts-file-exfil`
  - `case_studies/ts-c2-beacon`

### External assets (fetched on demand)

These are not global installs; scripts clone or download into `mcp-sandboxscan/external/` when you run upstream case studies or tests:

| Script | Purpose |
|--------|---------|
| `./scripts/fetch-cpython-wasi.sh` | CPython WASI `python.wasm` |
| `./scripts/fetch-go-sdk-examples.sh` | go-sdk `examples/server/hello` |
| `./scripts/fetch-fastmcp-examples.sh` | PrefectHQ/fastmcp examples |
| `./scripts/fetch-typescript-sdk-examples.sh` | modelcontextprotocol/typescript-sdk |

Rust `rust-mcp-filesystem` is cloned manually; see [Real Rust MCP Server](#real-rust-mcp-server).

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

## Go Ecosystem Case Studies

Go support uses the same capability-driven pipeline as Python and Rust:

- **WASI tools** (`go-benign`, `go-env-leak`, `go-file-exfil`, `go-c2-beacon`): `GoWasiAdapter` builds `GOOS=wasip1 GOARCH=wasm` artifacts and runs them in `WasiPreview1`.
- **Native MCP stdio** (`go-mcp-echo`, `go-mcp-env-leak`, `go-mcp-c2-beacon`): `mcp-protocol` capability routes to `NativeMcpAdapter`; servers use [modelcontextprotocol/go-sdk](https://github.com/modelcontextprotocol/go-sdk).
- **Upstream go-sdk** (`go-mcp-upstream-hello`): official `examples/server/hello` from go-sdk `v1.1.0` under `external/go-sdk`.

Build a Go WASI subject:

```bash
cd mcp-sandboxscan
cargo run --bin mcp-sandboxscan -- --subject case_studies/go-env-leak/subject.toml --env DEMO_SECRET=SEKRET_0123456789abcdef
```

Run Go native MCP integration tests:

```bash
cd mcp-sandboxscan
cargo test --lib mcp::native_stdio::tests::go:: -- --nocapture
cargo test --lib scans_go_mcp_ -- --nocapture
```

Upstream go-sdk hello example (fetched once into `external/go-sdk/`):

```bash
cd mcp-sandboxscan
./scripts/fetch-go-sdk-examples.sh
cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/go-mcp-upstream-hello/subject.toml
```

Run upstream go-sdk tests:

```bash
cd mcp-sandboxscan
cargo test --lib driver_calls_upstream_go_sdk_hello -- --nocapture
cargo test --lib scans_go_mcp_upstream_hello_subject -- --nocapture
```

WASI builds default to:

```bash
GOOS=wasip1 GOARCH=wasm go build -o tool.wasm .
```

TinyGo (`tinygo build -target wasip1 -o tool.wasm .`) is optional; see `scripts/check-tinygo.sh`.

## Python / PyPI Ecosystem Case Studies

Python support mirrors Rust and Go with two execution paths:

- **WASI tools** (`python-benign`, `python-env-leak`, `python-file-exfil`): `PythonWasiAdapter` bundles scripts with the CPython WASI runtime and runs them in `WasiPreview1`.
- **Native MCP stdio via PyPI** (`python-mcp-server-fetch`, `python-fastmcp-*`): `mcp-protocol` capability routes to `NativeMcpAdapter`; each subject creates a local `.venv` and `pip install`s wheels from PyPI (`fastmcp`, `mcp-server-fetch`, etc.).

### CPython WASI runtime (WASI path)

WASI subjects need a `python.wasm` interpreter. Fetch the default build:

```bash
cd mcp-sandboxscan
./scripts/fetch-cpython-wasi.sh
```

Or point to an existing runtime:

```bash
export MCP_SANDBOXSCAN_PYTHON_WASM=/path/to/python.wasm
```

Run a Python WASI subject:

```bash
cd mcp-sandboxscan
cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-env-leak/subject.toml \
  --env DEMO_SECRET=SEKRET_0123456789abcdef

# file-exfil needs a mounted data directory
mkdir -p data && echo "top-secret" > data/secret.txt
cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-file-exfil/subject.toml \
  --data-dir ./data
```

Rust vs Python portability matrix (6 subjects):

```bash
cd mcp-sandboxscan
chmod +x demo/run_rust_python_matrix.sh
DATA_DIR="$(pwd)/data" ./demo/run_rust_python_matrix.sh
```

### PyPI native MCP servers

PyPI-based subjects install dependencies into `fixtures/<name>/.venv` or `external/fastmcp/examples/.venv` on first run. Venv directories are gitignored.

| Case study | PyPI package | Role |
|------------|--------------|------|
| `python-mcp-server-fetch` | `mcp-server-fetch` | Network fetch tool; egress proxy records blocked outbound |
| `python-fastmcp-echo` | `fastmcp` | Benign echo tool |
| `python-fastmcp-env-leak` | `fastmcp` | Env secret leaks into tool result |
| `python-fastmcp-c2-beacon` | `fastmcp` | C2 beacon via HTTP; egress intercepted |
| `python-fastmcp-upstream-echo` | `fastmcp` | Upstream [PrefectHQ/fastmcp](https://github.com/PrefectHQ/fastmcp) `examples/simple_echo.py` |

Run a PyPI MCP subject (build step runs `pip install` automatically):

```bash
cd mcp-sandboxscan
cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-fastmcp-echo/subject.toml

cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-fastmcp-env-leak/subject.toml \
  --env DEMO_SECRET=SEKRET_0123456789abcdef

cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-mcp-server-fetch/subject.toml
```

Upstream FastMCP examples (fetched once into `external/fastmcp/examples/`):

```bash
cd mcp-sandboxscan
./scripts/fetch-fastmcp-examples.sh
cargo run --bin mcp-sandboxscan -- \
  --subject case_studies/python-fastmcp-upstream-echo/subject.toml
```

Run Python / PyPI integration tests:

```bash
cd mcp-sandboxscan
cargo test --lib mcp::native_stdio::tests::python:: -- --nocapture
cargo test --lib pipeline::tests::scans_python_mcp_server_fetch_subject -- --nocapture
cargo test --lib pipeline::tests::scans_python_fastmcp_echo_subject -- --nocapture
cargo test --lib pipeline::tests::scans_python_fastmcp_env_leak_subject -- --nocapture
cargo test --lib pipeline::tests::scans_python_fastmcp_c2_beacon_subject -- --nocapture
cargo test --lib pipeline::tests::scans_python_fastmcp_upstream_echo_subject -- --nocapture

# WASI subjects (requires CPython WASI runtime)
cargo test --lib pipeline::tests::scans_python_env_leak_subject -- --nocapture --ignored
```

Note: the PyPI path is native MCP protocol testing (stdio JSON-RPC), not WASM sandbox execution. It validates MCP-level monitoring, egress proxying, and report generation for real Python MCP servers installed from PyPI.
