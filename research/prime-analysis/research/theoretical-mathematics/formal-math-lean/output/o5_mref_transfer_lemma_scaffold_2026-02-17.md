# O5 Mref Transfer Lemma Scaffold

Generated: 2026-02-17T19:06:03.600956+00:00

## Quantified Statement

There exist x0>=3 and constants C0_ref, a_ref, b_ref, C_H, A_H such that for all x>=x0 and W in {30,210,2310,30030}, |E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref| C_H (log x)^A_H.

## Fixed Regime

- `m_ref = 512`
- `Delta_M = 0` at `M=M_ref`.

## Current Constants

- `a_ref = -0.0013474693715061251`
- `b_ref = -0.05436122353654979`
- `C0_ref = 0.6334997766223893`
- `C_H = 3.47471643488075`
- `A_H = 1.2`

## RH Endpoint Class Check

- target: `|psi(x)-x| <= C sqrt(x) (log x)^2.0`
- meets exponent class now: `True`
- exponent gap `A_H-target`: `-0.8`

## Dependency Discharge Map

| dependency | status | required |
|---|---|---|
| A1_ref (O1) | grid_validated_only | Asymptotic proof of reference residual bound at M_ref. |
| A3_ref (O3) | grid_validated_only | Asymptotic bridge-growth theorem at M_ref with explicit constants. |
| Base uniformity (O4) | grid_validated_only | Uniform-in-W theorem constants for all x>=x0. |

## Grid Check (Mref Transfer RHS)

- holds on grid: `True`
- ratio max: `0.7900031984345878`
- gap min: `0.165034640080594`

## Forced Next Steps

- Convert A1 reference residual from sampled-sup constant to asymptotic theorem.
- Convert A3 m_ref bridge envelope from calibrated holdout fit to asymptotic theorem.
- Write final O5 implication theorem that cites a standard RH-equivalent endpoint criterion.
