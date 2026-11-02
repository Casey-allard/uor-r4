# REPORT: one-model-end-to-end, claude-260 (the item's scope, widened for the copy read's key)

Lab: claude. [ORDER](./ORDER.md). PR #3578, item #3407.

**In one paragraph.** The item's `scope:` now also lists the six files the copy read's two-token key changes: the float export, the snapshot, the artifact format, the integer read, and the integer fixtures and tests. Nothing else in the item changed, and the item stays open. No model was trained or served in this unit. The key itself is the next unit, claude-261.

## What did not work

1. **The first form of this ORDER held the whole key unit,** with the six files in its own "Files in scope". The loop's guard reads an ORDER's scope from the item file on `main`, so it refused the first edit to `crates/uor-r4-float-train/src/fbs_export.rs`. The float read's edits made before that refusal (`flock_read_hamming.rs`, `in_space_long.rs`, `bin/train-in-space.rs`) were saved as a patch, `~/uor-r4/local/claude/claude-261-float.patch` (275 lines, outside the repository), and taken out of this branch. They are claude-261's to finish, test and review.

## What changed

- `docs/roadmap/items/one-model-end-to-end.md`: `scope:` adds
  - `crates/uor-r4-float-train/src/fbs_export.rs`
  - `crates/uor-r4-integer/src/fbs_snapshot.rs`
  - `crates/uor-r4-integer/src/stack/format.rs`
  - `crates/uor-r4-integer/src/stack/hamming.rs`
  - `crates/uor-r4-integer/src/stack/synthetic.rs`
  - `crates/uor-r4-integer/src/stack/tests.rs`
- `cargo run -p xtask -- roadmap render` leaves `docs/roadmap/README.md` unchanged: the render does not print scopes.

## Cancel probe

No open order of another lab names these files (`gh pr list --state open` shows only #3578). DeepSeek's #3576, the last PR on `stack/format.rs`, is this unit's base. Classed instrument.

## Measurements

No model compute ran, so no tokens per second were measured: this unit changes only the item's scope.

## Checks run at the head

- `cargo run -p xtask -- roadmap render --check`: current.
- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-260/ORDER.md`: passed.
- `uor-harness loop pr-check --local`: passed, with the D76 advisories.
- `git diff --check origin/main...HEAD`: passed.

## Skills used

None applied. The loop's guard refusal and the item file were read directly.

## Still open

1. **The copy read's two-token key, trained as served on (b) (claude-261).** A copy head's 64-bit code gets the key of `x_{s−1}` in its first half and of `x_{s−2}` in its second, and its query `x_t`'s and then `x_{t−1}`'s. The distance is still one popcount, and only a repeat of both tokens is an exact match. Arms (b) and (k2) at three seeds, with TRAIN unchanged; DeepSeek's FIT plain-against-served comparison folds in as its probe.
2. **(w)'s width-512 export,** unblocked by DeepSeek's #3576: re-export the sealed (w) checkpoints and serve them, then run (w)'s three more seeds (claude-259's decision row 3).

References #3407.

## Plain summary

The next change to the model touches the code that exports and serves it, and the item's list of editable files left that code out. This unit adds those six files to the list, so the next unit can change the copy read's key in training, export and serving together.
