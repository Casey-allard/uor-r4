#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DETERMINISM_GATE_TESTS=(
  "tests/benchmark/test_real_determinism_gate.py"
)

for test_path in "${DETERMINISM_GATE_TESTS[@]}"; do
  if [[ ! -f "$test_path" ]]; then
    echo "[determinism-local] error: required determinism gate test is missing: $test_path" >&2
    exit 1
  fi
done

echo "[determinism-local] running real determinism gate suite"
echo "[determinism-local] test_count=${#DETERMINISM_GATE_TESTS[@]}"
PYTHONPATH=src python -m pytest --maxfail=1 -q "${DETERMINISM_GATE_TESTS[@]}"
