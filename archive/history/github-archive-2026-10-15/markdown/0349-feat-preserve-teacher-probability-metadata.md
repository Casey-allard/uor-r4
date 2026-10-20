# #349 — feat: preserve teacher probability metadata

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T20:48:07Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-08-02T21:25:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/349
- **Merged:** 2026-08-02T21:25:19Z
- **Merge commit:** 34e10629b7187c8c83bd61309895ff81b1dc004b
- **Base / Head:** main / UOR-Foundation:issue-127-probability-entropy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- preserve compiler-side target log-probability, full-vocabulary entropy, top-8 mass, and target rank while teacher logits are available
- write aligned `shard-NN.bin.prob` sidecars without changing the existing 88-byte observation record ABI
- report teacher-forced message `bits/token` from additive log-domain information
- add crash-recovery, sidecar alignment, entropy, and message-information tests
- document the compiler/runtime boundary: no softmax, logarithm, division, or floating point enters deployed inference

## Why

Top-8 evidence weights are intentionally renormalized for induction and are not the original full-vocabulary probabilities. The sidecar preserves the missing likelihood and uncertainty information needed for honest entropy and message-probability measurements.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- focused core and graph-compiler probability/crash-recovery tests
- full graph-compiler package suite

The workspace test suite reaches the existing R4G1 allocation-census assertion: 2 allocations / 594 bytes in the pre-existing `predict_token + predict_distribution + predict_candidates + step` kernel census. This change does not touch that runtime path.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

