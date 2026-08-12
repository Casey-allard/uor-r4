# Fixed Error psi(x)-x Lower-Envelope Ledger (2026-02-24)

- Window: `[xmin, xmax] = [10000, 10000000]`
- Samples: `2400`
- Prime-power events used: `665134`
- Tail fraction: `0.5`
- Alignment threshold: `|cos| >= 0.98`

## Endpoint Upper Envelope (Empirical Window)

- `C_endpoint_sup_window = 6.490395126e-03`
- `C_endpoint_p95_window = 3.096920813e-03`

## Beta Ledger

| beta | tau_best | A | rho_sup | aligned_n | c_obs_q10 | c_tri_q10 | w_obs_grid | w_tri_grid | tri_pos_frac | x_cross(obs_q10) | x_cross(tri_q10) | x_cross(w_obs) | x_cross(w_tri) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.5800 | 14.134725 | 4.997692e-02 | 4.061058 | 155 | 1.088259e-02 | 0.000000e+00 | 1.272127e-01 | 4.942176e-02 | 0.5484 | 2.303050e+48 | NA | 5.409916e+29 | 2.181818e+37 |
| 0.6000 | 14.134725 | 3.908858e-02 | 4.017141 | 156 | 8.106226e-03 | 0.000000e+00 | 9.215742e-02 | 3.660807e-02 | 0.5641 | 6.874222e+37 | NA | 8.505175e+22 | 9.339504e+28 |
| 0.6200 | 14.134725 | 3.061768e-02 | 3.969438 | 156 | 6.032482e-03 | 0.000000e+00 | 6.676215e-02 | 2.652517e-02 | 0.5769 | 1.605444e+31 | NA | 7.256910e+18 | 8.355879e+23 |

Finite-window lower-envelope ledger only. Theorem closure still requires asymptotic quantifiers and theorem-grade main-term/remainder constants.
