# K1 L2A Explicit Ledger (2026-02-24)

## Config
- gamma_n: 478.942181535
- n_head: 256
- beta: 0.62
- eta: 0.01
- theta: 0.5
- x1: 1e+21
- xmax: 1e+120
- x_grid: 8000
- t_grid: 80000
- a0: 0.020802216684209868

## Explicit Band Bound
- c_band_eta: 0.37966167416

## External High Envelopes
- c_fks: 13.8490372919
- c_bellotti_2025_model: 0.96692523665
- c_selected_min: 0.96692523665

## Remainder Candidates
- c_rem_model_A_log2_over_xtheta: 1.199140212e-07
- c_rem_model_B_xhalfminusbeta_log2_over_xtheta: 3.62134554623e-10

## Combined Candidates
- c_total_A: 1.34658703072
- c_total_B: 1.34658691117

## Interpretation
- This is an explicit-only constant ledger: no empirical band/high decomposition. Remaining theorem work is to justify the chosen remainder normalization and replace model-level high envelope usage with the exact target theorem chain.
