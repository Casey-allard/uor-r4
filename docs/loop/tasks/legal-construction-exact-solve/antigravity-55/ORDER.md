# ORDER: legal-construction-exact-solve, antigravity-55

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `642674f97`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This repair unit (`antigravity-55`, repairing `kimi`'s `CHANGES` review on PR `#3152` at `953a2312932299d6d223ae1668a8e3d52d7dcd9a`) resolves the `disjunctive_ms` runtime regression (`1,809,808 ms` vs Unit 10's `1,116,550 ms`) and the `REPORT.md` accuracy findings:

1. **Cut `disjunctive_ms` below Unit 10 (`1,116,550 ms`), keeping Unit 11's rational outputs bit-for-bit (`src/search.rs`):** reduce `dyadic_to_rational` with `BigRational::new_raw` after stripping shared factors of two. Sum with a dyadic shift-and-add `sum_rationals`. Cache `DualPoolEntry { pi, rc, root_terms, root_lb, root_span }` so leaf bounds and `tighten_leaf_mask` update only over the split coordinates (at most 23 of 1,920).
2. **Repair `antigravity-54/REPORT.md` and record `antigravity-55/REPORT.md`:** describe the code as delivered (`arj`-first pricing, `dual_rc_pool` capped at `160` with `.find` deduplication, the `split66`/`split66_none` assertions), and record `disjunctive_ms` and its direction in both.

## Cancel probe (run first)

Verified `kimi`'s review comment `IC_kwDOS1s4Zc8AAAABfI8RYg` on PR `#3152` and inspected `src/search.rs:2012-2025,2178-2567` and `antigravity-54/REPORT.md`: `eval_branch`, leaf pool tightening, and `tighten_leaf_mask` re-sum all `1,920` `BigRational` terms from scratch across all `24 * 160` `(leaf, pool_rc)` pairs, causing the `+693,258 ms` increase in `disjunctive_ms`. Probe confirms the repair is needed.

## Files in scope

- `docs/labs/legal-construction-exact-solve/solver/src/search.rs`
- `docs/labs/legal-construction-exact-solve/README.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-54/REPORT.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-55/ORDER.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-55/REPORT.md`
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs` (added by claude, 24 Oct, finishing this PR under owner direction: kimi's addendum and claude's CHANGES need tests)

## Done when

- Both `solve_ms` and `disjunctive_ms` on `attempt2` are lower than Unit 10 (`264,832 ms` and `1,116,550 ms`) with bit-for-bit identical Unit 11 rational outputs (`disjunctive_tree_lower_bound`, `disjunctive_tree_eliminated_codes = 6,395`, `disjunctive_tree_fixed_coeffs = 115`, `disjunctive_tree_binary_or_fixed_coeffs = 185`), and `antigravity-54/REPORT.md` + `antigravity-55/REPORT.md` accurately describe the delivered code, tests, and timings.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.
- The split-candidate sort visits scored candidates before unscored ones, pinned by a >16-candidate ordering test; pooling is pinned by a with-pool versus without-pool test (both with planted faults).

**Note (claude, 24 Oct, D68 decision review).** The first bullet and the third cannot both hold at the head, because the Unit 11 outputs came from the unscored-first sort. They are met in sequence instead:
- At `0884462c0`, the speed-up alone gives Unit 11's outputs bit-for-bit.
- At the head, the sort fix makes them strictly tighter: 6,406 / 115 / 186 and a higher lower bound.

`REPORT.md` records both runs.

## Stop and ask if

- Any file outside the listed scope would need to change.
