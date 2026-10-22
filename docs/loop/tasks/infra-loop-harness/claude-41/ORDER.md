# ORDER: infra-loop-harness, claude-41 (give m1-a8-erosion-diagnosis its notebook page)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `b00d9edad` or later `origin/main`.

## Goal

The item `m1-a8-erosion-diagnosis` (#2829) says in its own step 6: "Record the result as one row in `docs/research/notebook/m1-discrete-attention.md`". Its `scope:` does not include that page, and the item declares no `notebook:` field. `loop pr-check` already allows an append-only row in an item's declared notebook (`loopprcheck.rs:468`), and `notebook:` is a validated item field (`xtask/src/roadmap.rs:211`, `loopclaim.rs:410`).

This unit adds `notebook: docs/research/notebook/m1-discrete-attention.md` to that item, so its closing unit can add the row.

## Cancel probe (run first)

At the base, does `docs/roadmap/items/m1-a8-erosion-diagnosis.md` already declare `notebook:`, or list the notebook in `scope:`? If it does, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/m1-a8-erosion-diagnosis.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item declares that `notebook:`, and the diff adds only that line.
2. `cargo xtask roadmap check` and `roadmap render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The roadmap check rejects the field, or the render changes outside this item.
