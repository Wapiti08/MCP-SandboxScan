#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/external/typescript-sdk"
MARKER="${DEST}/package.json"
TAG="${TYPESCRIPT_SDK_TAG:-v1.29.0}"

if [[ -f "${MARKER}" ]]; then
  echo "typescript-sdk already present at ${DEST}"
  exit 0
fi

mkdir -p "${ROOT}/external"
rm -rf "${DEST}"

echo "Cloning modelcontextprotocol/typescript-sdk (${TAG})"
git clone --depth 1 --branch "${TAG}" \
  https://github.com/modelcontextprotocol/typescript-sdk.git "${DEST}"

echo "Installed upstream typescript-sdk under ${DEST}"