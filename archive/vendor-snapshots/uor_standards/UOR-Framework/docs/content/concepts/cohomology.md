# Cohomology

## Definition

**Cohomology** is the algebraic dual of homology. Where chain groups map
downward via boundary operators, **cochain groups** map upward via coboundary
operators. The {@class https://uor.foundation/cohomology/CochainGroup} C^k is
the dual of the chain group C_k, consisting of linear functionals on k-chains.

The coboundary operator δ^k : C^k → C^{k+1} satisfies **δ² = 0**, dual to
the boundary property ∂² = 0. This is encoded as the identity
`coboundarySquaredZero`.

The **cohomology group** H^k = ker(δ^k) / im(δ^{k-1}) measures obstructions
to extending local data to global data.

## Sheaf Theory

A {@class https://uor.foundation/cohomology/Sheaf} assigns algebraic data to
each open set of the constraint topology, subject to restriction and gluing
axioms. The key components are:

| Class | Role |
|-------|------|
| {@class https://uor.foundation/cohomology/Stalk} | Local data at a single point (constraint) |
| {@class https://uor.foundation/cohomology/Section} | Compatible assignment of data over an open set |
| {@class https://uor.foundation/cohomology/GluingObstruction} | Failure of local sections to assemble globally |

A **local section** is a section defined over a small open neighborhood. The
**restriction maps** ensure that sections agree on overlaps.

## De Rham Duality

The identity `deRhamDuality` establishes a natural pairing between homology
and cohomology:

> H_k × H^k → R

This pairing is non-degenerate: every homology class can be detected by
evaluating a cohomology class on it. The identity `sheafCohomologyBridge`
connects sheaf cohomology to the abstract cochain definition.

## Local-to-Global Principle

The identity `localGlobalPrinciple` captures when local sections can be glued
into global sections. Specifically:

- If H^1 = 0 (no gluing obstructions), every compatible family of local
  sections extends to a unique global section.
- If H^1 is nontrivial, the {@class https://uor.foundation/cohomology/GluingObstruction}
  classes in H^1 classify the distinct ways that gluing can fail.

This principle is the cohomological foundation of the resolution pipeline:
local constraint satisfaction does not always imply global resolution. See
[Sheaf Semantics](sheaf-semantics.html) for worked examples of gluing
obstructions and their resolution-theoretic interpretation.
