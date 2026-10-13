# K1 W29 Local Research Audit (2026-02-24)

## Target
- `R6DualBandTailRepresentationCandidateShapeTerm`
- Source: `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

## Clause Audit
- exists witness with finite-window bound: **empirical_support_only**
  - evidence: /Users/adminamn/Documents/New project/research/output/r6_piecewise_majorant_growth_scan_2026-02-24_eta_fixed.md
- eta fixed to 0.01, x1 fixed to 1e21, 256 omitted modes: **empirical_support_only**
  - evidence: /Users/adminamn/Documents/New project/research/output/r6_tail_representation_scan_2026-02-24.md, /Users/adminamn/Documents/New project/research/output/r6_tail_representation_candidate_modes_2026-02-24_topk.md
- exact equality for all x>=x1: **not_established**
  - note: all local runs are finite-grid least-squares fits and explicitly labeled as evidence, not proof
  - evidence: /Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_top256_eta001_x1e21_r1e-1.md
- for all E, all right-half zeros s: **not_established**
  - note: local probes evaluate truncated explicit-formula surrogate on finite grids only

## New Stress Tests
### 4096-grid mode-count sweep (x1=1e21, eta=0.01, ridge=0.1)
- k=64, tail_points=273, coeffs=129, tail_sup_ratio=0.331850, tail_rmse_ratio=0.272001, all_sup_ratio=1.061637
- k=128, tail_points=273, coeffs=257, tail_sup_ratio=0.202560, tail_rmse_ratio=0.167517, all_sup_ratio=1.038249
- k=256, tail_points=273, coeffs=513, tail_sup_ratio=0.052312, tail_rmse_ratio=0.061398, all_sup_ratio=1.090609
- k=384, tail_points=273, coeffs=769, tail_sup_ratio=0.052648, tail_rmse_ratio=0.061139, all_sup_ratio=1.113097
- k=512, tail_points=273, coeffs=1025, tail_sup_ratio=0.000504, tail_rmse_ratio=0.000418, all_sup_ratio=0.940695

### 8192-grid mode-count sweep (x1=1e21, eta=0.01, ridge=0.1)
- k=64, tail_points=547, coeffs=129, tail_sup_ratio=0.373634, tail_rmse_ratio=0.307132, all_sup_ratio=1.051540
- k=128, tail_points=547, coeffs=257, tail_sup_ratio=0.292355, tail_rmse_ratio=0.246544, all_sup_ratio=1.022688
- k=256, tail_points=547, coeffs=513, tail_sup_ratio=0.229113, tail_rmse_ratio=0.169146, all_sup_ratio=1.098483
- k=384, tail_points=547, coeffs=769, tail_sup_ratio=0.112331, tail_rmse_ratio=0.091572, all_sup_ratio=1.038705
- k=512, tail_points=547, coeffs=1025, tail_sup_ratio=0.032644, tail_rmse_ratio=0.033007, all_sup_ratio=1.022019

### Ridge sensitivity (8192 grid, k=256)
- ridge=0, tail_sup_ratio=0.155687, tail_rmse_ratio=0.105154, all_sup_ratio=123499535245.259094
- ridge=1e-4, tail_sup_ratio=0.225609, tail_rmse_ratio=0.163690, all_sup_ratio=12.916638
- ridge=1e-3, tail_sup_ratio=0.231493, tail_rmse_ratio=0.167068, all_sup_ratio=3.722810
- ridge=1e-2, tail_sup_ratio=0.230799, tail_rmse_ratio=0.168411, all_sup_ratio=1.060351
- ridge=1e-1, tail_sup_ratio=0.229113, tail_rmse_ratio=0.169146, all_sup_ratio=1.098483
- ridge=1, tail_sup_ratio=0.225607, tail_rmse_ratio=0.170732, all_sup_ratio=1.065535

### Out-of-sample cross-validation (evaluate fits on dense 32768 grid)
- fit@4096 -> eval@32768: tail_points=2185, sup_ratio=0.522822, rmse_ratio=0.317272
- fit@8192 -> eval@32768: tail_points=2185, sup_ratio=0.400088, rmse_ratio=0.242451

### Residual high-frequency evidence (8192-grid, k=256 fit)
- Top correlations with frequencies beyond first 256 zeros remain nonzero; top 10:
  - omega=760.282366984, corr_amp=1.082e-03
  - omega=976.178502421, corr_amp=9.431e-04
  - omega=730.416482123, corr_amp=9.389e-04
  - omega=961.669572474, corr_amp=9.092e-04
  - omega=513.668985555, corr_amp=9.092e-04
  - omega=528.406213852, corr_amp=9.089e-04
  - omega=728.758749796, corr_amp=9.054e-04
  - omega=903.099674443, corr_amp=8.699e-04
  - omega=586.742771891, corr_amp=8.630e-04
  - omega=931.009211337, corr_amp=8.595e-04

## Interpretation
- Original 4096-grid tail run at x1=1e21 used 273 tail samples vs 513 fit coefficients (underdetermined).
- Dense-grid 8192 run removes this underdetermination and weakens 256-mode fit quality (tail sup ratio ~0.229 vs ~0.052 previously).
- Out-of-sample cross-validation on 32768 grid degrades further (sup ratio ~0.523 from 4096-fit; ~0.400 from 8192-fit).
- Residual retains significant correlations at higher zeta frequencies beyond the first 256 modes.

## Math-Research Next Options
- Derive explicit-formula truncation theorem: finite 256-mode part + rigorously bounded infinite residual for x>=1e21.
- Prove cancellation-driven residual majorant with explicit eta via zero-density / pair-correlation style bounds.
- Only if such residual is shown identically zero (currently unsupported) could exact finite equality clause be closed as-is.
