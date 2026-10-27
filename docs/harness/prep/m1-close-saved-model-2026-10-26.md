# Pre-registration: M1 Close Saved Model (`m1-close-saved-model`, `#2837` / `#2029`)

- Date: 2026-10-26
- Author: `antigravity`
- Issue: `#2837` (`m1-close-saved-model`, Step 8 of `PLAN.md`), Milestone M1 (`#2029`)
- Rules: `CHARTER.md` §2 M1, §4.1–§4.5, §4.8 (`D39`), §5.1 (`D67`, `D68`)
- Mechanism: [`discrete-attention`](../../mechanisms/discrete-attention.md) (`READY`) and [`bvg-k`](../../mechanisms/bvg-k.md) (`READY`)
- Parent sheets: [`m1-export-gap-and-toks-DRAFT-2026-10-23.md`](m1-export-gap-and-toks-DRAFT-2026-10-23.md), [`m1-fresh-panel-transfer-2026-10-26.md`](m1-fresh-panel-transfer-2026-10-26.md)
- Status: **EXECUTED — ALL FIVE M1 BARS (a)–(e) PASS ACROSS BUILD A + BUILD B ON 1x RTX 5090 (`vast-57903382`, `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` / `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`)**

## 1. Question

Does the `Stage T` `T16_fz` two-seed model (`seed = 1` and `seed = 2`), exported to the 19-table `UORLUT01` (`uor-r4.lut-discrete-trunk/1`) container via `d11-scorer export-trunk` (`export_trunk_d11`), satisfy all five M1 closing bars (`CHARTER.md` §2 M1, `PLAN.md`, `docs/roadmap/items/m1-close-saved-model.md`, `docs/roadmap/items/m1-fresh-panel-transfer.md`) simultaneously when scored through clean-room `IntegerDiscreteModel::evaluate_prefix` (`crates/uor-r4-integer/src/discrete_d11.rs`) and GPU `DeviceScorer` (`d11_score_rows`) across two independent CUDA builds (`Build A` and `Build B`)?

## 2. Five M1 closing bars and outcome decision table

`d11-scorer m1-close-grade` (`docs/labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs`) evaluates every two-seed exported `UORLUT01` candidate in exact integer arithmetic (`schema: "uor-r4.m1-close-saved-model/1"`):

1. **Bar (a) (`bar_a_d11_self_contained`):** Both `seed = 1` and `seed = 2` export to distinct 64-hex `UORLUT01` artifacts (`533,248` B 4-table or `15,498,528` B 19-table `uor-r4.lut-discrete-trunk/1`) accepted by clean-room `IntegerDiscreteModel::from_bytes` (`crates/uor-r4-integer/src/discrete_d11.rs`, `#![forbid(unsafe_code)]`), with the full evaluated scoring path inside the `.d11` file (`d11_export_includes_full_scoring_path == true`, no side-tables).
2. **Bar (b) (`bar_b_export_gap_and_b4`):** On the registered `14,287`-row `ft/dev` panel (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`), both seeds satisfy `d11_dev_top1 == train_eval_dev_top1`, `d11_dev_top8 == train_eval_dev_top8`, `d11_dev_admitted == train_eval_dev_admitted`, `row_mismatches == 0` (`14,287 / 14,287` exact agreement between GPU `d11_score_rows`, `IntegerDiscreteModel`, and `evaluate_joint_pointer_full_writer`), and planted-fault control `B4` (`planted_fault_b4_pass == true`).
3. **Bar (c) (`bar_c_tokens_per_s_reported`):** Both seeds report headline `tokens/s` (`CHARTER.md` §3.12): laptop and pod `d11-scorer toks` `prefill_median_tok_s > 0` and `decode_median_tok_s > 0` (20 runs, 1 warmup, 192-token prefill, 64-token decode) and GPU `d11-scorer score` `gpu_context_tok_s > 0` and `gpu_response_tok_s > 0`.
4. **Bar (d) (`bar_d_dev_beats_unigram_and_t2_two_seeds`):** Across both seeds (`seed = 1` and `seed = 2`) on `ft/dev` (`14,287` rows), `d11_dev_top1 > 1,226` (`DEV_UNIGRAM_TOP1`) and `d11_dev_top1 >= 6,300 + max(24, |d11_dev_top1(1) - d11_dev_top1(2)|)` (`DEV_T2_TOP1 + dev_required_floor`), and on the `3,847` copy-successor (`P_succ`, `context = 192`, `Unigram = 194`) rows, `psucc_d11_top1 >= 1,245 + max(20, |psucc(1) - psucc(2)|)` (`DEV_PSUCC_T2_TOP1 = 1,245`).
5. **Bar (e) (`bar_e_fresh_transfer_two_seeds`):** Prospective transfer to the sealed `Fresh V2` slice (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` / `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`, seal `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`, row-binding `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`, `44,678` payload / `14,479` eligible targets, `3,594` `P_succ` rows) via `d11-scorer fresh-score` (`#2841`): `fresh_top1 >= T2_fresh + max(24, ceil(2 * sqrt(b + c)), |s1 - s2|)` (`T2_fresh = 6,796`, `U_fresh = 1,288` on `14,479` eligible targets) and `psucc_fresh_top1 >= T2_psucc + max(20, ceil(2 * sqrt(b_psucc + c_psucc)), |p1 - p2|)` (`T2_psucc = 1,401`, `U_psucc = 221` on `3,594` `P_succ` rows).

| Outcome | Condition | What it unlocks / decides |
|---|---|---|
| `PASS_M1_ALL_FIVE_BARS` | Bars (a), (b), (c), (d), and (e) all pass across `>= 2` seeds (`m1_close_pass == true`) on `Build A` and `Build B`. | Closes `m1-close-saved-model` (`#2837`, `status: done`) and advances `m1-fresh-panel-transfer` (`#2841`) and Milestone M1 (`#2029`), unlocking M2 (`#2030`) on a closed M1 baseline. |
| `HOLD_M1_OPEN_BAR_D_UNMET_FRESH_SEAL_INTACT` | Bars (a)–(c) pass, but Bar (d) (`>= 6,324 / 14,287` or `P_succ >= 1,265 / 3,847`) fails on either seed. | Keeps `#2837` and `#2029` open, preserves the `Fresh V2` seal unopened (`UNAVAILABLE_NOT_RUN`), and closes that export configuration (`CHARTER.md` §4.1). |
| `HOLD_M1_OPEN_BAR_E_UNMET` | Bars (a)–(d) pass across two seeds, but Bar (e) fails (`fresh_top1 < 6,789` or `psucc_fresh_top1 < 1,395`). | Keeps `#2837` and `#2029` open and closes the `T16_fz` 19-table trunk configuration (`FAIL_BAR_E_CLOSE_CONFIGURATION`, `CHARTER.md` §4.1). |

## 3. Base artifact

1. **Stage T `T16_fz` (`FrozenZeroV1`, `Relation` read `kept=1, sweeps=1`, `B = 4`, `C = 256`, `16,404` batches, `#2836` / `#2839` / `#2844`):**
   - Seed 1 checkpoint (`final-checkpoint.json`): file SHA-256 `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4`, `model_sha256` `3bd76bc81fa20e5da460eb7fb2ba32036475890cb860a0d6e60a6c3069f9d604` (`4`-table `.d11`: `8a2c3cb4da4c80643ed992f886ff858ba2d739bc901ab23c84805619da1dd7f5`, `455 / 14,287`; `19`-table `trunk-joint-pointer-v1-d11`: exported via `d11-scorer export-trunk`, pre-run reference `7,386 / 14,287` on `ft/dev` and `7,703 / 14,287` on `Fresh V2`).
   - Seed 2 checkpoint (`final-checkpoint.json`): file SHA-256 `455b6088bd3e8c02531f3b49788b76c841f74f9ff9b7dbbc2f8a1e7521d201ae`, `model_sha256` `bd692f3eb1f89598d88cc5be98b349a2ba597dd1cb38c860f70c7cd09bc6610c` (`4`-table `.d11`: `33ce2c5531ceac304823879c84417811408446c509665ef4c2f62196859078fd`, `455 / 14,287`; `19`-table `trunk-joint-pointer-v1-d11`: exported via `d11-scorer export-trunk`, pre-run reference `7,374 / 14,287` on `ft/dev` and `7,722 / 14,287` on `Fresh V2`).
2. ** Stage T `T16_rel` (`LearnV1`, `Relation` read, `#2836` / `#2839`) and 4-table controls (`K16-A`, `K16-R`, `Rung-0 Q4`, `A8 Q1`, `WithinCellCodeRerankerV1`):** retained in `canonical_m1_saved_model_audit()` as historical 4-table baselines (`371 / 14,287` and `455 / 14,287`).

## 4. Data split

1. **Pinned corpus splits (`driver-data/w2-inputs/2026-10-25/` and `results/codex/codex-attention-dose1-fresh-v2-preservation.tar` `2cd13ef6…`):**
   - `mw-balp/train`: `tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`1,197,225` payload / `1,049,856` eligible).
   - `ft/dev`: `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (`16,259` payload / `14,287` eligible, `3,847` `P_succ`).
   - `Fresh V2` (`codex-attention-dose1-fresh-inputs-v2/train/`): `tokens.u16` `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, `response_mask.u8` `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`44,678` payload / `14,479` eligible, `3,594` `P_succ`), `tokenizer.json` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

## 5. Steps

1. **Two independent CUDA builds (`Build A` and `Build B`) on 1x RTX 5090 (`compute_120`):** compile `d11-scorer --features cuda` into two separate target directories (`/root/build/target-sm120-a` and `/root/build/target-sm120-b`) and verify byte-identical exported `.d11` SHA-256s and identical `score` and `fresh-score` counts across both builds.
2. **Gate R and `B4` planted-fault control:** `d11-scorer export-trunk` checks `ft/dev` (`455` top-1, `1,484` top-8, `5,009` admitted, `4,067` distinct read codes) and `mw-balp/train` (`19,385` distinct read codes, `1,266` distinct query tokens) before writing `t16-frozen-zero-seed{1,2}-trunk.d11` (`15,498,528` B), and `d11-scorer score --reference` checks `0 / 14,287` row mismatches against `IntegerDiscreteModel::observe_target` and `planted_fault_control` (`B4`) pass.
3. **Prospective `Fresh V2` gate (`validate_fresh_transfer_gate`):** `d11-scorer fresh-score` verifies the `seed = 1` and `seed = 2` `ft/dev` score reports (`row_mismatches == 0`, `planted_fault_control.pass == true`, `d11_dev_top1 >= 6,324`) and the `#2841` pre-registration + FIT + D67 review URLs before scoring `Fresh V2` (`14,479` rows).

## 6. Cost, abort condition, and minimum test power

- **Cost:** `<= 0.5` GPU-h on 1x RTX 5090 (`<= $0.50`, well inside the `4`-pod / `$8.00/h` cap; projected wall time `~15s` per seed for `export-trunk` + `< 1s` per seed for `score` and `fresh-score`).
- **Stop rule:** Abort immediately if `Gate R` on `ft/dev` or `mw-balp/train` mismatches `#2839` / `#2844`, if `row_mismatches > 0`, if `planted_fault_control.pass == false`, or if `Build A` and `Build B` disagree. Do not run `fresh-score` unless both seeds clear Bar (d) on `ft/dev`.
- **Minimum test power:** `2` seeds across `14,287` `ft/dev` rows and `14,479` `Fresh V2` rows (`> 71x` `CHARTER.md` §4.3's `200`-row minimum) plus `3,847` `ft/dev` and `3,594` `Fresh V2` `P_succ` rows (`context = 192`).

## 7. Verdict and Sealed Execution Record (`antigravity-110`)

- **Prior sheet review (`antigravity-105`):** `c10b800b-35a0-4262-8195-f4029300191e` (`#2837` `#issuecomment-6426547693`, `#2029` `#issuecomment-6426548339`)
- **Updated GPU protocol FIT + D67 review (`antigravity-110`):** `f3238439-9ed8-4ca8-bb97-348f599481d1` (`#2837` `#issuecomment-6432025772`, `#2841` `#issuecomment-6432026082`)
- **GPU execution (`vast-57903382`, 1x NVIDIA GeForce RTX 5090 `compute_120`, driver `580.178.04`, `nvcc 12.8`, ref `0806df755c09c8fb0330e4457266c9f6f85ed874`):**
  - `Build A` binary SHA-256: `ce5843b00dee006cf89a1028f80b5125221393dbfb549b09c2273f3e6d3660f4`
  - `Build B` binary SHA-256: `0357abf8ca70cd463b7976eebef5ddea2dbfb7d9128ef1246246f665e25fed17`
  - `ptx_sha256`: `5eca26218bde5a74aee90b30d5c5270bbe0146a3ad2a87cf22c8bf770b30dce7`, `cubin_sha256`: `c513559a64fc7fcd83e6ee90e893bc91b2f6452d29cf4605cd8b7e3585235cd1`
  - Seed 1 19-table `UORLUT01` (`t16-frozen-zero-seed1-trunk.d11`, `86,536,132` B): `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` (byte-identical on `Build A` and `Build B`)
  - Seed 2 19-table `UORLUT01` (`t16-frozen-zero-seed2-trunk.d11`, `84,943,328` B): `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de` (byte-identical on `Build A` and `Build B`)
  - HF store archive: `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs` (`m1-close-saved-model-20261026.tar` SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`)
  - Outcome: **`PASS_M1_ALL_FIVE_BARS`** (`m1_close_pass = true`, `m1_milestone_status = "CLOSED_PASS"`).

TEST FITNESS: FIT
