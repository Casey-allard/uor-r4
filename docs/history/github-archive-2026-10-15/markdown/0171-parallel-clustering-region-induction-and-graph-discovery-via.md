# #171 — Parallel clustering, region induction, and graph discovery via immutable fragments

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:02Z
- **Updated:** 2026-07-25T05:13:47Z
- **Closed:** 2026-07-25T05:13:47Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/171
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG), #167 (reproducibility), #168 (jobs), #169 (memory), #170 (observation shards).
Extends: #60 (multiresolution cover induction), #71 (hidden-state trace lane), #70 (cover fineness sweep), #125 (edge algebras), #131 (planning, for plan-macro discovery).
Source: plan §4.1; direction PDF §§4, 7, 11.

## Motivation

Cover induction and graph discovery (`crates/uor-r4-graph-compiler/src/{induction.rs,quantum_cover.rs,graph.rs}`) is the compiler's algorithmic core and its highest nondeterminism risk under parallelism: clustering assignment, region splits, overlap and edge discovery all *feel* parallel but produce graph topology — exactly where completion-order or atomic-ID designs would break byte reproducibility. This issue owns the discovery pattern for that layer; routing-program synthesis (#172) and emission/residual compilation (#173) are separate owners.

## Scope

Deterministically parallelize:

- assignment steps in clustering;
- independent child-region compilation;
- region split evaluation;
- overlap discovery;
- parent/child relationship discovery;
- semantic-neighbor discovery;
- predictive-forward-edge discovery;
- evidential-predecessor-index construction;
- causal-transition candidate discovery;
- plan-macro recurrence discovery;
- graph-fragment generation.

Required design pattern (normative):

```text
parallel local discovery
    ↓
worker-local immutable fragments
    ↓
stable canonical sort
    ↓
deterministic deduplication
    ↓
sequential canonical ID assignment
```

Explicitly forbidden:

- shared mutable graph insertion;
- shared atomic node-ID allocation;
- lock-contended global vectors;
- completion-order-dependent graph structure;
- nondeterministic `HashMap` iteration (use `BTreeMap`/sorted keys — the existing `induction.rs` style — feeding canonical output);
- first-finished-candidate-wins decisions.

Canonical IDs, offsets, and topology-affecting decisions happen only in the sequential finalization spine per #166; discovery produces immutable fragments keyed by content or stable positional keys.

## Non-goals

- Changing clustering objectives or cover semantics (#70/#127/#60 lineage owns those); this issue parallelizes the existing algorithms.
- Routing-program/XOR-polynomial search (#172); emission/residual/certificate compilation (#173).
- Distributed clustering or GPU clustering (non-goal).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler/src/{induction.rs,quantum_cover.rs,graph.rs,patch_induction.rs}`
- plan §4.1; R4G1.md §7

## Dependencies

- #165, #166, #167, #168, #169, #170.
- Consumed by: #173 (emission compilation consumes discovered regions), #175 (regions/sec metrics).

## Deliverables

1. Fragment-based parallel discovery for the stages above on the #165 executor.
2. Canonical merge implementation (stable sort → dedup → sequential ID assignment).
3. Byte-equality evidence: sequential, parallel, and repeated parallel runs at different thread counts produce byte-identical artifacts (via #167's harness).

## Acceptance criteria

- Byte-identical artifact output across: sequential execution; parallel execution; repeated parallel execution with different thread counts (1/2/4/N).
- No forbidden pattern appears in discovery code (review + targeted tests; e.g. a test that hammers scheduling perturbation without output change).
- Canonical ID assignment is provably outside worker code (IDs assigned only on the finalization spine).
- Tie-breaking at every selection point is documented and deterministic.
- Four gates green; #167 harness passes.

## Formal claim classification

- Thread-count-invariant graph topology and IDs: **Guarantee**, status **Structural** (harness-backed).
- Clustering quality and speedup: **Empirical Criterion** (quality per the Gate C lineage; speed per #175).

## Test requirements

- **Unit:** fragment merge stability, dedup correctness, canonical-key ordering.
- **Property:** merge output invariant under fragment interleaving/permutation.
- **Reproducibility:** #167 harness over clustering-heavy fixtures.
- **BDD/Cucumber:** scenario pinning identical cover output sequential vs. parallel on a small corpus, per `features/suites/` conventions.
- **Benchmark:** regions/sec, clustering wall-clock (feeds #175).

## Memory considerations

Fragment buffers are worker-local and bounded per #169; large covers stream fragments through the spill/resume path (#59 precedent) rather than accumulating in memory.

## Determinism considerations

The core of this issue: discovery order is free; canonical order is not. Every semantic decision (splits, memberships, edges) derives from sorted keys and documented tie-breaks, never from scheduling.

## CPU portability and no-GPU statement

CPU-native discovery. No CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, GPU-resident intermediates, or accelerator-specific graph formats.

## Traceability

- Plan §4.1; direction PDF §§4, 7, 11; R4G1.md §7; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#60, #70, #71, #122, #125, #127, #129, #131, #157, #165, #166, #167, #168, #169, #170, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

