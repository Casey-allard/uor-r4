# REPORT: m1-fresh-panel-transfer, antigravity-111

- **Lab:** `antigravity` (`antigravity-111`, PR `#3318`)
- **Item:** [`m1-fresh-panel-transfer`](../../../../roadmap/items/m1-fresh-panel-transfer.md) (`#2841`, Step 7 of [`PLAN.md`](../../../../../PLAN.md))
- **Order:** [`docs/loop/tasks/m1-fresh-panel-transfer/antigravity-111/ORDER.md`](ORDER.md)
- **Decision card:** [`docs/loop/tasks/m1-fresh-panel-transfer/antigravity-111/DECISION.md`](DECISION.md)
- **Pre-registration sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md)
- **Evidence:** [`docs/labs/m1-fresh-panel-transfer/evidence/`](../../../../labs/m1-fresh-panel-transfer/evidence/) (`trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`, `t16-frozen-zero-trunk-fresh-score-rposition.json`, `t16-frozen-zero-trunk-fresh-score.json`)
- **HF store archive:** `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/m1-close-saved-model-20261026.tar` (SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`)

Closes #2841

## What did not work

1. **Stale `TRAIN`/`DEV` SHA-256 strings and `UNAVAILABLE_NOT_RUN` status in `m1-fresh-panel-transfer`'s scoped documentation (`classification: instrument`):**
   - While `antigravity-109` (PR `#3311`) updated `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` in `docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs` and `antigravity-110` (PR `#3312`) executed the single-shot GPU `d11-scorer fresh-score` evaluation on the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`) across `Build A` and `Build B` on `1x RTX 5090` (`vast-57903382`), `m1-fresh-panel-transfer`'s own scoped files (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`, `docs/labs/m1-fresh-panel-transfer/README.md`, `docs/start/faq.md:111`, and `docs/research/open-questions.md:27`) still carried `UNAVAILABLE_NOT_RUN` / `"a genuinely fresh test has not run"` and lines 60–64 of `m1-fresh-panel-transfer-2026-10-26.md` still had the pre-fix `TRAIN`/`DEV` SHA-256 strings (`3dee858fe16f…` / `0919e90f813b…`).
   - We wrote a failing-first test (`m1_fresh_panel_transfer_evidence_and_sheet_pins_pass_bar_e` in `docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`), confirmed it failed first on the missing `docs/labs/m1-fresh-panel-transfer/evidence/` files and then on the stale `3dee858fe16f…` pin in `m1-fresh-panel-transfer-2026-10-26.md`, committed the sealed `Fresh V2` receipts and reconciled the prep-sheet pins, and verified `23 / 23` `d11-scorer` tests pass.

## What changed

1. **Committed sealed `Fresh V2` transfer receipts in `docs/labs/m1-fresh-panel-transfer/evidence/` and added regression test (`tests/stage1.rs`):**
   - Added [`trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`](../../../../labs/m1-fresh-panel-transfer/evidence/trunk-joint-pointer-v1-d11-fresh-gate-receipt.json), [`t16-frozen-zero-trunk-fresh-score-rposition.json`](../../../../labs/m1-fresh-panel-transfer/evidence/t16-frozen-zero-trunk-fresh-score-rposition.json), and [`t16-frozen-zero-trunk-fresh-score.json`](../../../../labs/m1-fresh-panel-transfer/evidence/t16-frozen-zero-trunk-fresh-score.json) to `docs/labs/m1-fresh-panel-transfer/evidence/`.
   - Added `m1_fresh_panel_transfer_evidence_and_sheet_pins_pass_bar_e` to [`docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs).
2. **Updated pre-registration sheet, lab report, FAQ, open questions, notebook, and roadmap (`status: done`):**
   - Updated [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md), [`docs/labs/m1-fresh-panel-transfer/README.md`](../../../../labs/m1-fresh-panel-transfer/README.md), [`docs/start/faq.md`](../../../../start/faq.md), [`docs/research/open-questions.md`](../../../../research/open-questions.md), [`docs/research/notebook/m1-discrete-attention.md`](../../../../research/notebook/m1-discrete-attention.md), [`docs/roadmap/items/m1-fresh-panel-transfer.md`](../../../../roadmap/items/m1-fresh-panel-transfer.md) (`status: done`), and [`docs/roadmap/README.md`](../../../../roadmap/README.md).

## Measurements

- **Serving & GPU throughput (`d11-scorer toks` / `score`, `CHARTER.md` §3.12):**
  - Apple M1 laptop (`d11-scorer toks`, 1 warmup, 20 runs, 192 prefill / 64 decode): Seed 1 (`04652a3d…`, `86,536,132` B) **`11,026 tok/s`** decode median (`2,596,805 tok/s` prefill median); Seed 2 (`1b49e8bd…`, `84,943,328` B) **`10,573 tok/s`** decode median (`2,764,260 tok/s` prefill median).
  - RTX 5090 host CPU (`d11-scorer toks`): Seed 1 **`11,017 tok/s`** decode median (`2,347,618 tok/s` prefill median); Seed 2 **`10,591 tok/s`** decode median (`2,339,181 tok/s` prefill median).
  - 1x RTX 5090 GPU (`d11-scorer score`, `vast-57903382`): Seed 1 **`183,206 tok/s`** response (`34,625,934 tok/s` context, `77,652 us` Build A / `77,983 us` Build B); Seed 2 **`181,086 tok/s`** response (`34,225,309 tok/s` context, `78,896 us`).
- **Two-seed `Fresh V2` transfer (`0b7e754c…` / `a1cda529…`, `14,479` eligible targets, `3,594` `rposition` `P_succ` rows; `Build A` `ce5843b0…` == `Build B` `0357abf8…`):**
  - `G-Fresh-Gate` (`ft/dev` `14,287` rows): `d11_dev_top1 = 7,002` / `7,001 >= 6,324`, `psucc_d11_top1 = 1,779` / `1,770 >= 1,265`, `0` export gap, `B4` pass (**PASS**).
  - `G-Fresh-Full` (`14,479` rows, `U_fresh = 1,279`, `T2_fresh = 6,582`, `two_seed_spread_full = 13`):
    - Seed 1: **`7,215 / 14,479`** (`49.83%`, `top-8 = 7,688`), **`+5,936` vs `U_fresh`** ($b = 5,977, c = 41$, floor `156`), **`+633` vs `T2_fresh`** ($b = 1,149, c = 516$, floor `82`) (**PASS**).
    - Seed 2: **`7,228 / 14,479`** (`49.92%`, `top-8 = 7,696`), **`+5,949` vs `U_fresh`** ($b = 5,990, c = 41$, floor `156`), **`+646` vs `T2_fresh`** ($b = 1,163, c = 517$, floor `82`) (**PASS**).
  - `G-Fresh-Psucc` (`3,594` `rposition` `P_succ` rows, `U_psucc = 229`, `T2_psucc = 1,240`, `two_seed_spread_psucc = 2`):
    - Seed 1: **`1,713 / 3,594`** (`47.66%`), **`+473` vs `T2_psucc`** ($b = 569, c = 96$, floor `52`) (**PASS**; `windows(2)` `4,900` rows: `2,333` vs `1,863`, `+470`, floor `53`).
    - Seed 2: **`1,711 / 3,594`** (`47.61%`), **`+471` vs `T2_psucc`** ($b = 567, c = 96$, floor `52`) (**PASS**; `windows(2)` `4,900` rows: `2,329` vs `1,863`, `+466`, floor `53`).
  - `G-Fresh-Integrity`: `refused_rows = 0`, `reference_mismatches = 0`, `b4_pass = true` (**PASS**).
  - Outcome: **`PASS_BAR_E_FRESH_TRANSFER`** (`bar_e_pass = true`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --no-fail-fast`: **`23 / 23` passed** (`3` `m1_close`, `10` `parity`, `7` `stage1`, `3` `toks`; `1` ignored `the_registered_dev_panel_has_14287_rows`).
- `NVCC="$PWD/local/antigravity/fake-nvcc/nvcc" UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda --all-targets`: **PASS**.
- `uor-harness loop gates --full`: **PASS**.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`, `context-guard`

## Still open

- None for `#2841` (`m1-fresh-panel-transfer` is `status: done`).

## Plain summary

Recorded the sealed two-seed `Fresh V2` (`0b7e754c…` / `a1cda529…`) GPU `d11-scorer fresh-score` transfer results (`7,215` and `7,228 / 14,479` vs `6,582` `T2`, `P_succ = 1,713` and `1,711 / 3,594` vs `1,240`, `0` export gap, `11,026` / `10,573 tok/s` laptop decode, `183,206` / `181,086 tok/s` RTX 5090 response, `PASS_BAR_E_FRESH_TRANSFER`) in `docs/labs/m1-fresh-panel-transfer/`, reconciled the prep-sheet `TRAIN`/`DEV` SHA-256 pins, added a regression test in `tests/stage1.rs`, and closed `#2841` (`status: done`).
