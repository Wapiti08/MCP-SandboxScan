#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

OBF_WASM="fixtures/obfus_fs_violation/tool.wasm"
EXFIL_WASM="fixtures/tool_return_file_exfil/tool.wasm"

# Require DATA_DIR from caller (host dir that contains secret.txt)
DATA_DIR="${DATA_DIR:-}"
if [[ -z "$DATA_DIR" ]]; then
  echo "ERROR: DATA_DIR is not set."
  echo "Example: DATA_DIR=/path/to/host_data ./demo/run_static_vs_dynamic.sh"
  exit 1
fi

if [[ ! -f "$DATA_DIR/secret.txt" ]]; then
  echo "ERROR: not found: $DATA_DIR/secret.txt"
  exit 1
fi

# The token you expect to leak (must match the content in secret.txt)
TOKEN="${TOKEN:-FILESECRET_9f2c1a7b}"


contains_bytes () {
  local file="$1"
  local needle="$2"
  python3 - <<PY "$file" "$needle"
import sys
p, needle = sys.argv[1], sys.argv[2].encode()
data = open(p, "rb").read()
print("YES" if needle in data else "NO")
PY
}

run_demo () {
  local name="$1"
  local wasm="$2"
  shift 2
  # pass DATA_DIR so /data is available
  DATA_DIR="$DATA_DIR" "$@" cargo run --quiet --bin demo -- "$wasm"
}

echo "=== Static vs Dynamic demo ==="
echo "DATA_DIR=$DATA_DIR"
echo

# --- 1) Obfuscated FS violation
echo "[1] Obfuscated FS violation"
echo -n "Static: wasm contains literal \"/etc/passwd\" ? "
contains_bytes "$OBF_WASM" "/etc/passwd"

out="$(run_demo obf "$OBF_WASM")"
# dynamic evidence: look for READ_ERR in stderr block
echo -n "Dynamic: observed READ_ERR/denied in stderr? "
echo "$out" | awk '
  BEGIN{in_stderr=0; found=0}
  /^stderr:/{in_stderr=1; next}
  /^== Summary ==/{in_stderr=0}
  {
    if(in_stderr && ($0 ~ /READ_ERR/ || tolower($0) ~ /denied/ || tolower($0) ~ /not permitted/ || tolower($0) ~ /permission/)) found=1
  }
  END{ print(found ? "YES" : "NO") }
'
echo

# --- 2) File exfiltration via /data/secret.txt
echo "[2] File exfiltration (/data/secret.txt)"
echo -n "Static: wasm contains injected token (FILESECRET...)? "
contains_bytes "$EXFIL_WASM" "$TOKEN"

out="$(run_demo exfil "$EXFIL_WASM")"
flows="$(echo "$out" | awk -F': ' '/^[[:space:]]*flows: /{print $2; exit}')"
echo "Dynamic: flows = ${flows:-NA} (expect >0 if token reaches tool-return sink)"
echo