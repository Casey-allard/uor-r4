# K1 Branch-A Empirical C_A Scan (2026-02-24)

## Config
- zeta_zeros_file: research/data/zeta_zeros_odlyzko_100k.json
- max_zeta_zeros: 100000
- n_head: 256
- beta: 0.55
- eta_target: 0.01
- x1: 1e+21
- xmax: 1e+30
- grid_size: 1024
- zero_chunk: 512
- thetas: 20 values from 0.13 to 0.7

| theta | eta_raw | asymptotic_feasible | clipped_frac | CA_sup | CA_q99 | eta_fit | C_eta_target |
|---:|---:|:---:|---:|---:|---:|---:|---:|
| 0.13 | -0.3200 | no | 0.000 | 1.04291e-12 | 5.28699e-13 | 0.1129 | 0.0230719 |
| 0.16 | -0.2900 | no | 0.000 | 1.68461e-12 | 1.01185e-12 | 0.1481 | 0.0120691 |
| 0.19 | -0.2600 | no | 0.482 | 4.20196e-12 | 2.32255e-12 | 49.3210 | 0.00465372 |
| 0.22 | -0.2300 | no | 0.871 | 4.93162e-12 | 3.05904e-12 | 22.1493 | 0.00141138 |
| 0.25 | -0.2000 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.28 | -0.1700 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.31 | -0.1400 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.34 | -0.1100 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.37 | -0.0800 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.40 | -0.0500 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.43 | -0.0200 | no | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.46 | 0.0100 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.49 | 0.0400 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.52 | 0.0700 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.55 | 0.1000 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.58 | 0.1300 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.61 | 0.1600 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.64 | 0.1900 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.67 | 0.2200 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |
| 0.70 | 0.2500 | yes | 1.000 | 0 | 0 | 0.0000 | 0 |

## Best by Empirical C_A
- theta: 0.13
- CA_sup: 1.0429121744548879e-12
- eta_raw: -0.31999999999999995

## Interpretation
- `CA_sup` estimates the finite-range constant needed for Branch-A remainder inequality using 100k-zero surrogate tails.
- Asymptotic feasibility still requires `eta_raw > eta_target`; rows marked `no` fail that condition for x->infinity.
