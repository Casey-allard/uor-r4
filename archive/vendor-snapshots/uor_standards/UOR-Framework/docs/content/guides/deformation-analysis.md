# Deformation Analysis

The **deformation analysis** guide explains how the
{@class https://uor.foundation/resolver/ModuliResolver} computes the local geometry
of the moduli space at a given CompleteType.

## ModuliResolver

The {@class https://uor.foundation/resolver/ModuliResolver} takes a
{@class https://uor.foundation/type/CompleteType} as input via
{@prop https://uor.foundation/resolver/moduliTarget} and produces a
{@class https://uor.foundation/homology/DeformationComplex} via
{@prop https://uor.foundation/resolver/moduliDeformation}.

The resolver:
1. Constructs the deformation complex of T
2. Computes tangent and obstruction dimensions
3. Determines the holonomy stratum containing T
4. Records the results in a {@class https://uor.foundation/observable/StratificationRecord}

## Stratification Record

The {@class https://uor.foundation/observable/StratificationRecord} captures the
holonomy stratification of M_n at a given Witt level:
- {@prop https://uor.foundation/observable/stratificationLevel} — the Witt level
- {@prop https://uor.foundation/observable/stratificationStratum} — links to each
  {@class https://uor.foundation/type/HolonomyStratum} in the decomposition

## Worked Example

Consider a CompleteType T in M_2 (Witt level Q2, R_4 = Z/16Z) with basis
size 4 and trivial holonomy (FlatType):

**Step 1 — Deformation complex.** Construct Def(T):
- H^0 = Aut(T) = {id} (T has no non-trivial automorphisms in D_4)
- H^1 has dimension 3 (three independent first-order deformation directions)
- H^2 = 0 (no obstructions)

The {@prop https://uor.foundation/homology/tangentDimension} is 3 and the
{@prop https://uor.foundation/homology/obstructionDimension} is 0.

**Step 2 — Holonomy stratum.** Since T is a FlatType, it lies in the flat stratum
with codimension 0 ({@ind https://uor.foundation/op/MD_5}). The
{@prop https://uor.foundation/type/stratumCodimension} is 0.

**Step 3 — Versal deformation.** By {@ind https://uor.foundation/op/MD_7}, T admits
a versal deformation of dimension 3 (matching H^1). Since H^2 = 0, any deformation
family through T preserves completeness ({@ind https://uor.foundation/op/MD_8}).

**Step 4 — Tower map site.** The site of M_2 -> M_3 over T has dimension 1
({@ind https://uor.foundation/op/MD_9}) since the obstruction to lifting vanishes.

## Complexity

The ModuliResolver runs in O(n x basisSize^2) time
({@ind https://uor.foundation/op/MR_3}), dominated by the deformation complex
computation. Identity {@ind https://uor.foundation/op/MR_1} ensures that the
resolver's output agrees with the MorphospaceBoundary when restricted to the
achievability boundary.
