# REPORT: m1-fresh-panel-transfer, antigravity-99

Lab: antigravity. Item: [m1-fresh-panel-transfer](../../../../roadmap/items/m1-fresh-panel-transfer.md). Order: [ORDER.md](ORDER.md).

References #2841

## What did not work

- **No saved discrete model has yet passed M1 bars (a)–(d) on `ft/dev` across two seeds, so the fresh sealed panel remains unopened (`UNAVAILABLE_NOT_RUN`):**
  - In [`m1-export-gap-and-toks`](../../../labs/m1-d11-artifact-scorer/README.md) (`#2839`, PR `#3287`), the exported Stage T checkpoints scored `455 / 14,287` (`t16-frozen-zero` seed 1), `506 / 14,287` (`t16-frozen-zero` seed 2), `371 / 14,287` (`t16-relation` seed 1), and `337 / 14,287` (`t16-relation` seed 2) on `ft/dev` through `UORLUT01` at exact zero export gap (`14,287 / 14,287` row agreement), below the DEV unigram table (`1,226 / 14,287`) and DEV `T2` successor table (`6,300 / 14,287`).
  - Per Step 4 of [`m1-fresh-panel-transfer.md`](../../../../roadmap/items/m1-fresh-panel-transfer.md) and `CHARTER.md` §4.3 / §4.8, the sealed fresh panel (`0b7e754c…` / `a1cda529…`) is not opened or scored until `m1-export-gap-and-toks` produces a saved model that passes M1 bars (a)–(d) on `ft/dev` across two seeds (`UNAVAILABLE_NOT_RUN`).
- **Instrument gap found and fixed (`classification: instrument`):**
  - While the legacy training-side `crates/uor-r4-discrete/src/attention_transfer.rs` enforced a two-seed DEV `KEEP` check before opening the K16 `Split::Fresh` slice (`4bd108ad…`), the D11 integer export scorer `docs/labs/m1-d11-artifact-scorer/instrument/` (`d11-scorer rows` / `d11-scorer score` via `rows_from_corpus` and `dev_rows`) had no refusal check preventing a caller from passing a `Split::Fresh` corpus or the sealed `fresh_v2` (`0b7e754c…` / `a1cda529…`) or K16 (`82ead8db…` / `8287d20d…`) files before a sealed two-seed DEV bar-(a)–(d) decision.
  - Our failing-first test (`dev_rows_and_rows_from_corpus_refuse_sealed_fresh_panel_before_sealed_dev_decision` in [`docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs)) failed before the fix (`rows_from_corpus` returned 5 rows on a `Split::Fresh` corpus instead of refusing) and passed once `refuse_sealed_fresh_source` and `validate_fresh_transfer_gate` were wired in.

## What changed

- **Step 1 — Scoring-path refusal guard and gated `d11-scorer fresh-score` ([`docs/labs/m1-d11-artifact-scorer/instrument/`](../../../labs/m1-d11-artifact-scorer/instrument/)):**
  - Added [`src/fresh_transfer.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs) and updated [`src/lib.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/lib.rs), [`src/main.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/main.rs), and [`tests/stage1.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs).
  - `refuse_sealed_fresh_source`: unconditionally refuses `Split::Fresh`, the pinned Fresh V2 digests (`0b7e754c…`, `a1cda529…`), and the pinned K16 fresh digests (`82ead8db…`, `8287d20d…`) on `rows_from_corpus` and `dev_rows` (`d11-scorer rows` / `d11-scorer score`) with `SEALED_FRESH_PANEL_REFUSED` before `PreparedCorpus::load` touches disk.
  - `validate_fresh_transfer_gate`, `fresh_rows_from_corpus`, `fresh_rows`, and `d11-scorer fresh-score SEED1_LUT SEED2_LUT FRESH_CONFIG_JSON DECISION_RECEIPT_JSON REPORT_JSON`: requires a two-seed DEV decision receipt (`schema: "uor-r4.m1-fresh-transfer-gate/1"`, `dev_decision: "PASS_BARS_A_TO_D_TWO_SEEDS"`) proving that `seed = 1` and `seed = 2` both beat DEV unigram (`1,226 / 14,287`) and DEV `T2` (`6,300 / 14,287`) by `>= max(24, |d11_top1(1) - d11_top1(2)|)` with `export_gap_b2_pass == true`, `planted_fault_b4_pass == true`, `tokens_per_s_reported == true`, distinct 64-hex `artifact_sha256` pins, and posted `#2841` FIT and D67 pre-run review URLs before opening any fresh file.
  - `build_train_count_tables`, `score_fresh_against_count_tables`, `mcnemar_floor`, and `evaluate_fresh_transfer_two_seeds`: pure integer count-table builder (`U_train` unigram and `G_train` `T2` bigram from the `1,049,856` matched TRAIN rows of `mw-balp/train` `3dee858f…` / `b5e7352d…`) and exact integer McNemar floor evaluator (`k * k >= 4 * (b + c)`).
- **Step 2 — Design note on `#2841`:**
  - Posted `#issuecomment-6423580146` defining "the gain" on the fresh slice (`UORLUT01` integer top-1 beating frozen `U_train` and `G_train` on both seeds across all `14,479` fresh targets and on the `3,594` `P_succ` subset), what is frozen before the slice opens, and who opens it (`d11-scorer fresh-score`).
- **Step 3 — Pre-registration sheet, stream seal verification, and non-author `TEST FITNESS: FIT` + D68 review:**
  - Authored [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md), verified the stored seal hashes by streaming SHA-256 without extracting or scoring any row, and logged non-author `TEST FITNESS: FIT` and D68 `DECISION REVIEW: APPROVE` on `#2841` (`#issuecomment-6423732020`).
- **Step 5 — Lab note and notebook row:**
  - Created [`docs/labs/m1-fresh-panel-transfer/README.md`](../../../labs/m1-fresh-panel-transfer/README.md) and added one row to [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md).

## Measurements

- **Decode throughput (`tokens/s`):** Held on a DEV bar-(a)–(d)-passing model (`UNAVAILABLE_NOT_RUN` on `fresh_v2`; the underlying `d11-scorer` instrument scores at `232,126–232,879` eligible response `tokens/s` and `710,362–712,665` payload `tokens/s` on 1x RTX 5090 in [`m1-export-gap-and-toks`](../../../labs/m1-d11-artifact-scorer/README.md)).
- **Stream seal verification without opening or scoring row contents (`$0` GPU spend):**
  - `codex-attention-dose1-fresh-v2-preservation.tar` (`5,614,080` B, MD5 `318fb430c9726c9600a897eb1e3d767a`, SHA-256 `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`):
    - `tokens.u16` (`89,420` B, `44,678` payload tokens): `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`
    - `response_mask.u8` (`44,678` B, `14,479` eligible targets, `3,594` `P_succ` rows): `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`
    - `report.json` (`4,047` B): `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`
    - `ordered-row-binding.private.jsonl` (`325,399` B): `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`
    - `manifest.json` (`4,098` B): `b41b8344cd4ca162ab2ea7d6749b826041d06b660546ed69c8205a5f64caca44`
  - `admission-redirect-transfer-20261015-codex-v2.tar` (`3,276,288` B, MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, SHA-256 `004523df17711fc5eed4afa4ebadd2ce27a3fa001d3c327873fdb6cbdb3212fb`):
    - `binding.json` (`4,084` B): `4bd108ad4b1ee4673f87e3b87ef89a01a563a556330fa577d06e675f6be7f460`
    - `fresh/tokens.u16` (`1,010,234` B): `82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b`
    - `fresh/response_mask.u8` (`505,085` B, `2,000` eligible targets): `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4`

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --no-fail-fast`: `18 / 18` passed (`10` `parity`, `5` `stage1`, `3` `toks`).
- `NVCC="$PWD/local/antigravity/fake-nvcc/nvcc" UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda --all-targets`: `exit 0`.
- `uor-harness loop report --check --order docs/loop/tasks/m1-fresh-panel-transfer/antigravity-99/ORDER.md`: `PASS`.
- `uor-harness loop gates --full`: `11 / 11` gates `GREEN`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`.

## Still open

- Running the single-shot GPU fresh-panel transfer evaluation (`d11-scorer fresh-score` on `fresh_v2` `0b7e754c…` / `a1cda529…`) once `m1-bvg-k-stage-t-training` (`#2836`) and `m1-export-gap-and-toks` (`#2839`) produce a saved discrete model that passes M1 bars (a)–(d) on `ft/dev` across two seeds, and closing `#2841` when that fresh transfer number is recorded.

## Plain summary

We hardened the D11 integer export scorer (`d11-scorer`) so it refuses to open or score the sealed fresh panel (`fresh_v2` `0b7e754c…` / `a1cda529…` or K16 `82ead8db…` / `8287d20d…`) unless presented with a two-seed DEV decision receipt proving that a saved model has passed M1 bars (a)–(d) on `ft/dev` (`14,287` rows), verified the stored fresh archive hashes by streaming SHA-256 without opening or scoring any row, and registered the M1 bar (e) fresh-panel transfer pre-registration sheet (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`) with non-author `TEST FITNESS: FIT` and D68 decision review on `#2841`.
