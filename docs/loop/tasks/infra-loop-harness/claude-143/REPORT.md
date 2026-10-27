# REPORT: infra-loop-harness, claude-143

Lab: claude. Order: [ORDER.md](ORDER.md). Item maintenance only: one item's `scope:` line. The umbrella item stays open.

**Speed.** No model, run or benchmark is in this unit; nothing to time.

## What did not work, first

- **The new scope is not proven claimable until it is on main.** `loop claim` reads the item from `origin/main`. The claim is re-run after the merge, as done-when (3) asks.
- **The umbrella's done-when does not move.**

## What changed

- `docs/roadmap/items/in-space-trunk-to-text.md`, the `scope:` line only:
  - removed `docs/roadmap/README.md`, one of the nine shared registration files in `loopclaim.rs`'s `SHARED_FILES`, which no item's scope may list;
  - added `crates/uor-r4-float-train/src/flock_read*.rs`, for the item's next unit: a float-trainable Flock read in the in-space trunk.
- `docs/roadmap/README.md` is unchanged: the render does not depend on the scope.

## Measurements

None. This unit makes no measurement.

## Checks run at the head

- `cargo xtask roadmap check`: 96 items OK.
- `cargo xtask roadmap render --check`: current, with no change to the README.

## Skills used

- **holding-the-goal.** Only the line the claim refused on was changed, plus the one path the next unit needs.

## Still open

- **Re-run `loop claim in-space-trunk-to-text` after the merge.**

## Plain summary

This fixes one line of a roadmap entry, so the next piece of work on the model's context reading can be started through the project's normal process.
