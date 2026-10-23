# REPORT: kimi-14, infra-loop-harness

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- Nothing failed inside this task. The task exists because PR 8's `pr-check --local`
  `scope` rule FAILs at `690150b03`: the kimi-11 repair's rendered-lock both-way tests
  live in `crates/uor-r4-harness/tests/loop_prcheck.rs`, which the item scope does not
  name, and `pr-check` reads scope at the merge base, so PR 8 cannot widen it for itself.

## What changed

One entry in one line: `docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:` gains
`crates/uor-r4-harness/tests/loop_prcheck.rs`. Diff: 1 file, +1/-1. The ORDER estimated a
one-entry diff; the diff is exactly that.

## Measurements

Nothing measured; a one-line metadata change.

## Checks run at the head

At the head commit:

- Cancel probe: `grep -c loop_prcheck` on the item's `scope:` at `1c728ff87` printed 0
  before the change (task stands).
- `xtask roadmap check` — PASS.
- `uor-harness loop render --check` — PASS.
- `uor-harness loop report --check` — PASS.
- `uor-harness loop pr-check --local` — PASS.

## Skills used

None.

## Still open

- PR 8 (#2982) merges fresh main once this lands, re-binds receipts, and goes back to
  claude and deepseek for re-review; tracked on #2913.

## Plain summary

The big notes-migration pull request was asked by its reviewers to add tests in a file
that the task's official list of allowed files did not include. The checker reads that
list from main, not from the pull request, so the list must be fixed on main first. This
change adds that one test file to the list — nothing else.
