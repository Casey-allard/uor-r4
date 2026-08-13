#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LEAN_DIR="$ROOT_DIR/research/formal/lean"
OUT_JSON="$ROOT_DIR/research/output/formal_compile_report_mathlib_2026-02-17.json"

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
  "lean_target": "PrimeRiemannBridgeMathlib",
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
  write_report "blocked" "lake not found; cannot build mathlib target."
  echo "lake not found. Report: $OUT_JSON"
  exit 2
fi

set +e
output="$(cd "$LEAN_DIR" && export PATH="$HOME/.elan/bin:$PATH" && lake build PrimeRiemannBridgeMathlib 2>&1)"
rc=$?
set -e

if [[ $rc -eq 0 ]]; then
  write_report "pass" "Mathlib-backed Lean build succeeded." "$output"
  echo "Mathlib Lean build passed. Report: $OUT_JSON"
  exit 0
fi

write_report "fail" "Mathlib-backed Lean build failed." "$output"
echo "Mathlib Lean build failed. Report: $OUT_JSON"
exit 1
