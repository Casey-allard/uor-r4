# #1180 — feat(native_geometric): implement mixed multi-operator composition (Add/Sub) (#1140, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T01:41:17Z
- **Updated:** 2026-09-08T01:42:39Z
- **Closed:** 2026-09-08T01:42:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1180
- **Merged:** 2026-09-08T01:42:19Z
- **Merge commit:** 4f7dafa20d931f57405b5c90f0f34fbb60bd8b42
- **Base / Head:** main / UOR-Foundation:issue-1140-mixed-operator-composition
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Extends the typed geometric value runtime to support mixed multi-operator composition (`ValueAction::Sub` alongside `Add` and `Copy`), verifying causal non-commutative provenance tracking, causal intervention necessity, and autonomous mixed-operator generation.

- Implements checked exact $Z[\phi]$ subtraction `ZPhi::checked_sub` and `ValueAction::Sub`.
- Expands proposal addressing from 272 to 528 choices (`0..16` Copy, `16..272` Add, `272..528` Sub) with non-commutative operand rank encoding `b = (rank_a << 8) | rank_b`.
- Tracks subtraction execution in `ValueWork.subtractions`.
- Validates single-step non-commutative subtraction ($20 - 7 = 13$, `subtractions: 1`).
- Validates causal Add -> Sub state transition ($20 + 15 = 35 -> 35 - 8 = 27$): Operator 2 consumes Operator 1's committed output (`write_id`), preserving exact causal provenance (`operand_ids: [write_id_1, next_id]`, `operand_values: [35, 8]`).
- Proves causal necessity via intervention: ablating Operator 1's committed record from state causes Operator 2's prediction to diverge.
- Validates autonomous mixed Add/Sub generation emitting intermediate and final values (`35`, `27`) during token generation while tracking `additions: 1`, `subtractions: 1`, `source_refreshes: 1`.
- Compiles generated mixed chained Rust programs with `rustc --edition=2021` and executes with exit code 0.
- Preserves zero runtime heap allocations across `#![no_std]` hot path in `tests/native_geometric_allocations.rs`.

References #1140, #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

