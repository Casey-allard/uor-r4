# #168 — Compiler thread-pool, jobs configuration, and oversubscription policy

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:43:50Z
- **Updated:** 2026-07-25T02:50:33Z
- **Closed:** 2026-07-25T02:50:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/168
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor).
Source: plan §4.1; `crates/uor-r4-graph-cli` (CLI surface), AGENTS.md command conventions.

## Motivation

Compiler concurrency currently has no controls: no `--jobs`, no env override, no named threads, no policy for oversubscription when libraries (or an external teacher backend) thread internally. Multicore compilation without explicit controls produces CI flakiness, laptop-unfriendly defaults, and irreproducible performance claims (#175 needs controlled thread counts to measure anything). This issue owns the configuration surface; it does not own the executor (#165) or the memory model (#169).

## Scope

Define compiler concurrency controls:

- **Dedicated Rayon thread pool** owned by the executor (#165) where appropriate; **no hard dependency on the Rayon global pool** unless a written architecture decision in the plan justifies it.
- **`--jobs N` CLI option** on the compiler CLI (`uor-r4-graph-cli`), with `N` validated (typed error on invalid values).
- **Environment-variable override** `R4_COMPILER_THREADS`, with documented precedence (CLI > env > default).
- **Deterministic default thread-count policy** — the default is a documented function (e.g. logical CPUs, capped); it affects time only, never output (per #167).
- **Thread naming** for diagnosability (e.g. `r4-compile-<n>`).
- **Oversubscription handling:** document or detect interaction with libraries that may already use threads (blake3, teacher-backend clients, nextest/CI parallelism); define expected behavior when a teacher backend batches internally (compiler-side concurrency stays bounded regardless).
- **Bounded queues** between stages; **cancellation and graceful error propagation** through the #165 executor.
- **Memory-aware limiting hooks:** the jobs configuration exposes the knob #169's budget model uses to cap expensive in-flight tasks.
- **Optional CPU-affinity research:** recorded as future work, explicitly not required initially.
- **No GPU device-count or GPU scheduling logic** is introduced anywhere in the configuration surface.

## Non-goals

- The executor itself (#165); memory budgets (#169); per-stage parallelism (#170–#173).
- Async runtimes; distributed compilation; GPU scheduling (non-goal).

## Affected crates/modules/docs

- `crates/uor-r4-graph-cli`, `crates/uor-r4-graph-compiler` (config types), `.github/workflows/ci.yml` (CI thread-count pinning), `docs/r4_graph_compiler_implementation_plan.md`

## Dependencies

- #165 (consumes the executor's pool-ownership seam).
- Consumed by: #169 (budget model needs the knob), #170–#173, #175 (benchmarks sweep the same knob).

## Deliverables

1. Compiler concurrency configuration type with the precedence chain above.
2. `--jobs N` + `R4_COMPILER_THREADS` wiring in the CLI; named, owned thread pool.
3. Oversubscription documentation (or detection) including teacher-backend batching behavior.
4. Tests for configuration parsing, precedence, and typed errors.

## Acceptance criteria

- `--jobs`, env override, and default policy behave as documented; invalid values produce typed errors.
- Thread count provably affects timing only: outputs remain byte-identical across settings (delegated to #167's harness, run here across the configuration surface).
- No use of the Rayon global pool unless the documented decision says so.
- No GPU device-count, GPU detection, or GPU scheduling logic exists in the configuration code (checked by #174's audit).
- Four gates green.

## Formal claim classification

- The configuration surface: **Definition**.
- "Thread count affects time only": **Guarantee**, status **Structural** (via #167's harness).

## Test requirements

- **Unit:** parsing, precedence, validation, naming.
- **Property:** configuration round-trip; default policy determinism.
- **Reproducibility:** harness run across configurations (with #167).
- **BDD/Cucumber:** none required (CLI plumbing; may add a scenario if conventions expect CLI coverage).
- **Benchmark:** consumed by #175.

## Memory considerations

Default policy must be documented against #169's memory model (threads × per-worker scratch ≤ budget).

## Determinism considerations

Configuration never changes semantic output — only scheduling; the issue states this as a contract, verified by #167.

## CPU portability and no-GPU statement

Controls are CPU-only: no CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats; no GPU device-count or scheduling logic is introduced.

## Traceability

- Plan §4.1; monograph (#133) compiler-stages chapter; traceability matrix (#137).

## Related issues and PRs

#58, #122, #157, #165, #166, #167, #169, #170, #171, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

