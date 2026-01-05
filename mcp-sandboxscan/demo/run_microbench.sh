#!/usr/bin/env bash
set -euo pipefail

# Ensure run from repo root even if you execute from demo/
cd "$(dirname "$0")/.."

LONG_TOKEN="SEKRET_0123456789abcdef0123456789abcdef"
SHORT_TOKEN="abcd"

MICRO_WASM="fixtures/tool_return_microbench/tool.wasm"
NOISE_WASM="fixtures/tool_return_noise/tool.wasm"

run_demo () {
  local wasm="$1"
  shift
  env "$@" cargo run --quiet --bin demo -- "$wasm"
}

print_row () {
  local name="$1"
  local out="$2"

  # remove possible Windows CRLF
  out="$(printf "%s" "$out" | tr -d '\r')"

  local flows has_flow sinks
  flows="$(printf "%s\n" "$out" | awk -F': ' '/^[[:space:]]*flows: /{print $2; exit}')"
  has_flow="$(printf "%s\n" "$out" | awk -F': ' '/^[[:space:]]*has_external_to_prompt_flow: /{print $2; exit}')"
  sinks="$(printf "%s\n" "$out" | awk -F': ' '/^[[:space:]]*sinks: /{print $2; exit}')"

  flows="${flows:-NA}"
  has_flow="${has_flow:-NA}"
  sinks="${sinks:-NA}"

  echo -e "${name}\tflows=${flows}\thas_flow=${has_flow}\tsinks=${sinks}"
}

echo -e "CASE\tflows\thas_flow\tsinks"

out="$(run_demo "$MICRO_WASM" "DEMO_SECRET=$LONG_TOKEN" "MODE=plain")"
print_row "TP_plain" "$out"

out="$(run_demo "$MICRO_WASM" "DEMO_SECRET=$LONG_TOKEN" "MODE=split")"
print_row "ROB_split_partial_leak" "$out"

out="$(run_demo "$MICRO_WASM" "DEMO_SECRET=$LONG_TOKEN" "MODE=base64")"
print_row "FN_base64" "$out"

out="$(run_demo "$MICRO_WASM" "DEMO_SECRET=$LONG_TOKEN" "MODE=truncate")"
print_row "FN_truncate" "$out"

out="$(run_demo "$MICRO_WASM" "DEMO_SECRET=$LONG_TOKEN" "MODE=meta_path")"
print_row "FN_meta_path" "$out"

out="$(run_demo "$NOISE_WASM" "DEMO_SECRET=$SHORT_TOKEN")"
print_row "FP_short_token_collision" "$out"

out="$(run_demo "$NOISE_WASM" "DEMO_SECRET=$LONG_TOKEN")"
print_row "FP_long_token" "$out"

out="$(run_demo "$NOISE_WASM")"
print_row "FP_no_env" "$out"
