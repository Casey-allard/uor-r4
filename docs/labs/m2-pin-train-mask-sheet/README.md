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
## deepseek-74 follow-up (2026-10-24)

- Recorded the owner answer (`2026-10-24T03:42Z`, #2851 / #2028 `6379430093`): both the D37-basis sheet and a discrete-law sheet are wanted; deepseek covers the PM role meanwhile.
- Registered the D37 export-gap statistic and band in the D37-basis sheet: `G_top1 = |top1_f32 - top1_int|`, `G_recall8 = |recall8_f32 - recall8_int|` on the same 14,287 DEV rows; disqualify iff `max(G_top1, G_recall8) > max(0.83 pp (119 rows), two-seed spread)`.
- Pinned the literal SHAs with their true reachability (`9e904a834` on main; `2fd6358bb` squash-merged source, object present) and pre-registered the fresh-transfer slice gate (`>= 200` rows; path + SHA-256; **not yet sealed**).
- Fixed the stale `physical sheet PENDING` text in `docs/harness/milestones.json`.
- **Remaining FIT blockers on the D37 float sheet:** D29 float module (#2823), D11 float-trunk exporter (#2820), and a sealed float fresh-transfer slice. No training, hold or bar changed; nothing was run.

## antigravity-104 discrete-law M2 pre-registration sheet & `TEST FITNESS: FIT` (2026-10-26, `#2851`)

- **Discrete-law pre-registration sheet:** [`docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`](../../harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md)
- **Non-author `TEST FITNESS` verdict:** **`TEST FITNESS: FIT - M2 #2030 discrete-geometric-learning`** (independent fresh-context non-author reviewer `d929c424-0c35-4319-9260-6c9e41913c06`; posted on `#2851` and `#2030`).
- **What this unit delivered (`antigravity-104`, PR `#3300`):**
  1. **Completed the discrete-law route of `#2851`:** Following the owner's scope ruling (`2026-10-24T03:42Z`, [`#2028` comment `6379430093`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6379430093), *"I want both because we do not know which we will ultimately need"*) and the completion of `m1-bvg-k-stage-t-training` (`#2836`, merged in `#3239` at `c695ad21a72c5b91def22458eebbe796e07d5392`), `m1-trunk-assembly-and-training` (`#2844`, merged in `#3258` at `6f38dd393865a0b4320a2e28bee3be675130dc16`), and `m2-fresh-heldout-panel` (`#2849`, merged in `#3147`/`#3148` at `f1f55c0c1095d9b4da8457f655ac72d42d9f0180` / `0201e026b6358db4951c5a972b278e1538b198b4`), wrote the one-page discrete-law M2 pre-registration sheet [`docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`](../../harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md).
  2. **Pinned all four Amendment 6 count-reference items on the current discrete law:**
     - `mw-balp/train/tokens.u16` (`7,512,797` tokens, sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`) and `mw-balp/train/response_mask.u8` (`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`);
     - `expected_train_targets=2425925` (`expected_eligible_targets = 2425925`), enforced fail-closed by `crates/uor-r4-discrete/src/controller.rs:184-191` and `docs/labs/m1-bvg-k-stage-t-training/configs/t16-{frozen-zero,relation}-seed{1,2}.json`;
     - frozen count-reference source commits `2fd6358bbc52d2a202a872ad0954ebb12894606f` (`11` finite controls PASS) and PR `#2328` merge `9e904a83448c451f738d5c1d45224dfdf73eb84e` (ancestor of `main`);
     - the discrete CUDA trainer and Stage 3 trunk CLI (`stage-t-run` and `trunk-run` in `crates/uor-r4-discrete`, replacing the frozen float `geometric-stack train` CLI on the discrete branch) with deterministic compiled pins (`driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`, `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`, `kernel_sha256 = 0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6`) and zero-gap `D11` (`uor-r4.lut-discrete/1`) integer artifact export.
  3. **Pinned both rung-0 rank-margin and M2 grounded-reply acceptance bars and sealed fresh panels:**
     - Rung-0 rank-margin bars (`top-1` and `recall@8` >= count predictor + `5 pp` across `Seed 1` and `Seed 2` on `14,287` `ft/dev` rows and `14,287` `fresh_v2` rows);
     - M2 grounded-reply bars (`complete >= 256 / 512` on `m2-reply512` and `pairs_both_complete_distinct >= 52 / 128` with single-row `complete >= 104 / 256` on the sealed 256-row `m2-fresh256` panel at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`, graded by the existing deterministic complete-reply evaluator unchanged).
  4. **Recorded the notebook rows in [`docs/research/notebook/m2-grounded-reply.md`](../../research/notebook/m2-grounded-reply.md) and reconciled [`docs/harness/milestones.json`](../../harness/milestones.json):** Both the discrete-law sheet (`TEST FITNESS: FIT`) and the retained D37 companion sheet (`m2-pin-train-mask-sheet-2026-10-23.md`, `TEST FITNESS: NOT_FIT` pending `#2823` and `#2820`) are on record; M2 training and rental stay `HELD` (`training_hold: true`, `rental_hold: true`), and `#2851` is closed (`status: done`).
