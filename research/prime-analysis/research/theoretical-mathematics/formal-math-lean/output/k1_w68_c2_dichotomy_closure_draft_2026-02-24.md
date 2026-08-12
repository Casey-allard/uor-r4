# K1 W68 Draft: C2 Dichotomy Closure at `c0 = 1/2` (2026-02-24)

## Target
Close contract `C2(c0)` for one explicit positive buffer, choosing `c0 = 1/2`.

Contract form:
For every `K`, there exists `k >= K` with
`cos(2*pi*rho*k + beta0) >= c0`.

## Assumptions
1. `rho = tau2/tau1` with `rho in (1,2)`.
2. Certified ratio interval excludes `3/2` (equivalently, rational denominator `q=2` is impossible).

## Proposition C2-1/2
Under assumptions (1)-(2), `C2(1/2)` holds.

### Proof (case split)
1. If `rho` is irrational:
   equidistribution of `k*rho mod 1` gives
   `dens{k : cos(2*pi*rho*k + beta0) >= 1/2} = arccos(1/2)/pi = 1/3 > 0`,
   so the set is infinite.

2. If `rho` is rational:
   write `rho = p/q` reduced.
   From `rho in (1,2)`, `q=1` is impossible.
   From assumption (2), `q=2` is impossible.
   Hence `q>=3`.
   By the periodic geometry lemma (W68 RG1/RG2):
   `max_j cos(2*pi*p*j/q + beta0) >= cos(pi/q) >= cos(pi/3) = 1/2`,
   and the witness residue repeats infinitely often.
   Therefore `C2(1/2)` holds.

Thus both branches imply `C2(1/2)`.

## Consequence for K1 Lemma Stack
With this dichotomy packet, C2 no longer depends on fitted/stable `beta0`.
Remaining math focus is then:
1. finalize symbolic eventual rounding-preservation with `c0=1/2`,
2. close one-sided tail theorem constants `C*x^{-eta}` for the `q<a1` gate.

