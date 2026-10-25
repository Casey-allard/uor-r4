# Occupancy-Preserving Rank Objective (`r-rank-objective-keeps-cells`, `#2883`)

- **Issue / Roadmap Item:** [`#2883`](https://github.com/UOR-Foundation/uor-r4/issues/2883) (`docs/roadmap/items/r-rank-objective-keeps-cells.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/training.md`)
- **Lab:** `antigravity` (ORDER [`docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-67/ORDER.md`](../../loop/tasks/r-rank-objective-keeps-cells/antigravity-67/ORDER.md))
- **Pre-Registration Sheet:** [`docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`](../../harness/prep/r-rank-objective-keeps-cells-2026-10-25.md) (`TEST FITNESS: FIT`, [`#2883` comment `6396718125`](https://github.com/UOR-Foundation/uor-r4/issues/2883#issuecomment-6396718125))
- **Current Status:** `held` (`status: ready -> status: held` via D68 decision review by `de3909b1-e564-4390-8228-77ba6927e041`; pre-registration sheet and exact kernel design complete; GPU execution gated on `#2836` `m1-bvg-k-stage-t-training` and widening item scope to `crates/uor-r4-discrete/**`)

---

## 1. Cancel Probe & Root-Cause Proof of the `K16-R` Occupancy Collapse

### 1.1 Exact Kernel Mechanism Behind `OCCUPANCY_COLLAPSE_TRIGGERED`

In `crates/uor-r4-discrete/src/discrete.cu` lines 383–394, the existing rank objective `OccupiedCellRankHingeV1` (`s[11] == 1`) computes:

```cuda
U rank=1,occupancy=0;
if(s[11]==1 || telemetry)for(U cell=0;cell<256;++cell)if(occupied[cell]){
    ++occupancy;U distance=address_distance(q,g,s,cell);
    if(cell_precedes(distance,cell,ad,target_cell))++rank;
}
W rm=rank>s[3]?rank-s[3]:0;
```

with row loss numerator $\mathcal{L}_{\text{tr}} = 8\,T + w\,\text{rm}$ ($w = 8B = 32$, $K = s[3] = 16$). This formulation has two coupled structural defects (**proof**, directly from `discrete.cu:383-388`):

1. **Vacating a cell strictly reduces `rank` across the dataset:** Because the loop increments `rank` only for `if (occupied[cell])`, any token move on block `0` or block `s[1]-1` that vacates `old_cell` (`occupied[old_cell]` transitions from `true` to `false`) and lands in an already-occupied `new_cell` decreases total `occupancy` by `1` and decreases `rank` by `1` on **every** row where `cell_precedes(d(q, old_cell), old_cell, ad, target_cell)` holds.
2. **Crowding an occupied cell incurs zero rank penalty (`C-R2`):** By `single_count_per_occupied_cell_invariance` (`crates/uor-r4-discrete/src/objective.rs:55,94-98`), an occupied rival cell contributes at most `+1` to `rank` whether it holds `1` token or `3,400` tokens. Thus, herding all $V = 4,096$ tokens into $C_{\text{occ}} \le 20$ cells bounds $r_{\text{occ}} \le 20$ and $\text{rm} = \max(0, r_{\text{occ}} - 16) \le 4$ on every row, collapsing the $32\,\text{rm}$ term to near zero while bloating the admitted candidate pool $\bar{n}$ to almost the entire vocabulary.

### 1.2 Empirical Consequence in `K16-R` vs `#2831` (`m1-a8-full-dose-rerun`)

| Run / Arm | Objective | Initial Cells | Terminal / Q1 Occupied Cells | Admitted Pool $\bar{n}$ (tokens/row) | DEV `b_3(Full)` (`/ 14,287`) | DEV `b_3(P_succ)` (`/ 3,847`) | DEV `top-1` (`/ 14,287`) | `G-Occ` Verdict |
| :--- | :--- | ---: | ---: | ---: | ---: | ---: | ---: | :--- |
| `K16-A` Seed 1 | `NearestAddressMarginV1` (`A`) | `16` | `209 / 256` | `351.8` | `2,237` | — | `65` | PASS (control) |
| `K16-A` Seed 2 | `NearestAddressMarginV1` (`A`) | `16` | `192 / 256` | `376.1` | `3,167` | — | `463` | PASS (control) |
| `K16-R` Seed 1 | `OccupiedCellRankHingeV1` (`R`) | `16` | **`20 / 256`** | **`3,403.9`** | `12,089` | — | `101` | **FAIL (`OCCUPANCY_COLLAPSE_TRIGGERED`)** |
| `K16-R` Seed 2 | `OccupiedCellRankHingeV1` (`R`) | `16` | **`19 / 256`** | **`3,480.7`** | `12,466` | — | `230` | **FAIL (`OCCUPANCY_COLLAPSE_TRIGGERED`)** |
| `#2831` `FrozenZero` (`Q1`) | `NearestAddressMarginV1` (`A`) + `A8` + `D40` | `256` (`16`/cell) | `>= 192 / 256` | `~256` | **`4,292`** | **`1,106`** | **`871`** | PASS (`Q1`) |

---

## 2. Design of `OccupiedCellRankCrowdHingeV1` (`R_occ`)

To raise full-panel admission (`b_3(Full) >= 4,894 / 14,287`) without collapsing cell occupancy, `OccupiedCellRankCrowdHingeV1` (`R_occ`) replaces `OccupiedCellRankHingeV1` (`R`) with three exact integer mechanisms:

1. **Fixed-Grid Cell Rank Hinge ($R_{\text{grid}}$ over all 256 cells):**
   $$r_{\text{grid}}(\text{row}) = 1 + \#\bigl\{c \in \{0,\dots,255\} : \text{cell\_precedes}(d(q, c), c, \text{ad}, c_{\text{target}})\bigr\}, \qquad R_{\text{grid}}(\text{row}) = \max(0,\; r_{\text{grid}}(\text{row}) - K)$$
   Because $r_{\text{grid}}$ is computed over all 256 cells of the fixed $(r_{\text{first}}, r_{\text{last}}) \in 16 \times 16$ root-pair grid (regardless of whether `occupied[c]` is `true` or `false`), vacating a cell has **zero** effect on $r_{\text{grid}}$ for any other token. The only way to reduce $R_{\text{grid}}(\text{row})$ is to move the query $q$ or the target token's endpoint blocks so that `target_cell` moves closer to $q$ in the 256-cell geometric grid.
2. **Integer Cell-Crowding Hinge ($C_{\text{crowd}}$ above $n_{\max} = 24$ tokens/cell):**
   Let `cell_count[c]` ($n(c)$) be the exact number of vocabulary tokens currently in cell $c \in \{0,\dots,255\}$ (at A8 initialization, $n(c) = 16$ for all $c$). Define:
   $$C_{\text{crowd}}(\text{row}) = \max\bigl(0,\; n(c_{\text{target}}) - 24\bigr) + \sum_{k=0}^{K-1} \max\bigl(0,\; n(\text{admitted}[k]) - 24\bigr)$$
   and total row loss numerator:
   $$\mathcal{L}_{R\text{-occ}}(\text{row}) = 8\,T(\text{row}) + 32\,R_{\text{grid}}(\text{row}) + 8\,C_{\text{crowd}}(\text{row})$$
   Any move that crowds a cell beyond $24$ tokens ($1.5 \times$ the uniform A8 quota) pays $+8$ per excess token on every row that targets or admits that cell.
3. **Hard Move-Time `G-Occ` Invariant in `discrete_counterfactual`:**
   During candidate evaluation (`discrete_counterfactual`), if a virtual table move (`kind == 1`) on block `0` or block `s[1]-1` causes total occupied cells `occupancy < 128` or destination cell multiplicity `cell_count[new_cell] > 32`, `writer(...)` returns `~(W)0` (ineligible candidate, never selected by `discrete_choose`). This enforces $\text{occ} \ge 128$ and $\bar{n} \le 16 \times 32 = 512$ as a hard step-by-step invariant.

---

## 3. Why Execution Is Held (`status: held`)

1. **Scope Boundary (`docs/roadmap/items/r-rank-objective-keeps-cells.md` line 10):** The item's `scope:` is `[docs/harness/prep/r-rank-objective-keeps-cells*, docs/labs/r-rank-objective-keeps-cells/**, docs/loop/tasks/r-rank-objective-keeps-cells/**]`. Adding `FixedGridRankHingeV1` and `OccupiedCellRankCrowdHingeV1` to `crates/uor-r4-discrete/src/{objective.rs,discrete.cu}` requires widening the item's scope to `crates/uor-r4-discrete/**`.
2. **Upstream M1 Stage T Prerequisite (`#2836`):** As demonstrated in `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`), `K = 1` single-coordinate greedy updates coupled with `64`-batch whole-interval rollback halt early on `consecutive_rejections` (`batch 1,600` on `Relation`, `batch 5,248` on `FrozenZero`). Running the 3-arm $\times$ 2-seed `R_occ` comparison on the qualified Stage T `BVG-K` (`K = 16`) trainer (`#2836`, Step 4 of `PLAN.md`) ensures all `16,404` batches complete without `K = 1` cadence-granularity stops.
