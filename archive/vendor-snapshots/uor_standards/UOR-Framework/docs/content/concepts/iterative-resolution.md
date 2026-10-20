# Iterative Resolution

## Definition

**Iterative resolution** extends the resolution process into a learning loop.
Rather than computing a partition in a single pass, the resolver proceeds
iteratively: each iteration applies a constraint, pins sites, and checks
whether the budget is closed. The process converges when all sites are pinned.

## Resolution State

A {@class https://uor.foundation/resolver/ResolutionState} tracks the progress
of an iterative resolution:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/resolver/isComplete} | xsd:boolean | True when all sites are pinned |
| {@prop https://uor.foundation/resolver/iterationCount} | xsd:nonNegativeInteger | Iterations performed so far |
| {@prop https://uor.foundation/resolver/siteDeficit} | FreeRank | Remaining unpinned sites |
| {@prop https://uor.foundation/resolver/convergenceRate} | xsd:decimal | Sites pinned per iteration |

A {@class https://uor.foundation/resolver/Resolver} links to its state via
{@prop https://uor.foundation/resolver/resolutionState}.

## Refinement Suggestions

When the resolution is incomplete, the resolver produces
{@class https://uor.foundation/resolver/RefinementSuggestion} objects that
guide the next iteration:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/resolver/suggestedAxis} | MetricAxis | Which metric axis to explore |
| {@prop https://uor.foundation/resolver/suggestedClass} | owl:Class | Which constraint class to apply |
| {@prop https://uor.foundation/resolver/targetSites} | SiteIndex | Which sites to target |

Suggestions are linked from the state via
{@prop https://uor.foundation/resolver/suggestion}.

## Refinement Steps

Each iteration produces a {@class https://uor.foundation/derivation/RefinementStep}
that records the type-level change:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/derivation/previousType} | TypeDefinition | Type before this step |
| {@prop https://uor.foundation/derivation/appliedConstraint} | Constraint | Constraint applied |
| {@prop https://uor.foundation/derivation/refinedType} | TypeDefinition | Type after this step |
| {@prop https://uor.foundation/derivation/sitesClosed} | xsd:nonNegativeInteger | Sites pinned in this step |

{@class https://uor.foundation/derivation/RefinementStep} is a subclass of
{@class https://uor.foundation/derivation/DerivationStep}, parallel to
{@class https://uor.foundation/derivation/RewriteStep} for term-level rewriting.

## Convergence

The resolution loop terminates when:
1. {@prop https://uor.foundation/resolver/isComplete} is `true`
2. {@prop https://uor.foundation/partition/isClosed} is `true` on the free rank budget
3. {@prop https://uor.foundation/resolver/siteDeficit} shows zero free sites

The {@prop https://uor.foundation/resolver/convergenceRate} tracks the rate of
progress, enabling early detection of stalled resolutions.

## Example: Three-Iteration Resolution

```
Iteration 0: 0/8 sites pinned  (convergenceRate = 0.0)
Iteration 1: apply ResidueConstraint::new(8, 1) → 3/8 pinned (rate = 3.0)
Iteration 2: apply CarryConstraint::new(2)      → 6/8 pinned (rate = 3.0)
Iteration 3: apply DepthConstraint::new(0, 4)   → 8/8 pinned (rate = 2.67, complete)
```

Each constraint is a `BoundConstraint<O, B>` instance constructed via the
v0.2.1-compatible type-alias ctors. See
[Constraint Algebra](constraint-algebra.html) for the parametric form.

When resolution stalls (convergence rate drops to zero), the
{@class https://uor.foundation/resolver/CechNerve} provides topological
diagnostics via the [ψ-Pipeline](../guides/psi-pipeline.html) — Betti numbers
reveal whether the stall is due to cyclic constraint dependencies or
insufficient constraint coverage.
