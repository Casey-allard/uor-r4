# K1 W66 C2 Assumption Contract (2026-02-24)

## Purpose
Pin C2 to an explicit theorem contract so math work does not drift:
no pipeline refactoring, only closure of the remaining analytic assumptions.

## Contract C2(c0)
Fix `c0 in (0,1)`. Let
`Theta_k = 2*pi*rho*k + beta0`.

Required statement:

For every `K`, there exists `k >= K` with
`cos(Theta_k) >= c0`.

Equivalent set form:
`{k in N : cos(Theta_k) >= c0}` is infinite.

## Accepted proof branches
Any one branch is sufficient.

### Branch I (Irrational rotation)
Assume `rho` is irrational.
Then by equidistribution of `k*rho mod 1`:

`dens{ k : cos(Theta_k) >= c0 } = arccos(c0)/pi > 0`.

Therefore infinitely many such `k`.

### Branch II (Rational periodic witness)
Assume `rho = p/q` with `gcd(p,q)=1` and `q>=2`.
Then `cos(Theta_k)` is periodic with period `q`.

If there exists `j in {0,...,q-1}` with
`cos(2*pi*p*j/q + beta0) >= c0`,
then infinitely many `k` satisfy the same inequality (periodic repetition).

## Why this contract is exact for current track
The rounding lemma needs strictly positive buffer `c0 > 0`.
So the weaker non-buffered fact (`cos >= 0` infinitely often) is not enough.

## Current evidence
- Buffered finite-N density diagnostics (all five fitted windows):
  `/Users/adminamn/Documents/New project/research/output/k1_w66_tau12_buffered_rotation_density_2026-02-24.json`
- Rational-approximation diagnostics for `rho`:
  `/Users/adminamn/Documents/New project/research/output/k1_w66_tau_ratio_rational_approx_2026-02-24.json`

These support Branch I strongly in practice, but they are not proofs.

## Remaining work item for C2
Choose and close one branch theorem-grade:
1. prove/import Branch I (`rho` irrational + equidistribution route), or
2. prove/import Branch II with an explicit periodic witness.

