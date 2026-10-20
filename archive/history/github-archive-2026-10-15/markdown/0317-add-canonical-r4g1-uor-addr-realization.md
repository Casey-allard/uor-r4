# #317 — Add canonical R4G1 uor-addr realization

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T23:28:53Z
- **Updated:** 2026-08-01T00:46:32Z
- **Closed:** 2026-08-01T00:46:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/317
- **Merged:** 2026-08-01T00:46:32Z
- **Merge commit:** 7bb3ccbb6509eada715e4d2c0aa594dd701e7869
- **Base / Head:** main / UOR-Foundation:issue-264-r4g1-realization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the first in-repository `uor-addr` realization for R4G1 artifacts and switches external graph artifact κ reporting/serving attestation to the representation-level address.

- Adds an allocation-gated two-level CBOR Merkle skeleton:
  - per-section κ commits to section ID, flags, and payload;
  - artifact κ commits to the canonical section-ID-ordered list.
- Keeps R4G1 wire CIDs as exact-byte runtime integrity checks.
- Switches scored graph reporting and active serving artifact attestation from raw container hashes to the realization.
- Adds positive/negative conformance vectors and documents the realization contract.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo test -p uor-r4-graph-format --offline` (20 unit, 2 fuzz-smoke, 3 realization, 26 stage-1, 46 stage-2 passed)
- `cargo test --workspace --offline` reached 79 graph-compiler tests passed but has one unrelated pre-existing fixture failure: `observation_text::tests::crash_trims_converge_to_single_pass_kappa` cannot find its tokenizer fixture.

Closes #264

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

