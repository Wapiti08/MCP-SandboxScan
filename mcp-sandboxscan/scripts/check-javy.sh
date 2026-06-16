#!/usr/bin/env bash
set -euo pipefail

if command -v javy >/dev/null 2>&1; then
  echo "javy: $(javy --version 2>&1)"
  exit 0
fi

cat <<'EOF'
Javy is not installed.

TypeScript WASI case studies in this repo use:

  javy compile -o tool.wasm main.js

Install:

  brew install javy
  # or download from the Javy releases page
EOF

exit 1
