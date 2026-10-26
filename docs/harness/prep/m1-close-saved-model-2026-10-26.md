# Pre-registration: M1 Close Saved Model (`m1-close-saved-model`, `#2837` / `#2029`)

- Date: 2026-10-26
- Author: `antigravity`
- Issue: `#2837` (`m1-close-saved-model`, Step 8 of `PLAN.md`), Milestone M1 (`#2029`)
- Rules: `CHARTER.md` §2 M1, §4.1–§4.5, §4.8 (`D39`), §5.1 (`D67`, `D68`)
- Mechanism: [`discrete-attention`](../../mechanisms/discrete-attention.md) (`READY`) and [`bvg-k`](../../mechanisms/bvg-k.md) (`READY`)
- Parent sheets: [`m1-export-gap-and-toks-DRAFT-2026-10-23.md`](m1-export-gap-and-toks-DRAFT-2026-10-23.md), [`m1-fresh-panel-transfer-2026-10-26.md`](m1-fresh-panel-transfer-2026-10-26.md)
- Status: **REGISTERED — FIVE-BAR AUDIT + PATH A POST-ROTATION JOINT-POINTER TRUNK D11 EXECUTED; BAR (d) MET (`7,702`/`7,701` `roots0`, `7,386`/`7,374` `roots_final` > `6,300` `T2`), HELD AT BAR (a) CLEAN-ROOM `uor-r4-integer` 19-TABLE READER (`OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`)**

## 1. Question

Does any saved discrete geometric language model (`crates/uor-r4-discrete`) on `main` satisfy all five M1 closing bars (`CHARTER.md` §2 M1, `PLAN.md`, `docs/roadmap/items/m1-close-saved-model.md`) simultaneously across `>= 2` seeds (`seed = 1` and `seed = 2`) when scored through the exported integer container (`UORLUT01`, `uor-r4.lut-discrete/1`, `IntegerDiscreteModel::evaluate_prefix` and GPU `d11_score_rows`)? And when the five-bar audit shows that no saved 4-table `UORLUT01` model clears Bar (d) (`> 6,300 / 14,287` `T2` on `ft/dev`), what exact structural bottlenecks cap `UORLUT01` and what `CHARTER.md` §4.4 pivot unlocks Bar (d) while keeping the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`) intact?

## 2. Five M1 closing bars and outcome decision table

`d11-scorer m1-close-grade` (`docs/labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs`) evaluates every two-seed exported `UORLUT01` candidate in exact integer arithmetic (`schema: "uor-r4.m1-close-saved-model/1"`):

1. **Bar (a) (`bar_a_d11_self_contained`):** Both `seed = 1` and `seed = 2` export to distinct 64-hex `UORLUT01` artifacts (`533,248` B) accepted by clean-room `IntegerDiscreteModel::from_bytes` (`crates/uor-r4-integer/src/discrete_d11.rs`, `#![forbid(unsafe_code)]`), and the exported `UORLUT01` container includes the candidate's full evaluated scoring path (`d11_export_includes_full_scoring_path == true`, with no omitted host side-tables).
2. **Bar (b) (`bar_b_export_gap_and_b4`):** On the registered `14,287`-row `ft/dev` panel (`0919e90f…` / `a838bb1a…`), both seeds satisfy `d11_dev_top1 == train_eval_dev_top1`, `d11_dev_top8 == train_eval_dev_top8`, `d11_dev_admitted == train_eval_dev_admitted`, `row_mismatches == 0` (`14,287 / 14,287` exact row agreement with `discrete-stack eval-corpus`), and planted-fault control `B4` (`planted_fault_b4_pass == true`).
3. **Bar (c) (`bar_c_tokens_per_s_reported`):** Both seeds report headline `tokens/s` (`CHARTER.md` §3.12): laptop `d11-scorer toks` `prefill_median_tok_s > 0` and `decode_median_tok_s > 0` (20 runs, 1 warmup, 192-token prefill, 64-token decode) and GPU `eval-corpus` `gpu_context_tok_s > 0` and `gpu_response_tok_s > 0`.
4. **Bar (d) (`bar_d_dev_beats_unigram_and_t2_two_seeds`):** Across both seeds (`seed = 1` and `seed = 2`) on `ft/dev` (`14,287` rows), `d11_dev_top1 > 1,226` (`DEV_UNIGRAM_TOP1`) and `d11_dev_top1 >= 6,300 + max(24, |d11_dev_top1(1) - d11_dev_top1(2)|)` (`DEV_T2_TOP1 + dev_required_floor`), and on the `3,847` copy-successor (`P_succ`, `context = 192`, `Unigram = 194`) rows, `psucc_d11_top1 >= 1,245 + max(20, |psucc(1) - psucc(2)|)` (`DEV_PSUCC_T2_TOP1 = 1,245`).
5. **Bar (e) (`bar_e_fresh_transfer_two_seeds`):** Prospective transfer to the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`, seal `df5d074a…`, row-binding `c2d4c7e5…`, `14,479` eligible targets / `14,287` matched batch-walk rows, `3,594` `P_succ` rows) via `d11-scorer fresh-score` (`#2841`). If `bars_a_to_d_pass == false`, `Fresh V2` MUST NOT be opened or scored (`fresh_transfer_status == "UNAVAILABLE_NOT_RUN"`, `bar_e_fresh_transfer_two_seeds == false`, `dev_decision == "FAIL_BAR_D_HOLD_FRESH_SEAL"`). Under `CHARTER.md` §2 and `m1-close-saved-model.md`, `NOT_RUN` is never `PASS`.

| Outcome | Condition | What it unlocks / decides |
|---|---|---|
| `PASS_M1_ALL_FIVE_BARS` | Bars (a), (b), (c), (d), and (e) all pass across `>= 2` seeds (`m1_close_pass == true`). | Closes `m1-close-saved-model` (`#2837`, `status: done`) and Milestone M1 (`#2029`), unlocking M2 (`#2030`) on a closed M1 baseline. |
| `HOLD_M1_OPEN_BAR_D_UNMET_FRESH_SEAL_INTACT` | Bars (a), (b), and (c) pass on the exported 4-table `UORLUT01` artifacts (`T16_fz` `455 / 14,287`, `T16_rel` `371 / 14,287`), but Bar (d) (`>= 6,324 / 14,287`) is unmet (`m1_close_pass == false`). | Keeps `#2837` and `#2029` open (`OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`), preserves the `Fresh V2` seal unopened (`UNAVAILABLE_NOT_RUN`), and registers the `CHARTER.md` §4.4 two-path pivot (Path A discrete post-rotation joint-hierarchy D11 export; Path B offline float-trained SpiralCore/FBS trunk exported via `#2818`). |
| `HOLD_M1_OPEN_BAR_E_UNMET` | Bars (a)–(d) pass across two seeds, but Bar (e) is `UNAVAILABLE_NOT_RUN` or `FAIL_BAR_E_CLOSE_CONFIGURATION`. | Keeps `#2837` and `#2029` open; if `UNAVAILABLE_NOT_RUN`, dispatches `#2841` `d11-scorer fresh-score`; if `FAIL_BAR_E_CLOSE_CONFIGURATION`, closes that configuration (`CHARTER.md` §4.1). |

## 3. Base artifacts, candidate inventory, and data splits

All saved discrete models on `main` are audited by `canonical_m1_saved_model_audit()`:

1. **Stage T `T16_fz` (`FrozenZeroV1`, `Flock` read, `B = 4`, `C = 256`, `16,404` batches, `#2836` / `#2839`):**
   - Seed 1 checkpoint `3bd76bc81fa20e5da460eb7fb2ba32036475890cb860a0d6e60a6c3069f9d604`, `UORLUT01` `8a2c3cb4da4c80643ed992f886ff858ba2d739bc901ab23c84805619da1dd7f5` (`533,248` B): `d11_dev_top1 = 455`, `top8 = 1,484`, `admitted = 5,009`, `0` row mismatches, `B4` pass, laptop `12,551` prefill / `12,630` decode `tok/s`, GPU `56,836,282` context / `232,879` response `tok/s`.
   - Seed 2 checkpoint `bd692f3eb1f89598d88cc5be98b349a2ba597dd1cb38c860f70c7cd09bc6610c`, `UORLUT01` `33ce2c5531ceac304823879c84417811408446c509665ef4c2f62196859078fd` (`533,248` B): `d11_dev_top1 = 455`, `top8 = 1,484`, `admitted = 5,009`, `0` row mismatches, `B4` pass, laptop `12,663` prefill / `12,523` decode `tok/s`, GPU `56,652,733` context / `232,126` response `tok/s`.
2. **Stage T `T16_rel` (`LearnV1`, `Relation` read, `B = 4`, `C = 256`, `16,404` batches, `#2836` / `#2839`):**
   - Seed 1 checkpoint `ec80dd872af8914393448b532325410e5890b6a6bdac495a1140d03c2a1a1d44`, `UORLUT01` `166403ca59268a16b0c5c7f7c225a15a3f1dbef797dfba4d38daec5c9b04fb5d` (`533,248` B): `d11_dev_top1 = 371`, `top8 = 1,517`, `admitted = 5,009`, `0` row mismatches, `B4` pass, laptop `1,237` prefill / `1,197` decode `tok/s`, GPU `56,836,282` context / `232,879` response `tok/s`.
   - Seed 2 checkpoint `9f56c3a4ab6998a180cfbe1cb6cb428749c6984ef11484abff2f5e625a703165`, `UORLUT01` `3e52357b3b7fdcb123db339c5767b5685d7ccbc80511a8431b538ce0936de3a0` (`533,248` B): `d11_dev_top1 = 371`, `top8 = 1_517`, `admitted = 5,009`, `0` row mismatches, `B4` pass, laptop `1,179` prefill / `1,118` decode `tok/s`, GPU `56,935,299` context / `233,284` response `tok/s`.
3. **Prior / auxiliary & Path A post-rotation discrete checkpoints on `main`:** `K16-A` (`747`), `K16-R` (`359`), `Rung-0 Q4` (`eb43ff38…`, `367`), `#2831` `A8 Q1` (`c6882e43…`, `765`), `#3267` `WithinCellCodeRerankerV1` (`2,135`), `#2844` `TrunkAssemblyV1` (`2,988`–`2,989` `T16_fz` / `2,227`–`2,226` `T16_rel`), and `antigravity-106` Path A `trunk-joint-pointer-v1-d11` (`export_trunk_d11` / `import_trunk_d11`, 19-table `UORLUT01` `uor-r4.lut-discrete-trunk/1`, `0` export gap: joint-hierarchy `6,943`/`6,943` `roots0` and `6,674`/`6,665` `roots_final` `> 6,324` `T2`, `6,875`/`6,875 > 6,765` `fresh_v2`; joint-pointer `7,702`/`7,701` `roots0` and `7,386`/`7,374` `roots_final` `> 6,324`, `P_succ = 2,090`/`2,086 > 1,245`, `fresh_v2 = 7,703`/`7,722 > 6,765`, `fresh_v2 P_succ = 2,408`/`2,413 > 1,375`; held at Bar (a) until `crates/uor-r4-integer` accepts 19-table `uor-r4.lut-discrete-trunk/1`).

## 4. Instrument steps and `CHARTER.md` §4.4 pivot

1. **Five-bar negative-gate tests (`tests/m1_close.rs`):** Setting aside each bar `(a)`, `(b)`, `(c)`, `(d)`, `(e)` (`NOT_RUN`), or the premature-fresh-unsealing check fails `evaluate_m1_closing_bars` / `validate_fresh_transfer_gate`; restoring it passes.
2. **Path A discrete trunk hierarchy + orbit-root causal pointer + 19-table D11 export (`crates/uor-r4-discrete/src/trunk.rs`):** `select_level3_joint_hierarchy_global`, `orbit_root_causal_successor`, `select_level3_joint_pointer_global`, `evaluate_joint_pointer_full_writer`, and `export_trunk_d11` / `import_trunk_d11` (`uor-r4.lut-discrete-trunk/1`, 19 tables) clear Bar (d) on both seeds (`7,702`/`7,701` `roots0`, `7,386`/`7,374` `roots_final`, `2,090`/`2,086` `P_succ`) with `0` export gap (`Bar (b)`). Next step to close Bar (a) + Bar (e): wire 19-table `uor-r4.lut-discrete-trunk/1` into `crates/uor-r4-integer/src/discrete_d11.rs` and run `#2841` `d11-scorer fresh-score`.
3. **Path B offline float-trained trunk export (`#2818`, `d11-exporter-float-trunk`):** Offline float-trained SpiralCore / FBS multi-layer trunk exported to `UORLUT01` (`CHARTER.md` §3.6 / `D37`).

## 5. Cost, abort condition, and minimum test power

- **Cost:** `$0.00` (`0` GPU hours in `antigravity-105` / `antigravity-106`; all evaluations run on stored GPU streams from `#2839` and `#2844`).
- **Stop rule:** Do not mark `#2837` `done` or open `Fresh V2` via `d11-scorer fresh-score` while `m1_close_pass == false`.
- **Minimum test power:** `2` seeds across `14,287` `ft/dev` rows (`71.4x` `CHARTER.md` §4.3's `200`-row minimum) and `3,847` `P_succ` rows (`context = 192`); full-panel floor `max(24, ceil(2 * sqrt(b + c)), |s1 - s2|)`.

## 6. Verdict

- **Reviewer:** Fresh-context non-author subagent `c10b800b-35a0-4262-8195-f4029300191e` (`gemini-4-argon-high`, 2026-10-26)
- **Posted review & D68 owner log:** `#2837` (`#issuecomment-6426547693`) and `#2029` (`#issuecomment-6426548339`)

TEST FITNESS: FIT
