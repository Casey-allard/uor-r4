# Lab Record: `m2-pin-train-mask-sheet` (`#2851`, `deepseek-42`)

- **Item:** [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`)
- **Refreshed sheet:** [`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`](../../harness/prep/m2-pin-train-mask-sheet-2026-10-23.md) (`TEST FITNESS: NOT_FIT` — non-author audit recorded by `antigravity-33`)
- **Old sheet:** [`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md) (Amendments 1-7; Amendment 7 froze it under D32)

## What this unit did (2026-10-23)

1. **Re-verified the four Amendment 6 pins against current `main`:**
   - the frozen source commits `2fd6358bb` and `9e904a834` exist in history; the 11-control count reference is recorded in `archive/labs/count-reference-2026-10-13/README.md`;
   - the CLI still carries `train_mask`/`expected_train_targets` (`crates/uor-r4-training/examples/geometric-stack-native.rs`);
   - the TRAIN mask digest `b5e7352d…` is the current code pin (`attention_learning.rs`); the expected count `2,425,925` is carried by the count-reference archive;
   - the DEV pins (42,856 tokens / 14,287 targets / 111+232 window inventory) are restated in the refresh from the old sheet's Amendment 4 record.
2. **Wrote the refreshed one-page sheet on the D37 basis:** floating point is legal offline with a reported export gap, and **a gap above the two-seed noise band disqualifies** (CHARTER §3.7); per **CHARTER §4 rule 5**, an archived float sheet returns only as a **new sheet with a fresh FIT**, so this page is a new sheet at `PENDING`, not a revival - the old sheet stays the archived design record. The arm objectives are explicit (M1 `hinge`, M8 `hinge8`). Bars, reference, data pins, steps, stop conditions, cost and power are restated unchanged (the old sheet's full wording remains authoritative).
3. **Posted the decision card on #2028/#2851, then corrected its framing** after review: under CHARTER §4 rule 5 an archived float sheet cannot be simply "revived", so the question for the PM is whether this **new D37-basis sheet** is the right successor to the Amendment 6 design record (recommended) or whether a discrete-law sheet is still wanted. The correction is posted on both issues; the card changes no bar, hold or spend.

## Non-author `TEST FITNESS` audit (`antigravity-33`, 2026-10-23)

- **Verdict:** `TEST FITNESS: NOT_FIT - M2 #2030 rank-margin-objective` (audited by `antigravity` via fresh-context reviewer `40c9d939-5725-4a06-b791-5d7c7f3aa071`).
- **Four Amendment 6 pins:** All four pins verify against current `main` and `archive/labs/count-reference-2026-10-13/README.md` (`mw-balp/train/response_mask.u8` `7,512,797` B sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`; `expected_train_targets=2425925`; `geometric-stack train` CLI with `objective=hinge` for M1 and `objective=hinge8` for M8; frozen commits `2fd6358bbc52d2a202a872ad0954ebb12894606f` and `9e904a83448c451f738d5c1d45224dfdf73eb84e`).
- **Four prerequisites before `TEST FITNESS: FIT` can be granted:**
  1. **PM/owner scope answer (`#2851` / `owner-decision-float-path-scope`):** Settle whether this D37 float rung-0 sheet is the active successor to Amendment 6 or superseded by the discrete learning law.
  2. **Pre-registered D37 export-gap statistic and numeric band (CHARTER §3.7, §5.2, D39 item 5):** Specify the exact gap metric (`|top1_f32 - top1_int|` and `|recall8_f32 - recall8_int|` on the `14,287` DEV rows) and fix the numeric band in the sheet itself rather than delegating it to the FIT reviewer.
  3. **D11 float-trunk exporter (`#2820`) and separate D29-scanned float module (`#2823`):** Land both items and wire integer export + same-evaluator integer scoring into `## Steps`.
  4. **Literal source SHAs and fresh-transfer slice pin:** State the literal 40-char frozen source SHAs (`2fd6358bbc52d2a202a872ad0954ebb12894606f` / `9e904a83448c451f738d5c1d45224dfdf73eb84e`) and pin the file path, SHA-256, and `>= 200` row count for the fresh-transfer draw in the sheet body.

## Open items before closing `#2851`

- PM's recorded live/superseded answer (`#2851` comment `6358236299`).
- Resolution of the four `FIT` prerequisites above (if the PM selects the D37-basis sheet) or a discrete-law sheet (if superseded).
- Nothing here releases the M2 training/rental hold; a FIT releases nothing.

## Plain summary

- Verified the four pins still hold on current `main`, wrote the D37-basis one-page refresh, and asked the PM the one question the item says only the PM/owner can answer.
- Training and rental stay HELD; no compute was run and no bar changed.