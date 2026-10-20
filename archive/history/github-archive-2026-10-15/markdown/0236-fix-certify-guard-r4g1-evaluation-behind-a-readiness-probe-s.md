# #236 — fix(certify): guard R4G1 evaluation behind a readiness probe, skip vacuous runs (#232)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T02:55:05Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-07-28T03:11:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/236
- **Merged:** 2026-07-28T03:11:42Z
- **Merge commit:** 36b91fc8f62faa10aace9d2dd72ab658736158dc
- **Base / Head:** main / UOR-Foundation:issue-232-guard-r4g1-eval
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #232.

## What

`certify`'s C-section (R4G1 graph path) previously ran the full held-out evaluation — ~2.5 of 3.2 hours, single-threaded — even when the graph was untrained/unscored, terminating in an all-zero row that reads as a measured failure rather than an absent measurement.

- **Readiness probe** (`r4g1_readiness`, new certifier-side module): runs `predict_distribution` over the first 64 held-out positions (deterministic, in order) and classifies the output shape — `Ready`, `NoScoredEmission` (every prediction at `ScoreQ::MIN`), or `ConstantPrediction` (one token for every context — the constant root-fallback shape). Degenerate shapes skip the full evaluation with an explicit `SKIPPED — … no measurement recorded` marker.
- **Allocation hoist**: one reusable `node_scores` buffer across probe + evaluation, replacing a `node_count`-sized `vec!` allocated per held-out position (for a parameter the runtime doesn't read).
- **Honest WB field**: the `WB 0.0000 bits/token` in the C row was a hardcoded string literal, never a computation — now prints `WB n/a (not computed on the graph path)`.

## Scope

Certifier-side only. No runtime-crate changes: P-4 kernel scan, allocation census, and the `no_std` surface untouched; artifact bytes and κs unchanged.

Out of scope (follow-ups): evaluating `graph/score.r4g1` inside legacy certify; the C-section's `hist` construction feeding rotation offsets as context lags (separate defect, to be filed).

## Gates (macOS arm64, offline)

- `cargo test --workspace --offline` ✅ (incl. 3 new tests in `tests/r4g1_eval_readiness.rs`)
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` ✅
- `cargo fmt --check` ✅
- `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`) ✅

Expected impact: ~3.2h → ~0.7h for legacy-path `certify` iteration when no trained graph is present (#232's measurement).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

