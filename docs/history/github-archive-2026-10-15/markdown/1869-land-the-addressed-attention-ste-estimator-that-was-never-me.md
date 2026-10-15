# #1869 — Land the addressed-attention STE estimator that was never merged (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:45:45Z
- **Updated:** 2026-10-09T13:18:59Z
- **Closed:** 2026-10-08T02:47:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1869
- **Merged:** 2026-10-08T02:47:20Z
- **Merge commit:** 36b78ce92004b80d435233e413dcd9a510e72397
- **Base / Head:** main / UOR-Foundation:deepseek/aa-ste-land
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this lands

`deepseek/aa-ste` was completed but never delivered: the addressed-attention straight-through gate estimator existed only on a pushed branch. Owner direction of 7 October is explicit that completed work must reach `main`, so this delivers it:

- `crates/uor-r4-core/src/native_geometric/addressed_attention/ste.rs` — the opt-in straight-through estimator for the addressed-attention gate.
- `crates/uor-r4-core/src/native_geometric/addressed_attention/ste_tests.rs` — its focused tests.
- `crates/uor-r4-core/src/bin/addressed-attention-arith-ste.rs` — the arithmetic-ste probe binary.
- `crates/uor-r4-core/src/native_geometric/addressed_attention/mod.rs` — module registration (5 lines).

Rebased onto current `main` with no conflicts. It is additive: new module plus registration, no change to any existing path's behaviour.

## Verification at the exact head

- `cargo check -p uor-r4-core --all-targets` — clean, no errors.
- `cargo test -p uor-r4-core ste` — **52 passed / 0 failed** (1,140 filtered).
- Merge-base check: `git rebase origin/main` applied the single commit without conflict.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

