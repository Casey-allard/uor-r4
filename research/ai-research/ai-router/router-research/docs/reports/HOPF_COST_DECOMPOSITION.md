# HOPF Cost Decomposition (INC-0047 near-full-proxy confirm)

- Experiment: `inc0047_near_full_proxy_scale_confirm`
- Config: `configs/proxy_transfer_inc0047_near_full_proxy_scale_confirm.json`
- Generated: `2026-03-06T11:01:40`
- Recommendation: Promote HOPF_K25_BASE_IT40_P2_STATIC as stabilized proxy-transfer candidate; it passes the configured route-health gate and beats R0 on both quality and runtime.

## Route Summary
| Route | Health | MSE | Total (s) | Runtime vs baseline | MSE vs baseline | Shells | Sectors | Buckets |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `HOPF_K25_BASE_IT40_P2_STATIC` | pass | 0.003881 | 15.904 | 0.500 | 0.999 | 2.00 | 4.00 | 8.00 |
| `HOPF_PHI2_BAND_IT40_P2_STATIC` | pass | 0.003881 | 16.079 | 0.505 | 0.999 | 2.00 | 10.50 | 18.25 |
| `R0` | fail | 0.003885 | 31.826 | 1.000 | 1.000 | 1.00 | 8.00 | 8.00 |

## Timing Breakdown
| Route | Dataset | Chart Opt | Routing Eval | Train Route | Train Update | Training EMA | Growth | Total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `HOPF_K25_BASE_IT40_P2_STATIC` | 0.281 | 13.057 | 0.215 | 0.007 | 0.177 | 0.194 | 1.691 | 15.904 |
| `HOPF_PHI2_BAND_IT40_P2_STATIC` | 0.245 | 12.973 | 0.183 | 0.009 | 0.183 | 0.209 | 1.986 | 16.079 |
| `R0` | 0.229 | 11.659 | 0.120 | 15.589 | 0.293 | 15.907 | 3.088 | 31.826 |

## Timing Read
- `chart_opt` dominates when it is near the route total.
- `training_route` isolates per-step rerouting cost inside the EMA phase.
- `training_update` isolates bucket prediction and EMA writes inside the EMA phase.

