# ORDER: infra-loop-harness, claude-143 (item maintenance: the in-space-trunk-to-text scope)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `c54c68388`.

Setup row: 2 (item-file maintenance: an item's scope).

## Goal

**Fix the `scope:` line of `docs/roadmap/items/in-space-trunk-to-text.md` so the item can be claimed again.**

- `loop claim in-space-trunk-to-text` refuses. The scope lists `docs/roadmap/README.md`, one of the nine shared registration files that, by Casey's 21–22 Oct call, any loop PR may change and no item's scope may list.
- This unit removes it.
- It adds `crates/uor-r4-float-train/src/flock_read*.rs`. That is where the item's next unit puts a float-trainable Flock read: the integer Flock read exists only in the discrete crate, and the in-space trunk has no read over its context. The unit is named in #3327's REPORT and on #2028.
- No harness or loop code changes, so this stays within the harness freeze.

## Cancel probe (run first)

Cancel if the scope is already fixed on main, or another open order edits the item.

Result (2026-10-27T11:30Z, `origin/main` at `c54c68388`):

- `git show origin/main:docs/roadmap/items/in-space-trunk-to-text.md | grep '^scope:'` still lists `docs/roadmap/README.md`.
- No open PR touches the item: #3327 and #3328 are merged, and #3329 is this unit.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/in-space-trunk-to-text.md` (the `scope:` line only)
- `docs/roadmap/README.md` (rendered, if the render changes)
- `docs/loop/tasks/infra-loop-harness/claude-143/**` (this task folder)

## Done when

1. `cargo xtask roadmap check` and `roadmap render --check` pass.
2. The fix is on main.
3. `uor-harness loop claim in-space-trunk-to-text` no longer refuses on the shared-file rule.

## Stop and ask if

Another scope rule refuses the claim after the fix: fix that in the same unit. Never stop and wait.
