# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:47:49.440337+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 20 |
| O4 | open | 1 |
| O5 | open | 20 |

## Latest Event

- obligation: `O5`
- summary: Refreshed O5 blocker order after U-DIAG closure; contract now leaves U-UNIFORMITY as the remaining O3-gated blocker.
- removed assumption: Removed stale O5 ordering that included U-DIAG as open.
- remaining work: Close U-UNIFORMITY, then discharge O1 and O4 for final implication closure.
- evidence:
  - `research/output/o5_integrated_implication_draft_2026-02-17.json`
  - `research/output/o3_unconditionalization_contract_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
