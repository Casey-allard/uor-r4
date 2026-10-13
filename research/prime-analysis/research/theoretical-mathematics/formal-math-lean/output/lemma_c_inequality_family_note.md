# Lemma C Inequality Family Comparison

Generated: February 17, 2026

Compared probes:
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z64_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z128_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z192_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z256_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_gauss100_z192_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_gauss100_z256_ref512.json`

## Summary Table
- none, M=64: `C1=0.375`, `C2=0`, objective `0.375`, holds grid `True`, max gap `-9.324e-02`
- none, M=128: `C1=0.375`, `C2=0`, objective `0.375`, holds grid `True`, max gap `-1.259e-02`
- none, M=192: `C1=4.125`, `C2=3.119`, objective `7.244`, holds grid `True`, max gap `0`
- none, M=256: `C1=1.6875`, `C2=0.03717`, objective `1.7247`, holds grid `True`, max gap `0`
- gaussian100, M=192: `C1=0`, `C2=501.615`, objective `501.615`, holds grid `True`, max gap `0`
- gaussian100, M=256: `C1=0`, `C2=3790.161`, objective `3790.161`, holds grid `True`, max gap `0`

## Interpretation
1. All tested configurations satisfy the finite-grid cover inequality (zero violations).
2. Coefficient quality varies strongly; some solutions are numerically loose/ill-conditioned.
3. Most stable theorem-facing finite-window candidate in this family: **kernel=none, M=128**.

Reason:
- small objective (`0.375`) with exact same compact constants as M=64,
- much smaller truncation error than M=64,
- still positive strict gap margin (not boundary-fit `max_gap=0`).

## Working Candidate For Lemma C (finite-window)
Use
- kernel=`none`, `M=128`, `M_ref=512`
- inequality constants: `C1=0.375`, `C2=0`
- affine map: `a=-0.009292146698774062`, `b=-0.11850135707099743`.

Next mathematical step: derive analytic (not fitted) `C1` tied to Lemma B truncation constants and explicit-formula smoothing remainder.
