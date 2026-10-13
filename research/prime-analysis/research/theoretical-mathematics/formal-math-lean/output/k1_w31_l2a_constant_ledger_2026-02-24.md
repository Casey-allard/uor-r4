# K1 W31 L2A Constant Ledger (2026-02-24)

Formulas used:
- `E_psi(x) < 9.22022*(log x)^(3/2)*exp(-0.8476836*sqrt(log x))` (FKS corollary).
- `Delta(x) << exp(55*A0)*exp(-omega(x))`, `A0=1/48.0718`, with VK asymptotic `omega(x)` model (Bellotti 2025 theorem-shape).

| x | omega_vk(x) | FKS E_psi bound | Bellotti2025 Delta bound |
|---:|---:|---:|---:|
| 1e+21 | 1.661299 | 8.539247e+00 | 5.962013e-01 |
| 1e+24 | 1.787734 | 6.944711e+00 | 5.253913e-01 |
| 1e+30 | 2.021844 | 4.613094e+00 | 4.157290e-01 |
