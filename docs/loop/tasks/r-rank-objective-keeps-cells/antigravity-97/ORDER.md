# ORDER: r-rank-objective-keeps-cells, antigravity-97

Lab: antigravity. Item: [r-rank-objective-keeps-cells](../../../../roadmap/items/r-rank-objective-keeps-cells.md). Base: `df9ac0f25`.

## Goal

The item's done-when reads:

> A rank objective that passes G-Occ (more than 16 of 256 cells, plus the D39 condition) and is evaluated against the count tables.

## Cancel probe (run first)

- **Probe executed (`2026-10-26`, classed `mechanism` — configuration negative on cell-only rank hinges without within-cell token discrimination; `mechanism_negative: false`):**
  1. Verified `#2883` prerequisites: `#2831` (`PR #3192`, `Q1` `G-Occ PASS`) and `#2836` (`PR #3239`, HF `stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`) are `done`. On both seeds (`1`, `2`), `Arm-A` (`T16_fz`, `AddressCellsA8` + `D40` + `BVG-K` `K = 16`) passes `G-Occ` (`occupied_cells = 256 / 256 > 16`, `>= 128`, `mean_admitted_tokens = 256 <= 512`, `max_cell_tokens = 16 <= 32`) and `G-Adm` (`DEV b_3(Full) = 5,009 / 14,287 >= 4,894`, `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106`), while `OuterL1V1` `DEV top-1 = 455 / 14,287` (`T16_rel = 371`). `#3267` (`PR #3280`) proved on `T16_fz` that `4,208 / 5,009` (`84.01%`) of admitted DEV targets lose inside `target_cell` (`1,597 / 1,765 = 90.48%` of missed `T_restr(A_W) = 2,140` hits).
  2. Verified `FixedGridRankHingeV1` (`R_grid`, `s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`R_occ`, `s[11]=3`): $r_{\text{grid}}$ counts all 256 fixed-grid cells whether occupied or empty (`C-R7a`), $8\,C_{\text{crowd}}$ penalizes $n(c) > 24$ (`C-R7b`), and the move-time `G-Occ` guard refuses $\text{occupied\_cells} < 128$ or $n(c_{\text{new}}) > 32$ (`C-R7c`), eliminating the `K16-R` collapse (`20` and `19` cells) and holding `G-Occ`. Under `AddressCellsA8` (`256 / 256` cells with $n(c) = 16 \le 24$), $r_{\text{grid}} \equiv r_{\text{occ}}$, $C_{\text{crowd}} \equiv 0$, and $R_{\text{grid}} = 0$ on all `5,009` admitted DEV rows, supplying zero within-cell ranking signal (`C-R7d`).
  3. Under CHARTER §3.5, §5.1, and D68, authored `DECISION.md` (`D68 DECISION REVIEW: APPROVE`) to qualify `C-R1..C-R6` and `C-R7a..C-R7d` and grade `#2883` against `G-Occ` and the count tables (`U = 1,226`, `T2 = 6,300`, `T(c(x)) = 4,085`, `U_restr(A_W) = 1,127`, `T_sym_restr(A_W) = 1,909`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`) as **`FAIL_RANK_GAIN_AT_HIGH_OCC`** (`mechanism_negative: false`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-rank-objective-keeps-cells*`
- `docs/labs/r-rank-objective-keeps-cells/**`
- `docs/loop/tasks/r-rank-objective-keeps-cells/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

1. A fresh-context `research` subagent approves `DECISION.md` (`D68 DECISION REVIEW: APPROVE`) and logs it on `#2883`.
2. `crates/uor-r4-discrete/src/objective.rs` implements `RankOccupancyPolicy` (`words 0..3`), row-loss (`8*T + 32*R_grid + 8*C_crowd`, overflow status `12`), move-time `G-Occ` refusal (`occ >= 128`, `n(c) <= 32`), and controls `C-R1..C-R6` and `C-R7a..C-R7d` without modifying `discrete.cu` or `driver_source_sha256`.
3. `crates/uor-r4-discrete/src/stage_t.rs` and `stage-t-run` implement `grade-rank-objective`, writing `docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json` (`uor-r4.rank-objective-keeps-cells-grade/v1`) evaluated against `G-Occ`, `G-Adm`, and the count tables.
4. `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`, `docs/labs/r-rank-objective-keeps-cells/README.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/r-rank-objective-keeps-cells.md` (`status: done`), `docs/roadmap/README.md`, and `REPORT.md` (`Closes #2883`) pass `loop gates --full` and D66 review.

## Stop and ask if

- Any control (`C-R1..C-R6`, `C-R7a..C-R7d`) or SHA-256 check fails: classify `instrument` vs `mechanism` first and decide via D68 without waiting.
- `discrete.cu` or `driver_source_sha256` would be altered: keep all new rank-occupancy evaluation in `objective.rs` and `stage_t.rs`.
