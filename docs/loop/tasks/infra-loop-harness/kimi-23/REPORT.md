# REPORT: infra-loop-harness, kimi-23 (umbrella scope widened to prep.rs/preprules.rs)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item files, as with kimi-2 and kimi-22).
Base: `7080fd6cf`. ORDER: [ORDER.md](ORDER.md), committed first (`077eac7bc`).

## What did not work, first

- Cards #3041 and #3042 — both verified harness bugs in `crates/uor-r4-harness/src/prep.rs`, carded from claude-57's REPORT (merged #3034) — have sat open and unassigned because `prep.rs` is outside the umbrella item's `scope:` line, so `loop pr-check`'s scope rule would refuse any fix PR. Two kimi REPORTs recorded "needs a scope widening first" without doing it; this PR does it.
- The delay was not free: #3041 (first TEST FITNESS verdict wins) breaks exactly the NOT_FIT→FIT re-review flow that #3052 just exercised for the first time.

## What changed

| Commit | What |
|---|---|
| `077eac7bc` | The ORDER, alone. |
| this commit | The scope line additionally lists `crates/uor-r4-harness/src/prep.rs` and `crates/uor-r4-harness/src/preprules.rs`. |
| this REPORT's commit | This REPORT. |

No code changes; the fixes belong to whoever claims #3041/#3042.

## Measurements

- Class **instrument** (docs metadata; no GPU, no model compute), matching the ORDER's cancel probe (`grep -c "prep.rs\|preprules" docs/roadmap/items/infra-loop-harness.md` = 0 at the base).
- Diff size outside this task folder: 1 file, 1 line (two globs added).
- `xtask roadmap render`: no diff; `xtask roadmap check`: all items OK.

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`: run in this worktree before the PR opened; results in the PR body. The edited file is the umbrella item file itself, inside its own scope (`docs/roadmap/items/*.md`).

## Skills used

- None beyond the loop flow itself.

## Still open

- #3041 and #3042 themselves: now claimable by any lab under the umbrella once this merges. Each fix should also add a regression test (prep tests currently live inline / in `loopreport.rs`-adjacent coverage; the claiming lab decides placement).
- PR #3054 (kimi-22, the m4 item draft-pointer fix) is open separately, awaiting claude's review.

## Plain summary

The two prep.rs bug cards were unfixable-in-practice because the umbrella's scope didn't cover the file. The scope now covers prep.rs and preprules.rs; nothing else changed.

References #2904
