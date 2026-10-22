# ORDER: infra-loop-harness, claude-46 (give legal-construction-exact-solve its notebook page)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `18c03cebc`.

## Goal

`legal-construction-exact-solve` (#2828) says in step 6: "No notebook page covers optimizers, so add one row on a new page `docs/research/notebook/<line>.md`". The design note on #2828 (6351533475) names the line `legal-construction`. The item's `scope:` does not cover `docs/research/notebook/`, so this unit adds `notebook: docs/research/notebook/legal-construction.md`, as #2997 did for `m1-a8-erosion-diagnosis`. Under `pr-check`, a page that does not exist at the base keeps every base line trivially, so the closing unit can create it.

## Cancel probe (run first)

At `18c03cebc`, does the item already declare `notebook:`? If it does, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/legal-construction-exact-solve.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item declares that `notebook:`, and the diff is one added line.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The roadmap check refuses a `notebook:` page that does not exist yet.
