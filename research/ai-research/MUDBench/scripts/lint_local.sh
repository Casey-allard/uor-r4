#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if command -v ruff >/dev/null 2>&1; then
  echo "[lint-local] running ruff check"
  ruff check src tests examples
elif command -v flake8 >/dev/null 2>&1; then
  echo "[lint-local] running flake8"
  flake8 src tests examples
else
  echo "[lint-local] error: neither 'ruff' nor 'flake8' is installed." >&2
  echo "[lint-local] install one linter to satisfy local static validation." >&2
  exit 1
fi

echo "[lint-local] running syntax compile checks"
PYTHONPATH=src python -m compileall -q src examples tests
