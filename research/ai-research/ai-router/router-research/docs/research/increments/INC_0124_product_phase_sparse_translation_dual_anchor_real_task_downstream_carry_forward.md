# INC-0124: Product Phase Sparse Translation Dual-Anchor Real-Task Downstream Carry-Forward

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0123` converts the completed downstream extension into one explicit
downstream LM-proxy real-task comparison:
- lower bank remains systems-only by default
- upper bank remains quality-near systems promotion by default
- the upper-bank bounded-backfill route remains optional comparator-only

The next honest question is whether later downstream real-task work can carry
that exact comparison forward without reopening nondefault sparse translated
routes by habit.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0121` packet manifest, `INC-0122` downstream
  extension, and `INC-0123` downstream comparison read
- keep the lower-bank story systems-only unless a later branch explicitly
  requires a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when a downstream
  real-task question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Start the next downstream real-task branch from the exact downstream
   comparison.
2. Record any comparator reintroduction explicitly.
3. Keep packet scope fixed while testing the next downstream real-task
   question.

## Acceptance
- later downstream real-task branches inherit the explicit downstream
  comparison verbatim by default
- nondefault sparse translated routes only return when a later branch contract
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
  - `results/analysis/inc0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.json`
- Report:
  - `docs/reports/INC0124_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_CARRY_FORWARD.md`

## Reading
- The explicit downstream LM-proxy real-task comparison now has one downstream
  carry-forward contract:
  - lower-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: `carry_as_systems_only_default`
  - upper-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: `carry_as_promoted_real_task_default`
  - upper-bank optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - recommendation: `optional_only`
- This closes the branch positive/explanatory:
  - downstream real-task work now has one explicit downstream carry-forward
    contract
  - later branches no longer need to infer the default downstream comparison
    set from the earlier extension or comparison artifacts

## Resume Note
Resume from the completed `INC-0124` downstream LM-proxy real-task
carry-forward contract. The next branch should turn that exact carry-forward
contract into one reusable downstream real-task packet manifest (`INC-0125`)
rather than rebuilding sparse translated route forks.
