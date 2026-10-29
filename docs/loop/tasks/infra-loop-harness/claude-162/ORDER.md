# ORDER: infra-loop-harness, claude-162 (item maintenance: the map's next units as items)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `5fee6d037`.

Setup row: 2 (item-file maintenance: new gap items and PLAN's owner-direction rows).

## Goal

The owner, 28 Oct 2026, in claude's chat: "ok yes, my vote is to pivot to your diagnosed recommended new roadmap". PLAN records that direction as done "when all three are merged". All three now are:
- `fbs-cost-profile` (claude-157, claude-158);
- `pipeline-map-revision` (claude-159);
- `route-plus-memory` (claude-161, cell A at both seeds).

The revised [pipeline map](../../../../model/pipeline-map.md)'s order of work names the next units, R2 to R5. This unit turns them into items, so `loop claim` can take them:
- **`writer-admission` (R2):** make the cheap readout correct. claude-150's writer held at seed 1 and never learned at seed 2.
- **`fbs-serving-kernels` (R3):** split each served layer, then vector kernels exact against `step_native` (claude-158: the trunk is 98.6% of a served token).
- **`training-step-cost` (R4):** split the backward, and a cheaper rival search (claude-157).
- **`turn-memory-integer` (R5, the memory's part):** serve claude-161's turn memory from the integer artifact. `integer-hamming-read` already holds the Hamming read's part.
- **`turn-memory-reply`:** the memory beyond a reply's first word (claude-161's REPORT, "Still open").

It also sets PLAN's pivot direction to `done` and lists the next items in PLAN.

**Each item's `scope:`** lists every file its first unit will touch, checked against the tree at the base. This is claude-157's lesson: an item's scope is read from `origin/main`, so it cannot be widened inside its own unit.

## Cancel probe (run first)

Run at `5fee6d037`:
- **Not already present:** no item named `writer-admission`, `fbs-serving-kernels`, `training-step-cost`, `turn-memory-integer` or `turn-memory-reply` exists.
- **No overlap:** no open PR touches `PLAN.md` or `docs/roadmap/items/`.
- **The three pivot items** are `done` on main.

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `docs/roadmap/items/*.md` (the five new items only)
- `PLAN.md` (the pivot direction's row and the pivot list)
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-162/**`

## Done when

1. Five items, each with the owner's words or the measured gap it answers, what is missing with sources, a done-when, `## How to start`, `## Rules that apply`, `## Sources`, a checked scope and `status: ready`.
2. PLAN's pivot direction is `done`, and PLAN lists the five.
3. `cargo xtask roadmap render --check` passes. The PR is merged and verified.

## Stop if

- **An item's needed file does not exist and is not a new file the unit creates.** Name it as new, or leave it out.

No step waits on Casey.
