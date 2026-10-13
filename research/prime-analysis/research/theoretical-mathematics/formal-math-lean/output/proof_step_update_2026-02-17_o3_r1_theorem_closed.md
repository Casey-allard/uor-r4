# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T21:21:11.407331+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 4 |
| O2 | open | 5 |
| O3 | open | 26 |
| O4 | open | 2 |
| O5 | open | 25 |

## Latest Event

- obligation: `O3`
- summary: Closed strict requirement O3-R1 at theorem level via L-OFFABS theorem artifact; strict table now records theorem_closed_total increment.
- removed assumption: Removed theorem-open status for O3-R1 by replacing draft-only state with theorem_closed artifact and explicit constants.
- remaining work: Close O3-R2, O3-R3, and O3-R4 theorem requirements via direct lemmas L-OFFSIGN, L-DIAG, and L-ASM.
- evidence:
  - `research/output/o3_l_offabs_theorem_closure_2026-02-17.json`
  - `research/output/proof_requirement_closure_table_2026-02-17.json`
  - `research/output/proof_canonical_manifest.json`
