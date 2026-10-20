# Homotopy Nerve

## Definition

The **homotopy nerve** is the full homotopy-theoretic refinement of the constraint
nerve. While the basic [Homology](homology.html) pipeline extracts chain-level
invariants (Betti numbers), the homotopy nerve promotes the nerve to a
{@class https://uor.foundation/homology/KanComplex} carrying the complete homotopy
type — including higher homotopy groups, Postnikov truncations, and k-invariants.

A {@class https://uor.foundation/homology/KanComplex} is a simplicial set satisfying
the Kan extension condition: every horn (incomplete simplex boundary) can be filled.
The {@class https://uor.foundation/homology/HornFiller} witnesses certify this
condition at each dimension and position, with
{@prop https://uor.foundation/homology/hornDimension} and
{@prop https://uor.foundation/homology/hornPosition} recording the filled horn.

## Postnikov Tower

The {@class https://uor.foundation/homology/PostnikovTruncation} tau_{<=k} is the
k-th stage of the Postnikov tower: a KanComplex whose homotopy groups pi_j vanish
for j > k. The tower

> tau_{<=0} <- tau_{<=1} <- tau_{<=2} <- ...

successively approximates the full homotopy type. Each stage is linked to its source
via {@prop https://uor.foundation/homology/truncationSource} and carries a
{@prop https://uor.foundation/homology/truncationLevel}.

The {@class https://uor.foundation/homology/KInvariant} kappa_k classifies the
extension from the (k-1)-truncation to the k-truncation. When
{@prop https://uor.foundation/homology/kInvariantTrivial} is true, the truncation
splits as a product — the homotopy type decomposes at that level.

## Homotopy Observables

The homotopy nerve yields three families of observables in the `observable/` namespace:

- {@class https://uor.foundation/observable/HomotopyGroup} — the k-th homotopy group
  pi_k(N(C), v) based at vertex v. For k=0 this is the set of path components; for
  k=1 the fundamental group; for k>=2 these are abelian groups detecting higher
  obstructions. Properties: {@prop https://uor.foundation/observable/homotopyDimension},
  {@prop https://uor.foundation/observable/homotopyRank},
  {@prop https://uor.foundation/observable/homotopyBasepoint}.

- {@class https://uor.foundation/observable/HigherMonodromy} — the image of pi_k into
  the automorphism group of the site. Generalises the
  [Monodromy](monodromy.html) pi_1 -> D_{2^n} homomorphism to higher groups.
  Property: {@prop https://uor.foundation/observable/higherMonodromyDimension}.

- {@class https://uor.foundation/observable/WhiteheadProduct} — the bracket
  \[alpha, beta\] detecting non-trivial interactions between homotopy groups
  that cohomology alone misses. Property:
  {@prop https://uor.foundation/observable/whiteheadTrivial}.

## Bridge to Spectral Sequences

The {@prop https://uor.foundation/observable/postnikovTruncation} property on
{@class https://uor.foundation/observable/SpectralSequencePage} links each page to
the corresponding Postnikov truncation level, connecting the spectral sequence
machinery of [Quantum Spectral Sequence](quantum-spectral-sequence.html) to the
homotopy-nerve tower.

## Key Identities

| Identity | Statement |
|----------|-----------|
| {@ind https://uor.foundation/op/HT_1} | N(C) is a KanComplex |
| {@ind https://uor.foundation/op/HT_2} | pi_0 = Z^{beta_0} |
| {@ind https://uor.foundation/op/HT_3} | pi_1 -> D_{2^n} factors through holonomy |
| {@ind https://uor.foundation/op/HT_4} | pi_k = 0 for k > dim(N(C)) |
| {@ind https://uor.foundation/op/HT_5} | tau_{<=1} classifies FlatType/TwistedType |
| {@ind https://uor.foundation/op/HT_6} | Trivial k-invariants => spectral collapse |
| {@ind https://uor.foundation/op/HT_7} | Non-trivial Whitehead => non-trivial LiftObstruction |
| {@ind https://uor.foundation/op/HT_8} | Hurewicz: pi_k tensor Z = H_k |
