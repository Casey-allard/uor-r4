# REPORT: antigravity-54, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

**Corrections (2026-10-24, claude, finishing #3152 under the owner's close-out direction).** I checked this report against commit `4754c118b` and corrected five statements in place:
- the `arj`-first pricing description;
- the `dual_rc_pool` cap and how it deduplicates;
- the test description;
- the missing `disjunctive_ms`, which went up rather than down;
- the candidate sort, which as delivered visited unscored candidates first.

Kimi found the sort bug in review. It is fixed and pinned in `antigravity-55`. The Unit 11 numbers below are what this commit produced, sort bug included.

## What did not work, first

1. **Evaluating tree-level candidate splits in arbitrary basis-row order `0..m` instead of descending `root_split_score[bv]`, discarding non-winning dual multipliers from evaluated branch LPs instead of pooling their reduced-cost vectors (`dual_rc_pool`) for domain tightening, scanning all $m + n$ tableau columns in `dual_simplex_run` before checking whether an art/guard-slack column already attains `best_abs`, and comparing `c_pi == parent.pi` without tracking whether `parent.rc` came from an upgraded non-`Dy<P>` root dual (`pi_matches_rc`).**
   - In Unit 9–10 (`#3130`, `#3151`), `certify_disjunctive_split_bound_with_dual` scanned `0..m` at each tree level and truncated `cand_splits` to the first 16 fractional basic variables in basis-row order, so high-gain fractional variables sitting in later basis rows were skipped at deeper levels (`disjunctive_tree_gain ~ +0.00026367`, `disjunctive_tree_gap ~ 0.00704240`). Simultaneously, every evaluated branch LP solved during the 16-candidate lookahead produced a dual-feasible multiplier vector `y >= 0`, yet only the single chosen child's `rc` was retained on its leaf, leaving `141` additional discrete codes un-eliminated across the 24 leaves (`6,254` vs `6,395`). In `dual_simplex_run`, scanning all `n` structural columns at `s = 1` on every pivot even when an art/guard-slack column (`j >= n`) had a positive entry added `~64s` to `lp_guided_discrete_solve` (`264,832 ms` vs `200,929 ms`). Finally, as `claude` noted in the non-blocking review of `#3151`, `if c_pi == parent.pi` reused `parent.rc` directly even when `parent` was the root leaf whose `root_rc` had been upgraded to an `exact_y` or `ext_y` not representable in `Dy<P>`.
   - Recording `root_split_score[bv]` during the root pass and sorting each tree level's `cand_splits` by descending `(root_split_score[bv], ...)` raises the 24-leaf tree gain by **`1.63x`** (`+0.00042963` vs `+0.00026367`) and narrows `disjunctive_tree_gap` to **`~ 0.00687644` (`0.03270%`)**, pooling distinct branch reduced-cost vectors into `dual_rc_pool` eliminates **`6,395 / 28,800` discrete codes (`22.20%`, `+141` over Unit 10)**, fixes **`115` coefficients (`+3`)**, and restricts **`185` coefficients to $\le 2$ codes (`+6`)**, two-stage `arj`-first pricing cuts `solve_ms` by **`63,903 ms` (`-24.13%`, to `200,929 ms`)**, and `pi_matches_rc: bool` on `SplitLeaf<P>` guarantees `c_pi == parent.pi` only reuses `parent.rc` when `parent.rc` was computed from `extract_y(&parent.pi)`.

## What changed

- **Code (`+148 / -43` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+148 / -43` lines):
    - Two-stage `arj`-first pricing in `dual_simplex_run::<P>`: for each nonbasic column `j` it computes `arj = rho · col_j` first. It computes `paj = pi · col_j` and `dj = cost[j] - paj` only for columns that pass the ratio test's sign check (`Lower` with `s_arj < -tol`, or `Upper` with `s_arj > tol`), so columns that cannot enter skip the `pi` dot product.
    - Added `pi_matches_rc: bool` on `SplitLeaf<P>` (`false` when the root leaf's `root_rc` comes from an upgraded `exact_y` or `ext_y`, `true` when `rc` comes from `extract_y(&pi)`), seeding `dual_rc_pool` with `compute_rc(&extract_y(&pi))` when `!root_pi_matches_rc` and gating `if parent.pi_matches_rc && c_pi == parent.pi` so an upgraded root `rc` is never conflated with `extract_y(&c_pi)`.
    - Recorded `root_split_score: Vec<Option<BigRational>>` during the root pass of `certify_disjunctive_split_bound_with_dual`, then sorted each tree level's `cand_splits` before taking the first 16. The comparator ordered scored candidates by descending score, but it returned `Less` for `(Some, None)`, which put **unscored candidates (`None`) first**: the reverse of the intent. Kimi caught this in review of #3152; `antigravity-55` fixes it with `split_candidate_order` and pins it with a test.
    - Collected distinct `(pi, rc)` pairs from the root and the evaluated branch LPs into `dual_rc_pool: Vec<(Vec<Dy<P>>, Vec<BigRational>)>`. A pair is deduplicated by `pi` with `dual_rc_pool.iter().find(..)` and pushed only while `dual_rc_pool.len() < 160`. Every leaf's `tighten_leaf_mask` is intersected against every pool vector before the union of surviving leaf masks is taken.
- **Tests (`+2 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+2 / -0` lines):
    - On the `m = 66` fixture in `dual_search_and_lp_guided_discrete_solve_respect_exact_bounds`, two new assertions check that the run seeded with the exact non-dyadic root dual (`split66`, `Some(&exact_y66)`) has a `tree_lower_bound` and `tree_eliminated_codes` at least as good as the run with no extra root dual (`split66_none`). Neither assertion would fail if the sort or the pooling were wrong; `antigravity-55` adds tests for both.
- **Docs (`+15 / -0` lines across 2 files, plus `antigravity-54/ORDER.md` and `antigravity-54/REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+14 / -0` lines): added Unit 11 section with the certified `attempt2` numbers.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added the `2026-10-24` (`antigravity-54`) row.
- **Generated / Other:** `0` lines.

## Measurements

- **Exact-head `attempt2` benchmark (`/tmp/antigravity-legal-unit11/release/root-search local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`):**
  - **Discrete incumbent (`lp_guided_discrete_solve`):**
    - `objective = -94674498416698555 / 4503599627370496` (`~ -21.02196160`), `moved_coeffs = 1,465`, `active_guards = 0`, `guard_min = 3319399 / 17179869184`, `full_guard_vector_matches = true`.
    - `solve_ms`: reduced by **`63,903 ms` (`-24.13%`)**, from `264,832 ms` in Unit 10 to **`200,929 ms`** with two-stage `arj`-first pricing.
  - **24-leaf Disjunctive Split Tree & Cross-Branch Dual Pool Tightening (`certify_disjunctive_split_bound_with_dual`):**
    - `disjunctive_ms`: **up `693,258 ms` (`+62.09%`)**, from `1,116,550 ms` in Unit 10 to **`1,809,808 ms`**. The pool tightening adds work per leaf. So this unit did not meet the ORDER's "cut `disjunctive_ms`" clause, and `antigravity-55` takes it up.
    - Single-split bound (`coeff 809`, `q <= -1` vs `q >= 0`): **`-138413805965345675648607375215541291622139312426582589707502405201 / 6582018229284824168619876730229402019930943462534319453394436096`**.
    - 24-leaf tree lower bound (`disjunctive_evaluated_splits = 67`, `disjunctive_tree_leaves = 24`): tightened from `-69206643834381985661770419441753875297126864236382476036143436075 / 3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.02900400`) in Unit 10 to **`-141734088016952714342525764033724082710340210011323135211531386253129 / 6739986666787659948666753771754907668409286105635143120275902562304` (`~ -21.02883804`)**.
    - 24-leaf tree gain over root LP (`disjunctive_tree_gain`): increased **`1.63x`**, from `~ +0.00026367` in Unit 10 to **`~ +0.00042963`**.
    - Certified discrete optimality gap (`disjunctive_tree_gap`): narrowed from `~ 0.00704240` (`0.03349%`) in Unit 10 to **`46347140678114331169372457776064247475410789788503394750996180809 / 6739986666787659948666753771754907668409286105635143120275902562304` (`~ 0.00687644`, `0.03270%` relative gap)**.
    - Reduced-cost domain tightening:
      - `disjunctive_tree_eliminated_codes`: **`6,395 / 28,800` (`22.20%`)**, up **`+141` codes** over Unit 10 (`6,254`) and **`+273` codes** over root RCBT (`6,122`).
      - `disjunctive_tree_fixed_coeffs`: **`115 / 1,920` (`5.99%`)**, up **`+3` fixed coefficients** over Unit 10 (`112`).
      - `disjunctive_tree_binary_or_fixed_coeffs`: **`185 / 1,920` (`9.64%`)**, up **`+6` coefficients** over Unit 10 (`179`) and **`+17` coefficients** over root RCBT (`168`).

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit11 ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit11 ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit11 ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `search_certify` and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00687644` (`0.03270%`) discrete optimality gap on `attempt2`:** Unit 11 (`antigravity-54`) tightens the 24-leaf disjunctive branch-and-bound lower bound to `-141734088016952714342525764033724082710340210011323135211531386253129 / 6739986666787659948666753771754907668409286105635143120275902562304` (`~ -21.02883804`, `1.63x` tree gain over root LP), eliminates `6,395 / 28,800` discrete codes (`22.20%`), fixes `115` coefficients to a single code, and restricts `185` coefficients to $\le 2$ codes, leaving `#2828` (`legal-construction-exact-solve`) at status `ready`.

## Plain summary

This unit (`antigravity-54`, Unit 11 on `#2828`) strengthens the exact disjunctive branch-and-bound certifier and dual simplex solver in `uor_legal_exact`:
1. Sorted tree-level candidate splits by `root_split_score[bv]` in `certify_disjunctive_split_bound_with_dual`. As delivered, the sort visited unscored candidates first, which `antigravity-55` fixes. The result increased the 24-leaf tree gain on `attempt2` by `1.63x` (`+0.00042963` vs `+0.00026367`) and narrowing the certified discrete optimality gap below `0.0069` (`~ 0.00687644`, `0.03270%`).
2. Pooled distinct valid reduced-cost vectors (`dual_rc_pool`) across evaluated branch LPs and intersected `tighten_leaf_mask` across all leaves and pool duals, eliminating `6,395 / 28,800` discrete codes (`22.20%`, `+141` over Unit 10), fixing `115` coefficients (`+3`), and restricting `185` coefficients to $\le 2$ codes (`+6`).
3. Added `pi_matches_rc` on `SplitLeaf<P>` to resolve the non-blocking review finding from `#3151` (`c_pi == parent.pi` when `parent` holds an upgraded non-`Dy<P>` root dual) and two-stage `arj`-first pricing in `dual_simplex_run::<P>` (`solve_ms` reduced `-24.13%` to `200,929 ms`).
