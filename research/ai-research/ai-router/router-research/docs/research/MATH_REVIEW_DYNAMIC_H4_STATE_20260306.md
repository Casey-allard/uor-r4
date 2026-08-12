# Math Review: Dynamic Hyperbolic State (2026-03-06)

## Purpose
Formalize the next geometry branch after the translated retrieval branch failed operational confirm.

The live question is no longer whether static Hopf geometry has signal.
That is already established.
The live question is whether the missing architectural gain requires time to live inside the geometry rather than outside it as a scalar schedule.

## Working Diagnosis
The current routed frontier still treats time mostly as:
- shell growth pressure
- convergence/divergence control
- training schedule

That is mathematically weaker than a true dynamic geometric state.

The strongest next objects are:
1. `H^4 + T_xH^4`
2. `H^4 x H^4`

## Why This Branch Exists
The static branch already established:
- Poincare-ball global alignment matters
- Hopf-aware `H^4` angular treatment is real
- local widening laws can help, but they do not fully cash in the geometry
- current translated retrieval cost no longer fails mainly on routed local search

So the next missing piece may be dynamic state, not another static shell law.

## Formulation A: `H^4 + T_xH^4`
Interpret each routed point as:
- `x_t in H^4`
- `u_t in T_{x_t}H^4`

Where:
- `x_t` is the geometric routing location
- `u_t` is the local transport / momentum / flow state

Conceptually:
- `x_t = (r_t, chi_t, theta1_t, theta2_t)`
- `u_t = (p_r, p_chi, p_theta1, p_theta2)`

Natural distance sketch:
- `d((x,u),(x',u'))^2 = d_H(x,x')^2 + lambda_u * ||PT_{x->x'}(u) - u'||^2`

where `PT` is parallel transport.

### Meaning
This is the conservative dynamic branch.
It says the route has one hyperbolic geometry and one local flow field attached to it.

## Formulation B: `H^4 x H^4`
Interpret each routed point as:
- `x_t in H^4`
- `y_t in H^4`

Where:
- `x_t` is the routing location / geometric address
- `y_t` is a second hyperbolic field carrying dynamic transport state

Possible meanings for `y_t`:
- flow field
- memory-pressure field
- convergent/divergent transport field
- imaginary / retrieval transport field

Natural product metric sketch:
- `d((x,y),(x',y'))^2 = d_H(x,x')^2 + lambda_y * d_H(y,y')^2`

### Meaning
This is the stronger hypothesis.
It says dynamic state is not just a tangent correction; it needs its own hyperbolic geometry.

## Important Coordinate Clarification
This branch means hyperbolic polar structure on both factors, not flat Euclidean polar coordinates in 8D.

So the intended full object is:
- first factor: `H^4` routing position in geodesic-polar / Hopf-aware coordinates
- second factor: `H^4` dynamic field in geodesic-polar form as well

The current Slice A code does **not** yet implement full manifold-native polar coordinates on the second factor.
It only tests whether a second hyperbolic factor is worth building by using an origin-ball surrogate from flow.

## What The Current Code Can Support Right Now
The repo does not yet implement:
- general-point logarithmic maps
- general parallel transport on `H^4`

So Slice A should use a controlled surrogate, not pretend those operators already exist.

## Slice A Surrogate Law
Start from the current routed chart coordinate `z_t` for each sequential proxy context.

Define static position:
- `x_t = exp_0(z_t)`

Define past-flow surrogate:
- `delta_t = z_t - z_{t-1}`

Normalize by the train-window flow scale:
- `u_t = (flow_scale / q95(||delta||)) * delta_t`

Then compare:
1. Static `H^4`
   - state = `x_t`
2. Tangent surrogate `H^4 + T_xH^4`
   - state = `(x_t, u_t)`
   - metric = `d_H(x,x')^2 + lambda_u ||u-u'||^2`
3. Product surrogate `H^4 x H^4`
   - `y_t = exp_0(u_t)`
   - metric = `d_H(x,x')^2 + lambda_y d_H(y,y')^2`

This is not the final geometry.
It is a disciplined pilot that can tell us whether dynamic state carries measurable predictive or neighborhood value.

## Why The Surrogate Is Acceptable
- it preserves current Poincare-ball global alignment on the position branch
- it does not fake a full arbitrary-point transport implementation
- it lets the repo answer the first actual branch question:
  - does dynamic state improve sequence coherence and target-neighborhood quality at all?

## Diagnostic Criteria For Slice A
A dynamic branch is interesting only if it improves at least one of:
- target prediction MSE from geometric nearest neighbors
- target top-1 from geometric nearest neighbors
- sequence coherence ratio (consecutive-state distance vs random-pair distance)

without introducing obvious geometric nonsense.

## Decision Rule
- If `H^4 + T_xH^4` wins cleanly, move next to proper tangent transport.
- If `H^4 x H^4` wins cleanly, treat the second hyperbolic factor as a real architecture branch.
- If neither wins, dynamic geometry stays open mathematically but loses priority to a cheaper systems target.

## Current View
The stronger user hypothesis remains plausible:
- time likely needs to evolve with the geometry
- `H^4 x H^4` is a distinct candidate, not just a restatement of 8 variables

But Slice A must prove signal before the repo earns a full route-law rewrite.
