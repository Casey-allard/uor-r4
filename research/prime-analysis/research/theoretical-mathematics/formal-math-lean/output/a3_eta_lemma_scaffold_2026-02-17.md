# A3 Eta Lemma Scaffold

Generated: 2026-02-17T15:35:58.550116+00:00

## Candidate Lemma Statement

There exist constants `C_eta, x0` such that for all `x >= x0` and all tested bases `W`:

`eta_+(x;W) <= C_eta * (log x)^4`.

With this bound and deterministic identity

`|H(x;W)| <= sqrt((1 + eta_+(x;W)) * E2(x;W) / x)`,

derive

`|H(x;W)| <= C_H * (log x)^{A_H}`.

## Current Calibrated Constants

- Base branch (`research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json`): `A_eta=4`, `C_eta=1.21701344784`, `A_H=1.2`, `C_H=2.60722567538`
- Stress branch (`research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json`): `A_H=1.1`, `C_H=4.52674835019`
- Fixed-`A_eta` probe suggests minimum safety near `2.69862` for held-out validity (`research/output/a3_eta_exponent_probe_2026-02-17.json`).
- `A_eta=4` justification run has held-out ratio `0.693214` under safety 3.0 (`research/output/a3_eta4_justification_probe_2026-02-17_sf3.json`).

## Proof Obligations Remaining

1. Replace empirical safety factor by symbolic constants from an explicit offdiag bound.
2. Prove base-uniform control over positive offdiag ratio for all `x >= x0`.
3. Bound `E2(x)/x` analytically in the same regime to avoid calibration dependence.
4. Convert finite-grid verification into asymptotic implication with explicit `x0` and constants.

## Practical Next Action

Draft a sign-sensitive analytic inequality chain for offdiag terms (not pure absolute bilinear expansion) and compare resulting symbolic constants against the calibrated `C_eta` budget above.

Current status update:
- absolute bilinear symbolic chain is now tested in
  `/Users/adminamn/Documents/New project/research/output/a3_offdiag_symbolic_chain_note.md`
  and is too loose for current theorem constants.
