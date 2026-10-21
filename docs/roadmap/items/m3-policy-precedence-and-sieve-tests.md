---
id: m3-policy-precedence-and-sieve-tests
title: Pin OpPolicy precedence and store-miss sieve coupling with tests
milestone: M3
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-training/**, docs/labs/m3-policy-precedence-and-sieve-tests/**]
---

## Why it matters

Removes ambiguity in memory behaviour before M3 qualification. Two choices in the memory path are documented only in comments, so a later edit could change them without any test failing. The session code involved is described in [the relation read](../../mechanisms/relation-read.md) and [exact memory](../../mechanisms/exact-memory.md).

## What is missing

No explicit unit tests assert the documented precedence of `OpPolicy::UnlessQuery` versus `OpPolicy::TableStatements` when the table and op-model disagree (`relation_compiler.rs`). In `stack_grounded_session.rs`, a query that resolves to a relation id and misses the store returns `StoreRead::Absent` ("Memory: none.") without consulting the conversation log sieve, which runs only on `MemoryEffect::Unresolved`. The intended coupling is undocumented and untested (#1738).

## Done when

Unit tests for both precedences and a documented, tested store-miss and sieve behaviour.

## How to start

1. Read `OpPolicy` (`crates/uor-r4-training/src/relation_compiler.rs`, around line 1071) and `SavedCompiler::action` (around line 2219). The two arms differ: `TableStatements` returns the table action unless the table read a statement, then prefers the op model's statement; `UnlessQuery` returns the table action only when the table read a query, otherwise prefers the op model's statement. Both keep the table action when it carries a derived relation id (`>= DERIVED_RELATION_ID_BASE`).
2. Read the existing tests near `a_saved_compiler_reloads_exactly_and_compiles_turns` (around line 2808) and `an_op_policy_parses_and_needs_an_op_model` (around line 3205) to see how a compiler with both an op model and a table is built in tests.
3. Post a failing or pending test as the first deliverable. Write tests in the `#[cfg(test)]` module of `relation_compiler.rs` that build one compiler whose table and op model disagree, then assert the action under each policy: table says query, op says statement (the policies differ); table says statement with a closed id, op says another statement (both prefer the op); table says statement with a derived id (the table wins under both); table says nothing. Name the tests for the rule they pin.
4. Read `emitter_suffix` (the log-recall selection) and `apply_action` in `crates/uor-r4-training/src/stack_grounded_session.rs` (around lines 650–760). Write a session test beside `exact_spans_updates_conflicts_and_absence_remain_distinct` (around line 2405) with `log_recall` set: a query that resolves to a relation and misses the store must give `RecallDisposition::Absent` and must not call the sieve; an `Unresolved` turn must call it and give `RecallDisposition::LogValue`. Decide with the PM whether that coupling is intended; if so, document it in the doc comment on `MemoryEffect` and `RecallDisposition`. If it is not, file a decision card instead of changing behaviour.
5. Run `~/.cargo/bin/cargo test -p uor-r4-training relation_compiler` and `~/.cargo/bin/cargo test -p uor-r4-training stack_grounded_session`, then `~/.cargo/bin/cargo fmt --check` and `uor-harness check`. Show that each new test fails when the rule it pins is broken, for example by swapping the arms locally.
6. Open a PR that says `Closes #<this issue>`. Record the result as one row in `docs/research/notebook/m3-durable-memory.md`. No GPU run is needed.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1738)
- `crates/uor-r4-training/src/relation_compiler.rs` (`OpPolicy`, line 1071; `action`, lines 2219–2300)
- `crates/uor-r4-training/src/stack_grounded_session.rs` (`apply_action` and `emitter_suffix`, lines 650–760)
