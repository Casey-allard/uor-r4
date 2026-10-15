# #166 — Compiler stage ownership and parallelization DAG (parallel-safe / deterministic-merge / bounded / canonical finalization)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:43:19Z
- **Updated:** 2026-07-25T01:17:43Z
- **Closed:** 2026-07-25T01:17:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/166
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor abstraction).
Source: plan §4.1, §5 (phase structure); compiler pipeline per #129/#130; direction PDF §§7, 11.

## Motivation

The compiler pipeline (`crates/uor-r4-graph-compiler/src/{observation,observation_text,induction,quantum_cover,residual,routing,pack,graph,patch_induction,perturbation}.rs` plus `uor-r4-graph-certify`) has no documented concurrency model: nobody can say which stages may run in parallel, which need a deterministic merge, and which must stay sequential to protect canonical bytes. Before any stage is parallelized (#170–#173), the stage inventory needs one normative dependency graph with a declared concurrency class per stage and an owner for every parallel boundary.

## Scope

Formally classify every compiler stage into one of four classes:

- **parallel-safe** — embarrassingly independent units, order-preserving output;
- **parallel with deterministic merge** — parallel discovery plus stable-sort/dedup/ordered-reduction finalization;
- **bounded parallel** — parallelism limited by memory budget (#169) or backend behavior (teacher probing);
- **sequential canonical finalization** — must run single-threaded to protect canonical form.

Stage inventory to review (at minimum): corpus partitioning; teacher-probe request preparation; trace normalization; contextual feature extraction; behavioral fingerprint generation; paraphrase and counterfactual analysis; distance and divergence calculation; nearest-neighbor/candidate-neighbor search; recursive clustering; region proposal; overlap discovery; parent/child relationship discovery; transition discovery; XOR-polynomial and routing-program search; mask and threshold search; radius calibration; collision analysis; shortlist-recall evaluation; region emission compilation; residual compilation; quantization analysis; empirical certification; graph-fragment construction; artifact section construction; canonical sorting; node/edge ID assignment; final offset calculation; final serialization; root hashing; signing.

The following are **expected to remain deterministic finalization steps unless proven otherwise**: canonical sort; deterministic deduplication; stable ID assignment; final offset assignment; final artifact packing; root hashing; signing. Any proposal to parallelize one of these carries the burden of proof (byte-equality evidence per #167).

Deliverable form: a documented stage DAG (in `docs/`, cross-linked from the plan) mapping each stage to its class, its crate/module, its inputs/outputs, its concurrency justification, and the owner of each parallel boundary.

## Non-goals

- Implementing the parallelism (that is #170–#173).
- The executor mechanism (#165), thread configuration (#168), memory model (#169).
- Reordering the pipeline itself or changing stage semantics — this issue classifies the existing/planned pipeline, it does not redesign it.

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/*` (module list above), `crates/uor-r4-graph-certify/src/*`, `crates/uor-r4-graph-cli`
- `docs/r4_graph_compiler_implementation_plan.md` (stage DAG section), monograph (#133) compiler-stages chapter

## Dependencies

- #165 (executor the classified stages will run on).
- Consumed by: #167 (reproducibility harness targets the DAG's finalization spine), #169 (per-stage memory estimates attach to DAG nodes), #170–#173 (implement the parallel-safe/merge stages), #175 (benchmarks report per-DAG-node scaling).

## Deliverables

1. Versioned stage DAG document with the four-class classification for every stage above.
2. Explicit ownership assignment for each parallel boundary (which issue/module owns each edge).
3. Per-stage inputs/outputs and determinism justification, cross-linked to modules.

## Acceptance criteria

- Every stage in the inventory is classified; no stage is left "unspecified."
- The seven finalization steps are classified sequential-canonical-finalization or carry byte-equality proof obligations.
- The DAG is checked for consistency: no stage classified parallel-safe consumes completion-order-dependent input.
- Each parallel boundary names its owning issue (#170/#171/#172/#173) and its module.
- Document merged with four gates green + claim-wording check.

## Formal claim classification

- The classification and DAG: **Definition**.
- "Finalization spine is sequential and canonical": **Guarantee**, status **Structural** once #167's harness exists; **Unproven** until then.

## Test requirements

- **Unit:** none directly (documentation deliverable); a doc-consistency test may assert the DAG covers every compiler module.
- **Property / Reproducibility / Benchmark:** none here; owned by downstream issues.
- **BDD/Cucumber:** none.

## Memory considerations

The DAG is the attachment point for #169's per-stage memory estimates; stages holding full-corpus state must be marked.

## Determinism considerations

The DAG makes the chain-wide rule structural: parallel discovery never assigns canonical IDs; deterministic merge and canonical finalization are explicit DAG nodes upstream of serialization, hashing, and signing (#167).

## CPU portability and no-GPU statement

The DAG is CPU-native. No stage may be classified in a way that requires CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats. Teacher-backend interaction points are marked as external boundaries (correctness must not depend on a GPU teacher backend; see #174).

## Traceability

- Plan §4.1/§5; direction PDF §§7, 11; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#21, #59, #122, #129, #130, #157, #165, #167, #168, #169, #170, #171, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

