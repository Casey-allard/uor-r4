# D68 Decision Card: Occupancy-Preserving Rank Objective (`r-rank-objective-keeps-cells`, `#2883`)

- **Lab:** `antigravity`
- **Order:** [`docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/ORDER.md`](ORDER.md)
- **Item / Issue:** [`docs/roadmap/items/r-rank-objective-keeps-cells.md`](../../../../roadmap/items/r-rank-objective-keeps-cells.md) ([`#2883`](https://github.com/UOR-Foundation/uor-r4/issues/2883))
- **Pre-Registration Sheet:** [`docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`](../../../../harness/prep/r-rank-objective-keeps-cells-2026-10-25.md) (`TEST FITNESS: FIT`, [`#2883` comment `6396718125`](https://github.com/UOR-Foundation/uor-r4/issues/2883#issuecomment-6396718125))
- **Authority:** `CHARTER.md` §3.5 (*"Spend GPU only where a run can change a decision"*), §4.1–§4.6, §5.1, D67, D68, D69

---

## 1. Whole-Project Context & What `#2883` Tests

1. **Why `#2883` Exists:**
   - In the `K16-A` vs `K16-R` GPU run (`#2029` comment `6257335075`, [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../../../../labs/k16-outcome-call-2026-10-19/README.md)), `OccupiedCellRankHingeV1` (`R`, `s[11] = 1`) collapsed from `16` initial cells to **`20 / 256`** (Seed 1, $\bar{n}_R = 3,403.9$) and **`19 / 256`** (Seed 2, $\bar{n}_R = 3,480.7$) occupied cells vs `209 / 256` ($\bar{n}_A = 351.8$) and `192 / 256` ($\bar{n}_A = 376.1$) under `NearestAddressMarginV1` (`A`, `s[11] = 0`).
   - This triggered `OCCUPANCY_COLLAPSE_TRIGGERED` on both `G-Occ` clauses (`occupied_cells < max(32, ceil(0.5 * occ_A))` and $\bar{n}_R > 2 \cdot \bar{n}_A$) because `if (occupied[cell]) ++rank` (`discrete.cu:401-406`, historically `383-388`) rewards vacating preceding cells (`-1` rank across the dataset whenever a cell is emptied) and charges zero penalty for crowding thousands of tokens into a single cell (`C-R2`).
   - Roadmap item `r-rank-objective-keeps-cells` (`#2883`) was created with done-when:
     > *"A rank objective that passes G-Occ (more than 16 of 256 cells, plus the D39 condition) and is evaluated against the count tables."*
2. **Pre-Registration (`antigravity-67`, `#3212`):**
   - `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md` (`TEST FITNESS: FIT`) pre-registered `FixedGridRankHingeV1` (`Arm-R-Grid`, `s[11] = 2`) and `OccupiedCellRankCrowdHingeV1` (`Arm-R-Occ`, `s[11] = 3`):
     $$r_{\text{grid}}(\text{row}) = 1 + \#\bigl\{c \in \{0,\dots,255\} : \text{cell\_precedes}(d(q, c), c, \text{ad}, c_{\text{target}})\bigr\}, \qquad R_{\text{grid}}(\text{row}) = \max(0,\; r_{\text{grid}}(\text{row}) - K)$$
     $$C_{\text{crowd}}(\text{row}) = \max\bigl(0,\; n(c_{\text{target}}) - 24\bigr) + \sum_{k=0}^{K-1} \max\bigl(0,\; n(\text{admitted}[k]) - 24\bigr)$$
     $$\mathcal{L}_{R\text{-occ}}(\text{row}) = 8\,T(\text{row}) + 32\,R_{\text{grid}}(\text{row}) + 8\,C_{\text{crowd}}(\text{row})$$
     together with a move-time `G-Occ` invariant refusing any endpoint move (`block 0` or `block B-1`) that drops $\text{occupied\_cells} < 128$ or raises $n(c_{\text{new}}) > 32$.
   - Execution was held until `m1-bvg-k-stage-t-training` (`#2836`) completed and `crates/uor-r4-discrete/**` was added to `#2883`'s scope. Both prerequisites are now complete (`#3239`, `#3268`).

---

## 2. Upstream GPU Evidence Now Sealed on `main` (`#2831`, `#2836`, `#3267`)

Since `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md` was written on 25 Oct 2026, two decisive GPU units have completed and sealed their artifacts:

1. **Two-Seed Stage T `BVG-K` (`K = 16`) GPU Run (`#2836`, PR `#3239`, HF archive `stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`, [`docs/labs/m1-bvg-k-stage-t-training/grade.json`](../../../../labs/m1-bvg-k-stage-t-training/grade.json) `7917ec85941d919db14ab67567fe824c87d6d2edf18bbd54138bf320b6ea4c81`):**
   - Under `AddressCellsA8` (`256 / 256` cells occupied at `16` tokens/cell) + `D40` (`3,847`-row TRAIN anchor floor `>= 1,187`) + `BVG-K` (`K = 16` coordinates/step), **`Arm-A` (`T16_fz`, `NearestAddressMarginV1`) already passes both `G-Occ` and `G-Adm` on both seeds (`1` and `2`):**
     - `G-Occ`: `occupied_cells = 256 / 256 > 16` (`>= max(32, ceil(0.5 * 256)) = 128`), `mean_admitted_tokens = 256 <= 2 * 256 = 512`, `max_cell_tokens = 16 <= 32` (**`G-Occ PASS`**).
     - `G-Adm`: `DEV b_3(Full) = 5,009 / 14,287 >= 4,894` (`+765` over cp0 `4,244`, `+717` over `#2831` `Q1` `4,292`), `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106` (**`G-Adm PASS`**).
     - `DEV writer top-1` (`OuterL1V1`): `455 / 14,287` (`top-8 = 1,484 / 14,287`, `P_succ top-1 = 107 / 3,847`, `P_succ top-8 = 318 / 3,847`, `strict_wins = 93`; `T16_rel = 371 / 14,287`, `top-8 = 1,124 / 14,287`, `P_succ top-1 = 78 / 3,847`, `P_succ top-8 = 243 / 3,847`, `strict_wins = 172`).
     - Lawful count-table baselines on the `14,287`-row DEV panel (`t16-frozen-zero-seed1/w2-summary.json` `19009681fc7f81cd3af581b77b10e418d57d1437eb07f7ecd139cfcd5187f22c`, `t16-relation-seed1/w2-summary.json` `565e0aa10c6ec93fdaa8b70e510f23dfd612d2602826ff9176b3986861089e10`):
       - Unconditional Unigram `U`: `1,226 / 14,287`
       - Unconditional Bigram `T2`: `6,300 / 14,287`
       - Full-panel read-code table `T(c(x))`: `4,085 / 14,287` (`T16_rel = 3,680`)
       - Admitted-set restricted unigram `U_restr(A_W)`: `1,127 / 14,287` (`T16_rel = 942`)
       - Admitted-set restricted symmetric code table `T_sym_restr(A_W)`: `1,909 / 14,287` (`T16_rel = 1,377`)
       - Admitted-set restricted read-code table `T_restr(A_W)`: `2,140 / 14,287` (`top-8 = 4,006`, `P_succ = 376 / 3,847`; `T16_rel = 1,602`, `top-8 = 3,007`, `P_succ = 327 / 3,847`)
       - Admitted-set restricted bigram-x-code table `G_restr(A_W)`: `3,084 / 14,287` (`top-8 = 4,674`, `P_succ = 517 / 3,847`; `T16_rel = 2,364`, `top-8 = 3,529`, `P_succ = 496 / 3,847`).

2. **Within-Cell Read-Code Reranker Decomposition (`#3267`, PR `#3280`, [`docs/labs/m1-within-cell-writer/stage-t-decomposition.json`](../../../../labs/m1-within-cell-writer/stage-t-decomposition.json) `e9dd940c0f58a1c494bb5d8f38a485528b6693bcaa67b23df3f1d867bec76cc6`):**
   - On `T16_fz`'s `5,009` admitted DEV rows (`G-Occ PASS`, `G-Adm PASS`):
     - **`4,208 / 5,009` (`84.01%`) of admitted DEV target tokens lose to a rival token inside their own endpoint cell `target_cell`** (`admitted_target_loses_within_own_cell_rows = 4,208`).
     - Of the `1,765` DEV rows where `T_restr(A_W)` (`2,140`) hits `top-1` and `OuterL1V1` (`455`) misses (`t_restr_hits_w_misses = 1,765`), **`1,597 / 1,765` (`90.48%`, `415` within-cell only + `1,182` both within-cell and cross-cell, with `168` cross-cell only) lose to a rival token inside `target_cell`**.
     - Replacing `OuterL1V1` within-cell ranking on the exact same `5,009` admitted cells with `WithinCellCodeRerankerV1` (`#3267`) immediately raises `DEV top-1` from `455` to **`2,140 / 14,287`** (`reranker_code_pooled`, `top-8 = 4,006`, `+1,685` rows over `455`, beating `Unigram = 1,226` by `+914` rows, `U_restr(A_W) = 1,127` by `+1,013` rows, `T_sym_restr(A_W) = 1,909` by `+231` rows, and `reranker_code_permuted_null = 753` by `+1,387` rows), **`2,142 / 14,287`** (`reranker_code_cell_prio`, `+1,687` rows over `455`), and **`1,706 / 14,287`** (`reranker_code_top1_cell_only`, `+1,251` rows over `455`), while keeping `occupied_cells = 256 / 256`, `G-Occ = PASS`, and `b_3(Full) = 5,009` identical.

---

## 3. Proof of Decision-Table Outcome Invariance (`C-R7a..C-R7d`)

Why a 6-run (`~2.2h`, `~$2.40`) `16,404`-batch GPU rerun of `Arm-R-Grid` and `Arm-R-Occ` under `OuterL1V1` cannot change the registered decision-table outcome (`FAIL_RANK_GAIN_AT_HIGH_OCC`):

1. **`R_occ` and `R_grid` Provably Satisfy `G-Occ` (`C-R7a`, `C-R7b`, `C-R7c`):**
   - By `C-R7a`, $r_{\text{grid}}(\text{row})$ sums over all 256 cells of the fixed $16 \times 16$ $(r_0, r_3)$ grid regardless of `occupied[c]`; vacating a cell produces $\Delta r_{\text{grid}} = 0$ and $\Delta R_{\text{grid}} = 0$ on every row, removing the `K16-R` collapse gradient.
   - By `C-R7b`, any cell with $n(c) > 24$ pays $+8\,(n(c) - 24)$ on every row targeting or admitting $c$.
   - By `C-R7c`, any endpoint move (`block 0` or `block B-1`) that drops $\text{occupied\_cells} < 128$ or raises $n(c_{\text{new}}) > 32$ returns `~(W)0` (`u64::MAX`), enforcing $\text{occupied\_cells} \ge 128 > 16$, $\max_c n(c) \le 32$, and $\bar{n} \le 16 \times 32 = 512 \le 2\,\bar{n}_A$ at every step.
2. **Algebraic Identity and Zero Within-Cell Signal Under `AddressCellsA8` (`C-R7d`):**
   - At `AddressCellsA8` initialization (`256 / 256` cells occupied with $n(c) = 16 \le 24$ for all $c$) and throughout every `Combine` move (`gates 0..20`), `Relation` move (`gates 20..45`), and interior token-vector move (`block 1` and `block 2`), token endpoint cells $(r_0, r_3)$ are invariant: `occupied[c] = true` for all $256$ cells and $n(c) = 16 \le 24$, so $r_{\text{grid}}(\text{row}) \equiv r_{\text{occ}}(\text{row})$ and $C_{\text{crowd}}(\text{row}) \equiv 0$.
   - On every row where `target_cell` is admitted ($r_{\text{grid}}(\text{row}) \le K = 16$ — which holds on all `5,009` admitted DEV rows of `T16_fz`), $R_{\text{grid}}(\text{row}) = \max(0, r_{\text{grid}}(\text{row}) - 16) = 0$ is identically zero!
   - Even on non-admitted rows ($r_{\text{grid}}(\text{row}) > 16$), $R_{\text{grid}}(\text{row})$ and $C_{\text{crowd}}(\text{row})$ depend **only on the 8-bit cell id $(r_0, r_3)$** and are invariant across all `16` vocabulary tokens sharing `target_cell`.
3. **Registered Decision-Table Resolution (`FAIL_RANK_GAIN_AT_HIGH_OCC`):**
   - Because `Arm-A` (`T16_fz`) already passes `G-Occ` (`256 / 256` cells) and `G-Adm` (`5,009 >= 4,894`), and `84.01%` (`4,208 / 5,009`) of admitted DEV targets lose **within their own cell** where $R_{\text{grid}} = 0$ and $C_{\text{crowd}} = 0$ have zero gradient (`#3267`), replacing $32\,A$ with $32\,R_{\text{grid}} + 8\,C_{\text{crowd}}$ under single-vector `OuterL1V1` scoring resolves to the third row of the registered decision table (`docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md` §3):
     > **`FAIL_RANK_GAIN_AT_HIGH_OCC`:** *"`Arm-R-Occ` holds `G-Occ` ($\text{occ} \ge 128$, $\bar{n} \le 2 \bar{n}_A$) and `G-Adm` ($\ge 4,894$), but `DEV top-1` fails to beat `Arm-A` by $\ge +192$ rows — Confirms cell rank alone is not the remaining top-1 bottleneck once `G-Occ` and `D40` hold; retain `NearestAddressMarginV1` (`A`) and pivot to within-cell token-distance capacity (`#2834`, `#2877`, `#3267`)."*

---

## 4. Decision & Action Plan

1. **Approve `FAIL_RANK_GAIN_AT_HIGH_OCC` (`mechanism_negative: false`) under CHARTER §3.5, §5.1, and D68:**
   - Do not spend `$2.40` / `2.2 GPU-hours` on a 6-run `16,404`-batch rerun whose decision-table outcome is invariant (`CHARTER.md` §3.5: *"Spend GPU only where a run can change a decision"*).
   - Implement `RankOccupancyPolicy` (`NearestAddressMarginV1` `word = 0`, `OccupiedCellRankHingeV1` `word = 1`, `FixedGridRankHingeV1` `word = 2`, `OccupiedCellRankCrowdHingeV1` `word = 3`), exact integer row-loss evaluation (`8*T + 32*R_grid + 8*C_crowd` with `u64` overflow status `12`), move-time `G-Occ` refusal (`occ >= 128`, `n(c) <= 32`), and finite controls `C-R1..C-R6` and `C-R7a..C-R7d` in `crates/uor-r4-discrete/src/objective.rs` without modifying `discrete.cu` or `driver_source_sha256`.
   - Implement `grade-rank-objective` in `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs` to generate the pinned grade artifact `docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json` (`uor-r4.rank-objective-keeps-cells-grade/v1`), grading `K16-A`, `K16-R`, `#2831` `Q1`, `#2836` `Stage T` (`T16_fz` and `T16_rel`), `#3267` (`WithinCellCodeRerankerV1`), and `R_grid` / `R_occ` against `G-Occ`, `G-Adm`, and the count tables (`U = 1,226`, `T2 = 6,300`, `T(c(x)) = 4,085`, `U_restr(A_W) = 1,127`, `T_sym_restr(A_W) = 1,909`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`).
   - Close `r-rank-objective-keeps-cells` (`#2883`, `status: done`) with one notebook row in `docs/research/notebook/m1-discrete-attention.md`.
2. **Reversibility:**
   - All `R_grid` (`s[11]=2`) and `R_occ` (`s[11]=3`) evaluators and finite controls (`C-R1..C-R7d`) remain in `crates/uor-r4-discrete/src/objective.rs` (`READY` in `docs/mechanisms/rank-margin.md`). If Casey or a future multi-cell sparse initialization (`occ_0 < 256`) wants a full 6-run `R_occ` GPU sweep, it can be dispatched immediately.

---

## 5. Fresh-Context D68 Decision Review Record

- **Reviewer:** Fresh-context `research` subagent `96f12503-9939-466d-83c0-3268bf63d4ab` (`antigravity`, CHARTER §5.1, D68)
- **Reviewed Commit:** `042105d6c` (`DECISION.md` `3cbd4bf7e1040f4be294b21f60d1802e7ec0f9564ed38209e4498c656a09bba2`)
- **Verdict:** `D68 DECISION REVIEW: APPROVE`
