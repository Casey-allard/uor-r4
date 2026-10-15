# #321 — Remediate sign loss in Hopf sector projection

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T00:50:01Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-01T01:02:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/321
- **Merged:** 2026-08-01T01:02:02Z
- **Merge commit:** 60e5917b2c597d5a9a271de5f496a800b8f5c467
- **Base / Head:** main / UOR-Foundation:issue-306-hopf-projection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements #306 by replacing the sign-restricted block-norm projection that under-addressed Hopf sectors.

- Projects each 128-dimensional state block onto a deterministic signed Blake3-derived direction.
- Preserves within-block direction and sign before four-component normalization.
- Adds a regression test for positive/negative direction preservation.
- Documents the #303 baseline and the recorded #306 D3 measurement.
- Keeps this change in the exploratory floating-point router; the deployed integer runtime kernel is untouched.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p uor-r4-router --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test --workspace --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`

The ignored D3 measurement command exits cleanly, but this checkout does not contain the D3 corpus and therefore reports a documented vacuous skip.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

