# INC-0133: Product Phase Sparse Event Translation Lower-Bank Contract Refresh

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0132` resolved the lower-bank sparse-event translated selection:
- default systems route =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- balanced quality comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- quality-first comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- stale historical comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`

## Branch Contract
- do not reopen lower-bank measurement or mechanism questions
- refresh the broader/task-side/downstream default contract exactly once
- keep the upper-bank default unchanged
- keep the stale bounded-backfill lower-bank route available only as a
  historical comparator

## Minimal Scope
1. Rewrite the explicit lower-bank default route in the broader comparison
   contract.
2. Refresh the task-side and downstream inherited packet docs from that new
   lower-bank default.
3. Stop after the contract refresh; do not launch new retrieval runs.

## Acceptance
- one explicit lower-bank default route is inherited everywhere current
  contracts still matter
- `SBI030` and `SBI080` remain explicit comparators rather than default routes
- the historical lower-bank bounded-backfill route is no longer carried by
  default

## Resume Note
Resume from the completed `INC-0132` selection audit. This branch is pure
contract refresh, not new lower-bank experimentation.

## Artifacts
- `results/analysis/inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh.json`
- `docs/reports/INC0133_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_CONTRACT_REFRESH.md`

## Result
- the lower-bank default is now inherited consistently across broader,
  task-side, real-task, and downstream contract layers:
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- `SBI030` and `SBI080` are now explicit lower-bank comparators on those
  same surfaces rather than stale nondefault leftovers:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- the old lower-bank bounded-backfill route is now historical-only in the
  refreshed contract:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- the upper-bank default remains unchanged:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Decision
- close `INC-0133` positive/explanatory
- stop carrying the stale lower-bank bounded-backfill route as the default on
  broader/task-side/downstream surfaces
- move next to one refreshed real-task comparison branch using the new
  lower-bank contract rather than more inheritance-only artifacts
