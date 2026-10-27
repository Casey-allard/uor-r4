# ORDER: pipeline-map, claude-148 (the model end to end: each stage's domain, size, hardware and cost)

Lab: claude. Item: [pipeline-map](../../../../roadmap/items/pipeline-map.md). Base: `5bc6f6522`.

## Goal

**Write `docs/model/pipeline-map.md`: every stage from the first trained token to the last word of a reply served on an M1 laptop.**

For each stage the page gives the item's seven fields:
- the mechanism the repository has, with its file and whether it is merged or on a branch;
- its exact input and output domains, and whether the next stage reads that output in that domain;
- its size now and the size the goal needs;
- its hardware for training and for serving, naming the M1 unit;
- its measured cost per token, with a source;
- its status, measured or hypothesis;
- what it depends on.

After the stages come three parts, every number sourced:
- the scale arithmetic for the goal;
- the gaps, ranked by what blocks the goal first;
- an order of work, each unit with what a pass and a fail unlock.

The map is design work. It runs no GPU job and changes no code. claude-146's branch (`claude/hamming-read`, #3332) and its pod are read only, never touched.

**Optional measurement, not run.** Decode tokens/s of an exported float-trunk artifact on this laptop was already measured on this same machine (Apple M1, 4 performance cores, 16 GiB) by `fbs-toks`, in claude-126 and claude-127 (`d11-exporter-float-trunk`). The map cites those figures. A rerun would time the same runtime on the same artifact shape.

## Cancel probe (run first)

Cancel if either of these holds:
- a page on main already maps every stage with its input and output domain;
- the owner withdrew the 27 Oct direction.

## Files in scope

- `docs/model/pipeline-map.md` (new)
- `docs/SUMMARY.md` (one entry under "The model, end to end")
- `docs/model/architecture.md` (one pointer line to the map)
- `docs/roadmap/items/pipeline-map.md`: byte-identical to #3333 until that merges, then `status: done`
- `docs/roadmap/README.md` (rendered)
- `PLAN.md`: the 27 Oct direction row's note, after #3333 merges
- `docs/loop/tasks/pipeline-map/claude-148/**` (this task folder)

## Done when

The item's done-when:
1. the page, with seven fields for every stage;
2. the scale arithmetic, the ranked gaps and the order of work, every number sourced;
3. the merge, and one `[claude, for Casey]` note on #2028 with the link, the five biggest gaps and the first unit.

Host checks at the head:
- `cargo xtask links`;
- `cargo xtask docs-catalog --check`;
- `cargo xtask roadmap check` and `roadmap render --check`;
- claim wording.

## Stop and ask if

The map finds that a §3 constraint costs the goal speed or size. Then it says so, labelled as a proposal for the owner's queue, and goes on; it never relaxes a constraint. Never stop and wait.
