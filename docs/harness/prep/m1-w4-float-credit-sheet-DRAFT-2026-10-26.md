# M1 Stage 2 (W4): Conditional Offline Float-Credit Writer Training & Export-Gap Probe — Pre-Registration (DRAFT)

- **Line:** M1 Stage 2 (`m1-w4-float-credit-sheet`, [#2846](https://github.com/UOR-Foundation/uor-r4/issues/2846), milestone [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)).
- **Lab:** `antigravity` (`antigravity-91`).
- **Run:** `m1-w4-float-credit-sheet-stage2`
- **Mechanism under test:** Conditional D37 offline float-credit writer training ([`crates/uor-r4-float-train/src/native_vector_writer.rs`](../../../crates/uor-r4-float-train/src/native_vector_writer.rs) and [`crates/uor-r4-float-train/src/fbs_export.rs`](../../../crates/uor-r4-float-train/src/fbs_export.rs)) exported to `UORLUT01` ([`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs), [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs)) against the discrete `K = 1` (`#2831`) and `BVG-K` `K = 16` (`#2836`) baselines (`CHARTER.md §3.1–§3.7`, `§4.8`, D37, D39 item 4 & item 5, D73).
- **Status:** Pre-registered (`TEST FITNESS: FIT`), **GPU execution HELD under D68** ([`DECISION.md`](../../loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md)): `PLAN.md:58, 93` and [`m1-w4-float-credit-sheet.md:19`](../../roadmap/items/m1-w4-float-credit-sheet.md) condition `W4` on `W2 = H_law` and `W3 (BVG-K)` failing, whereas [`m1-w2-capacity-capture`](../../labs/m1-w2-capacity-capture/README.md) (`#2845`) returned **`H_mixed`** (`T = 3,414 <= M + F = 6,435`), [`m1-w2-code-utilization`](../../labs/m1-w2-code-utilization/README.md) (`#3179`) returned **`QB = B2`** (`T_proj = 3,552 < 6,435`, declining `G1`), and [`m1-bvg-k-stage-t-training`](../../labs/m1-bvg-k-stage-t-training/README.md) (`#2836`) localized the `455` vs `2,140` / `3,084` gap on `5,009` admitted cells to within-cell token selection ([`m1-within-cell-writer`](../../roadmap/items/m1-within-cell-writer.md), `#3267`) and carrier/address capacity ([`m1-wider-code-carrier`](../../roadmap/items/m1-wider-code-carrier.md), `#2847`), while float-trunk D11 export parity ([`d11-exporter-float-trunk`](../../roadmap/items/d11-exporter-float-trunk.md), `#2820`) remains in progress.

## 1. Why

In the post-dose Stage 2 sequence (`PLAN.md:58, 90-93`, [`d37-reactivation-architect.md` §6 W4](../../labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md), [`synthesis.md:110, 125-127`](../../labs/repo-renormalization-2026-10-19/audit/synthesis.md)), `W4` is the conditional Stage 2 fallback if and only if:
1. `W2` (`#2845` / `#3179`) establishes `H_law` (that the 16-bit read code already carries `> M + F = 6,435` DEV top-1 hits and the discrete search law is the sole bottleneck); and
2. `W3` (`BVG-K` Stage T, `#2836`) fails to learn ranking at matched budget; and
3. The discrete within-cell selection (`#3267`) and wider-carrier (`#2847`) mechanisms fail to close the within-cell ranking gap (`455` writer top-1 vs `2,140` `T_restr(A_W)` and `3,084` `G_restr(A_W)` on the `5,009` admitted DEV rows of `T16_fz`).

This sheet completes the non-compute preparation required by [`docs/roadmap/items/m1-w4-float-credit-sheet.md:19-24, 38`](../../roadmap/items/m1-w4-float-credit-sheet.md): it fixes the D39 item 5 disqualifying export-gap rule at FIT time, defines the mandatory Stage-1 bounded export-gap probe before any full training arm, and pins the exact 5-arm comparison and stop rules so the sheet is ready if its entry condition is ever triggered.

## 2. What changes

No changes to `crates/uor-r4-discrete` (`driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` untouched). When the entry gate (§9) is met, `W4` runs in the separately scanned D37 module `crates/uor-r4-float-train` (`zero` dependency edge on `crates/uor-r4-training`, verified by `uor-harness check`):
- **Allowed primitive list (`d37-reactivation-architect.md` §8 item 4):** Exact piecewise-linear L1 distances and hinge margins (`+`, `-`, `*`, `/`, `sqrt`, `abs`, `min`, `max`, `round`, `floor`) over the 16-root E8 orbit and 256 root-pair address cells (`margin_rank_rows`, `margin_rank_grad`, `straight_through_fwd` in `crates/uor-r4-float-train/src/cuda_stack.cu`). Zero `exp`, `log`, `pow`, `softmax`, `log_softmax`, `softplus`, `sigmoid`, `tanh`, `GELU`, or dense matrix multiplication (`CHARTER.md §3.2, §3.3`).
- **Two-stage execution protocol (`d37-reactivation-architect.md` §6 W4):**
  1. **Stage-1 Export-Gap Probe (`256` batches = `16,384` rows, Seeds `1` and `2`):** Runs `256` offline float hinge-credit updates on the `4,096 x 32` table coordinates from the pinned `A8` codebook (`6600bdb1…`), snaps each 8D block to its nearest L1 E8 root in the 16-root palette (`snap_to_nearest_root`), exports to `UORLUT01` (`uor-r4.lut-discrete/1`), and measures the paired float-vs-integer export gap on the `14,287`-row DEV panel before any full pass is booked.
  2. **Stage-2 Full Pass (`16,404` batches = `1,049,856` rows, Seeds `1` and `2`, gated on Stage-1 PASS):** Runs one canonical pass of `16,404` batches (`B = 64, C = 192`) on `2x RTX 5090` with `64`-batch D40 anchor audits on the `3,847`-row TRAIN copy-successor anchor (`885789a7…`), snapping and exporting to `UORLUT01` at `Q1` (`4,096`), `Q2` (`8,202`), `Q3` (`12,303`), and `Q4` (`16,404`), and scoring every decision metric strictly through the exported D11 integer artifact (`CHARTER.md §3.7`, D39 item 5, D73).

## 3. Bar

Registered under `CHARTER.md §5.1`, D39 item 5, D68, and D73 ([`DECISION.md`](../../loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md)):

| Bar | Measure | Registered threshold (must hold on both Seed `1` and Seed `2`) |
|---|---|---|
| **B1 (Disqualifying Export-Gap Rule — D39 item 5)** | Paired difference between pre-export snapped/float evaluation and exported `UORLUT01` integer evaluation (`IntegerDiscreteModel::observe_target`) on the `14,287`-row DEV panel | Must not exceed the empirical two-seed noise band from [`k16-outcome-call-2026-10-19`](../../labs/k16-outcome-call-2026-10-19/README.md) and [`m1-export-gap-and-toks-DRAFT-2026-10-23.md`](m1-export-gap-and-toks-DRAFT-2026-10-23.md): **`\Delta_{\text{top1}} = |\text{top1}_{\text{float}} - \text{top1}_{\text{D11}}| \le 398` rows (`2.79%`)**, **`\Delta_{\text{top8}} \le 748` rows (`5.24%`)**, and **`\Delta_{\text{adm}} \le 930` rows (`6.51%`)**, with post-snap `UORLUT01` vs `discrete.cu` integer parity at **`0` mismatches (`14,287 / 14,287`)**. Exceeding this band on either seed **disqualifies** the float candidate (`EXPORT_GAP_DISQUALIFIED`). |
| **B2 (Stage 2 Integer Quality vs Count Tables — D39 / §4.8)** | Exported `UORLUT01` integer `top1_correct` on the `14,287`-row DEV panel (`ft/dev`) | **Stage 2a:** beats the model-matched unigram table **`U = 1,226 / 14,287`** (`8.58%`) and the `T16_fz` admission-restricted tables **`T_restr(A_W) = 2,140`** and **`G_restr(A_W) = 3,084`** on both seeds. **Stage 2b (M1 close bar §4.8):** beats both **`U = 1,226 / 14,287`** and the full-panel T2 query-token bigram **`G = 6,300 / 14,287`** (`44.10%`) on both seeds (`top1_d11 >= 6,435 = M + F`). |
| **B3 (Matched Discrete Non-Inferiority / Gain)** | Exported `UORLUT01` integer `top1_correct` vs matched discrete `BVG-K` (`T16_fz = 455`, `T16_rel = 371` in `#2836`) and `K = 1` (`T1_fz = 871` at `Q1` in `#2831`) | Beats `max(T1_fz, T16_fz) = 871` by more than the paired discordance floor `F = max(24, ceil(2 * sqrt(b + c)))` on the `14,287`-row DEV panel on both seeds. |
| **B4 (Admission Guard `G-Adm`)** | Exported `UORLUT01` integer target-cell admission on DEV (`14,287` rows) and `P_succ` (`3,847` rows) | Full-DEV `target_cell_admitted >= 4,220 / 14,287` (`29.54%`) and `P_succ` `target_cell_admitted >= 676 / 3,847` (`17.57%`) on both seeds. |
| **B5 (D11 Zero-Multiply & Throughput Reporting)** | Native + WASM static instruction audits and client `tokens/s` | `0` float, `0` mul, `0` div instructions in `audit_zero_matmul_serving.py` and `audit_zero_matmul_wasm.py`, with prompt-reading (`192`-token context) and reply-writing (`64`-token decode) `tokens/s` reported first (`CHARTER.md §3.6, §3.10`). |

## 4. Base artifact

- **Pinned `A8` base codebook (`V = 4,096, B = 4, C = 192, K = 8`):** `address-structured-v1.json` (`caseyallard/uor-r4-store:vast-archive/vast-56660582/uor-r4/runs/claude/address-cells-preflight-20261019/tables/a8-rebuilt.json`), file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, `table_sha256 = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` (`4,096 / 4,096` distinct vectors, `256` occupied cells, `16` tokens/cell, checkpoint-0 DEV `top1_correct = 133`, `top8_correct = 513`, `target_cell_admitted = 4,244 / 14,287`).
- **Sealed discrete comparators on the exact same base codebook and data:**
  - `W1-K1` (`#2831`, [`docs/labs/m1-a8-full-dose-rerun/README.md`](../../labs/m1-a8-full-dose-rerun/README.md)): `T1_fz` (`Q1` DEV `top1 = 871`, `top8 = 1,455`, `admitted = 4,292`) and `T1_rel` (`batch 1,600` DEV `top1 = 133`).
  - `W3-BVG-K` (`#2836`, [`docs/labs/m1-bvg-k-stage-t-training/README.md`](../../labs/m1-bvg-k-stage-t-training/README.md)): `T16_fz` Seeds `1` & `2` (`report.json` `127d9044…` / `28c0d58e…`, DEV `top1 = 455`, `top8 = 1,484`, `admitted = 5,009`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`) and `T16_rel` Seeds `1` & `2` (`report.json` `756e76db…` / `c0bf457d…`, DEV `top1 = 371`, `top8 = 1,124`, `admitted = 3,827`, `T_restr(A_W) = 1,602`, `G_restr(A_W) = 2,364`).
  - `W2` count-table references (`#2845`, `record.json` `69717971a6f4e6f4a6dede6ad5eee630c4abdbda13e4c8dd434ef126a74c1abc`): `U = 1,226`, `G = 6,300`, `T(Q4) = 3,414`.

## 5. Data split and panel

- **TRAIN stream (`16,404` batches of `64` windows of length `C = 192`, `1,049,856` scored target tokens):**
  - `ft/train/tokens.u16`: SHA-256 `49fda0ad5ce9b7edceffc6ac2ab76fa2c5df52dc55d409edbb3113dc6971038b` (`caseyallard/uor-r4-store:data/ft-train.tar`);
  - `ft/train/response_mask.u8`: SHA-256 `e85e965b7aa4cf75be655fa26a5ca021ad0a5f08ee5f9a69060ebfa43e1ed720`.
- **TRAIN copy-successor D40 audit anchor (`3,847` rows):**
  - `anchor-manifest.json`: file SHA-256 `885789a71420751660c69c82ec01df7c6bfc285dc4a87e28648e1ead050e909b` (`manifest_sha256 = f85466ff2880ab00ee6648b63d1cb8577fd8ba75425161f770091285460784e8`).
- **Frozen DEV evaluation panel (`14,287` scored response rows across `224` batches, `3,847` `P_succ` rows):**
  - `ft/dev/tokens.u16`: SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`caseyallard/uor-r4-store:data/ft-dev.tar`);
  - `ft/dev/response_mask.u8`: SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **Five registered arms:**
  1. **`W4-Float-Exported` (Seeds `1` and `2`):** Differentiable writer hinge over `8 * T + 32 * A` on the `4,096 x 32` table coordinates in `crates/uor-r4-float-train`, snapped to the 16-root E8 palette, exported to `UORLUT01`, and scored on DEV through `IntegerDiscreteModel`.
  2. **`W3-BVG-K` (`T16_fz` & `T16_rel`, Seeds `1` and `2`):** Sealed Stage T discrete `K = 16` checkpoints (`#2836`).
  3. **`W1-K1` (`T1_fz` & `T1_rel`):** Sealed `#2831` discrete `K = 1` checkpoints.
  4. **`W4-RandomRefit` (negative control):** Random root-table perturbation at matched per-block Hamming distance (`\Delta_{\text{blocks}}` matched to `W4-Float-Exported`), exported to `UORLUT01`.
  5. **`W4-ZeroStep` (`K = 0` identity control):** Zero float steps from `address-structured-v1.json` (`6600bdb1…`), snapped and exported to `UORLUT01` (must reproduce checkpoint-0 `top1 = 133`, `top8 = 513`, `admitted = 4,244` with `0` mismatches).

## 6. Source

- [`crates/uor-r4-float-train/src/native_vector_writer.rs`](../../../crates/uor-r4-float-train/src/native_vector_writer.rs) and [`crates/uor-r4-float-train/src/cuda_stack.cu`](../../../crates/uor-r4-float-train/src/cuda_stack.cu) (`margin_rank_rows`, `margin_rank_grad`, `straight_through_fwd`, `adam_update`).
- [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) (`export_d11_to_path`, `uor-r4.lut-discrete/1`) and [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel::from_path`, `observe_target`).
- [`crates/uor-r4-discrete/src/writer_bucket_device.rs`](../../../crates/uor-r4-discrete/src/writer_bucket_device.rs) (`BucketWriterEngine`, qualified in `#2840` at `81,533 tokens/s` on `A8` `ft/dev`).

## 7. Steps

1. Complete non-compute preparation (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`, `docs/labs/m1-w4-float-credit-sheet/README.md`, `DECISION.md`) and obtain fresh-context non-author `TEST FITNESS: FIT` and `D68` decision reviews.
2. **Entry Gate Check (§9):** Do **not** rent a GPU pod while `W2 = H_mixed` (`T = 3,414 <= M + F = 6,435`) and discrete within-cell selection (`#3267` `m1-within-cell-writer`), wider carrier (`#2847` `m1-wider-code-carrier`), or float-trunk D11 export parity (`#2820` `d11-exporter-float-trunk`) are pending.
3. If and only if the §9 entry gate is met and a fresh-context D67 pre-run review passes:
   - Run the **Stage-1 Export-Gap Probe (`256` batches, Seeds `1` and `2`)** on `1x RTX 5090` (`< 10` min, `< $0.30`). Verify `W4-ZeroStep` exact checkpoint-0 reproduction (`133 / 513 / 4,244`, `0` mismatches) and check `B1` (`\Delta_{\text{top1}} \le 398`, `\Delta_{\text{top8}} \le 748`, `\Delta_{\text{adm}} \le 930`). Abort immediately if `B1` fails.
   - If Stage-1 passes `B1` on both seeds, run the **Stage-2 Full Pass (`16,404` batches, Seeds `1` and `2`)** on `2x RTX 5090` (`~2–4` GPU-h, `~$3–$6`), export `Q1..Q4` to `UORLUT01`, score DEV through `IntegerDiscreteModel`, and run the native + WASM zero-multiply audits and laptop `tokens/s` benchmark (`B1..B5`).

## 8. Controls and negatives

1. **`W4-ZeroStep` (`0` updates) exactness control:** Exporting the un-updated float table initialized from `address-structured-v1.json` (`6600bdb1…`) must match the discrete checkpoint-0 evaluation row-by-row on all `14,287` DEV rows (`top1 = 133`, `top8 = 513`, `admitted = 4,244`, `0` mismatches). Any mismatch is an instrument fault (`D31`).
2. **`W4-RandomRefit` negative control:** Random root replacements at matched Hamming distance (`\Delta_{\text{blocks}}`) must degrade DEV `top1_correct` and `target_cell_admitted` relative to `W4-Float-Exported` and `W4-ZeroStep`.
3. **Planted export-quantization fault:** Perturbing one root coordinate before `UORLUT01` export (`cells[0] ^= 1`) must be caught by the `B1` parity check (`> 0` mismatches).
4. **Static primitive & dependency gate:** `uor-harness check` and `audit_zero_matmul_serving.py` / `audit_zero_matmul_wasm.py` must confirm `0` dependency edges from `uor-r4-float-train` to `uor-r4-training`, `0` softmax/squashing/matmul primitives in training, and `0` float/mul/div instructions in the served `UORLUT01` runtime.

## 9. Stop conditions and outcomes

- **Pre-run Entry Gate (currently HOLDING execution under D68):**
  - `PLAN.md:58, 93` and `m1-w4-float-credit-sheet.md:19` specify that `W4` runs *"only if W2 = H_law and W3 (BVG-K) fails"*. Because `W2` (`#2845` / `#3179`) returned **`H_mixed`** (`T = 3,414 <= M + F = 6,435`, `T_proj = 3,552 < 6,435`) and Stage T (`#2836`) showed that `5,009 / 14,287` target cells are already admitted (`T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084` vs `OuterL1V1` writer `455`), `m1-w4-float-credit-sheet` stays **`status: held`** while `#3267` (`m1-within-cell-writer`), `#2847` (`m1-wider-code-carrier`), and `#2820` (`d11-exporter-float-trunk`) run.
- **Stage-1 / Stage-2 runtime stop rules (if entry gate is later released):**
  - **`EXPORT_GAP_DISQUALIFIED`:** If `\Delta_{\text{top1}} > 398`, `\Delta_{\text{top8}} > 748`, or `\Delta_{\text{adm}} > 930` at Stage-1 (`256` batches) or `Q1..Q4` on either seed, stop immediately; allow at most one pre-registered rounding/snap successor repair (`d37-reactivation-architect.md` §6 W4), after which a second gap violation closes the float-writer configuration (`D22`).
  - **`DIAG_STOP_CONSEC_REJ` / `G-Adm` failure:** Stop if three consecutive 64-batch D40 anchor audits reject or if DEV admission falls below `4,220` (`Full`) or `676` (`P_succ`).
  - **Outcome table:**
    - **`B1..B5` all pass (including `top1_d11 >= 6,435` on both seeds):** `W4` passes the §4.8 M1 bar through the D11 export (under D73); proceed to `m1-fresh-panel-transfer` (`#2841`) and `m1-close-saved-model` (`#2837`).
    - **`B1`, `B3`, `B4`, `B5` pass and `1,226 < top1_d11 < 6,435`:** Float credit improves within-cell table placement over discrete `K = 1` / `K = 16` without exceeding the export-gap band, but full-panel `G = 6,300` still requires the versioned within-cell writer (`#3267`) or wider carrier (`#2847`).
    - **`B1` fails (`EXPORT_GAP_DISQUALIFIED`) or `B3` fails (`top1_d11 <= 871`):** Close the `W4` float-table configuration at tested scope (`D22`); retain the discrete learning path (`#3267`, `#2847`).

## 10. Cost and minimum test power

- **Preparation cost (this unit, `antigravity-91`):** **`$0.00` GPU** (host documentation, sheet registration, and fresh-context FIT + D68 reviews only).
- **Conditional GPU cost (only if §9 entry gate is met later):** Stage-1 probe `< $0.30` (`1x RTX 5090`, `~10` min); Stage-2 full two-seed run `~$4.00–$6.00` (`2x RTX 5090`, `~2–3` h), within the `$5–$10` item estimate and well inside the `<= 4` pods and `<= $8.00/h` cluster caps.
- **Minimum test power:** Two seeds (`Seed 1`, `Seed 2`), `14,287` frozen DEV rows (`3,847` `P_succ` rows), paired McNemar discordance floor `F = max(24, ceil(2 * sqrt(b + c)))` (`d > 24` and `d^2 > 4 * (b + c)`), and exact `14,287`-row `UORLUT01` export parity verification.

## 11. Verdict

```text
TEST FITNESS: FIT - M1 #2029 m1-w4-float-credit-sheet
sheet: docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md
reviewer: antigravity (non-author; fresh-context adversarial FIT reviewer 454cc9f1-3f1d-4b16-b291-4df23b2b23eb for #2846 in antigravity-91)
note: All 11 required sections are complete and concise. B1 fixes the D39 item 5 disqualifying export-gap rule on the 14,287-row DEV panel to the empirical two-seed noise band (\Delta_top1 <= 398, \Delta_top8 <= 748, \Delta_adm <= 930, 0 post-snap UORLUT01 mismatches). A 256-batch Stage-1 export-gap probe gates any 16,404-batch Stage-2 run across the 5 matched arms (W4-Float-Exported, W3-BVG-K, W1-K1, W4-RandomRefit, W4-ZeroStep) on pinned A8 (6600bdb1...). Strictly enforces CHARTER.md §3 (crates/uor-r4-float-train with 0 dependency on uor-r4-training, explicit piecewise-linear primitive allowlist with 0 softmax/squashing/matmul, and D11 UORLUT01 integer-only decision metrics). GPU execution is properly held under Section 9 until the entry gate is met.
date: 2026-10-26
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
