# INC-0118: Product Phase Sparse Translation Dual-Anchor Task-Side Extension

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0117` converted the dual-anchor packet into one broader sparse translated
comparison read:
- lower bank remains systems-only by default
- upper bank remains quality-near systems promotion by default
- the packet is now the fixed inheritance point for broader downstream work

The next honest question is whether a task-side extension can use that exact
packet without reopening nondefault sparse translated routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0116` packet and the `INC-0117` broader-comparison
  read
- keep the lower-bank story systems-only unless the task-side branch
  explicitly requires a nondefault comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when the task-side
  question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next task-side branch from the exact dual-anchor packet.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the downstream task-side question.

## Acceptance
- the next task-side extension inherits the dual-anchor packet verbatim by
  default
- nondefault sparse translated routes only return when the task-side branch
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
  - `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
- Report:
  - `docs/reports/INC0118_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_TASK_SIDE_EXTENSION.md`

## Reading
- The completed dual-anchor packet and broader sparse translated read now
  extend directly onto the real-task side:
  - task-side surface:
    `docs/reports/REAL_TASK_COMPARISON.md`
  - default task-side route ids:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- The default task-side read is now explicit:
  - lower bank remains `systems-only` by default via
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - upper bank remains `quality-near systems promotion` by default via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` remains
    optional comparator-only
- This closes the branch positive/explanatory:
  - the exact dual-anchor packet now carries to the real-task side without
    reopening nondefault sparse translated routes
  - later explicit real-task comparisons can inherit one fixed task-side packet
    rather than rebuilding route forks from older configs

## Resume Note
Resume from the completed `INC-0118` task-side extension artifact. The next
branch should carry the completed dual-anchor task-side extension into the next
explicit real-task comparison (`INC-0119`) rather than rebuilding sparse
translated route forks.
