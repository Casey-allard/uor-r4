# #175 — Compiler parallelism benchmarks and scaling certificates

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:22Z
- **Updated:** 2026-07-25T05:13:46Z
- **Closed:** 2026-07-25T05:13:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/175
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #167 (byte-identical outputs are the premise of any scaling claim), #168 (jobs config), #169 (peak-RSS harness), #170, #171, #172, #173 (the stages being measured), #174 (CPU-only environments).
Extends: #27 (performance certificate fields lineage), #77 (CI trend tracking precedent), #161 (certificate machinery).
Source: plan §8 (acceptance gates), §9.17; `scripts/check_gate_c_regression.py` (trend-alarm precedent).

## Motivation

Parallelizing the compiler (#165–#173) without measurement is unfalsifiable: nobody could say which stages scale, where memory bandwidth saturates, how much time the sequential finalization spine costs, or whether oversubscription hurts. This issue owns the empirical measurement and reporting of compiler parallelism — the only chain issue whose claims are empirical certificates rather than structural guarantees.

## Scope

Define compiler parallelism benchmarks and scaling certificates. Measure at minimum:

- wall-clock compilation time; CPU utilization;
- speedup versus sequential mode; efficiency versus thread count;
- peak RSS (harness per #169);
- bytes processed; observations per second (#170); routing candidates evaluated per second (#172); regions processed per second (#171); certificate shards processed per second (#173);
- canonical finalization time; time spent in sequential bottlenecks (per the #166 DAG).

Benchmark matrix:

- thread counts: 1, 2, 4, and higher where available (via #168's configuration);
- representative small, medium, and large artifacts;
- a memory-constrained configuration (#169).

The report must distinguish:

- stages that scale;
- stages limited by memory bandwidth;
- stages limited by teacher-backend behavior;
- sequential canonical finalization;
- oversubscription effects.

Reporting follows the #27/#161 certificate conventions (CID'd, hardware metadata, honest measurement-vs-estimate labeling) and the #77 trend-alarm precedent for regression detection. **No GPU benchmark is required or desired.**

## Non-goals

- Changing stage implementations (measurement only).
- GPU benchmarking of any kind (explicit non-goal).
- Teacher-backend performance tuning (external boundary; only its *effect* on compiler scaling is characterized).
- Runtime inference benchmarks (owned by the runtime certificate lineage #27/#161).

## Affected crates/modules/docs

- `crates/uor-r4-graph-cli` (benchmark driver surface), `crates/uor-r4-graph-certify` (report/cert format), `scripts/` (trend tracking), `.github/workflows/ci.yml`
- plan §8, §9.17; `docs/transformerless/BASELINE.md`

## Dependencies

- All of #165–#174 (measures their outputs; requires #167's byte-identical premise and #174's CPU-only environments).

## Deliverables

1. Benchmark harness driving full compilation across the matrix above on pinned corpora.
2. Scaling certificate/report format (per-stage scaling, bottleneck classification, hardware metadata).
3. Regression alarms for compiler compile-time and peak-RSS baselines.
4. A documented CPU-only benchmark environment.

## Acceptance criteria

- The full metric list is reported per stage and end-to-end across the 1/2/4/N thread sweep on small/medium/large fixtures plus the memory-constrained configuration.
- Reports classify scaling stages, bandwidth-limited stages, teacher-limited stages, sequential finalization, and oversubscription effects separately — no single composite number hides regressions.
- Every scaling claim is made against byte-identical outputs (#167 harness confirmation attached).
- Regression alarms exist for compile time and peak RSS baselines.
- All benchmark environments are CPU-only; no GPU benchmark exists or is required.
- Four gates green; claim-wording check passes (scaling = empirical, never worded as guarantee).

## Formal claim classification

- Speedup, efficiency, RSS, and throughput numbers: **Empirical Criterion**, status **Empirical** (pinned fixtures, hardware metadata).
- "Parallel outputs are byte-identical to sequential" remains **Structural** (#167) — benchmark reports must not reword it as empirical.

## Test requirements

- **Unit:** report schema round-trip; metric computation on fixture timings.
- **Property:** none required.
- **Reproducibility:** benchmark harness itself runs deterministically (fixed corpora; timing values vary, artifact bytes must not).
- **BDD/Cucumber:** none required (reporting layer; may mirror the Gate C trend precedent).
- **Benchmark:** this issue *is* the benchmark deliverable.

## Memory considerations

Peak-RSS measurement per #169 is a first-class reported metric; the memory-constrained configuration is part of the matrix.

## Determinism considerations

Timing varies; artifacts must not. Every benchmark run attaches #167's byte-equality confirmation for the produced artifact.

## CPU portability and no-GPU statement

Benchmarks run on CPU-only environments. No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats; no GPU benchmark is required or desired.

## Traceability

- Plan §8, §9.17; #27, #77, #161 lineage; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#27, #77, #122, #157, #161, #165, #166, #167, #168, #169, #170, #171, #172, #173, #174; PRs #55, #86.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

