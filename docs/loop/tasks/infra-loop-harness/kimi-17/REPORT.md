# REPORT: kimi-17, infra-loop-harness

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first item draft failed `roadmap check` twice: the `## Sources` section was missing,
  then the section order was wrong (`## Rules that apply` precedes `## Sources`). Both
  fixed by amending the item commit; the validator's messages named each problem exactly.

## What changed

One new item file, `docs/roadmap/items/infra-loop-render-review.md` (the render-PR review
gap claude named on #3031: `loop review` cannot count a review for a PR with no ORDER, so
the recurring `kimi/render` refresh stands outside the counted-review record), plus the
re-rendered roadmap README. Diff: 2 files, item +49, README +1 row. The ORDER estimated
exactly this.

## Measurements

Nothing measured; a new item card.

## Checks run at the head

- Cancel probe: the item file did not exist at `dea171338` (task stood).
- `xtask roadmap check` — PASS (89 items).
- `xtask roadmap render` — README regenerated and committed.
- `uor-harness loop render --check` — PASS.
- `uor-harness loop report --check` — PASS.
- `uor-harness loop pr-check --local` — PASS.

## Skills used

None.

## Still open

- The carded item itself (`infra-loop-render-review`, ready, open to any lab).
- After merge, `xtask roadmap sync --apply` creates its issue (PM duty).

## Plain summary

The recurring "render" pull request refreshes the project's status pages and, by design,
carries no work order — but the review counter requires one, so these PRs cannot earn a
counted review. Claude flagged this gap while reviewing the first render PR. This change
writes the gap down as a proper roadmap item so any lab can pick it up and fix it.
