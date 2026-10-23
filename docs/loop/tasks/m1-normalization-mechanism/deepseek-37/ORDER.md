# ORDER: m1-normalization-mechanism, deepseek-37 (non-author TEST FITNESS review of the draft sheet)

Lab: deepseek. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `30febc5ad`. Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

Act as the non-author `TEST FITNESS` reviewer for antigravity's draft `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` (CHARTER §4.5, D22): read the sheet, the merged `OrbitRootSnapNormV1` implementation (`crates/uor-r4-discrete/src/normalization_mechanism.rs`) and its six finite controls, check that the configuration is pinnable, and record the verdict on the sheet plus one honest review body. This unit clears no compute.

## Cancel probe (run first)

Class **instrument** (host only, docs verdict; no GPU, no training). At the base: the sheet exists with `TEST FITNESS: PENDING`; no non-author FIT review is recorded on #2842; the code module with 6 controls is on `main` (`cargo test -p uor-r4-discrete --lib normalization_mechanism` = 6 passed). Task stands.

## Files in scope

- `docs/harness/prep/m1-normalization-mechanism*` (the verdict and status line)
- `docs/labs/m1-normalization-mechanism/**` (lab-record note of the review)
- this task folder

## Done when

- `uor-harness prep fit` recorded the verdict (one receipt) and the rendered `TEST FITNESS: ...` block is in the sheet, with the top Status line matching.
- The verdict names every concrete prerequisite, in priority order, with the evidence for each: the owner bar, the ambiguous DEV pin, and any other unpinned configuration element I find.
- The six finite controls, the no-float/no-matmul path and the whole-number export are verified by command, not repeated from the sheet.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- The verdict would need an owner value (it does not: it records PENDING prerequisites, not values).
- A file outside the scope above must change.