# Threat model

This document defines the security boundary of MCP-SandboxScan `v0.1.0-alpha.1`.

## System and trust assumptions

MCP-SandboxScan analyzes tools that can receive attacker-influenced arguments and can
access sensitive environment variables, files, or network data. The scanner and its
configuration are trusted. Tool inputs, remote content, and the analyzed subject are
not trusted. The host running a native subject is inside that subject's security
boundary unless the operator adds OS- or VM-level isolation.

The LLM is treated as influenceable rather than inherently malicious: direct or
indirect prompt injection may cause it to send dangerous tool arguments or expose a
tool result to subsequent model context.

## Assets and security goals

Protected assets include secrets passed with `--env`, files exposed with `--data-dir`,
the integrity of the analyst host, and the confidentiality of data returned to an LLM.

The scanner has three goals:

1. Detect observed flows from sensitive sources to LLM-visible sinks.
2. Contain WASI subjects using explicit preopened directories and denied network
   access while recording relevant evidence.
3. Produce auditable execution, MCP transcript, event, source, sink, and flow records.

## Threats in scope

| Threat | Example | Detection or mitigation |
|---|---|---|
| Environment disclosure | A tool returns an API key after an injected request | `EnvVar` source to tool-result or prompt sink flow |
| File disclosure | A tool reads `/data/secret.txt` into its result | `FileRead` source to tool-result flow |
| Network exfiltration | A tool connects to a C2 or exfiltration endpoint | network attempt/request telemetry and flow evidence |
| Unsafe host access during WASI analysis | A module tries arbitrary file or socket access | capability-limited WASI execution and denied network attempts |
| Confused deputy behavior | A legitimate server is induced to misuse authorized access | adversarial case studies plus MCP-aware result inspection |

## Execution boundaries

| Property | WASI subject | Native MCP stdio subject |
|---|---|---|
| Host filesystem | Only configured preopens | OS user permissions; not contained by SandScope |
| Network | Denied by default and observed | Proxy-based observation; not a complete network sandbox |
| Subprocesses | Not generally available | May be available under OS user permissions |
| MCP fidelity | Tool-like stdout/JSON sinks | Full JSON-RPC handshake and `tools/call` results |
| Flow analysis | String-level source-to-sink matching | Same analysis, with MCP result sinks |

Run unknown native subjects only inside a disposable container or VM with minimal
credentials and network access. The project Docker image isolates the scanner process;
it does not automatically containerize a child server mounted or installed at runtime.

## Non-goals and known limitations

- Proving that a subject is safe or that no data flow exists.
- Detecting every encoded, encrypted, fragmented, or semantically transformed leak.
- Preventing timing, resource-exhaustion, or other side channels.
- Replacing production MCP authorization, least privilege, egress policy, or review.
- Providing kernel-level isolation for native MCP servers.
- Treating every detected flow as a vulnerability; benign echo behavior needs review.

Telemetry can contain secret values, captured stdout/stderr, tool arguments, and MCP
payloads. Store reports as sensitive security artifacts, redact before sharing, and do
not upload them to public issue trackers without inspection.

## Reporting security issues

Do not include live credentials or private scan reports in a public issue. Use the
repository's private vulnerability-reporting channel when enabled, or contact the
maintainers privately before disclosure.
