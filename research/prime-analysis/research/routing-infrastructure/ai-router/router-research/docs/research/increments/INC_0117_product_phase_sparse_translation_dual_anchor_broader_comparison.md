# INC-0117: Product Phase Sparse Translation Dual-Anchor Broader Comparison

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0116` froze the default dual-anchor broader-comparison packet for the
fixed sparse translated branch:
- lower anchor:
  - `DENSE_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- upper anchor:
  - `DENSE_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- upper-bank bounded-backfill route remains comparator-only

The next honest question is whether a broader hardware-side or task-side branch
can now start from this fixed dual-anchor packet without silently reopening the
old sparse translated route forks.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact default packet from `INC-0116`
- keep the lower-bank story systems-only unless the new branch explicitly asks
  for a pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill point when the new branch
  explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next broader comparison from the exact `INC-0116` packet.
2. Record any comparator reintroduction explicitly instead of rebuilding the
   old route pair by habit.
3. Keep packet scope fixed while testing the broader downstream question.

## Acceptance
- later broader comparison work inherits the `INC-0116` packet verbatim by
  default
- nondefault sparse translated routes only return when the branch contract says
  so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json`
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
- Report:
  - `docs/reports/INC0117_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON.md`

## Reading
- The fixed `INC-0116` packet now has one explicit broader sparse translated
  comparison read rather than an implied carry-forward:
  - lower-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - classification: `systems-only`
    - baseline: `DENSE_Q01_T2500`
    - top-1 tolerance gap abs: `0.00744`
  - upper-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - classification: `quality-near systems promotion`
    - baseline: `DENSE_Q01_T40000`
    - top-1 tolerance gap abs: `0.00144`
- The upper-bank bounded-backfill route remains explicit but comparator-only:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- This closes the branch positive/explanatory:
  - lower bank remains systems-first by default
  - upper bank remains the near-frontier routed default by default
  - later branches no longer need to infer the dual-anchor comparison read from
    older dense-frontier artifacts

## Resume Note
Resume from the completed `INC-0117` broader comparison audit. The next branch
should carry that exact packet and read into the task-side extension branch
(`INC-0118`) rather than rebuilding sparse translated route forks.
