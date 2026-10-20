# Analytical Completeness

## Definition

**Analytical completeness** means that the UOR ontology provides a complete
topological and spectral characterization of the resolution process. Three
structures make this possible: the Cech nerve, Betti numbers, and the
index theorem.

## Cech Nerve

The {@class https://uor.foundation/resolver/CechNerve} is the
{@class https://uor.foundation/homology/SimplicialComplex} whose vertices are
constraints and where a k-simplex exists iff the corresponding k+1 constraints
have nonempty pin intersection. Identity HA_1 formalizes this construction.

The nerve's topology governs resolution behavior:
- **Trivial homology** (all Betti numbers zero) → smooth convergence
- **Non-trivial homology** → potential stalls (identity HA_2)

### Nerve Construction Example

Consider three constraints on R_4 = Z/16Z:

| Constraint | Type | Pins |
|-----------|------|------|
| C_1 | {@ind https://uor.foundation/type/residueConstraintKind} (mod 2, residue 1) | Sites {0} |
| C_2 | {@ind https://uor.foundation/type/depthConstraintKind} (depth 1–2) | Sites {0, 1} |
| C_3 | {@ind https://uor.foundation/type/carryConstraintKind} (pattern "10") | Sites {1} |

Compatible subsets (nonempty pin intersection) form simplices:
- **0-simplices** (vertices): {C_1}, {C_2}, {C_3}
- **1-simplices** (edges): {C_1, C_2} (share site 0), {C_2, C_3} (share site 1)
- **2-simplices**: none — C_1 and C_3 pin disjoint sites

The nerve is a path graph: C_1 — C_2 — C_3. This
{@class https://uor.foundation/homology/SimplicialComplex} has 3 vertices,
2 edges, and is contractible.

## Computing Betti Numbers

From the nerve above, the {@class https://uor.foundation/homology/ChainGroup}
construction gives:
- C_0 = free group on {C_1, C_2, C_3} (rank 3)
- C_1 = free group on {(C_1,C_2), (C_2,C_3)} (rank 2)

The {@class https://uor.foundation/homology/BoundaryOperator} ∂_1 maps each
edge to the difference of its endpoints. Computing:
- ker(∂_1) = 0 (no cycles)
- im(∂_1) has rank 2

The {@class https://uor.foundation/homology/HomologyGroup} results:
- H_0 = ker(∂_0) / im(∂_1) has rank 1 → **β_0 = 1** (one connected component)
- H_1 = ker(∂_1) / im(∂_2) = 0 → **β_1 = 0** (no loops)

Since all higher Betti numbers are zero, the nerve is contractible — resolution
converges without topological obstruction. See [Homology](homology.html) for the
full chain complex machinery.

## Betti Numbers as Observables

{@class https://uor.foundation/observable/BettiNumber} β_k = rank(H_k(N(C)))
counts the k-dimensional holes in the constraint configuration:
- beta_0 counts connected components — constraint clusters that interact independently.
- beta_1 counts loops — cyclic constraint dependencies that may stall resolution.
- Higher beta_k detect higher-dimensional voids.

The Betti-entropy theorem (HA_3) gives a lower bound on residual entropy:

> S_residual ≥ Σ_k β_k × ln 2

## Spectral Gap

The {@class https://uor.foundation/observable/SpectralGap} λ_1 is the smallest
positive eigenvalue of the Cech nerve Laplacian. Identity IT_6 shows
that λ_1 lower-bounds the convergence rate of iterative resolution: larger
spectral gaps mean faster convergence.

## The UOR Index Theorem

The capstone identity IT_7a connects curvature, topology, and entropy:

> Σ κ_k - χ(N(C)) = S_residual / ln 2

where κ_k is the total curvature at site k, χ is the Euler characteristic
(χ = Σ(-1)^k β_k), and S_residual is the residual Shannon entropy. This is
the UOR analog of the Atiyah-Singer index theorem.

In the example above, χ = beta_0 - beta_1 = 1 - 0 = 1. With n = 4 sites and
χ = 1, IT_7c gives a resolution cost lower bound of n - χ = 3 constraint
applications.

Consequences:
- **IT_7b**: S_residual = (Σ κ_k - χ) × ln 2
- **IT_7c**: Resolution cost ≥ n - χ(N(C))
- **IT_7d**: Resolution is complete iff χ(N(C)) = n and all β_k = 0

See the [ψ-Pipeline Guide](../guides/psi-pipeline.html) for how to compute
these invariants step by step.
