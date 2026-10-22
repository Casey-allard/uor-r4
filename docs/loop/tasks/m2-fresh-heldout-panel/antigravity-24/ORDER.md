# ORDER: m2-fresh-heldout-panel, antigravity-24

Lab: antigravity. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `f703cf790`.

## Goal

> A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

Repair PR `#2985` findings from review `6349065628` (HIGH bar metric alignment with owner registration `6350135678` on `#2030`, MEDIUM real `ByteBpeTokenizer` and grader verification + saved replay-audit labeling, and LOW cleanup).

## Cancel probe (run first)

At `f703cf790`, `uor-harness loop next` reports `GO repair #2985 — a counted CHANGES at its head stands (claude)` (`6349065628`). Owner (Casey) approved registering `pairs_both_complete_distinct >= 52/128` (`40.625%` of the 128 opposite-role history pairs, with single-row `complete >= 104/256`) on `#2030` (`6350135678`). Result: task is live. Class: instrument.

## Calls

- Update `manifest.json`, `docs/harness/panels/m2-fresh256.json`, `docs/labs/m2-fresh-heldout-panel/README.md`, `docs/mechanisms/native-geometric.md`, and `docs/reference/storage.md` to name `pairs_both_complete_distinct >= 52/128` (`complete >= 104/256`) as the registered M2 fresh-panel bar metric per `#2030` (`6350135678`).
- Link `uor-r4-tokenizer` and `uor-r4-core` in `docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml` to verify all panel strings against the real `ByteBpeTokenizer` (`archive/evidence/2026-09-29-16m-discovery/tokenizer.json`) and grade all 4 controls with the real `FrozenAnswers` + `ByteBpeTokenizer` + `pair_id` grader logic from `native-bank-generalization.rs` (`pairs_both_complete_distinct == 0` for each control).
- Compute `complete_rows = complete_ids.len()` from `reviewer-rank177.json`, label `saved_audit_replay_diff_rows: 0` as copied from the saved 2026-10-10 replay audit (`development-replay-audit-summary.json`), and remove the redundant `!= 0` check in `lib.rs`.

## Files in scope

- `docs/harness/panels/**`, `docs/mechanisms/native-geometric.md`, `docs/labs/m2-fresh-heldout-panel/**`, `docs/reference/storage.md`, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-24/**`

## Done when

- Instrument unit tests, `contract::tests::every_format_passes_on_the_real_state`, `xtask links`, `loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Any out-of-scope file, owner path, or GPU compute is needed.
