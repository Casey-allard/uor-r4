# DECISION (D68): Register W4 Export-Gap Rule (`D39` Item 5) and Hold `m1-w4-float-credit-sheet` (`#2846`) Under Unmet `W2 = H_law` Precondition

Lab: `antigravity`. Unit: `antigravity-91`. Item: [`m1-w4-float-credit-sheet`](../../../../roadmap/items/m1-w4-float-credit-sheet.md) (`#2846`, Milestone M1 `#2029`). Base: `origin/main` `5a104c6f8410223abbbec4dc5e48042384da3583`.

## 1. The Question

[`docs/roadmap/items/m1-w4-float-credit-sheet.md`](../../../../roadmap/items/m1-w4-float-credit-sheet.md) (`#2846`) and [`PLAN.md`](../../../../PLAN.md) (lines `58` and `93`) define `W4` as conditional:
- `PLAN.md:58`: *"1 | `m1-w2-capacity-capture` (#2845) | nothing; FIT done | Stage T, `m1-address-key-full-vector` (#2834), `m1-w4-float-credit-sheet` (#2846; with step 4, and only if W2 gives H_law and W3 fails)"*
- `PLAN.md:93`: *"W4 is one float-credit sheet, run only if W2 gives H_law and W3 fails (conditional, D37)."*
- `m1-w4-float-credit-sheet.md:19`: *"W4 is conditional: only if W2 = H_law and W3 (BVG-K) fails. It is one decimal-training run in a separate scanned module (float is legal in offline training only, D37), about $5 to $10 GPU plus build work, with a disqualifying export-gap rule fixed per sheet at FIT time (D39 item 5)... Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now."*

When `D77` (`claude-82`) batch-flipped 25 `held` items to `ready`, `m1-w4-float-credit-sheet.md` was set to `status: ready` with `depends_on: [m1-w2-capacity-capture, m1-bvg-k-stage-t-training]`. Now that both `#2845` (`m1-w2-capacity-capture`) and `#2836` (`m1-bvg-k-stage-t-training`) are `done`, `uor-harness loop next` offers `m1-w4-float-credit-sheet` (`#2846`) first (alphabetically ahead of `m1-wider-code-carrier` `#2847` and `m1-within-cell-writer` `#3267`), locking `crates/uor-r4-discrete/**`.

Under `CHARTER.md §5.1` (`D67`, `D68`) and `D39` items 4–5:
1. What disqualifying export-gap rule (`D39` item 5) and Stage 2 bars should be fixed in [`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`](../../../harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md)?
2. Given the completed results of `#2845` (`W2 = H_mixed`), `#3179` (`QB = B2`, `G1` declined), `#2836` (`Stage T`), and `#2820` (`d11-exporter-float-trunk`), should `W4` spend `$5–$10` of GPU compute now, or should `m1-w4-float-credit-sheet.md` be set to `status: held` (`status_only_edit`) after completing and FIT-clearing its pre-registration sheet?

## 2. The Decision

1. **Register the `B1..B5` bars and D39 item 5 disqualifying export-gap rule in [`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`](../../../harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md):**
   - **`B1` (Disqualifying Export-Gap Rule — D39 item 5):** On the `14,287`-row DEV panel (`ft/dev`), the export gap between pre-export float/snapped evaluation and exported `UORLUT01` integer evaluation (`IntegerDiscreteModel::observe_target`) must not exceed the empirical two-seed K16 noise band (`\Delta_{\text{top1}} \le 398` rows, `\Delta_{\text{top8}} \le 748` rows, `\Delta_{\text{adm}} \le 930` rows), with post-snap `UORLUT01` vs `discrete.cu` integer parity at `0` mismatches (`14,287 / 14,287`). A Stage-1 bounded probe (`256` batches) gates any full training pass, and exceeding `B1` on either seed disqualifies the float candidate (`EXPORT_GAP_DISQUALIFIED`).
   - **`B2..B5`:** Exported `UORLUT01` integer `top1_correct` must beat `U = 1,226` and `T_restr(A_W) = 2,140` / `G_restr(A_W) = 3,084` (Stage 2a) and `G = 6,300` (`top1_d11 >= 6,435 = M + F`, Stage 2b / §4.8), beat matched discrete `max(T1_fz, T16_fz) = 871` by `> F` (`B3`), maintain `G-Adm` (`>= 4,220` full DEV and `>= 676` `P_succ`, `B4`), and pass the native + WASM zero-multiply audits with `tokens/s` reported first (`B5`).
2. **Set [`docs/roadmap/items/m1-w4-float-credit-sheet.md`](../../../../roadmap/items/m1-w4-float-credit-sheet.md) to `status: held` (`status_only_edit`, re-rendering `docs/roadmap/README.md`) and spend `$0` GPU on `W4` now:**
   - Complete the non-compute preparation (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md` with fresh-context `TEST FITNESS: FIT`, plus `docs/labs/m1-w4-float-credit-sheet/README.md`), and hold GPU execution of `#2846` so `uor-harness loop next` immediately offers the active discrete Stage 1/2 mechanism items (`m1-wider-code-carrier` `#2847` and `m1-within-cell-writer` `#3267`) and releases the `crates/uor-r4-discrete/**` scope lock.

## 3. Why (Whole-Project Context)

1. **The explicit `W2 = H_law` precondition for `W4` (`PLAN.md:58, 93`, `m1-w4-float-credit-sheet.md:19`, `D39` item 4) is NOT met:**
   - In [`m1-w2-capacity-capture`](../../../labs/m1-w2-capacity-capture/README.md) (`#2845`, `claude-87`), the exact read-code count table `T` scored `3,414 / 14,287` against `M + F = 6,300 + 135 = 6,435`, resolving **`H_mixed`** (row 3 of the W2 sheet, `3,021` rows below `H_law`). The W2 report explicitly recorded (`docs/labs/m1-w2-capacity-capture/README.md:54`): *"Per the sheet, T did not cross M + F. So the next work is a code-utilization and capacity diagnostic, not a change to the learning law. **The W4 float-credit sheet runs only on H_law, so it does not run.**"*
   - In [`m1-w2-code-utilization`](../../../labs/m1-w2-code-utilization/README.md) (`#3179`, `claude-95`), the stride-doubling projection `T_proj = 3,552` remained `2,883` rows below `H_law` (`6,435`), declining `G1` (`QB = B2`) and confirming that `H_law` is ruled out.
   - Both prior Stage 2 D68 decision reviews — [`claude-107/DECISION.md:34`](../../m1-bvg-k-stage-t-training/claude-107/DECISION.md) and [`antigravity-84/DECISION.md:17, 37-40, 54-55`](../../m1-bvg-k-stage-t-training/antigravity-84/DECISION.md) — already reviewed this exact question and dropped `#2846` (`W4`) from Stage T's outcome routing because `W2 != H_law`.

2. **Stage T (`#2836`) localized the remaining bottleneck to parameterless `OuterL1V1` within-cell selection (`#3267`), not continuous vs discrete table search:**
   - In `#2836` ([`docs/labs/m1-bvg-k-stage-t-training/README.md:19-27`](../../../labs/m1-bvg-k-stage-t-training/README.md)), discrete `BVG-K` (`K = 16`) accepted `854` coordinate moves (`T16_fz`) and `1,505` coordinate moves (`T16_rel`), raising full-DEV target cell admission to **`5,009 / 14,287` (`35.06%`, `G-Adm` PASS)** and raising the checkpoint-restricted count tables on those exact `5,009` admitted cells to **`T_restr(A_W) = 2,140 / 14,287`** and **`G_restr(A_W) = 3,084 / 14,287`** (both well above `U = 1,226`).
   - Yet the parameterless `OuterL1V1` writer (`(8*T + w*A)/(8*w)`) selected only `455 / 14,287` top-1 tokens (`9.08%` of the `5,009` admitted rows, vs `42.72%` for `T_restr(A_W)` and `61.57%` for `G_restr(A_W)` on the same admitted cells).
   - Moving the shared `4,096 x 32` table coordinates via offline float gradients still feeds the **same parameterless `OuterL1V1` writer and 5-value endpoint address key** after `UORLUT01` root quantization (`d37-reactivation-architect.md` §3.2, §3.5), while introducing a quantization export gap. By contrast, [`m1-within-cell-writer`](../../../../roadmap/items/m1-within-cell-writer.md) (`#3267`) and [`m1-wider-code-carrier`](../../../../roadmap/items/m1-wider-code-carrier.md) (`#2847`) directly target the within-cell selection and carrier bottlenecks on the zero-export-gap discrete path.

3. **Holding `#2846` unblocks the active M1 discrete items in `uor-harness loop next`:**
   - Because `loop next` sorts `ready` M1 items with equal unblock counts alphabetically (`m1-w4-float-credit-sheet`, `m1-wider-code-carrier`, `m1-within-cell-writer`), leaving `#2846` as `status: ready` causes `loop next` to offer `#2846` ahead of `#2847` and `#3267` and locks `crates/uor-r4-discrete/**`. Setting `status: held` (`status_only_edit`, matching `antigravity-74` `#3220` and `claude-114` on `#2839`) aligns the item's machine-readable status with `PLAN.md:58, 93` and `antigravity-84/DECISION.md:17`.

## 4. Rejected Alternatives

- **Spend `$5–$10` GPU to run `W4` now despite `W2 = H_mixed`:**
  - Rejected because it violates `PLAN.md:58, 93` (*"only if W2 gives H_law and W3 fails"*), `docs/labs/m1-w2-capacity-capture/README.md:54` (*"The W4 float-credit sheet runs only on H_law, so it does not run"*), `antigravity-84/DECISION.md:17`, and `CHARTER.md §5.1` (`D67`, which refuses GPU runs whose whole-project precondition is unmet).
- **Leave `m1-w4-float-credit-sheet.md` at `status: ready` without running it:**
  - Rejected because `uor-harness loop next` would immediately re-offer `m1-w4-float-credit-sheet` on the very next session, starving `m1-wider-code-carrier` (`#2847`) and `m1-within-cell-writer` (`#3267`) (`claude-107/DECISION.md:28`).
- **Close `#2846` as `status: done` without a GPU run:**
  - Rejected because `#2846`'s `Done when` requires *"Sheet FIT'd and run once; integer top-1 through the D11 export reported against both count tables; export gap within the two-seed band on the same evaluator."* Setting `status: held` preserves `#2846` as a staged conditional fallback (`CHARTER.md §4.1`, `D22`) without falsely claiming its GPU run is done.

## 5. Cost to Reverse

Low (`1`-line `status: held -> ready` edit in `docs/roadmap/items/m1-w4-float-credit-sheet.md` plus `cargo run -p xtask -- roadmap render`). Because the pre-registration sheet `docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md` is completed and FIT-cleared in this unit, if `#3267` and `#2847` later close without solving within-cell ranking and `#2820` clears the float export-gap bar, `#2846` can be flipped back to `ready` and executed immediately.

## 6. Review Record (`TEST FITNESS: FIT` and `D68 DECISION REVIEW`)

- **Reviewer:** `antigravity` fresh-context non-author reviewer (`454cc9f1-3f1d-4b16-b291-4df23b2b23eb`)
- **Verdicts:**
  - `TEST FITNESS: FIT` on `docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`
  - `DECISION REVIEW (D68): PASS`

```text
DECISION REVIEW (D68): PASS
card: docs/loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md
item: m1-w4-float-credit-sheet (#2846, Milestone M1 #2029)
reviewer: antigravity fresh-context non-author subagent (454cc9f1-3f1d-4b16-b291-4df23b2b23eb)
verdict: PASS — Verified against CHARTER.md (§3.1-§3.7, §4.5, §4.8, §5.1), PLAN.md (lines 50-96), DECISIONS.md (D37, D39, D67, D68, D73, D79), m1-w2-capacity-capture (#2845), m1-w2-code-utilization (#3179), and m1-bvg-k-stage-t-training (#2836, claude-107/DECISION.md, antigravity-84/DECISION.md). Both PLAN.md:58,93 and m1-w4-float-credit-sheet.md:19 condition W4 on W2 = H_law and W3 failing. W2 (#2845) returned H_mixed (T = 3,414 <= M + F = 6,435) and explicitly recorded that W4 does not run (README.md:54); #3179 confirmed QB = B2 (T_proj = 3,552 < 6,435); and Stage T (#2836) localized the remaining gap on the 5,009 admitted DEV cells (455 writer top-1 vs 2,140 T_restr(A_W) and 3,084 G_restr(A_W)) to parameterless within-cell selection (#3267 m1-within-cell-writer) and carrier capacity (#2847 m1-wider-code-carrier). Registering B1..B5 (fixing the D39 item 5 disqualifying export-gap rule to \Delta_top1 <= 398, \Delta_top8 <= 748, \Delta_adm <= 930 with a 256-batch Stage-1 probe) and setting m1-w4-float-credit-sheet.md to status: held ($0 GPU) is the right whole-project decision and unblocks #2847 and #3267 in loop next.
date: 2026-10-26T11:02:38Z
```
