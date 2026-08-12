# INC-0123: Product Phase Sparse Translation Dual-Anchor Real-Task Downstream Comparison

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0122` turns the downstream LM-proxy real-task packet manifest into one
explicit downstream extension artifact:
- lower bank remains systems-only by default
- upper bank remains quality-near systems promotion by default
- optional comparator reintroduction is now explicit rather than implicit

The next honest question is whether the next explicit downstream real-task
comparison can inherit that exact extension artifact without reopening
nondefault sparse translated routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0122` downstream real-task extension artifact
- keep the lower-bank story systems-only unless the branch explicitly requires
  a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when the downstream
  real-task question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next explicit downstream real-task comparison from the exact
   manifest-backed downstream extension artifact.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the next downstream real-task
   question.

## Acceptance
- the next explicit downstream real-task comparison inherits the downstream
  extension artifact verbatim by default
- nondefault sparse translated routes only return when the branch contract says
  so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
  - `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
- Report:
  - `docs/reports/INC0123_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_COMPARISON.md`

## Reading
- The explicit downstream LM-proxy real-task comparison is now fixed from the
  completed downstream extension artifact:
  - lower-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - classification: `systems-only`
    - recommendation: `carry_as_systems_only_default`
  - upper-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - classification: `quality-near systems promotion`
    - recommendation: `carry_as_promoted_real_task_default`
  - optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - recommendation: `optional_only`
- This closes the branch positive/explanatory:
  - the first explicit downstream LM-proxy real-task comparison now inherits
    the exact downstream extension artifact instead of reopening route forks
  - downstream work now has an exact promotion recommendation rather than only
    a packet or extension contract

## Resume Note
Resume from the completed `INC-0123` explicit downstream LM-proxy real-task
comparison. The next branch should carry that completed downstream comparison
into the downstream carry-forward branch (`INC-0124`) rather than rebuilding
sparse translated route forks.
