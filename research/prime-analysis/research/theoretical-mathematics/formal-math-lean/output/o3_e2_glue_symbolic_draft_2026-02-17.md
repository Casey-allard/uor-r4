# O3 E2 Glue Symbolic Draft

Generated: 2026-02-17T19:28:00.484028+00:00

## Lemma Target

- `E2-GLUE`
- There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.

## Assembly Rule

If Diag <= C_diag (log x)^A_diag, Offdiag <= C_off (log x)^A_off, Rem <= C_rem (log x)^A_rem, then E2/x <= C_E2 (log x)^A_E2 where A_E2=max(A_diag,A_off,A_rem), C_E2=C_diag+C_off+C_rem.

## Target Constraints

- `A_E2_max_required = 0.0`
- `C_E2_min_chain_compatibility = 5.751450514658472`
- Show A_E2 <= A_E2_max_required and C_E2 compatible with O3 chain.

## Remaining To Formalize

- Track exact exponents from E2-DIAG, E2-OFFDIAG-SIGN, E2-REM outputs.
- Establish common x0 where all three component inequalities hold simultaneously.
- Derive final C_E2 and verify compatibility with O3 target pack.
