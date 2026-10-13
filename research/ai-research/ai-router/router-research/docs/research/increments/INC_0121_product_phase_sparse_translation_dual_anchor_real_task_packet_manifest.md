# INC-0121: Product Phase Sparse Translation Dual-Anchor Real-Task Packet Manifest

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0120` freezes the explicit LM-proxy dual-anchor real-task carry-forward
contract:
- lower bank remains the systems-only default
- upper bank remains the promoted real-task default
- the upper-bank bounded-backfill route remains optional comparator-only

The next honest question is whether downstream real-task work can inherit that
exact comparison as one reusable packet manifest instead of reconstructing the
route set from older comparison artifacts.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- inherit the exact `INC-0120` real-task carry-forward contract
- keep the lower-bank story systems-only unless a later branch explicitly
  requires a nondefault pruning/quality comparator
- keep the upper-bank promoted route as the sole default upper-bank routed
  representative
- only reintroduce the upper-bank bounded-backfill route when a downstream
  real-task question explicitly needs a pruning/systems comparator

## Minimal Scope
1. Convert the explicit real-task carry-forward contract into one reusable
   downstream packet manifest.
2. Record exact default, optional, and excluded route ids.
3. Keep packet scope fixed while testing the next downstream real-task
   question.

## Acceptance
- downstream real-task branches inherit one exact packet manifest by default
- nondefault sparse translated routes only return when a later branch contract
  says so explicitly

## Evidence
- Analyses:
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
- Report:
  - `docs/reports/INC0121_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_PACKET_MANIFEST.md`

## Reading
- The downstream LM-proxy real-task packet is now explicit and reusable:
  - packet id:
    `inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest`
  - mode:
    `downstream_dual_anchor_real_task_default_packet`
  - default route ids:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - excluded-by-default lower-bank route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
- This closes the branch positive/explanatory:
  - downstream real-task work now has one exact packet manifest to inherit
  - later branches no longer need to reconstruct the route set from older
    broader or task-side artifacts

## Resume Note
Resume from the completed `INC-0120` real-task carry-forward contract. The next
branch should carry that exact downstream packet manifest into the next
downstream real-task question (`INC-0122`) rather than rebuilding sparse
translated route forks.
