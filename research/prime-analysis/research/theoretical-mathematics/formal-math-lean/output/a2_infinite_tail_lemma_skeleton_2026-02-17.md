# A2 Infinite-Tail Lemma Skeleton

Generated: 2026-02-17T16:50:57.571668+00:00

## Candidate Statement

For wheel family W and all x >= x0, M >= M0: Delta_M(x;W) <= C_delta * (log x)^beta * tau_infty(M), with tau_infty(M)->0.

## Frozen Constants

- `beta_fixed`: `2.6`
- `C_delta_uplifted`: `0.0013231345976502436`
- `density_a`: `0.15915494309189535`
- `density_b`: `0.5`
- `kernel`: `gaussian`
- `kernel_scale`: `100.0`
- `m_ref`: `512`
- `gamma_ref`: `826.905810954`

## Tau Majorant Table

| M | tau_finite_ref | tau_infinite_extra_majorant | tau_infinite_majorant |
|---:|---:|---:|---:|
| 64 | 0.729868701081 | 1.90560525102e-29 | 0.729868701081 |
| 128 | 0.00320370327984 | 1.90560525102e-29 | 0.00320370327984 |
| 192 | 2.9624307922e-06 | 1.90560525102e-29 | 2.9624307922e-06 |
| 256 | 6.64399217023e-10 | 1.90560525102e-29 | 6.64399217023e-10 |

## Validation Snapshot

- `train_holds`: `True`
- `valid_holds`: `True`
- `valid_ratio_max`: `0.947859440603`
- `valid_max_gap_delta_minus_rhs`: `-5.23935442499e-10`

## Analytic Obligations (O2)

- Replace heuristic N'(t)_up = a log t + b with a rigorous explicit zero-density/count bound valid on [gamma_ref, inf).
- Prove sum-to-integral tail domination for weighted term t/sqrt(1/4+t^2) * K(t).
- Show tau_infty(M) is monotone nonincreasing in M and tends to 0 with explicit rate.
- Derive final C_delta independent of finite sampled n-grid and m-grid.

## Sources

- `research/output/a2_theorem_constant_pack_refresh_2026-02-17_sf3p5.json`
- `research/output/a2_infinite_tail_uplift_refresh_2026-02-17_sf3p5.json`
