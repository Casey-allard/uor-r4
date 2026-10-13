# K1 L2A Inverse Feasibility Map (2026-02-24)

## Config
- gamma_n: 478.942181535
- eta_target: 0.01
- x1: 1e+21
- xmax: 1e+120
- x_grid: 5000
- beta_min: 0.51
- beta_max: 0.75
- beta_step: 0.01
- theta_min: 0.13
- theta_max: 0.75
- theta_step: 0.01
- t_grid: 120000
- a0: 0.020802216684209868
- c_targets: [20.0, 30.0, 50.0, 100.0]

## External High Constants (eta target)
- c_fks: 13.8490372919
- c_bellotti_2025_model: 0.96692523665
- c_selected: 0.96692523665

## Global Best (largest admissible C_A) by C_target
- C_target=20: beta_lower=0.75, theta=0.75, eta_raw=0.5000, c_floor=0.968628, max_CA=1.58708e+08
- C_target=30: beta_lower=0.75, theta=0.75, eta_raw=0.5000, c_floor=0.968628, max_CA=2.42102e+08
- C_target=50: beta_lower=0.75, theta=0.75, eta_raw=0.5000, c_floor=0.968628, max_CA=4.08888e+08
- C_target=100: beta_lower=0.75, theta=0.75, eta_raw=0.5000, c_floor=0.968628, max_CA=8.25853e+08

## Per-beta Best (largest admissible C_A)
| beta_lower | C_target=20 | C_target=30 | C_target=50 | C_target=100 |
|---:|:---:|:---:|:---:|:---:|
| 0.51 | infeasible | infeasible | infeasible | infeasible |
| 0.52 | infeasible | infeasible | infeasible | infeasible |
| 0.53 | infeasible | infeasible | theta=0.49, eta=0.020, CA_max=0.0002389 | theta=0.68, eta=0.210, CA_max=33.35 |
| 0.54 | theta=0.47, eta=0.010, CA_max=6.309e-35 | theta=0.56, eta=0.100, CA_max=0.06836 | theta=0.72, eta=0.260, CA_max=251.2 | theta=0.75, eta=0.290, CA_max=1.598e+04 |
| 0.55 | theta=0.61, eta=0.160, CA_max=0.9045 | theta=0.75, eta=0.300, CA_max=949.1 | theta=0.75, eta=0.300, CA_max=1.147e+04 | theta=0.75, eta=0.300, CA_max=3.778e+04 |
| 0.56 | theta=0.75, eta=0.310, CA_max=2040 | theta=0.75, eta=0.310, CA_max=1.057e+04 | theta=0.75, eta=0.310, CA_max=2.764e+04 | theta=0.75, eta=0.310, CA_max=7.031e+04 |
| 0.57 | theta=0.75, eta=0.320, CA_max=1.214e+04 | theta=0.75, eta=0.320, CA_max=2.598e+04 | theta=0.75, eta=0.320, CA_max=5.366e+04 | theta=0.75, eta=0.320, CA_max=1.229e+05 |
| 0.58 | theta=0.75, eta=0.330, CA_max=2.852e+04 | theta=0.75, eta=0.330, CA_max=5.096e+04 | theta=0.75, eta=0.330, CA_max=9.586e+04 | theta=0.75, eta=0.330, CA_max=2.081e+05 |
| 0.59 | theta=0.75, eta=0.340, CA_max=5.508e+04 | theta=0.75, eta=0.340, CA_max=9.149e+04 | theta=0.75, eta=0.340, CA_max=1.643e+05 | theta=0.75, eta=0.340, CA_max=3.463e+05 |
| 0.60 | theta=0.75, eta=0.350, CA_max=9.816e+04 | theta=0.75, eta=0.350, CA_max=1.572e+05 | theta=0.75, eta=0.350, CA_max=2.753e+05 | theta=0.75, eta=0.350, CA_max=5.705e+05 |
| 0.61 | theta=0.75, eta=0.360, CA_max=1.68e+05 | theta=0.75, eta=0.360, CA_max=2.638e+05 | theta=0.75, eta=0.360, CA_max=4.553e+05 | theta=0.75, eta=0.360, CA_max=9.34e+05 |
| 0.62 | theta=0.75, eta=0.370, CA_max=2.814e+05 | theta=0.75, eta=0.370, CA_max=4.366e+05 | theta=0.75, eta=0.370, CA_max=7.472e+05 | theta=0.75, eta=0.370, CA_max=1.524e+06 |
| 0.63 | theta=0.75, eta=0.380, CA_max=4.651e+05 | theta=0.75, eta=0.380, CA_max=7.17e+05 | theta=0.75, eta=0.380, CA_max=1.221e+06 | theta=0.75, eta=0.380, CA_max=2.48e+06 |
| 0.64 | theta=0.75, eta=0.390, CA_max=7.632e+05 | theta=0.75, eta=0.390, CA_max=1.172e+06 | theta=0.75, eta=0.390, CA_max=1.989e+06 | theta=0.75, eta=0.390, CA_max=4.031e+06 |
| 0.65 | theta=0.75, eta=0.400, CA_max=1.247e+06 | theta=0.75, eta=0.400, CA_max=1.909e+06 | theta=0.75, eta=0.400, CA_max=3.234e+06 | theta=0.75, eta=0.400, CA_max=6.546e+06 |
| 0.66 | theta=0.75, eta=0.410, CA_max=2.031e+06 | theta=0.75, eta=0.410, CA_max=3.105e+06 | theta=0.75, eta=0.410, CA_max=5.253e+06 | theta=0.75, eta=0.410, CA_max=1.063e+07 |
| 0.67 | theta=0.75, eta=0.420, CA_max=3.302e+06 | theta=0.75, eta=0.420, CA_max=5.044e+06 | theta=0.75, eta=0.420, CA_max=8.529e+06 | theta=0.75, eta=0.420, CA_max=1.724e+07 |
| 0.68 | theta=0.75, eta=0.430, CA_max=5.364e+06 | theta=0.75, eta=0.430, CA_max=8.19e+06 | theta=0.75, eta=0.430, CA_max=1.384e+07 | theta=0.75, eta=0.430, CA_max=2.797e+07 |
| 0.69 | theta=0.75, eta=0.440, CA_max=8.708e+06 | theta=0.75, eta=0.440, CA_max=1.329e+07 | theta=0.75, eta=0.440, CA_max=2.246e+07 | theta=0.75, eta=0.440, CA_max=4.537e+07 |
| 0.70 | theta=0.75, eta=0.450, CA_max=1.413e+07 | theta=0.75, eta=0.450, CA_max=2.156e+07 | theta=0.75, eta=0.450, CA_max=3.643e+07 | theta=0.75, eta=0.450, CA_max=7.359e+07 |
| 0.71 | theta=0.75, eta=0.460, CA_max=2.293e+07 | theta=0.75, eta=0.460, CA_max=3.498e+07 | theta=0.75, eta=0.460, CA_max=5.909e+07 | theta=0.75, eta=0.460, CA_max=1.194e+08 |
| 0.72 | theta=0.75, eta=0.470, CA_max=3.719e+07 | theta=0.75, eta=0.470, CA_max=5.674e+07 | theta=0.75, eta=0.470, CA_max=9.584e+07 | theta=0.75, eta=0.470, CA_max=1.936e+08 |
| 0.73 | theta=0.75, eta=0.480, CA_max=6.033e+07 | theta=0.75, eta=0.480, CA_max=9.204e+07 | theta=0.75, eta=0.480, CA_max=1.554e+08 | theta=0.75, eta=0.480, CA_max=3.14e+08 |
| 0.74 | theta=0.75, eta=0.490, CA_max=9.785e+07 | theta=0.75, eta=0.490, CA_max=1.493e+08 | theta=0.75, eta=0.490, CA_max=2.521e+08 | theta=0.75, eta=0.490, CA_max=5.092e+08 |
| 0.75 | theta=0.75, eta=0.500, CA_max=1.587e+08 | theta=0.75, eta=0.500, CA_max=2.421e+08 | theta=0.75, eta=0.500, CA_max=4.089e+08 | theta=0.75, eta=0.500, CA_max=8.259e+08 |

## Interpretation
- This map isolates the remaining theorem obligation numerically: supply a Branch-A constant `C_A` and a justified `beta_lower` regime that land above one of these feasibility thresholds.
- It does not prove RH; it quantifies the exact constant envelope still needed for the final math bridge.
