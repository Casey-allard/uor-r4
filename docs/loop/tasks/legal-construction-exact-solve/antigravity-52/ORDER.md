# ORDER: antigravity-52, legal-construction-exact-solve (repair after counted CHANGES on #3130)

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `5844d911c`. Repair for PR #3130.

## Why this repair

Independent fresh-context review by `kimi` at `c8ec27bc154501c7ace905040583e086af5a75f9` on PR #3130 returned a counted `CHANGES` with two blocking claim/doc findings and three follow-up items:

1. **Same-dual scope for `tree_eliminated_codes >= root_eliminated_codes`:** `antigravity-51/REPORT.md` line 10 stated that intersecting each leaf's domain reduced-cost test under $y_{\text{root}}$ and $y^{(l)}$ guarantees `tree_eliminated_codes >= root_eliminated_codes` on every instance, whereas at $m > 64$ `certify_disjunctive_split_bound` used its `Dy<P>` search `root_y` while `root-search` passed `dual_y_ref` to `certify_reduced_cost_fixing`. Count inclusion across different root duals requires sharing the same root dual.
2. **README `n_viol` threshold wording:** `docs/labs/legal-construction-exact-solve/README.md` line 255 stated $1 \le \text{n\_viol} \le 25$, whereas `search.rs` caps greedy guard-violation repair at $1 \le \text{n\_viol} \le 5$ when $m \ge 64$ ($1 \le \text{n\_viol} \le 25$ when $m < 64$).
3. **Follow-ups:** (a) add an explicit `deltas == delta(codes)` check in `lp_guided_discrete_solve`; (b) add $m \ge 64$ test coverage in `tests/search_certify.rs`; and (c) document `max_leaves = 1` vs `max_leaves >= 2` semantics on `DisjunctiveSplitCertificate`.

## Cancel probe (run first)

Ran `grep -n "1 \\\\le \\\\text{n\\\\_viol} \\\\le 25" docs/labs/legal-construction-exact-solve/README.md` and inspected `certify_disjunctive_split_bound` in `docs/labs/legal-construction-exact-solve/solver/src/search.rs` at `c8ec27bc154501c7ace905040583e086af5a75f9`: the README line omits the $m \ge 64$ cap of 5, and `certify_disjunctive_split_bound` does not accept the caller's exact root dual `dual_y_ref` at $m > 64$. Probe confirms the repair is needed.

## Files in scope

- `docs/labs/legal-construction-exact-solve/solver/src/search.rs`
- `docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`
- `docs/labs/legal-construction-exact-solve/README.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-51/REPORT.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-52/ORDER.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-52/REPORT.md`

## Done when

- `certify_disjunctive_split_bound_with_dual::<P>(p, upper_bound, max_leaves, extra_root_y)` accepts an optional root dual multiplier slice `extra_root_y` (passed as `Some(dual_y_ref)` by `root-search`), upgrades `root_lb`/`root_y` when `extra_root_y` certifies a tighter root bound, and intersects every leaf's domain reduced-cost survival mask with `extra_root_y` so that every code eliminated by `certify_reduced_cost_fixing(p, dual_y_ref, upper_bound)` is guaranteed eliminated on every leaf for any $m$.
- `lp_guided_discrete_solve` independently verifies `disc_rat[i] == delta(i, cur_q[i])` for all $i \in \{0, \dots, n-1\}$ from scratch over `BigRational`.
- `DisjunctiveSplitCertificate` documents `max_leaves = 1` vs `max_leaves >= 2` semantics.
- `tests/search_certify.rs` includes an $m = 66$ ($m \ge 64$) test verifying exact bounds, survival-mask subset inclusion, and `tree_eliminated_codes >= rcbt.eliminated_codes`.
- `README.md` line 255 and `antigravity-51/REPORT.md` line 10 are updated to match the exact implementation and same-dual scope.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside the listed scope would need to change.
