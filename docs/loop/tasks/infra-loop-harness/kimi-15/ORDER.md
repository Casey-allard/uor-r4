# ORDER: infra-loop-harness, kimi-15 (widen PR 8's scope to the rendered chart figures)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `61cfeb01c`.

## Goal

Widen `infra-loop-pr08-migration`'s `scope:` by one entry: `docs/figures/**`.

PR 8 (#2982) migrates the notes (STATUS, NOW, claims) into `archive/`; the chart figures
under `docs/figures/` render from those records, so under the PR 8 tree
`xtask figures-charts` legitimately produces new output and the docs CI job fails with
"stale chart figures" (results-share-by-week.svg, timeline.svg,
research-lines-timeline.svg). Fresh main renders clean — the staleness is caused by the
PR's own migration, so the re-render belongs in PR 8. `pr-check` reads the item scope at
the merge base, so the widening must land on main first. Same pattern as #2991, #3000 and
#3026.

## Cancel probe (run first)

At `61cfeb01c`, is `docs/figures/**` absent from
`docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:`? If it is already listed, the
task is void. Class: instrument. Result: absent (the scope line has no `docs/figures`
entry), so the task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-pr08-migration.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item's `scope:` lists `docs/figures/**`, and the diff is that one entry.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR already changes this item's `scope:` line.
