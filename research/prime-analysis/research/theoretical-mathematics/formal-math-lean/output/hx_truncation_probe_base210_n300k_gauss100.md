# H(x) Truncation Probe

Generated: 2026-02-17T03:28:47.148038+00:00

- base: 210
- n_max: 300000
- u_mode: log
- zero_kernel: gaussian
- kernel_scale: 100.0
- zero limits: [32, 64, 96, 128, 192, 256]

## Bridge Metrics by Zero Limit

- M=32 corr=-0.404815 slope=-0.026848 rmse=0.187428
- M=64 corr=-0.401347 slope=-0.026613 rmse=0.187741
- M=96 corr=-0.402451 slope=-0.026795 rmse=0.187641
- M=128 corr=-0.402337 slope=-0.026783 rmse=0.187652
- M=192 corr=-0.402336 slope=-0.026783 rmse=0.187652
- M=256 corr=-0.402336 slope=-0.026783 rmse=0.187652

## Consecutive Truncation Differences

- 32 -> 64: max_abs_diff=0.957815 l2_diff=0.404375
- 64 -> 96: max_abs_diff=0.096495 l2_diff=0.047031
- 96 -> 128: max_abs_diff=0.010826 l2_diff=0.005221
- 128 -> 192: max_abs_diff=0.000722 l2_diff=0.000327
- 192 -> 256: max_abs_diff=0.000001 l2_diff=0.000000
