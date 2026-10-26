# REPORT: r-rank-objective-keeps-cells, antigravity-97

Lab: antigravity. Item: [r-rank-objective-keeps-cells](../../../../roadmap/items/r-rank-objective-keeps-cells.md) ([`#2883`](https://github.com/UOR-Foundation/uor-r4/issues/2883)). Order: [ORDER.md](ORDER.md). Decision Card: [DECISION.md](DECISION.md).

Closes #2883

## What did not work

1. **Occupancy-Collapsing Rank Hinge (`OccupiedCellRankHingeV1`, `R`, `s[11] = 1`) in `K16-R` (`OCCUPANCY_COLLAPSE_TRIGGERED`, `G-Occ FAIL`):**
   - In `crates/uor-r4-discrete/src/discrete.cu:401-406` (historically `383-388`), `OccupiedCellRankHingeV1` increments `rank` only when `if (occupied[cell])` holds and charges zero penalty for cell crowding (`C-R2`).
   - Vacating a preceding cell reduces `rank` by `1` on every row whose target cell was preceded by the vacated cell (`C-R7a`: vacating 30 preceding cells drops `rank_occ` from `81` to `51`), herding `4,096` tokens into **`20 / 256`** (Seed 1, $\bar{n}_R = 3,403.9$, `DEV b_3(Full) = 12,089`, `DEV top-1 = 101`) and **`19 / 256`** (Seed 2, $\bar{n}_R = 3,480.7$, `DEV b_3(Full) = 12,466`, `DEV top-1 = 230`) occupied cells in `K16-R` vs `209 / 256` ($\bar{n}_A = 351.8$) and `192 / 256` ($\bar{n}_A = 376.1$) under `NearestAddressMarginV1` (`A`), failing both `G-Occ` clauses (`occupied_cells < max(32, ceil(0.5 * occ_A))` and $\bar{n}_R > 2\,\bar{n}_A$).
2. **Cell-Only Rank Hinges (`FixedGridRankHingeV1` `s[11]=2` and `OccupiedCellRankCrowdHingeV1` `s[11]=3`) Paired with Single-Vector `OuterL1V1` Token Scoring (`FAIL_RANK_GAIN_AT_HIGH_OCC`, `G-Rank FAIL`, `mechanism_negative: false`):**
   - Replacing `OccupiedCellRankHingeV1` (`s[11]=1`) with `FixedGridRankHingeV1` (`R_grid`, `s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`R_occ`, `s[11]=3`) provably eliminates the `K16-R` collapse (`C-R7a..C-R7c`), holding `G-Occ` (`occupied_cells = 256 / 256 > 16`, `>= 128`, $\bar{n} = 256.0 \le 512.0$, $\max_c n(c) = 16 \le 32$) and `G-Adm` (`DEV b_3(Full) = 5,009 / 14,287 >= 4,894`, `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106`, `TRAIN` anchor `1,342 / 3,847 >= 1,187`) across both seeds (`1` and `2`).
   - However, under `AddressCellsA8` (`256 / 256` cells occupied at $n(c) = 16 \le 24$), $r_{\text{grid}}(\text{row}) \equiv r_{\text{occ}}(\text{row})$ and $C_{\text{crowd}}(\text{row}) \equiv 0$; every non-endpoint move (`Combine`, `Relation`, interior blocks `1..B-1`) leaves cell occupancy and multiplicities invariant; and on all `5,009` admitted DEV rows ($r_{\text{grid}}(\text{row}) \le K = 16$), $R_{\text{grid}}(\text{row}) = 0$ (`C-R7d`).
   - Because **`4,208 / 5,009` (`84.01%`) of admitted DEV targets in `T16_fz` lose to a rival token inside their own cell `target_cell`** (`1,597 / 1,765 = 90.48%` of missed `T_restr(A_W) = 2,140` hits, `#3267`), where $R_{\text{grid}}$ and $C_{\text{crowd}}$ are constant (`0`) across all 16 tokens in `target_cell`, cell-only rank hinges paired with `OuterL1V1` score `DEV top-1 = 455 / 14,287` (`r_occ_minus_arm_a_top1 = 0 < +192`, trailing `Unigram = 1,226`, `U_restr(A_W) = 1,127`, `T_sym_restr(A_W) = 1,909`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`, $T(c(x)) = 4,085$, and `T2 = 6,300`).
   - Per CHARTER §4.1 (D22), this negative closes only the **cell-only rank objective + single-vector `OuterL1V1` configuration** (`mechanism_negative: false`), never the `rank-margin` mechanism (`R_grid` and `R_occ` remain `READY`), while pairing the same `G-Occ`-compliant `5,009` admitted cells with `WithinCellCodeRerankerV1` (`#3267`) immediately lifts `DEV top-1` to **`2,140 / 14,287`** (`2,142` cell-priority, beating `Unigram = 1,226` by `+914` rows).
3. **Round 1 D68 Decision Card Citation Discrepancies Fixed Before Approval:**
   - Fresh-context D68 reviewer `96f12503-9939-466d-83c0-3268bf63d4ab` caught four citation discrepancies in the initial draft of `DECISION.md` (`grade.json` SHA-256 `7917ec85...`, `T16_fz` `top-8 = 1,484` / `P_succ top-1 = 107` / `strict_wins = 93` and `T16_rel = 371`, `T16_rel` count-table parentheticals, and `#3267` `415 + 1,182 = 1,597` / `reranker_code_pooled = 2,140`). All four were fixed in `042105d6c` and verified (`D68 DECISION REVIEW: APPROVE`).

## What changed

  - **Size Summary (`11` files changed: `code 3 file(s) +1424/-3, docs 6 file(s) +279/-36, generated 1 file(s) +3/-3, other 1 file(s) +188/-0, tests 0 file(s) +0/-0`; item had no numeric line estimate, ratio `N/A`, reported per D48):**
  - **Suborders issued (`docs/loop/README.md` §Subagents):**
    - `local/antigravity/antigravity-97/orders/1.md` (D68 decision review, `96f12503-9939-466d-83c0-3268bf63d4ab`).
    - `local/antigravity/antigravity-97/orders/2.md` (D66 exact-head review Slice A: `crates/uor-r4-discrete/src/objective.rs`).
    - `local/antigravity/antigravity-97/orders/3.md` (D66 exact-head + claims review Slice C: `docs/**` and `rank-objective-grade.json`).
    - `local/antigravity/antigravity-97/orders/4.md` (D66 exact-head review Slice B: `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs`).
  - **Code & Unit Tests (`3` Rust files, `+1,424 / -3` lines):**
    - [`crates/uor-r4-discrete/src/objective.rs`](../../../../../crates/uor-r4-discrete/src/objective.rs) (`+620 / -0` lines): Added `RankOccupancyPolicy` (`NearestAddressMarginV1` `word = 0`, `OccupiedCellRankHingeV1` `word = 1`, `FixedGridRankHingeV1` `word = 2`, `OccupiedCellRankCrowdHingeV1` `word = 3`), `cell_precedes`, `occupied_cell_rank`, `fixed_grid_cell_rank`, `rank_hinge`, `admitted_cells_k`, `cell_crowding_hinge`, `move_time_g_occ_allowed`, `checked_row_loss_numerator`, `evaluate_g_occ_contract`, `RankOccupancyControlReceipt`, `verify_rank_occupancy_controls()` (`C-R1..C-R6`, `C-R7a..C-R7d`), and TDD unit test `rank_occupancy_policies_and_finite_controls_c_r1_to_c_r7d_pass`.
    - [`crates/uor-r4-discrete/src/stage_t.rs`](../../../../../crates/uor-r4-discrete/src/stage_t.rs) (`+594 / -0` lines): Added `RANK_OBJECTIVE_GRADE_SCHEMA` (`uor-r4.rank-objective-keeps-cells-grade/v1`), `RankObjectiveVerdict`, `K16OccupancyCollapseBaseline`, `RankObjectiveWithinCellSummary`, `RankObjectiveSeedSummary`, `RankObjectiveGradeReport`, `grade_rank_objective(...)`, and TDD unit test `grade_rank_objective_verifies_g_occ_g_adm_and_count_tables`.
    - [`crates/uor-r4-discrete/src/bin/stage-t-run.rs`](../../../../../crates/uor-r4-discrete/src/bin/stage-t-run.rs) (`+213 / -1` lines): Added `build_rank_objective_seed_summary` and the `grade-rank-objective --out <path>` CLI subcommand.
  - **Generated Grade Receipt & Documentation (`8` files, `+497 / -41` lines):**
    - [`docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json`](../../../../labs/r-rank-objective-keeps-cells/rank-objective-grade.json) (`+188 / -0` lines, SHA-256 `c49382254a4d96db804ee51339255fdf3d357fb79f0e9cc4c85ccf2ffa9765d9`).
    - [`docs/labs/r-rank-objective-keeps-cells/README.md`](../../../../labs/r-rank-objective-keeps-cells/README.md) (`+57 / -37` lines).
    - [`docs/research/notebook/m1-discrete-attention.md`](../../../../research/notebook/m1-discrete-attention.md) (`+1 / -0` lines).
    - [`docs/roadmap/items/r-rank-objective-keeps-cells.md`](../../../../roadmap/items/r-rank-objective-keeps-cells.md) (`+1 / -1` lines, `status: ready -> done`) and [`docs/roadmap/README.md`](../../../../roadmap/README.md) (`+3 / -3` lines).
    - [`docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/ORDER.md`](ORDER.md), [`DECISION.md`](DECISION.md), and [`REPORT.md`](REPORT.md).

## Measurements

- **Conditions & Pinned Upstream GPU Artifacts:**
  - Stage T two-seed `BVG-K` (`K = 16`) GPU run (`#2836`, `2x NVIDIA GeForce RTX 5090` `vast-57710022`, HF archive `stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`, `grade.json` `7917ec85941d919db14ab67567fe824c87d6d2edf18bbd54138bf320b6ea4c81`, `step0-receipt.json` `0ca7d95568adf216ba3f2abda16fbcbe3fc73bd72b96c7d0665c694000d5be50`).
  - Within-Cell Read-Code Reranker GPU decomposition (`#3267`, `1x NVIDIA GeForce RTX 5090` `vast-57824058`, `stage-t-decomposition.json` `e9dd940c0f58a1c494bb5d8f38a485528b6693bcaa67b23df3f1d867bec76cc6`).
  - Command to reproduce `docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json` (`c49382254a4d96db804ee51339255fdf3d357fb79f0e9cc4c85ccf2ffa9765d9`):
    ```sh
    ~/.cargo/bin/cargo run --manifest-path crates/uor-r4-discrete/Cargo.toml --bin stage-t-run -- grade-rank-objective --out docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json
    ```
- **Key Measured Numbers (`14,287` DEV rows, `3,847` `P_succ` rows, Seeds `1` and `2`):**
  - **Finite Controls (`C-R1..C-R6`, `C-R7a..C-R7d`):** `10 / 10 PASS` (`all_controls_pass = true`), plus Stage T `Step 0` `5 / 5 PASS` (`AKd`, `V1d`, `KId`, `V8d`, `V9d`).
  - **`G-Occ` Contract:**
    - `K16-R` (`OccupiedCellRankHingeV1`, `s[11]=1`): `20 / 256` (`< 105`, $\bar{n}_R = 3,403.9 > 703.6$) on Seed 1 and `19 / 256` (`< 96`, $\bar{n}_R = 3,480.7 > 752.2$) on Seed 2 (**`G-Occ FAIL`**).
    - `Arm-A` (`T16_fz`), `Arm-R-Grid` (`s[11]=2`), and `Arm-R-Occ` (`s[11]=3`) under `AddressCellsA8` + `D40`: `occupied_cells = 256 / 256 > 16` (`>= 128`), $\bar{n} = 256.0 \le 512.0$, $\max_c n(c) = 16 \le 32$ (**`G-Occ PASS` on both seeds**).
  - **`G-Adm` Contract:** `DEV b_3(Full) = 5,009 / 14,287 >= 4,894`, `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106`, `TRAIN` anchor `1,342 / 3,847 >= 1,187` (**`G-Adm PASS` on both seeds**).
  - **`DEV` Ranking & Count-Table Baselines (`14,287` rows):**
    - `Arm-R-Occ` / `Arm-R-Grid` / `Arm-A` (`OuterL1V1` single-vector writer): `DEV top-1 = 455 / 14,287` (`top-8 = 1,484 / 14,287`, `P_succ top-1 = 107 / 3,847`, `P_succ top-8 = 318 / 3,847`, `strict_wins = 93`; `T16_rel = 371 / 14,287`).
    - Unconditional Unigram `U`: `1,226 / 14,287`
    - Admitted-set Unigram `U_restr(A_W)`: `1,127 / 14,287` (`T16_rel = 942`)
    - Admitted-set Symmetric Code `T_sym_restr(A_W)`: `1,909 / 14,287` (`T16_rel = 1,377`)
    - Admitted-set Read-Code Table `T_restr(A_W)`: `2,140 / 14,287` (`top-8 = 4,006`, `P_succ = 376 / 3,847`; `T16_rel = 1,602`)
    - Admitted-set Bigram-x-Code `G_restr(A_W)`: `3,084 / 14,287` (`top-8 = 4,674`, `P_succ = 517 / 3,847`; `T16_rel = 2,364`)
    - Full-panel Read-Code Table $T(c(x))$: `4,085 / 14,287` (`T16_rel = 3,680`)
    - Unconditional Bigram `T2`: `6,300 / 14,287`
    - Within-Cell Read-Code Reranker (`WithinCellCodeRerankerV1`, `#3267`) on the same `5,009` admitted cells: **`2,140 / 14,287`** (`reranker_code_pooled`, `top-8 = 4,006`, $d = +1,685 > 86$ vs `OuterL1V1` `455` and $d = +1,387 > 80$ vs permuted null `753`), **`2,142 / 14,287`** (`reranker_code_cell_prio`), and **`1,706 / 14,287`** (`reranker_code_top1_cell_only`), beating `Unigram = 1,226` by `+914` rows.

## Checks run at the head

- **TDD Red -> Green Verification:**
  - Confirmed `objective::tests::rank_occupancy_policies_and_finite_controls_c_r1_to_c_r7d_pass` failed when `verify_rank_occupancy_controls()` was stubbed (`task-448` / exit 101) and passed (`3 / 3`) when restored.
  - Confirmed `stage_t::tests::grade_rank_objective_verifies_g_occ_g_adm_and_count_tables` failed when `grade_rank_objective(...)` was stubbed (`task-481` / exit 101) and passed (`6 / 6`) when restored.
- **Full `uor-r4-discrete` Test Suite & CLI Verification:**
  - `~/.cargo/bin/cargo test --manifest-path crates/uor-r4-discrete/Cargo.toml --lib --no-fail-fast objective::tests stage_t::tests` -> PASS (`9 / 9`).
  - `~/.cargo/bin/cargo run --manifest-path crates/uor-r4-discrete/Cargo.toml --bin stage-t-run -- grade-rank-objective --out docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json` -> PASS (`c49382254a4d96db804ee51339255fdf3d357fb79f0e9cc4c85ccf2ffa9765d9`).
- **Loop Gates & Policy Checks:**
  - `uor-harness loop report --check --order docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/ORDER.md` -> PASS.
  - `uor-harness loop gates --full` -> PASS (`11 / 11`).
  - `uor-harness loop pr-check --local` -> PASS.

## Skills used

- `uor-research-architect`: Used to verify the exact kernel mechanics of `OccupiedCellRankHingeV1` (`discrete.cu:401-406`), design `FixedGridRankHingeV1` (`s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`s[11]=3`) without touching `discrete.cu` or `driver_source_sha256`, and prove `C-R7d` outcome invariance on `AddressCellsA8`.
- `scope-guard`: Kept all edits strictly inside `r-rank-objective-keeps-cells`'s declared scope (`crates/uor-r4-discrete/**`, `docs/harness/prep/r-rank-objective-keeps-cells*`, `docs/labs/r-rank-objective-keeps-cells/**`, `docs/loop/tasks/r-rank-objective-keeps-cells/**`, `docs/research/notebook/m1-discrete-attention.md`, plus shared roadmap files).
- `solution-integrity`: Verified all 10 finite controls (`C-R1..C-R6`, `C-R7a..C-R7d`) with TDD red-then-green checks and cross-checked every number against committed Stage T and Within-Cell JSON receipts via fresh-context D68 review (`96f12503-9939-466d-83c0-3268bf63d4ab`).
- `gates`: Ran `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local`.

## Still open

- Roadmap item `r-rank-objective-keeps-cells` (`#2883`) is `done` (`Closes #2883`).
- `FixedGridRankHingeV1` (`s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`s[11]=3`) remain `READY` in `crates/uor-r4-discrete/src/objective.rs` (`docs/mechanisms/rank-margin.md`) for any future sparse-occupancy initialization (`occ_0 < 256`), while M1 closing work proceeds on the within-cell read-code reranker / wider-carrier path (`#2847`, `#2841`, `#2837`).

## Plain summary

- **What problem this task solved:** Every vocabulary word in the model sits in one of 256 geometric "address cells" (an $16 \times 16$ grid based on the word's first and last geometric blocks). During prediction, the model first picks a shortlist of $K = 16$ cells closest to the context query, and then ranks the words inside those 16 cells. An earlier training loss (`OccupiedCellRankHingeV1`, tested in `K16-R`) measured how many *occupied* cells beat the target word's cell. Because empty cells did not count toward the penalty, training found a shortcut: it emptied out 236 of the 256 cells and crammed all 4,096 vocabulary words into just 19 or 20 cells (`OCCUPANCY_COLLAPSE_TRIGGERED`), making the shortlist useless.
- **How the new rank objective fixes the collapse:** We implemented `FixedGridRankHingeV1` (`R_grid`, mode `2`) and `OccupiedCellRankCrowdHingeV1` (`R_occ`, mode `3`) in `crates/uor-r4-discrete/src/objective.rs`. Instead of counting only occupied cells, `R_grid` counts all 256 cells of the fixed geometric grid whether they are occupied or empty, so emptying a cell never lowers the loss (`C-R7a`). In addition, `R_occ` adds an exact integer crowding penalty (`+8` per word above 24 words in a cell, `C-R7b`) and refuses any move that drops occupied cells below 128 or crowds a cell past 32 words (`C-R7c`).
- **What the count-table evaluation showed:** On the two-seed `AddressCellsA8` Stage T models (`#2836`), where all 256 cells start evenly populated with 16 words per cell (`G-Occ PASS`) and `5,009 / 14,287` development targets land inside the 16-cell shortlist (`G-Adm PASS`), the cell rank penalty is already zero on all `5,009` admitted rows (`C-R7d`). Yet `84.01%` (`4,208 / 5,009`) of those admitted targets still lose to one of the other 15 words **inside their own cell** when scored by a single 32-integer vector (`OuterL1V1`, scoring `455 / 14,287` vs `1,226` for the unigram word-frequency table and `6,300` for the two-word `T2` table). Pairing the same `G-Occ`-compliant 256-cell shortlist with the within-cell read-code table (`WithinCellCodeRerankerV1`, `#3267`) immediately lifts accuracy from `455` to `2,140 / 14,287` (beating the unigram table by `+914` rows), proving that within-cell word selection — not cell-level rank — is the remaining bottleneck.
