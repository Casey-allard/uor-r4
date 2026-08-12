# Asymptotic Uplift Memo (Post-A4)

Generated: February 17, 2026
Primary artifact: `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_none_z128_ref512_n10m.json`

## Finite-Window Threshold Candidate
Based on the extended A4 run (`n_max` up to `10^7`, `x_step=10^4`), the unified theorem RHS check has:
- zero violations,
- strict negative max gap (`lhs-rhs = -0.1969...`),
- stable base-uniform behavior.

Working finite-window threshold candidate:
- `x0_candidate = 10^6` (all tested scales from `10^6` to `10^7` satisfy the unified bound with margin).

## Unified Constant Pack (extended run)
- `a_ref = -0.0020043789187396974`
- `b_ref = -0.09058032886261758`
- `C0_ref = 0.6025582352013629`
- `C_delta = 4.6379772373825106e-05`
- `tau_M = 383.99980444354367` (tail_Fp, `M=128`, `M_ref=512`)
- `C_H = 6.132322375874561e-07`
- fixed exponents: `beta=2.6`, `A_H=7.2`

## Current Theorem Form (calibrated)
For tested windows and all `W in {30,210,2310,30030}`:
\[
|E(x)|/\sqrt{x}
\le
|a_{ref}|C_H(\log x)^{A_H}
+
|b_{ref}|+C0_{ref}
+
|a_{ref}|C_\Delta(\log x)^\beta\tau_M.
\]

## What remains to claim asymptotic theorem
1. Replace calibrated constants by analytically derived constants independent of data window.
2. Prove the same inequality for all `x >= x0` (not only sampled x-grid).
3. Prove `x0` exists with explicit value under the adopted analytic hypotheses.
4. Connect resulting endpoint to a standard RH-equivalent criterion with the required sharpness.

## Practical next action
Start an analytic note that proves the deterministic inequality chain with symbolic constants, then treat each constant block (`C0_ref`, `C_delta`, `C_H`) as separate lemmas to close.
