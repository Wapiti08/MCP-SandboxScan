# MCP-SandboxScan v0.1.0-alpha.1

This first alpha release packages the MCP-aware dynamic scanner for early testing.

Highlights:

- WASM/WASI sandboxed scans and native MCP stdio monitoring
- structured source, sink, flow, execution, and protocol telemetry
- labeled evaluation suites across Rust, Go, Python, and TypeScript
- binaries for Linux, macOS, and Windows
- `linux/amd64` and `linux/arm64` image at `ghcr.io/wapiti08/mcp-sandboxscan:v0.1.0-alpha.1`

This is an alpha. Review the [threat model](https://github.com/Wapiti08/MCP-SandboxScan/blob/v0.1.0-alpha.1/THREAT_MODEL.md), especially the native-execution and detection limitations, before scanning untrusted servers.

See the [changelog](https://github.com/Wapiti08/MCP-SandboxScan/blob/v0.1.0-alpha.1/CHANGELOG.md) for the complete release summary.
