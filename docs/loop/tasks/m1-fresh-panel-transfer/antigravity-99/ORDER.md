# ORDER: m1-fresh-panel-transfer, antigravity-99

Lab: antigravity. Item: [m1-fresh-panel-transfer](../../../../roadmap/items/m1-fresh-panel-transfer.md). Base: `5607dcae8`.

## Goal

Execute Steps 1–3 and 5 of `m1-fresh-panel-transfer` (`#2841`, Step 7 of `PLAN.md`):

1. **Scoring-path refusal check & gate (`Step 1`):** Audit both the legacy training-side `crates/uor-r4-discrete/src/attention_transfer.rs` gate and the D11 export scorer `docs/labs/m1-d11-artifact-scorer/instrument/` (`d11-scorer`). Write a failing test showing `d11-scorer` (`rows_from_corpus` and `dev_rows`) does not refuse a `Split::Fresh` or sealed `fresh_v2` (`0b7e754c…` / `a1cda529…`) / `K16` (`82ead8db…` / `8287d20d…`) corpus before a sealed two-seed DEV decision, then implement `refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `fresh_rows`, `fresh_rows_from_corpus`, and `d11-scorer fresh-score` so the fresh slice cannot be opened or scored without a sealed two-seed DEV bar-(a)–(d) `PASS_BARS_A_TO_D_TWO_SEEDS` decision receipt.
2. **Design note on `#2841` (`Step 2`):** Post the prospective design note on `#2841` defining "the gain" on the fresh slice (read with CHARTER §4.8 bar (d)), what is frozen before the slice opens, and who opens it.
3. **One-page pre-registration sheet, seal verification & fresh-context `TEST FITNESS: FIT` (`Step 3`):** Draft `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` registering the fresh-panel threshold and noise floor bound to the sealed bytes (`0b7e754c…`, `a1cda529…`, `df5d074a…`, `c2d4c7e5…`, `4bd108ad…`), verify the seal hashes against the stored archives (`codex-attention-dose1-fresh-v2-preservation.tar` and `admission-redirect-transfer-20261015-codex-v2.tar`) without opening or scoring the slice, and obtain a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2841`.
4. **Keep the fresh slice unopened (`Step 4`) & record (`Step 5`):** Because no saved model has passed M1 bars (a)–(d) on DEV on two seeds (`t16-frozen-zero` `455/14,287` and `t16-relation` `371/14,287` in `#3287`), keep the fresh slice sealed (`UNAVAILABLE_NOT_RUN`), write `docs/labs/m1-fresh-panel-transfer/README.md`, add one row to `docs/research/notebook/m1-discrete-attention.md`, and merge with `References #2841`.

## Cancel probe (run first)

1. Check whether `m1-d11-artifact-scorer-instrument` (`rows_from_corpus` / `dev_rows`) currently refuses `Split::Fresh` or sealed `fresh_v2` (`0b7e754c…` / `a1cda529…`) before touching disk or building rows, and verify the stored archive hashes (`codex-attention-dose1-fresh-v2-preservation.tar` and `admission-redirect-transfer-20261015-codex-v2.tar`) by streaming SHA-256 without opening or scoring the slice.
2. **Cancel probe result (executed 2026-10-26, classed `instrument`):**
   - `codex-attention-dose1-fresh-v2-preservation.tar` (`5,614,080` bytes, MD5 `318fb430c9726c9600a897eb1e3d767a`, SHA-256 `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`) stream-verifies `tokens.u16` `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` (`89,420` bytes), `response_mask.u8` `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`44,678` bytes), `report.json` `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f` (`4,047` bytes), `ordered-row-binding.private.jsonl` `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06` (`325,399` bytes), and `manifest.json` `b41b8344cd4ca162ab2ea7d6749b826041d06b660546ed69c8205a5f64caca44` (`4,098` bytes).
   - `admission-redirect-transfer-20261015-codex-v2.tar` (`3,276,288` bytes, MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, SHA-256 `004523df17711fc5eed4afa4ebadd2ce27a3fa001d3c327873fdb6cbdb3212fb`) stream-verifies `binding.json` `4bd108ad4b1ee4673f87e3b87ef89a01a563a556330fa577d06e675f6be7f460` (`4,084` bytes), `fresh/tokens.u16` `82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b` (`1,010,234` bytes), `fresh/response_mask.u8` `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4` (`505,085` bytes), and `SHA256SUMS` `044b2e444e809890037763c09ef60d22ad13b9c507ff7ce783c089068eccdeaa`.
   - `m1-d11-artifact-scorer-instrument`'s `rows_from_corpus` and `dev_rows` currently lack a refusal guard against `Split::Fresh` and the sealed fresh digests (`0b7e754c…` / `a1cda529…` / `82ead8db…` / `8287d20d…`), which we fix with a failing-first unit test in this unit.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-fresh-panel-transfer*`
- `docs/labs/m1-fresh-panel-transfer/**`
- `docs/loop/tasks/m1-fresh-panel-transfer/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/research/open-questions.md`
- `docs/start/faq.md`

## Done when

1. A failing-first unit test in `docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs` proves that `rows_from_corpus` and `dev_rows` refuse `Split::Fresh` and the sealed fresh panel hashes (`0b7e754c…`, `a1cda529…`, `82ead8db…`, `8287d20d…`), and that `validate_fresh_transfer_gate` / `fresh_rows` / `fresh_rows_from_corpus` refuse to open or score the fresh panel unless a sealed two-seed DEV bar-(a)–(d) `PASS_BARS_A_TO_D_TWO_SEEDS` decision receipt is supplied.
2. The Step 2 design note is posted on `#2841`.
3. `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` passes `prep.sheet_shape`, verifies the stored seal hashes without opening or scoring the slice, and carries a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2841`.
4. `docs/labs/m1-fresh-panel-transfer/README.md`, one row in `docs/research/notebook/m1-discrete-attention.md`, and `REPORT.md` (`References #2841`) are committed, and `uor-harness loop gates --full` is green.

## Stop and ask if

- Never stop to wait on Casey; if any bar or scope question arises, run a fresh-context D68 decision review and log `[antigravity] owner log:` on `#2841`.
- Never open, extract, or score the sealed fresh panel (`0b7e754c…` / `a1cda529…` or `82ead8db…` / `8287d20d…`) before a saved model passes M1 bars (a)–(d) on DEV across two seeds.
