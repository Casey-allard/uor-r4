# REPORT: antigravity-53, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Constructing the $m = 66$ test instance in `tests/search_certify.rs` with `g66` aligned with `planted66` (`nz_y = 0`), performing per-entry `BigRational` subtraction (`gcd`) across all `838` candidate branch evaluations in `compute_rc` and 15 multiplications per coefficient in `term_best_for` (`1,484,988 ms` on `attempt2`), and duplicating `certify_domain_dual_bound`'s length/sign check in `certify_disjunctive_split_bound_with_dual`.**
   - In Unit 9 (`#3130`, merged at `63bacc915`), the $m = 66$ test instance in `tests/search_certify.rs` had `g66` pulling in the same sign pattern as `planted66`, so the unconstrained minimum already satisfied `J delta >= 0` (`exact_y66` was all zero, `evaluated_splits = 0`, and `tree_leaves = 1`). Independent review by `claude` planted two faults — forcing `ext_ok = true` and disabling the `extra_root_y` root lower-bound upgrade — and showed that `search_certify` stayed green on both faults. Additionally, `claude`, `deepseek`, and `kimi` noted that `compute_rc` in `certify_disjunctive_split_bound_with_dual` performed per-entry `BigRational` subtraction (`~1,485s` on `attempt2`), `root-search` lacked a CLI flag to control/skip the disjunctive leaf budget and did not report `"disjunctive_ms"`, `search_certify` did not pass `Some(&iter0_y)` in the 69-instance comparison against `rcbt`, and `search.rs:1833-1842` duplicated `certify_domain_dual_bound`'s length/sign validation.
   - Replacing `p66` in `tests/search_certify.rs` with an $m = 66$ instance that induces a non-dyadic exact basis dual multiplier ($y_1 = 1/3$, `nz_y66 > 0`), fractional basic splits (`evaluated_splits > 0`, `tree_leaves > 1`), and an orthogonal row-0 test dual `y_ext66` catches both planted faults, while pre-scaling dyadic `p.g` and `p.guards` once (`DyadicProblem`) to accumulate dyadic guard rows in `BigInt` shift-and-subtract before reducing once per column `i` (`dyadic_to_rational`) and evaluating `term_best_for` at the interval's extreme $\delta_i(q)$ cuts `disjunctive_ms` by `368,438 ms` (`-24.81%`, from `1,484,988 ms` to `1,116,550 ms`) with 100% bit-for-bit identical `BigRational` outputs.

## What changed

- **Code (`+266 / -64` lines across 2 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+236 / -55` lines):
    - Removed the duplicate `extra_root_y` length/sign pre-check so `certify_domain_dual_bound(p, ext_y, &root_bounds)?` is the single validation path.
    - Added `dyadic_decomp`, `dyadic_to_rational`, and `DyadicProblem` inside `certify_disjunctive_split_bound_with_dual` so `compute_rc` accumulates dyadic dual rows `y_r` via pure `BigInt` shift-and-subtract over a single shared power-of-two denominator $2^{\text{max\_g\_exp} + \text{max\_y\_exp}}$ before reducing once per column `i`, with exact fallback for any non-dyadic $y_r$ (such as $y_1 = 1/3$ in `p66`).
    - Evaluated `term_best_for` by selecting the extreme $\delta_i(q)$ on `[q_lo, q_hi]` ($\min \delta_i$ when $\text{rc}_i > 0$, $\max \delta_i$ when $\text{rc}_i < 0$) and multiplying `rc_i` at most once per coefficient, cached `rc: Vec<BigRational>` on `SplitLeaf<P>`, and computed `fallback_bound` incrementally from `parent.rc[bv]`.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+30 / -9` lines):
    - Added optional CLI flag `--disjunctive-leaves <N>` (default `24`, `0` to skip the disjunctive split tree) and `"disjunctive_ms"` in `"reduced_cost_fixing"`.
- **Tests (`+118 / -18` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+118 / -18` lines):
    - Passed `Some(&iter0_y)` in the 69-instance suite comparison against `rcbt` and verified equality with `certify_disjunctive_split_bound::<120>`.
    - Upgraded the $m = 66, n = 8$ ($m \ge 64$) test so row 1 ($3 \delta_2 - 2 \delta_1 \ge 0$ with $g_1 = -2, g_2 = 1$) forces an active non-dyadic exact dual multiplier $y_1 = 1/3$ (`nz_y66 > 0`) and fractional basic variable $\delta_2 = \frac{2}{3} \delta_1$ (`evaluated_splits > 0`, `tree_leaves > 1`), verifying `split66_none.root_lower_bound < want_dlp66` and `split66.root_lower_bound == want_dlp66` (catching a disabled root upgrade), testing `Some(&y_ext66)` on row 0 where `exact_y66[0] == 0` so code `-7` on coeff 0 survives under `exact_y66` and is eliminated only by `ext_ok` (catching `ext_ok = true`), and asserting `disc66.codes[i]` survives in all `rcbt66` and `split66` survival sets.
- **Docs (`+15 / -0` lines across 2 files, plus `antigravity-53/ORDER.md` and `antigravity-53/REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+14 / -0` lines): added Unit 10 section documenting the planted-fault-verified $m = 66$ test, the exact dyadic shift-accumulation speedup, and `--disjunctive-leaves <N>`.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added the `2026-10-24` (`antigravity-53`) row.
- **Generated / Other:** `0` lines.

## Measurements

- **Planted-fault verification on `tests/search_certify.rs` (`m = 66`):**
  - **Fault 1 (`let ext_ok = true;` at `search.rs:2470`):** `cargo test --locked --test search_certify dual_search_and_lp_guided_discrete_solve_respect_exact_bounds` **FAILED** at `tests/search_certify.rs:716` (`ext_ok intersection must eliminate code -7 on coeff 0`). Restored.
  - **Fault 2 (disabling `if ext_lb > root_lower_bound` at `search.rs:1981`):** `cargo test --locked --test search_certify dual_search_and_lp_guided_discrete_solve_respect_exact_bounds` **FAILED** at `tests/search_certify.rs:677` (`left: -48073745847554457403354355346805795501/10633823966279326983230456482242756608 != right: -217/48`). Restored.
  - **Clean tree:** `cargo test --locked --test search_certify` PASS (`9/9` green in `19.68s`).
- **Exact-head `attempt2` benchmark (`/tmp/antigravity-legal-unit10/release/root-search local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`):**
  - **24-leaf disjunctive certification runtime (`disjunctive_ms`):** reduced by **`368,438 ms` (`6m08s`, `-24.81%`)**, from `1,484,988 ms` (`24m45s`) before `DyadicProblem` + extreme-delta `term_best_for` to **`1,116,550 ms` (`18m37s`)** after.
  - **Certified `attempt2` outputs (`diff -u` excluding `*_ms` is 0 lines — 100% bit-for-bit identical):**
    - Discrete incumbent: **`-94674498416698555 / 4503599627370496` (`~ -21.02196160`)**, `moved_coeffs = 1,465`, `active_guards = 0`, `guard_min = 3319399 / 17179869184`, `full_guard_vector_matches = true`.
    - Single-split bound (`coeff 809`): **`-138413805965345675648607375215541291622139312426582589707502405201 / 6582018229284824168619876730229402019930943462534319453394436096`**.
    - 24-leaf tree lower bound (`disjunctive_evaluated_splits = 67`, `disjunctive_tree_leaves = 24`): **`-69206643834381985661770419441753875297126864236382476036143436075 / 3291009114642412084309938365114701009965471731267159726697218048`**, `disjunctive_tree_gap = 23176609638528624975306367560506907056161491492519094356136235 / 3291009114642412084309938365114701009965471731267159726697218048`.
    - Reduced-cost fixing: root `eliminated_codes = 6,122` (`fixed_coeffs = 112`), 24-leaf tree `disjunctive_tree_eliminated_codes = 6,254` (`disjunctive_tree_fixed_coeffs = 112`, `disjunctive_tree_binary_or_fixed_coeffs = 179`).

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit10 ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit10 ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit10 ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including planted-fault-verified `search_certify` and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00704240` (`0.03349%`) discrete optimality gap on `attempt2`:** Unit 9–10 (`antigravity-51`..`53`) pushes the certified discrete incumbent to `-94674498416698555 / 4503599627370496` (`~ -21.02196160`), certifies a 24-leaf disjunctive branch-and-bound lower bound `~ -21.02900400` (`+0.00026367` above the root LP), eliminates `6,254 / 28,800` discrete codes (`21.72%`), fixes `112` coefficients to a single code, and restricts `179` coefficients to $\le 2$ codes, leaving `#2828` (`legal-construction-exact-solve`) at status `ready`.

## Plain summary

This unit (`antigravity-53`, Unit 10 on `#2828`) resolves all follow-up items from the multi-lab reviews of Unit 9 (`#3130`):
1. Upgraded the $m = 66$ ($m \ge 64$) test in `tests/search_certify.rs` to induce an active non-dyadic exact dual multiplier ($y_1 = 1/3$, `nz_y66 > 0`), fractional basic splits (`evaluated_splits > 0`, `tree_leaves > 1`), and an orthogonal row-0 dual `y_ext66`, verified by planting both faults (`ext_ok = true` and disabling the `extra_root_y` root upgrade) that `search_certify` turns red on each, and passed `Some(&iter0_y)` in the 69-instance suite comparison against `rcbt`.
2. Added exact dyadic `BigInt` shift-accumulation (`DyadicProblem`) in `compute_rc` and extreme-delta selection in `term_best_for` inside `certify_disjunctive_split_bound_with_dual`, plus `--disjunctive-leaves <N>` and `"disjunctive_ms"` in `root-search`, cutting the 24-leaf disjunctive certification time on `attempt2` by `368,438 ms` (`-24.81%`, from `1,484,988 ms` to `1,116,550 ms`) with 100% bit-for-bit identical rational outputs.
3. Removed the duplicate `extra_root_y` length/sign pre-check in `certify_disjunctive_split_bound_with_dual` so `certify_domain_dual_bound` is the single validation path.
