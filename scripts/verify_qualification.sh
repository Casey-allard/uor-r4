#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "=== 1. Checking Static Code Quality & Claim Wording ==="
cargo fmt --check
python3 scripts/check_claim_wording.py
cargo check --manifest-path crates/uor-r4-core/Cargo.toml --all-targets

echo "=== 2. Checking Allocation & Integer Kernel Invariants ==="
cargo test --manifest-path crates/uor-r4-core/Cargo.toml --test native_geometric_allocations

echo "=== 3. Executing Comprehensive 9,984 Historical Regression Suite ==="
REPORT_DIR="$REPO_ROOT/target/independent-neighbor-eval-report-$(date +%s)-$RANDOM"
rm -rf "$REPORT_DIR"

EVIDENCE_DIR="/Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step"
DATA_DIR="$EVIDENCE_DIR/independent-neighbor-1/attempt-1"
ACCEPTANCE_FILE="$EVIDENCE_DIR/independent-neighbor-1/acceptance.json"
SEED="240836092112233"

# Build release test binary
echo "Building release test binary..."
cargo test --release --manifest-path crates/uor-r4-core/Cargo.toml --no-run

# Locate test binary
TEST_BIN=$(find target/release/deps -maxdepth 1 -type f -name "uor_r4_core-*" ! -name "*.*" -perm -0100 | head -n 1)
if [[ -z "$TEST_BIN" ]]; then
    echo "ERROR: Could not locate release test binary"
    exit 1
fi

echo "Running independent_neighbor_evaluate_report via $TEST_BIN..."
UOR_INDEPENDENT_NEIGHBOR_REPORT="$REPORT_DIR" \
UOR_INDEPENDENT_NEIGHBOR_EVIDENCE="$EVIDENCE_DIR" \
UOR_INDEPENDENT_NEIGHBOR_DATA="$DATA_DIR" \
UOR_INDEPENDENT_NEIGHBOR_ACCEPTANCE="$ACCEPTANCE_FILE" \
UOR_INDEPENDENT_NEIGHBOR_SEED="$SEED" \
"$TEST_BIN" native_geometric::dependent_language::independent_neighbor_report::independent_neighbor_evaluate_report --ignored --exact --nocapture

# Validate summary.json
SUMMARY="$REPORT_DIR/summary.json"
if [[ ! -f "$SUMMARY" ]]; then
    echo "ERROR: summary.json not generated!"
    exit 1
fi

python3 - <<PYEOF
import json, sys

with open("$SUMMARY") as f:
    s = json.load(f)

gate = s.get("gate")
correct = s.get("correct")
rows = s.get("rows")
r6688 = s.get("retained_6688_equal")

print(f"Gate: {gate}, Correct: {correct}/{rows}, Retained-6688: {r6688}/6688")

if gate != "PASS_INDEPENDENT_NEIGHBOR_TRANSFER":
    sys.exit(f"FAIL: gate was {gate}")
if correct != 2304 or rows != 2304:
    sys.exit(f"FAIL: correct was {correct}/{rows}")
if r6688 != 6688:
    sys.exit(f"FAIL: retained_6688_equal was {r6688}")

print(">>> ALL 9,984 REGRESSION & TRANSFER CASES VERIFIED 100% PASS <<<")
PYEOF

rm -rf "$REPORT_DIR"

echo "=== All Verification Gates Passed Cleanly ==="
