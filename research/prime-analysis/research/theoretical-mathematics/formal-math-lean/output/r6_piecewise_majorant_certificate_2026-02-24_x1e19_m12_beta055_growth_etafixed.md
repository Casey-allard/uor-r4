# R6 Piecewise Majorant Certificate Candidate

Generated: 2026-02-24T14:19:36.764772+00:00

## Candidate term
- `R6DualBandPiecewisePowerMajorantWitnessTerm`

## Decomposition metadata
- beta: 0.55
- tau_main: 14.134725142
- a_main: -0.000420156718853
- b_main: -0.0344243358517
- dominant_index: 0
- mode_count: 12
- decay_term_coeff: -29.8489862231

## Finite-window certificate
- C: 0.290778802866
- eta: 0.0343010349523
- x0: 4.0080248261e+16
- x1: 1e+19
- window_grid_points: 819
- window_max_ratio_to_bound: 0.952380952381
- window_mean_ratio_to_bound: 0.291661697243

## Checks
- mode_count_ge_6: True
- dominant_index_valid: True
- beta_gt_half: True
- eta_pos: True
- c_window_nonneg: True
- window_bound_verified: True

## Remaining asymptotic obligation
- open: True
- required_shape: For all x >= x1, |R(x)/x^beta| <= C*x^(-eta) with the same C, eta as the finite window certificate.
- x1: 1e+19
- eta: 0.0343010349523
- C: 0.290778802866

## Interpretation
- status: finite-window-certificate-ready
- note: This artifact certifies the finite-window piece of the R6 piecewise contract and leaves only the asymptotic tail theorem obligation open.
