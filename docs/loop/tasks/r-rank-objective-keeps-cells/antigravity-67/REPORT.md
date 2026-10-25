# REPORT: r-rank-objective-keeps-cells, antigravity-67

References #2883

## What did not work

- **Negative 1 — `OccupiedCellRankHingeV1` (`R`, `s[11] == 1`) collapses cell occupancy by rewarding cell vacation (`discrete.cu:383-388`, `K16-R`):** In `crates/uor-r4-discrete/src/discrete.cu` lines 383–388, `OccupiedCellRankHingeV1` increments `rank` only over `if (occupied[cell])` and counts each occupied cell once (`C-R2`: `single_count_per_occupied_cell_invariance`). Whenever a candidate move vacates `old_cell` (`occupied[old_cell] = false`) and enters an already-occupied `new_cell`, total `occupancy` drops by `1` and `rank` drops by `1` for every row whose `target_cell` was preceded by `old_cell`, while crowding `new_cell` incurs zero rank penalty. In `K16-R` (`docs/labs/k16-outcome-call-2026-10-19/README.md`), this herded `4,096` tokens into `20` (seed 1) and `19` (seed 2) occupied cells (`\bar{n}_R = 3,403.9` and `3,480.7` vs `\bar{n}_A = 351.8` and `376.1`), triggering `G-Occ` (`OCCUPANCY_COLLAPSE_TRIGGERED`) on both clauses and capping `DEV top-1` at `101` and `230`.
- **Negative 2 — Missing kernel implementation inside narrow `docs/**` scope and upstream Stage T dependency (`#2836`):** Neither `FixedGridRankHingeV1` (`s[11] = 2`) nor `OccupiedCellRankCrowdHingeV1` (`s[11] = 3`) is wired into `crates/uor-r4-discrete/src/{objective.rs,discrete.cu}`, which lies outside `r-rank-objective-keeps-cells.md` line 10's `docs/**` scope (`instrument`). Moreover, `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`) proved that full-dose `K = 1` single-coordinate updates + `64`-batch whole-interval rollback (`D40`) halt early on `consecutive_rejections`, making `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) the prerequisite for full `16,404`-batch `BVG-K` (`K = 16`) runs.

## What changed

- **Files changed:** `6` files (`+254 / -4` lines across `docs/`; `0` code lines, `0` test lines, `251` doc lines, `3` generated roadmap lines; within the `docs/**` scope of `r-rank-objective-keeps-cells`).
  - `docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-67/ORDER.md`: Recorded the cancel probe (`instrument`) and exact scope/done-when contract.
  - `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`: Authored the one-page pre-registration sheet (`TEST FITNESS: FIT`) for the 3-arm $\times$ 2-seed `OccupiedCellRankCrowdHingeV1` (`R_occ`) comparison.
  - `docs/labs/r-rank-objective-keeps-cells/README.md`: Authored the lab design record proving why `OccupiedCellRankHingeV1` (`R`) collapsed in `K16-R` and specifying `FixedGridRankHingeV1` ($R_{\text{grid}}$ over all 256 cells), `OccupiedCellRankCrowdHingeV1` ($8T + 32 R_{\text{grid}} + 8 C_{\text{crowd}}$ above $n_{\max} = 24$), and the move-time `G-Occ` floor ($\text{occ} \ge 128$, $\max_c n(c) \le 32$).
  - `docs/roadmap/items/r-rank-objective-keeps-cells.md` & `docs/roadmap/README.md`: Transitioned `status: ready -> status: held` following the `D68` decision review (`#2883` comment `6396718125`) and re-rendered the roadmap index.
  - `docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-67/REPORT.md`: Recorded this unit report.

## Measurements

- **Conditions:** Host inspection of `crates/uor-r4-discrete/src/{objective.rs,discrete.cu}` and sealed GPU artifacts from `K16` (`docs/labs/k16-outcome-call-2026-10-19/README.md`) and `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`) at base commit `1538c2f96f11451d16ad5eeece86505eba44b770`.
- **Command:** `uor-harness check` and `uor-harness loop gates --full`.
- **Key Numbers:**
  - `K16-R` (`OccupiedCellRankHingeV1`): `20 / 256` (seed 1) and `19 / 256` (seed 2) occupied cells; $\bar{n}_R = 3,403.9$ and $3,480.7$ tokens/row; `DEV b_3(Full) = 12,089` and `12,466 / 14,287`; `DEV top-1 = 101` and `230 / 14,287` (`OCCUPANCY_COLLAPSE_TRIGGERED`).
  - `K16-A` (`NearestAddressMarginV1` control): `209 / 256` and `192 / 256` occupied cells; $\bar{n}_A = 351.8$ and $376.1$ tokens/row; `DEV b_3(Full) = 2,237` and `3,167 / 14,287`; `DEV top-1 = 65` and `463 / 14,287`.
  - `#2831` `FrozenZero` at `Q1` (`4,096` batches, `A8` + `D40` active gate `floor = 1,187 / 3,847`): `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top-1 = 871 / 14,287`, `DEV top-8 = 1,455 / 14,287`.

## Checks run at the head

- `uor-harness loop gates --full`: `11 / 11` GREEN (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `loop-next-sources`, `loop-next-wait`, `loop-next-owner-direction`, `git-hooks`, `loop-migrate`, `loop-review-changes-blocks`).
- `uor-harness check`: `16 / 16` PASS (including `prep.sheet_shape` on `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`).
- `~/.cargo/bin/cargo run -p xtask -- roadmap check` & `~/.cargo/bin/cargo run -p xtask -- docs-catalog --check`: PASS.
- `uor-harness loop report --check --order docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-67/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS (`0` errors, `0` warnings).

## Skills used

- `uor-research-architect`: Used to diagnose the exact `if (occupied[cell])` + `C-R2` cell-vacation incentive in `discrete.cu:383-388` and design the integer `OccupiedCellRankCrowdHingeV1` (`R_occ`) formulation.
- `scope-guard` & `solution-integrity`: Used to keep all edits within the item's `docs/**` scope and transition the item to `status: held` via `D68` decision review until `#2836` (`m1-bvg-k-stage-t-training`) completes.

## Still open

- Wiring `FixedGridRankHingeV1` (`s[11] = 2`) and `OccupiedCellRankCrowdHingeV1` (`s[11] = 3`) + `C-R7` finite controls into `crates/uor-r4-discrete` and executing the 3-arm $\times$ 2-seed GPU run once `m1-bvg-k-stage-t-training` (`#2836`) completes and `r-rank-objective-keeps-cells` scope is widened to `crates/uor-r4-discrete/**`.

## Plain summary

In the earlier `K16` experiment, the rank-based training objective (`R`, which tries to pull the right drawer into the top 16 opened drawers) cheated by herding all 4,096 vocabulary words into just 19 or 20 drawers instead of spreading them across all 256 drawers. Because the formula only counted drawers that had at least one word in them and charged no penalty for overfilling a drawer, emptying a drawer automatically improved every word's drawer rank while forcing the final word-level guess to pick from ~3,400 words at once. This unit proves why that happened in the CUDA kernel, pre-registers a fixed integer rank objective (`R_occ`, reviewed as `TEST FITNESS: FIT`) that ranks across all 256 fixed geometric drawers and charges an explicit integer penalty whenever a drawer holds more than 24 words, and places the GPU run on hold (`status: held`) until the batched `BVG-K` (`K = 16`) trainer (`#2836`) is ready.
