# REPORT: antigravity-52, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Evaluating Disjunctive Partition Reduced-Cost Fixing (`certify_disjunctive_split_bound`) without threading the caller's exact root dual `dual_y_ref`, and leaving the $m \ge 64$ greedy repair cap (`n_viol <= 5`) implicit in `README.md`.**
   - In `antigravity-51` (`c8ec27bc154501c7ace905040583e086af5a75f9`), `certify_disjunctive_split_bound` intersected each leaf's domain reduced-cost survival mask with its internal `root_y` (derived from `blp::certify_basis_with_dual` on $m \le 64$ or `Dy<P>` dual search on $m > 64$), whereas `root-search` passed `dual_y_ref` (`exact_y` from `blp::certify_basis_with_dual` when available) to `certify_reduced_cost_fixing`. Independent review by `kimi` on PR #3130 identified that count inclusion `tree_eliminated_codes >= rcbt.eliminated_codes` is only guaranteed across all $m$ when both operators evaluate root reduced-cost fixing under the same root dual, and noted that `README.md` line 255 stated $1 \le \text{n\_viol} \le 25$ without naming the $m \ge 64$ cap of $5$.
   - Adding `certify_disjunctive_split_bound_with_dual::<P>(p, upper_bound, max_leaves, extra_root_y: Option<&[BigRational]>)` (wired with `Some(dual_y_ref)` in `root-search`), intersecting every leaf's survival mask with `extra_root_y`, adding an explicit from-scratch `disc_rat[i] == delta(i, cur_q[i])` check in `lp_guided_discrete_solve`, documenting `max_leaves = 1` semantics on `DisjunctiveSplitCertificate`, adding $m = 66$ ($m \ge 64$) test coverage in `tests/search_certify.rs`, and updating `README.md` line 255 and `antigravity-51/REPORT.md` line 10 resolves all blocking and non-blocking findings.

## What changed

- **Code (`+94 / -34` lines across 2 files in `antigravity-52`; `+823 / -104` lines vs `origin/main`):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+87 / -32` lines in `antigravity-52`; `+805 / -104` lines vs `origin/main`):
    - Added `certify_disjunctive_split_bound_with_dual::<P>(p, upper_bound, max_leaves, extra_root_y: Option<&[BigRational]>)` (with `certify_disjunctive_split_bound` delegating with `None`), validating `extra_root_y` via `certify_domain_dual_bound`, upgrading `root_lower_bound`/`root_y` whenever `extra_root_y` certifies a tighter root bound, and intersecting every leaf's domain reduced-cost survival mask with `extra_root_y` so that `tree_surviving_codes[i] \subseteq rcbt(extra_root_y).surviving_codes[i]` and `tree_eliminated_codes >= rcbt(extra_root_y).eliminated_codes` hold identically for every $m$.
    - Clarified `max_leaves = 1` (`leaf_count = 1`, `evaluated_splits = 0`, `best_single_split_bound == tree_lower_bound == root_lower_bound`) vs `max_leaves >= 2` field semantics on `DisjunctiveSplitCertificate`.
    - Added explicit independent `disc_rat[i] == delta(i, cur_q[i])` verification over `BigRational` for all $i \in \{0, \dots, n-1\}$ in `lp_guided_discrete_solve`.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+7 / -2` lines in `antigravity-52`; `+18 / -0` lines vs `origin/main`):
    - Wired `search::certify_disjunctive_split_bound_with_dual::<120>(&p, &d.objective, 24, Some(dual_y_ref)).ok()` so root RCBT and disjunctive tree RCBT share the identical root dual `dual_y_ref`.
- **Tests (`+101 / -0` lines across 1 file in `antigravity-52`; `+146 / -0` lines vs `origin/main`):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+101 / -0` lines in `antigravity-52`; `+146 / -0` lines vs `origin/main`):
    - Added negative-control tests for `certify_disjunctive_split_bound_with_dual` (rejecting wrong-length and negative `extra_root_y`), `max_leaves = 1` semantics verification, code-level survival subset inclusion (`tree_surviving_codes[i] \subseteq rcbt.surviving_codes[i]`), and a deterministic $m = 66, n = 8$ ($m \ge 64$) instance test.
- **Docs (`+6 / -6` lines across 2 files in `antigravity-52`, plus `antigravity-52/ORDER.md` and `antigravity-52/REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+4 / -4` lines): stated $1 \le \text{n\_viol} \le 5$ on $m \ge 64$ ($1 \le \text{n\_viol} \le 25$ on $m < 64$) and documented `certify_disjunctive_split_bound_with_dual`.
  - [`docs/loop/tasks/legal-construction-exact-solve/antigravity-51/REPORT.md`](../antigravity-51/REPORT.md) (`+2 / -2` lines): scoped line 10 and line 48 to `certify_disjunctive_split_bound_with_dual(..., Some(dual_y_ref))`.
- **Generated / Other:** `0` lines.

## Measurements

- **Test suite (`tests/search_certify.rs`):**
  - All 9 finite controls, 60 random instances, and the new $m = 66, n = 8$ ($m \ge 64$) instance pass exact bound monotonicity (`root_lower_bound <= best_single_split_bound <= tree_lower_bound <= disc.objective`), code-level survival subset inclusion (`tree_surviving_codes[i] \subseteq rcbt.surviving_codes[i]`), and count monotonicity (`tree_eliminated_codes >= rcbt.eliminated_codes`).
- **Saved `attempt2` case (`root-search`):**
  - Because `d.dual_y` and `exact_y` (`dual_y_ref`) coincide on `attempt2`, passing `Some(dual_y_ref)` to `certify_disjunctive_split_bound_with_dual::<120>(&p, &d.objective, 24, Some(dual_y_ref))` preserves all Unit 9 certified numbers identically: discrete incumbent **`-94674498416698555 / 4503599627370496` (`~ -21.02196160`)**, `1,465` moved coefficients, `0/380` active guards (`guard_min = 3319399 / 17179869184`), 24-leaf disjunctive lower bound **`-69206643834381985661770419441753875297126864236382476036143436075 / 3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.02900400`)**, root `eliminated_codes = 6,122`, and tree `eliminated_codes = 6,254` (`112` fixed coefficients, `179` binary-or-fixed coefficients).

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit9 ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `search_certify` with $m = 66$ coverage and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00704240` (`0.03349%`) discrete optimality gap on `attempt2`:** Unit 9 (`antigravity-51` + `antigravity-52`) pushes the certified discrete incumbent to `-94674498416698555 / 4503599627370496` (`~ -21.02196160`), certifies a 24-leaf disjunctive branch-and-bound lower bound `~ -21.02900400` (`+0.00026367` above the root LP), eliminates `6,254 / 28,800` discrete codes (`21.72%`), fixes `112` coefficients to a single code, and restricts `179` coefficients to $\le 2$ codes, leaving `#2828` (`legal-construction-exact-solve`) at status `ready`.

## Plain summary

This repair order addresses all blocking and non-blocking findings from `kimi`'s review of PR #3130:
1. Added `certify_disjunctive_split_bound_with_dual` accepting `extra_root_y: Option<&[BigRational]>` (passed as `Some(dual_y_ref)` by `root-search`) and intersecting every leaf's survival mask with `extra_root_y`, so that every code eliminated by root RCBT under `dual_y_ref` is provably eliminated by the disjunctive split tree across all $m$ (`m < 64` and `m >= 64`).
2. Updated `README.md` line 255 to state the exact $1 \le \text{n\_viol} \le 5$ cap for $m \ge 64$ ($1 \le \text{n\_viol} \le 25$ for $m < 64$) and scoped `antigravity-51/REPORT.md` line 10 to the shared root dual.
3. Added explicit `disc_rat[i] == delta(i, cur_q[i])` verification in `lp_guided_discrete_solve`, documented `max_leaves = 1` semantics on `DisjunctiveSplitCertificate`, and added $m = 66$ ($m \ge 64$) test coverage in `tests/search_certify.rs`.
