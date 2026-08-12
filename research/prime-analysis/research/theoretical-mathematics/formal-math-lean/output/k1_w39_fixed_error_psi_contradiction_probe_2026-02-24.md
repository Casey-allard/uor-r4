# Fixed Error psi(x)-x Contradiction Probe (2026-02-24)

- Window: `[xmin, xmax] = [10000, 10000000]`
- Samples: `2400`
- Prime-power events used for psi(x): `665134`
- Sign changes: `377`

## Endpoint Envelope (Empirical Window)

- `C_endpoint_sup_window = 6.490395126e-03` in `|E*(x)| <= C_endpoint * x^(1/2) * (log x)^2`
- `C_endpoint_p95_window = 3.096920813e-03`

## Lower-vs-Upper Contradiction Map

| beta | q10 tail-peak c_beta | q25 c_beta | median c_beta | tail max | crossover(q10) | crossover(q25) | crossover(median) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.5500 | 4.493594e-02 | 6.887293e-02 | 1.073988e-01 | 3.161926e-01 | 9.282657e+71 | 1.051223e+67 | 5.438140e+61 |
| 0.5800 | 2.890001e-02 | 4.571872e-02 | 6.996932e-02 | 2.012948e-01 | 2.305723e+41 | 8.571345e+37 | 4.319764e+34 |
| 0.6000 | 2.160800e-02 | 3.381151e-02 | 5.140525e-02 | 1.540665e-01 | 1.478642e+32 | 2.893786e+29 | 6.746208e+26 |
| 0.6200 | 1.640038e-02 | 2.504004e-02 | 3.831698e-02 | 1.186846e-01 | 2.323781e+26 | 1.660301e+24 | 9.486384e+21 |

Finite-window empirical contradiction-map only. To upgrade to theorem form, replace empirical c_beta and C_endpoint with theorem-grade constants and asymptotic quantifiers.
