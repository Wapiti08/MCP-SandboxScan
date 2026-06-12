#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/external/fastmcp"
MARKER="${DEST}/examples/simple_echo.py"

if [[ -f "${MARKER}" ]]; then
  echo "FastMCP examples already present at ${DEST}/examples"
  exit 0
fi

mkdir -p "${ROOT}/external"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

echo "Cloning PrefectHQ/fastmcp (sparse: examples/)"
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/PrefectHQ/fastmcp.git "${TMP}/fastmcp"
(
  cd "${TMP}/fastmcp"
  git sparse-checkout set examples
)

rm -rf "${DEST}"
mv "${TMP}/fastmcp" "${DEST}"
echo "Installed upstream FastMCP examples under ${DEST}/examples"
