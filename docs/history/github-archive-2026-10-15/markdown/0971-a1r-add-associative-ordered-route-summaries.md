# #971 — A1R: add associative ordered route summaries

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T07:28:44Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-08-27T07:30:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/971
- **Merged:** 2026-08-27T07:30:06Z
- **Merge commit:** cabc922751d8f1bc3c1d3a291a0a1a18897fcdd0
- **Base / Head:** main / UOR-Foundation:issue-967-associative-ordered-summaries
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

- adds an exact associative, noncommutative canonical-H4 fold with O(1) incremental append
- exposes ordered scope state without changing frozen #952 legacy trace bytes or candidate admission
- exercises the bounded A1R anti-recall probe and CLI report
- records terminal `RETAIN_STATE_ONLY`: candidate-relative states are distinct on 6/6 queries, but shortest Cayley distance ties both candidates on 6/6

Canonical report kappa: `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`

This is structural state evidence only. It does not establish full recursive attention, inference, generation, correctness, reasoning, or product readiness.

## Bounded evidence

- 3/3 fixed matched contrasts and 6/6 candidate queries
- 12/12 exact candidate payload inversions
- 6/6 exact incremental/prefix checks
- 120/120 fixed H4 permutation census
- 1,728,000/1,728,000 exact H4 associativity checks
- support, row, candidate, and work budgets remained equal
- existing-additive ranking is explicitly `NOT_EXERCISED_NO_PREDECLARED_ADDITIVE_SCORER`

## Focused checks

- `cargo test -p uor-r4-core --offline --test recursive_geometric_attention_967` — 2 passed
- `cargo test -p uor-r4-core --offline --test recursive_geometric_attention_952` — 2 passed
- `cargo check -p uor-r4-core --lib --offline` — passed
- `cargo check --bin r4 --offline` — passed
- exact executor source-closure test — passed
- WASM library check — passed with existing target-specific warnings
- `cargo fmt --check`, `git diff --check`, and claim-wording gate — passed

## Successor safety

The exact native chain is `#967 → #970 → #969 → #953`. #970 is the unassigned A1P candidate-relative readout/placement falsifier and blocks A1Q/#969; #969 continues to block generation/#953. Closing this issue therefore cannot promote attention or generation.

Closes #967

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

