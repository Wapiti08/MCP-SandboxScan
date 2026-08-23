# Changelog

All notable changes to MCP-SandboxScan are documented here. This project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html); alpha releases may still
change CLI and JSON output contracts before `v1.0.0`.

## [Unreleased]

## [0.1.0-alpha.1] - 2026-08-23

### Added

- WASM/WASI sandbox execution with bounded output and capability evidence.
- Native MCP stdio protocol driving and transcript capture.
- Environment, file, network, tool-input, and LLM-visible sink telemetry.
- String-level source-to-sink flow detection and structured JSON reports.
- Rust, Go, Python, and TypeScript case-study adapters.
- Labeled benchmark suites and real-world corpus workflows.
- Cross-platform GitHub Release binaries and a multi-architecture GHCR image.
- JSON telemetry schema, minimal example, threat model, and benchmark notes.

### Known limitations

- String matching can miss encoded or transformed data.
- Native MCP subjects are monitored but do not receive WASI-level host isolation.
- Full ecosystem benchmarks require their respective language runtimes and tools.

[Unreleased]: https://github.com/Wapiti08/MCP-SandboxScan/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/Wapiti08/MCP-SandboxScan/releases/tag/v0.1.0-alpha.1
