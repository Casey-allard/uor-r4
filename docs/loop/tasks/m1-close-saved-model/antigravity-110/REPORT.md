# REPORT: m1-close-saved-model, antigravity-110

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Order: [`ORDER.md`](ORDER.md). Decision: [`DECISION.md`](DECISION.md). Issue: `#2837` (Milestone M1 `#2029`).

Closes #2837

## What did not work

- **Three instrument defects were caught and repaired with failing-first tests (`classification: instrument`, not mechanism negatives):**
  1. `export_and_verify_trunk_artifact` in [`crates/uor-r4-discrete/src/trunk.rs`](../../../../../crates/uor-r4-discrete/src/trunk.rs) originally called 4-table `export_d11(&base_snap)` instead of 19-table `export_trunk_d11(&base_snap, step_table, norm_frames, tables)`. Repaired and verified by round-trip unit test (`trunk_d11_19_table_container_roundtrips_with_zero_export_gap_and_refuses_corruption`).
  2. `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` in [`src/fresh_transfer.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs) carried hand-expanded trailing hex chars (`c9d0…`, `b94a…`, `4c8f…`, `7e8a…`) instead of the exact digests from `report.json` (`df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`) and `w2-inputs/2026-10-25/driver-data/` (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`), and `fresh_rows` in [`src/lib.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/src/lib.rs) called `config.validate()` without normalizing `Split::Fresh` to `Split::Dev`. Repaired with failing-first tests in [`tests/stage1.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs).
  3. `is_prefix_copy_successor` in [`src/fresh_transfer.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs) originally used `prefix.windows(2).any(|w| w[0] == query)` (any occurrence of `query` in `prefix`, counting `5,182` `ft/dev` rows and `4,900` `Fresh V2` rows) rather than `prefix[..prefix.len() - 1].iter().rposition(|&tok| tok == query)` (`attention_evidence::is_copy_successor`, most recent occurrence of `query` in `prefix`, counting the registered `3,847` `ft/dev` `P_succ` rows and `3,594` `Fresh V2` `P_succ` rows). Repaired with a failing-first test in [`tests/stage1.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs); both definitions (`3,594` `rposition` rows: `1,713` / `1,711` vs `T2_psucc = 1,240`; `4,900` `windows(2)` rows: `2,333` / `2,329` vs `1,863`) pass the `P_succ` McNemar bar by `+473` / `+471` and `+470` / `+466` rows.
- **4-table `UORLUT01` Stage T baseline negative (`classification: mechanism` configuration bound on 4-table `UORLUT01`, never the mechanism):** As recorded in `antigravity-105`, the 4-table Stage T exports (`stage-t-t16-fz` `8a2c3cb4…` / `33ce2c55…` scoring `455 / 14,287`, and `stage-t-t16-rel` `166403ca…` / `3e52357b…` scoring `371 / 14,287`) pass Bars (a), (b), and (c) but fail Bar (d) (`< 6,324`) and are refused by `validate_fresh_transfer_gate` (`FAIL_BAR_D_HOLD_FRESH_SEAL`).

## What changed

1. **19-table `export_and_verify_trunk_artifact` and `d11-scorer export-trunk` (`crates/uor-r4-discrete/src/trunk.rs`, `docs/labs/m1-d11-artifact-scorer/instrument/src/{lib,main}.rs`):**
   - Updated `export_and_verify_trunk_artifact` (`trunk.rs`) to emit and round-trip verify the 19-table `uor-r4.lut-discrete-trunk/1` container (`export_trunk_d11` / `import_trunk_d11`).
   - Added `export_trunk_with_scorer` (`src/lib.rs`) and `d11-scorer export-trunk CHECKPOINT_JSON CONFIG_JSON OUT_LUT REPORT_JSON` (`src/main.rs`) to verify `Gate R` on `ft/dev` (`455` top-1, `1,484` top-8, `5,009` admitted, `4,067` distinct read codes, `0` mismatches vs `IntegerDiscreteModel`) and `mw-balp/train` (`16,404` batches = `1,049,856` rows, `19,385` distinct read codes, `1,266` distinct query tokens) on the GPU before building `TrunkTables` and exporting the 19-table `.d11` artifact.
2. **Exact `fresh_transfer.rs` digest pins, `fresh_rows` `Split::Fresh` normalization, and `rposition` `is_prefix_copy_successor` (`docs/labs/m1-d11-artifact-scorer/instrument/src/{fresh_transfer,lib}.rs`, `tests/stage1.rs`):**
   - Pinned exact `DEV_TOKENS_SHA256`, `DEV_MASK_SHA256`, `TRAIN_TOKENS_SHA256`, and `TRAIN_MASK_SHA256` in `src/fresh_transfer.rs`, normalized `Split::Fresh` to `Split::Dev` inside `fresh_rows` after `validate_fresh_transfer_gate` passes, and aligned `is_prefix_copy_successor` with `attention_evidence::is_copy_successor` (`rposition`).
3. **Two-build GPU execution on `1x RTX 5090` (`vast-57903382`) and five-bar M1 closing audit (`src/m1_close.rs`, `tests/m1_close.rs`, `docs/labs/m1-close-saved-model/**`, `docs/harness/prep/m1-close-saved-model-2026-10-26.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-close-saved-model.md`, `docs/roadmap/README.md`):**
   - Executed `d11-scorer export-trunk`, `d11-scorer score --reference`, `d11-scorer fresh-score`, and `d11-scorer toks` across two independent ahead-of-time CUDA builds (`Build A` `ce5843b00dee006cf89a1028f80b5125221393dbfb549b09c2273f3e6d3660f4` and `Build B` `0357abf8ca70cd463b7976eebef5ddea2dbfb7d9128ef1246246f665e25fed17`, `ptx_sha256` `5eca26218bde5a74aee90b30d5c5270bbe0146a3ad2a87cf22c8bf770b30dce7`, `cubin_sha256` `c513559a64fc7fcd83e6ee90e893bc91b2f6452d29cf4605cd8b7e3585235cd1`) on `vast-57903382`, archived all 40 output files to `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs` (`m1-close-saved-model-20261026.tar` SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`), tore down the pod, ran laptop `d11-scorer toks` on Apple M1, updated `m1-close-grade.json` (`m1_close_pass: true`, `m1_milestone_status: "CLOSED_PASS"`), and transitioned `docs/roadmap/items/m1-close-saved-model.md` to `status: done`.

## Measurements

- **Headline throughput (`CHARTER.md` §3.12):**
  - **Apple M1 laptop (`d11-scorer toks`, 1 warmup, 20 runs, 192-token prefill, 64-token decode):**
    - Seed 1 (`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, `86,536,132` B): decode median **`11,026 tok/s`** (`p95 = 10,487 tok/s`), prefill median **`2,596,805 tok/s`** (`p95 = 1,226,507 tok/s`).
    - Seed 2 (`1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`, `84,943,328` B): decode median **`10,573 tok/s`** (`p95 = 10,438 tok/s`), prefill median **`2,764,260 tok/s`** (`p95 = 2,533,248 tok/s`).
  - **RTX 5090 pod host CPU (`d11-scorer toks`):** Seed 1 decode median **`11,017 tok/s`** (`2,347,618 tok/s` prefill median); Seed 2 decode median **`10,591 tok/s`** (`2,339,181 tok/s` prefill median).
  - **RTX 5090 GPU (`d11-scorer score` over `14,287` `ft/dev` rows, `compute_120`):** Seed 1 **`183,206 tok/s`** response (**`34,625,934 tok/s`** context, `77,652 us` Build A / `77,983 us` Build B); Seed 2 **`181,086 tok/s`** response (**`34,225,309 tok/s`** context, `78,896 us`).
- **Five M1 closing bars (a)–(e) (`m1-close-grade.json`, `PASS_M1_ALL_FIVE_BARS` across `Build A` and `Build B`):**
  1. **Bar (a) (`bar_a_d11_self_contained = true`):** Seed 1 (`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, `86,536,132` B) and Seed 2 (`1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`, `84,943,328` B) accepted by clean-room `crates/uor-r4-integer::IntegerDiscreteModel::from_bytes` with zero side-tables.
  2. **Bar (b) (`bar_b_export_gap_and_b4 = true`):** `14,287 / 14,287` exact agreement (`0` export gap, `refused = 0`) against `IntegerDiscreteModel::observe_target` and `planted_fault_control` (`B4`) pass (`1,260` / `1,262` cell flips, `656` / `682` route/gate flips; `G-Adm` `5,009 >= 4,894` base / `9,012` & `9,010` post-rotation; `G-Occ` `256 / 256` occupied cells).
  3. **Bar (c) (`bar_c_tokens_per_s_reported = true`):** Positive serving `tokens/s` on laptop (`11,026` / `10,573 tok/s` decode), pod CPU (`11,017` / `10,591 tok/s` decode), and GPU (`183,206` / `181,086` response `tok/s`).
  4. **Bar (d) (`bar_d_dev_beats_unigram_and_t2_two_seeds = true`):** On `ft/dev` (`14,287` rows, `Unigram = 1,226`, `T2 = 6,300`, target `>= 6,324`), Seed 1 scores **`7,002 / 14,287`** (`49.01%`, **`+702` over `T2`**, `top-8 = 7,485`, `admitted = 9,012`) and Seed 2 scores **`7,001 / 14,287`** (`49.00%`, **`+701` over `T2`**, `top-8 = 7,478`, `admitted = 9,010`). On `ft/dev` `P_succ` (`3,847` rows, `Unigram = 194`, `T2_psucc = 1,245`, target `>= 1,265`), Seed 1 scores **`1,779 / 3,847`** (`46.24%`, **`+534` over `T2_psucc`**, `top-8 = 1,906`) and Seed 2 scores **`1,770 / 3,847`** (`46.01%`, **`+525` over `T2_psucc`**, `top-8 = 1,899`).
  5. **Bar (e) (`bar_e_fresh_transfer_two_seeds = true`, `#2841`):** On sealed `Fresh V2` (`0b7e754c…` / `a1cda529…`, `14,479` rows, `3,594` `P_succ` rows; `Unigram = 1,279`, `T2 = 6,582`, `T2_psucc = 1,240`), Seed 1 scores **`7,215 / 14,479`** (`49.83%`, **`+5,936` over Unigram** vs floor `156` [$b = 5,977, c = 41$]; **`+633` over `T2`** vs floor `82` [$b = 1,149, c = 516, \text{spread} = 13$], `top-8 = 7,688`) and **`1,713 / 3,594`** on `P_succ` (`47.66%`, **`+473` over `T2_psucc`** vs floor `52` [$b = 569, c = 96, \text{spread} = 2$]); Seed 2 scores **`7,228 / 14,479`** (`49.92%`, **`+5,949` over Unigram** vs floor `156` [$b = 5,990, c = 41$]; **`+646` over `T2`** vs floor `82` [$b = 1,163, c = 517, \text{spread} = 13$], `top-8 = 7,696`) and **`1,711 / 3,594`** on `P_succ` (`47.61%`, **`+471` over `T2_psucc`** vs floor `52` [$b = 567, c = 96, \text{spread} = 2$]), with `0` reference mismatches and `B4` pass across both seeds (`PASS_BAR_E_FRESH_TRANSFER`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --target aarch64-apple-darwin --no-fail-fast`: **`22 / 22` passed** (`3` `m1_close`, `10` `parity`, `6` `stage1`, `3` `toks`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-discrete --target aarch64-apple-darwin --no-fail-fast`: **passed**.
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/antigravity-110/ORDER.md`: **`OK`**.
- `uor-harness loop pr-check --local`: **`OK`**.
- `uor-harness loop gates --full`: **`11 / 11 GREEN`**.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`.

## Still open

- None for `m1-close-saved-model` (`#2837`); next unit closes `m1-fresh-panel-transfer` (`#2841`) with the sealed `Fresh V2` transfer evidence and advances to Milestone M2 (`#2030`).

## Plain summary

Across two independent ahead-of-time CUDA builds (`Build A` and `Build B`) on `1x RTX 5090` (`vast-57903382`) and Apple M1 laptop serving (`11,026` / `10,573 tok/s` decode median, `183,206` / `181,086 tok/s` GPU response), the two-seed 19-table `UORLUT01` (`uor-r4.lut-discrete-trunk/1`) artifacts `04652a3d…` and `1b49e8bd…` pass all five M1 closing bars (a)–(e) simultaneously (`7,002` / `7,001 > 6,300` on `ft/dev` and `7,215` / `7,228 > 6,582` on sealed `Fresh V2`, `0` export gap, `B4` pass), closing `#2837`.
