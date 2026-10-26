# ORDER: m2-pin-train-mask-sheet, antigravity-104

Lab: antigravity. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md). Base: `bc5cad8da`.

## Goal

The item's done-when reads:

> A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## Cancel probe (run first)

- Verify on CPU (no GPU, no training, classified **instrument** if any check fails):
  1. `git cat-file -t 2fd6358bbc52d2a202a872ad0954ebb12894606f` (`commit`) and `git merge-base --is-ancestor 9e904a83448c451f738d5c1d45224dfdf73eb84e HEAD` (exit 0) for the count-reference source and PR `#2328` merge commit, plus `6f38dd393865a0b4320a2e28bee3be675130dc16` (`#3258`, `m1-trunk-assembly-and-training`), `c695ad21a72c5b91def22458eebbe796e07d5392` (`#3239`, `m1-bvg-k-stage-t-training`), and `0201e026b6358db4951c5a972b278e1538b198b4` / `f1f55c0c1095d9b4da8457f655ac72d42d9f0180` (`#3148` / `#3147`, `m2-fresh-heldout-panel`) as ancestors of `HEAD`.
  2. Verify the four mechanical TRAIN-mask and DEV pins in `crates/uor-r4-discrete/src/controller.rs:184-199`, `crates/uor-r4-discrete/src/attention_learning.rs:1563-1566`, `docs/labs/m1-bvg-k-stage-t-training/configs/t16-frozen-zero-seed{1,2}.json`, `docs/labs/m1-bvg-k-stage-t-training/configs/t16-relation-seed{1,2}.json`, `docs/harness/panels/m2-reply512.json`, and `docs/harness/panels/m2-fresh256.json`:
     - `mw-balp/train/tokens.u16` (`7,512,797` payload tokens, sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`);
     - `mw-balp/train/response_mask.u8` (`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, `expected_train_targets=2425925` / `expected_eligible_targets=2425925`);
     - `ft/dev/tokens.u16` (`42,856` payload tokens, sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`) and `ft/dev/response_mask.u8` (`42,856` bytes, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `expected_dev_targets=14287`);
     - `ByteBpeTokenizer` `tokenizer.json` sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`;
     - `m2-fresh256` sealed artifacts (`plan.json` `122,268` B sha256 `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`, `inputs.json` `524,850` B sha256 `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`, `labels.json` `68,000` B sha256 `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`, `manifest.json` `1,384` B sha256 `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`, HF store commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`).
  3. Run `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-harness --lib preprules::tests -- --no-fail-fast`.
- **Cancel probe result (`2026-10-26`, classified `instrument`):** PASS — all commits (`2fd6358bbc52d2a202a872ad0954ebb12894606f`, `9e904a83448c451f738d5c1d45224dfdf73eb84e`, `c695ad21a72c5b91def22458eebbe796e07d5392`, `6f38dd393865a0b4320a2e28bee3be675130dc16`, `f1f55c0c1095d9b4da8457f655ac72d42d9f0180`, `0201e026b6358db4951c5a972b278e1538b198b4`) and all four mechanical pins and sealed `m2-fresh256` digests verify on `main`, and `preprules::tests` passes (`15 / 15`).

## Files in scope

- `docs/harness/prep/m2-pin-train-mask-sheet*`
- `docs/harness/milestones.json`
- `docs/labs/m2-pin-train-mask-sheet/**`
- `docs/research/notebook/m2-grounded-reply.md`

## Done when

1. The one-page discrete-law M2 pre-registration sheet `docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md` is written, carrying:
   - the response-masked TRAIN reference pins (`mw-balp/train/tokens.u16` `7,512,797` tokens sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `mw-balp/train/response_mask.u8` `7,512,797` bytes sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, `expected_train_targets=2425925`), DEV pins (`ft/dev/tokens.u16` `42,856` tokens sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `ft/dev/response_mask.u8` `42,856` bytes sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `expected_dev_targets=14287`), tokenizer pin (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`), and frozen count-reference source SHAs (`2fd6358bbc52d2a202a872ad0954ebb12894606f` and PR `#2328` merge `9e904a83448c451f738d5c1d45224dfdf73eb84e`);
   - the current discrete trainer and Stage 3 trunk in `crates/uor-r4-discrete` (`stage-t-run` from `#2836` at `c695ad21a72c5b91def22458eebbe796e07d5392` and `trunk-run` / `TrunkAssemblyV1` from `#2844` at `6f38dd393865a0b4320a2e28bee3be675130dc16`, with unchanged `driver_source_sha256` `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` and `geometry_sha256` `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`);
   - both rung-0 rank-margin bars (`top-1` and `recall@8` >= count predictor + 5 pp on `14,287` `ft/dev` rows across Seeds 1 and 2, plus fresh transfer on `fresh_v2` `14,287` rows) and the M2 grounded-reply milestone bars (`complete >= 256 / 512` on `m2-reply512` and `pairs_both_complete_distinct >= 52 / 128` with single-row `complete >= 104 / 256` on the sealed 256-row `m2-fresh256` panel, graded by the existing deterministic complete-reply evaluator unchanged);
   - all 8 required `prep.sheet_shape` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS line`) and no prohibited teacher/grader;
   - an independent fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER.md §4.5`), posted on `#2851` and `#2030`.
2. `docs/labs/m2-pin-train-mask-sheet/README.md` and `docs/harness/milestones.json` are updated to record the discrete-law M2 pre-registration sheet and its `TEST FITNESS: FIT` alongside the retained D37 sheet (`m2-pin-train-mask-sheet-2026-10-23.md`, which remains `NOT_FIT` pending `#2823` and `#2820`), while preserving `training_hold: true` and `rental_hold: true` on M2 (the sheet runs nothing and releases no hold).
3. `docs/research/notebook/m2-grounded-reply.md` records the result row for `m2-pin-train-mask-sheet` (`#2851`, plus the staged `m2-fresh256` panel row deferred from `#2849`), `docs/roadmap/items/m2-pin-train-mask-sheet.md` is set to `status: done` (`Closes #2851`), and `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any `prep.sheet_shape`, `prep.no_prohibited_teacher`, `xtask links`, or `xtask roadmap check` gate fails — fix the sheet or links immediately in-worktree and re-verify without waiting.
