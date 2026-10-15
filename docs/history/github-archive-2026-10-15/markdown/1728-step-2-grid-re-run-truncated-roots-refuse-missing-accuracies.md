# #1728 — Step 2 grid: re-run truncated roots; refuse missing accuracies (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:59:10Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-10-05T07:11:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1728
- **Merged:** 2026-10-05T07:11:52Z
- **Merge commit:** 7addd69a53a35c170681a1b4e0d154ad90267372
- **Base / Head:** main / UOR-Foundation:codex/step2-script-full-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Follow-up to #1720.

## Problem
`scripts/pod/step2-parity-bench.sh` treated any sealed root whose report said `"status": "complete"` as done and skipped it on rerun. A root that stopped at `MAX_SECONDS` is also `complete`, but `mode=decide` (#1720) drops it as not full-budget, so that seed stayed missing on every rerun.

## Change
**Script**
- `root_state` (replaces `complete`) reads the root's `report.json` with a short `python3` one-liner. A root counts as done only when status is complete, `stopped_early_at_max_seconds=false` and `steps_completed == STEPS`.
- A sealed, complete root that falls short of that budget is moved aside to `<root>.incomplete-<UTC stamp>`. It is never deleted, and the seed is re-run. The new attempt uses a fresh path: a path whose root was moved aside is not reused. Roots already moved aside are not scanned again.
- If a sealed `report.json` cannot be parsed or lacks the budget fields, the script reports it on stderr, the job exits 255 and xargs stops launching jobs. With `pipefail`, the script then exits non-zero.
- Failed (status not complete) and unsealed roots behave as before: the seed gets a new `-aN` attempt root.
- The script now checks up front that `python3` is installed.

**Cheap reviewer nits**
- `qk_jj` now has 3 seeds, both in the script default `ARMS` and in decide's `required_seeds` and the `rules.required_seeds` record. The default grid is now 63 runs.
- `decide.rs`: a new `seed_cells` function makes a cell without a finite numeric `full_accuracy` an error naming the root and the cell. Before, it became a NaN and dropped out of the seed minimum and the means. `rule_full_accuracy` gets the same check; the fact report always writes both fields.
- `decision.md` now has one more sentence: the conv arm is a width-4 causal depthwise conv over k_t..k_{t-3}, so it reaches lags 1-3, while F2 reaches lag 1 only.

## Checks at this head
- `cargo test --offline -p uor-r4-training --example mqar-bench`: 14 passed. This includes the new `decide::tests::a_missing_or_non_numeric_accuracy_is_refused_not_dropped`, which covers an absent, null or string `full_accuracy` and an absent `rule_full_accuracy`.
- `rustfmt --check` on the example passes. Workspace `cargo fmt --check` also reports an existing diff in `src/relation_compiler.rs` on main, which this PR does not touch.
- `bash -n` passes on the script.
- Shell harness: I extracted `root_state` and `run_job` and ran them with a fake binary. Results:
  - A seed truncated at max_seconds and one short of its steps were each moved aside and re-run (`-a2`).
  - A full-budget root was skipped.
  - A failed root got a new `-a2` attempt, and the next pass skipped everything.
  - A `report.json` without the budget fields stopped xargs with exit status 255.
- I did not run the real GPU grid.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5989797786 — 2026-10-05T07:10:00Z — Casey-allard

Adversarial review at 138f3e14e842b4e6bac39c564d5d0ae3fa5710bc:

**Verdict: APPROVE.** I found no blocking issues. The fix closes the stated items.

**What I checked at the exact head** (git-archive copy in `$TMPDIR`, shared `CARGO_TARGET_DIR=~/.cache/uor-claude-target`, `-j 2`, job lock registered and then removed):
- `cargo test --offline -j 2 -p uor-r4-training --example mqar-bench`: 14 passed, including `decide::tests::a_missing_or_non_numeric_accuracy_is_refused_not_dropped`.
- `rustfmt --check` on `decide.rs` passes. `bash -n` passes on the script.
- I ran the **whole script end to end**, not an extract, with a fake `BIN`. The only edit was the tokenizer sha line in a harness copy. It ran under macOS `/bin/bash` 3.2.
  - Pass 1:
    - The full-budget root was skipped.
    - A root with `stopped_early_at_max_seconds=true` and a root at 9 of 10 steps were each moved to `<root>.incomplete-<stamp>` and re-run as `-a2`.
    - The failed root and the unsealed root each got `-a2`.
    - decide ran.
  - Pass 2: all 5 seeds were skipped, and no `.incomplete-*` root was scanned.
  - A sealed root missing `stopped_early_at_max_seconds` produced a python KeyError and the job exited 255. xargs then printed "exited with status 255; aborting", the script exited 1, and **`mode=decide` did not run** (pipefail plus `set -e`).
- The script and `decide.rs` agree on full budget. Both require `stopped_early_at_max_seconds=false` and steps_completed equal to the planned steps. The script uses `$STEPS`; decide uses the report's own `task.common.steps`. The report writer (`fact.rs:1062-1063`) always emits both fields.
- qk_jj needs 3 seeds in all three places: `required_seeds()`, the `rules.required_seeds` JSON, and the script's default `ARMS`. The grid is 63 runs (5x3 + 3x2 = 21 per pattern, times 3 patterns).
- Name globbing cannot collide: `rararr-qk-s1-a*` does not match `qk_jj` roots.

**Non-blocking notes**
1. Moved-aside roots stay under `runs/`, and decide reads every directory there. Consider a calibration root that trained its *own* full budget at a different `STEPS`. The script calls it "truncated" and moves it aside, but decide would still count it. `training_fingerprint` keeps `steps`, so decide would refuse the grid with "different configuration", and someone would have to move that root out by hand. The failure is loud, not silent. The header claims a calibration root is "moved aside and re-run" when `STEPS` differs, so it should mention this case. Calibrating at the grid's `STEPS` avoids the problem.
2. When `steps_completed > STEPS`, the root is labelled "truncated". The handling is correct, but the label is wrong.
3. The pre-existing `relation_compiler.rs` fmt diff on main is outside this PR.

I did not run the real GPU grid.


### comment 5989811039 — 2026-10-05T07:11:00Z — Casey-allard

Lab-lead exact-head check at 138f3e14e842b4e6bac39c564d5d0ae3fa5710bc: the adversarial review approved with no blocking items. Truncated and capped runs are now refused, or moved aside and re-run, and decision rules can no longer overlap. Queuing.


---
_Comment/review ids in this file: 5989797786, 5989811039_

