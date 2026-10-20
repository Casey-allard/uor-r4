# Monodromy

## Definition

**Monodromy** in the UOR Framework describes how constraint types transform
under parallel transport around closed loops in the constraint nerve. For a
{@class https://uor.foundation/type/ConstrainedType} over the ring R_n, the
monodromy group is a subgroup of the dihedral group D\_{2^n}.

A {@class https://uor.foundation/observable/ClosedConstraintPath} represents a
closed loop in the constraint nerve returning to the same site assignment.
The {@prop https://uor.foundation/observable/pathLength} counts the number of
constraint steps and {@prop https://uor.foundation/observable/pathConstraints}
lists the traversed constraints.

## Holonomy Group

The {@class https://uor.foundation/observable/HolonomyGroup} accumulates
{@class https://uor.foundation/observable/DihedralElement} generators from all
closed paths. Its order is given by
{@prop https://uor.foundation/observable/holonomyGroupOrder}.

A {@class https://uor.foundation/observable/MonodromyClass} classifies the type
according to its holonomy subgroup structure.

Each {@class https://uor.foundation/observable/Monodromy} individual records:

- {@prop https://uor.foundation/observable/monodromyLoop} — the closed path
- {@prop https://uor.foundation/observable/monodromyElement} — the dihedral
  element produced by parallel transport
- {@prop https://uor.foundation/observable/isTrivialMonodromy} — whether the
  element is the identity

## Flat and Twisted Types

Types are classified by their holonomy:

- {@class https://uor.foundation/type/FlatType} — trivial holonomy (identity
  subgroup); disjoint with TwistedType.
- {@class https://uor.foundation/type/TwistedType} — non-trivial holonomy
  (proper subgroup of D\_{2^n}); disjoint with FlatType.

The {@class https://uor.foundation/resolver/MonodromyResolver} computes the
HolonomyGroup by enumerating closed paths via
{@prop https://uor.foundation/resolver/monodromyTarget} and returns the result
via {@prop https://uor.foundation/resolver/holonomyResult}.

## Identity Algebra (MN\_ series)

| Identity | Statement |
|----------|-----------|
| MN\_1 | HolonomyGroup(T) ≤ D\_{2^n} for any ConstrainedType over R_n |
| MN\_2 | Purely additive constraints give trivial holonomy (FlatType) |
| MN\_3 | neg + bnot in closed path gives full dihedral holonomy |
| MN\_4 | Non-trivial HolonomyGroup ⟹ β₁ ≥ 1 |
| MN\_5 | CompleteType ⟹ β₁=0 ⟹ trivial holonomy ⟹ FlatType |
| MN\_6 | monodromy(p₁ · p₂) = monodromy(p₁) · monodromy(p₂) in D\_{2^n} |
| MN\_7 | TwistedType ⟹ H²(N(C(T')); ℤ/2ℤ) ≠ 0 for any QuantumLift |

## Related

- {@class https://uor.foundation/observable/HolonomyGroup}
- {@class https://uor.foundation/type/TwistedType}
- {@class https://uor.foundation/type/FlatType}
- {@class https://uor.foundation/resolver/MonodromyResolver}
- [Quantum Spectral Sequence](quantum-spectral-sequence.html)
- [Type Completeness](type-completeness.html)
