#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LEAN_DIR="$ROOT_DIR/research/formal/lean"
OUT_JSON="$ROOT_DIR/research/output/formal_compile_report_completion_kernel_2026-02-17.json"

timestamp() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

write_report() {
  local status="$1"
  local message="$2"
  local compile_output="${3:-}"
  cat >"$OUT_JSON" <<JSON
{
  "timestamp_utc": "$(timestamp)",
  "lean_target": "PrimeRiemannBridgeCompletionKernel",
  "status": "$status",
  "message": "$message",
  "compile_output": $(python3 - <<PY
import json
print(json.dumps("""$compile_output"""))
PY
)
}
JSON
}

if ! command -v lake >/dev/null 2>&1; then
  write_report "blocked" "lake not found; cannot build completion kernel."
  echo "lake not found. Report: $OUT_JSON"
  exit 2
fi

set +e
output="$(cd "$LEAN_DIR" && export PATH="$HOME/.elan/bin:$PATH" && lake env lean PrimeRiemannBridgeCompletionKernel.lean 2>&1)"
rc=$?
set -e

if [[ $rc -eq 0 ]]; then
  write_report "pass" "Completion-kernel Lean check succeeded." "$output"
  echo "Completion-kernel Lean check passed. Report: $OUT_JSON"
  exit 0
fi

write_report "fail" "Completion-kernel Lean check failed." "$output"
echo "Completion-kernel Lean check failed. Report: $OUT_JSON"
exit 1
