# Pre-Registration: Occupancy-Preserving Rank Objective (`r-rank-objective-keeps-cells`)

- **Issue / Roadmap Item:** [`#2883`](https://github.com/UOR-Foundation/uor-r4/issues/2883) (`docs/roadmap/items/r-rank-objective-keeps-cells.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/training.md`)
- **Author Lab:** `antigravity`
- **Date:** 2026-10-25
- **Upstream Prerequisites:**
  - `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`, merged `cb2b5748905dc77b122325101b62dd7c29fc1413` — `status: done`)
  - `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md` — `BVG-K` `K = 16` batched-coordinate training)

---

## 1. Question and Base Artifact

**Question:** Does replacing the occupancy-collapsing rank hinge `OccupiedCellRankHingeV1` (`R`, which counts only occupied cells $r_{\text{occ}}$ and pays zero penalty for vacating cells or crowding thousands of tokens into $19\text{--}20$ cells) with an **occupancy-preserving integer rank objective** (`OccupiedCellRankCrowdHingeV1`, `R_occ`, combining fixed-grid cell rank $R_{\text{grid}}$ over all $256$ root-pair cells plus an integer cell-crowding hinge $C_{\text{crowd}}$ above $n_{\max} = 24$ tokens/cell and a hard move-time `G-Occ` floor $\text{occ} \ge 128$, $\max_c n(c) \le 32$) hold cell occupancy well above `G-Occ` ($\text{occ}_{R\text{-occ}} \ge \max(32, \lceil 0.5 \cdot \text{occ}_A \rceil)$ and $\bar{n}_{R\text{-occ}} \le 2 \cdot \bar{n}_A$) while improving full-panel admission (`b_3(Full) >= 4,894 / 14,287`) and DEV `top-1` over `NearestAddressMarginV1` (`A`)?

- **Why `OccupiedCellRankHingeV1` (`R`) collapsed in `K16-R` (exact diagnosis from `crates/uor-r4-discrete/src/discrete.cu` lines 383–388):**
  1. In `writer(...)` (`discrete.cu:383-388`), `rank` is incremented (`++rank`) only when `if (occupied[cell])` holds and `cell_precedes(distance, cell, ad, target_cell)`, and each occupied cell counts once regardless of token multiplicity (`C-R2`).
  2. Whenever a virtual move vacates `old_cell` (`occupied[old_cell] = false`) and lands in an already-occupied `new_cell` (`occupied[new_cell] = true`), total occupied cells `occupancy` drops by `1` and `rank` drops by `1` for every row whose `target_cell` was preceded by `old_cell`, while `new_cell` still contributes at most `1` to `rank`.
  3. In `K16-R` (`docs/labs/k16-outcome-call-2026-10-19/README.md`), this gradient toward cell vacation herded 4,096 tokens into `20` (seed 1) and `19` (seed 2) occupied cells (`\bar{n}_R = 3,403.9` and `3,480.7` admitted tokens/row vs `\bar{n}_A = 351.8` and `376.1` under `A` with `209` and `192` occupied cells), triggering `G-Occ` (`OCCUPANCY_COLLAPSE_TRIGGERED`) on both clauses and capping `DEV top-1` at `101` and `230` (below `A` seed 2's `463`).
- **Exact Specification of `OccupiedCellRankCrowdHingeV1` (`R_occ`, `s[11] == 2`):**
  1. **Fixed-Grid Cell Rank Hinge ($R_{\text{grid}}$):** Count all $256$ cells of the $16 \times 16$ $(r_{\text{first}}, r_{\text{last}})$ root-pair grid that strictly precede `target_cell` in `(address_distance(q, g, s, cell), cell)`:
     $$r_{\text{grid}}(\text{row}) = 1 + \#\bigl\{c \in \{0,\dots,255\} : \text{cell\_precedes}(d(q, c), c, \text{ad}, c_{\text{target}})\bigr\}, \qquad R_{\text{grid}}(\text{row}) = \max(0,\; r_{\text{grid}}(\text{row}) - K)$$
     Because $r_{\text{grid}}$ ranges over all $256$ geometric cells whether occupied or empty, vacating a cell never reduces $r_{\text{grid}}$ or $R_{\text{grid}}$.
  2. **Integer Cell-Crowding Hinge ($C_{\text{crowd}}$):** Let $n(c) = \#\{v \in \{0,\dots,V-1\} : \text{token\_cell}(v) = c\}$ be the exact integer token multiplicity in cell $c$ (uniform A8 baseline $n_0 = 4096 / 256 = 16$). With crowd threshold $n_{\max} = 24$ ($1.5 \times n_0$), define:
     $$C_{\text{crowd}}(\text{row}) = \max\bigl(0,\; n(c_{\text{target}}) - 24\bigr) + \sum_{k=0}^{K-1} \max\bigl(0,\; n(\text{admitted}[k]) - 24\bigr)$$
  3. **Integer Row Loss Numerator ($\mathcal{L}_{R\text{-occ}}$, D32-compliant `u64` with `checked_scaled_add` overflow status `12`):**
     $$\mathcal{L}_{R\text{-occ}}(\text{row}) = 8\,T(\text{row}) + w\,R_{\text{grid}}(\text{row}) + 8\,C_{\text{crowd}}(\text{row}), \qquad w = 8\,B = 32$$
  4. **Move-Time `G-Occ` Invariant:** Any candidate coordinate move on block `0` or block `B - 1` (`s[1] - 1`) that results in $\text{occupancy} < 128$ or $n(c_{\text{new}}) > 32$ is refused at candidate evaluation (`cost = ~(W)0`), guaranteeing $\text{occ} \ge 128$ and $\bar{n} \le 16 \times 32 = 512$ at every step.
- **Base Artifact & Pinned Inputs:**
  - **Codebook & Cell Layout:** `PrimeZeta` injective codebook (`4,096 / 4,096` distinct token vectors, $B = 4$, $d = 32$) with `AddressCellsA8` (`256 / 256` cells occupied at `16` tokens/cell at init, `b_3(P_succ) = 1,351 / 3,847`, `b_3(Full) = 4,244 / 14,287`).
  - **Active Admission Controller (`D40`):** `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, fixed 3,847-row TRAIN anchor manifest `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` (`floor = 1,187 / 3,847`).
  - **Pinned Corpus Shards (BPE `d36d3e87`, $V = 4,096$, $L = 128$):**
    - `train.bin`: SHA-256 `49263fe24ecc3ad2d8bfabbc6dc1fbd95fb5fbecbefdfbe0fa0ee6bfefcf0d6f` (`19,165` sequences, `2,425,925` targets)
    - `val.bin` (`DEV` panel): SHA-256 `53dfca085fa4be04875e424eefeb43952d0b44ab6d5d6ec2d590ae7cf5ff7e71` (`14,287` evaluation positions; `P_succ = 3,847` rows)
    - `fresh_v2` transfer slice (`crates/uor-r4-discrete/fixtures/fresh_panel_v2.bin`): SHA-256 `c1f40ab0578d20ca337282b9efbe12b29f21f99a11cb17689708ee46f2181167` (`384` rows; sealed until a DEV gate passes)

---

## 2. Arms, Data Split, and Steps

All arms run on a single NVIDIA GeForce RTX 5090 (`sm_120`) under the Stage T `BVG-K` (`K = 16` coordinates/step) + A8 + `D40` recipe across two deterministic seeds (`seed in {1, 2}`) for `16,404` batches (`64` rows/batch):

| Arm ID | Objective Policy | Formula ($\mathcal{L}(\text{row})$, $w = 32$) | Move-Time `G-Occ` Guard | Role |
| :--- | :--- | :--- | :--- | :--- |
| **`Arm-A`** (`NearestAddressMarginV1`, `s[11] = 0`) | Control (`A`) | $8\,T + 32\,A$, $A = \max(0, 8 + d_{\text{addr}}(q, c_{\text{target}}) - d_{\text{addr}}^{\text{rival}})$ | None (naturally holds $\text{occ} \ge 192$ in K16-A) | Baseline nearest-address margin control |
| **`Arm-R-Grid`** (`FixedGridRankHingeV1`, `s[11] = 2`, $\lambda_{\text{crowd}} = 0$) | Ablation (`R_grid` only) | $8\,T + 32\,R_{\text{grid}}$, $R_{\text{grid}} = \max(0, r_{\text{grid}} - 16)$ over all $256$ cells | $\text{occ} \ge 128$, $n(c) \le 32$ | Isolates fixed-grid cell rank from the explicit crowd hinge |
| **`Arm-R-Occ`** (`OccupiedCellRankCrowdHingeV1`, `s[11] = 3`) | Treatment (`R_occ`) | $8\,T + 32\,R_{\text{grid}} + 8\,C_{\text{crowd}}$ ($n_{\max} = 24$) | $\text{occ} \ge 128$, $n(c) \le 32$ | Primary occupancy-preserving rank objective |

- **Data Split:**
  - **Training:** `train.bin` (`19,165` sequences, `2,425,925` targets) + 3,847-row `D40` TRAIN anchor manifest `f85466ff...`.
  - **Development (`DEV`):** `val.bin` (`14,287` positions; `P_succ = 3,847` rows), evaluated at `batch = 0`, `Q1 = 4,096`, `Q2 = 8,192`, and `16,404`.
  - **Fresh Transfer (`fresh_v2`):** `384`-row sealed slice, evaluated once at the terminal checkpoint if and only if an arm passes all DEV bars (`G-Occ`, `G-Adm`, `G-Rank`).
- **Execution Steps (Probe Ladder):**
  1. **Step 1 — Finite CUDA Control Qualification (`C-R1..C-R7`, ~3 min):** Execute finite device controls verifying `R_grid` invariance under cell vacation (`C-R7a`), exact positive integer crowd penalty $8\,C_{\text{crowd}}$ when $n(c) > 24$ (`C-R7b`), move-time refusal when $\text{occ} < 128$ or $n(c) > 32$ (`C-R7c`), virtual-vs-applied identity (`C-R3`), and `u64` overflow status `12` (`C-R5`).
  2. **Step 2 — 256-Batch Throughput & Occupancy Probe (~5 min):** Run `256` batches of `Arm-R-Occ` (seed 1) on RTX 5090, confirming projected wall time $\le 45$ min/run and $\text{occ} \ge 200 / 256$.
  3. **Step 3 — Paired 3-Arm $\times$ 2-Seed GPU Run (`6` runs):** Execute `Arm-A`, `Arm-R-Grid`, and `Arm-R-Occ` on seeds `1` and `2`, recording DEV `top-1`, `top-8`, `b_3(Full)`, `b_3(P_succ)`, `occupied_cells`, $\bar{n}$, count-table comparisons, and decode `tokens/s`.

---

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum Test Power (CHARTER §4.3):**
  - `2` independent seeds (`seed in {1, 2}`) per arm across all `14,287` DEV positions (`3,847` `P_succ` rows) plus the `384`-row `fresh_v2` transfer panel.
  - Observed K16-A two-seed range on DEV `top-1` is `|463 - 65| = 398` rows; bars require both seeds to pass `G-Occ` and `G-Adm` and require a mean `DEV top-1` gain $\ge +192$ rows (`+1.34%`) with non-regression on every seed.
- **Registered Bars:**
  1. **Bar `G-Occ` (Occupancy & Crowd Bar — D39 / K16 Contract):** On **both** seeds (`1` and `2`), at `Q1`, `Q2`, and `16,404`:
     $$\text{occ} \ge \max\bigl(32,\; \lceil 0.5 \cdot \text{occ}_{\text{Arm-A}} \rceil\bigr) \quad \text{AND} \quad \bar{n} \le 2 \cdot \bar{n}_{\text{Arm-A}} \quad \text{AND} \quad \max_{c} n(c) \le 32$$
  2. **Bar `G-Adm` (Full-Panel & Copy-Target Admission Bar):** On **both** seeds (`1` and `2`), DEV full-panel admission $b_3(\text{Full}) \ge 4,894 / 14,287$ (`K16` admission bar, vs `4,292` at `#2831` `Q1`) and DEV copy-target admission $b_3(P_{\text{succ}}) \ge 1,106 / 3,847$.
  3. **Bar `G-Rank` (Ranking & Count-Table Bar):** Mean DEV `top-1` across seeds `1` and `2` exceeds `Arm-A` by $\ge +192$ rows, **both** seeds beat the `#2831` `Q1` mark (`871 / 14,287`), and both seeds are scored against the lawful count-table baselines (`Unigram = 1,226 / 14,287`, `T2 successor = 6,300 / 14,287`, CHARTER §4.8) with decode `tokens/s` non-regressing (`>= 95%` of `Arm-A`).
  4. **Bar `G-Transfer` (Fresh-Panel Transfer):** Positive `fresh_v2` `top-1` and `b_3(Full)` gain over `Arm-A` on both seeds.
- **Decision Table:**

| Outcome | Condition | Decision & What It Unlocks |
| :--- | :--- | :--- |
| **`PASS_RANK_OCC_AND_UNIGRAM`** | `Arm-R-Occ` (or `Arm-R-Grid`) passes `G-Occ`, `G-Adm`, `G-Rank`, `G-Transfer`, and beats `Unigram` (`1,226 / 14,287`) on both seeds | Promote the winning objective to `ACTIVE` in `docs/mechanisms/rank-margin.md`, close `#2883` (`status: done`), and adopt it for `m1-close-saved-model` (`#2837`). |
| **`PASS_RANK_OCC_KEEP_ITERATE`** | `Arm-R-Occ` passes `G-Occ`, `G-Adm`, `G-Rank`, and `G-Transfer` (`> 871` on both seeds, $\ge +192$ over `Arm-A`), but stays below `Unigram` (`1,226`) | Close `#2883` (`status: done` on its `G-Occ` + count-table evaluation contract) and combine `R_occ` with `r-full-epoch-revisit-schedules` (`#2878`) and `m1-address-key-full-vector` (`#2834`). |
| **`FAIL_RANK_GAIN_AT_HIGH_OCC`** | `Arm-R-Occ` holds `G-Occ` ($\text{occ} \ge 128$, $\bar{n} \le 2 \bar{n}_A$) and `G-Adm` ($\ge 4,894$), but `DEV top-1` fails to beat `Arm-A` by $\ge +192$ rows | Confirms cell rank alone is not the remaining top-1 bottleneck once `G-Occ` and `D40` hold; retain `NearestAddressMarginV1` (`A`) and pivot to within-cell token-distance capacity (`#2834`, `#2877`). |
| **`INSTRUMENT_STOP`** | Any finite control `C-R1..C-R7` fails or `status == 12` overflow fires | Classify as `instrument` (CHARTER §4.2), repair kernel before spending GPU budget. |

---

## 4. Cost and Stop Conditions

- **Estimated Compute & Cost:**
  - `1 x NVIDIA GeForce RTX 5090` (`~$0.65/h`–`$1.05/h` on Vast.ai, inside the 4-pod / `$8/h` fleet cap).
  - Step 1 (`C-R1..C-R7`) + Step 2 (`256`-batch probe): `~8 min`.
  - Step 3 (`6` runs $\times$ `~22 min/run` under `BVG-K` `K = 16`): `~2.2 hours` (`~$1.50`–`$2.40` total).
- **Stop Conditions:**
  - **Immediate Instrument Stop:** Abort before Step 3 if any finite control `C-R1..C-R7` fails or if Step 2 projects $> 60$ min/run.
  - **Mid-Run Occupancy Stop:** If `occupied_cells < 128` or $\bar{n} > 2 \cdot \bar{n}_A$ at `Q1` (`4,096` batches) on seed 1, abort that arm immediately (`G-Occ` breach).
  - **Scope & Dependency Gate:** Execution of Steps 1–3 is held (`status: held`) until `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) completes and `r-rank-objective-keeps-cells` scope is widened to include `crates/uor-r4-discrete/**`.

---

## 5. Review Verdict

- **Reviewer:** Fresh-context non-author `research` subagent `de3909b1-e564-4390-8228-77ba6927e041` (`antigravity`, CHARTER §4.5, §5.1, D68)
- **Posted Record:** Issue [`#2883` comment `6396718125`](https://github.com/UOR-Foundation/uor-r4/issues/2883#issuecomment-6396718125)

TEST FITNESS: FIT
