# REPORT: m3-policy-precedence-and-sieve-tests, antigravity-7

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2857.

## What did not work, first

- In `antigravity-6`, `REPORT.md` and `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` stated that `resolve_op_policy` is "called by `SavedCompiler::action`" and `resolve_recall_line` is "called by `GroundedSession::emitter_suffix`", whereas both callers remain under `#[cfg(any())]` (`SavedCompiler::action` at `relation_compiler.rs:2349` and `GroundedSession::emitter_suffix` at `stack_grounded_session.rs:832`) and are neither built nor tested; only the extracted helper rules are built and tested.
- In `antigravity-6`, `crates/uor-r4-training/src/lib.rs` enabled `pub mod relation_compiler;` and `pub mod stack_grounded_session;` unconditionally in the non-test library build, altering the native qualification graph's module surface without a PM decision card, and `resolve_recall_line`'s doc comment asserted the store-miss / sieve coupling as settled rather than descriptively documenting the current behaviour pending PM decision.
- Negative gate 1 (swapping `OpPolicy::TableStatements` and `OpPolicy::UnlessQuery` in `resolve_op_policy`): `cargo test -p uor-r4-training --lib op_policy_precedences_and_derived_id_guard_resolve_disagreements` failed as expected (`left: Assert { relation: 1, span: SourceSpan { start: 20, end: 25 } }`, `right: Unresolved { reason: "the turn states no relation" }`), then passed when restored.
- Negative gate 2 (dropping the `!table_derived` guard in `resolve_op_policy`): `cargo test -p uor-r4-training --lib op_policy_precedences_and_derived_id_guard_resolve_disagreements` failed as expected (`left: Correct { relation: 4, span: SourceSpan { start: 11, end: 17 } }`, `right: Assert { relation: 12957859, span: SourceSpan { start: 11, end: 17 } }`), then passed when restored.
- Negative gate 3 (allowing `StoreRead::Absent` to consult `log_recall` in `resolve_recall_line`): `cargo test -p uor-r4-training --lib store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` failed as expected (`left: Some("Memory: from-sieve:put blue:absent.")`, `right: Some("Memory: none.")`), then passed when restored.

## What changed

- Gated `pub mod relation_compiler;` and `pub mod stack_grounded_session;` in `crates/uor-r4-training/src/lib.rs` with `#[cfg(test)]` so the non-test native qualification library graph is unchanged while `cargo test -p uor-r4-training --lib` compiles and runs all 23 `relation_compiler` and 1 `stack_grounded_session` unit tests.
- Updated the doc comments on `resolve_op_policy` (`crates/uor-r4-training/src/relation_compiler.rs`) and `MemoryEffect`, `RecallDisposition`, and `resolve_recall_line` (`crates/uor-r4-training/src/stack_grounded_session.rs`), as well as `docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, to state explicitly that `SavedCompiler::action` and `GroundedSession::emitter_suffix` remain `#[cfg(any())]`, unbuilt and untested, that the tests pin the extracted helper functions directly, and that the store-miss vs `log_recall` coupling is documented descriptively pending the PM decision card on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675`).
- Added the `m3-durable-memory` notebook row to the in-scope `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` (with the scope/notebook question queued on the PM decision card in `#2028`), and reverted `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` and `docs/roadmap/README.md` to `status: ready` until the PM decision card is settled.
- Diff size by class: ~350 lines of tests, ~165 lines of code (`resolve_op_policy`, `resolve_recall_line`, and `#[cfg(test)]` / `#[cfg(any())]` gates), ~220 lines of docs (`docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, `ORDER.md`s, `REPORT.md`s, doc comments), 0 lines generated; total ~735 lines against the initial ORDER's 200-line estimate (ratio 3.68x, driven by extracting the pure helpers, gating historical items inside `relation_compiler.rs` and `stack_grounded_session.rs` under `#[cfg(test)]`, and adding the `antigravity-7` repair order/report).

## Measurements

- Host non-test library check (`CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo check -p uor-r4-training --lib` on macOS aarch64): exit 0 (`relation_compiler` and `stack_grounded_session` remain excluded from the non-test library build).
- Host unit tests (`CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo test -p uor-r4-training --lib` on macOS aarch64): 37 passed, 0 failed, 0 ignored in 1.01s (including 23 tests in `relation_compiler::tests` and 1 test in `stack_grounded_session::tests`).

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-training -- --check`: exit 0.
- `CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo check -p uor-r4-training --lib`: exit 0.
- `CARGO_TARGET_DIR=local/antigravity-target ~/.cargo/bin/cargo test -p uor-r4-training --lib`: 37 passed, 0 failed.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render --check`: exit 0.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` stays `status: ready` pending the PM decision card in `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675`) on (1) whether `#[cfg(test)]` helper pinning satisfies `#2857` or whether `SavedCompiler::action` / `GroundedSession::emitter_suffix` should be re-admitted, (2) whether the store-miss vs `log_recall` coupling is settled as intended, and (3) whether to widen `scope:` / `notebook:` to include `docs/research/notebook/m3-durable-memory.md`.

## Plain summary

- In `crates/uor-r4-training/src/relation_compiler.rs`, `resolve_op_policy` extracts the policy-resolution rule from the historical `SavedCompiler::action` (`#[cfg(any())]`, which remains unbuilt and untested) so the four `OpPolicy` variants (`Op`, `Table`, `TableStatements`, `UnlessQuery`) and the `!table_derived` guard (`>= DERIVED_RELATION_ID_BASE`) can be unit-tested directly under `#[cfg(test)]`.
- In `crates/uor-r4-training/src/stack_grounded_session.rs`, `resolve_recall_line` extracts the recall-line rule from the historical `GroundedSession::emitter_suffix` (`#[cfg(any())]`, which remains unbuilt and untested), descriptively documents the current code's behaviour (`StoreRead::Absent` emits `"Memory: none."` without calling `log_recall`, while `MemoryEffect::Unresolved` consults `log_recall`), and pins that behaviour with a unit test under `#[cfg(test)]`.
- Both modules are gated with `#[cfg(test)]` in `crates/uor-r4-training/src/lib.rs` so the non-test native qualification library graph is unchanged, the `m3-durable-memory` notebook row is recorded in `docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, and `status: ready` is held on `#2857` pending the PM decision card on `#2028`.
