# ORDER: m1-a8-erosion-diagnosis, claude-42 (close the item: notebook row and status)

Lab: claude. Item: [m1-a8-erosion-diagnosis](../../../../roadmap/items/m1-a8-erosion-diagnosis.md) (#2829). Base: `8d488457e` or later `origin/main`. Closing PR (`Closes #2829`).

## Goal

The item's done-when reads: "A diagnosis note from the retained checkpoints and move logs that states which causal claims it does and does not support; recorded as one notebook row." The note is `docs/labs/m1-a8-erosion-diagnosis/README.md`, delivered by #2992, #2994 and #2996. This unit records the one notebook row in the item's declared notebook (#2997), flips the item's status to `done` and re-renders the roadmap README.

## Cancel probe (run first)

At the base, is the item already `done`, or does `m1-discrete-attention.md` already carry a row for this diagnosis? If either is true, the task is void. Class: instrument.

## Files in scope

- `docs/research/notebook/m1-discrete-attention.md`: one appended row, through the item's `notebook:`.
- `docs/roadmap/items/m1-a8-erosion-diagnosis.md`: status only.
- `docs/roadmap/README.md` (shared): re-render.
- This task folder.

## Done when

1. **The notebook row is appended.** It uses the verdict word **inconclusive**: the evidence cannot separate placement from the read. It makes no causal claim.
2. **The status is `done`.** `cargo xtask roadmap check` and `roadmap render --check` pass after the re-render.
3. **The checks pass:** `report --check` and `pr-check --local`.

## Stop and ask if

- Anyone argues the done-when needs the device admission run. The item text says that run is out of scope.
