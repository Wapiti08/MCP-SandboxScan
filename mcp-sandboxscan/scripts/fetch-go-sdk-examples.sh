#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/external/go-sdk"
MARKER="${DEST}/examples/server/hello/main.go"
TAG="${GO_SDK_TAG:-v1.1.0}"

if [[ -f "${MARKER}" ]]; then
  echo "go-sdk examples already present at ${DEST}/examples/server/hello"
  exit 0
fi

mkdir -p "${ROOT}/external"
rm -rf "${DEST}"

echo "Cloning modelcontextprotocol/go-sdk (${TAG})"
git clone --depth 1 --branch "${TAG}" \
  https://github.com/modelcontextprotocol/go-sdk.git "${DEST}"

echo "Installed upstream go-sdk under ${DEST}"
