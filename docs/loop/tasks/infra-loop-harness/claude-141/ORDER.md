# ORDER: infra-loop-harness, claude-141 (item maintenance: add the in-space-trunk-to-text item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `f3a6f0f72`.

Setup row: 2 (item-file maintenance: a new gap item).

## Goal

**Add one roadmap item file, `docs/roadmap/items/in-space-trunk-to-text.md`, and its rendered row, so that the research unit that does the work can name an item that exists on main.**

- The research unit is `in-space-trunk-to-text/claude-142`, PR #3327.
- `loop pr-check` refuses an ORDER whose item does not exist at the merge base. #3327 adds the item itself, so its `order` rule fails.
- The roadmap's own rule applies: "A new gap is a new item file, added in a pull request." This unit is that pull request.
- The item records Casey's 27 Oct direction: train in the trunk's own space, decode at the end, look at context, show the text.
- It does not move the umbrella's done-when. No harness or loop code changes, so this falls within the harness freeze, which forbids harness and loop development only.

## Cancel probe (run first)

Cancel if the item already exists on main, or another open order adds or edits it.

Result (2026-10-27T10:45Z, `origin/main` at `f3a6f0f72`):

- `git ls-tree origin/main docs/roadmap/items/in-space-trunk-to-text.md` finds nothing.
- The only open PR that touches the file is #3327, this lab's research unit, which adds the same file byte for byte.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/in-space-trunk-to-text.md` (new, byte-identical to #3327's at `838b0a73a`)
- `docs/roadmap/README.md` (rendered by `cargo xtask roadmap render`)
- `docs/loop/tasks/infra-loop-harness/claude-141/**` (this task folder)

## Done when

1. `cargo xtask roadmap check` passes with the item, and `cargo xtask roadmap render --check` passes.
2. The item is on main.
3. After #3327 merges main, its `loop pr-check` `order` rule passes.

## Stop and ask if

The item text needs to change from #3327's. If it does, change it in #3327, and keep this file byte-identical to it. Never stop and wait.
