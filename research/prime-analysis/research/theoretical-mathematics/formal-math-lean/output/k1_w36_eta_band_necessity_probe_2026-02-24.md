# K1 Eta-Band Necessity Probe (2026-02-24)

## Config
- gamma_n: 478.942181535
- eta: 0.01
- theta: 0.5
- x1: 1e+21
- betas: [0.5001, 0.51, 0.52, 0.55]
- xmax_list: [1e+30, 1e+60, 1e+120, 1e+240]
- x_grid: 6000
- t_grid: 120000

| beta | xmax | C_band | ratio_vs_prev |
|---:|---:|---:|---:|
| 0.5001 | 1e+30 | 333.459 | - |
| 0.5001 | 1e+60 | 2817.85 | 8.45036 |
| 0.5001 | 1e+120 | 45566 | 16.1705 |
| 0.5001 | 1e+240 | 2.84937e+06 | 62.5326 |
| 0.5100 | 1e+30 | 168.284 | - |
| 0.5100 | 1e+60 | 717.659 | 4.26457 |
| 0.5100 | 1e+120 | 2955.57 | 4.11835 |
| 0.5100 | 1e+240 | 11988 | 4.05608 |
| 0.5200 | 1e+30 | 84.3418 | - |
| 0.5200 | 1e+60 | 180.268 | 2.13735 |
| 0.5200 | 1e+120 | 207.371 | 1.15035 |
| 0.5200 | 1e+240 | 207.371 | 1 |
| 0.5500 | 1e+30 | 11.4026 | - |
| 0.5500 | 1e+60 | 11.4026 | 1 |
| 0.5500 | 1e+120 | 11.4026 | 1 |
| 0.5500 | 1e+240 | 11.4026 | 0.999999 |

## Interpretation
- Growth in `C_band` as `xmax` increases indicates the x^{-eta} majorant constant does not stabilize in the tested range.
- Stabilization indicates finite majorant compatibility for that `(beta, eta, theta)` regime.
