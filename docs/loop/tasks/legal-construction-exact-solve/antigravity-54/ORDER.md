# ORDER: legal-construction-exact-solve, antigravity-54

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `2b107324f`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This unit (`antigravity-54`, Unit 11 on `#2828`) tightens the certified disjunctive branch-and-bound lower bound and Disjunctive Partition RCBT on `attempt2`, accelerates `dual_simplex_run::<P>`, and closes the non-blocking finding from `claude`'s review of Unit 10 (`#3151`, merged at `2b107324f`):

1. **Root-split-ranked candidate branching and Cross-Branch Dual Pool Tightening (`search::certify_disjunctive_split_bound_with_dual`):**
   - Rank each leaf's fractional basic splits by descending root single-split bound `root_split_score[bv]` before `.take(16)` (rather than arbitrary basis-row order `0..m`) so every tree level branches on the highest-impact fractional basic variables first.
   - Collect distinct evaluated branch `rc` vectors into a bounded `dual_rc_pool` to tighten both per-leaf exact Weak Duality lower bounds ($\max_{r \in \text{pool}} L(r, B^{(l)})$) and per-leaf Disjunctive Partition RCBT survival masks ($\bigwedge_{r \in \text{pool}} \text{mask}(r, B^{(l)})$) with zero additional LP solves or `compute_rc` calls.
2. **Guard on the `c_pi == parent.pi` shortcut (`claude` review note on `#3151`):**
   - Track `pi_matches_rc: bool` on `SplitLeaf<P>` (`true` iff `leaf.rc` was computed from `extract_y(&leaf.pi)`, `false` when `leaf.rc` came from an upgraded `exact_y` or `ext_y`) so `if parent.pi_matches_rc && c_pi == parent.pi` never skips `extract_y(&c_pi)` when the parent holds an upgraded non-`Dy<P>` root dual.
3. **Two-stage `arj`-first pricing in `dual_simplex_run::<P>` (`src/search.rs:686-709`):**
   - Compute `arj = rho · col_j` first and evaluate `paj = pi · col_j` only for nonbasic columns `j` that pass the dual-simplex sign test (`(st[j] == State::Lower && s_arj < -tol) || (st[j] == State::Upper && s_arj > tol)`), skipping `75–90%` of redundant `Dy<P>` inner products per pivot while preserving bit-for-bit identical `Dy<P>` pivot trajectories.

## Cancel probe (run first)

Inspected `src/search.rs:686-709,2175-2385` on `origin/main` (`2b107324f`): `dual_simplex_run` computes `paj` on every nonbasic column before testing `s_arj`, `SplitLeaf` lacks `pi_matches_rc`, `cand_splits.iter().take(16)` uses basis-row order instead of root split gain, and `eval_rc_mask` uses only `{root_rc, leaf.rc, extra_rc}`. Probe confirms the unit is needed.

## Files in scope

- `docs/labs/legal-construction-exact-solve/solver/src/search.rs`
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`
- `docs/labs/legal-construction-exact-solve/README.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-54/ORDER.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-54/REPORT.md`

## Done when

- `certify_disjunctive_split_bound_with_dual` tracks `pi_matches_rc`, ranks leaf candidate splits by root split bound, and tightens leaf lower bounds and Disjunctive Partition RCBT across `dual_rc_pool`, improving `disjunctive_tree_lower_bound` and `disjunctive_tree_eliminated_codes` on `attempt2` while `dual_simplex_run` uses `arj`-first pricing to cut `solve_ms` and `disjunctive_ms`.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside the listed scope would need to change.
