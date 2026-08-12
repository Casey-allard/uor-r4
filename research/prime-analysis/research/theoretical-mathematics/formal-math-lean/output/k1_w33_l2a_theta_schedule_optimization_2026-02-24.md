# K1 W33 L2A Theta Schedule Optimization (2026-02-24)

## Objective
Choose `T(x)=x^theta` to minimize the explicit combined constant candidate
\[
C_{\text{total},A}(\theta)=C_{\text{band}}(\theta)+C_{\text{high}}+C_{\text{rem},A}(\theta),
\]
at fixed `eta=0.01`, `beta=0.55`, `x>=1e21`.

## Method
Used explicit-only ledger script:
- `/Users/adminamn/Documents/New project/research/k1_l2a_explicit_ledger.py`

Scanned `theta` from `0.13` to `0.30` (step `0.01`), with:
- `x in [1e21, 1e120]`
- Trudgian-like explicit `N(T)` envelope for band integral bound.
- High-envelope candidate = min(FKS, Bellotti-2025-model) at eta-target.
- Remainder A model: `(log x)^2 / x^theta`.

## Selected results
| theta | C_band | C_rem_A | C_total_A |
|---:|---:|---:|---:|
| 0.13 | 0.401130 | 7.061061 | 8.429116 |
| 0.16 | 0.732320 | 1.655274 | 3.354520 |
| 0.18 | 1.009428 | 0.629318 | 2.605671 |
| 0.19 | 1.164744 | 0.388034 | 2.519704 |
| 0.20 | 1.331182 | 0.239260 | 2.537367 |
| 0.22 | 1.697281 | 0.090964 | 2.755171 |
| 0.25 | 2.329067 | 0.021324 | 3.317316 |
| 0.30 | 3.601012 | 0.001901 | 4.569837 |

Best in this scan:
- `theta* = 0.19`, with `C_total_A ≈ 2.519704`.

Dedicated high-resolution run:
- `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_explicit_ledger_theta019_2026-02-24.md`

## Interpretation
- `theta=0.13` solved the non-empty finite-band issue, but is suboptimal once explicit remainder constants are included.
- `theta≈0.19` gives the best current explicit tradeoff under model-A remainder normalization.
- Next derivation step should carry `theta=0.19` as the primary endpoint schedule unless theorem-level remainder analysis forces a different normalization.
