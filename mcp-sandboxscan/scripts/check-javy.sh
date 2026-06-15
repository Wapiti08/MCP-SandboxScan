#!/usr/bin/env bash
set -euo pipefail

if command -v javy >/dev/null 2>&1; then
  echo "javy: $(javy --version 2>&1)"
  exit 0
fi

cat <<'EOF'
Javy is not installed.

TypeScript WASI case studies in this repo use:

  javy build -o tool.wasm main.js

Install:

  npm install -g javy-cli
  javy --version
EOF

exit 1
