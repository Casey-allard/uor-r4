# ORDER: m3-policy-precedence-and-sieve-tests, antigravity-6

Lab: antigravity. Item: [m3-policy-precedence-and-sieve-tests](../../../../roadmap/items/m3-policy-precedence-and-sieve-tests.md). Base: `f1c78da8e`.

## Goal

The item's done-when reads:

> Unit tests for both precedences and a documented, tested store-miss and sieve behaviour.

## Cancel probe (run first)

Ran `rg -n 'OpPolicy::(TableStatements|UnlessQuery)|DERIVED_RELATION_ID_BASE' crates/uor-r4-training/src/relation_compiler.rs` and `rg -n 'log_recall|RecallDisposition::(Absent|LogValue)' crates/uor-r4-training/src/stack_grounded_session.rs`.

Result:
1. `OpPolicy::TableStatements` and `OpPolicy::UnlessQuery` appear in `SavedCompiler::action` (`relation_compiler.rs:2233-2293`) and in the table-only rejection check (`relation_compiler.rs:2827-2829`), with no unit test exercising `SavedCompiler::action` when `self.op.is_some()` and the table and op model disagree (including the `*relation >= DERIVED_RELATION_ID_BASE` guard).
2. In `stack_grounded_session.rs:743-765`, `MemoryEffect::Read { read: StoreRead::Absent }` returns `RecallDisposition::Absent` (`"Memory: none."`) while `self.log_recall` is consulted only on `MemoryEffect::Unresolved if read`, with no doc comment on `MemoryEffect` or `RecallDisposition` and no unit test pinning that `StoreRead::Absent` never invokes the `log_recall` sieve.

The work is not already on `main`; continue.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/labs/m3-policy-precedence-and-sieve-tests/**`

## Done when

- `crates/uor-r4-training/src/relation_compiler.rs` has unit tests exercising `SavedCompiler::action` across `OpPolicy::{Op, Table, TableStatements, UnlessQuery}` when the table and op model disagree:
  - table says query, op says statement;
  - table says statement with a closed relation id, op says another statement (and when op says a non-statement);
  - table says statement with a derived relation id (`>= DERIVED_RELATION_ID_BASE`), op says a closed-id statement;
  - table says nothing (`Unresolved`), op says statement (where `TableStatements` and `UnlessQuery` differ).
- `crates/uor-r4-training/src/stack_grounded_session.rs` documents the intended store-miss (`StoreRead::Absent` -> `RecallDisposition::Absent`, sieve not called) vs `MemoryEffect::Unresolved` (`log_recall` sieve consulted -> `RecallDisposition::LogValue`) coupling on `MemoryEffect` and `RecallDisposition`, and pins it with a unit test beside `exact_spans_updates_conflicts_and_absence_remain_distinct`.
- `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` records the design note and negative gates.

## Stop and ask if

- Any file outside `crates/uor-r4-training/**`, `docs/labs/m3-policy-precedence-and-sieve-tests/**`, `docs/loop/tasks/m3-policy-precedence-and-sieve-tests/**`, or the item/roadmap files must change.
- Any GPU run or change to serving behavior is required.
