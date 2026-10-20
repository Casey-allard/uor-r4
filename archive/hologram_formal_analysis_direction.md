# Hologram / R4 Formal Analysis Direction

> **Historical formal direction.** This document preserves the original
> teacher-to-semantic-graph programme and its proof questions. It no longer
> controls implementation order. Current authority is the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

Source document: [`hologram_formal_analysis_direction.pdf`](hologram_formal_analysis_direction.pdf)

This document is the repository index for the July 2026 formal-analysis direction. The
GitHub implementation tracker is [#122](https://github.com/UOR-Foundation/uor-r4/issues/122).
The direction is to compile trained neural behavior into an immutable, bounded,
transformerless semantic graph. Runtime execution remains deterministic graph
propagation over compressed semantic state using the existing integer-only,
allocation-free contract.

## Claim classes

Every statement in the formal design must be labeled as one of these:

- **Definition:** an architectural object or mathematical term.
- **Objective:** a quantity optimized by the offline compiler.
- **Guarantee:** a structural property of the artifact or runtime.
- **Empirical criterion:** a measured property with a declared distribution,
  protocol, sample count, and uncertainty.
- **Assumption:** a condition required by a proof or certificate but not established
  by the implementation itself.

The project does not claim exact teacher equivalence, human-level reasoning, or that
plausible language output proves coherent internal state transitions.

## Architectural direction

1. Model a semantic state space `S` and typed transitions `T: S x A -> S`.
2. Represent one shared node space with multiple edge algebras: semantic, causal,
   temporal, constraint, goal-progress, evidence/provenance, refinement, forward,
   and reverse edges.
3. Define a holographic encoding `H(x)` as overlapping projections with partial
   recoverability, distributed evidence, and progressive fidelity.
4. Treat compilation as semantic compression from teacher behavior to a graph
   artifact, with predictive distortion and storage/runtime cost measured together.
5. Optimize predictive utility, future-state information, artifact size, and runtime
   cost using declared compiler objectives, including information-bottleneck proxies.
6. Separate semantic state transitions and bounded planning from language emission;
   language is one output adapter.
7. Prove structural properties such as determinism, bounded work, valid references,
   replay, provenance, termination, and fallback safety. Measure teacher fidelity,
   semantic validity, and candidate recall empirically.

## Implementation sequence

The formal-direction issues are intentionally layered over the lower-level R4G1 and
runtime backlog in issues #11-#34.

1. [#123](https://github.com/UOR-Foundation/uor-r4/issues/123) Formal vocabulary,
   notation, and claim classes.
2. [#124](https://github.com/UOR-Foundation/uor-r4/issues/124) Semantic state model
   and typed graph dynamics.
3. [#125](https://github.com/UOR-Foundation/uor-r4/issues/125) Shared node space
   and multiple edge algebras.
4. [#126](https://github.com/UOR-Foundation/uor-r4/issues/126) Holographic encoding,
   partial reconstruction, and progressive fidelity.
5. [#127](https://github.com/UOR-Foundation/uor-r4/issues/127) Predictive entropy
   and information-bottleneck objectives.
6. [#128](https://github.com/UOR-Foundation/uor-r4/issues/128) Intervention and
   counterfactual behavioral probes.
7. [#129](https://github.com/UOR-Foundation/uor-r4/issues/129) Reference
   floating-point compiler and intermediate representation.
8. [#130](https://github.com/UOR-Foundation/uor-r4/issues/130) Boolean, mask,
   popcount, and fixed-point lowering.
9. [#131](https://github.com/UOR-Foundation/uor-r4/issues/131) Bounded future-state
   optimization and planning.
10. [#132](https://github.com/UOR-Foundation/uor-r4/issues/132) Structural graph
    and planner proofs.
11. [#133](https://github.com/UOR-Foundation/uor-r4/issues/133) Formal monograph
    and implementation specification.
12. [#134](https://github.com/UOR-Foundation/uor-r4/issues/134) Semantic reasoning
    versus language emission.
13. [#135](https://github.com/UOR-Foundation/uor-r4/issues/135) Graph invariant
    ownership and loader validation.
14. [#136](https://github.com/UOR-Foundation/uor-r4/issues/136) Semantic compression
    and rate-distortion reporting.
15. [#137](https://github.com/UOR-Foundation/uor-r4/issues/137) PDF-to-implementation
    traceability matrix.

## Evidence and compatibility

- The reference compiler may use floating point and allocation; the deployed runtime
  may not inherit those dependencies.
- Reference-graph results must be differentially testable against lowered R4G1
  artifacts and the scalar runtime.
- All empirical reports must carry source, corpus, graph, metric, operation,
  benchmark, and provenance identifiers where applicable.
- Structural guarantees require tests, fuzzing, proof artifacts, or explicit
  `Unproven` status in the proof matrix.
- Existing TLA3/TLA4/TLS1 artifacts and transformerless behavior remain readable
  during migration. No graph path replaces the default path before its quality and
  runtime gates pass.

## Section coverage

| PDF section | Repository issue(s) |
| --- | --- |
| 1 Definitions, objectives, guarantees | #123 |
| 2 Semantic state manifold | #124 |
| 3 Graph dynamics | #124, #131 |
| 4 Multiple edge algebras | #125 |
| 5 Mathematical hologram | #126 |
| 6 Predictive entropy | #127 |
| 7 Semantic compression | #129, #136 |
| 8 Information bottleneck | #127 |
| 9 Graph invariants | #135 |
| 10 Thinking versus language | #134 |
| 11 Compiler as core contribution | #128-#130 |
| 12 Future-state optimization | #131 |
| 13 Structural proofs | #132 |
| 14-15 Overall direction and project description | #122, #133 |
| 16 Monograph structure | #133 |
| 17 Immediate research sequence | #122, #137 |
