# K1 Candidate-Shape Exactness Stress Test (2026-02-24)

## Config
- zeta_zeros_file: research/data/zeta_zeros_odlyzko_100k.json
- loaded_zero_count: 100000
- m_values: [20000, 50000]
- betas: [0.51, 0.55]
- x_windows: [{'x_min': 1e+21, 'x_max': 1e+22}, {'x_min': 1e+23, 'x_max': 1e+24}]
- grid_size: 1024
- head_count: 256
- chunk_a: 512
- chunk_b: 1024

## Summary
- all_rows_robust_nonzero: True
- robust_rows / total_rows: 8 / 8
- min_robust_margin_ratio_to_total: 2.745297e-01
- min_sup_residual_ratio_to_total: 2.745297e-01
- max_chunk_discrepancy_ratio_to_total: 1.329013e-15

| M | beta | x-window | sup_res/total | robust_margin/total | chunk_discrepancy/total | robust_nonzero |
|---:|---:|---:|---:|---:|---:|:---:|
| 20000 | 0.5100 | [1.0e+21, 1.0e+22] | 3.373102e-01 | 3.373102e-01 | 8.897718e-16 | yes |
| 20000 | 0.5100 | [1.0e+23, 1.0e+24] | 2.745297e-01 | 2.745297e-01 | 1.019279e-15 | yes |
| 20000 | 0.5500 | [1.0e+21, 1.0e+22] | 3.353922e-01 | 3.353922e-01 | 8.175782e-16 | yes |
| 20000 | 0.5500 | [1.0e+23, 1.0e+24] | 2.945283e-01 | 2.945283e-01 | 8.187866e-16 | yes |
| 50000 | 0.5100 | [1.0e+21, 1.0e+22] | 3.380166e-01 | 3.380166e-01 | 8.956925e-16 | yes |
| 50000 | 0.5100 | [1.0e+23, 1.0e+24] | 2.767731e-01 | 2.767731e-01 | 1.329013e-15 | yes |
| 50000 | 0.5500 | [1.0e+21, 1.0e+22] | 3.287756e-01 | 3.287756e-01 | 1.207644e-15 | yes |
| 50000 | 0.5500 | [1.0e+23, 1.0e+24] | 2.943988e-01 | 2.943988e-01 | 1.152564e-15 | yes |

## Interpretation
- If all rows are robustly nonzero, finite-head exact identity is not supported for this explicit-formula surrogate across tested tail windows; this favors a finite-head-plus-majorant theorem target over exact finite collapse.
