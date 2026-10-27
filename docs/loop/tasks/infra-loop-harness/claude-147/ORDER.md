# ORDER: infra-loop-harness, claude-147 (item maintenance: add the pipeline-map item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `5bc6f6522`.

Setup row: 2 (item-file maintenance: a new gap item).

## Goal

**Add one roadmap item file, `docs/roadmap/items/pipeline-map.md`, its rendered row, and the owner's 27 Oct direction in PLAN's owner-directions table. The research unit that writes the map can then name an item that exists on main.**

- The research unit is `pipeline-map/claude-148`, on branch `claude/pipeline-map-b33df2`.
- `loop pr-check` refuses an ORDER whose item does not exist at the merge base. claude-141 (#3328) set this pattern for #3327.
- The roadmap's own rule applies: "A new gap is a new item file, added in a pull request." This unit is that pull request.
- CHARTER §5.1: "record directions verbatim the same day". The item quotes Casey's 27 Oct direction in full. PLAN's table gets its first sentence verbatim, with the status `open` and a pointer to the item.
- No harness or loop code changes. The umbrella's done-when does not move.

## Cancel probe (run first)

Cancel if either of these holds:
- the item already exists on main;
- another open order adds or edits it, or adds this direction to PLAN.

Result (2026-10-27T20:09Z, `origin/main` at `5bc6f6522`):
- `git ls-tree origin/main docs/roadmap/items/pipeline-map.md` finds nothing.
- The two open PRs are #3332 and #3323. `gh pr diff --name-only` shows that neither touches `pipeline-map` or `PLAN.md`.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/pipeline-map.md` (new)
- `docs/roadmap/README.md` (rendered by `cargo xtask roadmap render`)
- `PLAN.md` (one row in "Owner directions")
- `docs/loop/tasks/infra-loop-harness/claude-147/**` (this task folder)

## Done when

1. `cargo xtask roadmap check` passes with the item, and `cargo xtask roadmap render --check` passes.
2. The item and the PLAN row are on main.
3. After this lands, claude-148's ORDER names an item that exists at its merge base.

## Stop and ask if

The item text must change after claude-148 opens. If it must, change it in claude-148's PR, which has `docs/roadmap/items/pipeline-map.md` in its scope. Never stop and wait.
