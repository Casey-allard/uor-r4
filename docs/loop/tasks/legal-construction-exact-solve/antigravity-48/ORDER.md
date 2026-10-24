# ORDER: legal-construction-exact-solve, antigravity-48

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `30a2aaa4f`. Item PR (`References #2828`).

## Goal

Unit 5 (`antigravity-47`, PR #3107) certified the exact rational optimal objective of `bb::relaxation` (`~ -21.03019469`) and `delta_hull_lp` (`~ -21.02926767`) on `attempt2` and found an exact guard-feasible discrete assignment at `-94651472617665723/4503599627370496` (`~ -21.01684884`), leaving a `0.01241883` (`0.059%`) discrete optimality gap and `0` active guards (`guard_min ≈ +0.000626 > 0`).

This unit (class: exact-arithmetic host instrument under PM ruling `6342969530` and owner direction `6355973721`) narrows the discrete optimality bracket and certifies exact Reduced-Cost Bound Tightening (RCBT) over `BigRational`:
1. **Exact rational dual multiplier extraction (`blp::certify_basis_with_dual`) and RCBT certificate (`search::certify_reduced_cost_fixing`):** extract the exact rational dual multiplier vector `y = -π >= 0` from the certified `delta_hull_lp` basis, certify the exact rational Lagrangian bound `L(y)` equal to the `delta_hull_lp` optimum, and provably eliminate every discrete destination `q in -7..=7` whose exact reduced-cost penalty `(g_i - (J^T y)_i)(δ_i(q) - δ_i^*)` exceeds `U - L(y)`.
2. **Tighter LP-guided discrete solve:** refine the margin-tightening schedule (`bump_pad` backoff) and add best-gain-first + 2-coefficient (pair) exact rational polish in `lp_guided_discrete_solve` to ride the guard boundaries down toward `0` and tighten the discrete upper bound `U`.
3. **Clean crate clippy:** fix the 3 inherited `needless_range_loop` warnings in `solver/src/bin/single-move.rs` noted in PR #3107 review so `cargo clippy --all-targets -- -D warnings` passes across the entire solver crate.
4. **Record** the tightened discrete upper bound, exact RCBT variable/code elimination census, and commands in `docs/labs/legal-construction-exact-solve/README.md` (Unit 6) and `docs/research/notebook/legal-construction.md`.

## Cancel probe (run first)

Ran `grep -rn "certify_reduced_cost\|reduced_cost_fixing" docs/labs/legal-construction-exact-solve/` at `30a2aaa4f`: no match. Class: instrument.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-48/**`

## Done when

1. `blp::certify_basis_with_dual` returns the exact rational dual multipliers `π` alongside `LpResult::Optimal`, and `search::certify_reduced_cost_fixing` proves on all finite controls and random instances that the exact branch-and-bound optimal assignment lies inside the RCBT-surviving domain `D_i' ⊆ D_i` for every coefficient `i`.
2. Deliberate break: a planted non-optimal discrete incumbent `U < L(y)` or a negative dual multiplier `y_r < 0` is rejected by `certify_reduced_cost_fixing`.
3. On `attempt2` (384/384 sha256 verified), `root-search` certifies the exact rational RCBT variable-fixing census and tightened discrete upper bound `U < -94651472617665723/4503599627370496`.
4. All solver tests, `tests/no_floats.rs`, `cargo clippy --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass.

## Stop and ask if

- Any file outside the declared scope must change.
