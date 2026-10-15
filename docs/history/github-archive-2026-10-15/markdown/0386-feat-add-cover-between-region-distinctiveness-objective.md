# #386 — feat: add cover between-region distinctiveness objective

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T18:03:39Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-03T18:19:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/386
- **Merged:** 2026-08-03T18:19:33Z
- **Merge commit:** 586e12cdaef26a7794cbd40106c8ca3a255f84bd
- **Base / Head:** main / UOR-Foundation:issue-379-cover-distinctiveness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- add a schema-2, default-off between-region distinctiveness term to cover induction
- score deterministic top-64 child/prior overlap against the global next-token prior and expose it in split audits and cover reports
- add `--distinctiveness-weight` to `cover-sweep`, with CLI coverage and README documentation

Closes #379.

## Validation
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`

## Measurement
Fixture sweep at `--distinctiveness-weight 1.0` with probability emission selection and contrast shrinkage: the 9-point frontier recommends `k0=8/gain=0.1/budget=128`, +17,472 bytes for +0.0052 Rule 1+2 top-1 agreement over the default row. The report records schema 2 and the applied weight per point.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

