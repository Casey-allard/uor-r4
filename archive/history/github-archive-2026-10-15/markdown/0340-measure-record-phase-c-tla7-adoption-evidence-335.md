# #340 — measure: record Phase C TLA7 adoption evidence (#335)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T23:14:45Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-01T23:29:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/340
- **Merged:** 2026-08-01T23:29:15Z
- **Merge commit:** 7f742b66dbde88da4de321af53631b285485550b
- **Base / Head:** main / UOR-Foundation:issue-335-phase-c-followup
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Completes the remaining Phase C adoption evidence for #335 after the 500k/TLA7 re-pin in #338.

- Adds compiler/test corpus-path overrides so κ re-pin review can use preserved corpora without replacing default fixtures.
- Adds an explicit persisted TLA7 bundle/code equality witness to the certifier.
- Adds the Phase C mantissa-bit fold row and a narrow `R4_CERTIFY_PHASE_C_ONLY=1` measurement mode.
- Records the 500k store-shape decision: cpy8 improves quality but exceeds the WB regression bound; the mantissa fold stays recorded but is not adopted.

## Evidence

500k/TLA7 rows:

- shipped A: 34.7% / 39.0% / 8.0249 WB / 179,068 keys
- cpy8 residual: 35.3% / 39.6% / 8.5247 WB / 195,650 keys
- mantissa fold: 34.6% / 38.9% / 8.3097 WB / 162,119 keys

The persisted TLA7 witness is 512/512 on the 1,346,836-byte container.

## Verification

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all -- --check`
- graph-format no_std + alloc checks
- claim-wording gate

Closes #335.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

