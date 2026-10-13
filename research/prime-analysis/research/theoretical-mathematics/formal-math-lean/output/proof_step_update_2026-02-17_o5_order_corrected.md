# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:45:19.954147+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 18 |
| O4 | open | 1 |
| O5 | open | 18 |

## Latest Event

- obligation: `O5`
- summary: Refreshed O5 blocker order from corrected contract so implication path starts at U-OFFDIAG.
- removed assumption: Removed stale O5 ordering that still listed U-REM as open.
- remaining work: Resolve U-OFFDIAG, U-DIAG, U-UNIFORMITY, then discharge O1 and O4 for final implication closure.
- evidence:
  - `research/output/o5_integrated_implication_draft_2026-02-17.json`
  - `research/output/o3_unconditionalization_contract_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
