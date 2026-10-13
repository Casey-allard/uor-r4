# Project Context: 4D Polar / Hyperbolic Routing

## The mechanism
We route points using:

1) Map points from Poincaré ball to tangent at origin:
   - `v = log_0(x)`
2) Apply chart (optional):
   - `z = chart(v)`
     - rotation `R ∈ SO(d)` (optional)
     - scaling:
       - global diagonal `diag(exp(s))`, or
       - radial-bin diagonal `diag(exp(S[bin(r)]))`
3) Route:
   - `shell = floor(||z|| / delta_r)`
   - `sector = argmax( normalize(z) · C_k )` (k-means on sphere), OR a phase/polar sectoring mode

Then local memory and growth operate within each `(shell, sector)` bucket.

## Global Alignment Constraint
- Poincare-ball structure is a foundational part of the routing architecture, not just a preprocessing map.
- A previous session identified Poincare balls as important for globally aligned routing. That should remain active context for all future geometry branches.
- The current mathematical drift risk is flattening into tangent/chart space and then overfitting shell/sector laws that no longer preserve enough of the original hyperbolic global alignment.
- Future routing laws should either preserve that global alignment from the original `B^4` / `H^4` picture or explain precisely what replaces it.

## Canonical Mathematical Contract (2026-03-10)
- The active coupled-field object is asymmetric `H^4 x H^4`, not flat `R^8`.
- First factor:
  - `R(x) = (r, chi, theta1, theta2)` on `H^4 ~= r x S^3`
  - role: base routing geometry
- Second factor:
  - `F(x) ∈ H^4`
  - role: discrete complex-value field
- Coupling:
  - `K(x) = K_base(R(x)) ⊕ K_field(F(x))`
  - plus a geometry-induced phase update law rather than a free phase heuristic
- Notation guardrail:
  - do not describe this program as vague `U^4`
  - use hyperbolic polar geometry plus `U(1)`, `U(1)^2`, or `SU(2)` only when the branch actually uses fiber/group transport

## "What we learned so far" (from pasted runs)
- Radial scaling + growth budget helps.
- `sector_mode=kmeans` is generally stronger than `sector_mode=phase2` on post-growth test MSE (seed variance exists).
- `time_pressure_lambda` in [0.25, 1.2] tends to hurt pre-growth MSE and doesn't reliably improve post-growth.

## Why we’re moving to Codex
Runs are too slow for manual exploration. We need:
- machine-readable run summaries
- caching + fast dev mode
- automated staged sweeps
- a decisions log

## Implementation status (2026-03-05)
- CLI now includes runtime controls (`fast_dev`, early-stop, cache knobs, run tags).
- Route modes include `kmeans`, `phase2`, `phase4d`, and `complex2`.
- Runs emit `__JSON_SUMMARY__` for parser/summarizer automation.
- Staged sweep and gate-note automation are available via `runs/run_pipeline.sh`.

## Imported Strategic Context (2026-03-10)
The project base directory contains a larger markdown theory corpus than the initial four-file import.

Primary imported notes:
- `EVIDENCE_SUMMARY.md`
- `GEOMETRIC_COMPUTATION_HYPOTHESIS.md`
- `NEXT_CRITICAL_EXPERIMENTS.md`
- `THEORY_SKETCH.md`

Additional imported theory corpus:
- `hyperbolic_router_math_review.md`
- `geometric_routing_kill_tests.md`
- `phase_transport_hypothesis.md`
- `event_driven_geometric_routing_model.md`
- `geometric_theory_of_mind_routing.md`
- `geometric_intelligence_hypothesis.md`
- `minimal_theorem_for_spectral_emergence.md`
- `spectral_emergence_moonshot.md`
- `experimental_protocol_geometric_ai_routing.md`
- `adaptive_field_computer_moonshot.md`

Canonical import record:
- `docs/research/IMPORTED_GUIDANCE_20260310.md`

These are not replacing the canonical docs, but they sharpen the project framing in a useful way.

### Imported hypothesis framing
- The project is not just “find a better router.”
- The stronger framing is:
  - geometry
  - spectral structure
  - phase transport
  - sparse event-driven computation
- In other words, the long-range goal is to determine whether computation itself can emerge from structured motion and interaction on the routing manifold rather than from dense recomputation.

### Imported manifold guidance
- Candidate geometric substrates explicitly include:
  - hyperbolic spaces `H^n`
  - fibered phase structures
  - product manifolds with coupled fields
- This is consistent with the current live product branch:
  - `H^4 x H^4` in hyperbolic polar structure on both factors
- It also supports keeping phase transport and coupled auxiliary fields as first-class research directions instead of treating them as isolated heuristics.
- The broader theory corpus sharpens the current intended object:
  - `H^4 x H^4` in hyperbolic polar structure on both factors remains mathematically coherent
  - one factor can act as base routing geometry
  - the other can act as transport, retrieval, correction, or key field

### Imported operator sketch
- A useful generic routing sketch from the new notes is:
  - `K(x, y) = exp(-d_M(x, y))`
- This should not replace the current implemented route law directly, but it is a good canonical abstraction for:
  - compatibility
  - sparse interaction strength
  - event-triggered routing
- It also reinforces the current product direction:
  - treat `H^4 x H^4` as a live candidate manifold rather than a speculative side branch
  - keep the second factor available for transport, retrieval, or correction-field roles

### Imported spectral and phase framing
- The additional theory files make two larger claims explicit:
  - spectral modes may emerge from geometry/operator structure rather than being separately engineered
  - phase transport may be a cheap computational primitive rather than only a routing decoration
- These claims remain unproven in this repo, but they are now part of the standing research context and should shape branch selection.

### Imported falsification ladder
The active mechanism-first doctrine is now:
1. embedding stability
2. measure-consistent shell routing
3. Hopf angular correctness
4. phase transport necessity
5. spectral emergence measurement
6. sparse event-driven trainability
7. hardware-efficiency confirmation

### Imported evidence stance
- The new evidence summary is directionally consistent with the repo:
  - structured routing looks promising
  - curved embeddings preserve some hierarchy
  - sparsity can emerge
- But the evidence remains insufficient.
- That matches the repo’s existing discipline:
  - do not claim dense-model replacement from proxy evidence alone
  - do not mistake geometric promise for a production win before runtime and stability survive hard tests
- It also sharpens a missing requirement:
  - future branch docs should say explicitly whether they are testing routing quality, spectral structure, phase transport, sparse event behavior, or hardware cost
- The moonshot hardware notes are now preserved as secondary context:
  - useful for long-range direction
  - not sufficient reason to skip software-side falsification
