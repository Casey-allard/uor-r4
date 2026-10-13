# INC-0115: Product Phase Sparse Translation Promoted Upper-Bank Carry-Forward

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0114` promoted a single upper-bank dense-near routed reference from the
completed sparse translated pair:
- promoted route:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- supporting comparator:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- no further upper-bank dense rescue remains queued

The next honest question is whether future broader hardware-side or task-side
comparison can now move forward using the single promoted upper-bank routed
reference instead of carrying the full upper-bank pair.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the promoted upper-bank dense-near routed reference fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- keep the bounded-backfill upper-bank point only as a supporting comparator,
  not as a parallel live lead
- keep the lower-bank dense read frozen as systems-only
- reopen future comparison only through carry-forward of the promoted
  upper-bank reference

## Minimal Scope
1. Use the promoted upper-bank dense-near routed reference as the sole
   upper-bank representative in the next broader comparison packet.
2. Keep the supporting bounded-backfill point available only when a pruning or
   systems comparator is explicitly needed.
3. Record the simplified carry-forward contract so later branches stop
   reopening the full upper-bank pair by default.

## Acceptance
- future upper-bank carry-forward work defaults to the promoted soft sparse
  reference
- the old upper-bank pair only reopens if a later branch finds a real
  regression against the promoted reference

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.json`
  - `results/analysis/inc0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.json`
- Report:
  - `docs/reports/INC0115_PRODUCT_PHASE_SPARSE_TRANSLATION_PROMOTED_UPPER_BANK_CARRY_FORWARD.md`

## Reading
- `INC-0114` promoted the upper-bank soft sparse route as the single
  upper-bank dense-near routed reference:
  - promoted route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - supporting comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- The carry-forward contract now freezes the default broader-comparison packet
  to a dual-anchor four-route set:
  - `DENSE_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - `DENSE_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- The lower-bank soft sparse point is now excluded by default:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - reason: lower-bank dense claims remain systems-only, not dual-track
- The upper-bank bounded-backfill point is now comparator-only by contract:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - it only returns when a later branch explicitly needs an upper-bank
    pruning/systems side-by-side comparator
- This closes the branch positive/explanatory:
  - later broader comparison work now has one default upper-bank route, not a
    live upper-bank fork
  - later packets should inherit the dual-anchor default set instead of
    silently reopening the old pair

## Resume Note
Resume from the completed `INC-0115` carry-forward contract. The next branch
should use the dual-anchor default broader-comparison packet rather than
reopen the old upper-bank pair or the lower-bank pruning-only point by default.
