# H(x) Truncation Probe

Generated: 2026-02-17T03:22:57.016482+00:00

- base: 210
- n_max: 300000
- u_mode: log
- zero limits: [32, 64, 96, 128, 192, 256]

## Bridge Metrics by Zero Limit

- M=32 corr=-0.453066 slope=-0.036679 rmse=0.182729
- M=64 corr=-0.517135 slope=-0.051566 rmse=0.175438
- M=96 corr=-0.329860 slope=-0.024366 rmse=0.193501
- M=128 corr=-0.329003 slope=-0.025341 rmse=0.193563
- M=192 corr=-0.222766 slope=-0.014252 rmse=0.199823
- M=256 corr=-0.188361 slope=-0.011233 rmse=0.201305

## Consecutive Truncation Differences

- 32 -> 64: max_abs_diff=3.085651 l2_diff=1.040891
- 64 -> 96: max_abs_diff=3.920952 l2_diff=2.387638
- 96 -> 128: max_abs_diff=1.059446 l2_diff=0.433911
- 128 -> 192: max_abs_diff=3.722305 l2_diff=1.651818
- 192 -> 256: max_abs_diff=1.105779 l2_diff=0.510179
