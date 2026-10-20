# #126 — Specify holographic encoding, partial reconstruction, and progressive fidelity metrics

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:42Z
- **Updated:** 2026-07-24T17:07:17Z
- **Closed:** 2026-07-24T17:07:17Z
- **Labels:** graph-compiler
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/126
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §5.

Turn "hologram" from a metaphor into a measurable representation contract. For an observation `x`, the compiler emits overlapping projections `H(x)={h0,...,hk}`; additional projections or refinement should improve behavioral recovery without requiring any one node or bit to encode the whole concept.

## Scope

- Define the projection family, membership identity, ablation semantics, and reconstruction/behavioral recovery operator.
- Use a declared divergence or agreement metric between teacher `P_theta(.|x)` and graph `P_G(.|H(x))`.
- Measure partial recoverability of each projection, distributed evidence, and progressive fidelity as depth/membership increases.
- Add probes for paraphrase invariance, projection ablation, perturbation stability, and cross-context reuse.
- Keep semantic validity and teacher fidelity as empirical claims with pinned distributions and confidence intervals.

## Acceptance criteria

- A certificate schema records projection set, ablation protocol, divergence, sample count, confidence interval, and corpus CIDs.
- A deterministic fixture demonstrates partial recovery and progressive-fidelity calculations.
- Degenerate encodings (single-node memorization, empty projections, and duplicate projections) are rejected or explicitly reported.
- The report distinguishes structural encoding facts from measured teacher agreement and does not claim exact reconstruction.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

