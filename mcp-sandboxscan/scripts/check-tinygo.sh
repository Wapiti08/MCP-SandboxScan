#!/usr/bin/env bash
set -euo pipefail

if command -v tinygo >/dev/null 2>&1; then
  echo "tinygo: $(tinygo version)"
  exit 0
fi

cat <<'EOF'
TinyGo is not installed.

Go WASI case studies in this repo default to:

  GOOS=wasip1 GOARCH=wasm go build -o tool.wasm .

TinyGo is optional. To use it instead, install TinyGo and set subject [build] to:

  tinygo build -target wasip1 -o tool.wasm .

Install options:
  https://tinygo.org/getting-started/install/
EOF

exit 1
