# Pre-Registration Sheet: `read-binding-on-lawful-read` (`#2889`)

- **Date:** 2026-10-25
- **Lab:** `antigravity`
- **Issue:** [`#2889`](https://github.com/UOR-Foundation/uor-r4/issues/2889) (`read-binding-on-lawful-read`)
- **Mechanisms:** [`docs/mechanisms/read-binding.md`](../../mechanisms/read-binding.md), [`docs/mechanisms/flock-read.md`](../../mechanisms/flock-read.md), [`docs/mechanisms/relation-read.md`](../../mechanisms/relation-read.md)
- **Prior Evidence:** [`archive/labs/read-binding-powered-2026-10-11/README.md`](../../../archive/labs/read-binding-powered-2026-10-11/README.md), [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu)

## 1. Mechanism Under Test

We test **`DiscreteReadBindingHingeV1`**: porting read-binding supervision (`archive/labs/read-binding-powered-2026-10-11/README.md`) from its archived, banned softmax-mass formulation ($\mathcal{L}_{\text{bind}} = -\log(\max_h \sum_{j \in B} a_j^{(h)} + 10^{-6})$ on a 29M dense matmul float stack, banned by `CHARTER §3.2` `D23` and `§3.3` `D25`/`D30`) to the lawful `D32` integer `Flock` (`MODE_FLOCK = 1`, `s[8] = 1`) and `Relation` (`MODE_RELATION = 2`, `s[8] = 2`) reads in [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`flock_vector`, lines 193–321).

Given the pre-registered `binding_labels.jsonl` sidecar on the 120-item multi-fact memory panel (`archive/labs/read-binding-powered-2026-10-11/README.md`, 3 seeds $\times$ 40 rows = 120 rows), each row supplies:
- **Bound value leaf span** $B(\text{row}) \subset [0, L-1)$: causal leaf positions holding the value of the queried attribute (`bound_span`).
- **Competing value leaf spans** $C(\text{row}) \subset [0, L-1)$: causal leaf positions holding the values of the non-queried attributes in the same context (`competing_spans`).

In `flock_vector` (`crates/uor-r4-discrete/src/discrete.cu` lines 275–314), at each active block $b \in [0, B_{\text{blk}})$ and atom level $l = 0$, every causal leaf node $i \in [0, L-1)$ has:
1. **Positioned root transport distance** $d_{\text{pos}}(q_b, i) \in \{0, 2, 4, 6, 8\}$ (`positioned_root_cost`, line 287).
2. **Signed graph relation score** $\text{rel}_b[i] \in \mathbb{Z}$ (`lines 217–266`, identically $0$ in `MODE_FLOCK` and the multi-sweep signed graph-edge sum over the 5 edge types in `MODE_RELATION`).
3. **Top-$K_{\text{kept}}$ admission set** $\operatorname{kept}_{b, l=0}(\text{row}) = \{\text{top}[0], \dots, \text{top}[\text{found}-1]\}$ (`lines 290–308`, $K_{\text{kept}} = s[9] \in [1, 8]$), ordered by $(\text{rel}_b[i]\;\text{desc},\; d_{\text{pos}}(q_b, i)\;\text{asc},\; \text{hamming}\;\text{asc},\; \text{last}[i]\;\text{desc},\; i\;\text{asc})$.

We define the integer **composite node rank key** (pure bit-shift and addition/subtraction, zero multipliers, zero float):
$$R_b(i) = (\text{rel}_b[i] \ll 3) - (int)d_{\text{pos}}(q_b, i)$$
so that $R_b(j) > R_b(c)$ if and only if node $j$ strictly outranks node $c$ on the primary $(\text{rel\_score desc}, \text{distance asc})$ keys of `flock_vector` (`line 293`) whenever $|\text{rel}_b[j] - \text{rel}_b[c]| \ge 1$ or $\text{rel}_b[j] = \text{rel}_b[c]$. Let:
$$R_{\max}(B) = \max_{b \in [0, B_{\text{blk}})} \max_{j \in B(\text{row})} R_b(j), \qquad R_{\min\text{-gap}}(B, C) = \max_{b \in [0, B_{\text{blk}})} \left(\max_{j \in B(\text{row})} R_b(j) - \max_{c \in C(\text{row})} R_b(c)\right).$$
The **discrete read-binding hinge** on row $\text{row}$ is:
$$H_{\text{bind}}(\text{row}) = \max\!\Big(0,\; m_{\text{bind}} - R_{\min\text{-gap}}(B, C)\Big) + W_{\text{miss}} \cdot \mathbf{1}\!\left[\forall b \in [0, B_{\text{blk}}),\; B(\text{row}) \cap \operatorname{kept}_{b, l=0}(\text{row}) = \emptyset\right]$$
with integer margin $m_{\text{bind}} = 2$ (one root-cost step in `MODE_FLOCK`, or $\ge 1$ relation unit in `MODE_RELATION`) and top-$K_{\text{kept}}$ miss penalty $W_{\text{miss}} = 4$. During `discrete_counterfactual` training, the row loss is augmented by a shift-only weight (`w_bind_shift` $\in \{0, 1, 2\}$, adding $H_{\text{bind}}(\text{row}) \ll \text{w\_bind\_shift}$ when enabled):
$$L_{\text{total}}(\text{row}) = L_{\text{writer}}(\text{row}) + (H_{\text{bind}}(\text{row}) \ll \text{w\_bind\_shift}).$$
At evaluation and D11 serving time, $H_{\text{bind}}$ is never evaluated (`binding_labels` sidecar is training-only): the exported `UORLUT01` integer artifact runs unaltered `Flock` or `Relation` reads with **zero** extra operations (`CHARTER §3.6`).

### Four Matched Arms (Across 3 Seeds: `20261009`, `20261010`, `20261011`)
1. **`Arm-0-Flock-Unbound` (`MODE_FLOCK = 1`, $H_{\text{bind}}$ off):** Unbound `Flock` read (`s[8] = 1, s[9] = 7`) trained with standard writer loss only ($L_{\text{writer}}$).
2. **`Arm-1-Flock-Bound` (`MODE_FLOCK = 1`, $H_{\text{bind}}$ on, `w_bind_shift = 1`):** `Flock` read (`s[8] = 1, s[9] = 7`) trained with $L_{\text{writer}}(\text{row}) + (H_{\text{bind}}(\text{row}) \ll 1)$ from step 0.
3. **`Arm-2-Relation-Unbound` (`MODE_RELATION = 2`, $H_{\text{bind}}$ off):** Unbound `Relation` read (`s[8] = 2, s[9] = 7, s[10] = 2`) trained with standard writer loss only ($L_{\text{writer}}$).
4. **`Arm-3-Relation-Bound` (`MODE_RELATION = 2`, $H_{\text{bind}}$ on, `w_bind_shift = 1`):** `Relation` read (`s[8] = 2, s[9] = 7, s[10] = 2`) trained with $L_{\text{writer}}(\text{row}) + (H_{\text{bind}}(\text{row}) \ll 1)$ from step 0, where $H_{\text{bind}}$ directly supervises both root-table moves (`kind = 1`), `position_steps` (`s[14]`), and the $5 \times S$ signed relation-edge gates (`kind = 3`, `s[1]*s[5] + rg`).

## 2. Failure Mode Targets

1. **Wrong-value attraction under multi-fact context (`archive/labs/read-binding-powered-2026-10-11/README.md`):** Without read-binding supervision, a multi-fact context causes `Flock`/`Relation` to admit competing value leaves $c \in C(\text{row})$ into `top[0..kept]` at higher or equal rank to the queried fact's bound value leaves $j \in B(\text{row})$ (`47 / 120` wrong-value errors on the unbound baseline).
2. **Relation-gate indifference (`docs/mechanisms/relation-read.md`):** Without direct supervision on which earlier value node should win `top[0..kept]`, `discrete_counterfactual` proposals on the $5 \times S$ relation-edge gates (`kind = 3`) receive credit only when flipping a single edge trit immediately changes the final snapped writer root $m$ and token distance $d(q, y)$, leaving relation gates flat when multiple value leaves tie.
3. **Zero-cost serving preservation (`CHARTER §3.6`, `D11`):** Because $H_{\text{bind}}(\text{row})$ is a training-time counterfactual objective over existing `Flock`/`Relation` state (`rel[i]`, `distance`, `top[0..found]`), serving cost is bit-identical to unbound `Flock`/`Relation`.

## 3. Pass Bar

Evaluated on the held-out 120-item multi-fact memory panel (`40` items $\times$ `3` seeds `20261009`, `20261010`, `20261011`, matching `archive/labs/read-binding-powered-2026-10-11/README.md` and the `#2889` done-when):
1. **Primary Wrong-Value Error Reduction Bar (`#2889` done-when):** At least one bound lawful read arm (`Arm-1-Flock-Bound` vs `Arm-0-Flock-Unbound`, or `Arm-3-Relation-Bound` vs `Arm-2-Relation-Unbound`) reduces pooled wrong-value errors across the 120-item panel by **$\ge 10$ items** (outside the 3-seed binomial noise band of $\pm 6$ items), with wrong-value errors non-increasing on **all 3 seeds individually** and strictly decreasing on **$\ge 2 / 3$ seeds**.
2. **Memory Exact-Match & No-Regression Bar:** Pooled exact-match memory accuracy on the 120-item panel increases by **$\ge 6 / 120$ items** over the matched unbound read (`Arm-1` vs `Arm-0`, or `Arm-3` vs `Arm-2`) and beats the deterministic unigram and `T2` count tables on the panel (`CHARTER §4.4`).
3. **Held-Out Bound Node Admission Diagnostic:** Using `trace` (`crates/uor-r4-discrete/src/discrete.cu` lines 267–310, `lv[4+k]`), the fraction of held-out memory rows where at least one bound leaf $j \in B(\text{row})$ enters `top[0..kept]` strictly at rank 0 (`lv[4] \in B(\text{row})`) increases by **$\ge 15$ percentage points** over the matched unbound control, and stage-silence ablation (`gates = 0`) drops accuracy to the no-memory floor (`L-009`, `L-010`).
4. **Throughput & Lawfulness:** Decode tokens/s reported as the lead metric (`CHARTER §1`), within $\pm 2\%$ of the matched unbound `Flock`/`Relation` arm (`0` extra instructions at evaluation/serving), and `0` banned tokens (`D23`/`D25`/`D32`).

## 4. Controls

1. **Matched Unbound Lawful Read Controls (`Arm-0-Flock-Unbound`, `Arm-2-Relation-Unbound`):** Identical seed, corpus, schedule, and architecture with `w_bind_shift = 0` ($H_{\text{bind}}$ disabled).
2. **Stage-Silence / No-Memory Control (`L-009`, `L-010`):** Zeroing all binder gates (`gates[b*S + l] = 0`) at evaluation time on the trained `Arm-1` and `Arm-3` checkpoints to prove the wrong-value error reduction is carried by the `Flock`/`Relation` read rather than surface weights.
3. **Permuted-Sidecar Negative Control (Finite Qualification):** Swapping $B(\text{row}) \leftrightarrow C(\text{row})$ in a finite CUDA test must invert $R_{\min\text{-gap}}(B, C)$ and drive `discrete_counterfactual` to select the swapped span, proving the instrument is non-vacuous (`CHARTER §4.2`).
4. **Deterministic Count-Table Baselines (`CHARTER §4.4`):** Unigram and `T2` successor count tables evaluated on the same 120-item panel.

## 5. Stop Rule

- **Instrument vs. Mechanism Classification (`CHARTER §4.1, §4.2`):** Before any GPU training run, finite CUDA unit tests must verify:
  1. Exact integer calculation of $R_b(i) = (\text{rel}_b[i] \ll 3) - d_{\text{pos}}(q_b, i)$ and $H_{\text{bind}}(\text{row})$ against a host integer reference on synthetic 3-fact rows (`100%` bit-exact match);
  2. Strict reduction of $H_{\text{bind}}(\text{row})$ after one `discrete_counterfactual` sweep over `table` (`kind = 1`) and `gates` (`kind = 3`).
  Any mismatch is an instrument fault and stops the run until repaired.
- **Compute Cap & Wall-Time Stop:** Single `1x RTX 5090` pod ($\le \$0.60/\text{hr}$, total budget $\le \$2.50$, wall time $\le 2.5\text{ hours}$ for all 4 arms $\times$ 3 seeds). If throughput probe projects $> 30\text{ minutes}$ per arm, stop and optimize before full execution.
- **Outcome Classification:** If both `Arm-1-Flock-Bound` and `Arm-3-Relation-Bound` pass the finite counterfactual decrease check but fail to reduce wrong-value errors by $\ge 10 / 120$ on the held-out panel, close the `DiscreteReadBindingHingeV1` atom-level configuration (`CHARTER §4.1`) and record whether bound-node admission (`lv[4] \in B`) improved without writer transport separation or failed to generalize to held-out rows.

TEST FITNESS: FIT
