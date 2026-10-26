# M1 Stage 2 (W4): Conditional Offline Float-Credit Writer Sheet & D68 Hold Record (`m1-w4-float-credit-sheet`, `#2846`)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-91`, PR `#3275`)
- **Milestone:** M1 (`#2029`), Stage 2 conditional fallback in [`PLAN.md:58, 93`](../../../PLAN.md)
- **Roadmap Item:** [`docs/roadmap/items/m1-w4-float-credit-sheet.md`](../../roadmap/items/m1-w4-float-credit-sheet.md) (`#2846`)
- **Pre-Registration Sheet:** [`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`](../../harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md)
- **Decision Card:** [`docs/loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md`](../../loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md)

---

## 1. Summary

[`docs/roadmap/items/m1-w4-float-credit-sheet.md`](../../roadmap/items/m1-w4-float-credit-sheet.md) (`#2846`) and [`PLAN.md`](../../../PLAN.md) (lines `58` and `93`) define `W4` as a conditional Stage 2 fallback:
> *"W4 is conditional: only if W2 = H_law and W3 (BVG-K) fails. It is one decimal-training run in a separate scanned module (float is legal in offline training only, D37), about $5 to $10 GPU plus build work, with a disqualifying export-gap rule fixed per sheet at FIT time (D39 item 5)... Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now."*

In this unit (`antigravity-91`):
1. **Completed the non-compute preparation deliverable (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`)** and obtained a fresh-context non-author `TEST FITNESS: FIT` review fixing:
   - **`B1` (Disqualifying Export-Gap Rule — D39 item 5):** On the `14,287`-row DEV panel (`ft/dev`), the export gap between pre-export float/snapped evaluation and exported `UORLUT01` integer evaluation (`IntegerDiscreteModel::observe_target`) must not exceed the empirical two-seed K16 noise band (`\Delta_{\text{top1}} \le 398` rows, `\Delta_{\text{top8}} \le 748` rows, `\Delta_{\text{adm}} \le 930` rows), with post-snap `UORLUT01` vs `discrete.cu` integer parity at `0` mismatches (`14,287 / 14,287`). A Stage-1 bounded probe (`256` batches) gates any full training pass, and exceeding `B1` on either seed disqualifies the float candidate (`EXPORT_GAP_DISQUALIFIED`).
   - **`B2..B5` (Stage 2 & §4.8 Integer Quality, Discrete Non-Inferiority, Admission, and Zero-Multiply Serving):** Exported `UORLUT01` integer `top1_correct` must beat `U = 1,226` and `T_restr(A_W) = 2,140` / `G_restr(A_W) = 3,084` (`B2a`) and `G = 6,300` (`top1_d11 >= 6,435 = M + F`, `B2b` / `CHARTER.md §4.8`), beat matched discrete `max(T1_fz, T16_fz) = 871` by `> F` (`B3`), maintain `G-Adm` (`>= 4,220` full DEV and `>= 676` `P_succ`, `B4`), and pass the native + WASM zero-multiply static audits with `tokens/s` reported first (`B5`).
2. **Evaluated the Stage 2 entry gate under a fresh-context D68 whole-project decision review ([`DECISION.md`](../../loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md)) and set `#2846` to `status: held` (`$0` GPU spent):**
   - `W2` (`#2845`, [`docs/labs/m1-w2-capacity-capture/README.md:5, 27-29, 54`](../m1-w2-capacity-capture/README.md)) returned **`H_mixed`** (`T = 3,414 <= M + F = 6,435`) and explicitly recorded: *"The W4 float-credit sheet runs only on H_law, so it does not run."*
   - `W2` code utilization (`#3179`, [`docs/labs/m1-w2-code-utilization/README.md`](../m1-w2-code-utilization/README.md)) returned **`QB = B2`** (`T_proj = 3,552 < 6,435`), declining `G1` and confirming `H_law` is not met.
   - `W3` (`#2836`, [`docs/labs/m1-bvg-k-stage-t-training/README.md`](../m1-bvg-k-stage-t-training/README.md)) expanded DEV target-cell admission to `5,009 / 14,287` (`G-Adm` PASS) and checkpoint-restricted tables to `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084`, localizing the `455` vs `2,140` / `3,084` gap to parameterless `OuterL1V1` within-cell token selection ([`m1-within-cell-writer`](../../roadmap/items/m1-within-cell-writer.md), `#3267`) and carrier/address capacity ([`m1-wider-code-carrier`](../../roadmap/items/m1-wider-code-carrier.md), `#2847`), while both [`claude-107/DECISION.md:34`](../../loop/tasks/m1-bvg-k-stage-t-training/claude-107/DECISION.md) and [`antigravity-84/DECISION.md:17`](../../loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/DECISION.md) dropped `#2846` from Stage T's fallback routing.

---

## 2. Stage 2 Comparative Baseline Table (`14,287`-Row Frozen DEV Panel)

| Arm / Comparator | Source Record | DEV `target_cell_admitted` (`/ 14,287`) | DEV `P_succ` Admitted (`/ 3,847`) | DEV `top1_correct` (`/ 14,287`) | DEV `top8_correct` (`/ 14,287`) | Export Gap (`\Delta_{\text{top1}}`) | Status |
|---|---|---:|---:|---:|---:|---:|---|
| **`A8` Checkpoint 0 (`W4-ZeroStep` target)** | `#2840` (`qualification-build-a.json` `37419bdc…`) | `4,244` (`29.71%`) | `1,351` (`35.12%`) | `133` (`0.93%`) | `513` (`3.59%`) | `0` | Qualified (`81,533 tok/s` direct writer) |
| **`W1-K1` (`T1_fz` at `Q1`)** | `#2831` (`antigravity-57`) | `4,292` (`30.04%`) | `1,106` (`28.75%`) | `871` (`6.10%`) | `1,455` (`10.18%`) | `0` | Sealed (`STOP_FOR_DIAGNOSIS` at batch `5,248`) |
| **`W3-BVG-K` (`T16_fz`, Seeds `1` & `2`)** | `#2836` (`antigravity-84`, `127d9044…` / `28c0d58e…`) | **`5,009` (`35.06%`)** | **`1,216` (`31.61%`)** | `455` (`3.18%`) | **`1,484` (`10.39%`)** | `0` | Sealed (`G-Adm` PASS; within-cell gap -> `#3267`) |
| **`W3-BVG-K` (`T16_rel`, Seeds `1` & `2`)** | `#2836` (`antigravity-84`, `756e76db…` / `c0bf457d…`) | `3,827` (`26.79%`) | `1,144` (`29.74%`) | `371` (`2.60%`) | `1,124` (`7.87%`) | `0` | Sealed (`425` relation moves committed) |
| **`T_restr(A_W)` on `T16_fz` admitted cells** | `#2836` (`w2-summary.json` `19009681…`) | `5,009` (`35.06%`) | — | **`2,140` (`14.98%`)** | **`4,006` (`28.04%`)** | `0` | Beats `U = 1,226` (`+914` rows) |
| **`G_restr(A_W)` on `T16_fz` admitted cells** | `#2836` (`w2-summary.json` `19009681…`) | `5,009` (`35.06%`) | — | **`3,084` (`21.59%`)** | **`4,674` (`32.72%`)** | `0` | Beats `U = 1,226` (`+1,858` rows) |
| **Unrestricted `W2` code table `T(Q4)`** | `#2845` (`record.json` `69717971…`) | — | — | `3,414` (`23.90%`) | `7,558` (`52.90%`) | `0` | **`H_mixed`** (`3,414 <= M + F = 6,435`) |
| **Model-matched Unigram `U`** | `#2845` (`record.json` `69717971…`) | — | — | `1,226` (`8.58%`) | `4,060` (`28.42%`) | `0` | Co-primary lower reference |
| **Full-panel T2 Query-Token Bigram `G`** | `#2845` (`record.json` `69717971…`) | — | — | `6,300` (`44.10%`) | `11,263` (`78.83%`) | `0` | Co-primary upper reference (`M + F = 6,435`) |
| **`W4-Float-Exported` (registered sheet)** | [`m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`](../../harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md) | Bar `>= 4,220` | Bar `>= 676` | Bar `> 1,226` / `>= 6,435` | — | Bar `<= 398` | **`TEST FITNESS: FIT`; HELD under D68 (`W2 != H_law`)** |
