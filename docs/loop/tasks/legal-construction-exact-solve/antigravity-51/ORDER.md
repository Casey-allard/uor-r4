# ORDER: legal-construction-exact-solve, antigravity-51

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md). Base: `8c3847f5d`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This unit (Unit 9) tightens the certified discrete upper bound $U$, the certified discrete lower bound, and Reduced-Cost Bound Tightening (RCBT) on the 1,960-variable / 380-guard real case (`attempt2`) by:
1. Upgrading `lp_guided_discrete_solve::<P>` in `docs/labs/legal-construction-exact-solve/solver/src/search.rs` from single-code steps (`step in [-1, 1]`) and first-improving `p_idx` exits to multi-code transitions (`new_code in 0..15`), global steepest-net-gain selection, and dual-slack-guided zero-reduced-cost exchange polish targeting the `70.87%` complementary-slackness loss $\sum_r y_r (G_r \cdot \delta)$ and the `30` non-zero-reduced-cost coefficients.
2. Adding exact rational domain-restricted and disjunctive-split dual bound certification (`certify_domain_dual_bound` and `certify_disjunctive_split_bound`) in `search` and `root-search` to certify an exact rational discrete lower bound $L_{\text{disj}} \ge L(y)$ by splitting fractional basic LP variables across their adjacent discrete codebook values $(m_k \le m_k^-) \lor (m_k \ge m_k^+)$.

## Cancel probe (run first)

Verified at base `8c3847f5d`:
- `lp_guided_discrete_solve::<120>` in `docs/labs/legal-construction-exact-solve/solver/src/search.rs` considers only single-code transitions (`step in [-1, 1]`) during multi-compensator polish and exits on the first `p_idx` with a feasible net improvement, stopping at certified discrete incumbent `-94671994445923515 / 4503599627370496` (`~ -21.02140560`) and primal-dual gap `35407583296 / 4503599627370496` (`~ 0.00786207`, `0.03739%`).
- `search` and `root-search` certify only the root Lagrangian dual bound $L(y)$ and do not yet certify domain-restricted or disjunctive-split discrete lower bounds $L_{\text{disj}} \ge L(y)$ on fractional basic variables.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`

## Done when

- `lp_guided_discrete_solve::<P>` in `docs/labs/legal-construction-exact-solve/solver/src/search.rs` strictly improves the certified discrete incumbent $U$ on `attempt2` below `-94671994445923515 / 4503599627370496` (`~ -21.02140560`) via multi-code transitions, steepest-net-gain selection, and dual-slack-guided exchange polish.
- `search` and `root-search` certify domain-restricted and disjunctive-split exact rational lower bounds (`certify_domain_dual_bound`, `certify_disjunctive_split_bound`) and tightened `certify_reduced_cost_fixing` / `certify_gap_decomposition` metrics on `attempt2`.
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs` verifies the new certificates across finite controls and random instances, and `docs/labs/legal-construction-exact-solve/README.md` and `docs/research/notebook/legal-construction.md` record the Unit 9 measurements on `attempt2`.

## Stop and ask if

- Any file outside the 4 scope paths above must change.
- GPU compute is required (this unit is purely CPU exact-rational + `Dy<120>` solver work).
- Any bar, hold, or item status change is needed before `U == L` (0% gap) is reached.
