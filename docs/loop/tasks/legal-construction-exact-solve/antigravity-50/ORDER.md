# ORDER: legal-construction-exact-solve, antigravity-50

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md). Base: `bb90970c2`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This unit (Unit 8) tightens the certified discrete upper bound $U$ and Reduced-Cost Bound Tightening (RCBT) domain reduction on the 1,960-variable / 380-guard real case (`attempt2`) via deep multi-schedule Warm-Started Simplex Diving (diving fractional basic variables to integrality across fine batch sizes) and expanded exact rational multi-compensator polish, and certifies the exact rational Lagrangian gap decomposition $U - L(y) = \sum_i \bar{c}_{i,q_i}(y) + \sum_r y_r (G_r \cdot \delta)$ in `search` and `root-search`.

## Cancel probe (run first)

Verified at base `bb90970c2`:
- `lp_guided_discrete_solve::<120>` stops Simplex Diving after 6 rounds of `dive_batch = 45`, leaving residual fractional basic variables to simultaneous rounding and stopping at discrete objective `-94666121428139195 / 4503599627370496` (`~ -21.02010201`), a `0.00916544` (`0.04358%`) optimality gap above `L(y) = -94707398899627557 / 4503599627370496` (`~ -21.02926745`).
- `search` and `root-search` do not yet certify the exact rational decomposition of the remaining optimality gap $U - L(y)$ into reduced-cost loss $\sum_i \bar{c}_{i,q_i}(y)$ and complementary-slackness loss $\sum_r y_r (G_r \cdot \delta)$ on the discrete incumbent.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`

## Done when

- `lp_guided_discrete_solve::<P>` in `docs/labs/legal-construction-exact-solve/solver/src/search.rs` performs deep multi-schedule Warm-Started Simplex Diving and expanded exact rational multi-compensator polish, strictly improving the certified discrete incumbent $U$ on `attempt2` below `-94666121428139195 / 4503599627370496` (`~ -21.02010201`) and further tightening `certify_reduced_cost_fixing`.
- `search` and `root-search` certify the exact rational Lagrangian gap decomposition $U - L(y) = \sum_i \bar{c}_{i,q_i}(y) + \sum_r y_r (G_r \cdot \delta)$ (`reduced_cost_loss` + `complementary_slackness_loss == gap`) alongside tightened `eliminated_codes`, `fixed_coeffs`, and `guard_max_slack`.
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs` verifies the new invariants across finite controls and random instances, and `docs/labs/legal-construction-exact-solve/README.md` and `docs/research/notebook/legal-construction.md` record the Unit 8 measurements on `attempt2`.

## Stop and ask if

- Any file outside the 4 scope paths above must change.
- GPU compute is required (this unit is purely CPU exact-rational + `Dy<120>` solver work).
- Any bar, hold, or item status change is needed before `U == L(y)` (0% gap) is reached.
