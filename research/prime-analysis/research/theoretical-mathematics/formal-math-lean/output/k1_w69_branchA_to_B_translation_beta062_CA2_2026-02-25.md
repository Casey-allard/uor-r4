# K1 Branch-A -> Branch-B Translation Ledger (2026-02-24)

## Config
- gamma_n: 478.942181535
- beta_lower: 0.62
- eta_target: 0.01
- x1: 1e+21
- xmax: 1e+120
- x_grid: 10000
- theta_start: 0.39
- theta_end: 0.49
- theta_step: 0.005
- t_grid: 120000
- c_a: 2.0
- a0: 0.020802216684209868

## Threshold
- theta_min_for_eta_target_under_branch_A: 0.39

## External High Constants
- c_fks: 13.8490372919
- c_bellotti_2025_model: 0.96692523665
- c_selected: 0.96692523665

| theta | feasible | eta_raw | c_band | c_rem_factor/CA | c_total (CA applied) |
|---:|:---:|---:|---:|---:|---:|
| 0.39 | yes | 0.0100 | 0.21647 | 7.19566e+33 | 1.43913e+34 |
| 0.40 | yes | 0.0150 | 0.222931 | 21653.6 | 43308.5 |
| 0.40 | yes | 0.0200 | 0.229483 | 5413.41 | 10828 |
| 0.41 | yes | 0.0250 | 0.236126 | 2405.96 | 4813.12 |
| 0.41 | yes | 0.0300 | 0.242861 | 1353.35 | 2707.92 |
| 0.41 | yes | 0.0350 | 0.249686 | 866.146 | 1733.51 |
| 0.42 | yes | 0.0400 | 0.256603 | 601.49 | 1204.2 |
| 0.42 | yes | 0.0450 | 0.26361 | 441.911 | 885.053 |
| 0.43 | yes | 0.0500 | 0.270709 | 338.338 | 677.914 |
| 0.43 | yes | 0.0550 | 0.277899 | 265.381 | 532.007 |
| 0.44 | yes | 0.0600 | 0.28518 | 208.387 | 418.025 |
| 0.45 | yes | 0.0650 | 0.292552 | 163.633 | 328.525 |
| 0.45 | yes | 0.0700 | 0.300016 | 128.49 | 258.247 |
| 0.46 | yes | 0.0750 | 0.30757 | 100.895 | 203.065 |
| 0.46 | yes | 0.0800 | 0.315216 | 79.2264 | 159.735 |
| 0.47 | yes | 0.0850 | 0.322953 | 62.2114 | 125.713 |
| 0.47 | yes | 0.0900 | 0.330781 | 48.8506 | 98.9989 |
| 0.47 | yes | 0.0950 | 0.3387 | 38.3592 | 78.0241 |
| 0.48 | yes | 0.1000 | 0.34671 | 30.121 | 61.5557 |
| 0.48 | yes | 0.1050 | 0.354811 | 23.6521 | 48.626 |
| 0.49 | yes | 0.1100 | 0.363003 | 18.5725 | 38.4749 |

## Best Feasible
- theta: 0.49
- eta_raw: 0.10999999999999988
- c_band: 0.36300347073921135
- c_rem_factor_per_CA: 18.572482887518913
- c_total: 38.47489448242712

## Interpretation
- This ledger enforces the Branch-A normalization gate; rows below theta_min are marked infeasible.
- To change absolute totals, replace `c_a` with theorem-grade remainder constants from imported sources.
