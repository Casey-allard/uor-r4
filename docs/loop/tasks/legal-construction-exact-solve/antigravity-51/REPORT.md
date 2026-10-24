# REPORT: antigravity-51, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Requiring a single compensator `i2` to 100% repair the worst violated guard `r0` before entering `top_partials`, and evaluating reduced-cost fixing on disjunctive split leaves under `leaf.y` alone.**
   - In Unit 8 (`antigravity-50`), `run_polish` filtered out any first compensator `i2` with `after1_guard[r0] + j_dy[r0][i2] * diff2_dy < min_slack_dy` before pushing to `top_partials`; consequently, 2- and 3-compensator repairs (`(i1, i2, i3)` and `(i1, i2, i3, i4)`) could never cooperatively repair a deep violation on `r0` across multiple compensators. Additionally, on a disjunctive branch-and-bound leaf $l$ with tightened dual multipliers $y^{(l)}$ ($L(y^{(l)}, B^{(l)}) \ge L(y_{\text{root}}, B_{\text{root}})$), evaluating reduced-cost fixing under $y^{(l)}$ alone can fail to eliminate a code $(j, q)$ whose reduced cost had larger magnitude under $y_{\text{root}}$ than under $y^{(l)}$.
   - Allowing partial `r0` repairs (`new_r0_s >= after1_guard[r0] / 2`, closing $\ge 50\%$ of `r0`'s deficit) into `top_partials` when no 1-compensator pair exists, expanding `i1` transitions to `mult in 1..=14` (`step2 in -6..=6`, `step3 in -4..=4`), and intersecting each leaf's exact domain reduced-cost test on $B^{(l)}$ under both the shared root dual $y_{\text{root}}$ (`Some(dual_y_ref)`) and $y^{(l)}$ improves the certified discrete incumbent to **`-94674498416698555 / 4503599627370496` (`-21.02196160...`)**, guarantees `tree_eliminated_codes >= root_eliminated_codes` relative to `certify_reduced_cost_fixing(p, dual_y_ref, upper_bound)` on every instance (`m < 64` and `m >= 64`), and eliminates **`6,254 / 28,800` discrete codes (`21.72%`, `+433` over Unit 8)** on `attempt2`.

## What changed

- **Code (`+763 / -104` lines across 2 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+750 / -104` lines):
    - Added `certify_domain_dual_bound(p, y, code_bounds)` for basis-free exact rational Weak Duality lower-bound certification over any domain-restricted discrete box $B = \prod_i [q_{\min, i}, q_{\max, i}]$.
    - Added `DisjunctiveSplitCertificate` and `certify_disjunctive_split_bound::<P>(p, upper_bound, max_leaves)` to evaluate 1-variable and multi-leaf disjunctive branch-and-bound splits $(q_i \le q_{\text{down}}) \lor (q_i \ge q_{\text{down}} + 1)$ across all fractional basic variables of `delta_hull_lp(p)`, certify every leaf lower bound in exact `BigRational` arithmetic, and perform Disjunctive Partition Reduced-Cost Fixing across the exhaustive leaf partition under both $y_{\text{root}}$ and $y^{(l)}$.
    - Upgraded `lp_guided_discrete_solve::<P>` with cooperative partial-`r0` multi-compensator polish, full-range `mult in 1..=14` primary transitions (`step2 in -6..=6`, `step3 in -4..=4`), expanded direct greedy guard-violation repair ($1 \le \text{n\_viol} \le 5$ on $m \ge 64$, up to 45 steps with `step in -3..=3`), `viol_rows`/`rem12_rows` pre-filtering, `Copy` candidate records, and post-`single_sweep` deduplication.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+13 / -0` lines):
    - Added `"disjunctive_single_split_bound"`, `"disjunctive_single_split_gain"`, `"disjunctive_best_split_coeff"`, `"disjunctive_tree_lower_bound"`, `"disjunctive_tree_gain"`, `"disjunctive_tree_gap"`, `"disjunctive_tree_leaves"`, `"disjunctive_evaluated_splits"`, `"disjunctive_tree_eliminated_codes"`, `"disjunctive_tree_fixed_coeffs"`, and `"disjunctive_tree_binary_or_fixed_coeffs"` to the `"reduced_cost_fixing"` JSON output.
- **Tests (`+45 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+45 / -0` lines):
    - Added negative-control and exact-monotonicity tests for `certify_domain_dual_bound` and `certify_disjunctive_split_bound` across all 9 finite controls and 60 random instances (`root_lower_bound <= best_single_split_bound <= tree_lower_bound <= bb_res.objective`, `tree_eliminated_codes >= rcbt.eliminated_codes`, and optimal discrete code survival in `tree_surviving_codes`).
- **Docs (`+16 / -0` lines across 2 files, plus `ORDER.md` and `REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+15 / -0` lines): added Unit 9 section.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added Unit 9 row.
- **Generated / Other:** `0` lines.

## Measurements

All measurements were taken on the laptop (`macOS`, release profile for `root-search`, sha256-verified read-only `attempt2` directory at `local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`, 384/384 files verified):

- **Command:**
  ```sh
  cd docs/labs/legal-construction-exact-solve/solver
  CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo build --release --locked --bin root-search
  /tmp/antigravity-legal-unit9/release/root-search /Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2
  ```
- **Results (two consecutive runs produced identical non-timing output):**
  1. **Cooperative partial-repair multi-compensator LP-guided discrete solve (`search::lp_guided_discrete_solve::<120>`):**
     - Moved coefficients: **`1,465` of `1,920`**.
     - Guard verification: **`active_guards = 0` of `380`**, **`guard_min = 3319399 / 17179869184` (`~ +0.00019321 > 0`)**, `full_guard_vector_matches = true`.
     - Exact rational objective: **`-94674498416698555 / 4503599627370496` (`~ -21.021961597411334`)** (improving on Unit 8's `-94671994445923515 / 4503599627370496` by `-0.00055599`).
  2. **Root Lagrangian Gap Decomposition (`search::certify_gap_decomposition`) and Root RCBT (`search::certify_reduced_cost_fixing`):**
     - Root primal-dual gap $U - L(y_{\text{root}}) \approx 0.007306079116978$ (`0.03474%` relative gap, down from Unit 8's `0.00786207`).
     - Exact gap split (`gap_identity_holds = true`): **`reduced_cost_loss ≈ 0.00264151` (`36.15%` of root gap)** + **`complementary_slackness_loss ≈ 0.00466457` (`63.85%` of root gap, cut by `16.29%` from Unit 8's `0.00557203`)**, with **`1,882` of `1,920` coefficients (`98.02%`) at exact zero reduced-cost penalty**.
     - Root eliminated codes: **`6,122` of `28,800` discrete codes (`21.26%`, `+301` over Unit 8)**; root fixed coefficients: **`112` of `1,920` (`+10` over Unit 8)**.
  3. **Disjunctive-Split Lower Bound and Disjunctive Partition RCBT (`search::certify_disjunctive_split_bound_with_dual::<120>(&p, &d.objective, 24, Some(dual_y_ref))`):**
     - Evaluated all **`67` fractional basic variables** of `delta_hull_lp(p)` (`disjunctive_evaluated_splits = 67`).
     - Best single-variable disjunctive split (`coeff 809`, `q <= -1` vs `q >= 0`): **`-138413805965345675648607375215541291622139312426582589707502405201 / 6582018229284824168619876730229402019930943462534319453394436096` (`~ -21.02908274`, gain `+0.00018493`)**.
     - 24-leaf disjunctive branch-and-bound split tree (`disjunctive_tree_leaves = 24`): **`-69206643834381985661770419441753875297126864236382476036143436075 / 3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.02900400`, gain `+0.00026367`)**, shrinking the **tree-certified discrete optimality gap $U - L_{\text{tree}}$ by `10.43%` from Unit 8's `0.00786207` (`0.03739%`) to `0.00704240` (`0.03349%` relative gap)**.
     - Disjunctive Partition RCBT across the 24 exhaustive leaves: provably eliminates **`6,254` of `28,800` discrete codes (`21.72%`, `+132` beyond root RCBT and `+433` over Unit 8)**, fixes **`112` of `1,920` coefficients** to a single proven optimal destination code (`+10` over Unit 8), and restricts **`179` of `1,920` coefficients (`+27` over Unit 8)** to at most 2 surviving codes.

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `search_certify` and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00704240` (`0.03349%`) discrete optimality gap on `attempt2`:** Unit 9 pushes the certified discrete incumbent to `-94674498416698555 / 4503599627370496` (`~ -21.02196160`), certifies a 24-leaf disjunctive branch-and-bound lower bound `~ -21.02900400` (`+0.00026367` above the root LP), eliminates `6,254 / 28,800` discrete codes (`21.72%`), fixes `112` coefficients to a single code, and restricts `179` coefficients to $\le 2$ codes, leaving `#2828` (`legal-construction-exact-solve`) at status `ready`.

## Plain summary

In Unit 8, a 3- or 4-coefficient repair move was only explored if the first compensating coefficient `i2` completely fixed the worst violated guard `r0` on its own, which blocked cooperative repairs where two or three compensators share the repair of `r0`. In Unit 9, we allow `i2` moves that close at least 50% of `r0`'s deficit into the partial-repair pool and expand candidate step transitions to the full discrete range (`mult in 1..=14`). This improves the certified guard-feasible discrete incumbent on `attempt2` to **`-94674498416698555 / 4503599627370496` (`-21.02196160...`)** and cuts guard complementary-slackness loss by **16.29%**.

On the dual side, we added an exact rational domain-restricted Weak Duality certificate (`certify_domain_dual_bound`) and a 24-leaf disjunctive branch-and-bound split certificate (`certify_disjunctive_split_bound`) that branches on fractional basic variables of `delta_hull_lp` across adjacent discrete codes $(q_i \le q_{\text{down}}) \lor (q_i \ge q_{\text{down}} + 1)$ and intersects reduced-cost fixing across the exhaustive leaf partition. Together, the stronger discrete incumbent and the 24-leaf disjunctive split certificate reduce the certified discrete optimality gap by **10.43%** to **`0.007042` (`0.03349%`)**, eliminate **6,254 of the 28,800 discrete codes (`21.72%`, `+433` over Unit 8)**, fix **112 coefficients** to a single proven optimal destination code, and restrict **179 coefficients** to at most 2 surviving codes.
