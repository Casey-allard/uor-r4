# #326 — feat: add opt-in proof-carrying inference witnesses (#266)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T01:40:55Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-01T03:54:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/326
- **Merged:** 2026-08-01T03:54:37Z
- **Merge commit:** fe6f6fc16e26835dc42c04423fd158a4d71cfdf8
- **Base / Head:** main / UOR-Foundation:issue-266-proof-witness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements #266 with an opt-in proof-carrying response path for R4G1 generation.

- Adds `include_witness: true` support to `POST /api/r4g1/generate`.
- Emits compact per-token claims: region κ, region id, traversal depth, resolution status, engine, token, and widening.
- Extends `POST /api/uor/verify` to replay those claims against the loaded artifact.
- Returns typed rejection reasons for token, region, depth, status, engine, widening, and length mismatches.
- Keeps default inference on the existing allocation-free step path; witness assembly uses the server-side reference scorer only when requested.
- Adds valid/tampered conformance coverage and user-facing documentation.

## Verification

Passing:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo test --workspace --offline --exclude uor-r4-core`
- focused and full status-policy tests
- `python3 scripts/check_claim_wording.py`

Known baseline failure:
- `cargo test --workspace --offline` reaches the existing `uor-r4-core/tests/allocation_census.rs` R4G1 section and reports 2 allocations / 594 bytes for `predict_token + predict_distribution + predict_candidates + step × 10`; the transformerless census and the rest of the workspace suite pass. This diff does not modify the failing runtime-census files.

## Follow-up boundary

The witness κ is anchored to the canonical artifact and NODE-section addresses. Standalone exported region objects/resolver manifests remain the separate #263 scope.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

