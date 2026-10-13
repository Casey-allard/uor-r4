# K1 W48 Robust Admissible Tau Constants (2026-02-24)

## Objective
Extract a conservative finite-window admissible-mode set and corresponding lower-envelope constants for the contradiction route.

## Inputs
- `/Users/adminamn/Documents/New project/research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x1e7_beta062.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x3e7_beta062.json`

Fixed parameters:
- `beta = 0.62`
- alignment gate `a0 = 0.98`

## Robust admissibility criterion
For each tau, require:
\[
\max\{\mathrm{rr\_cofinal}^{(1e7)},\ \mathrm{rr\_cofinal}^{(3e7)}\} < a_0.
\]

This yields 7 robust admissible taus:
\[
\{14.134725,\ 21.022040,\ 25.010858,\ 30.424876,\ 32.935062,\ 40.918719,\ 43.327073\}.
\]

## Conservative constants (cross-window)
For each robust tau:
- `rr_max = max(rr_cofinal_1e7, rr_cofinal_3e7)`
- `delta = a0 - rr_max`
- `A_min = min(amplitude_1e7, amplitude_3e7)`
- `c_cons = A_min * delta`

These produce a finite-window conservative lower-envelope constant candidate
\[
|E_\*(x)| \gtrsim c_{\mathrm{cons}} x^\beta
\]
on aligned cofinal points (empirical regime).

| tau | rr_max | delta | A_min | c_cons |
|---:|---:|---:|---:|---:|
| 14.134725 | 0.439166 | 0.540834 | 2.920877e-02 | 1.631385e-02 |
| 21.022040 | 0.206532 | 0.773468 | 1.963351e-02 | 1.525365e-02 |
| 25.010858 | 0.622525 | 0.357475 | 1.749863e-02 | 6.255007e-03 |
| 30.424876 | 0.126941 | 0.853059 | 1.559156e-02 | 1.328807e-02 |
| 32.935062 | 0.377044 | 0.602956 | 1.469283e-02 | 8.859754e-03 |
| 40.918719 | 0.319234 | 0.660766 | 1.032938e-02 | 6.824715e-03 |
| 43.327073 | 0.703592 | 0.276408 | 9.642122e-03 | 2.665013e-03 |

## Interpretation
1. Robust admissible-mode evidence persists for a nontrivial finite tau family.
2. Conservative candidate constants `c_cons` are explicitly computable and positive.
3. This supports the admissible-mode contradiction strategy more directly than single-window single-tau choices.

## Remaining theorem work
Prove analytically that at least one tau in an admissible family satisfies the cofinal ratio bound
\[
\liminf_{aligned}\frac{|R(x)|}{A} < a_0,
\]
then use `delta = a0 - q_tau` to obtain theorem-grade lower envelope and close the endpoint contradiction step.

