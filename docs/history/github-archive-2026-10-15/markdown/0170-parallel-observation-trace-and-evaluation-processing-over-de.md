# #170 — Parallel observation, trace, and evaluation processing over deterministic shards

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:01Z
- **Updated:** 2026-07-25T05:59:06Z
- **Closed:** 2026-07-25T05:59:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/170
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #167 (reproducibility harness), #168 (jobs), #169 (memory budget).
Extends: #59 (observation pipeline v2: content-addressed IDs, shard spill/resume, teacher trace surface), #72/#90/#91 (corpus observation runs).
Source: plan §4.1 (parallelism partitions by content-addressed sample ID with ordered reductions); direction PDF §§7, 11.

## Motivation

Observation, trace, and evaluation processing is the compiler's most voluminous embarrassingly-parallel work: documents, contexts, and trace shards are independent units already content-addressed per #59. Today they process sequentially (`crates/uor-r4-graph-compiler/src/{observation,observation_text}.rs`, certify evaluation modules). This issue owns coarse-grained CPU parallelism for that layer only — clustering (#171), routing synthesis (#172), and emission/certificate compilation (#173) are separate owners.

## Scope

Parallelize, over independent observations/shards:

- per-document processing; per-context processing; per-trace-shard normalization;
- contextual occurrence extraction; future-distribution summaries;
- counterfactual-difference extraction; paraphrase-invariance measurement;
- semantic-coherence evaluation; OOD evaluation; teacher-agreement evaluation; cross-entropy evaluation; quantization-loss evaluation;
- teacher-probe **request preparation** (batching prompts/requests). Teacher execution itself is external: if an integrated teacher backend exists, this issue's scope is the CPU-native orchestration and analysis around it, the batching interface, and the explicit contract that compilation correctness and supported operation never depend on a GPU teacher backend.

Required constraints:

- **coarse shards, not one task per token** (`par_chunks`-style batching per #169);
- worker-local scratch buffers (reused, per #169);
- deterministic shard IDs (content-addressed or positional per #59, never scheduling-derived);
- deterministic aggregation order (ordered reductions per plan §4.1);
- bounded in-flight work within the #169 budget model;
- **no shared `Mutex<Vec<_>>` accumulation; no atomic counter for semantic IDs; no order-dependent floating-point reductions where byte reproducibility is required** (#167 policy — evaluation metrics feeding canonical output use fixed-order or integer accumulation);
- CPU only, no GPU dependency (see statement below).

## Non-goals

- Teacher-model implementation or backend selection (external; may use whatever the operator chooses — the compiler must function correctly and be fully supported without it).
- Clustering/region induction parallelism (#171); routing-program search (#172); emission/residual compilation (#173).
- Changing observation record formats (#59 owns that lineage).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/{observation.rs,observation_text.rs,perturbation.rs}`, `crates/uor-r4-graph-certify/src/{anti_degeneracy.rs,predictive_sufficiency.rs,compare.rs,long_context.rs,shortlist_evaluator.rs}` (evaluation shards), `crates/uor-r4-model-source`
- plan §4.1; #59's pipeline docs

## Dependencies

- #165, #166, #167, #168, #169.
- Consumed by: #171 (clustering consumes these shards), #175 (obs/sec metrics).

## Deliverables

1. Shard-parallel observation/trace/evaluation pipeline on the #165 executor.
2. Deterministic shard-ID and ordered-reduction implementation.
3. Sequential-vs-parallel equivalence tests (unit outputs and, where shards feed artifacts, via #167's harness).
4. Teacher-backend boundary documentation (CPU-native orchestration; external backend; batching interface; no GPU dependence for correctness).

## Acceptance criteria

- Sequential-vs-parallel equivalence tests pass on pinned fixtures, including thread-count variation (1/2/4/N per #167).
- No shared mutable accumulation, no atomic semantic IDs, no order-sensitive FP reductions in canonical paths (code review + targeted lints/tests).
- Shard counts and per-shard work respect the #169 budget model; in-flight work is bounded.
- Teacher-backend boundary is documented with the three-way distinction (orchestration / external backend / batching interface) and a statement that correctness does not depend on a GPU backend.
- Four gates green.

## Formal claim classification

- Shard equivalence and deterministic aggregation: **Guarantee**, status **Structural** (harness-backed).
- Throughput improvements: **Empirical Criterion** (#175).

## Test requirements

- **Unit:** shard-ID determinism, per-shard output equality, aggregation ordering.
- **Property:** aggregation invariant under shard-count variation for identical input sets.
- **Reproducibility:** via #167 where shards feed canonical output.
- **BDD/Cucumber:** scenario for deterministic observation-shard processing if pipeline conventions expect one.
- **Benchmark:** observations/sec, shards/sec (feeds #175).

## Memory considerations

Shard sizing and worker scratch follow #169; streaming spill per #59 precedent when shards exceed budget.

## Determinism considerations

Shard IDs and aggregation order are content/position-derived only; completion order never influences outputs; canonical quantities avoid order-sensitive FP reduction (#167).

## CPU portability and no-GPU statement

CPU-native. No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, GPU detection or fallback logic, tensor accelerators, neural engines, vendor BLAS, GPU-resident intermediates, or accelerator-specific formats. An external teacher may use any backend the operator chooses; the compiler's correctness and supported operation must not depend on it.

## Traceability

- Plan §4.1; direction PDF §§7, 11; #59; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#59, #72, #122, #128, #129, #157, #165, #166, #167, #168, #169, #171, #172, #173, #174, #175; PRs #90, #91, #95.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

