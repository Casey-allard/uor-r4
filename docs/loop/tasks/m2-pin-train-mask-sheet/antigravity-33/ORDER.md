# ORDER: m2-pin-train-mask-sheet, antigravity-33

Lab: antigravity. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md). Base: `e490ffecc`.

## Goal

The item's done-when reads:

> A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## Cancel probe (run first)

Class: instrument / pre-registration audit (host-only, no GPU). At `e490ffecc`, `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` (`deepseek-42`, PR #3037) is on `main` at `TEST FITNESS: PENDING`, and `docs/labs/m2-pin-train-mask-sheet/README.md` line 20 requests a non-author `TEST FITNESS` review from Claude, Kimi, or `antigravity`. Task stands.

## Files in scope

- `crates/uor-r4-training/**`
- `data/**`
- `docs/harness/prep/m2-pin-train-mask-sheet*`
- `docs/harness/milestones.json`
- `docs/labs/m2-pin-train-mask-sheet/**`

## Done when

- Non-author `TEST FITNESS` audit (`uor-harness prep fit -m m2 --verdict NOT_FIT --sheet docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md --mechanism rank-margin-objective`) is completed across the 5-criterion rubric and recorded in `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` and `docs/labs/m2-pin-train-mask-sheet/README.md`, with the four concrete prerequisites for `FIT` documented and posted on `#2030` and `#2851`.
- `uor-harness prep-check` reads `m2-pin-train-mask-sheet-2026-10-23.md` as `NOT_FIT`; `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.
- Item `m2-pin-train-mask-sheet` stays `status: ready` until the PM/owner scope answer and the four `FIT` prerequisites land (`References #2851, #2030`).

## Stop and ask if

- Any bar, hold, spend cap, or D37 export-gap numeric band would have to be set by the reviewer (per CHARTER §5.2, a reviewer cannot set a missing bar or numeric band).
- Any file outside the item scope would need to change.
