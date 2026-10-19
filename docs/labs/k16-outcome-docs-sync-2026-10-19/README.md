# K16 outcome synced into README and lab entry — 2026-10-19

The registered K16 outcome call (#2723, `5dec1b4e8`) was already in NOW/STATUS/current-state, but README still
called the interpretation the "immediate priority" and its M1 table row said "registered scientific
interpretation pending"; docs/labs README still listed it as pending. This docs-only correction records the
completed outcome (`OCCUPANCY_COLLAPSE_TRIGGERED`, no KEEP; admission arm A; fresh NOT_TRIGGERED, seal kept)
and keeps normal close/final accounting explicitly pending.

Independent evidence verification: `evidence/k16-outcome-fields.jsonl` SHA-256 `ea6f9048…` matches; every
`target_admitted_rows` sum identity and every `n = admitted_token_count_sum / 14,287` value replays; the
G-Occ `2 × n_A` thresholds are 703.6/752.2 as corrected. Host-only; no model/scorer execution.

Docs-only; claim-wording PASS with the existing checker; no build, no compute, $0.