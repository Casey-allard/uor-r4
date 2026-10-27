# M1 Fresh Panel Transfer — Scoring-Path Gate, Pre-Registration, Seal Verification, and Two-Seed Fresh V2 Evaluation (`m1-fresh-panel-transfer`, `#2841`)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-99` PR `#3292`, `antigravity-109` PR `#3311`, `antigravity-110` PR `#3312`, `antigravity-111` PR `#3318`)
- **Issue:** `#2841` ([`m1-fresh-panel-transfer`](../../roadmap/items/m1-fresh-panel-transfer.md), Step 7 of [`PLAN.md`](../../../PLAN.md))
- **Pre-registration sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md)
- **Scorer instrument:** [`docs/labs/m1-d11-artifact-scorer/instrument/`](../m1-d11-artifact-scorer/instrument/)
- **Committed evidence:**
  - [`evidence/trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`](evidence/trunk-joint-pointer-v1-d11-fresh-gate-receipt.json)
  - [`evidence/t16-frozen-zero-trunk-fresh-score-rposition.json`](evidence/t16-frozen-zero-trunk-fresh-score-rposition.json) (`3,594` `rposition` `P_succ` rows)
  - [`evidence/t16-frozen-zero-trunk-fresh-score.json`](evidence/t16-frozen-zero-trunk-fresh-score.json) (`4,900` `windows(2)` `P_succ` rows)
- **HF store archive:** `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/m1-close-saved-model-20261026.tar` (SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`)
- **Verdict:** **positive (`PASS_BAR_E_FRESH_TRANSFER`, `bar_e_pass = true` on both seeds; closes `#2841`)**
- **Decode & scoring throughput (`tokens/s`, `CHARTER.md` §3.12):**
  - **Apple M1 laptop (`d11-scorer toks`, 1 warmup, 20 runs, 192 prefill / 64 decode):**
    - Seed 1 (`04652a3d…`, `86,536,132` B): **`11,026 tok/s`** decode median (`2,596,805 tok/s` prefill median)
    - Seed 2 (`1b49e8bd…`, `84,943,328` B): **`10,573 tok/s`** decode median (`2,764,260 tok/s` prefill median)
  - **RTX 5090 host CPU (`d11-scorer toks`):**
    - Seed 1: **`11,017 tok/s`** decode median (`2,347,618 tok/s` prefill median)
    - Seed 2: **`10,591 tok/s`** decode median (`2,339,181 tok/s` prefill median)
  - **1x RTX 5090 GPU (`d11-scorer score`, `vast-57903382`, `sm_120`):**
    - Seed 1: **`183,206 tok/s`** eligible response (`34,625,934 tok/s` context, `77,652 us` Build A / `77,983 us` Build B)
    - Seed 2: **`181,086 tok/s`** eligible response (`34,225,309 tok/s` context, `78,896 us`)

---

## 1. Headline and Negatives First

1. **Initial instrument findings repaired prior to opening the sealed slice (`classification: instrument`):**
   - In `antigravity-99` (PR `#3292`), while the legacy training-side `crates/uor-r4-discrete/src/attention_transfer.rs` enforced a two-seed DEV `KEEP` check before opening the K16 `Split::Fresh` slice (`4bd108ad…`), the D11 integer export scorer `docs/labs/m1-d11-artifact-scorer/instrument/` (`d11-scorer rows` / `d11-scorer score` via `rows_from_corpus` and `dev_rows`) had no refusal check preventing a caller from passing a `Split::Fresh` corpus or the sealed `fresh_v2` (`0b7e754c…` / `a1cda529…`) or K16 (`82ead8db…` / `8287d20d…`) files before a sealed two-seed DEV bar-(a)–(d) decision. We added `inst::fresh_transfer` (`refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `fresh_rows`, `fresh_rows_from_corpus`, `build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`, and `d11-scorer fresh-score`) and verified that the Stage T 4-table `UORLUT01` models (`455` / `506` and `371` / `337` `< 6,300` on `ft/dev`) were refused before touching the fresh slice.
   - In `antigravity-109` (PR `#3311`), we reconciled `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` in `fresh_transfer.rs` with the canonical `mw-balp/train` (`3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`) and `ft/dev` (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`) archives and aligned `is_prefix_copy_successor` (`rposition`) with `attention_evidence::is_copy_successor` (`3,594` `P_succ` rows on `Fresh V2`).
   - In `antigravity-111` (PR `#3318`), we reconciled the remaining stale `TRAIN`/`DEV` SHA-256 strings in `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` and committed the sealed `Fresh V2` receipts into `docs/labs/m1-fresh-panel-transfer/evidence/`.
2. **Positive fresh-panel transfer result (`PASS_BAR_E_FRESH_TRANSFER`):**
   - After `trunk-joint-pointer-v1-d11` (`T16_fz` `FrozenZeroV1` Seeds `1` and `2`, exported to the 19-table `UORLUT01` `uor-r4.lut-discrete-trunk/1` container `04652a3d…` and `1b49e8bd…`) passed M1 Bars (a)–(d) on `ft/dev` (`7,002` and `7,001 / 14,287 >= 6,324`, `P_succ = 1,779` and `1,770 / 3,847 >= 1,265`, `0` export gap, `B4` pass) and `#2841` received fresh-context non-author `TEST FITNESS: FIT` and D67 pre-run approval (`#issuecomment-6432026082`), `d11-scorer fresh-score` was executed across two independent ahead-of-time CUDA builds (`Build A` `ce5843b00dee006cf89a1028f80b5125221393dbfb549b09c2273f3e6d3660f4` and `Build B` `0357abf8ca70cd463b7976eebef5ddea2dbfb7d9128ef1246246f665e25fed17`, `ptx_sha256` `5eca26218bde5a74aee90b30d5c5270bbe0146a3ad2a87cf22c8bf770b30dce7`, `cubin_sha256` `c513559a64fc7fcd83e6ee90e893bc91b2f6452d29cf4605cd8b7e3585235cd1`) on `1x RTX 5090` (`vast-57903382`):
     - **Full `Fresh V2` panel (`14,479` eligible targets, `Unigram = 1,279`, `T2 = 6,582`, `two_seed_spread_full = 13`):**
       - **Seed 1 (`04652a3d…`):** **`7,215 / 14,479`** (`49.83%`, `top-8 = 7,688`), **`+5,936` over Unigram** ($b = 5,977, c = 41$, required McNemar floor `156`), **`+633` over `T2 = 6,582`** ($b = 1,149, c = 516$, required McNemar floor `82`).
       - **Seed 2 (`1b49e8bd…`):** **`7,228 / 14,479`** (`49.92%`, `top-8 = 7,696`), **`+5,949` over Unigram** ($b = 5,990, c = 41$, required McNemar floor `156`), **`+646` over `T2 = 6,582`** ($b = 1,163, c = 517$, required McNemar floor `82`).
     - **`P_succ` copy-successor subset (`3,594` `rposition` rows, `Unigram = 229`, `T2_psucc = 1,240`, `two_seed_spread_psucc = 2`):**
       - **Seed 1:** **`1,713 / 3,594`** (`47.66%`), **`+473` over `T2_psucc = 1,240`** ($b = 569, c = 96$, required McNemar floor `52`). *(Under `windows(2).any` on `4,900` rows: `2,333 / 4,900` vs `1,863`, `+470` [$b = 585, c = 115$, floor `53`]).*
       - **Seed 2:** **`1,711 / 3,594`** (`47.61%`), **`+471` over `T2_psucc = 1,240`** ($b = 567, c = 96$, required McNemar floor `52`). *(Under `windows(2).any` on `4,900` rows: `2,329 / 4,900` vs `1,863`, `+466` [$b = 581, c = 115$, floor `53`]).*
     - **Integrity & Parity (`G-Fresh-Integrity`):** `refused_rows = 0`, `reference_mismatches = 0` (`14,479 / 14,479` exact row agreement with `crates/uor-r4-integer::IntegerDiscreteModel::observe_target`), `b4_pass = true`, `parity_ok = true`, `bar_e_pass = true`, `outcome = "PASS_BAR_E_FRESH_TRANSFER"`.

---

## 2. Two-Seed Fresh V2 Transfer Table (`d11-scorer fresh-score`)

| Gate / Metric | Seed 1 (`04652a3d…`) | Seed 2 (`1b49e8bd…`) | Bar / Floor | Status |
|---|---|---|---|---|
| **G-Fresh-Gate** (`ft/dev` `14,287` rows, `T2 = 6,300`) | `7,002 / 14,287` (`+702`), `P_succ = 1,779 / 3,847` (`+534`) | `7,001 / 14,287` (`+701`), `P_succ = 1,770 / 3,847` (`+525`) | `>= 6,324` full, `>= 1,265` `P_succ`, `B2` `0` gap, `B4` pass | **PASS** |
| **G-Fresh-Full vs Unigram** (`14,479` rows, `U_fresh = 1,279`) | **`7,215`** (`+5,936`, $b = 5,977, c = 41$) | **`7,228`** (`+5,949`, $b = 5,990, c = 41$) | `>= 156` ($\max(24, \lceil 2\sqrt{b+c}\rceil, 13)$) | **PASS** |
| **G-Fresh-Full vs `T2`** (`14,479` rows, `T2_fresh = 6,582`) | **`7,215`** (**`+633`**, $b = 1,149, c = 516$) | **`7,228`** (**`+646`**, $b = 1,163, c = 517$) | `>= 82` ($\max(24, \lceil 2\sqrt{b+c}\rceil, 13)$) | **PASS** |
| **G-Fresh-Psucc vs `T2_psucc`** (`3,594` `rposition` rows, `T2_psucc = 1,240`) | **`1,713`** (**`+473`**, $b = 569, c = 96$) | **`1,711`** (**`+471`**, $b = 567, c = 96$) | `>= 52` ($\max(20, \lceil 2\sqrt{b+c}\rceil, 2)$) | **PASS** |
| **G-Fresh-Psucc (`windows(2)` variant)** (`4,900` rows, `T2_psucc = 1,863`) | **`2,333`** (**`+470`**, $b = 585, c = 115$) | **`2,329`** (**`+466`**, $b = 581, c = 115$) | `>= 53` ($\max(20, \lceil 2\sqrt{b+c}\rceil, 4)$) | **PASS** |
| **G-Fresh-Integrity** (`14,479` rows) | `refused = 0`, `mismatches = 0`, `b4_pass = true` | `refused = 0`, `mismatches = 0`, `b4_pass = true` | `0` refused, `0` mismatches, `B4` pass | **PASS** |
| **Overall Verdict** | `seed_pass = true` | `seed_pass = true` | Both seeds pass all 4 gates | **`PASS_BAR_E_FRESH_TRANSFER`** |

---

## 3. Scoring-Path Refusal Guard and Prospective Gate (`src/fresh_transfer.rs`)

In [`docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs`](../m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs), [`src/lib.rs`](../m1-d11-artifact-scorer/instrument/src/lib.rs), [`src/main.rs`](../m1-d11-artifact-scorer/instrument/src/main.rs), and [`tests/stage1.rs`](../m1-d11-artifact-scorer/instrument/tests/stage1.rs):

- **Unconditional refusal on `rows_from_corpus` and `dev_rows` (`refuse_sealed_fresh_source`):**
  - Rejects `Split::Fresh` (`split == "fresh"`), the pinned Fresh V2 digests (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`), the pinned K16 fresh digests (`82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b`, `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4`), and the Stage 3 `cut_fresh_slice` digests (`1f8bf88d8deee51574c68ffd8976ce4415fc8aed5602213d6905a37a138517ea`, `fe87363920e19bf072e9fe3a7289e6cd2f1abde6c2431caae8f5c8494d3829d5`) with `SEALED_FRESH_PANEL_REFUSED` **before** `Corpus::load` touches disk.
- **Sealed two-seed DEV bar-(a)–(d) gate (`validate_fresh_transfer_gate`):**
  - Before `fresh_rows`, `fresh_rows_from_corpus`, or `d11-scorer fresh-score` opens any fresh corpus file, `validate_fresh_transfer_gate` validates a `FreshTransferDecisionReceipt` (`schema == "uor-r4.m1-fresh-transfer-gate/1"`, `dev_decision == "PASS_BARS_A_TO_D_TWO_SEEDS"`).
- **Frozen TRAIN count-table builder and exact integer McNemar evaluator (`build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`):**
  - Builds `U_train` and `G_train` (`T2`) strictly from the `1,049,856` matched TRAIN rows (`16,404` batches of `64` rows) of `mw-balp/train` (`3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`) via `uor_r4_discrete::w2_capacity::{build_tables, predict}`.

---

## 4. Reproducible Verification Commands

```sh
# Run the full m1-d11-artifact-scorer-instrument test suite (23 / 23 tests, including m1_fresh_panel_transfer_evidence_and_sheet_pins_pass_bar_e):
CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --no-fail-fast

# Type-check the CUDA scorer path with stub nvcc:
NVCC="$PWD/local/antigravity/fake-nvcc/nvcc" UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda --all-targets
```
