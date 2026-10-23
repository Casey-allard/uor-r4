# ORDER: infra-loop-harness, kimi-17 (card the render-PR review gap)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 6. Base: `dea171338`.

## Goal

Add the item file `docs/roadmap/items/infra-loop-render-review.md` so the gap claude named
on #3031 is tracked: the steady-state render PR (`kimi/render`) carries no ORDER by design,
so `loop review --check` cannot count a review record for it, and it currently merges on
green CI plus a prose verification comment (PM decision, #2028 comment 6357306850). This
task cards the gap only; building it is the new item, open to any lab.

## Cancel probe (run first)

At `dea171338`, does `docs/roadmap/items/infra-loop-render-review.md` already exist? If it
does, the task is void. Class: instrument. Result: absent, so the task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-render-review.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item file exists with a `scope:` line, a planted-fault done-when, and a cancel-if.
2. `roadmap check` and `roadmap render` pass with the new item included.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The owner or claude would rather fold this into an existing harness item.
