# O3 E2 Remainder Symbolic Draft

Generated: 2026-02-17T19:28:00.483834+00:00

## Lemma Target

- `E2-REM`
- There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.

## Proposed Remainder Inequality

Rem(x;W) <= C_rem * (log x)^A_rem with C_rem assembled from truncation/tail majorants.

## Candidate Constants From O2

- `C_delta_uplifted = 0.0013231345976502436`
- `beta_fixed = 2.6`

Remainder control should inherit explicit tail bounds from O2 and be uniform in W.

## Remaining To Formalize

- Define remainder term in E2 decomposition with exact dependency on truncation level.
- Map O2 tail majorant into E2 remainder bound without losing uniformity in W.
- Extract explicit C_rem and A_rem for E2 assembly lemma.
