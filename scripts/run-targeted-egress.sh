#!/usr/bin/env bash
set -euo pipefail

IMAGE="sandscope-targeted-egress"
SUBJECT_VOLUME="sandscope-targeted-subjects"
RESULT_DIR="$PWD/mcp-sandboxscan/reports/declared-only-targeted"
PLAN="/workspace/mcp-sandboxscan/experiments/declared_only_plan.json"

if command -v docker >/dev/null 2>&1; then
  DOCKER=(docker)
elif [[ -x /Applications/Docker.app/Contents/Resources/bin/docker ]]; then
  PATH="/Applications/Docker.app/Contents/Resources/bin:$PATH"
  export PATH
  DOCKER=(/Applications/Docker.app/Contents/Resources/bin/docker)
else
  echo "error: docker is required for the isolated targeted-egress experiment" >&2
  exit 127
fi

"${DOCKER[@]}" build \
  -f Dockerfile.targeted \
  -t "$IMAGE" \
  .

"${DOCKER[@]}" volume create "$SUBJECT_VOLUME" >/dev/null
mkdir -p "$RESULT_DIR"

# Phase 1: install/build dependencies.
# No credentials are passed and the workspace is read-only.
"${DOCKER[@]}" run --rm \
  --cpus 4 \
  --memory 8g \
  --pids-limit 512 \
  -v "$PWD:/workspace:ro" \
  -v "$SUBJECT_VOLUME:/subjects" \
  "$IMAGE" \
  prepare \
  --plan "$PLAN"

# Phase 2: execute without external network.
# strace observes direct sockets that ignore HTTP_PROXY.
"${DOCKER[@]}" run --rm \
  --network none \
  --cap-add SYS_PTRACE \
  --security-opt seccomp=unconfined \
  --add-host api.figma.com:127.0.0.1 \
  --add-host login.salesforce.com:127.0.0.1 \
  --add-host api.bitbucket.org:127.0.0.1 \
  --add-host ip-api.com:127.0.0.1 \
  --add-host controlled.atlassian.net:127.0.0.1 \
  --cpus 2 \
  --memory 4g \
  --pids-limit 256 \
  -v "$PWD:/workspace:ro" \
  -v "$SUBJECT_VOLUME:/subjects" \
  -v "$RESULT_DIR:/results" \
  "$IMAGE" \
  run \
  --plan "$PLAN"
