# #173 — Parallel emission, residual, quantization, and certificate compilation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:13Z
- **Updated:** 2026-07-25T05:13:46Z
- **Closed:** 2026-07-25T05:13:46Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/173
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #167 (reproducibility), #168 (jobs), #169 (memory), #171 (discovered graph), #172 (routing programs).
Extends: #19 (ScoreQ residuals), #64/#68/#69 (residual redesign lineage), #20 (certificate schema), #130 (lowering).
Source: plan §4.1; R4G1.md §§4-6 (HEAD bounds, sections, validation); direction PDF §§7, 11.

## Motivation

Emission-table compilation, residual computation, quantization, and certificate generation are per-node/per-region/per-shard workloads — naturally parallel — but their outputs land directly in canonical artifact sections and CID-addressed certificates (`crates/uor-r4-graph-compiler/src/{residual.rs,pack.rs}`, `crates/uor-r4-graph-certify`). An order-sensitive accumulation here silently changes artifact bytes or certificate hashes. This issue owns that compilation layer's parallelism; observation shards (#170), clustering (#171), and routing search (#172) are separate owners.

## Scope

Parallel processing by node, region, transition, or evaluation shard for:

- token-emission tables;
- parent/base residuals;
- overlap residuals (no-double-counting per #158 semantics);
- transition score residuals;
- fixed-point quantization (ScoreQ Q16.16 + per-table storage descriptors, R4G1.md §9.3);
- pruning;
- candidate-recall measurement;
- empirical certificate generation;
- operation-cost estimation;
- proof/certificate fragment construction.

Constraints:

- final artifact ordering, offsets, and hashes remain canonical — fragment outputs are immutable and sorted before packing (per #166's finalization spine);
- **no shared order-sensitive accumulation** anywhere in residual or certificate construction;
- quantization results deterministic (fixed rounding mode, fixed operation order; no order-sensitive FP in quantization of canonical values per #167);
- pruning decisions derive from canonical keys and documented tie-breaks, never completion order;
- CPU-only implementation (statement below).

## Non-goals

- Residual/scoring semantics themselves (#158 owns semantics; this issue parallelizes their compilation).
- Certificate schema changes (#161 owns the schema lineage via #20).
- Clustering (#171) and routing search (#172).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/{residual.rs,pack.rs,graph.rs}`, `crates/uor-r4-graph-certify/src/*` (certificate fragment generation), `crates/uor-r4-graph-format` (ArtifactBuilder canonical serializer)
- R4G1.md §§4-6, §9.3

## Dependencies

- #165, #166, #167, #168, #169, #171, #172.
- Consumed by: #175 (certificate shards/sec, finalization-time metrics).

## Deliverables

1. Shard/region-parallel emission, residual, and quantization compilation on the #165 executor.
2. Parallel certificate-fragment generation with canonical assembly.
3. Determinism evidence: identical residuals, quantization results, and certificates across thread counts.

## Acceptance criteria

- Deterministic residual output and deterministic quantization results: byte-identical EMIT/EXCT/residual sections across 1/2/4/N threads (via #167 harness).
- Identical certificates (bytes and CIDs) across thread counts.
- No shared order-sensitive accumulation (review + scheduling-perturbation tests).
- Final ordering/offsets/hashes produced only on the canonical finalization spine.
- CPU-only implementation; no GPU or tensor-runtime dependency.
- Four gates green.

## Formal claim classification

- Deterministic residual/quantization/certificate output under parallelism: **Guarantee**, status **Structural** (harness-backed).
- Quantization-loss values and speedup: **Empirical Criterion** (loss per #158's certificate; speed per #175).

## Test requirements

- **Unit:** quantization rounding stability; fragment assembly order; residual section equality.
- **Property:** fragment merge invariant under interleaving; quantization invariant under batching variation.
- **Reproducibility:** #167 harness over emission-heavy fixtures; certificate byte equality.
- **BDD/Cucumber:** covered at artifact level by #167's scenario.
- **Benchmark:** regions/sec, certificate shards/sec (feeds #175).

## Memory considerations

Per-region/per-shard work units sized per #169; certificate fragments streamed or bounded, not accumulated unboundedly.

## Determinism considerations

All canonical quantities (scores, residuals, quantized values, certificate fields) computed with fixed operation order or integer arithmetic; assembly strictly through sort → dedup → sequential ID/offset assignment (#166 spine).

## CPU portability and no-GPU statement

CPU-native. No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, GPU-resident intermediates, or accelerator-specific formats.

## Traceability

- Plan §4.1; R4G1.md §§4-6, §9.3; direction PDF §§7, 11; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#19, #20, #64, #68, #69, #122, #130, #157, #158, #161, #165, #166, #167, #168, #169, #171, #172, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

