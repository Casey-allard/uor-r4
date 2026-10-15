# #550 — feat(#510): R5 tranche — graph-compiler future-state planner to total (44→43)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T17:28:38Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T17:47:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/550
- **Merged:** 2026-08-09T17:47:12Z
- **Merge commit:** 65ce0061bb34a17c44a3c06fdeac330e173ba3e6
- **Base / Head:** main / UOR-Foundation:r5/gc-planner
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `future_state_planner.rs` off its `Result<_, PlannerError>` surface to a total form. **The module is now fully total (0 sanctioned-violation sites).** The `PlannerError` enum, its `Display`/`Error` impls, and `use std::fmt` are removed.

## Surface
`BoundedGraphPlanner::plan(...) -> Option<PlanTrajectory>` — `None` when no trajectory is a product of these inputs: the start state is unknown or forbidden, or no plan reaches the goal within the horizon and frontier bounds. A bounded search either yields a plan or there is none; the former `PlannerError` variants (`UnknownState` / `InitialStateForbidden` / `HorizonExceeded` / `FrontierExhausted`, plus two never-constructed variants) were diagnostic detail, not a sanctioned condition, and collapse to `None`. The diagnostic-only counters `forbidden_states_entered` and `horizon_capped` existed solely for those error returns and are removed.

`plan()` has no production callers — `semantic_emission_decoupling` uses the `PlanTrajectory` struct, not `plan()`; only tests and BDD call it.

## Ripple (same PR)
`tests/bdd.rs` — the `plan_error` World field is dropped for `plan_rejected: bool`; the compute/attempt steps set it on `None`; the two failure assertions check it. The `verify_determinism` differential closure now returns `Option<PlanTrajectory>` (still `PartialEq`, still deterministic). The import drops `PlannerError`.

## Verification
audit-limits graph-compiler **44 → 43** (future_state_planner.rs 0 sites). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

