# O1 A1_ref Asymptotic Scaffold

Generated: 2026-02-17T19:07:30.661254+00:00

## Quantified Statement

There exist x0>=3 and C0_ref>=0 such that for all x>=x0 and W in {30,210,2310,30030}, |E(x)/sqrt(x) - (a_ref H_W^(M_ref)(x) + b_ref)| <= C0_ref.

## Decomposition Plan

- Residual <= smoothing_term + link_term.
- Bound each term uniformly in W and x>=x0.
- Set C0_ref = C_smooth + C_link.

## Current Constant Candidates

- `C_smooth_uplifted = 0.13799625454896874`
- `C_link_uplifted = 0.6205773860913274`
- `C0_uplifted_sum = 0.9102883687683553`
- `a_ref = -0.0013474693715061251`
- `b_ref = -0.05436122353654979`
- `m_ref = 512`

## Grid Validation Snapshot

- valid holds: `True`
- valid ratio max: `0.8528446006626982`
- valid max gap (lhs-rhs): `-0.13395384841820834`

## Remaining O1 Blockers

- Replace finite-window sup calibration by asymptotic smoothing bound valid for all x>=x0.
- Prove link-term bound without sampled-grid dependence.
- Prove wheel-family uniformity of C0_ref constants for all W in {30,210,2310,30030}.
