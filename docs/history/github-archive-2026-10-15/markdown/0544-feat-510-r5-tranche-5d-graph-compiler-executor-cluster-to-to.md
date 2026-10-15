# #544 — feat(#510): R5 tranche 5d — graph-compiler executor cluster to total

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T13:49:48Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-09T14:08:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/544
- **Merged:** 2026-08-09T14:08:37Z
- **Merge commit:** 7b2952d2f778f90f03c626d6a59a9e44e3d4abb6
- **Base / Head:** main / UOR-Foundation:r5/gc-executor
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The largest `uor-r4-graph-compiler` piece: make the compiler executor total, which flows through every parallel-map consumer. **79 → 59.**

## What changed
- **executor.rs** — remove `CompileError`. `CompilerExecutor::map` → `fn map<I,O,F>(&self, inputs, f) -> Vec<O> where F: Fn(&I)->O + Sync`: the worker closure is infallible (no `WorkerError`), and a worker panic **propagates** (Sequential maps directly; Rayon's `par_iter().map().collect()` re-raises the panic in the caller) rather than being contained into an `ExecutionPanic` variant. `RayonExecutor::new -> Self` (a pool that won't build is a host defect → panic). Under R5 a worker panic is a defect that propagates, never a sanctioned condition.
- **Consumers** made total (drop `Result<_,String>`): `residual.rs`, `pack.rs`, `routing.rs` (`synthesize_routing_program` et al. — the fixed 512 MiB budget and non-empty candidate set are documented invariants, hence `.expect`), `reproducibility.rs` (`verify_reproducibility -> ReproducibilityReport`; `is_byte_identical` carries a mismatch; `ReproducibilityError` removed). `induction.rs` `emit_r4g1` drops the `?` on the now-total routing (it stays `Result` — its other legs are a later tranche).
- **Ripple**: graph-certify `certificate.rs` (adapts to the total executor, keeps its own `Result` for now); proof-model `verify_compiler_executor_compliance` / `verify_parallel_reproducibility_compliance`; `tests/bdd.rs` + `features/suites/compiler_executor.feature` — the worker-error scenario is removed (no fallible worker) and the panic scenario now asserts the panic **propagates** (`catch_unwind`).

## Verification
- `cargo build --workspace --all-targets` clean; `cargo build -p uor-r4-graph-compiler -p uor-r4-graph-certify --target wasm32-unknown-unknown` clean (Rayon is cfg-not-wasm; wasm uses Sequential)
- graph-compiler 86 lib tests + proof-model 18 lib tests pass; **BDD 100/100 scenarios, 333/333 steps**; `clippy -D warnings` clean; `cargo fmt --check` clean
- `audit-deferral` (R4) passes; `audit-limits` graph-compiler **79 → 59**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

