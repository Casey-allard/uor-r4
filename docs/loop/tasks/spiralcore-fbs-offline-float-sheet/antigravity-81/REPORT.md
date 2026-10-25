# REPORT: spiralcore-fbs-offline-float-sheet, antigravity-81

Lab: antigravity. Item: [spiralcore-fbs-offline-float-sheet](../../../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891). References #2891.

## What did not work

- Initial draft of `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md` listed `14,287` targets for `fresh_v2` (matching `ft/dev`); the fresh-context non-author `TEST FITNESS` reviewer (`c217e9a3-92f6-4836-9077-d75c682ebfad`) noted that the full `fresh_v2` artifact (`0b7e754c…`) contains `14,479` eligible targets across `44,678` payload tokens (`crates/uor-r4-discrete/src/attention_transfer.rs`). Updated Sections 4, 5, and 6 of the sheet to state the exact `14,479`-target (`44,678`-token) count for `fresh_v2`.

## What changed

- [`docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md): Authored the one-page pre-registration sheet for `spiralcore-fbs-offline-float-sheet` (`#2891`) with all 8 `REQUIRED_BLOCKS` (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`), prospectively justifying the 8D block counts (`width = 256` = $32 \times 8\text{D}$ input atoms, `mlp_hidden = 384` = $48 \times 8\text{D}$ MLP atoms in 4-stage base-3 binder trees), defining 4 matched geometric arms (`Fbs_1Atom_Active`, `Fbs_2Atom_Mix`, `Fbs_IdentityTransport_Control`, `Fbs_FrozenQueries_Null`) across seeds `1` and `2` (`1,024` steps on `mw-balp/train`), pinning all input and kernel SHA-256 digests, gating `tokens/s` (`CHARTER.md` §1) and the D11 integer table export gap (`CHARTER.md` §3.7), and recording the fresh-context non-author `TEST FITNESS: FIT` and D68 `DECISION REVIEW: APPROVE` verdicts.
- [`docs/mechanisms/spiralcore-fbs.md`](../../../mechanisms/spiralcore-fbs.md): Linked [`docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md) (`TEST FITNESS: FIT`) under "What would reopen it".
- [`docs/loop/tasks/spiralcore-fbs-offline-float-sheet/antigravity-81/ORDER.md`](ORDER.md) and [`docs/loop/tasks/spiralcore-fbs-offline-float-sheet/antigravity-81/REPORT.md`](REPORT.md): Task order and report for `antigravity-81`.
- **Size:** 0 code lines, 0 test lines, 4 docs files added/modified (~140 lines total; within estimate).

## Measurements

- `uor-harness check`: `0 error(s), 0 warning(s)` (including `prep.sheet_shape` across all 65 sheets in `docs/harness/prep/`).
- `uor-harness loop report --check --order docs/loop/tasks/spiralcore-fbs-offline-float-sheet/antigravity-81/ORDER.md`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `11/11 GREEN`.

## Checks run at the head

- `uor-harness check`
- `uor-harness loop report --check --order docs/loop/tasks/spiralcore-fbs-offline-float-sheet/antigravity-81/ORDER.md`
- `uor-harness loop pr-check --local`
- `uor-harness loop gates --full`

## Skills used

- `scope-guard`: Kept all edits strictly within the declared docs/prep/task scope of `spiralcore-fbs-offline-float-sheet` (`#2891`).
- `solution-integrity`: Verified all SHA-256 digests (`mw-balp/train`, `ft/dev`, `fresh_v2`, `tokenizer.json`, `spiralcore_fbs.cu`, `spiralcore_fbs.rs`, `cuda_stack.cu`) and all 8 `REQUIRED_BLOCKS` against `crates/uor-r4-harness/src/prep.rs`.

## Still open

- Restore `crates/uor-r4-float-train/**, crates/uor-r4-integer/**` to `spiralcore-fbs-offline-float-sheet.md` once `PR #3231` merges, implement the runner binary, pass the D67 pre-run review, and execute the probe ladder and 2-seed run on a leased RTX 5090 pod.

## Plain summary

Before running the full-rung offline-float SpiralCore/FBS projection test (`#2891`) on a GPU, `CHARTER.md` §4.5 requires a one-page pre-registration sheet with a non-author `TEST FITNESS: FIT` review. This unit authors `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md` with four matched geometric arms (`Fbs_1Atom_Active`, `Fbs_2Atom_Mix` testing 2-atom routed capacity, `Fbs_IdentityTransport_Control`, and `Fbs_FrozenQueries_Null`), pre-registered bars (`G-Fbs-Sheet-Fit`, `G-Fbs-Capacity`, `G-Fbs-ExportGap`, `G-Fbs-Transfer`), instrument checks (`I-1..4`), `tokens/s` and D11 integer export-gap gates, and records its fresh-context `TEST FITNESS: FIT` and D68 decision reviews.
