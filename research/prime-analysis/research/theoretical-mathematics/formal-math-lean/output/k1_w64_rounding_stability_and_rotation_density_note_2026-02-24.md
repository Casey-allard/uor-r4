# K1 W64 Rounding Stability + Rotation Density Note (2026-02-24)

## Objective
Strengthen C2 from phase mechanics, not residual fitting:
1. rounding stability of tau1 anchors to integers (`W=0`),
2. rotation-density behavior of tau2 signs on anchor phases.

## Rounding stability (`W=0`)
Using `fixed_error_psi_tau12_rounding_stability_probe.py` on windows
\(x\in\{10^7,2\cdot10^7,3\cdot10^7,4\cdot10^7,5\cdot10^7\}\):

- For anchor subset `cos2(anchor) >= c0` with `c0 ∈ {0,0.1,...,0.5}`:
  - minimum anchor fraction across windows: `0.5`,
  - `sign_preserved_fraction = 1.0`,
  - `cos1_alignment_fraction = 1.0`,
  - `both_fraction = 1.0`.
- Conservative minima across windows on that subset:
  - `min cos2(integer) ≈ 0.684818`,
  - `min |cos1(integer)| ≈ 0.999999885`.

So, in current windows, rounding did not break either sign or tau1 alignment on the positive-anchor half.

## Rotation-density check
For anchor rotation sequence
\[
\Theta_k = 2\pi\rho k + \beta_0,\quad \rho=\tau_2/\tau_1\approx1.487262003881,
\]
finite-`N` densities up to `N=300000` match
\[
\mathbb{P}(\cos\Theta_k \ge c_0)=\arccos(c_0)/\pi
\]
to within about `1e-5` for `c0 = 0,0.2,0.4,0.6,0.8`.

This empirically supports the C2 case-split route (periodic rational / equidistribution irrational).

## Practical C2 takeaway
A theorem-target formulation with buffered positivity is now plausible:
- pick any fixed `c0` in `(0, 0.68]`,
- show infinitely many anchors with `cos2(anchor) >= c0`,
- show rounding error eventually preserves `cos2(integer) >= 0` and `|cos1(integer)| >= a1`.

Artifacts:
- `research/output/k1_w64_tau12_rounding_stability_2026-02-24_x1e7.md`
- `research/output/k1_w64_tau12_rounding_stability_2026-02-24_x2e7.md`
- `research/output/k1_w64_tau12_rounding_stability_2026-02-24_x3e7.md`
- `research/output/k1_w64_tau12_rounding_stability_2026-02-24_x4e7.md`
- `research/output/k1_w64_tau12_rounding_stability_2026-02-24_x5e7.md`
- `research/output/k1_w62_tau12_rotation_sign_probe_2026-02-24.md`
