# K1 W68: Rational-Branch Periodic Geometry Lemma for C2 (2026-02-24)

## Setup
Let
`Theta_k = 2*pi*rho*k + beta0`, with `rho = p/q` in lowest terms and `q >= 2`.

Then modulo `2*pi`,
`{Theta_k : k in Z}` is the finite orbit
`{beta0 + 2*pi*j/q : j = 0,...,q-1}` (up to permutation by `p`).

## Lemma RG1 (Uniform cosine witness on a q-cycle)
For every `beta0` and every reduced `p/q`:

`max_{0<=j<q} cos(2*pi*p*j/q + beta0) >= cos(pi/q)`.

### Proof
The `q` orbit points are equally spaced by arc length `2*pi/q`.
Partition the circle into Voronoi arcs around those points. Any target angle
(in particular angle `0`) is within angular distance at most `pi/q` from some orbit point.
Call that point angle `u`. Then `|u| <= pi/q` modulo `2*pi`, hence
`cos(u) >= cos(pi/q)`.
This gives the bound above.

## Corollary RG2 (Infinite buffered anchors on rational branch)
Fix `c0 <= cos(pi/q)`.
By RG1 there exists residue `j*` with
`cos(2*pi*p*j*/q + beta0) >= c0`.
Then for `k = j* + m*q` (`m >= 0`),
`Theta_k = 2*pi*p*m + (2*pi*p*j*/q + beta0)`,
so `cos(Theta_k)` equals the same witness value for all `m`.
Therefore `{k : cos(Theta_k) >= c0}` is infinite.

## Denominator-3 Corollary (q>=3 gives fixed buffer 1/2)
If `q >= 3`, then
`cos(pi/q) >= cos(pi/3) = 1/2`.
So rational-branch C2 holds with the branch-uniform buffer `c0 = 1/2`,
independent of `beta0`.

## Consequence for Current K1 Track
The remaining rational C2 work is reduced to denominator exclusion of `q=2`.
Since `rho in (1,2)`, denominator `q=2` corresponds to `rho = 3/2`.
Any certified ratio interval excluding `3/2` therefore yields `q>=3`,
and thus a theorem-level rational C2 buffer `c0=1/2` (or stronger if higher `q` is certified).

Numerical interval certificates produced in W68:
- narrow interval file:
  `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_2026-02-24.json`
- conservative interval file:
  `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_conservative_2026-02-24.json`

