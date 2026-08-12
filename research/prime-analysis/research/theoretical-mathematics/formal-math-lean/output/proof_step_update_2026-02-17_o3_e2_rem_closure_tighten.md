# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:31:23.522391+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 11 |
| O4 | open | 1 |
| O5 | open | 13 |

## Latest Event

- obligation: `O3`
- summary: Tightened E2-REM asymptotic closure by increasing log-schedule slope to lambda=2.0, reducing C_rem_uniform while preserving O2 assumption scope.
- removed assumption: Removed overly weak truncation schedule calibration that inflated uniform remainder constant.
- remaining work: Complete theorem-facing asymptotic closures for E2-DIAG and E2-OFFDIAG-SIGN, then finalize E2-GLUE.
- evidence:
  - `research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json`
  - `research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.md`
  - `research/output/proof_canonical_manifest.json`
