# Fixed Error psi(x)-x Contradiction Probe (2026-02-24)

- Window: `[xmin, xmax] = [10000, 30000000]`
- Samples: `2600`
- Prime-power events used for psi(x): `1858718`
- Sign changes: `426`

## Endpoint Envelope (Empirical Window)

- `C_endpoint_sup_window = 7.686628693e-03` in `|E*(x)| <= C_endpoint * x^(1/2) * (log x)^2`
- `C_endpoint_p95_window = 2.992686681e-03`

## Lower-vs-Upper Contradiction Map

| beta | q10 tail-peak c_beta | q25 c_beta | median c_beta | tail max | crossover(q10) | crossover(q25) | crossover(median) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.5500 | 4.331685e-02 | 6.240585e-02 | 1.016691e-01 | 3.031581e-01 | 2.054654e+74 | 1.369735e+70 | 2.633574e+64 |
| 0.5800 | 2.677608e-02 | 3.925663e-02 | 6.389719e-02 | 1.993233e-01 | 1.433439e+43 | 2.233558e+40 | 4.621709e+36 |
| 0.6000 | 1.950211e-02 | 2.858782e-02 | 4.722661e-02 | 1.509408e-01 | 5.915460e+33 | 3.223930e+31 | 2.739631e+28 |
| 0.6200 | 1.436089e-02 | 2.108361e-02 | 3.445688e-02 | 1.143024e-01 | 7.197244e+27 | 9.031182e+25 | 2.754724e+23 |

Finite-window empirical contradiction-map only. To upgrade to theorem form, replace empirical c_beta and C_endpoint with theorem-grade constants and asymptotic quantifiers.
