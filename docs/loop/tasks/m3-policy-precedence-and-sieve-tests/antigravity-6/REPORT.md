# REPORT: m3-policy-precedence-and-sieve-tests, antigravity-6

Lab: antigravity. Order: [ORDER.md](ORDER.md).

Closes #2857.

## What did not work, first

- Running `cargo test -p uor-r4-training relation_compiler` and `cargo test -p uor-r4-training stack_grounded_session` initially matched 0 tests because `pub mod relation_compiler;` and `pub mod stack_grounded_session;` in `crates/uor-r4-training/src/lib.rs` had been gated with `#[cfg(any())]` when commit `50b4f1d41` (`#2345`) switched `pub mod geometric_stack;` to `geometric_stack_native.rs`.
- Removing `#[cfg(any())]` from `pub mod relation_compiler;` and `pub mod stack_grounded_session;` without gating their historical `Softmax` / `SparseSoftmax` / `Trunk` / `StackModel`-dependent definitions failed `check-claim-wording`'s D23/D32 ratchet check (`crates/uor-r4-training/src/relation_compiler.rs` contained 4 historical `softmax`/`exp` sites and 12 `discrete-float` sites). We resolved this by gating only the historical `Softmax`/`SparseSoftmax`/`Trunk`/`StackModel` items and tests with `#[cfg(any())]` inside `relation_compiler.rs` and `stack_grounded_session.rs`, and extracting the pure policy-resolution and recall-line-resolution functions (`resolve_op_policy` and `resolve_recall_line`) so all 24 `relation_compiler` and `stack_grounded_session` unit tests compile and execute under `cargo test -p uor-r4-training --lib` on the host with zero D23/D25/D32 ratchet sites.
- Negative gate 1 (swapping `OpPolicy::TableStatements` and `OpPolicy::UnlessQuery` in `resolve_op_policy`): `cargo test -p uor-r4-training --lib op_policy_precedences_and_derived_id_guard_resolve_disagreements` failed as expected (`left: Assert { relation: 1, span: SourceSpan { start: 20, end: 25 } }`, `right: Unresolved { reason: "the turn states no relation" }`), then passed when restored.
- Negative gate 2 (dropping the `!table_derived` guard in `resolve_op_policy`): `cargo test -p uor-r4-training --lib op_policy_precedences_and_derived_id_guard_resolve_disagreements` failed as expected (`left: Correct { relation: 4, span: SourceSpan { start: 11, end: 17 } }`, `right: Assert { relation: 12957859, span: SourceSpan { start: 11, end: 17 } }`), then passed when restored.
- Negative gate 3 (allowing `StoreRead::Absent` to consult `log_recall` in `resolve_recall_line`): `cargo test -p uor-r4-training --lib store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` failed as expected (`left: Some("Memory: from-sieve:put blue:absent.")`, `right: Some("Memory: none.")`), then passed when restored.

## What changed

- Extracted `pub fn resolve_op_policy` (called by `SavedCompiler::action`) in `crates/uor-r4-training/src/relation_compiler.rs`, enabled `pub mod relation_compiler;` in `crates/uor-r4-training/src/lib.rs`, gated the historical `Softmax`/`SparseSoftmax`/`Trunk` definitions with `#[cfg(any())]`, and added unit test `op_policy_precedences_and_derived_id_guard_resolve_disagreements` pinning all four table-vs-op combinations (`QueryCurrent`/`Query` vs statement, closed-ID statement vs statement/non-statement, derived-ID `>= DERIVED_RELATION_ID_BASE` statement vs closed-ID statement, and `Unresolved` vs statement/none).
- Extracted `pub fn resolve_recall_line` (called by `GroundedSession::emitter_suffix`) in `crates/uor-r4-training/src/stack_grounded_session.rs`, enabled `pub mod stack_grounded_session;` in `crates/uor-r4-training/src/lib.rs`, documented the `StoreRead::Absent` vs `MemoryEffect::Unresolved` log-sieve coupling on `MemoryEffect`, `RecallDisposition`, and `resolve_recall_line`, and added unit test `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it`.
- Added `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` documenting both behaviors and their tests, and set `status: done` in `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md`.
- Diff size by class: ~350 lines of tests, ~160 lines of code (`resolve_op_policy`, `resolve_recall_line`, and `#[cfg(any())]` gates), ~140 lines of docs (`docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, `ORDER.md`, `REPORT.md`, doc comments), 0 lines generated; total ~650 lines against the ORDER's 200-line estimate (ratio 3.25x, driven by extracting `resolve_op_policy`/`resolve_recall_line` and gating historical items in `relation_compiler.rs` and `stack_grounded_session.rs` so the modules compile under `lib.rs`).

## Measurements

- Host unit tests (`CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo test -p uor-r4-training --lib` on macOS aarch64): 37 passed, 0 failed, 0 ignored in 1.01s (including 23 tests in `relation_compiler::tests` and 1 test in `stack_grounded_session::tests`).
- Negative gate runs on the same host target: 3 of 3 targeted mutations caused immediate test failures in `op_policy_precedences_and_derived_id_guard_resolve_disagreements` and `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it`.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-training -- --check`: exit 0.
- `CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo test -p uor-r4-training --lib`: 37 passed, 0 failed.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- None for `#2857`; `m3-policy-precedence-and-sieve-tests` is complete (`status: done`).

## Plain summary

- In the relation compiler (`crates/uor-r4-training/src/relation_compiler.rs`), a turn can be classified both by a phrase/table route and by a generated operation line (`Op: ...`), and an `OpPolicy` (`Op`, `Table`, `TableStatements`, or `UnlessQuery`) chooses which prediction wins when they disagree. In addition, open relations derive a relation identifier at or above `DERIVED_RELATION_ID_BASE` (`1_000_000`) from the turn's words, and a guard (`!table_derived`) prevents the op model's closed-set label from overwriting a derived open-relation statement.
- In a grounded session (`crates/uor-r4-training/src/stack_grounded_session.rs`), a query turn that resolves to a relation identifier whose slot in the exact store (`StackStore`) has never been written returns `StoreRead::Absent` ("proven store absence") and emits `"Memory: none."` (`RecallDisposition::Absent`) without consulting the optional conversation-log text sieve (`log_recall`), whereas a turn that the compiler leaves `MemoryEffect::Unresolved` consults `log_recall` and emits `RecallDisposition::LogValue`.
- Previously, both modules were disabled in `crates/uor-r4-training/src/lib.rs` with `#[cfg(any())]`, so neither behaviour ran in `cargo test -p uor-r4-training --lib`. This change enables both modules in `lib.rs` (gating only their retired historical float/trunk items), extracts `resolve_op_policy` and `resolve_recall_line`, documents the store-miss vs log-sieve contract, and pins both rules with unit tests verified against three deliberate breakages.
