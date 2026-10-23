# ORDER: m1-position-mechanism, deepseek-36 (non-author TEST FITNESS review of the draft sheet)

Lab: deepseek. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md) (#2843). Base: `9960de160`. Item PR (partial: `References #2843`; the item stays `ready`).

## Goal

Act as the non-author `TEST FITNESS` reviewer for antigravity's draft pre-registration `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (CHARTER §4.5, D22): read the sheet, the merged `PositionHolonomyV1` implementation (`crates/uor-r4-discrete/src/position_mechanism.rs`) and its five finite controls, check the configuration is pinnable, and record the verdict on the sheet plus one honest review body. This unit clears no compute; a `NOT_FIT` verdict is a gate result, not a mechanism result.

## Cancel probe (run first)

Class **instrument** (host only, docs verdict; no GPU, no training). At the base: the sheet exists with `TEST FITNESS: PENDING`; no non-author FIT review is recorded on #2843; the code module and its 5 controls are merged (`cargo test -p uor-r4-discrete --lib position_mechanism` = 5 passed). A prior reviewer has not already recorded a verdict. Task stands.

## Files in scope

- `docs/harness/prep/m1-position-mechanism*` (the verdict and status line)
- `docs/labs/m1-position-mechanism/**` (lab-record note of the review)
- this task folder

## Done when

- `uor-harness prep fit` has recorded the verdict (one receipt) and the rendered `TEST FITNESS: ...` block is pasted into the sheet, with the sheet's top Status line updated to match.
- The verdict names every concrete prerequisite it requires, in priority order, with the evidence for each: the owner bar link, the ambiguous DEV pin, and the unpinned new evaluation slice.
- The five finite controls, the no-float/no-matmul path and the D11 whole-number export are verified by command, not repeated from the sheet.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- The verdict would need an owner value (it does not: it records PENDING prerequisites, not values).
- A file outside the scope above must change.