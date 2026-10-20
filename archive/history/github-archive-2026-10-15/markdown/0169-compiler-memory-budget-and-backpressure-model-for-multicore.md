# #169 — Compiler memory-budget and backpressure model for multicore compilation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:43:51Z
- **Updated:** 2026-07-25T03:32:09Z
- **Closed:** 2026-07-25T03:32:09Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/169
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #168 (jobs configuration).
Extends: the sequential memory-budget machinery in `crates/uor-r4-graph-compiler/src/induction.rs:140-207` (`memory_budget_bytes`, `derive_batch_size`, `regions_budget`) and #129's memory-budget requirement.
Source: plan §4.1; #59 (shard spill/resume precedent).

## Motivation

The existing `--memory-budget` sizes sequential batches. Multicore compilation multiplies peak memory by the worker count unless budgets become concurrency-aware: N workers each holding a shard, scratch, and in-flight candidate batch can silently N× the peak RSS. Nothing today models per-stage memory, caps in-flight work, or defines failure behavior when a budget is exceeded. This issue owns the memory model; #168 owns the thread-count knob it uses.

## Scope

Define a compiler memory-budget and backpressure model:

- **Per-stage memory estimates** attached to the #166 stage DAG (bytes per shard, per worker scratch, per candidate batch).
- **Bounded shard size** and coarse batching (`par_chunks` or equivalent) as the default work-unit shape — never one task per token.
- **Worker-local reusable scratch memory** (per-worker buffers reused across tasks; no per-task allocation storms).
- **Streaming intermediate outputs** where appropriate (shard spill/resume precedent from #59), rather than materializing all results before merge.
- **Maximum in-flight candidate evaluations** — a budget-derived cap; **semaphore or budget-based limiting** of expensive tasks (e.g. large routing-candidate batches, certification shards).
- **Bounded queues** at every stage boundary; **no unbounded queue growth** anywhere in the pipeline.
- **Spill-to-disk research only if needed** — recorded as an option, not a requirement.
- **Deterministic behavior under constrained memory:** same artifact bytes at any budget ≥ some documented minimum; below the minimum, a **typed `BudgetExceeded` error** — never OOM-dependent correctness, never silent quality degradation.
- **Peak-memory benchmarks at multiple thread counts** (acceptance evidence; reporting feeds #175).
- **No GPU memory assumptions** anywhere in the model.

## Non-goals

- The executor (#165), jobs config (#168), stage implementations (#170–#173).
- Swapping/paging policy, OS-level tuning, distributed memory.

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/induction.rs` (existing budget machinery), new budget module; `crates/uor-r4-graph-certify`; `crates/uor-r4-graph-cli` (budget flags)
- plan §4.1, `docs/r4_graph_compiler_implementation_plan.md`

## Dependencies

- #165, #166, #168.
- Consumed by: #170–#173 (each stage implements within the model), #175 (peak-RSS reporting).

## Deliverables

1. Versioned memory-budget model document + budget types with typed `BudgetExceeded` errors.
2. Per-stage estimate table on the stage DAG.
3. In-flight caps (semaphore/budget-based) on expensive task classes; bounded queues.
4. Peak-RSS measurement harness across thread counts and memory-constrained configurations.

## Acceptance criteria

- Peak-memory benchmarks at 1/2/4/N threads and at a deliberately constrained budget are reported and committed as baselines.
- No unbounded queue growth (test: producer faster than consumer stays bounded).
- No OOM-dependent correctness: constrained-budget runs either complete with byte-identical output (per #167) or fail with typed `BudgetExceeded`.
- Budget exceedance failure is deterministic and documented (no retry-dependent semantics; if retry exists it is deterministic).
- Worker scratch is reused, not per-task allocated (allocation-profile evidence).
- No GPU memory concepts in the model.
- Four gates green.

## Formal claim classification

- "Compilation within budget B produces the canonical artifact or a typed budget error": **Guarantee**, status **Structural** (harness-backed).
- Peak-RSS numbers: **Empirical Criterion**.

## Test requirements

- **Unit:** budget arithmetic, cap enforcement, error paths.
- **Property:** output invariance across budget settings above the minimum.
- **Reproducibility:** constrained-budget byte equality (with #167).
- **BDD/Cucumber:** none required.
- **Benchmark:** peak-RSS sweep (feeds #175).

## Memory considerations

This issue is the memory model for the chain.

## Determinism considerations

Budget settings affect scheduling and batching only; canonical output is invariant above the minimum budget (#167 harness covers the constrained-budget leg).

## CPU portability and no-GPU statement

CPU memory only (RSS). No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes, GPU device memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats; no GPU memory assumptions.

## Traceability

- Plan §4.1; #59 (spill/resume); monograph (#133); traceability matrix (#137).

## Related issues and PRs

#59, #122, #129, #157, #165, #166, #167, #168, #170, #171, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

