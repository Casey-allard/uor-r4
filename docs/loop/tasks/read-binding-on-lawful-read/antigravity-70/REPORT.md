# REPORT: read-binding-on-lawful-read, antigravity-70

Lab: antigravity. Item: [read-binding-on-lawful-read](../../../../roadmap/items/read-binding-on-lawful-read.md) (#2889).
References #2889

## What did not work

- **Direct reuse of the archived read-binding supervision loss (`archive/labs/read-binding-powered-2026-10-11/README.md`, `docs/mechanisms/read-binding.md`) did not work under the Charter (`classified: mechanism` — configuration gap between softmax-mass supervision and `D32` integer `Flock`/`Relation` reads):**
  - The 2026-10-11 study (`read_binding_supervision = 0.1` from step 0 across 3 seeds `20261009`, `20261010`, `20261011` on `chat-29m-B`, `120` multi-fact memory items) supervised continuous softmax attention weights $m_{\text{bound}}^{(h)} = \sum_{j \in B} a_j^{(h)}$ via the negative log-probability objective $\mathcal{L}_{\text{bind}} = -\log(\max_h m_{\text{bound}}^{(h)} + 10^{-6})$ on a 29M dense matmul float stack (`crates/uor-r4-training/src/geometric_stack.rs`).
  - That formulation violates `CHARTER §3.2` (`D23`: no softmax, log-softmax, or normalized exponential anywhere, including losses and surrogates), `CHARTER §3.3` (`D25`/`D30`: no dense matmul), and `D32` (no float in discrete-model computation).
- **Executing the 3-seed GPU run inside this unit was not appropriate (`D68` whole-project decision review):**
  - `read-binding-on-lawful-read` is scoped to `[docs/harness/prep/read-binding-on-lawful-read*, docs/labs/read-binding-on-lawful-read/**, docs/loop/tasks/read-binding-on-lawful-read/**]` (excluding `crates/uor-r4-discrete/**`), and editing `crates/uor-r4-discrete/src/discrete.cu` while M1 Stage T (`#2836`) and A8 (`#2831`) are active would mutate `driver_source_sha256`. Furthermore, multi-fact conversation memory (`M2`/`M3`) sits downstream of `m1-bvg-k-stage-t-training` (`#2836`) and `m1-trunk-assembly-and-training` (`#2844`) (`PLAN.md` lines 50–86).

## What changed

- **Files changed:** 5 files (`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`, `docs/labs/read-binding-on-lawful-read/README.md`, `docs/loop/tasks/read-binding-on-lawful-read/antigravity-70/ORDER.md`, `docs/loop/tasks/read-binding-on-lawful-read/antigravity-70/REPORT.md`, `docs/roadmap/items/read-binding-on-lawful-read.md`, plus rendered `docs/roadmap/README.md`; 0 code lines, 0 test lines, ~285 documentation lines).
- **Lawful Discrete Read-Binding Formulation (`DiscreteReadBindingHingeV1`, [`docs/labs/read-binding-on-lawful-read/README.md`](../../../labs/read-binding-on-lawful-read/README.md)):**
  - In `flock_vector` (`crates/uor-r4-discrete/src/discrete.cu` lines 193–321), both `MODE_FLOCK` (`s[8] = 1`) and `MODE_RELATION` (`s[8] = 2`) select up to $K_{\text{kept}} = s[9] \in [1, 8]$ causal nodes (`top[0..found]`) ordered by $(\text{rel}_b[i]\;\text{desc},\; d_{\text{pos}}(q_b, i)\;\text{asc},\; \text{hamming}\;\text{asc},\; \text{last}[i]\;\text{desc},\; i\;\text{asc})$.
  - Defines the multiplier-free integer composite rank key $R_b(i) = (\text{rel}_b[i] \ll 3) - (int)d_{\text{pos}}(q_b, i) \in \mathbb{Z}$, which preserves the exact lexicographic dominance of `rel_score desc` over `distance asc` ($d_{\text{pos}} \in \{0, 2, 4, 6, 8\} \subset [0, 8)$).
  - Defines the integer bound-vs-competing hinge over labeled bound value leaves $B(\text{row})$ and competing value leaves $C(\text{row})$:
    $$H_{\text{bind}}(\text{row}) = \max\!\big(0,\; m_{\text{bind}} - \Delta R_{\max}(\text{row})\big) + W_{\text{miss}} \cdot \mathbf{1}\!\left[\forall b \in [0, s[1]),\; B(\text{row}) \cap \operatorname{kept}_{b, l=0}(\text{row}) = \emptyset\right]$$
    with $m_{\text{bind}} = 2$ and $W_{\text{miss}} = 4$, supervising `table` (`kind = 1`), `position_steps` (`s[14]`), and `gates` (`kind = 3`, including the $5 \times S$ relation edge trits) during `discrete_counterfactual` while adding **zero** operations at evaluation and `D11` serving time (`CHARTER §3.6`).
- **Pre-Registration Sheet & Fresh-Context Reviews ([`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`](../../../harness/prep/read-binding-on-lawful-read-2026-10-25.md)):**
  - Pre-registered the 4-arm $\times$ 3-seed (`20261009`, `20261010`, `20261011`, `120` memory rows) experiment (`Arm-0-Flock-Unbound`, `Arm-1-Flock-Bound`, `Arm-2-Relation-Unbound`, `Arm-3-Relation-Bound`), pass bars, controls, and stop rules.
  - Completed fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER §4.5`) and `D68` whole-project decision review (`CHARTER §5.1`) via reviewer `adf6ac3a-d390-4e18-a35b-57d57818e82f`, logged on [`#2889` (`issuecomment-6397278922`)](https://github.com/UOR-Foundation/uor-r4/issues/2889#issuecomment-6397278922), and transitioned `docs/roadmap/items/read-binding-on-lawful-read.md` to `status: held`.

## Measurements

- **Cancel probe inspection (`crates/uor-r4-discrete/src/discrete.cu` lines 193–326, `archive/labs/read-binding-powered-2026-10-11/README.md`):**
  - Confirmed `flock_vector` (`lines 282–308`) ranks causal candidates at level $l$ by `(rel[node] desc, distance asc, hamming asc, last[node] desc, node asc)` with `distance` $\in \{0, 2, 4, 6, 8\}$ and records selected candidate indices in `trace` (`lv[4+k] = node`, `line 308`).
  - Confirmed `0` occurrences of `read_binding` or `bind_spans` in `crates/uor-r4-discrete`.
- **No GPU compute was run in this unit:** per `CHARTER §4.5` and `§5.1`, this unit is zero-compute design, pre-registration (`TEST FITNESS: FIT`), and `D68` status classification (`status: held`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap render`: PASS
- `uor-harness loop report --check --order docs/loop/tasks/read-binding-on-lawful-read/antigravity-70/ORDER.md`: PASS
- `uor-harness loop pr-check --local`: PASS
- `uor-harness loop gates --full`: PASS

## Skills used

- `scope-guard`: Kept all changes strictly within `read-binding-on-lawful-read`'s declared scope (`docs/harness/prep/read-binding-on-lawful-read*`, `docs/labs/read-binding-on-lawful-read/**`, `docs/loop/tasks/read-binding-on-lawful-read/**`) and always-allowed roadmap files.
- `solution-integrity`: Constructed `DiscreteReadBindingHingeV1` directly from the existing integer quantities in `flock_vector` (`rel[i]` and `positioned_root_cost` `distance`) so that training-time read-binding supervision requires zero extra serving operations (`D11`).

## Still open

- Kernel implementation of `bind_hinge_cost` in `crates/uor-r4-discrete` and execution of the 4-arm $\times$ 3-seed RTX 5090 GPU run (`docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`) once `m1-bvg-k-stage-t-training` (`#2836`) and `m1-trunk-assembly-and-training` (`#2844`) complete and `crates/uor-r4-discrete/**` is free.

## Plain summary

When a conversation contains several facts (for example, both a user's city and a project's language) and a later question asks about only one of them, an unguided read can accidentally copy the wrong earlier value ("wrong-value error"). Previously, "read binding" (a training-time teacher signal that rewards the model's read for landing on the queried fact's earlier value span) reduced wrong-value errors from `47` to `30` out of `120` questions across 3 seeds, but it relied on softmax attention probabilities on a floating-point matrix-multiply model, which are banned by the UOR-R4 Charter. This unit ports read binding to the project's lawful integer `Flock` and `Relation` reads (`DiscreteReadBindingHingeV1`) using an exact integer rank-margin and top-$K$ admission hinge ($R_b(i) = (\text{rel}_b[i] \ll 3) - d_{\text{pos}}(q_b, i)$) with zero floating-point math, zero softmax, and zero extra cost at serving time, completes the pre-registration sheet and fresh-context `FIT` and `D68` reviews, and holds the GPU run until the active M1 and M2 trunk prerequisites finish.
