# ORDER: legal-construction-exact-solve, antigravity-53

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `63bacc915`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This unit (`antigravity-53`, Unit 10 on `#2828`) closes the follow-up verification, runtime, and algorithmic items from the multi-lab reviews (`claude`, `kimi`, `deepseek`) of Unit 9 (`#3130`, merged at `63bacc915`):

1. **Non-vacuous, planted-fault-verified $m \ge 64$ ($m = 66, n = 8$) test in `tests/search_certify.rs`:** Replace the vacuous $m = 66$ test (`nz_y = 0`, `evaluated_splits = 0`, `tree_leaves = 1`) with an $m = 66$ instance that induces an active non-dyadic exact basis dual multiplier ($y_1 = 1/3$, `nz_y66 > 0`), fractional basic splits (`evaluated_splits > 0`, `tree_leaves > 1`), and an orthogonal row-0 test dual `y_ext66` so planting either fault (`ext_ok = true` or disabling the `extra_root_y` root upgrade) causes `cargo test --locked --test search_certify` to FAIL, and pass `Some(&iter0_y)` in the 69-instance comparison against `rcbt`.
2. **Exact dyadic shift-accumulation in `certify_disjunctive_split_bound_with_dual`:** Eliminate the per-entry `BigRational` `gcd` bottleneck in `compute_rc` (`1,484,988 ms` on `attempt2`) by accumulating dyadic guard rows over a shared power-of-two denominator in `BigInt` before converting once per column `i`, and by evaluating `term_best_for` at the interval's extreme $\delta_i(q)$ rather than multiplying all 15 codes, while preserving bit-for-bit identical `BigRational` outputs and non-dyadic fallback.
3. **Single-path `extra_root_y` validation and `--disjunctive-leaves` + `disjunctive_ms` reporting:** Remove the duplicate `extra_root_y` length/sign pre-check in `certify_disjunctive_split_bound_with_dual`, add `--disjunctive-leaves <N>` (default `24`, `0` to skip) and `"disjunctive_ms"` to `root-search`, re-run `root-search` on `attempt2` at the exact head, and record the measured runtime and bit-for-bit identical certified numbers.

## Cancel probe (run first)

Inspected `tests/search_certify.rs:551-620` and `src/search.rs:1832-1841,1980-2025` on `origin/main` (`63bacc915`): `p66` has `g66` aligned with `planted66` (`nz_y = 0`), `search.rs:1832-1841` duplicates `certify_domain_dual_bound`'s validation, and `compute_rc` performs per-entry `BigRational` subtraction (`~1,485s` on `attempt2`). Probe confirms the unit is needed.

## Files in scope

- `docs/labs/legal-construction-exact-solve/solver/src/search.rs`
- `docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`
- `docs/labs/legal-construction-exact-solve/README.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-53/ORDER.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-53/REPORT.md`

## Done when

- `tests/search_certify.rs` tests an $m = 66$ instance with `nz_y66 > 0`, `evaluated_splits > 0`, `tree_leaves > 1`, and planted-fault verification where forcing `ext_ok = true` or disabling the `extra_root_y` root upgrade causes `cargo test --locked --test search_certify` to FAIL.
- `certify_disjunctive_split_bound_with_dual` uses exact dyadic `BigInt` shift-accumulation in `compute_rc` and extreme-delta selection in `term_best_for`, `root-search` supports `--disjunctive-leaves <N>` and reports `"disjunctive_ms"`, and `attempt2` produces bit-for-bit identical certified outputs.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside the listed scope would need to change.
