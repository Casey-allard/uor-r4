# Moduli Space

## Definition

The **moduli space** M_n is the space of all
{@class https://uor.foundation/type/CompleteType} instances over R_n at a given
Witt level. Its geometry is governed by the
{@class https://uor.foundation/homology/DeformationComplex} at each point, and its
stratification is indexed by holonomy classes from
[Monodromy](monodromy.html).

A {@class https://uor.foundation/type/ModuliSpace} carries:
- {@prop https://uor.foundation/type/moduliWittLevel} — the Witt level of the space
- {@prop https://uor.foundation/type/moduliPoint} — CompleteType members
- {@prop https://uor.foundation/type/moduliDimension} — the dimension of the space

## Deformation Complex

The {@class https://uor.foundation/homology/DeformationComplex} of a CompleteType T
is a {@class https://uor.foundation/homology/ChainComplex} whose cohomology groups
control the local geometry of M_n at T:

- H^0 = automorphisms of T
- H^1 = first-order deformations (tangent space)
- H^2 = obstructions to extending deformations

Properties: {@prop https://uor.foundation/homology/deformationBase},
{@prop https://uor.foundation/homology/tangentDimension},
{@prop https://uor.foundation/homology/obstructionDimension}.

## Holonomy Stratification

The moduli space decomposes into strata indexed by conjugacy classes of subgroups
of D_{2^n}:

- {@class https://uor.foundation/type/HolonomyStratum} — a single stratum, with
  {@prop https://uor.foundation/type/stratumHolonomyClass} linking to the indexing
  MonodromyClass and {@prop https://uor.foundation/type/stratumCodimension}
  recording its codimension in M_n.
- The flat stratum (trivial holonomy, codimension 0) is open and dense.
- Twisted strata have codimension >= 1.

## Deformation Families

A {@class https://uor.foundation/type/DeformationFamily} is a one-parameter family
{C_t} parameterizing a path in M_n. The property
{@prop https://uor.foundation/type/familyPreservesCompleteness} records whether every
member remains a CompleteType.

The {@class https://uor.foundation/type/VersalDeformation} is the formally universal
deformation of a CompleteType T: every other deformation factors through it
(possibly non-uniquely). Properties: {@prop https://uor.foundation/type/versalBase},
{@prop https://uor.foundation/type/versalDimension}.

## Moduli Tower

The {@class https://uor.foundation/type/ModuliTowerMap} M_n -> M_{n+1} is induced by
WittLift. Its site over T is the space of completions of the partial lift.
Property: {@prop https://uor.foundation/type/towerMapSource}.

This connects to the [Witt Spectral Sequence](quantum-spectral-sequence.html)
and incremental completeness pipeline.

## Key Identities

| Identity | Statement |
|----------|-----------|
| {@ind https://uor.foundation/op/MD_1} | dim(M_n) = basisSize(T) |
| {@ind https://uor.foundation/op/MD_2} | H^0(Def(T)) = Aut(T) intersect D_{2^n} |
| {@ind https://uor.foundation/op/MD_3} | H^1(Def(T)) = tangent space |
| {@ind https://uor.foundation/op/MD_4} | H^2(Def(T)) = LiftObstruction space |
| {@ind https://uor.foundation/op/MD_5} | FlatType stratum is open (codimension 0) |
| {@ind https://uor.foundation/op/MD_6} | TwistedType strata have codimension >= 1 |
| {@ind https://uor.foundation/op/MD_7} | VersalDeformation exists for every CompleteType |
| {@ind https://uor.foundation/op/MD_8} | Completeness-preserving iff H^2 = 0 along path |
| {@ind https://uor.foundation/op/MD_9} | Site dim = 1 when obstructionTrivial |
| {@ind https://uor.foundation/op/MD_10} | Site empty iff TwistedType at every level |
