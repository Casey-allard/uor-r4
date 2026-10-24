# ORDER: legal-construction-exact-solve, antigravity-47

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `127632bc3`. Item PR (`References #2828`).

## Goal

In Units 3a (`claude-50`) and 3b (`claude-54`), primal simplex on the 1,960-variable root relaxation (`bb::relaxation`, 440 rows) timed out after 7,800 s without returning a basis or certificate. Unit 4 (`deepseek-57`) found a single-coefficient greedy move (`~ -0.775051`) without an LP optimum or dual bound.

This unit (class: exact-arithmetic host instrument under PM ruling `6342969530` and owner direction `6355973721`) resolves the root-LP and dual-bound blocker:
1. Count `degenerate_pivots` and `bland_switches` in `SearchStats` and clamp refactor rounding dust in `Search::ratio`.
2. Implement float-free `Dy<P>` bounded-variable Dual Simplex with BFRT (`search_dual`), disjunction projection lifting (`try_project_disjunction_lp`, `lift_projected_found`), unit-pivot elimination in `blp::certify_basis`, basis-free Lagrangian dual bound (`certify_lagrangian_dual_bound`), and warm-started LP-guided discrete solver (`lp_guided_discrete_solve`).
3. Certify the exact optimal basis and rational objective of `bb::relaxation` (440×1,960) and `delta_hull_lp` (380×1,920) on `attempt2` (`case::load_verified`, 384/384 sha256 checks), and record the results in `docs/labs/legal-construction-exact-solve/README.md` (Unit 5) and `docs/research/notebook/legal-construction.md`.

## Cancel probe (run first)

Ran `grep -rn "dual_simplex\|certified.*true" docs/labs/legal-construction-exact-solve/` at `e8d6a624b`: no match. Class: instrument.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-47/**`

## Done when

1. On the finite controls, random LP suite, and node LPs, `Dy<120>` search plus `blp::certify_basis` matches `lp::solve` and `blp::solve`.
2. Planted faults: a corrupted basis is rejected by `blp::certify_basis` and repaired by `blp::solve_from_basis`; a negative dual multiplier $y_r < 0$ is rejected by `certify_lagrangian_dual_bound`.
3. On `attempt2` (384/384 sha256 verified), `root-search` certifies `bb::relaxation` and `delta_hull_lp` via `blp::certify_basis` in seconds and reports the Lagrangian dual bound and LP-guided discrete assignment.
4. Crate tests, `tests/no_floats.rs`, clippy, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass.

## Stop and ask if

- `blp::certify_basis` fails on `attempt2` and warm-start repair cannot finish in budget, or any file outside scope must change.
