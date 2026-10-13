# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:39:55.468014+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 16 |
| O4 | open | 1 |
| O5 | open | 14 |

## Latest Event

- obligation: `O3`
- summary: Derived O3 bridge constants directly from theorem-side eta budget and closed E2 assembly, replacing placeholder A_H/C_H in the bridge chain.
- removed assumption: Removed dependence on empirical scaffold H constants for O3 bridge step by introducing identity-based derived A_H/C_H from eta and E2 closures.
- remaining work: Unconditionalize the conditional DIAG/OFFDIAG/REM envelopes so derived O3 bridge constants become unconditional theorem constants.
- evidence:
  - `research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json`
  - `research/output/o3_e2_full_proof_draft_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
