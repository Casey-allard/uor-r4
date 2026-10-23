# ORDER: m2-fresh-heldout-panel, deepseek-43 (non-author TEST FITNESS review of `m2-fresh256`)

Lab: deepseek. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `a4a5e1efb`. `References #2849`; the item stays `ready` until the FIT and the seal upload both land.

## Goal

Perform prerequisite 1 of the panel's own FIT gate: a **non-author** `TEST FITNESS: FIT` review of the `m2-fresh256` construction / overlap exclusion / tokenizer encoding / control audit / content seal. Prerequisite 2 (a maintainer uploads the four sealed files to the HF store) is not in this unit; it keeps the panel `frozen: false`. This unit runs no training or evaluation.

## Cancel probe

Class **instrument** (host-only; no GPU). At the base: the panel JSON says `TEST FITNESS: PENDING` and `frozen: false`; the lab record's section 5 lists the two prerequisites; no FIT review is recorded. Task stands.

## Files in scope

- `docs/labs/m2-fresh-heldout-panel/**` (the FIT record)
- `docs/harness/panels/**` (the panel JSON's fitness note; bars unchanged)
- this task folder

## Done when

- The four sealed artifacts reproduce byte-for-byte from the instrument (plan/inputs/labels/manifest sizes and sha256 match the registered panel JSON).
- The audit evidence is verified by running the instrument: 74 historical literals parsed, 0 exact/content-word/2-gram overlaps, 0 intra-panel duplicates, min edit distance 10, role/initial balance true, leak count 0, all four question-blind/role-swap controls 0.
- The lab record carries the non-author `TEST FITNESS: FIT` with its evidence and the one remaining prerequisite (HF upload); the panel JSON note is updated to match; no bar, row, seed or grader changes.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record pass at the head.

## Stop and ask if

- The reproduction mismatches a registered digest (then it is an instrument defect, not a negative).
- The HF upload would be attempted (no HF write access here; that stays the maintainer's step).