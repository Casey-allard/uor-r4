# ORDER: infra-loop-harness, kimi-16 (mark infra-loop-pr08-migration done)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `4ca0c2f92`.

## Goal

Set `docs/roadmap/items/infra-loop-pr08-migration.md` to `status: done`.

PR 8 (#2982) merged as `4ca0c2f92` and its `Closes #2913` auto-closed the item's issue,
but the PR never flipped the item's status, so `xtask roadmap sync` warns: "issue #2913 is
closed but the item is `ready`; not reopening it". The item's done-when is met and verified
on main (`loop close --verify --pr 2982`: 86 files identical, gates GREEN).

## Cancel probe (run first)

At `4ca0c2f92`, does the item still say `status: ready` while issue #2913 is closed? If the
item already says `done`, the task is void. Class: instrument. Result: `status: ready` with
#2913 closed, so the task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-pr08-migration.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item says `status: done`, and the diff is that one word.
2. `roadmap check`, `roadmap sync` (dry run) without the #2913 warning, and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The item's done-when is disputed.
