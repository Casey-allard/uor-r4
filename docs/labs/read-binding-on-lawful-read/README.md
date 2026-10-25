# `read-binding-on-lawful-read` — Discrete Read-Binding Supervision on Lawful `Flock` and `Relation` Reads (`#2889`)

- **Status:** Pre-registered (`TEST FITNESS: FIT`) and **held (`D68`)** pending M1 Stage T (`#2836`) and M2 trunk assembly (`#2844`)
- **Issue:** [`#2889`](https://github.com/UOR-Foundation/uor-r4/issues/2889) (`read-binding-on-lawful-read`)
- **Pre-registration sheet:** [`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`](../../harness/prep/read-binding-on-lawful-read-2026-10-25.md)
- **Mechanisms:** [`docs/mechanisms/read-binding.md`](../../mechanisms/read-binding.md), [`docs/mechanisms/flock-read.md`](../../mechanisms/flock-read.md), [`docs/mechanisms/relation-read.md`](../../mechanisms/relation-read.md)

---

## 1. Executive Summary & Cancel-Probe Classification

In [`archive/labs/read-binding-powered-2026-10-11/README.md`](../../../archive/labs/read-binding-powered-2026-10-11/README.md), read-binding supervision (`read_binding_supervision = 0.1` from step 0 across 3 seeds `20261009`, `20261010`, `20261011` on `chat-29m-B`, `120` multi-fact memory rows) demonstrated that supervising the read to land on the queried fact's earlier value span (`bound_span`) rather than competing value spans (`competing_spans`):
- Raised held-out bound read mass from `0.732` to `0.896 / 0.899 / 0.905`;
- Lowered competing value mass from `0.140` to `0.090 / 0.086 / 0.081`;
- Cut wrong-value errors from **`47 -> 30 / 120` (`-17` items; per-seed `-5, -11, -1`; and `49 -> 23 / 120` at 8,000 steps)**;
- Raised exact-match memory accuracy from **`57 -> 65 / 120` (`48 -> 64 / 120` at 8,000 steps)**.

### Why the Archived Formulation Cannot Run Under the Charter (`CHARTER §3.2, §3.3, D32`)

| Component | Archived Form (`read-binding-powered-2026-10-11`) | Charter Status | Lawful Discrete Replacement (`DiscreteReadBindingHingeV1`) |
|---|---|---|---|
| Read distribution | Softmax attention weights $a_j^{(h)} = \operatorname{softmax}(q_h \cdot k_j / \sqrt{d})$ | **BANNED** (`CHARTER §3.2` `D23`, `§3.3` `D25`) | Discrete top-$K_{\text{kept}}$ causal candidate admission in `flock_vector` (`MODE_FLOCK = 1`, `MODE_RELATION = 2`) |
| Bound score | Continuous probability mass $m_{\text{bound}}^{(h)} = \sum_{j \in B} a_j^{(h)}$ | **BANNED** (`D23`, `D32`) | Exact integer composite rank key $R_b(i) = (\text{rel}_b[i] \ll 3) - d_{\text{pos}}(q_b, i) \in \mathbb{Z}$ |
| Objective | Negative log-probability $\mathcal{L}_{\text{bind}} = -\log(\max_h m_{\text{bound}}^{(h)} + 10^{-6})$ | **BANNED** (`D23` log-probability over normalized exponential) | Exact integer rank-margin + top-$K_{\text{kept}}$ admission hinge $H_{\text{bind}}(\text{row}) \in \mathbb{Z}_{\ge 0}$ |
| Optimization | Float AdamW backpropagation through dense projections | **BANNED** on `D32` discrete path | Exact `discrete_counterfactual` evaluation over `table` (`kind = 1`), `gates` (`kind = 3`), and `position_steps` (`s[14]`) |

---

## 2. Exact Mathematical & Kernel Specification (`DiscreteReadBindingHingeV1`)

### 2.1 Candidate Ordering in `flock_vector` (`crates/uor-r4-discrete/src/discrete.cu`)

In [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`lines 217–314`), `flock_vector` executes both `MODE_FLOCK` (`s[8] = 1`) and `MODE_RELATION` (`s[8] = 2`). For every block $b \in [0, s[1])$ and binder stage $l \in [0, s[5])$, each causal node $i$ (`level[i] == l && last[i] + 1 < length`, `line 285`) has:
- **Signed graph relation score** $\text{rel}_b[i] \in \mathbb{Z}$:
  - In `MODE_FLOCK` (`s[8] = 1`): $\text{rel}_b[i] = 0$ for all $i$ (`line 218`).
  - In `MODE_RELATION` (`s[8] = 2`): initialized from $8 - d_{\text{pos}}(q_b, i) \in \{0, 2, 4, 6, 8\}$ (`line 227`) and updated over `sweeps = s[10]` sweeps across $5$ graph edge types (`parent/child`, `sibling`, `adjacent`, `same-token`, `same-frame`, `lines 229–266`) gated by $g_{b, l, e} \in \{-1, 0, +1\}$.
- **Positioned root cost** $d_{\text{pos}}(q_b, i) \in \{0, 2, 4, 6, 8\}$ (`positioned_root_cost`, `line 287`).
- **Top-$K_{\text{kept}}$ selection** (`lines 290–300`): inserts up to $K_{\text{kept}} = s[9] \in [1, 8]$ nodes into `top[0..found]` by:
  $$\big(\text{rel}_b[i]\;\text{desc},\; d_{\text{pos}}(q_b, i)\;\text{asc},\; \text{hamming}\;\text{asc},\; \text{last}[i]\;\text{desc},\; i\;\text{asc}\big).$$

### 2.2 Integer Composite Node Rank Key and Bound-vs-Competing Hinge

Because $d_{\text{pos}}(q_b, i) \in \{0, 2, 4, 6, 8\} \subset [0, 8)$, shifting $\text{rel}_b[i]$ left by 3 bits ($\times 8$, multiplier-free) produces the exact lexicographic integer key combining `rel_score desc` and `distance asc`:
$$R_b(i) = (\text{rel}_b[i] \ll 3) - (int)d_{\text{pos}}(q_b, i).$$
- In **`MODE_FLOCK` (`s[8] = 1`)**, $\text{rel}_b[i] = 0$, so $R_b(i) = -(int)d_{\text{pos}}(q_b, i) \in \{-8, -6, -4, -2, 0\}$.
- In **`MODE_RELATION` (`s[8] = 2`)**, each $+1$ increment in $\text{rel}_b[i]$ contributes $+8$ to $R_b(i)$, strictly dominating $d_{\text{pos}}(q_b, i) \in [0, 8)$ just as `rscore > key_r[at-1]` dominates `distance < key_d[at-1]` at `discrete.cu:293`.

Given a labeled memory row with bound leaf indices $B(\text{row}) \subset [0, L-1)$ (`level[j] == 0`) and competing value leaf indices $C(\text{row}) \subset [0, L-1)$ (`level[c] == 0`), define at each block $b \in [0, s[1])$:
$$R_b^{*}(B) = \max_{j \in B(\text{row})} R_b(j), \qquad R_b^{*}(C) = \max_{c \in C(\text{row})} R_b(c), \qquad \Delta R_b(\text{row}) = R_b^{*}(B) - R_b^{*}(C).$$
Taking the best active block $\Delta R_{\max}(\text{row}) = \max_{b \in [0, s[1])} \Delta R_b(\text{row})$ (mirroring $\max_h$ across heads in `docs/mechanisms/read-binding.md`), the **discrete read-binding hinge** is:
$$H_{\text{bind}}(\text{row}) = \max\!\big(0,\; m_{\text{bind}} - \Delta R_{\max}(\text{row})\big) + W_{\text{miss}} \cdot \mathbf{1}\!\left[\forall b \in [0, s[1]),\; B(\text{row}) \cap \operatorname{kept}_{b, l=0}(\text{row}) = \emptyset\right]$$
with:
- $m_{\text{bind}} = 2$ (requiring the best bound leaf to beat the best competing leaf by at least one root-cost step $\Delta d_{\text{pos}} \ge 2$ in `MODE_FLOCK` or relation score margin in `MODE_RELATION`);
- $W_{\text{miss}} = 4$ (penalizing any row where no bound leaf enters `top[0..found]` at level $l = 0$).

### 2.3 Device Kernel Integration (`crates/uor-r4-discrete`)

To preserve `driver_source_sha256` stability until M1 Stage T (`#2836`) closes (`run-antigravity.txt` line 205), `DiscreteReadBindingHingeV1` is specified as an additive sidecar buffer passed only during supervised memory training:
- **Sidecar layout (`bind_spans: &[u32]`, `4` words per row):**
  - `bind_spans[row * 4 + 0] = bound_start`, `bind_spans[row * 4 + 1] = bound_end` ($B(\text{row}) = [\text{bound\_start}, \text{bound\_end})$);
  - `bind_spans[row * 4 + 2] = comp_mask_lo`, `bind_spans[row * 4 + 3] = comp_mask_hi` (64-bit bitmask of competing value leaf positions $c \in C(\text{row})$ for $L \le 64$, or span table offset for $L > 64$).
  - When `bound_start == ABSENT`, $H_{\text{bind}}(\text{row}) = 0$ (non-memory rows and all evaluation/serving passes).
- **Counterfactual training (`row_loss`):**
  ```cuda
  __device__ U bind_hinge_cost(
      const int* rel_b, const U* dist_b, const U* top_b, U found_b,
      U bound_start, U bound_end, U comp_lo, U comp_hi
  ) {
      if (bound_start == ABSENT || bound_start >= bound_end) return 0u;
      int best_bound = -1073741824;
      int best_comp  = -1073741824;
      bool admitted = false;
      for (U k = 0; k < found_b; ++k) {
          if (top_b[k] >= bound_start && top_b[k] < bound_end) admitted = true;
      }
      for (U j = bound_start; j < bound_end; ++j) {
          int rj = (rel_b[j] << 3) - (int)dist_b[j];
          if (rj > best_bound) best_bound = rj;
      }
      for (U c = 0; c < 64u; ++c) {
          U bit = (c < 32u) ? ((comp_lo >> c) & 1u) : ((comp_hi >> (c - 32u)) & 1u);
          if (!bit) continue;
          int rc = (rel_b[c] << 3) - (int)dist_b[c];
          if (rc > best_comp) best_comp = rc;
      }
      int gap = best_bound - best_comp;
      U margin_cost = (gap < 2) ? (U)(2 - gap) : 0u;
      U miss_cost = admitted ? 0u : 4u;
      return margin_cost + miss_cost;
  }
  ```

---

## 3. Experimental Design & Verification Ladder

### Step 1: Finite CUDA Qualification (`test_discrete_read_binding_hinge_v1`)
1. **Exactness vs. Host Reference:** On 8 synthetic multi-fact rows (3 attributes per row, 1 queried attribute, 2 competing attributes), verify device `bind_hinge_cost` matches the Rust integer reference `100%` in both `MODE_FLOCK` (`s[8] = 1`) and `MODE_RELATION` (`s[8] = 2`).
2. **Counterfactual Decrease on `Flock` and `Relation`:** Verify one sweep of `discrete_counterfactual` over `table` (`kind = 1`) and `gates` (`kind = 3`, including relation edge gates) strictly reduces $\sum_{\text{row}} H_{\text{bind}}(\text{row})$ and moves a bound leaf $j \in B(\text{row})$ into `top[0]`.
3. **Permuted-Sidecar Negative Control:** Swapping $B(\text{row}) \leftrightarrow C(\text{row})$ strictly flips the selected rank-0 node (`top[0]`) from $B(\text{row})$ to $C(\text{row})$.

### Step 2: 3-Seed $\times$ 4-Arm GPU Run on the 120-Item Memory Panel
- **Hardware:** `1x RTX 5090` via `scripts/pod/uor-pod`.
- **Seeds:** `20261009`, `20261010`, `20261011` (`40` held-out memory rows per seed = `120` rows total, matching `archive/labs/read-binding-powered-2026-10-11/README.md`).
- **Metrics Reported:**
  1. Decode tokens/s (lead metric, `CHARTER §1`);
  2. Wrong-value errors `/ 120` (pooled and per seed);
  3. Exact-match memory accuracy `/ 120` (pooled and per seed) vs. unigram and `T2` count tables;
  4. Held-out rank-0 bound leaf admission rate (`lv[4] \in B(\text{row})`) and competing leaf admission rate (`lv[4] \in C(\text{row})`) via the `flock_vector` trace buffer;
  5. Stage-silence (`gates = 0`) ablation accuracy (`L-009`, `L-010`).

---

## 4. Whole-Project Chain & `D68` Decision Record (`status: held`)

Per `CHARTER §5.1` (`D68`) and `PLAN.md` (`lines 50–86`):
1. **Milestone Chain Context:** The active M1 critical path is `m1-bvg-k-stage-t-training` (`#2836`) $\to$ `m1-a8-full-dose-rerun` (`#2831`) $\to$ `m1-export-gap-and-toks` (`#2839`) $\to$ `m1-fresh-panel-transfer` (`#2841`) $\to$ `m1-close-saved-model` (`#2837`). Multi-fact conversation memory (`M2`/`M3`) sits downstream of `m1-trunk-assembly-and-training` (`#2844`).
2. **Scope & Pin Constraint:** `read-binding-on-lawful-read` (`#2889`) is scoped to `[docs/harness/prep/read-binding-on-lawful-read*, docs/labs/read-binding-on-lawful-read/**, docs/loop/tasks/read-binding-on-lawful-read/**]` (excluding `crates/uor-r4-discrete/**`), and editing `crates/uor-r4-discrete/src/discrete.cu` while M1 Stage T (`#2836`) and A8 (`#2831`) are running would mutate `driver_source_sha256`.
3. **Decision (`D68`):** With the lawful discrete formulation (`DiscreteReadBindingHingeV1`), pre-registration sheet (`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`), and fresh-context `TEST FITNESS: FIT` review complete, `read-binding-on-lawful-read` is placed in `status: held` until `#2836` (`m1-bvg-k-stage-t-training`) and `#2844` (`m1-trunk-assembly-and-training`) complete and `crates/uor-r4-discrete/**` is free for kernel integration.
