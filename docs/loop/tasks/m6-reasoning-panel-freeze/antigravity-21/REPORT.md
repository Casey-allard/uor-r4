# REPORT: m6-reasoning-panel-freeze, antigravity-21

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: [m6-reasoning-panel-freeze](../../../../roadmap/items/m6-reasoning-panel-freeze.md).

## What did not work, first

- **Claude's fresh-context review of `antigravity-20` (`a94ae87fa`) identified six required defects and five low-severity defects (`CHANGES` on PR `#2977`).**
  1. *Feasibility wording leaked abstention (`20 / 20`):* Every `"Altered feasibility constraint"` row (`idx 60..80`) was unsatisfiable, so a surface rule `"prompt.contains(\"feasibility\") -> ABSTAIN:UNSATISFIABLE"` scored `20 / 20`. We balanced the 20 feasibility rows into `10` satisfiable rows (unique integer solution in base 10) and `10` unsatisfiable rows (`ABSTAIN:UNSATISFIABLE`) using identical prompt wording across both sub-templates (`unsigned register R0` and `even-lattice closure`), so the keyword rule now scores only `10 / 20` (`0 / 10` on satisfiable feasibility rows, verified by `feasibility_wording_cannot_leak_abstention`).
  2. *Premature `status: done` and `"frozen": true`:* Step 5 of [`m6-reasoning-panel-freeze.md`](../../../../roadmap/items/m6-reasoning-panel-freeze.md) freezes the panel only after the owner has registered the numeric bar. Because the proposed bars on [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) await owner confirmation, we reverted `m6-reasoning-panel-freeze.md` to `status: ready`, re-rendered [`docs/roadmap/README.md`](../../../../roadmap/README.md), and set `"frozen": false` in [`docs/harness/panels/m6-reasoning.json`](../../../../harness/panels/m6-reasoning.json).
  3. *Self-certified `TEST FITNESS: FIT` in `README.md`:* Removed the author-written `TEST FITNESS: FIT` line in [`docs/labs/m6-reasoning-panel-freeze/README.md`](../../../../labs/m6-reasoning-panel-freeze/README.md) and set `TEST FITNESS: PENDING` awaiting non-author review on PR `#2977` and owner bar confirmation on `#2028`.
  4. *Unbacked external training-corpus overlap claim and `32` vs `30` triple count:* Fixed the triple count to `30` (`FORBIDDEN_CONTROL_TRIPLES.len() == 30`), added an integration check verifying that no other `docs/harness/panels/*.json` file contains `[M6-PANEL-V1::`, and stated explicitly that full external training-corpus byte scanning is deferred until an M6 training corpus is bound at qualification time.
  5. *`M6CountTableBaseline` peeked at `M6RowSpec` and used a 50% subsample:* Refactored `M6CountTableBaseline::context_key`, `observe`, `predict_context_table`, and `predict_family_modal` to accept `&M6UnscoredPromptRow` (which carries only `row_id`, `family`, `seed_index`, and `prompt`, with zero access to `M6RowSpec` or feasibility polarity), removed the 50% subsampling, and re-measured across `8` full 240-row seeded training draws (`context_table_scores = [11, 11, 10, 12, 10, 10, 11, 11]`, noise half-band `+-1 rows`).
  6. *Dead `ModularStepCap` branch (`idx 40..60`):* Previously `forbidden_rem` was always `(a + b) mod m`, so the `== forbidden_rem` branch was taken `20 / 20` times and `habit = a + b + c >= 65` could never collide with `expected < 53`. We balanced `ModularStepCap` into `10` branch-taken rows (`(a + b) mod m == forbidden_rem`) and `10` branch-not-taken rows (`(a + b) mod m != forbidden_rem`) with both `expected` and `habit` in `0..modulus`, verified by `modular_step_cap_balances_taken_and_not_taken_branches`.
  7. *All five LOW findings:* Enforced strict canonical integer parsing (`parse_canonical_i64`, rejecting `+5`, `007`, duplicate keys, and extra fields) in `Compositions` and `CodeTasks`; updated `VersionedMemoryMulSub` so `R0@initial = a + b` (`>= 2` carries, never equal to a prompt literal) and `decimal_borrows_sub` is computed on `(R0@previous * scale) - c`; expanded `NonDecimalRadix` to all 7 radices `{7, 8, 9, 11, 12, 13, 16}`; filtered out `CodeTasks` rows where any expected output integer appears in the prompt; and added 3 `--lib panel_m6` unit tests.

## What changed

- [`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs):
  - Balanced `AlteredConstraintSpec::FeasibilityConstraint` (`10` satisfiable / `10` unsatisfiable with identical prompt wording) and `AlteredConstraintSpec::ModularStepCap` (`10` branch-taken / `10` branch-not-taken with modular habits).
  - Updated `VersionedMemoryMulSub` (`R0@initial = a + b`, `decimal_borrows_sub` on `scaled - c`), `NonDecimalRadix` (`7` radices `{7, 8, 9, 11, 12, 13, 16}`), `CodeTasks` output-leakage filter, and `verify_zero_overlap()`.
  - Enforced strict canonical integer parsing (`parse_canonical_i64`) in `grade_composition_row` and `grade_code_task_row`.
  - Keyed `M6CountTableBaseline` strictly on `&M6UnscoredPromptRow` surface text and evaluated 8 full 240-row seeded draws without subsampling; added 3 unit tests (`panel_m6_*`).
- [`crates/uor-r4-harness/tests/panel_m6_reasoning.rs`](../../../../../crates/uor-r4-harness/tests/panel_m6_reasoning.rs):
  - Expanded the integration test suite to 7 tests (`panel_seed_determinism_and_sha256_seal`, `panel_overlap_exclusion_and_corpus_disjointness`, `family_oracles_accept_correct_and_reject_wrong_answers`, `feasibility_wording_cannot_leak_abstention`, `modular_step_cap_balances_taken_and_not_taken_branches`, `lawful_count_table_baseline_and_noise_measurement`, and `m6_reasoning_json_registry_is_staged_and_scorable`).
- [`docs/harness/panels/m6-reasoning.json`](../../../../harness/panels/m6-reasoning.json), [`docs/reference/storage.md`](../../../../reference/storage.md), [`docs/labs/m6-reasoning-panel-freeze/README.md`](../../../../labs/m6-reasoning-panel-freeze/README.md), [`docs/roadmap/items/m6-reasoning-panel-freeze.md`](../../../../roadmap/items/m6-reasoning-panel-freeze.md), and [`docs/roadmap/README.md`](../../../../roadmap/README.md):
  - Set `"frozen": false` and `status: ready`, updated the SHA-256 content seals (`5cf15b8c…` and `a52f3b03…`) and `+-1 rows` prompt-only count-table noise band, and set `TEST FITNESS: PENDING`.

## Measurements

- **Deterministic seed & content seal (`M6_PANEL_FROZEN_SEED_V1 = 0x004D_365F_5245_4153`):**
  - Unscored evaluation prompt JSONL (`to_canonical_jsonl_bytes(false)`): `240` rows, `79,625` bytes, SHA-256 `5cf15b8c97c7d6528e5dbff43f0b21df6de6b98e8845228bc10b7ea4439e7dc7`.
  - Sealed oracle specification JSONL (`to_canonical_jsonl_bytes(true)`): `240` rows, `206,197` bytes, SHA-256 `a52f3b0333a2832e1e3371c09954703cf3fa6b4364c22f9d4f3606a0f3600f0f`.
- **Oracle positive, negative, keyword-leakage, and branch-balance controls:**
  - Positive controls: `240 / 240` (`80 / 80` `Compositions`, `70 / 70` satisfiable `AlteredConstraints`, `10 / 10` abstention `AlteredConstraints`, `80 / 80` `CodeTasks`).
  - Negative controls: `0 / 240` (`60 / 60` `HabitCaptureError`, `10 / 10` `FalseAbstentionOnSatisfiableRow`, `10 / 10` `FailedToAbstainOnUnsatisfiableRow`).
  - Keyword-abstention control (`prompt.contains("feasibility") -> ABSTAIN:UNSATISFIABLE`): `10 / 240` overall (`10 / 20` on feasibility rows, `0 / 10` on satisfiable feasibility rows).
  - `ModularStepCap` branch controls: always-add scores `10 / 20` (`10` `HabitCaptureError`), never-add scores `10 / 20` (`10` `HabitCaptureError`).
- **Prompt-only `M6CountTableBaseline` across `8` full 240-row seeded draws:**
  - `numeric_only_scores`: `[3, 3, 1, 2, 1, 1, 1, 1]` (`1..=3 / 240`).
  - `context_table_scores`: `[11, 11, 10, 12, 10, 10, 11, 11]` (min `10 / 240`, max `12 / 240`, half-band **`+-1 rows`**).
  - `family_modal_ceiling`: `10 / 240` (`4.17%`).

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib panel_m6`: `3 passed; 0 failed`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test panel_m6_reasoning`: `7 passed; 0 failed`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo clippy -p uor-r4-harness --locked --all-targets -- -D warnings`: `0 warnings`.
- `uor-harness loop gates --full`: `8/8 PASS`.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`): verified deterministic D28 oracle construction, abstention-leakage elimination, and prompt-only count-table noise calibration.
- `solution-integrity` and `scope-guard`: kept all repairs strictly within [`m6-reasoning-panel-freeze.md`](../../../../roadmap/items/m6-reasoning-panel-freeze.md) `scope:`.

## Still open

- Owner confirmation of the proposed M6 numeric bars (`overall-pass-240 >= 168/240`, `compositions-80 >= 56/80`, `altered-constraints-80 >= 56/80`, `code-tasks-80 >= 56/80` across `>= 2` seeds) on [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) and non-author `TEST FITNESS: FIT` review before flipping `"frozen": true` and `status: done`.

## Plain summary

Repaired all six required findings and all five low-severity findings from Claude's `CHANGES` review of PR `#2977`: balanced the 20 `"Altered feasibility constraint"` rows (`10` satisfiable / `10` unsatisfiable with identical wording so keyword abstention scores only `10 / 20`), balanced `ModularStepCap` (`10` branch-taken / `10` branch-not-taken), keyed `M6CountTableBaseline` strictly on unscored `M6UnscoredPromptRow` surface features (`10..=12 / 240`, `+-1 rows` across 8 full seeded draws), enforced canonical integer parsing in `Compositions` and `CodeTasks`, removed self-certified `FIT`, and reverted `"frozen": false` and `status: ready` until owner bar registration and non-author FIT review. References #2866, References #2034.
