# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:40:02.507914+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 3 |
| O2 | open | 4 |
| O3 | open | 16 |
| O4 | open | 1 |
| O5 | open | 15 |

## Latest Event

- obligation: `O5`
- summary: Switched O5 integrated implication to use O3 bridge constants derived from eta+E2 closure when available.
- removed assumption: Removed stale reliance on legacy O3 scaffold constants inside integrated implication draft.
- remaining work: Discharge O1 and O4, and convert O3 conditional component envelopes into unconditional theorem lemmas for final implication closure.
- evidence:
  - `research/output/o5_integrated_implication_draft_2026-02-17.json`
  - `research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
