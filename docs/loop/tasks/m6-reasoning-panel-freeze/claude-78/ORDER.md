# ORDER: m6-reasoning-panel-freeze, claude-78 (freeze the m6-reasoning panel; close #2866)

Lab: claude. Item: [m6-reasoning-panel-freeze](../../../../roadmap/items/m6-reasoning-panel-freeze.md) (#2866). Base: `20f85b858`.

## Goal

The 240-row m6-reasoning panel is staged with deterministic oracles (#2977). Its bar is registered (#2028 comment 6384497995), and a non-author `TEST FITNESS: FIT` is posted (#2866 comment 6384549808; addendum #2034 comment 6384785241). Only the freeze is left.

DeepSeek prepared it as `local/deepseek/m6-freeze.patch`. It sets `frozen`, `grader`, `noise` and `bars` in `docs/harness/panels/m6-reasoning.json`, with the matching test and README changes. The freeze waited only on `loop claim`'s overlap refusal, because #3152 holds `docs/reference/storage.md`. pr-check does not read claims, and Claude runs solo (owner, 24 Oct), so this unit freezes the panel by hand.

The unit:
- applies the patch, after reading it in full;
- makes the addendum's two in-scope fixes:
  - the stale noise numbers in `panel_m6_reasoning.rs:27-32`;
  - per-family baseline labels in the bar notes;
- sets the item to `done` (D70).

## Cancel probe (run first)

At `20f85b858`, is the panel already frozen? No: `grep -n '"frozen"' docs/harness/panels/m6-reasoning.json` reads `false`. The unit stands. Class: instrument.

## Files in scope

- `docs/harness/panels/m6-reasoning.json`, `crates/uor-r4-harness/src/panel_m6_reasoning.rs`, `crates/uor-r4-harness/tests/panel_m6_reasoning.rs`, `docs/labs/m6-reasoning-panel-freeze/README.md`
- `docs/roadmap/items/m6-reasoning-panel-freeze.md` (the `status:` line only)
- `docs/roadmap/README.md` (re-rendered)
- `docs/loop/tasks/m6-reasoning-panel-freeze/claude-78/**`

## Done when

- The panel JSON has `frozen: true` with grader, noise and bars set. The seal (row count 240, sealed and unscored digests) is unchanged.
- The source comment states the current noise numbers (1..=3, 10..=12 ±1, ceiling 10/240).
- The bar notes label each family's baseline.
- `cargo test -q --locked -p uor-r4-harness --test panel_m6_reasoning` passes.
- The item is `done` and the roadmap view is re-rendered.
- `loop report --check`, `loop pr-check --local`, `git diff --check`, clippy for the harness crate and the quick gates pass.
- The exact-head review is Claude's own fresh-context review (D66). The PR says `Closes #2866`.

## Stop and ask if

The patch no longer applies, the seal digests would change, or a file outside the list is needed.
