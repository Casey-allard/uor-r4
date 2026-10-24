# ORDER: m2-pin-train-mask-sheet, deepseek-74 (record the owner answer and finish the sheet-level FIT prerequisites)

Lab: deepseek. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (#2851). Base: `f76e3f5d2`. This unit does not close the item: two prerequisites are separate dependency items and one is a data seal.

## Goal

The owner answered the live/superseded question on 2026-10-24T03:42Z (#2851, #2028 `6379430093`): *"I want both because we do not know which we will ultimately need."* This unit records that answer in the D37-basis refresh (`m2-pin-train-mask-sheet-2026-10-23.md`), resolves the two sheet-level `NOT_FIT` prerequisites — the D37 export-gap statistic + numeric band (as the PM role delegate, CHARTER §5.2; deepseek covers PM meanwhile) and the literal frozen source SHAs — pre-registers the fresh-transfer slice gate, updates the stale `docs/harness/milestones.json` cache text, and records the remaining external prerequisites in the lab README.

## Cancel probe (run first)

At `f76e3f5d2`: the sheet still carries `TEST FITNESS: NOT_FIT` with (1) the owner answer recorded but not reflected in the sheet, (2) the export-gap band delegated to the reviewer, (3) literal source SHAs only in the lab README, and (4) the fresh-transfer slice unpinned; `docs/harness/milestones.json` still says `physical sheet PENDING`. The task stands.

## Files in scope

- `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`
- `docs/harness/milestones.json`
- `docs/labs/m2-pin-train-mask-sheet/**`
- this folder

## Done when

- The sheet records the owner answer, the registered export-gap statistic and band (exact formula, anchored at `0.83 pp`/`119` rows worst-case Wilson plus the two-seed spread), the literal SHAs with their true reachability (`9e904a834` on main; `2fd6358bb` squashed source), and the pre-registered fresh-transfer slice gate (`>= 200` rows, path + SHA-256, not yet sealed).
- `docs/harness/milestones.json` no longer says the physical sheet is PENDING and names the remaining prerequisites; the lab README records what this unit did, the remaining `FIT` blockers (D29 float module #2823, D11 exporter #2820, sealed fresh-transfer slice) and that nothing is released.
- `loop report --check`, `loop pr-check --local`, a JSON parse of `milestones.json`, and `git diff --check` pass; a review records at the head.

## Stop and ask if

- A bar, hold or status change is needed, a file outside scope must change, or the owner answer would be used to release compute.
