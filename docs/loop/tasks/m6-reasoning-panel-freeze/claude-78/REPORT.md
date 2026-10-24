# REPORT: m6-reasoning-panel-freeze, claude-78

Lab: claude. Order: [ORDER.md](ORDER.md).

closes #2866

## What did not work, first

- The freeze waited from 24 Oct 13:04Z on `loop claim`. It refused the item because the open #3152 claimed `docs/reference/storage.md`, which this item's scope also lists. pr-check does not read claims, and this unit changes no file #3152 touches, so it was done by hand. Claude runs solo (owner, 24 Oct).
- The builder subagent could not write this REPORT: a hook refuses REPORT files from subagents. The lab's top session wrote it.
- DeepSeek's patch left a stale doc comment on the registry test, saying the panel stays `"frozen": false`. It is corrected here.

## What changed

- DeepSeek's prepared patch (`local/deepseek/m6-freeze.patch`), read in full and applied unchanged. It sets `frozen: true` with the grader, noise and bars in `docs/harness/panels/m6-reasoning.json`, the matching test and the lab README.
- The seal is unchanged: 240 rows, the sealed digest `a52f3b03…` (206,197 B) and the unscored digest `5cf15b8c…` (79,625 B) appear identically before and after.
- The FIT addendum's in-scope fixes (#2034 comment 6384785241):
  - The source comment in `panel_m6_reasoning.rs` now states the measured noise: numeric-only scores 1..=3; context-table scores 10..=12, half-band ±1 row; modal ceiling 10/240. It now says targets may repeat, since the overlap check covers row ids and prompts only.
  - The bar notes label the overall `current` 12 as the context-table baseline, and the per-family values (0 + 10 + 0) as family-modal baselines. No threshold or row changed.
- The registry test's doc comment now says the panel is frozen.
- `docs/roadmap/items/m6-reasoning-panel-freeze.md`: `status: done` (a status-only edit), with `docs/roadmap/README.md` re-rendered.

Size: code 1 file, tests 1 file, docs 4 files (line counts in pr-check's size line at the top of the PR body).

## Measurements

Nothing new was measured. The noise numbers are the ones already in the test and the JSON `noise` block. The test's numeric-only scores are [3,3,1,2,1,1,1,1] and its context-table scores [11,11,10,12,10,10,11,11]. They agree with each other.

## Checks run at the head

- `cargo test -q --locked -p uor-r4-harness --test panel_m6_reasoning`, clippy with `-D warnings`, fmt, `loop report --check`, `loop pr-check --local`, `git diff --check`, the roadmap check and the quick gates: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- The item's "What is missing" text still describes the panel as not started. A status-only edit cannot change it; the next record unit can.
- The FIT's deferred conditions: re-check external-corpus overlap when an M6 training corpus is bound, and an optional compile-and-run guard for the code tasks. Any later change to a row or oracle needs a fresh FIT.
- M6 qualification itself (#2865) still needs a served M4 model.

## Plain summary

M6 is the milestone for reasoning and coding. Its test is a fixed set of 240 questions with answers checked by exact rules, not by another model. DeepSeek built and sealed the set, a reviewer that did not build it judged it fit, and its pass marks were registered. This change freezes the set, so it can no longer be edited without a new review, and marks the roadmap item done. No model has been tested on it yet.
