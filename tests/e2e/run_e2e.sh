#!/usr/bin/env bash
# E2E Test Suite Runner for UOR-R4 Geometric Conversational Chatbot
set -euo pipefail

# Mandatory concurrency constraint
export CARGO_BUILD_JOBS=2

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "=== UOR-R4 Geometric Conversational Chatbot E2E Test Suite ==="
echo "Worktree : ${WORKTREE_DIR}"
echo "Script   : ${SCRIPT_DIR}/run_e2e_tests.py"
echo "Jobs     : ${CARGO_BUILD_JOBS}"
echo "Starting test execution across Tiers 1-4 (TAP 13)..."
echo

python3 "${SCRIPT_DIR}/run_e2e_tests.py" --worktree "${WORKTREE_DIR}" "$@"
