# #1005 — fix(#973): repair the canonical H4 frame census

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T00:50:53Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-30T00:52:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1005
- **Merged:** 2026-08-30T00:52:10Z
- **Merge commit:** c6b86b8f8dc5ea9e4e2c5567ae7d95ffecd8de73
- **Base / Head:** main / UOR-Foundation:issue-973-helm-d-manifold-census-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Repairs the Attempt 01 construction preflight without changing the frozen learned-manifold experiment.

The prior harness tried to discover all 120 registered H4 frames through one cumulative token walk. The frozen four lexical leaves generate a strict 24-element subgroup and that walk visits exactly 12 states, so the enumerator could never satisfy the synthetic registry-wide census.

This change:
- enumerates the exact canonical 120-frame H4 registry directly for covariance;
- keeps source-frame-permutation liveness on a separate real causal atlas;
- runs the cheap registry gate before expensive donor capture;
- preserves Attempt 01 byte-for-byte and records validation=false, checkpoint=false, D3=NOT_RUN;
- changes no population, donor, operator, fitter, threshold, decision branch, or validation seal.

## Verification

- RUST_TEST_THREADS=1 cargo test --workspace --offline --no-fail-fast
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo fmt --check
- cargo check -p uor-r4-graph-format --no-default-features --offline
- cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline
- cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline
- python3 scripts/check_claim_wording.py
- exact repaired preflight: 120/120, PASS
- independent cross-review: no P0-P2 findings

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

