# Bench Report: run-1781774525

- **Suite**: `small-ts`
- **Scan success rate**: 100.0% (4/4)

## Detection (oracle)

| Metric | Value |
|--------|-------|
| TP | 3 |
| FN | 0 |
| FP | 0 |
| TN | 1 |
| Errors | 0 |
| Precision | 1.000 |
| Recall | 1.000 |
| F1 | 1.000 |

## Support by language

| Language | Total | Scanned | Failed | DirectWasm | WasmShim | NativeOnly |
|----------|-------|---------|--------|------------|----------|------------|
| TypeScript | 4 | 4 | 0 | 4 | 0 | 0 |

## Verification

- Protocol vs stdout disagreements: 3
- Scan errors: 0

## Cases

| Subject | Scenario | WASM | Verdict | Flows | Rationale |
|---------|----------|------|---------|-------|----------|
| ts-benign | Benign | DirectWasm | NotDetected | 0 | no flows (expected) |
| ts-env-leak | EnvLeak | DirectWasm | Detected | 1 | flow from EnvVar: DEMO_SECRET |
| ts-file-exfil | FileExfil | DirectWasm | Detected | 1 | flow from secret.txt |
| ts-c2-beacon | C2Beacon | DirectWasm | Detected | 1 | network source with beacon evidence in flows or sinks |
