# ORDER: m2-pin-train-mask-sheet, antigravity-34

Lab: antigravity. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md). Base: `73534ba73`.

## Goal

The item's done-when reads:

> A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## Cancel probe (run first)

Class: doc repair (`#2851`). At `73534ba73` (merged PR #3062), Kimi's exact-head review (`6365383954`) noted that blocker `(c)` in `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` and `docs/labs/m2-pin-train-mask-sheet/README.md` cites `#2845` for `d11-exporter-float-trunk` and `#2846` for `float-training-module` instead of the live issue numbers `#2820` (`d11-exporter-float-trunk`) and `#2823` (`float-training-module`). Task stands.

## Files in scope

- `crates/uor-r4-training/**`
- `data/**`
- `docs/harness/prep/m2-pin-train-mask-sheet*`
- `docs/harness/milestones.json`
- `docs/labs/m2-pin-train-mask-sheet/**`

## Done when

- `#2845` is replaced with `#2820` (`d11-exporter-float-trunk`) and `#2846` is replaced with `#2823` (`float-training-module`) in `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` and `docs/labs/m2-pin-train-mask-sheet/README.md`.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass (`References #2851, #2030`).

## Stop and ask if

- Any file outside the item scope would need to change.
