# #172 — Parallel XOR-polynomial and routing-program synthesis with deterministic selection

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:11Z
- **Updated:** 2026-07-25T05:13:46Z
- **Closed:** 2026-07-25T05:13:46Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/172
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #167 (reproducibility), #168 (jobs), #169 (memory), #171 (regions being routed).
Extends: #15 (masked-Hamming calibrated radii), #130 (Boolean lowering), plan Phase 6 (Boolean routing, trigger-gated D5).
Source: plan §4.1, Phase 6; direction PDF §11; R4G1.md ROUT section validation.

## Motivation

Routing-program synthesis is a search problem: XOR tap sets, masks, thresholds, and radii are candidate spaces whose evaluation is independent per candidate — ideal CPU parallelism. But final selection decides ROUT bytecode that ships in the artifact, so selection must be insensitive to evaluation order and thread count. Nothing currently specifies candidate ordering, tie-breaking, or reduction for this search. This issue owns candidate-search parallelism only; clustering (#171) and emission compilation (#173) are separate owners.

## Scope

CPU-parallel search across:

- XOR tap sets; polynomial candidates; masks; thresholds; radii;
- Boolean routing programs; decision-DAG candidates;
- collision behavior; semantic-neighbor recall; shortlist recall;
- runtime operation cost; artifact-size cost; cache-cost estimates.

Each candidate or candidate batch evaluates independently; **final selection is deterministic**:

- **stable candidate ordering** — candidates carry canonical keys (tap-set tuple, mask bits, threshold) sorted before selection;
- **deterministic tie-breaking** — documented total order over (quality, cost, canonical key);
- **deterministic reduction** — best-candidate reduction by canonical comparison, never first-finished;
- **bounded candidate batches** — per #169 budget model;
- **no GPU kernels, no tensor runtime dependency** (statement below);
- **benchmark evidence showing CPU scaling** (feeds #175).

## Non-goals

- Changing ROUT semantics or the routing quality objectives (#15/D5 lineage); this parallelizes search over the existing objective.
- Clustering parallelism (#171); emission/residual compilation (#173).
- Any GPU kernel, shader, or tensor-runtime candidate evaluation (explicit non-goal).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/routing.rs` (currently a stub — 7 lines), routing-related sections of `induction.rs`; R4G1.md ROUT validation (`crates/uor-r4-graph-format/src/stage2.rs`)
- plan Phase 6 (D5 trigger conditions)

## Dependencies

- #165, #166, #167, #168, #169, #171.
- Consumed by: #173 (emission compilation follows routing), #175 (candidates/sec metrics).

## Deliverables

1. Candidate-space definitions with canonical keys and the documented total order for selection.
2. Batch-parallel candidate evaluation on the #165 executor within #169 budgets.
3. Deterministic selection implementation (canonical reduction + tie-breaking).
4. Identical-winner evidence across thread counts; CPU scaling benchmarks.

## Acceptance criteria

- Identical winner selection (same ROUT bytecode) across 1/2/4/N threads and across repeated runs.
- Stable candidate ordering and documented deterministic tie-breaking; reduction never depends on completion order.
- Candidate batches bounded per #169; no unbounded candidate queues.
- Benchmark evidence of CPU scaling (candidates evaluated/sec vs. thread count) reported to #175.
- No GPU kernels or tensor-runtime dependency anywhere in the search.
- Four gates green; #167 harness passes on routing-heavy fixtures.

## Formal claim classification

- Identical winner selection under parallelism: **Guarantee**, status **Structural** (harness-backed).
- Routing quality (recall/collision rates) and search speedup: **Empirical Criterion**.

## Test requirements

- **Unit:** canonical key ordering, tie-break correctness, reduction determinism.
- **Property:** selection invariant under candidate-evaluation-order permutation.
- **Reproducibility:** #167 harness; identical ROUT sections across thread counts.
- **BDD/Cucumber:** none required (search internals; artifact-level behavior covered by #167's BDD).
- **Benchmark:** candidates/sec scaling sweep (feeds #175).

## Memory considerations

Candidate batches sized by the #169 budget; per-candidate evaluation is scratch-reuse, not per-candidate allocation.

## Determinism considerations

Evaluation order is free; selection order is canonical. Cost estimates (op/bytes/cache) use deterministic integer or fixed-order arithmetic; no order-sensitive FP reduction feeds a canonical decision (#167 policy).

## CPU portability and no-GPU statement

CPU-native search. No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, GPU shader compilation, tensor accelerators, neural engines, vendor BLAS, GPU-resident intermediates, or accelerator-specific formats.

## Traceability

- Plan §4.1, Phase 6 (D5); direction PDF §11; R4G1.md ROUT validation; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#15, #122, #130, #157, #165, #166, #167, #168, #169, #171, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

