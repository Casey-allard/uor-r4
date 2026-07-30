#!/usr/bin/env bash
# preflight.sh — run the EXACT `fmt / clippy / tests / no_std / κ` CI gate
# locally, before every push. One command; a green preflight means the only
# CI surprises left are environment-level (network, runner), not code.
#
#   ./scripts/preflight.sh            # full gate (worktree-local target)
#   PREFLIGHT_FAST=1 ./scripts/...    # skip the two slowest steps (trend
#                                     # harness + BDD) for inner-loop use;
#                                     # NEVER push on a fast-only pass.
#
# Build isolation (2026-07-30 process decision): preflight defaults to the
# WORKTREE-LOCAL target dir. Sharing one CARGO_TARGET_DIR across worktrees
# caused three stale-binary incidents in one night (cargo fingerprints can
# resolve to another worktree's rlib for same-named crates); the first build
# per worktree is cold, everything after is warm and CANNOT be contaminated
# by builds from sibling worktrees. Set PREFLIGHT_TARGET to override.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR="${PREFLIGHT_TARGET:-$PWD/target}"
echo "preflight: target dir $CARGO_TARGET_DIR"

step() { echo; echo "== preflight: $1"; }

step "claim-wording gate"
python3 scripts/check_claim_wording.py

step "inference contract audit"
cargo test -q -p uor-r4-proof-model --lib inference_audit

step "cargo fmt --check"
cargo fmt --check

step "cargo clippy (--all-targets --all-features -D warnings)  [CI parity: not just --workspace]"
cargo clippy --workspace --all-targets --all-features -- -D warnings

step "workspace tests"
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --workspace -E 'not binary(bdd)'
else
  echo "(cargo-nextest not installed; falling back to cargo test)"
  cargo test --workspace -q
fi

if [ -z "${PREFLIGHT_FAST:-}" ]; then
  step "cucumber BDD suite"
  cargo test -q --test bdd
fi

step "doc tests"
cargo test -q --doc --workspace

step "no_std ladder (graph-format)"
cargo check -q -p uor-r4-graph-format --no-default-features

step "deterministic rebuild (Gate E slice)"
cargo test -q -p uor-r4-core --test deterministic_rebuild_test

if [ -z "${PREFLIGHT_FAST:-}" ]; then
  step "Gate C trend harness + regression/re-pin check (the one that ejects queue entries)"
  rm -rf trend_output
  cargo run -q --release --bin r4 -- transformerless score \
    --corpus-meta crates/uor-r4-core/tests/fixtures/c_meta.bin \
    --corpus-recs crates/uor-r4-core/tests/fixtures/c_recs.bin \
    --artifacts crates/uor-r4-core/tests/fixtures/tless_artifacts.bin \
    --out trend_output >/dev/null
  BASE_PIN=$(git show origin/main:docs/transformerless/gate_c_pinned.json 2>/dev/null || true)
  if [ -n "$BASE_PIN" ]; then
    ./scripts/check_gate_c_regression.py trend_output/score_report.json \
      --base-pin <(echo "$BASE_PIN")
  else
    ./scripts/check_gate_c_regression.py trend_output/score_report.json
  fi
fi

if [ -f /tmp/ref/out/model.bin ]; then
  step "κ-reproduction (checkpoint present)"
  TLESS_CHECKPOINT=/tmp/ref/out/model.bin cargo test -q -p uor-r4-core --release --test kappa_reproduction -- --ignored
else
  echo "== preflight: κ-reproduction SKIPPED (no /tmp/ref/out/model.bin — vacuous green, see AGENTS.md)"
fi

echo
echo "preflight: ALL GATES GREEN — safe to push"
