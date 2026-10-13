# K1 One-Sided Tail Contract Alignment (2026-02-24)

## Tail Bound Input
- beta: `0.62`
- eta: `0.01`
- x1: `1e+21`
- theta_best: `0.466`
- C_total: `124.2680805977631`

## Contract Translation
- `R2(x)=R(x)/x^beta`
- `max(-(R2 x),0) <= q*A1` is implied by `|R2(x)| <= q*A1`
- sufficient: `C_total*x^{-eta} <= q_target*A1`

## A1 Samples
- sample range: `[0.029241453574, 0.031722201975]` (n=5)

### A1_min_sample
- A1: `0.029241453574`
- normalized constant C/A1: `4249.723095446421`
- q(x1) from bound: `2620.358019947570`

| q_target | x_threshold | log10(x_threshold) |
|---:|---:|---:|
| 0.490000 | inf | 393.816455 |
| 0.686000 | inf | 379.203652 |
| 0.882000 | inf | 368.289205 |

### A1_median_sample
- A1: `0.030014504040`
- normalized constant C/A1: `4140.267666285466`
- q(x1) from bound: `2552.868349400321`

| q_target | x_threshold | log10(x_threshold) |
|---:|---:|---:|
| 0.490000 | inf | 392.683234 |
| 0.686000 | inf | 378.070430 |
| 0.882000 | inf | 367.155983 |

### A1_max_sample
- A1: `0.031722201975`
- normalized constant C/A1: `3917.385076104692`
- q(x1) from bound: `2415.440058292915`

| q_target | x_threshold | log10(x_threshold) |
|---:|---:|---:|
| 0.490000 | inf | 390.280018 |
| 0.686000 | inf | 375.667215 |
| 0.882000 | inf | 364.752768 |

Analytic contract-alignment report only.
