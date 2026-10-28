# ORDER: infra-loop-harness, claude-160 (item maintenance: two items' format)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `6ac7659af`.

Setup row: 2 (item-file maintenance: two items' sections, title and area).

## Goal

**Make `cargo xtask roadmap render --check` pass on main, and re-render.**

It fails with 8 problems, all in two items claude-156 wrote (found by claude-159's cancel probe):
- **`fbs-cost-profile.md`:** missing `## How to start`, `## Rules that apply` and `## Sources`.
- **`route-plus-memory.md`:** its title is 85 characters, over the limit of 80; its area `memory` is not one of the allowed areas; and the same three sections are missing.

This unit fixes the front matter and adds the sections, with no change to either item's goal, scope or done-when. It then runs `roadmap render`, which refuses while the check fails.

`route-plus-memory` is the next unit's item. Its `How to start` names the pipeline map's R1 and the archive's lesson.

## Cancel probe (run first)

Cancel if the check already passes on main, or another open order edits either item.

Result (2026-10-28, `origin/main` at `6ac7659af`):
- `roadmap render --check` fails with the 8 problems above.
- No open PR touches either item file.
- `loop claim` accepted this unit, so no open order overlaps it.

**Not cancelled** (instrument: none needed).

## Files in scope

- `docs/roadmap/items/fbs-cost-profile.md` (the three sections only)
- `docs/roadmap/items/route-plus-memory.md` (the title, the area and the three sections only)
- `docs/roadmap/README.md` and whatever else `roadmap render` writes (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-160/**`

## Done when

1. `cargo xtask roadmap render --check` passes.
2. `roadmap render` has re-rendered its outputs.
3. The fix is on main, and `loop claim route-plus-memory` accepts the item.

## Stop if

- **The render writes outside the files above.** Note it, and keep it only if it is the render's own output.

No step waits on Casey.
