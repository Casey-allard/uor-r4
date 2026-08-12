# INC-0059: Coupled `H^4 x H^4` Polar Flow Field

## Status
Active umbrella contract.

## Trigger
The translated retrieval branches have established two things:
- discrete complex / imaginary route keys are real and useful
- local repair by candidate expansion or simple in-bucket reranking is not enough to cash in the full advantage

## Hypothesis
The second `H^4` should stop being treated as only a key field or a local correction field.
Instead, the routing position factor and the retrieval / dynamic factor should be coupled directly as a shared polar-flow geometry on `H^4 x H^4`.
That should let time evolution live in the geometry itself rather than only in post-hoc scoring or shell heuristics.

User clarification now sharpens this:
- one `H^4` factor is the main routing geometry
- the second `H^4` factor is the discrete complex / imaginary value field
- the second factor should scale with the first
- both factors should be allowed to scale through phase shifts, giving two coupled scaling levers
- the deeper thesis is that geometry should force the phase behavior rather than phase being bolted on as an external trick
- the long-range claim is not only sparse routing, but that geometry can induce spectral structure that then supports phase transport

## Minimal Scope
1. Lock the product-space contract:
   - first factor = routing geometry
   - second factor = discrete complex-value field
2. Keep both factors in hyperbolic polar structure.
3. Keep phase as geometry-induced transport, not a free score knob.
4. Execute the mechanism-first kill ladder before deeper coupled-field claims.
5. Delegate the first live implementation slice to `INC-0060`.

## Acceptance
- establishes the canonical coupled-field contract used by downstream increments
- keeps `H^4 x H^4` asymmetric and geometrically interpretable
- does not collapse into flat Euclidean auxiliary coordinates

## Mathematical Rationale
The project goal is not just cheaper lookup; it is routing geometry that can scale better than brute-force dense hardware allocation.
The current evidence suggests:
- static hyperbolic geometry helps globally
- discrete complex addressing helps locally
- post-hoc local repair is too weak
The next coherent move is therefore to let the product space itself carry more of the routing and retrieval dynamics.

## Notation Guardrail
- Do not collapse this branch into vague `U^4` language.
- The likely group objects are more structured than that:
  - `U(1)` or `U(1)^2` for phase fibers
  - `SU(2)` / unit-quaternion structure for Hopf angular transport
  - hyperbolic polar geometry for the base manifold(s)
- The current branch should therefore be described as:
  - `H^4 x H^4` with Hopf/polar structure
  - first factor = routing geometry
  - second factor = discrete complex-value field
  - both factors coupled through phase-shift scaling
  - not a generic flat unitary latent

## Execution Handoff
- First implementation slice:
  - `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
- Canonical plan:
  - `docs/research/MECHANISM_FIRST_PLAN.md`
