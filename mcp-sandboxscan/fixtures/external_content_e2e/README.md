# External-content end-to-end experiment

This controlled experiment covers the complete chain requested by the threat
model:

```text
malicious GitHub issue
  -> controlled agent surrogate derives an MCP tools/call
  -> MCP server reads an environment canary
  -> canary appears in the MCP tool result
  -> SandScope reports the environment-to-tool-result flow
```

Run it from the repository root:

```bash
cargo run --locked --bin external_content_e2e
```

To retain the machine-readable artifact:

```bash
cargo run --locked --bin external_content_e2e -- \
  --output mcp-sandboxscan/reports/external-content-e2e.json
```

The default scenario is `github_issue_env_leak.json`. It invokes a
dependency-free local Python MCP fixture; it does not access GitHub, use real
credentials, or make network requests.

## Interpretation boundary

The driver is deliberately deterministic. It recognizes the controlled phrase
`MCP tool \`NAME\` with arguments \`JSON\`` in the untrusted issue body and turns
it into an MCP call. This isolates and reproduces the security consequence of
following an external instruction. It does **not** estimate the probability
that a production LLM follows prompt injection. The report records this limit
in `scope_note`.

Environment authority use is inferred from the unique canary appearing in the
tool result and the corresponding detected flow; it is not a syscall-level
environment-read trace.
