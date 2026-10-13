# K1 Branch-A -> Branch-B Translation Ledger (2026-02-24)

## Config
- gamma_n: 478.942181535
- beta_lower: 0.55
- eta_target: 0.01
- x1: 1e+21
- xmax: 1e+120
- x_grid: 8000
- theta_start: 0.46
- theta_end: 0.5
- theta_step: 0.01
- t_grid: 160000
- c_a: 5.03
- a0: 0.020802216684209868

## Threshold
- theta_min_for_eta_target_under_branch_A: 0.45999999999999996

## External High Constants
- c_fks: 13.8490372919
- c_bellotti_2025_model: 0.96692523665
- c_selected: 0.96692523665

| theta | feasible | eta_raw | c_band | c_rem_factor/CA | c_total (CA applied) |
|---:|:---:|---:|---:|---:|---:|
| 0.46 | yes | 0.0100 | 9.4961 | 7.19566e+33 | 3.61942e+34 |
| 0.47 | yes | 0.0200 | 9.95652 | 5413.41 | 27240.4 |
| 0.48 | yes | 0.0300 | 10.4277 | 1353.35 | 6818.76 |
| 0.49 | yes | 0.0400 | 10.9098 | 601.49 | 3037.37 |
| 0.50 | yes | 0.0500 | 11.4026 | 338.338 | 1714.21 |

## Best Feasible
- theta: 0.5
- eta_raw: 0.050000000000000044
- c_band: 11.402593803246658
- c_rem_factor_per_CA: 338.3382080915311
- c_total: 1714.2107057402982

## Interpretation
- This ledger enforces the Branch-A normalization gate; rows below theta_min are marked infeasible.
- To change absolute totals, replace `c_a` with theorem-grade remainder constants from imported sources.
