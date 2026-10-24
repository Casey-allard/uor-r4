# ORDER: legal-construction-exact-solve, antigravity-49

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `bc3f770f3`. Item PR (`References #2828`).

## Goal

Unit 6 (`antigravity-48`, PR #3113) eliminated 100% of the rounding loss between `certify_lagrangian_dual_bound` and `delta_hull_lp` (`L(y) ≈ -21.0292674520`), improved the certified discrete incumbent to `U = -94662137592886459 / 4503599627370496` (`≈ -21.0192174083`), and proved via `certify_reduced_cost_fixing` that `4,859 / 28,800` discrete codes (`16.87%`) are eliminated and `81 / 1,920` coefficients are fixed to a single optimal code, leaving a `0.01005004` (`0.0478%`) primal-dual gap.

This unit (class: exact-arithmetic host instrument under PM ruling `6342969530` and owner direction `6355973721`) tightens both the primal discrete upper bound `U` and the exact rational certificate on `attempt2`:
1. **Warm-started Simplex Diving + multi-compensator (`k <= 3`) exact rational repair polish (`search::lp_guided_discrete_solve`):**
   - At the optimal basis of `delta_hull_lp`, at least `1,920 - 380 = 1,540` nonbasic variables are already at legal discrete endpoints `q_i = ±7`, while only `<= 380` basic variables are fractional. Add warm-started `Dy<120>` Simplex Diving passes (fixing batches of fractional basic variables to legal discrete codes while re-optimizing the remaining basic variables along the guard hyperplanes) plus a greedy 1-to-3 compensator exact rational repair pass in `run_polish`.
2. **Exact rational Guard-Slack Slab Certificate + tighter RCBT (`search::certify_reduced_cost_fixing`):**
   - By the identity `∑_r y_r (G_r·δ) + ∑_i (c̄_i δ_i(q_i) - m_i) = g·δ - L(y) <= U - L(y)`, every guard row `r` with `y_r > 0` at any guard-feasible assignment `g·δ <= U` is trapped in the exact rational slab `0 <= G_r·δ <= (U - L(y)) / y_r`. Extend `ReducedCostFixing` / `certify_reduced_cost_fixing` to certify the per-guard upper slack bounds `guard_max_slack[r] = Some((U - L(y)) / y_r)` (and count constrained guards `constrained_guards`), and use the tighter `U` from Simplex Diving + multi-compensator polish to eliminate more discrete codes and fix more coefficients.
3. **Record** the tightened discrete upper bound `U`, narrowed optimality gap `U - L(y)`, expanded RCBT code/coefficient fixing census, and certified guard-slab counts in `docs/labs/legal-construction-exact-solve/README.md` (Unit 7) and `docs/research/notebook/legal-construction.md`.

## Cancel probe (run first)

Ran `grep -rn "dive_discrete\|probing_bound_tightening\|multi_compensator" docs/labs/legal-construction-exact-solve/` at `1f24217db`: no match. Class: instrument.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-49/**`

## Done when

1. `lp_guided_discrete_solve::<120>` with warm-started Simplex Diving + multi-compensator exact rational repair polish tightens the certified discrete incumbent on `attempt2` strictly below Unit 6's `-94662137592886459 / 4503599627370496` with `full_guard_vector_matches: true` and `guard_min >= 0`.
2. `certify_reduced_cost_fixing` certifies the tighter RCBT domain reduction (`eliminated_codes > 4,859`, `fixed_coeffs > 81` on `attempt2`) and the exact per-guard slack upper bounds `0 <= G_r·δ <= (U - L(y)) / y_r` for every guard with `y_r > 0`, verified against `bb::solve` across all finite controls and 60 random instances.
3. All solver tests, `tests/no_floats.rs`, `cargo clippy --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass.

## Stop and ask if

- Any file outside the declared scope must change.
