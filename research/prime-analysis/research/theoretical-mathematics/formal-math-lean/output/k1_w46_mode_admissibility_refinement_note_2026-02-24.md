# K1 W46 Mode-Admissibility Refinement Note (2026-02-24)

## Objective
Refine the mode-admissibility theorem target using cofinal-curve modeling of aligned remainder ratios.

## New artifacts
- Bound-candidate model:
  - `/Users/adminamn/Documents/New project/research/fixed_error_psi_mode_admissibility_bound.py`
  - `/Users/adminamn/Documents/New project/research/output/k1_w45_fixed_error_psi_mode_admissibility_bound_2026-02-24_x3e7_beta062.md`
- Cofinal-curve fit model:
  - `/Users/adminamn/Documents/New project/research/fixed_error_psi_cofinal_curve_fit.py`
  - `/Users/adminamn/Documents/New project/research/output/k1_w46_fixed_error_psi_cofinal_curve_fit_2026-02-24_x1e7_beta062.md`
  - `/Users/adminamn/Documents/New project/research/output/k1_w46_fixed_error_psi_cofinal_curve_fit_2026-02-24_x3e7_beta062.md`
- Cross-window robustness summary:
  - `/Users/adminamn/Documents/New project/research/output/k1_w46_cofinal_curve_robustness_summary_2026-02-24.md`

## Main findings

### 1) Global `D*x^{-eta}` envelope is too conservative
The W45 bound model yields `q_bound(xmax) > 1` for all taus at `x<=3e7`, while empirical cofinal ratios can be much smaller.
Interpretation: global sup envelopes over all points are not sharp enough for aligned-subsequence admissibility.

### 2) Cofinal-curve fit gives a better theorem-facing quantity
Fitting
\[
q(X):=\min_{aligned\ x\ge X}\frac{|R(x)|}{A}
\approx q_\infty - B X^{-\eta},
\]
produces tau-dependent candidate `q_inf_est`.

At `x<=3e7`, `beta=0.62`:
- 5 of 12 taus have `q_inf_est < 0.98`.

### 3) Cross-window robust admissibility is narrower
Comparing `x<=1e7` and `x<=3e7`:
- robust taus with `q_inf_est < 0.98` in both windows: **3**
  - `21.022039639`
  - `30.424876126`
  - `40.918719012`

This is stronger than selecting one tau from one window; it gives a stable admissible-mode candidate set.

## Theorem-target refinement
Replace single-mode target by:

1. There exists an admissible tau in a fixed finite zeta-zero mode family.
2. For that tau, aligned cofinal remainder-ratio curve satisfies
   \[
   \liminf_{aligned}\frac{|R(x)|}{A}\le q_\tau < a_0.
   \]
3. Then subsequence lower envelope follows with positive margin
   \[
   \delta_\tau = a_0-q_\tau > 0.
   \]

This aligns with the contradiction route and with current empirical evidence.

## Remaining hard math
Prove the admissible-tau existence statement analytically (not by finite-window fit), from explicit-formula decomposition and published tail controls.

