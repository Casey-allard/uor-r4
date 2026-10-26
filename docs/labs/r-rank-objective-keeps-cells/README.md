# Occupancy-Preserving Rank Objective (`r-rank-objective-keeps-cells`, `#2883`)

- **Issue / Roadmap Item:** [`#2883`](https://github.com/UOR-Foundation/uor-r4/issues/2883) (`docs/roadmap/items/r-rank-objective-keeps-cells.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/m1-discrete-attention.md`)
- **Lab:** `antigravity` (ORDER [`docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/ORDER.md`](../../loop/tasks/r-rank-objective-keeps-cells/antigravity-97/ORDER.md), D68 Decision Card [`docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/DECISION.md`](../../loop/tasks/r-rank-objective-keeps-cells/antigravity-97/DECISION.md))
- **Pre-Registration Sheet:** [`docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`](../../harness/prep/r-rank-objective-keeps-cells-2026-10-25.md) (`TEST FITNESS: FIT`, [`#2883` comment `6396718125`](https://github.com/UOR-Foundation/uor-r4/issues/2883#issuecomment-6396718125), SHA-256 `800f63e0472ea9ab9bb28ce2132d42afdfb5598bfd20394f16c7cc6b790dc986`)
- **D68 Decision Review:** Fresh-context `research` subagent `96f12503-9939-466d-83c0-3268bf63d4ab` (`D68 DECISION REVIEW: APPROVE`, [`#2883` comment `6422813286`](https://github.com/UOR-Foundation/uor-r4/issues/2883#issuecomment-6422813286))
- **Pinned Grade Artifact:** [`docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json`](rank-objective-grade.json) (`uor-r4.rank-objective-keeps-cells-grade/v1`, SHA-256 `c49382254a4d96db804ee51339255fdf3d357fb79f0e9cc4c85ccf2ffa9765d9`)
- **Current Status:** `done` (Verdict: **`FAIL_RANK_GAIN_AT_HIGH_OCC`**, `mechanism_negative: false` — configuration negative on cell-only rank objectives paired with single-vector `OuterL1V1`; `FixedGridRankHingeV1` `s[11]=2` and `OccupiedCellRankCrowdHingeV1` `s[11]=3` pass `G-Occ`, `G-Adm`, and all 10 finite controls `C-R1..C-R7d` and remain `READY`)

---

## 1. Root-Cause Diagnosis of the `K16-R` Occupancy Collapse & Exact Remedy (`C-R7a..C-R7d`)

### 1.1 Why `OccupiedCellRankHingeV1` (`R`, `s[11] = 1`) Collapsed in `K16-R`

In `crates/uor-r4-discrete/src/discrete.cu` lines 401–406 (historically 383–388), `OccupiedCellRankHingeV1` (`s[11] == 1`) computes:

```cuda
U rank=1,occupancy=0;
if(s[11]==1 || telemetry)for(U cell=0;cell<256;++cell)if(occupied[cell]){
    ++occupancy;U distance=address_distance(q,g,s,cell);
    if(cell_precedes(distance,cell,ad,target_cell))++rank;
}
W rm=rank>s[3]?rank-s[3]:0;
```

with row loss numerator $\mathcal{L}_{\text{tr}} = 8\,T + 32\,\text{rm}$ ($K = s[3] = 16$). This formulation has two coupled structural defects (**proof**, directly from `discrete.cu:401-406`):

1. **Vacating a cell strictly reduces `rank` across the dataset (`C-R7a`):** Because `++rank` runs only when `if (occupied[cell])` holds, any token move on block `0` or block `s[1]-1` that vacates `old_cell` (`occupied[old_cell]` transitions from `true` to `false`) and lands in an already-occupied `new_cell` decreases total `occupancy` by `1` and decreases `rank` by `1` on **every** row where `cell_precedes(d(q, old_cell), old_cell, ad, target_cell)` holds.
2. **Crowding an occupied cell incurs zero rank penalty (`C-R2`):** An occupied rival cell contributes at most `+1` to `rank` whether it holds `1` token or `3,400` tokens. Herding $V = 4,096$ tokens into $C_{\text{occ}} \le 20$ cells bounds $r_{\text{occ}} \le 20$ and $\text{rm} = \max(0, r_{\text{occ}} - 16) \le 4$ on every row, collapsing the $32\,\text{rm}$ term to near zero while bloating the admitted candidate pool $\bar{n}$ to almost the entire vocabulary (`3,403.9` and `3,480.7` tokens/row in `K16-R`).

### 1.2 Occupancy-Preserving Rank Objectives (`FixedGridRankHingeV1` `s[11]=2` & `OccupiedCellRankCrowdHingeV1` `s[11]=3`)

Implemented in [`crates/uor-r4-discrete/src/objective.rs`](../../../crates/uor-r4-discrete/src/objective.rs) (`RankOccupancyPolicy`) and qualified across ten finite controls (`C-R1..C-R6`, `C-R7a..C-R7d`):

1. **Fixed-Grid Cell Rank Hinge ($R_{\text{grid}}$, `C-R7a`):**
   $$r_{\text{grid}}(\text{row}) = 1 + \#\bigl\{c \in \{0,\dots,255\} : \text{cell\_precedes}(d(q, c), c, \text{ad}, c_{\text{target}})\bigr\}, \qquad R_{\text{grid}}(\text{row}) = \max(0,\; r_{\text{grid}}(\text{row}) - K)$$
   Because $r_{\text{grid}}$ ranges over all 256 cells of the fixed $16 \times 16$ $(r_0, r_{B-1})$ root-pair grid regardless of whether `occupied[c]` is `true` or `false`, vacating a cell produces $\Delta r_{\text{grid}} = 0$ and $\Delta R_{\text{grid}} = 0$ on every row (`c_r7a_fixed_grid_vacation_invariance_pass = true`).
2. **Integer Cell-Crowding Hinge ($C_{\text{crowd}}$ above $n_{\max} = 24$, `C-R7b`):**
   $$C_{\text{crowd}}(\text{row}) = \max\bigl(0,\; n(c_{\text{target}}) - 24\bigr) + \sum_{k=0}^{K-1} \max\bigl(0,\; n(\text{admitted}[k]) - 24\bigr)$$
   $$\mathcal{L}_{R\text{-occ}}(\text{row}) = 8\,T(\text{row}) + 32\,R_{\text{grid}}(\text{row}) + 8\,C_{\text{crowd}}(\text{row})$$
   Checked `u64` integer arithmetic (`checked_row_loss_numerator`) returns overflow status `12` (`c_r5_checked_u64_overflow_status_12_pass = true`, `c_r7b_integer_crowding_hinge_above_quota_pass = true`).
3. **Hard Move-Time `G-Occ` Invariant (`C-R7c`):**
   Any candidate endpoint move (`block 0` or `block B-1`) that drops $\text{occupied\_cells} < 128$ or raises $n(c_{\text{new}}) > 32$ is refused (`INELIGIBLE_CANDIDATE_COST = u64::MAX`), enforcing $\text{occupied\_cells} \ge 128 > 16$, $\max_c n(c) \le 32$, and $\bar{n} \le 16 \times 32 = 512 \le 2\,\bar{n}_A$ as a step-by-step invariant (`c_r7c_move_time_g_occ_floor_and_cap_refusal_pass = true`).
4. **Uniform `AddressCellsA8` Equivalence & Within-Cell Invariance (`C-R7d`):**
   Under `AddressCellsA8` (`occupied_cells = 256`, $n(c) = 16 \le 24$ for all $c$), $r_{\text{grid}}(\text{row}) \equiv r_{\text{occ}}(\text{row})$ and $C_{\text{crowd}}(\text{row}) \equiv 0$; every non-endpoint move (`Combine`, `Relation`, interior token blocks `1..B-1`) leaves cell occupancy and multiplicities invariant; and on all `5,009` admitted DEV rows ($r_{\text{grid}}(\text{row}) \le K = 16$), $R_{\text{grid}}(\text{row}) = 0$, so $\mathcal{L}_{R\text{-occ}}(\text{row}) = \mathcal{L}_{R\text{-grid}}(\text{row}) = 8\,T(\text{row})$ provides zero within-cell token-ranking signal (`c_r7d_a8_uniform_equivalence_and_within_cell_invariance_pass = true`).

---

## 2. Two-Seed `G-Occ`, `G-Adm`, and Count-Table Evaluation (`rank-objective-grade.json`)

Reproducible via:
```sh
~/.cargo/bin/cargo run --manifest-path crates/uor-r4-discrete/Cargo.toml --bin stage-t-run -- grade-rank-objective --out docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json
```

### 2.1 `G-Occ`, `G-Adm`, and `DEV` Writer Performance Across Arms (`14,287` DEV rows, `3,847` `P_succ` rows)

| Arm / Run | Policy (`s[11]`) | Occupied Cells (`/ 256`) | Required `G-Occ` Floor | Mean Pool $\bar{n}$ (tokens/row) | Max Cell $n(c)$ | `G-Occ` | DEV `b_3(Full)` (`/ 14,287`, bar `>= 4,894`) | DEV `b_3(P_succ)` (`/ 3,847`, bar `>= 1,106`) | `G-Adm` | DEV `top-1` (`/ 14,287`) | DEV `top-8` (`/ 14,287`) |
| :--- | :--- | ---: | ---: | ---: | ---: | :---: | ---: | ---: | :---: | ---: | ---: |
| `K16-A` Seed 1 (`#2029`) | `NearestAddressMarginV1` (`0`) | `209` | control | `351.8` | — | PASS | `2,237` | — | FAIL | `65` | — |
| `K16-A` Seed 2 (`#2029`) | `NearestAddressMarginV1` (`0`) | `192` | control | `376.1` | — | PASS | `3,167` | — | FAIL | `463` | — |
| `K16-R` Seed 1 (`#2029`) | `OccupiedCellRankHingeV1` (`1`) | **`20`** | `>= 105` | **`3,403.9`** | `> 3,400` | **FAIL** | `12,089` | — | — | `101` | — |
| `K16-R` Seed 2 (`#2029`) | `OccupiedCellRankHingeV1` (`1`) | **`19`** | `>= 96` | **`3,480.7`** | `> 3,480` | **FAIL** | `12,466` | — | — | `230` | — |
| `#2831` `Q1` `FrozenZero` | `NearestAddressMarginV1` (`0`) + `A8` + `D40` | `>= 192` | `>= 128` | `256.0` | `<= 32` | PASS | `4,292` | `1,106` | FAIL | `871` | — |
| `Stage T` `T16_rel` Seeds 1 & 2 (`#2836`) | `NearestAddressMarginV1` (`0`) + `LearnV1` | `256` | `>= 128` | `256.0` | `16` | **PASS** | `3,827` | `1,144` | FAIL | `371` | `1,124` |
| `Stage T` `Arm-A` (`T16_fz`) Seeds 1 & 2 (`#2836`) | `NearestAddressMarginV1` (`0`) + `A8` + `D40` | **`256`** | `>= 128` | **`256.0`** | **`16`** | **PASS** | **`5,009`** | **`1,216`** | **PASS** | `455` | `1,484` |
| `Arm-R-Grid` Seeds 1 & 2 (`#2883`, `OuterL1V1`) | `FixedGridRankHingeV1` (`2`) + `A8` + `D40` | **`256`** | `>= 128` | **`256.0`** | **`16`** | **PASS** | **`5,009`** | **`1,216`** | **PASS** | `455` | `1,484` |
| `Arm-R-Occ` Seeds 1 & 2 (`#2883`, `OuterL1V1`) | `OccupiedCellRankCrowdHingeV1` (`3`) + `A8` + `D40` | **`256`** | `>= 128` | **`256.0`** | **`16`** | **PASS** | **`5,009`** | **`1,216`** | **PASS** | `455` | `1,484` |
| `Arm-R-Occ` / `T16_fz` + `WithinCellCodeRerankerV1` (`#3267`) | `R_occ` / `A` (`256` cells, `b_3 = 5,009`) + `R_cell(c)` | **`256`** | `>= 128` | **`256.0`** | **`16`** | **PASS** | **`5,009`** | **`1,216`** | **PASS** | **`2,140`** (`2,142` cell-prio) | **`4,006`** |

### 2.2 Evaluation Against Lawful Count-Table Baselines (`14,287` DEV rows)

| Baseline / Model | DEV `top-1` (`/ 14,287`) | DEV `top-8` (`/ 14,287`) | DEV `P_succ top-1` (`/ 3,847`) | Comparison vs `Arm-R-Occ` (`OuterL1V1` = `455`) | Comparison vs `G-Occ` Admission + `WithinCellCodeRerankerV1` (`2,140`) |
| :--- | ---: | ---: | ---: | :--- | :--- |
| Unconditional Unigram `U` | `1,226` | — | — | `455 < 1,226` (`-771`) | **`2,140 > 1,226` (`+914`, PASS)** |
| Admitted-set Unigram `U_restr(A_W)` | `1,127` | — | — | `455 < 1,127` (`-672`) | **`2,140 > 1,127` (`+1,013`, PASS)** |
| Admitted-set Symmetric Code `T_sym_restr(A_W)` | `1,909` | — | — | `455 < 1,909` (`-1,454`) | **`2,140 > 1,909` (`+231`, PASS)** |
| Admitted-set Read-Code Table `T_restr(A_W)` | `2,140` | `4,006` | `376` | `455 < 2,140` (`-1,685`) | **`2,140 == 2,140` (`2,142 > 2,140` cell-prio)** |
| Admitted-set Bigram-x-Code `G_restr(A_W)` | `3,084` | `4,674` | `517` | `455 < 3,084` (`-2,629`) | `2,140 < 3,084` (`-944`) |
| Full-Panel Read-Code Table `T(c(x))` | `4,085` | — | — | `455 < 4,085` (`-3,630`) | `2,140 < 4,085` (`-1,945`) |
| Unconditional Bigram `T2` (M1 closing bar) | `6,300` | — | — | `455 < 6,300` (`-5,845`) | `2,140 < 6,300` (`-4,160`) |

### 2.3 Verdict: `FAIL_RANK_GAIN_AT_HIGH_OCC` (`mechanism_negative: false`)

1. **Occupancy & Admission (`G-Occ` and `G-Adm` PASS):** `FixedGridRankHingeV1` (`R_grid`, `s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`R_occ`, `s[11]=3`) provably eliminate `K16-R`'s `19–20`-cell collapse (`C-R7a..C-R7c`), holding `occupied_cells = 256 / 256 > 16` (`>= 128`), $\bar{n} = 256.0 \le 512.0$, $\max_c n(c) = 16 \le 32$, `DEV b_3(Full) = 5,009 / 14,287 >= 4,894`, and `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106` across both seeds.
2. **Why Cell-Only Rank Hinges Do Not Move `OuterL1V1` `top-1` (`G-Rank` FAIL):** On `T16_fz`'s `5,009` admitted DEV rows, `4,208 / 5,009` (`84.01%`) of admitted targets lose to a rival token **inside their own cell `target_cell`** (`1,597 / 1,765 = 90.48%` of missed `T_restr(A_W)` hits), where $R_{\text{grid}}(\text{row}) = 0$ and $C_{\text{crowd}}(\text{row}) = 0$ are constant across all 16 tokens in `target_cell` (`C-R7d`). Replacing single-vector `OuterL1V1` within-cell scoring with `WithinCellCodeRerankerV1` (`#3267`) on the exact same `G-Occ`-compliant `5,009` admitted cells immediately lifts `DEV top-1` from `455` to **`2,140 / 14,287`** (`2,142` cell-priority, beating `Unigram = 1,226` by `+914` rows).
