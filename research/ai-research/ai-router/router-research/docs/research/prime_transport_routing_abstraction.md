# Prime Transport Routing Abstraction

## Purpose

This note translates the current exact-layer results into a concrete
routing-state design hypothesis, without changing runtime code and without
leaving the exact recursive-system layer.

The three candidate routing states are:

- `R_static = (b, phi, r)`
- `R_min = (b, phi, r, next_return_gap)`
- `R_full = (b, spin_H)`

## Candidate A: Static Chart Only

State:

- `R_static = (b, phi, r)`

What it preserves:

- exact orbit location in the phase-fiber-scale chart
- static density context inherited from wheel depth and local stencil position
- the geometric placement of first-splitting sectors

What it loses:

- the dynamical branch distinctions that finite-horizon spin separates
- most of the predictive refinement that decides visible splitting

How it updates:

- under `j -> j + 1`, update base phase `b`
- update fiber phase `phi`
- wheel depth `r` changes only at lift

How it triggers refinement:

- it cannot trigger refinement reliably by itself
- it can only supply coarse geometric sectors in which unresolved predictive
  classes may later split

## Candidate B: Minimal Hybrid Predictive State

State:

- `R_min = (b, phi, r, next_return_gap)`

What it preserves:

- all static chart distinctions
- immediate forward return-memory
- a substantial fraction of the practical predictive distinctions used in the
  current first-splitting analysis

What it loses:

- richer return-grammar distinctions that full spin still resolves
- deeper branch identities when multiple predictive branches share the same
  immediate return gap

How it updates:

- update `b`
- update `phi`
- recompute or transport the next admissible-return distance

How it triggers refinement:

- unresolved classes with the same static chart state but different immediate
  return-memory are promoted earlier than under the static chart alone
- refinement is still delayed until collective first-splitting pressure appears

This is the smallest currently validated exact routing state that is genuinely
smaller than spin while still carrying useful predictive structure.

## Candidate C: Full Predictive State

State:

- `R_full = (b, spin_H)`

What it preserves:

- the full current predictive partition at horizon `H`
- the strongest currently available exact predictor of first visible splitting

What it loses:

- compactness
- simplicity of update and interpretation relative to smaller residual states

How it updates:

- shift the future word under `j -> j + 1`
- append the next admissibility bit

How it triggers refinement:

- refinement is explicit whenever predictive classes branch
- this is the exact but higher-complexity control state

## Minimal Routing Principle

The current exact-layer results support the following routing principle:

1. start with coarse exact state first
   - use static chart context as the initial routing address
2. delay refinement
   - because internal split is immediate but visible split is delayed
3. promote only when unresolved predictive classes begin collective
   first-splitting
   - this is the exact-layer mechanism behind visible threshold

So the routing picture is:

- coarse chart state provides the initial partition
- a small residual dynamical tag provides early predictive refinement
- full predictive state is only needed when the smaller tag fails to separate
  the relevant branch structure

## Comparison

The bounded comparison currently reads:

- `R_static = (b, phi, r)`
  - exact and interpretable
  - but too detailed to count as a compressed routing state and not sufficient
    as a predictive abstraction on its own

- `R_min = (b, phi, r, next_return_gap)`
  - membership purity `1.0`
  - split purity `0.69033629362971`
  - combined capture fraction of spin `0.7502981320146836`
  - label ratio to spin `0.6121906310452204`

- `R_full = (b, spin_H)`
  - membership purity `1.0`
  - split purity `1.0`
  - reference predictive partition

## Best Current Routing Abstraction

The best current routing abstraction is:

- `R_min = (b, phi, r, next_return_gap)`

This is mathematically justified by the exact-layer results because:

- the static chart is real and exact but not fully predictive
- visible threshold is governed by `density + first-splitting event`
- the smallest validated residual that materially improves predictive
  discrimination while remaining smaller than spin is `next_return_gap`

## Smallest Plausible Prototype

The smallest plausible routing prototype is:

- coarse routing on `(b, phi, r)`
- add `next_return_gap` as the minimal dynamic refinement tag
- promote to richer predictive state only when unresolved classes remain
  collectively first-splitting

This stays faithful to the exact-layer mechanism without pretending the full
spin state is already unnecessary.

## What Remains Uncertain

The current uncertainty is:

- whether a slightly richer but still compressed return-grammar object can
  improve materially on `next_return_gap` without losing the compression
  advantage
- whether full spin is ultimately necessary for robust predictive routing in
  the deeper-lift regime

## Recommendation

The first routing prototype target should be:

- `R_min = (b, phi, r, next_return_gap)`

Reason:

- it is the smallest currently justified predictive refinement of the exact
  chart
- it respects delayed refinement
- it is materially weaker than spin, but strong enough to serve as the first
  conservative prototype target
