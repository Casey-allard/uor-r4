# A2 Infinite-Tail Lemma Skeleton

Generated: 2026-02-17T21:48:03.624260+00:00

## Candidate Statement

For wheel family W and all x >= x0, M >= M0: Delta_M(x;W) <= C_delta * (log x)^beta * tau_infty(M), with tau_infty(M)->0.

## Frozen Constants

- `beta_fixed`: `2.6`
- `C_delta_uplifted`: `0.0013231345976502436`
- `density_model`: `n_diff_explicit`
- `density_a`: `0.15915494309189535`
- `density_b`: `0.5`
- `rvm_c0`: `0.5`
- `rvm_c1`: `1.0`
- `nbound_c1`: `0.1038`
- `nbound_c2`: `0.2573`
- `nbound_c3`: `9.3675`
- `nbound_h`: `1.0`
- `kernel`: `gaussian`
- `kernel_scale`: `100.0`
- `m_ref`: `512`
- `gamma_ref`: `826.905810954`

## Theorem Assumptions

- source: `HSW2021` (https://arxiv.org/abs/2107.06506)
- [HSW2021] Assume explicit zero-count error bound |N(T)-M(T)| <= 0.1038 log(T) + 0.2573 log log(T) + 9.3675 for T>=e, where M(T)=T/(2pi)*log(T/(2pi e)).
- Define derivative majorant by finite difference: N'(t)_up := (M(t+h)-M(t) + E(t+h)+E(t))/h with h>0.
- Use N'(t)_up in the weighted tail integral for tau_infty(M), with kernel-weighted integrability and monotone decay in M.

## Tau Majorant Table

| M | tau_finite_ref | tau_infinite_extra_majorant | tau_infinite_majorant |
|---:|---:|---:|---:|
| 64 | 0.729868701081 | 2.6564219021e-28 | 0.729868701081 |
| 128 | 0.00320370327984 | 2.6564219021e-28 | 0.00320370327984 |
| 192 | 2.9624307922e-06 | 2.6564219021e-28 | 2.9624307922e-06 |
| 256 | 6.64399217023e-10 | 2.6564219021e-28 | 6.64399217023e-10 |

## Validation Snapshot

- `train_holds`: `True`
- `valid_holds`: `True`
- `valid_ratio_max`: `0.947859440603`
- `valid_max_gap_delta_minus_rhs`: `-5.23935442499e-10`

## Analytic Obligations (O2)

- State and cite theorem-side assumptions for explicit zero-count bound constants used in N'(t)_up.
- Prove sum-to-integral tail domination for weighted term t/sqrt(1/4+t^2) * K(t).
- Show tau_infty(M) is monotone nonincreasing in M and tends to 0 with explicit rate.
- Derive final C_delta, x0, M0 independent of finite sampled n-grid and m-grid.

## Sources

- `research/output/a2_theorem_constant_pack_refresh_2026-02-17_sf3p5.json`
- `research/output/a2_infinite_tail_uplift_2026-02-17_n_diff_explicit_hsw2022_sf3p5.json`
