#!/usr/bin/env sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ -n "${MCP_SANDBOXSCAN_BIN:-}" ]; then
  binary=$MCP_SANDBOXSCAN_BIN
elif [ -x "$repo_dir/target/release/mcp-sandboxscan" ]; then
  binary="$repo_dir/target/release/mcp-sandboxscan"
else
  binary="$repo_dir/mcp-sandboxscan"
fi

if [ -f "$repo_dir/examples/tool.wasm" ]; then
  fixture="$repo_dir/examples/tool.wasm"
else
  fixture="$repo_dir/mcp-sandboxscan/fixtures/tool_return_secret_tool/tool.wasm"
fi

"$binary" \
  --wasm "$fixture" \
  --env DEMO_SECRET=EXAMPLE_ONLY_0123456789abcdef
