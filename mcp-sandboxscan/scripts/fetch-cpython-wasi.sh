#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="${ROOT}/external/cpython-wasi"
DEST="${DEST_DIR}/python.wasm"
PYTHON_VERSION="${CPYTHON_WASI_VERSION:-3.13.13}"
WASI_SDK="${CPYTHON_WASI_SDK:-24}"

mkdir -p "${DEST_DIR}"

if [[ -f "${DEST}" ]]; then
  echo "CPython WASI runtime already present at ${DEST}"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

ZIP="python-${PYTHON_VERSION}-wasi_sdk-${WASI_SDK}.zip"
URL="https://github.com/brettcannon/cpython-wasi-build/releases/download/v${PYTHON_VERSION}/${ZIP}"

echo "Downloading ${URL}"
if ! curl -fsSL "${URL}" -o "${TMP}/${ZIP}"; then
  cat >&2 <<EOF
Failed to download CPython WASI runtime.

Set MCP_SANDBOXSCAN_PYTHON_WASM to an existing python.wasm, or download a release from:
  https://github.com/brettcannon/cpython-wasi-build/releases
and extract it into:
  ${DEST_DIR}
EOF
  exit 1
fi

unzip -q "${TMP}/${ZIP}" -d "${TMP}/extract"
SRC="$(find "${TMP}/extract" -name 'python.wasm' -type f | head -n 1)"
if [[ -z "${SRC}" ]]; then
  echo "python.wasm not found in ${ZIP}" >&2
  exit 1
fi

EXTRACT_ROOT="$(dirname "${SRC}")"
cp -R "${EXTRACT_ROOT}/." "${DEST_DIR}/"
echo "Installed CPython WASI runtime under ${DEST_DIR}"
