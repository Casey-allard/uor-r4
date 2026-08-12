# O3 E2 Diagonal Symbolic Draft

Generated: 2026-02-17T19:26:47.265522+00:00

## Lemma Target

- `E2-DIAG`
- There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.

## Proposed Diagonal Inequality

Diag(x;W) <= C_diag * (log x)^A_diag for all x>=x0 and W in wheel family.

## Compatibility Conditions

- `A_diag_upper_target = 0.0`
- `C_diag_target_scale_hint = 5.751450514658472`

Diagonal bound should be established with explicit channel smoothness and wheel-density terms, kept separate from offdiag cancellation machinery.

## Remaining To Formalize

- Write explicit formula for diagonal summand and its wheel-filtered normalization.
- Prove asymptotic majorant for diagonal term uniformly in W.
- Extract explicit C_diag, A_diag constants compatible with E2 target pack.
