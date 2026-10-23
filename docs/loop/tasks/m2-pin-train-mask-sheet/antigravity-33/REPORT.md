# REPORT: m2-pin-train-mask-sheet, antigravity-33

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: `m2-pin-train-mask-sheet` (#2851). Base: `e490ffecc`. `References #2851, #2030`.

## What did not work, first

- **The refreshed sheet (`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`) cannot be stamped `TEST FITNESS: FIT` in its current state.** While all four Amendment 6 mechanical pins verify cleanly against `main` and `archive/labs/count-reference-2026-10-13/README.md`, four concrete governance and instrument blockers prevent clearing the sheet for compute (`TEST FITNESS: NOT_FIT`):
  1. **Unresolved PM/owner scope question (`#2851` / `owner-decision-float-path-scope`):** The PM/owner decision on whether this D37-basis float rung-0 sheet is the active successor to Amendment 6 or superseded by the discrete learning law is still open.
  2. **Delegated D37 export-gap numeric band (Criterion 2 / CHARTER §3.7, §5.2, D39 item 5):** Line 29 delegates fixing the numeric export-gap band to the FIT reviewer (*"the non-author FIT fixes the numeric band for this sheet before any run"*). Under CHARTER §5.2 a reviewer cannot author or change a bar or numeric band; the sheet itself must specify the exact gap statistic (e.g., `|top1_f32 - top1_int|` and `|recall8_f32 - recall8_int|` on the `14,287` DEV rows) and fix the band or pre-registered formula before requesting `FIT`.
  3. **Missing D11 float-trunk exporter (`#2845`) and separate D29-scanned float module (`#2846`) (Criteria 1 & 4 / CHARTER §3.7, D37 items 3 & 6):** `## Steps` runs `geometric-stack` inside `crates/uor-r4-training` (`model.scored_in_f32(...)`) with no D11 integer export or same-evaluator integer scoring step, so the CLI measures only the f32 proxy and cannot evaluate the D37 export-gap gate.
  4. **Missing literal source SHAs and fresh-transfer slice pin in the sheet body (Criterion 3):** The sheet body omits the literal 40-char frozen source SHAs (`2fd6358bbc52d2a202a872ad0954ebb12894606f` and `9e904a83448c451f738d5c1d45224dfdf73eb84e`, which appear only in `docs/labs/m2-pin-train-mask-sheet/README.md:10`) and does not pin the file path, SHA-256, or `>= 200` row count for the fresh-transfer draw (lines 21, 69).

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` | Recorded `TEST FITNESS: NOT_FIT - M2 #2030 rank-margin-objective` from fresh-context subagent `40c9d939-5725-4a06-b791-5d7c7f3aa071`, with verification of the 4 Amendment 6 pins and the 4 prerequisites required before `FIT` |
| `docs/labs/m2-pin-train-mask-sheet/README.md` | Recorded the non-author `TEST FITNESS: NOT_FIT` audit summary and updated the open items before closing `#2851` |
| `docs/loop/tasks/m2-pin-train-mask-sheet/antigravity-33/{ORDER,REPORT}.md` | Task order and report for `antigravity-33` |

## Measurements

- Host-only pre-registration audit (`$0`, no GPU):
  - Verified `mw-balp/train/response_mask.u8` (`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`) and `expected_train_targets=2425925` against `crates/uor-r4-discrete/src/attention_learning.rs:1367-1370`, `crates/uor-r4-training/examples/geometric-stack-native.rs:149,162,538,608,824`, and `archive/labs/count-reference-2026-10-13/README.md:32,84,95-96`.
  - Verified frozen commits `2fd6358bbc52d2a202a872ad0954ebb12894606f` and `9e904a83448c451f738d5c1d45224dfdf73eb84e` in git history.

## Checks run at the head

- `uor-harness prep fit -m m2 --verdict NOT_FIT --sheet docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md --mechanism rank-margin-objective`
- `uor-harness prep-check` (`not FIT / NOT_FIT / yes` on `m2-pin-train-mask-sheet-2026-10-23.md`)
- `uor-harness loop report --check --order docs/loop/tasks/m2-pin-train-mask-sheet/antigravity-33/ORDER.md`
- `uor-harness loop pr-check --local`

## Skills used

- `solution-integrity`: verified all four Amendment 6 pins directly against code and archive and audited D37/CHARTER §3.7/§5.2 compliance rather than rubber-stamping a `PENDING` float sheet whose exporter and numeric export-gap band are not yet in place.
- `scope-guard`: touched only `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`, `docs/labs/m2-pin-train-mask-sheet/README.md`, and this task folder.

## Still open

- PM/owner answer on `#2851` (`6358236299`) settling whether the D37-basis float rung-0 sheet is active or superseded, plus the four `FIT` prerequisites recorded in `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`. Item `m2-pin-train-mask-sheet` stays `status: ready`. References #2851, #2030.

## Plain summary

- Completed the requested non-author `TEST FITNESS` review of `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`: verified that all four Amendment 6 mechanical pins hold on current `main`, and recorded `TEST FITNESS: NOT_FIT` with the four concrete governance/instrument prerequisites required before the sheet can be cleared for compute.
