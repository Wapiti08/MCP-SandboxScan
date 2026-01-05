#!/usr/bin/env bash
set -euo pipefail

WASM="${1:-fixtures/evil_prompt_tool/tool.wasm}"
MAX_OUT="${2:-4096}"

cargo run --quiet --bin demo -- "$WASM" "$MAX_OUT"