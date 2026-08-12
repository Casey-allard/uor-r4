# R6 Truncation Residual Probe

Generated: 2026-02-24T18:03:32.349911+00:00

## Config
- zeta_zeros_file: research/data/zeta_zeros_odlyzko_100k.json
- max_zeta_zeros: 100000
- beta: 0.55
- x_min: 10000000.0
- x_max: 1e+24
- grid_size: 4096
- zero_chunk: 512
- x1: 1e+21
- tail_points: 723
- head_counts: [256, 512, 1024, 2048, 4096, 8192]

## Total Signal Reference
- total_tail_sup_abs: 0.05354035155214231
- total_all_sup_abs: 0.2924713947735583

## Residual Rows
- N=256, omitted=99744, tail_sup_ratio=0.278798, tail_rmse=4.441472e-03, eta=0.129993, C_tail=1.435532e+01, bound_ratio_tail=1.000000
- N=512, omitted=99488, tail_sup_ratio=0.231777, tail_rmse=3.711056e-03, eta=0.109135, C_tail=3.308187e+00, bound_ratio_tail=1.000000
- N=1024, omitted=98976, tail_sup_ratio=0.173084, tail_rmse=2.869835e-03, eta=0.087946, C_tail=9.888230e-01, bound_ratio_tail=1.000000
- N=2048, omitted=97952, tail_sup_ratio=0.130501, tail_rmse=2.248440e-03, eta=0.054572, C_tail=1.357449e-01, bound_ratio_tail=1.000000
- N=4096, omitted=95904, tail_sup_ratio=0.117756, tail_rmse=1.700988e-03, eta=0.084003, C_tail=4.216955e-01, bound_ratio_tail=1.000000
- N=8192, omitted=91808, tail_sup_ratio=0.072811, tail_rmse=1.205487e-03, eta=0.070332, C_tail=1.384842e-01, bound_ratio_tail=1.000000

## Interpretation
- This measures finite truncation residuals of a selected explicit-formula surrogate; it is evidence for tail-majorant behavior, not a formal theorem.
