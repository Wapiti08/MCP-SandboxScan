# Benchmark

The release includes a labeled smoke benchmark to make detection claims reproducible
and appropriately scoped. The checked-in result is
[`mcp-sandboxscan/reports/bench-smoke-test/summary.md`](https://github.com/Wapiti08/MCP-SandboxScan/blob/v0.1.0-alpha.1/mcp-sandboxscan/reports/bench-smoke-test/summary.md).
Release archives also include the Markdown and JSON summaries under `benchmark/`.

## v0.1.0-alpha.1 smoke result

| Suite | Cases | Scan success | TP / FP / TN / FN | Precision | Recall | F1 |
|---|---:|---:|---:|---:|---:|---:|
| `small-ts` | 4 | 4/4 (100%) | 3 / 0 / 1 / 0 | 1.000 | 1.000 | 1.000 |

The suite contains one benign case and controlled environment-leak, file-exfiltration,
and C2-beacon cases. These are synthetic fixtures with an oracle label. The numbers do
not estimate performance on arbitrary real-world MCP servers and should not be read as
a vulnerability rate.

## Reproduce

Install Javy and the Rust toolchain, then run from the repository root:

```bash
cargo run --locked --release --bin bench -- \
  --suite small-ts \
  --out-dir mcp-sandboxscan/reports/bench-v0.1.0-alpha.1
```

The output directory contains `summary.json`, `summary.md`, and optional per-case
`ScanReport` files. Use the machine-readable summary when comparing revisions.

For the larger but environment-dependent matrix:

```bash
cargo run --locked --release --bin bench -- \
  --suite full \
  --out-dir mcp-sandboxscan/reports/bench-full-v0.1.0-alpha.1
```

The full suite additionally needs Go, Python, Node.js/npm, CPython WASI assets, and any
native fixture dependencies documented in the README. Record tool versions and failed
cases when publishing results.

These ecosystem tests are intentionally separate from the environment-independent CI
unit tests because they may install packages, open a local egress-observation socket,
or require external runtimes. Run the labeled benchmark in a disposable environment
before publishing updated performance claims.
