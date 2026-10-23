# Lab Record: `m2-pin-train-mask-sheet` (`#2851`, `deepseek-42`)

- **Item:** [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`)
- **Refreshed sheet:** [`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`](../../harness/prep/m2-pin-train-mask-sheet-2026-10-23.md) (`TEST FITNESS: PENDING`)
- **Old sheet:** [`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md) (Amendments 1-7; Amendment 7 froze it under D32)

## What this unit did (2026-10-23)

1. **Re-verified the four Amendment 6 pins against current `main`:**
   - the frozen source commits `2fd6358bb` and `9e904a834` exist in history; the 11-control count reference is recorded in `archive/labs/count-reference-2026-10-13/README.md`;
   - the CLI still carries `train_mask`/`expected_train_targets` (`crates/uor-r4-training/examples/geometric-stack-native.rs`);
   - the TRAIN mask digest `b5e7352d…` is the current code pin (`attention_learning.rs`); the expected count `2,425,925` is carried by the count-reference archive;
   - the DEV pins (42,856 tokens / 14,287 targets / 111+232 window inventory) are restated in the refresh from the old sheet's Amendment 4 record.
2. **Wrote the refreshed one-page sheet on the D37 basis:** floating point is legal offline with a reported export gap, and **a gap above the two-seed noise band disqualifies** (CHARTER §3.7); per **CHARTER §4 rule 5**, an archived float sheet returns only as a **new sheet with a fresh FIT**, so this page is a new sheet at `PENDING`, not a revival - the old sheet stays the archived design record. The arm objectives are explicit (M1 `hinge`, M8 `hinge8`). Bars, reference, data pins, steps, stop conditions, cost and power are restated unchanged (the old sheet's full wording remains authoritative).
3. **Posted the decision card on #2028/#2851, then corrected its framing** after review: under CHARTER §4 rule 5 an archived float sheet cannot be simply "revived", so the question for the PM is whether this **new D37-basis sheet** is the right successor to the Amendment 6 design record (recommended) or whether a discrete-law sheet is still wanted. The correction is posted on both issues; the card changes no bar, hold or spend.

## Open items before closing `#2851`

- PM's recorded live/superseded answer.
- A non-author FIT review of the refreshed sheet (Claude/Kimi/antigravity; deepseek authored it).
- Nothing here releases the M2 training/rental hold; a FIT releases nothing.

## Plain summary

- Verified the four pins still hold on current `main`, wrote the D37-basis one-page refresh, and asked the PM the one question the item says only the PM/owner can answer.
- Training and rental stay HELD; no compute was run and no bar changed.