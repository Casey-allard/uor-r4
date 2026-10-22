# REPORT: m1-address-as-read, antigravity-17 (follow-up: prep.sheet_shape blocks)

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` (landed in `#2950`) failed `prep.sheet_shape` in `uor-harness check`:** The draft sheet carried `Bar`, `Stop`, `Cost`, and `TEST FITNESS: PENDING`, but omitted the four explicit heading substrings required by `crates/uor-r4-harness/src/prep.rs::REQUIRED_BLOCKS` (`base` for `Base artifact`, `split` for `Data split`, `step` for `Steps`, and `power` for `Minimum test power`), causing `uor-harness check` in the `harness rules and tests` CI job to fail with `error prep.sheet_shape [docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md]: missing block(s): Base artifact, Data split, Steps, Minimum test power`.

## What changed

This follow-up PR (`References #2833`) completes [`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md) to the full 8-block `prep.sheet_shape` specification:

- **Pre-registration sheet ([`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md)):** Added `## 3. Bar`, `## 4. Base artifact`, `## 5. Data split`, `## 6. Steps`, `## 7. Stop conditions`, and `## 8. Cost and minimum test power`, preserving `TEST FITNESS: PENDING` and all serving-time oracle bar and control specifications from `#2950`.
- **Size by class:** ~35 lines of documentation changes in `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` plus task records in `docs/loop/tasks/m1-address-as-read/antigravity-17/`.

## Measurements

- **`uor-harness check`:**
  - Before: `223 rules, 194 failed (1 error, 192 warn)` — `error prep.sheet_shape [docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md]`.
  - After: `0 errors` (`prep.sheet_shape` passes across all sheets in `docs/harness/prep/`).

## Checks run at the head

- `uor-harness check`: `0 errors`.
- `uor-harness loop gates --full`: `6 gate result(s) GREEN`.
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `verdict: PASS`.

## Skills used

- `uor-research-architect`: followed for D22 pre-registration completeness (`prep.sheet_shape`).

## Still open

- GPU evaluation of the serving-time address-as-read oracle remains HELD pending `m3-memory-panel-freeze` (`#2853`), M1 trunk training (`#2845`), non-author `TEST FITNESS: FIT` review on [`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md), and Casey's approval in chat.

## Plain summary

The draft pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` landed in `#2950` without four of the eight section headings checked by `uor-harness check` (`Base artifact`, `Data split`, `Steps`, and `Minimum test power`), which kept the `harness rules and tests` CI job red on `main`. This follow-up adds those four sections to the sheet while keeping `TEST FITNESS: PENDING`, restoring `uor-harness check` to zero errors.
