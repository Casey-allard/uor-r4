# INC-0119: Product Phase Sparse Translation Dual-Anchor Real-Task Comparison

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0118` extends the completed dual-anchor packet and broader sparse
translated read into the real-task side:
- lower bank remains systems-only by default
- upper bank remains quality-near systems promotion by default
- the exact inherited packet is now the task-side starting point

The next honest question is whether the next explicit real-task comparison can
inherit that exact dual-anchor task-side packet without reopening nondefault
sparse translated routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0116` packet and `INC-0118` task-side extension read
- keep the lower-bank story systems-only unless the branch explicitly requires
  a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when the task-side
  question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next explicit real-task comparison from the exact dual-anchor
   task-side packet.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the next downstream task-side
   question.

## Acceptance
- the next explicit real-task comparison inherits the dual-anchor task-side
  packet verbatim by default
- nondefault sparse translated routes only return when the branch contract
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
  - `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
- Report:
  - `docs/reports/INC0119_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_COMPARISON.md`

## Reading
- The explicit LM-proxy real-task comparison is now fixed from the completed
  dual-anchor packet and task-side extension:
  - lower-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - classification: `systems-only`
    - recommendation: `carry_as_systems_only_default`
  - upper-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - classification: `quality-near systems promotion`
    - recommendation: `carry_as_promoted_real_task_default`
  - optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - recommendation: `optional_only`
- This closes the branch positive/explanatory:
  - the first explicit LM-proxy real-task comparison now inherits the exact
    dual-anchor packet instead of reopening route forks
  - task-side work now has an exact promotion recommendation rather than only a
    packet/extension contract

## Resume Note
Resume from the completed `INC-0119` explicit LM-proxy real-task comparison.
The next branch should carry the completed explicit LM-proxy real-task
comparison into the downstream carry-forward branch (`INC-0120`) rather than
rebuilding sparse translated route forks.
