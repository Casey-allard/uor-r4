# O3 Asymptotic Inequality Draft

Generated: 2026-02-17T19:19:31.956399+00:00

## Core Template

Assume eta_+(x;W) <= C_eta (log x)^A_eta and E2(x;W)/x <= C_E2 (log x)^A_E2 for all x>=x0 and W in wheel family; then |H_W(x)| <= sqrt((1+eta_+(x;W))E2(x;W)/x).

## Current Constants

- `A_eta = 4.0`
- `C_eta_budget = 66.76254525892178`
- `A_H_empirical = 1.2`
- `C_H_budget = 19.741654586280088`
- `eps_sign = 0.0038168084184695417`
- `neg_over_abs_cap = 0.0019084042092348122`

## Exponent Gap Analysis

- `A_E2 needed for target A_H=2.0: 0.0`
- `A_E2 needed for empirical A_H=1.2: -1.6`
- Under naive product-exponent propagation, achieving low A_H requires strong control on E2/x exponent. This indicates proof must exploit structured cancellation/sign control beyond loose worst-case mixing.

## Forced O3 Next Obligations

- Prove explicit sign-sensitive offdiag inequality using quantified eps_sign and neg_over_abs_cap assumptions.
- Derive asymptotic E2/x bound with explicit exponent A_E2 and constants.
- Show combined chain yields an A_H class compatible with O5 endpoint target.
