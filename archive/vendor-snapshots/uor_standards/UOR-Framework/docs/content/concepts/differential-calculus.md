# Differential Calculus

## Definition

The **discrete differential calculus** of UOR defines two derivative operators
on functions f : R_n → R_n:

- **Ring derivative** ∂_R f(x) = f(succ(x)) - f(x), measuring change along
  the ring successor.
- **Hamming derivative** ∂_H f(x) = f(bnot(x)) - f(x), measuring change
  along the Hamming antipode.

These are encoded as {@class https://uor.foundation/op/Identity} individuals
DC_1 and DC_2 in the `op/` namespace.

## The Jacobian

The {@class https://uor.foundation/observable/Jacobian} decomposes the
incompatibility metric site by site. At position k:

> J_k(x) = |d_R(x, succ(x)) - d_H(x, succ(x))| restricted to site k

Key identities:
- **DC_6**: J_k(x) = ∂_R site_k(x)
- **DC_8**: rank(J(x)) = d_H(x, succ(x)) - 1 for generic x
- **DC_9**: Total curvature κ(x) = Σ_k J_k(x)
- **DC_11**: Curvature equipartition — each site contributes approximately
  equally to total curvature.

## Curvature-Weighted Resolution

Identity DC_10 shows that the optimal next constraint in iterative resolution
maximizes the Jacobian over free sites. This connects the differential
calculus to the resolution pipeline: curvature guides constraint selection.

## Worked Example: Ring Derivative on R_4

Consider f(x) = x² mod 16 on R_4 = Z/16Z. The ring derivative is
∂_R f(x) = f(succ(x)) - f(x) = (x+1)² - x² = 2x + 1 (mod 16).

| x | f(x) = x² | succ(x) | f(succ(x)) | ∂_R f(x) = 2x+1 |
|---|-----------|---------|------------|-----------------|
| 0 | 0 | 1 | 1 | 1 |
| 1 | 1 | 2 | 4 | 3 |
| 2 | 4 | 3 | 9 | 5 |
| 3 | 9 | 4 | 0 | 7 |
| 7 | 1 | 8 | 0 | 15 |

The derivative 2x+1 is always odd — it is a unit of Z/16Z for every x.
This means f(x) = x² has no stationary points, and every constraint
application changes the squared observable.

## Commutator Decomposition

Identity DC_4 shows that the fundamental commutator \[neg, bnot\](x) = 2 can
be recovered from the difference of ring and Hamming derivatives of negation.
This provides a differential-geometric interpretation of the critical identity.

## Connection to Resolution

Identity DC_10 shows that the optimal next constraint in iterative resolution
maximizes the {@class https://uor.foundation/observable/Jacobian} over free
sites. This connects differential calculus directly to the resolution loop:
each step selects the constraint whose curvature contribution is largest,
ensuring maximal progress per iteration. See
[Iterative Resolution](iterative-resolution.html) for the convergence analysis
and [Observables](observables.html) for the Jacobian's role as a curvature
observable.
