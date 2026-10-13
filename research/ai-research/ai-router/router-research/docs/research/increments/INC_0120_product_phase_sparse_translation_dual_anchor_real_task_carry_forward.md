# INC-0120: Product Phase Sparse Translation Dual-Anchor Real-Task Carry-Forward

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0119` converts the completed dual-anchor task-side extension into one
explicit LM-proxy real-task comparison read:
- lower bank remains systems-only by default
- upper bank remains quality-near systems promotion by default
- the upper-bank bounded-backfill route remains optional comparator-only

The next honest question is whether later downstream real-task work can carry
that exact comparison forward without reopening nondefault sparse translated
routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0116` packet, the `INC-0118` task-side extension, and
  the `INC-0119` real-task comparison read
- keep the lower-bank story systems-only unless a later branch explicitly
  requires a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when a downstream
  real-task question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next downstream real-task branch from the exact dual-anchor
   real-task comparison.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the next downstream real-task
   question.

## Acceptance
- later real-task branches inherit the explicit dual-anchor real-task
  comparison verbatim by default
- nondefault sparse translated routes only return when a later branch contract
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
- Report:
  - `docs/reports/INC0120_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_CARRY_FORWARD.md`

## Reading
- The explicit LM-proxy real-task comparison now has one downstream
  carry-forward contract:
  - lower-bank default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: `carry_as_systems_only_default`
  - upper-bank default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: `carry_as_promoted_real_task_default`
  - upper-bank optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - recommendation: `optional_only`
- This closes the branch positive/explanatory:
  - downstream real-task work now has one explicit carry-forward contract
  - later branches no longer need to infer the default comparison set from the
    earlier task-side extension or real-task comparison artifacts

## Resume Note
Resume from the completed `INC-0119` explicit real-task comparison artifact.
The next branch should turn that exact carry-forward contract into one reusable
downstream real-task packet manifest (`INC-0121`) rather than rebuilding sparse
translated route forks.
