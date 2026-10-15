# #181 — feat(compiler): add deterministic compiler executor abstraction (#165)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T00:22:35Z
- **Updated:** 2026-10-09T14:16:53Z
- **Closed:** 2026-07-25T00:41:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/181
- **Merged:** 2026-07-25T00:41:35Z
- **Merge commit:** 2bf9a1bd727e9180a91615b9dab95a0a28a52497
- **Base / Head:** main / UOR-Foundation:issue-165-compiler-executor
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #165. Adds a backend-neutral `CompilerExecutor` abstraction supporting both a normative `SequentialExecutor` reference implementation and a multicore `RayonExecutor` CPU implementation.

## Key Changes
1. **Executor Abstraction (`crates/uor-r4-graph-compiler/src/executor.rs`)**:
   - Defines `CompilerExecutor` trait with positional output mapping.
   - Implements `SequentialExecutor` (zero-concurrency reference implementation).
   - Implements `RayonExecutor` with owned `ThreadPool` configuration.
   - Implements deterministic error aggregation (by lowest input index).
   - Implements worker panic containment via `std::panic::catch_unwind`.
2. **Dependency Reconciliation**: Reconciled `rayon` dependencies across workspace crates to pinned `1.10.0`.
3. **Proof Model Integration (`crates/uor-r4-proof-model/src/structural_guarantees.rs`)**: Added `verify_compiler_executor_compliance`.
4. **Formal Vocabulary (`docs/formal_vocabulary.md`)**: Updated to v0.1.6.
5. **Cucumber BDD Suite (`features/suites/compiler_executor.feature`)**: Added scenarios for positional equivalence, error ordering, and panic containment.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: Passed (48/48 compiler tests passed)
- `cargo test --test bdd --offline`: Passed (80/80 BDD scenarios passed)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: Clean
- `python3 scripts/check_claim_wording.py`: Passed

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5075806038 — 2026-07-25T00:22:39Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5075806406 — 2026-07-25T00:22:44Z — Casey-allard

@copilot please review this PR

### comment 5075836144 — 2026-07-25T00:29:20Z — Copilot


> @copilot please review this PR

Review complete. Two issues were found and fixed in commit `da8589f`:

**1. Unreachable `&str` arm in `extract_panic_message`** (`executor.rs`)
`std::panic::catch_unwind` captures `Box<dyn Any + Send + 'static>`. A non-`'static` `&str` cannot be stored there — string-literal panics always arrive as `&'static str` and formatted panics as `String`. The intermediate `downcast_ref::<&str>()` branch was dead code and has been removed.

**2. Misleading proof model claim in `verify_compiler_executor_compliance`** (`structural_guarantees.rs`)
The function reported `"SequentialReference ≡ RayonParallel"` in its details string but only ran `SequentialExecutor`. Fixed by:
- Adding an explicit `RayonExecutor` equivalence assertion on non-wasm32 targets so the function actually exercises what it claims.
- Changing the obligation `kind` from `BoundedResource` to `Determinism` (the correct classification for an output-equivalence check).
- Updating the `details` string to precisely describe what is verified.

No other issues found — positional output ordering, deterministic error aggregation by lowest index, panic containment, and BDD coverage are all correctly implemented.


---
_Comment/review ids in this file: 5075806038, 5075806406, 5075836144_

