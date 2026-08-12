# INC-0122: Product Phase Sparse Translation Dual-Anchor Real-Task Downstream Extension

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0121` turns the explicit downstream real-task carry-forward contract into
one reusable packet manifest:
- lower bank remains the systems-only default
- upper bank remains the promoted real-task default
- the upper-bank bounded-backfill route remains optional comparator-only

The next honest question is whether the next downstream real-task branch can
inherit that exact packet manifest without reopening nondefault sparse
translated routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0121` downstream real-task packet manifest
- keep the lower-bank story systems-only unless a later branch explicitly
  requires a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when a downstream
  real-task question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next downstream real-task branch from the exact packet manifest.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the next downstream real-task
   question.

## Acceptance
- later downstream real-task branches inherit one exact packet manifest by
  default
- nondefault sparse translated routes only return when a later branch contract
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
- Report:
  - `docs/reports/INC0122_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_EXTENSION.md`

## Reading
- The downstream LM-proxy real-task inheritance is now explicit from the exact
  packet manifest:
  - default downstream route ids remain fixed:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - lower-bank downstream default remains `systems-only` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - recommendation: `carry_as_systems_only_default`
  - upper-bank downstream default remains `quality-near systems promotion` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - recommendation: `carry_as_promoted_real_task_default`
  - the upper-bank bounded-backfill route remains optional comparator-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- This closes the branch positive/explanatory:
  - later downstream real-task branches can inherit a manifest-backed
    extension artifact instead of inferring defaults from older comparison or
    carry-forward analyses
  - optional comparator reintroduction is now an explicit branch-level choice,
    not an implicit packet rebuild

## Resume Note
Resume from the completed `INC-0122` downstream real-task extension artifact.
The next branch should carry that exact downstream extension into the next
explicit downstream real-task comparison (`INC-0123`) rather than rebuilding
sparse translated route forks.
