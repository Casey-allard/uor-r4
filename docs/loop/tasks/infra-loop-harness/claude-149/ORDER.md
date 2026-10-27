# ORDER: infra-loop-harness, claude-149 (item maintenance: items for the pipeline map's next units)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `b349e65ca`.

Setup row: 2 (item-file maintenance: new gap items).

## Goal

**Add two roadmap item files, for the [pipeline map](../../../../model/pipeline-map.md)'s units U2 and U3, and their rendered rows, so a lab can claim each one.**

- The owner's direction of 27 Oct ("we may have to pause for a moment and try to refocus…") is open in PLAN. Its note says the map's order of work, U1–U9, "carries the steps still missing". `loop next` gives GO on that direction, and no unit of the map has an item to claim.
- **U1** was claude-146 (#3332, merged as `b349e65ca`). Its language run took the map's fail branch: "That configuration closes."
- **U2, one decoder** (`one-decoder`, new): the map's "first new unit". The served writer and the evaluated whole-table nearest disagree on one checkpoint (345 against 7,813); the `voronoi` objective's margin can be met (claude-142: 7,906), and whether it prevents the writer's later collapse is untested.
- **U3, an integer Hamming read** (`integer-hamming-read`, new): the serving path for the look-back claude-146 proved on recall. The integer runtime has no Hamming read.
- **U5, a deterministic FBS backward,** needs no new item. It is the open item `d11-exporter-float-trunk`'s next unit by claude-140's REPORT ("make the FBS backward reduction deterministic"), inside that item's scope.
- U4 and U6–U9 depend on U2 and U3, so their items follow those units' results.
- The roadmap's rule: "A new gap is a new item file, added in a pull request." No harness or loop code changes; the umbrella's done-when does not move.

## Cancel probe (run first)

Cancel if either item already exists on main, or another open order adds or edits it.

Result (2026-10-27T22:00Z, `origin/main` at `b349e65ca`):
- `git ls-tree origin/main docs/roadmap/items/one-decoder.md` and `…/integer-hamming-read.md` find nothing.
- The open PRs are this unit's draft #3335 and antigravity's #3323. `gh pr diff --name-only` shows neither touches these items or `docs/roadmap/README.md`.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/one-decoder.md` (new)
- `docs/roadmap/items/integer-hamming-read.md` (new)
- `docs/roadmap/README.md` (rendered by `cargo xtask roadmap render`)
- `docs/loop/tasks/infra-loop-harness/claude-149/**` (this task folder)

## Done when

1. `cargo xtask roadmap check` passes with both items, and `cargo xtask roadmap render --check` passes.
2. Both items are on main, each with a scope, a done-when, a cancel rule and its sources from the map.
3. `uor-harness loop next` can offer each one after this lands.

## Stop and decide (never stop and wait)

- If an item's scope overlaps an open order, narrow the scope; never wait for the order to close.
- If the map's evidence for a unit turns out wrong when read at its source, the item says what the source says and the REPORT records the difference.
