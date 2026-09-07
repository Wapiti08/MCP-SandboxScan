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

### Paired shallow and targeted runtime evidence (RQ4)

The checked-in corpus runs provide a shallow baseline for the targeted results.
Pair repositories by `repo_id` and include only repositories with `scan_ok: true`
in both modes. A flow repository has at least one recorded witness; witness counts
use `num_flows`, checked against the per-case `flows` arrays.

| Experiment | Common successful repos | Shallow flow repos / witnesses | Targeted flow repos / witnesses | Net increase repos / witnesses |
|---|---:|---:|---:|---:|
| Original | 33 | 0 / 0 | 12 / 60 | +12 / +60 |
| Supplementary | 27 | 0 / 0 | 10 / 48 | +10 / +48 |

Original sources: [shallow](mcp-sandboxscan/reports/corpus-run-1781854580/corpus_summary.json)
and [targeted](mcp-sandboxscan/reports/corpus-targeted-scan-ok-1781854580-v2/corpus_summary.json).
Shallow succeeded on 35 repositories and targeted on 33. The paired comparison
excludes `21st-dev/magic-mcp` and `executeautomation/mcp-playwright`, which failed
in targeted mode.

Supplementary sources: [shallow](mcp-sandboxscan/reports/budget-shallow-final/corpus_summary.json)
and [targeted](mcp-sandboxscan/reports/budget-targeted-final/corpus_summary.json).
Success counts were 28/35 and 27/35 respectively. The successful sets differ only
by `21st-dev/magic-mcp`, which is excluded from the paired comparison.

Among the 33 repositories successfully scanned in both modes in the original
experiment, shallow scanning yielded no witnesses, whereas targeted scanning
yielded 60 witnesses across 12 repositories, providing additional runtime evidence
within this matched subset. The supplementary experiment showed the same pattern:
zero shallow witnesses versus 48 targeted witnesses across 10 of the 27 common
successful repositories. These are observed runtime witnesses, not verified
vulnerabilities. The comparison describes these recorded runs and does not isolate
scan depth from every possible environment or version difference between runs.

Recompute the table and validate the underlying per-case counts without rerunning
the servers:

```bash
python3 scripts/compare-runtime-evidence.py
```

### Smoke benchmark

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
