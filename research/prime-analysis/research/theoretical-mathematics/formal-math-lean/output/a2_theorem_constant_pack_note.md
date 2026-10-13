# A2 Theorem Constant Pack Note (Fixed-Beta, Non-Regression)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/a2_theorem_constant_pack_gauss100_ref512_v2_sf3p5.json`

## What changed
- Replaced regression-style A2 calibration with a theorem-shaped constant extraction:
  - fix \(\beta\) in advance,
  - define \(\tau(M)\) explicitly from zero tails,
  - set \(C_\Delta\) as a train-grid sup ratio times safety factor.

Form used:
\[
\Delta_M(x;W)\le C_\Delta(\log x)^\beta \tau(M),
\quad
\tau(M)=\sum_{j>M}^{M_{ref}}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|}.
\]

## Current constants (gaussian kernel, scale 100)
- fixed \(\beta=2.6\),
- \(M\in\{64,128,192,256\}\), \(M_{ref}=512\),
- \(C_\Delta=1.3231345976502436\times10^{-3}\) (safety factor \(3.5\)).

Checks:
- train split (`n in {3e5,1e6}`): holds, zero violations,
- held-out split (`n in {2e6,5e6}`): holds, zero violations.

## Proof interpretation
- This is a stricter theorem scaffold than regression-fit A2 because the constant is a supremum majorant on a split protocol.
- Remaining analytic gap:
  1. replace finite-\(M_{ref}\) tail by true infinite-tail bound,
  2. derive \(C_\Delta\) from analytic channel perturbation inequalities instead of sampled supremum.
