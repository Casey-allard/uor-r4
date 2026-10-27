# REPORT: infra-loop-harness, claude-141

Lab: claude. Order: [ORDER.md](ORDER.md). Item maintenance only: one new roadmap item and its rendered row. The umbrella item stays open.

**Speed.** No model, run or benchmark is in this unit; nothing to time.

## What did not work, first

- **Nothing in this unit ran or measured a model.** The item it adds records results from #3327 (`in-space-trunk-to-text/claude-142`), where they are reviewed. They are not re-checked here.
- **The item's issue does not exist yet.** It is made by `cargo xtask roadmap sync` from main. A dry run from a branch would also rewrite five other issues from that branch's older files, so it was not applied.
- **The umbrella's done-when does not move.**

## What changed

- `docs/roadmap/items/in-space-trunk-to-text.md` (new). It is byte-identical to #3327's file at `838b0a73a77cbf6a6ade587812690eb55be03b62`, compared with `diff`. Its contents:
  - its goal: Casey's 27 Oct direction, train in the trunk's own space, decode only at the end, always look at context, show the text;
  - its state on 27 Oct;
  - a done-when in three parts, the first two on one checkpoint:
    - reply openings at least `197` (the commonest-token count `161` plus three binomial standard deviations), with top-1 at least `7,002`;
    - relative reach `R16 = shift(16) / shift(0)` at least `0.05`;
    - replies recorded at every checkpoint;
  - the rules that apply, and its sources.
- `docs/roadmap/README.md`: the row, rendered by `cargo xtask roadmap render`.

## Measurements

None. This unit makes no measurement.

## Checks run at the head

- `cargo xtask roadmap check`: 96 items OK.
- `cargo xtask roadmap render --check`: current.

## Skills used

- **holding-the-goal.** The item file is copied byte for byte from the research unit, not rewritten, so the two cannot drift.

## Still open

- **#3327 merges main once this lands,** and its `order` rule is rechecked.
- **The item's issue** is made at the next roadmap sync from main.

## Plain summary

This adds one entry to the project's roadmap, for the work of training the model in its own space until its replies answer their prompts. The entry has to be on the main branch before the pull request that does that work can merge.
