# Fixed Error psi(x)-x Lower-Envelope Ledger (2026-02-24)

- Window: `[xmin, xmax] = [10000, 30000000]`
- Samples: `2600`
- Prime-power events used: `1858718`
- Tail fraction: `0.5`
- Alignment threshold: `|cos| >= 0.98`

## Endpoint Upper Envelope (Empirical Window)

- `C_endpoint_sup_window = 7.686628693e-03`
- `C_endpoint_p95_window = 2.992686681e-03`

## Beta Ledger

| beta | tau_best | A | rho_sup | aligned_n | c_obs_q10 | c_tri_q10 | w_obs_grid | w_tri_grid | tri_pos_frac | x_cross(obs_q10) | x_cross(tri_q10) | x_cross(w_obs) | x_cross(w_tri) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.5800 | 14.134725 | 4.835210e-02 | 3.898784 | 167 | 8.073485e-03 | 0.000000e+00 | 7.512630e-02 | 3.248450e-02 | 0.6407 | 4.040372e+51 | NA | 2.552016e+35 | 5.580716e+41 |
| 0.6000 | 14.134725 | 3.754426e-02 | 3.796893 | 167 | 5.641893e-03 | 0.000000e+00 | 5.324798e-02 | 2.302056e-02 | 0.6647 | 6.321174e+40 | NA | 4.817940e+27 | 6.269992e+32 |
| 0.6200 | 14.134725 | 2.920877e-02 | 3.690674 | 167 | 4.082377e-03 | 0.000000e+00 | 3.774107e-02 | 1.631385e-02 | 0.6766 | 6.479364e+33 | NA | 9.144896e+22 | 1.702963e+27 |

Finite-window lower-envelope ledger only. Theorem closure still requires asymptotic quantifiers and theorem-grade main-term/remainder constants.
