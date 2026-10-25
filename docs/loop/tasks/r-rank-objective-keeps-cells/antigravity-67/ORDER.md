# ORDER: r-rank-objective-keeps-cells, antigravity-67

Lab: antigravity. Item: [r-rank-objective-keeps-cells](../../../../roadmap/items/r-rank-objective-keeps-cells.md). Base: `1538c2f96`.

## Goal

The item's done-when reads:

> A rank objective that passes G-Occ (more than 16 of 256 cells, plus the D39 condition) and is evaluated against the count tables.

Under the item's `docs/**` scope, this unit runs the cancel probe first, authors the one-page pre-registration sheet (`docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`) and lab design record (`docs/labs/r-rank-objective-keeps-cells/README.md`) for the occupancy-preserving rank objective (`OccupiedCellRankCrowdHingeV1`, `R_occ`), obtains the fresh-context non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and `D68` decision review (`status: ready -> status: held` until `#2836` `m1-bvg-k-stage-t-training` completes and scope includes `crates/uor-r4-discrete/**`), and posts the review on `#2883`.

## Cancel probe (run first)

- **Class:** `instrument` (missing pre-registration sheet + missing `OccupiedCellRankCrowdHingeV1` kernel/enum variant in `crates/uor-r4-discrete`, outside this item's `docs/**` scope, plus upstream M1 Stage T dependency `#2836`).
- **Command & Evidence:** Inspected `crates/uor-r4-discrete/src/{objective.rs,discrete.cu}`, `docs/labs/k16-outcome-call-2026-10-19/README.md`, `docs/labs/m1-a8-full-dose-rerun/README.md`, and `docs/mechanisms/rank-margin.md`.
- **Findings:**
  1. No pre-registration sheet `docs/harness/prep/r-rank-objective-keeps-cells*` or lab record `docs/labs/r-rank-objective-keeps-cells/**` exists yet (`instrument` missing).
  2. In `crates/uor-r4-discrete/src/discrete.cu` lines 383–388, `OccupiedCellRankHingeV1` (`s[11] == 1`, loss $8T + wR$ with $w = 32$ and $R = \max(0, r_{\text{occ}} - K)$) increments `rank` only over `if (occupied[cell])` and counts each occupied cell once (`C-R2`). Vacating a cell lowers `occupancy` and mechanically lowers `rank` across all rows, while crowding tokens into $19\text{--}20$ cells incurs zero rank penalty. In `K16-R` (`docs/labs/k16-outcome-call-2026-10-19/README.md`), this herded 4,096 tokens into `20` and `19` cells (`\bar{n}_R = 3,403.9` and `3,480.7` vs `\bar{n}_A = 351.8` and `376.1`), firing `G-Occ` (`OCCUPANCY_COLLAPSE_TRIGGERED`) on both clauses and leaving `DEV top-1 = 101` and `230`.
  3. Meanwhile, `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`) proved on RTX 5090 hardware that starting from the injective A8 cell layout (`256 / 256` cells at `16` tokens/cell) under the active `D40` admission-anchor gate (`floor = 1,187 / 3,847`) raises `FrozenZero` at `Q1` (`4,096` batches) to `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, and `DEV top-1 = 871 / 14,287` before `K = 1` cadence-granularity coupling (`STOP_FOR_DIAGNOSIS: consecutive_rejections`) halted the run.
  4. `docs/roadmap/items/r-rank-objective-keeps-cells.md` line 10 restricts `scope:` to `docs/**` (excluding `crates/uor-r4-discrete/**`), and `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) is the active M1 prerequisite for full-dose `BVG-K` (`K = 16`) training.

## Files in scope

- `docs/harness/prep/r-rank-objective-keeps-cells*`
- `docs/labs/r-rank-objective-keeps-cells/**`
- `docs/loop/tasks/r-rank-objective-keeps-cells/**`
- Always-allowed paths: `docs/roadmap/items/r-rank-objective-keeps-cells.md`, `docs/roadmap/README.md`

## Done when

- Cancel probe committed first in `ORDER.md`.
- `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md` and `docs/labs/r-rank-objective-keeps-cells/README.md` authored and reviewed (`TEST FITNESS: FIT` + `D68` decision review posted on `#2883`).
- `r-rank-objective-keeps-cells` set to `status: held`, `docs/roadmap/README.md` rendered, and `REPORT.md` passes all gates.

## Stop and ask if

- Never stop to wait; decide via `D68` decision review and log `[antigravity] owner log:` on `#2883`.
