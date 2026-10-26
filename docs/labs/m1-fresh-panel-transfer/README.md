# M1 Fresh Panel Transfer — Scoring-Path Gate, Pre-Registration, and Seal Verification (`m1-fresh-panel-transfer`, `#2841`)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-99`, PR `#3292`)
- **Issue:** `#2841` ([`m1-fresh-panel-transfer`](../../roadmap/items/m1-fresh-panel-transfer.md), Step 7 of [`PLAN.md`](../../../PLAN.md))
- **Pre-registration sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md)
- **Scorer instrument:** [`docs/labs/m1-d11-artifact-scorer/instrument/`](../m1-d11-artifact-scorer/instrument/)
- **Verdict:** **positive (instrument gate, seal verification, and pre-registration complete; fresh slice remains sealed `UNAVAILABLE_NOT_RUN` until a saved model passes M1 bars (a)–(d) on DEV across two seeds)**
- **Decode throughput (`tokens/s`):** Held on a DEV bar-(a)–(d)-passing model (`UNAVAILABLE_NOT_RUN` on `fresh_v2`; the underlying `d11-scorer` instrument scores at `232,126–232,879` eligible response `tokens/s` and `710,362–712,665` payload `tokens/s` on 1x RTX 5090 in [`m1-export-gap-and-toks`](../m1-d11-artifact-scorer/README.md)).

---

## 1. Headline and Negatives First

1. **No saved discrete model has yet passed M1 bars (a)–(d) on `ft/dev` across two seeds, so the fresh panel remains sealed (`UNAVAILABLE_NOT_RUN`):**
   - In [`m1-export-gap-and-toks`](../m1-d11-artifact-scorer/README.md) (`#2839`, PR `#3287`), the exported Stage T checkpoints scored `455 / 14,287` (`t16-frozen-zero` seed 1), `506 / 14,287` (`t16-frozen-zero` seed 2), `371 / 14,287` (`t16-relation` seed 1), and `337 / 14,287` (`t16-relation` seed 2) on `ft/dev` through `UORLUT01` at exact zero export gap (`14,287 / 14,287` row agreement), below the DEV unigram table (`1,226 / 14,287`) and DEV `T2` successor table (`6,300 / 14,287`).
   - Under Step 4 of [`m1-fresh-panel-transfer.md`](../../roadmap/items/m1-fresh-panel-transfer.md) and `CHARTER.md` §4.3 / §4.8, the sealed fresh panel (`0b7e754c…` / `a1cda529…`) must not be opened or scored until `m1-export-gap-and-toks` produces a saved model that passes M1 bars (a)–(d) on `ft/dev` across two seeds.
2. **Instrument finding fixed in this unit (`classification: instrument`):**
   - While the legacy training-side `crates/uor-r4-discrete/src/attention_transfer.rs` enforced a two-seed DEV `KEEP` check before opening the K16 `Split::Fresh` slice (`4bd108ad…`), the D11 integer export scorer `docs/labs/m1-d11-artifact-scorer/instrument/` (`d11-scorer rows` / `d11-scorer score` via `rows_from_corpus` and `dev_rows`) had no refusal check preventing a caller from passing a `Split::Fresh` corpus or the sealed `fresh_v2` (`0b7e754c…` / `a1cda529…`) or K16 (`82ead8db…` / `8287d20d…`) files before a sealed two-seed DEV bar-(a)–(d) decision.
   - We wrote a failing-first test (`dev_rows_and_rows_from_corpus_refuse_sealed_fresh_panel_before_sealed_dev_decision` in `docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`), verified it failed when the guard was absent (`rows_from_corpus` returned 5 rows on a `Split::Fresh` corpus instead of refusing), and implemented `inst::fresh_transfer` (`refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `fresh_rows`, `fresh_rows_from_corpus`, `build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`, and `d11-scorer fresh-score`).

---

## 2. Step 1 — Scoring-Path Refusal Check and Gated `d11-scorer fresh-score`

In [`docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs`](../m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs), [`src/lib.rs`](../m1-d11-artifact-scorer/instrument/src/lib.rs), [`src/main.rs`](../m1-d11-artifact-scorer/instrument/src/main.rs), and [`tests/stage1.rs`](../m1-d11-artifact-scorer/instrument/tests/stage1.rs):

- **Unconditional refusal on `rows_from_corpus` and `dev_rows` (`refuse_sealed_fresh_source`):**
  - Rejects `Split::Fresh` (`split == "fresh"`), the pinned Fresh V2 digests (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`), and the pinned K16 fresh digests (`82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b`, `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4`) with `SEALED_FRESH_PANEL_REFUSED` **before** `Corpus::load` touches disk.
- **Sealed two-seed DEV bar-(a)–(d) gate (`validate_fresh_transfer_gate`):**
  - Before `fresh_rows`, `fresh_rows_from_corpus`, or `d11-scorer fresh-score` opens any fresh corpus file, `validate_fresh_transfer_gate` validates a `FreshTransferDecisionReceipt` (`schema == "uor-r4.m1-fresh-transfer-gate/1"`, `dev_decision == "PASS_BARS_A_TO_D_TWO_SEEDS"`):
    1. Both `seed = 1` and `seed = 2` scored `14,287` eligible rows on `ft/dev` with `d11_top1 > 1,226` (DEV unigram) and `d11_top1 >= 6,300 + max(24, |d11_top1(1) - d11_top1(2)|)` (DEV `T2` bigram plus the two-seed noise floor);
    2. Both seeds passed `export_gap_b2_pass` (`14,287 / 14,287` exact row agreement between GPU `d11-scorer` and training-side `eval-corpus`), `planted_fault_b4_pass`, and `tokens_per_s_reported`;
    3. Both seeds carry distinct 64-hex `artifact_sha256` pins, and `fit_review_url` and `prerun_review_url` are recorded.
- **Frozen TRAIN count-table builder and exact integer McNemar evaluator (`build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`):**
  - Builds `U_train` (unconditional target frequency) and `G_train` (`T2` query-to-target successor frequency) strictly from the `1,049,856` matched TRAIN rows (`16,404` batches of `64` rows) of `mw-balp/train` (`3dee858f…` / `b5e7352d…`) via `uor_r4_discrete::w2_capacity::{build_tables, predict}` after refusing any non-`Split::Train` corpus or non-matching SHA-256.
  - Computes paired hits and discordant counts `(b, c)` on all `14,479` eligible fresh rows and on the `3,594` `P_succ` copy-successor rows (`is_prefix_copy_successor`), evaluating `ceil(2 * sqrt(b + c))` in pure integer arithmetic (`k * k >= 4 * (b + c)`).

---

## 3. Step 2 & Step 3 — Design Note, Seal Verification Without Opening, and Pre-Registration

1. **Design note posted on `#2841`:** `#issuecomment-6423580146` defines the gain on the fresh slice (`UORLUT01` integer top-1 beating frozen `U_train` and `G_train` on both seeds across all `14,479` fresh targets and on the `3,594` `P_succ` subset), lists all frozen pins (`0b7e754c…`, `a1cda529…`, `df5d074a…`, `c2d4c7e5…`, `4bd108ad…`, `3dee858f…`, `b5e7352d…`), and specifies the single-shot opening protocol (`d11-scorer fresh-score`).
2. **Stream seal verification without opening or scoring row contents:**
   - `icloud:UOR-R4/results/codex/codex-attention-dose1-fresh-v2-preservation.tar` (`5,614,080` bytes, MD5 `318fb430c9726c9600a897eb1e3d767a`, SHA-256 `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`):
     - `tokens.u16` (`89,420` bytes = 64-byte `UORT` header + `44,678 * 2` payload bytes across `400` documents with `0` TRAIN/DEV document overlap): SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`
     - `response_mask.u8` (`44,678` bytes, `14,479` eligible targets, `3,594` `P_succ` rows): SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`
     - `report.json` (`4,047` bytes): SHA-256 `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`
     - `ordered-row-binding.private.jsonl` (`325,399` bytes): SHA-256 `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`
     - `manifest.json` (`4,098` bytes): SHA-256 `b41b8344cd4ca162ab2ea7d6749b826041d06b660546ed69c8205a5f64caca44`
   - `icloud:UOR-R4/results/codex/admission-redirect-transfer-20261015-codex-v2.tar` (`3,276,288` bytes, MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, SHA-256 `004523df17711fc5eed4afa4ebadd2ce27a3fa001d3c327873fdb6cbdb3212fb`):
     - `binding.json` (`4,084` bytes): SHA-256 `4bd108ad4b1ee4673f87e3b87ef89a01a563a556330fa577d06e675f6be7f460`
     - `fresh/tokens.u16` (`1,010,234` bytes): SHA-256 `82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b`
     - `fresh/response_mask.u8` (`505,085` bytes): SHA-256 `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4`
     - `SHA256SUMS`: SHA-256 `044b2e444e809890037763c09ef60d22ad13b9c507ff7ce783c089068eccdeaa`
3. **One-page pre-registration sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md) registers `G-Fresh-Gate`, `G-Fresh-Full`, `G-Fresh-Psucc`, and `G-Fresh-Integrity` with non-author `TEST FITNESS: FIT` and D68 decision review (`DECISION REVIEW: APPROVE`, subagent `27655669-ec58-40ba-bcd0-b66880e8f91e`) logged on `#2841` (`#issuecomment-6423732020`).

---

## 4. Reproducible Verification Commands

```sh
# Run the full m1-d11-artifact-scorer-instrument test suite (18 / 18 tests, including the fresh-panel refusal and gate tests):
CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --no-fail-fast

# Type-check the CUDA scorer path with stub nvcc:
NVCC="$PWD/local/antigravity/fake-nvcc/nvcc" UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda --all-targets
```
