# ORDER: m6-reasoning-panel-freeze, antigravity-21

Lab: antigravity. Item: [m6-reasoning-panel-freeze](../../../../roadmap/items/m6-reasoning-panel-freeze.md). Base: `a94ae87fa4a28b619e759e08f66a14f42155ecb5`.

## Goal

Repair the six required findings and five low-severity findings from Claude's fresh-context `CHANGES` review of PR `#2977` at `a94ae87fa4a28b619e759e08f66a14f42155ecb5`:

1. **Eliminate `"feasibility"` wording leakage (`AlteredConstraints` idx `60..80`):** Balance the 20 `"Altered feasibility constraint"` rows into 10 satisfiable rows (unique base-10 integer `x` in `[L, R]` satisfying `(x + offset) mod m == target_rem` and `d | x` with `x != default_habit_answer`) and 10 unsatisfiable rows (`ABSTAIN:UNSATISFIABLE`), using identical prompt wording so `"contains 'feasibility' -> ABSTAIN:UNSATISFIABLE"` scores only `10/20` on the sub-family (`0/10` on satisfiable feasibility rows).
2. **Keep item `status: ready` and panel `"frozen": false` until owner bar registration and non-author FIT:** Revert `docs/roadmap/items/m6-reasoning-panel-freeze.md` to `status: ready`, re-render `docs/roadmap/README.md`, set `"frozen": false` in `docs/harness/panels/m6-reasoning.json`, and reference `#2866` (`References #2866`) instead of closing it prematurely.
3. **Remove self-certified `TEST FITNESS: FIT` from `README.md`:** State `TEST FITNESS: PENDING (awaiting non-author review on PR #2977 and owner bar registration on #2028)`.
4. **Align overlap-exclusion claims and counts:** State exact verified checks (`18` control pairs and `30` control triples from `m6_exact_arithmetic.rs`, within-panel uniqueness, cross-seed separation, and `[M6-PANEL-V1::...]` tag scan across repository fixtures) and drop unbacked external training-corpus claims until a training corpus is bound.
5. **Make `M6CountTableBaseline` key strictly on unscored `M6PromptRow` surface text:** Change `prompt_bucket_key`, `observe_row`, and `predict` to accept `&M6PromptRow` rather than `&M6PanelRow` so it cannot peek at `M6RowSpec` or `AlteredSubkind`, and re-measure the 8-seed count-table noise band.
6. **Fix the dead `ModularStepCap` branch (`AlteredConstraints` idx `40..60`):** Draw `forbidden_rem` so 10 rows take the `(a + b) mod m == forbidden_rem` branch and 10 rows take the `(a + b) mod m != forbidden_rem` branch, with both branches differing from the default habit `a + b + c`.
7. **Address all LOW findings:** Enforce canonical integer formatting (`val.to_string() == val_str`, no `+5`, `007`, duplicate keys, or junk tokens) in `Compositions` and `CodeTasks` grading; make `VersionTarget::Initial` combine `R0@initial + R1@initial` (requiring `>= 2` carries and never matching a literal in the prompt); compute `decimal_borrows_sub` on `(a + b) * scale - c`; use all 7 non-decimal radices `[3, 5, 7, 8, 9, 12, 16]`; and add `--lib panel_m6` unit tests alongside the integration test suite.

## Cancel probe (run first)

Ran `uor-harness loop review --check --pr 2977`:
- Confirmed `comment 1: [claude] CHANGES at a94ae87fa4a28b619e759e08f66a14f42155ecb5 on PR 2977: counted`.

## Files in scope

- `crates/uor-r4-harness/src/panel*.rs`
- `crates/uor-r4-harness/tests/panel*.rs`
- `docs/harness/panels/**`
- `docs/reference/storage.md`
- `docs/labs/m6-reasoning-panel-freeze/**`
- `docs/roadmap/items/m6-reasoning-panel-freeze.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/m6-reasoning-panel-freeze/antigravity-21/**`

## Done when

- `host:` `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test panel_m6_reasoning` (`7/7` tests) and `--lib panel_m6` (`3/3` tests) pass, `uor-harness loop gates --full` is green, `m6-reasoning.json` has `"frozen": false` with updated SHA-256 seals and prompt-only count-table noise band, and `m6-reasoning-panel-freeze.md` remains `status: ready`.

## Stop and ask if

- Any file outside the declared scope must change.
- Any GPU execution is needed.
