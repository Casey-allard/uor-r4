#!/usr/bin/env bash
set -euo pipefail

# Verifies Lean formal scaffold; writes a JSON report in research/output.
# This script expects Lean toolchain availability (lean/lake or elan).

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LEAN_FILE="$ROOT_DIR/research/formal/lean/PrimeRiemannBridge.lean"
OUT_JSON="$ROOT_DIR/research/output/formal_compile_report_2026-02-17.json"

timestamp() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

write_report() {
  local status="$1"
  local message="$2"
  local compile_output="${3:-}"
  cat >"$OUT_JSON" <<EOF
{
  "timestamp_utc": "$(timestamp)",
  "lean_file": "research/formal/lean/PrimeRiemannBridge.lean",
  "status": "$status",
  "message": "$message",
  "compile_output": $(python3 - <<PY
import json
print(json.dumps("""$compile_output"""))
PY
)
}
EOF
}

if command -v lean >/dev/null 2>&1; then
  set +e
  output="$(lean "$LEAN_FILE" 2>&1)"
  rc=$?
  set -e
  if [[ $rc -eq 0 ]]; then
    write_report "pass" "Lean compilation succeeded." "$output"
    echo "Lean check passed. Report: $OUT_JSON"
    exit 0
  fi
  write_report "fail" "Lean compilation failed." "$output"
  echo "Lean check failed. Report: $OUT_JSON"
  exit 1
fi

if command -v elan >/dev/null 2>&1; then
  write_report "blocked" "elan installed but lean missing from PATH/toolchain not activated."
  echo "Lean unavailable via elan toolchain activation. Report: $OUT_JSON"
  exit 2
fi

write_report "blocked" "Lean toolchain not installed (lean/elan missing)."
echo "Lean toolchain not installed. Report: $OUT_JSON"
exit 2
