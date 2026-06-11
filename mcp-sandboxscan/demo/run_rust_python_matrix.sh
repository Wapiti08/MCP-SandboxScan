#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

DATA_DIR="${DATA_DIR:-$(pwd)/data}"
mkdir -p "${DATA_DIR}"
echo "top-secret" > "${DATA_DIR}/secret.txt"

if [[ ! -f external/cpython-wasi/python.wasm ]]; then
  echo "Fetching CPython WASI runtime..."
  ./scripts/fetch-cpython-wasi.sh
fi

cargo run -q --bin mcp-sandboxscan -- --study \
  case_studies/rust-benign/subject.toml \
  case_studies/rust-env-leak/subject.toml \
  case_studies/rust-file-exfil/subject.toml \
  case_studies/python-benign/subject.toml \
  case_studies/python-env-leak/subject.toml \
  case_studies/python-file-exfil/subject.toml \
  --env "DEMO_SECRET=${DEMO_SECRET:-SEKRET_0123456789abcdef}" \
  --data-dir "${DATA_DIR}"
