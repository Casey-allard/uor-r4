# ORDER: m3-policy-precedence-and-sieve-tests, antigravity-7

Lab: antigravity. Item: [m3-policy-precedence-and-sieve-tests](../../../../roadmap/items/m3-policy-precedence-and-sieve-tests.md). Base: `f1c78da8e`.

## Goal

Repair order `antigravity-7` addressing Claude's `CHANGES` review on PR `#2947` (`https://github.com/UOR-Foundation/uor-r4/pull/2947#issuecomment-6335377863`).

The item's done-when reads:

> Unit tests for both precedences and a documented, tested store-miss and sieve behaviour.

## Cancel probe (run first)

Ran `uor-harness loop review --check --pr 2947` and inspected Claude's `CHANGES` review (`https://github.com/UOR-Foundation/uor-r4/pull/2947#issuecomment-6335377863`):
1. `REPORT` and `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` stated that `resolve_op_policy` is called by `SavedCompiler::action` and `resolve_recall_line` is called by `GroundedSession::emitter_suffix`, whereas both callers remain under `#[cfg(any())]` and are unbuilt and untested; the tests pin the extracted helper functions directly.
2. `crates/uor-r4-training/src/lib.rs` enabled `pub mod relation_compiler;` and `pub mod stack_grounded_session;` in the non-test library build without a PM decision card, and `resolve_recall_line`'s doc comment asserted the store-miss / sieve coupling as intended rather than descriptively documenting the current behaviour pending PM decision (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675`).
3. `docs/research/notebook/m3-durable-memory.md` is outside the item's `scope:`; the row is recorded in the in-scope `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` and queued on the PM decision card in `#2028`.
4. Hold `status: ready` on `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` until the PM decision card is settled.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/labs/m3-policy-precedence-and-sieve-tests/**`

## Done when

- `pub mod relation_compiler;` and `pub mod stack_grounded_session;` in `crates/uor-r4-training/src/lib.rs` are gated with `#[cfg(test)]` so the non-test native qualification library graph is unchanged while `cargo test -p uor-r4-training --lib` compiles and runs all 23 `relation_compiler` and 1 `stack_grounded_session` unit tests.
- Doc comments in `crates/uor-r4-training/src/stack_grounded_session.rs` and `crates/uor-r4-training/src/relation_compiler.rs`, `docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, and `docs/loop/tasks/m3-policy-precedence-and-sieve-tests/antigravity-7/REPORT.md` state explicitly that `SavedCompiler::action` (`:2345`) and `GroundedSession::emitter_suffix` (`:834`) remain `#[cfg(any())]` (unbuilt and untested), that the unit tests pin the extracted helper rules (`resolve_op_policy` and `resolve_recall_line`), and that `resolve_recall_line` documents the current code's store-miss vs `log_recall` coupling descriptively without settling the mechanism question (PM decision card posted at `#2028#issuecomment-6335479675`).
- `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` includes the `m3-durable-memory` notebook row in-scope and links the PM decision card.
- `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` holds `status: ready` (with `docs/roadmap/README.md` re-rendered to match) while the PM decision card is pending.

## Stop and ask if

- Any file outside `crates/uor-r4-training/**`, `docs/labs/m3-policy-precedence-and-sieve-tests/**`, `docs/loop/tasks/m3-policy-precedence-and-sieve-tests/**`, or the item/roadmap files must change.
- Any GPU run or change to serving behavior is required.
