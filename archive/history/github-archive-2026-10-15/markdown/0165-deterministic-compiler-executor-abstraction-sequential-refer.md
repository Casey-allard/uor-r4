# #165 — Deterministic compiler executor abstraction (sequential reference + Rayon CPU executors)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:42:36Z
- **Updated:** 2026-07-25T00:41:37Z
- **Closed:** 2026-07-25T00:41:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/165
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: plan §4.1 (parallelism partitions by content-addressed sample ID with ordered reductions), R4G1.md §7 (D2 reproducibility), direction PDF §§7, 11; precursors #21 (deterministic rebuild CI), #59 (content-addressed observation IDs), #129 (ordered reductions, deterministic compiler output).

## Motivation

`rayon` is already declared in `crates/uor-r4-graph-compiler/Cargo.toml` (1.8) and `crates/uor-r4-graph-certify/Cargo.toml` (1.10) but is **unused** — no `par_iter`/`ThreadPool` call exists anywhere in the workspace, there is no executor abstraction, and the two declared versions already drift. Every compiler-parallelism issue in this chain (stage DAG, thread config, memory budget, the four parallel-stage issues, benchmarks) presupposes one seam where deterministic algorithms meet CPU concurrency. This issue defines that seam so the core algorithms stay coupled to a stable trait, not to Rayon APIs.

## Scope

Define a compiler execution abstraction supporting both a sequential reference executor and a Rayon-backed CPU executor. Representative direction (adapt to repository conventions; do not mandate this exact trait):

```rust
pub trait CompilerExecutor {
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Result<Vec<O>, CompileError>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> Result<O, CompileError> + Sync;
}
```

Required elements:

- **`SequentialExecutor`** — the normative reference implementation; simple loop, zero concurrency, used by tests and by the reproducibility baseline (#167).
- **`RayonExecutor`** — CPU executor over an explicitly owned Rayon `ThreadPool` (not the global pool, unless a documented architecture decision justifies it).
- **Deterministic error aggregation** — errors collected and returned in stable input order, not completion order; typed `CompileError` taxonomy only.
- **Cancellation behavior** — defined semantics for aborting in-flight work on first error or explicit cancellation; no poisoning-sensitive global state.
- **Panic containment or prohibition** — panics in worker closures are either contained and surfaced as typed errors or prohibited by lint + test; no panic-based compiler control flow.
- **Thread-pool ownership, naming, and sizing** — the executor owns its pool; pool configuration comes from #168 (jobs config), not from ambient environment.
- **Stable shard ordering** — `map` results correspond positionally to inputs regardless of execution order; shard IDs are deterministic functions of input position or content address, never of scheduling.
- **Backend neutrality** — compiler algorithms depend only on the trait; a future alternate backend (or a test-only instrumented executor) plugs in without algorithm changes. No GPU executor: the trait's contract is CPU-native, and a GPU backend is a non-goal per the no-GPU statement below.
- **Reproducibility tests** — sequential vs. Rayon executor produce identical outputs for identical inputs (the harness that #167's artifact-level byte-equality builds on).

Also in scope: reconcile or remove the currently unused `rayon` declarations (1.8 vs 1.10) so the dependency appears exactly where the executor lives, at one pinned version.

## Non-goals

- Classifying which compiler stages parallelize (that is #166's stage DAG).
- Jobs/CLI/env thread configuration (#168) and memory budgeting (#169) — this issue consumes their outputs.
- Async runtimes (tokio exists only as a dev-dependency for BDD; no async compiler executor).
- Any GPU, accelerator, shader, or tensor executor (explicit non-goal).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler` (new executor module; `Cargo.toml` rayon declaration), `crates/uor-r4-graph-certify/Cargo.toml` (version reconciliation), `crates/uor-r4-graph-cli` (executor selection surface, wired by #168)
- `docs/r4_graph_compiler_implementation_plan.md` §4.1, `docs/transformerless/R4G1.md` §7

## Dependencies

- Foundation for: #166 (stage DAG), #167 (reproducibility), #168 (jobs config), #169 (memory budget), #170–#173 (parallel stages), #175 (benchmarks).
- Aligns with the CPU-only normative stance in #157 §6 (runtime contract); the compiler-side dependency audit is #174.

## Deliverables

1. `CompilerExecutor` trait (or repo-idiomatic equivalent) with `SequentialExecutor` and `RayonExecutor`.
2. Typed error aggregation with stable ordering; cancellation semantics documented and tested.
3. Pool ownership/naming/size wiring (configuration surface finalized with #168).
4. Reproducibility test harness comparing executors.
5. Reconciled rayon dependency (single pinned version, single home).

## Acceptance criteria

- Identical outputs (graph IDs, ordering, offsets, hashes, certificates) from `SequentialExecutor` and `RayonExecutor` on pinned fixtures, across thread counts.
- Errors surface as typed `CompileError` values in deterministic input order; worker panics cannot escape as process aborts (contained or prohibited, tested).
- No hidden global Rayon pool dependency unless a written decision in the plan/docs justifies it.
- Compiler algorithm code imports no Rayon APIs directly (code review + a lint or dependency-layer check).
- No GPU dependency or GPU executor anywhere in the abstraction.
- Four gates green: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo test --workspace --offline`, plus claim-wording checks.

## Formal claim classification

- Executor output equivalence (sequential ≡ parallel): **Guarantee**, status **Structural** (test-backed, extended by #167).
- The abstraction itself: **Definition**.

## Test requirements

- **Unit:** positional result mapping, error ordering, cancellation, panic containment, pool sizing/naming.
- **Property:** `map` over randomized inputs yields identical vectors under both executors for pure functions.
- **Reproducibility:** sequential vs. Rayon over the pinned mini-corpus fixtures (byte-level on serialized outputs).
- **BDD/Cucumber:** not required at this layer (consumed by higher-level BDD in #170–#173).
- **Benchmark:** deferred to #175.

## Memory considerations

The executor itself adds no per-task allocation policy; worker-local scratch and in-flight bounds are #169's model — the trait must not preclude them (e.g. no API that forces unbounded result buffering).

## Determinism considerations

The trait contract states the chain-wide rules: no canonical IDs during parallel discovery; no completion-order semantics; no shared mutable accumulation; no order-sensitive floating-point reductions for canonical output; thread count never influences results. Canonicalization happens downstream of the executor, before IDs/offsets/hashing/signing (#167).

## CPU portability and no-GPU statement

CPU-native only. The compiler (including this executor) must not require or depend on CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats — as architecture, default features, CI, tests, or production workflow. External teacher backends are out of scope here (see #170/#174).

## Traceability

- Plan §4.1; R4G1.md §7 (D2); direction PDF §§7, 11; monograph (#133) compiler-stages chapter; traceability matrix rows (#137); vocabulary entry for "compiler executor" if #123's spec is extended.

## Related issues and PRs

#21, #59, #129, #122, #157, #166, #167, #168, #169, #170, #171, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

