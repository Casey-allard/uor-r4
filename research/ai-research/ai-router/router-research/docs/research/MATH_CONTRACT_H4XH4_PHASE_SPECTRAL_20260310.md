# Math Contract: Coupled `H^4 x H^4` Phase / Spectral Routing

## Purpose
This file consolidates the exact local theory equations that define the intended coupled-geometry program.
It exists so future route-law work does not drift back into heuristic shell/sector tuning.

## Source Files
- `/Users/adminamn/ai-router/router-research/hyperbolic_router_math_review.md`
- `/Users/adminamn/ai-router/router-research/phase_transport_hypothesis.md`
- `/Users/adminamn/ai-router/router-research/event_driven_geometric_routing_model.md`
- `/Users/adminamn/ai-router/router-research/minimal_theorem_for_spectral_emergence.md`
- `/Users/adminamn/ai-router/router-research/geometric_routing_architecture_summary.md`
- `/Users/adminamn/ai-router/router-research/experimental_protocol_geometric_ai_routing.md`

## Canonical Object
The intended architecture is an asymmetric coupled product:

`(R, F) ∈ H^4 x H^4`

- `R`
  - base routing manifold
- `F`
  - coupled field
  - discrete complex-value field
  - candidate role in phase transport and addressing refinement

The contract from the theory corpus is:

`K(x) = K_base(x) ⊕ K_field(x)`

with routing on the base manifold and field modulation from the second factor.

## Base Routing Coordinates
The routing factor uses hyperbolic polar / Hopf coordinates.

Canonical routing sketch from the local theory corpus:

`Route(x) = Π(shell, sector)( r(x), χ(x), θ1(x), θ2(x) )`

Equivalent angular parameterization in complex coordinates:

`(z1, z2) ∈ C^2`

`z1 = e^(iθ1) cos(χ)`
`z2 = e^(iθ2) sin(χ)`

Alternative notation used in the event-driven note:

`x = (r, η, δ, α)`

with

`z1 = e^(i(α + δ/2)) cos(η)`
`z2 = e^(i(α - δ/2)) sin(η)`

Mapping between the notations:
- `η` corresponds to Hopf latitude / amplitude split
- `δ` is relative phase / base angular direction
- `α` is common fiber phase

## Hopf Routing Split
The central routing split from the local notes is:
- `(r, η, δ)` determines coarse routing location
- `α` determines cheap internal motion within a routed region

The Hopf map written in the local theory is:

`h(z1, z2) = (2 Re(z1 conj(z2)), 2 Im(z1 conj(z2)), |z1|^2 - |z2|^2)`

which becomes

`h(z1, z2) = (sin(2η) cos(δ), sin(2η) sin(δ), cos(2η))`

Important consequence:
- the common phase `α` disappears from the Hopf base
- the base point is controlled by `(η, δ)`
- `α` is a fiber variable and therefore the natural candidate for cheap internal transport

## Angular Metric
The local theory gives the `S^3` metric in Hopf-like coordinates.

Starting form:

`ds^2 = dη^2 + cos^2(η) dφ1^2 + sin^2(η) dφ2^2`

with

`φ1 = α + δ/2`
`φ2 = α - δ/2`

which yields

`ds^2 = dη^2 + dα^2 + (1/4) dδ^2 + cos(2η) dα dδ`

Interpretation:
- `dα` is the fiber transport direction
- `dδ` is the relative-phase / base angular direction
- `dη` is the latitude / amplitude direction

Special operating point from the local notes:
- at `η = π/4`, `cos(2η) = 0`
- the angular metric locally decouples
- balanced-amplitude states may therefore support cheap phase transport more cleanly

## Coupled-Field Phase Law
The strongest explicit phase law in the local theory corpus is:

`θ -> θ + F(x)`

and the connection-style transport form is:

`α_tilde_j = α_j + ∫_γ A`

Working interpretation for this repo:
- the second hyperbolic factor `F` is allowed to modulate transport on the routing factor `R`
- but phase must be induced through geometry / connection structure, not inserted as an unconstrained score term
- corrected experimental constraint from `INC-0063` / `INC-0064`:
  - a base-only Hopf transport law without direct coupled-field participation is
    mechanism-live once `alpha` bins are active
  - coupled-field transport is also mechanism-live and produces explicit field
    shift on the proxy schedule
  - near-term phase work should therefore move to the explicit product
    phase-field branch rather than re-litigating whether phase motion exists at
    all

## Compatibility Kernel
The imported theory corpus provides a generic compatibility sketch:

`K(x, y) = exp(-d_M(x, y))`

This is not the final implementation law by itself.
It is the canonical abstraction for:
- compatibility
- sparse interaction strength
- event-triggered routing
- operator construction for later spectral tests

## Spectral Claim
The spectral files provide the minimal operator story:

`L = D - A`

`L v_k = λ_k v_k`

`u(t) = Σ c_k e^(i sqrt(λ_k) t) v_k`

Interpretation:
- oscillatory modes can emerge from static compatibility geometry
- explicit oscillation rules are not required if the operator geometry is structured enough

This is the repo's canonical reason that phase and spectral claims remain core, not decorative.

## Program Consequence
The near-term order is therefore:
1. prove geometry routing works in the intended infinite hyperbolic/Hopf space
2. correct shell and angular laws until they are measure-consistent enough to trust
3. then test whether phase transport and spectral structure emerge from that geometry
4. only after that reopen event-driven / gated-intelligence computation

## Scope Guardrail
- Do not collapse this contract into flat `R^8`.
- Do not use vague `U^4` language.
- The useful structures in the local theory are:
  - hyperbolic geometry on `H^4`
  - Hopf / `S^3` angular coordinates
  - `U(1)` fiber phase
  - possibly `SU(2)`-like transport interpretation where appropriate
