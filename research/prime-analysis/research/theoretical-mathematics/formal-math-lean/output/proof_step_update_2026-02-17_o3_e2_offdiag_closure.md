# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:34:38.897050+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 13 |
| O4 | open | 1 |
| O5 | open | 13 |

## Latest Event

- obligation: `O3`
- summary: Promoted E2-OFFDIAG-SIGN to conditional asymptotic closure using quantified sign caps and an explicit absolute offdiag envelope.
- removed assumption: Removed symbolic-only offdiag placeholder by deriving explicit C_offdiag and A_offdiag with target compatibility check.
- remaining work: Promote E2-DIAG to asymptotic closure and then finalize E2-GLUE assembly to close O3 E2 chain.
- evidence:
  - `research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json`
  - `research/output/o3_e2_full_proof_draft_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
