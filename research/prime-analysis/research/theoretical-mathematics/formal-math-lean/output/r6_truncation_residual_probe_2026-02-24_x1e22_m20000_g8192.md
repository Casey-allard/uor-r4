# R6 Truncation Residual Probe

Generated: 2026-02-24T15:27:33.573977+00:00

## Config
- zeta_zeros_file: research/data/zeta_zeros_odlyzko_100k.json
- max_zeta_zeros: 20000
- beta: 0.55
- x_min: 10000000.0
- x_max: 1e+22
- grid_size: 8192
- zero_chunk: 512
- x1: 1e+21
- tail_points: 547
- head_counts: [64, 128, 256, 512, 1024, 2048]

## Total Signal Reference
- total_tail_sup_abs: 0.05084090168715349
- total_all_sup_abs: 0.2859604684513045

## Residual Rows
- N=64, omitted=19936, tail_sup_ratio=0.407894, tail_rmse=7.402908e-03, eta=0.183310, C_tail=1.986063e+02, bound_ratio_tail=1.000000
- N=128, omitted=19872, tail_sup_ratio=0.362758, tail_rmse=6.137032e-03, eta=0.132658, C_tail=1.201279e+01, bound_ratio_tail=1.000000
- N=256, omitted=19744, tail_sup_ratio=0.337215, tail_rmse=4.972366e-03, eta=0.194585, C_tail=2.337952e+02, bound_ratio_tail=1.000000
- N=512, omitted=19488, tail_sup_ratio=0.225710, tail_rmse=3.831352e-03, eta=0.103627, C_tail=1.921423e+00, bound_ratio_tail=1.000000
- N=1024, omitted=18976, tail_sup_ratio=0.194974, tail_rmse=2.983815e-03, eta=0.163688, C_tail=2.774563e+01, bound_ratio_tail=1.000000
- N=2048, omitted=17952, tail_sup_ratio=0.172859, tail_rmse=2.230927e-03, eta=0.096707, C_tail=9.454635e-01, bound_ratio_tail=1.000000

## Interpretation
- This measures finite truncation residuals of a selected explicit-formula surrogate; it is evidence for tail-majorant behavior, not a formal theorem.
