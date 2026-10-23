# REPORT: kimi-15, infra-loop-harness

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- Nothing failed inside this task. It exists because the docs CI job on #2982 failed with
  "stale chart figures (results-share-by-week.svg, timeline.svg,
  research-lines-timeline.svg)": the PR's notes migration re-dates the records the charts
  render from, so under the PR tree `xtask figures-charts` produces new output while the
  committed SVGs still match main. Fresh main renders clean (`figures-charts: all 4 chart
  figures are current` at `61cfeb01c`), confirming the staleness is the PR's own effect.
- A first attempt at this task regenerated the figures onto main instead of widening the
  scope; that was reverted (`git checkout -- docs/figures/`) before any commit.

## What changed

One entry in one line: `docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:` gains
`docs/figures/**`. Diff: 1 file, +1/-1 — the ORDER's estimate exactly.

## Measurements

Nothing measured; a one-line metadata change.

## Checks run at the head

- Cancel probe: the `scope:` line at `61cfeb01c` has no `docs/figures` entry (task stands).
- `xtask roadmap check` — PASS.
- `uor-harness loop render --check` — PASS.
- `uor-harness loop report --check` — PASS.
- `uor-harness loop pr-check --local` — PASS.

## Skills used

None.

## Still open

- #2982 regenerates the three chart SVGs once this lands, re-binds receipts, and returns
  to claude and deepseek for re-review at the new head; tracked on #2913.

## Plain summary

The notes-migration pull request changes the files that three status charts are drawn
from, so the charts must be redrawn inside that pull request. The rule checker reads the
list of files the task may touch from main, so that list must first gain the charts
folder on main. This change adds that one folder to the list — nothing else.
