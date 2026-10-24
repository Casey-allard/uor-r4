# ORDER: infra-loop-harness, claude-73 (the umbrella's scope gains CHARTER.md and docs/rules/changes.md)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `3ac553790`.

Casey asked for phase 0 in Claude's chat on 2026-10-24: "Yes, start phase 0: open the records PR with my words verbatim (single-lab rule, review inside the unit, pre-run and decision reviews, post-merge audits, owner queue as a log) and post the #2028 note with each lab's first move. I'll reopen the labs."

## Goal

The phase 0 records PR (claude-74) amends `CHARTER.md` and logs the change in `docs/rules/changes.md`. No roadmap item's `scope:` covers either path. `loop pr-check`'s scope rule reads item scopes at the merge base, so it would fail that PR, and `loop queue` refuses a PR whose pr-check fails.

This unit adds both paths to the `infra-loop-harness` scope, which already holds setup row A (charter text and decision rows). Both stay owner paths: changing either still needs the `## Owner approval` section that pr-check's owner-paths rule reads.

## Cancel probe (run first)

At `3ac553790`, does any item's `scope:` cover `CHARTER.md` or `docs/rules/changes.md`? No. `git grep -n "^scope:" -- docs/roadmap/items | grep -c -E "CHARTER|docs/rules"` prints `0`. The unit stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line only)
- `docs/loop/tasks/infra-loop-harness/claude-73/**`

## Done when

- The item's `scope:` lists `CHARTER.md` and `docs/rules/changes.md`, and nothing else in the item changes.
- The roadmap check passes: `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check`.
- `loop report --check`, `loop pr-check --local`, `git diff --check` and the claim-wording gate pass at the head.
- The exact-head review is Claude's own fresh-context review, which the gate at the base counts for a doc-only PR.

## Stop and ask if

Any other file has to change, or the roadmap check refuses the new paths.
