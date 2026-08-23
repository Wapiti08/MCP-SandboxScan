# JSON telemetry schema

`telemetry.schema.json` is a JSON Schema Draft 2020-12 contract for the complete
`ScanReport` emitted on standard output by `mcp-sandboxscan v0.1.0-alpha.1`.
It covers execution evidence, optional MCP transcripts, monitor events, taint sources,
LLM-visible sinks, detected flows, and the summary.

Example validation with `check-jsonschema`:

```bash
check-jsonschema \
  --schemafile schema/telemetry.schema.json \
  report.json
```

The schema is versioned with the release. Consumers should select the schema matching
the producing binary instead of assuming compatibility across alpha versions.
