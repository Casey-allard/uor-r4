# Address-structured cells pre-flight: staged arm configs — 2026-10-19

Five checkpoint-0 arm configs for `docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`, with SHA-256 in `evidence/SHA256SUMS`.

All five derive from the injective treatment config `docs/labs/injective-attention-runbook-2026-10-19/evidence/injective-relation-seed1.json`:
- Relation{8,1} LearnV1, objective A, ActiveLeafV1, interleaved, `rows_v1`, seed 1;
- Dose 1 pins;
- canonical pod paths under `/workspace/uor-r4/local/driver/data/`.

Each changes **only** the table pin and the writer `kept_cells`. This is checked with `jq` against the source config.

| Config | Table file | Writer K |
|---|---|---:|
| `p8.json` | PrimeZeta `83c1380a…` (`injective/`) | 8 |
| `a8.json` | `address_structured_v1` `6600bdb1…` (`address-cells/`) | 8 |
| `s8.json` | `address_shuffled_v1_seed_20261019` `a185bdca…` (`address-cells/`) | 8 |
| `p16.json` | PrimeZeta `83c1380a…` | 16 |
| `a16.json` | `address_structured_v1` `6600bdb1…` | 16 |

On the pod, the A8 and S8 tables are rebuilt with `address-cells-codebook CONFIG OUT structured` and `… shuffled:20261019` from the merged source, and must reproduce these file SHAs before any arm runs. Nothing has run; these are staged inputs only.
