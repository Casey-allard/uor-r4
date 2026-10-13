# INC-0039: Route/Memory Coordinate Separation

## Status
Completed.

## Hypothesis
The current architecture may be overloading one coordinate with two jobs:
- addressing buckets
- representing memory inside buckets

The next plausible route was:
- address with a geometry-aligned bounded route coordinate
- update prototypes and split memory in the full learned chart coordinate

## Mathematical Basis
1. Pure isometry improves global Poincare alignment.
2. Bounded isometry improves the geometry/runtime tradeoff, but not enough.
3. Full chart scale remains operationally useful.
4. The natural next test was dual coordinates:
   - route coordinate for shells/sectors
   - memory coordinate for prototype updates and local comparisons

## Implementation
- Added `memory_coord_mode={route_chart,full_chart}`.
- Kept shell/sector addressing on the route coordinate.
- Allowed bucket memory/prototype updates to use the full chart coordinate.
- Added tests proving:
  - keys stay fixed when only the memory coordinate changes
  - the returned memory coordinate switches to the full chart when requested
  - Poincare alignment is still measured on the route coordinate

## Evidence
- Config: `configs/proxy_transfer_inc0039_route_memory_screen.json`
- Analysis: `results/analysis/inc0039_route_memory_screen.json`
- Gate: `docs/governance/gates/gate_20260306_084204.md`

## 2-Seed Screen Means
- `HOPF_PHI2_BAND`: `mse=0.003910130`, `total=45.122s`, `pair_mae=0.103762`, health pass
- `DUAL_B050`: `0.003915606`, `60.174s`, `pair_mae=0.055238`, runtime fail
- `DUAL_B025`: `0.003919752`, `56.867s`, `pair_mae=0.004458`, runtime fail
- `DUAL_B075`: `0.003925511`, `58.174s`, `pair_mae=0.077111`, runtime fail
- `BOUND_B050`: `0.003925862`, `58.427s`, `pair_mae=0.055238`, runtime fail
- `HOPF_K25_BASE`: `0.003934246`, `41.476s`, `pair_mae=0.118315`, health pass
- `R0`: `0.003946221`, `45.127s`, collapsed

## Reading
- The branch itself failed operationally:
  - dual coordinates improved alignment strongly
  - dual coordinates sometimes recovered quality relative to the bounded baseline
  - dual coordinates did not recover runtime enough to pass the gate
- The more important result was on the frontier:
  - `HOPF_K25_BASE` passed the gate and beat `R0` on both quality and runtime in this controlled batch
  - `HOPF_PHI2_BAND` also passed the gate and beat `R0` on quality with runtime parity
- Interpretation:
  - route/memory separation is not the next primary fix
  - the current Hopf frontier is now strong enough to justify immediate confirm-level auditing

## Decision
- Kill route/memory separation as the current primary architecture branch.
- Promote a focused fairness/robustness confirm for the live Hopf frontier.
- Current frontier split:
  - operational routed candidate: `HOPF_K25_BASE`
  - widened quality candidate: `HOPF_PHI2_BAND`

## Next Control
- `docs/research/controls/CTRL_0003_hopf_frontier_confirm.md`
