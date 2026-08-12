# INC-0134: Product Phase Sparse Event Translation Dual-Anchor Real-Task Refresh Comparison

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0133` refreshed the live lower-bank contract without reopening
measurement:
- default lower-bank systems route =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- lower-bank balanced quality comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- lower-bank quality-first comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- historical lower-bank comparator only =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- unchanged upper-bank default =
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Branch Contract
- do not reopen lower-bank route-mechanism tuning
- use the refreshed lower-bank contract as inherited input
- run one explicit refreshed LM-proxy real-task comparison
- determine whether the lower-bank default and comparator reads still hold on
  the actual current task-side packet

## Minimal Scope
1. Build one refreshed dual-anchor real-task comparison packet from the
   `INC-0133` contract.
2. Compare the lower-bank default against `SBI030`, `SBI080`, dense lower
   baseline, dense upper baseline, and the unchanged upper-bank routed
   default.
3. Stop after the refreshed comparison and carry-forward decision; do not open
   another contract-only branch.

## Acceptance
- one explicit refreshed real-task comparison artifact exists
- the lower-bank default is either reaffirmed or replaced from real-task
  evidence rather than stale contract inheritance
- `SBI030` and `SBI080` remain clearly classified as promoted default,
  balanced comparator, or quality-first comparator from the real-task read
- the branch ends in a real carry-forward recommendation, not another packet
  rearrangement loop

## Artifacts
- `results/analysis/inc0134_product_phase_sparse_event_translation_dual_anchor_real_task_refresh_comparison.json`
- `docs/reports/INC0134_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_DUAL_ANCHOR_REAL_TASK_REFRESH_COMPARISON.md`

## Result
- the refreshed real-task comparison reaffirms
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the lower-bank
  systems-only default:
  - `top1=0.0446`
  - `cand_frac=0.193328`
  - `amortized=0.0899s`
  - `top1 delta vs dense = -0.0074`
- `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` remains the
  balanced lower-bank quality comparator:
  - `top1 delta vs default = +0.0018`
  - `amortized delta vs default = +0.0043s`
- `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500` remains the
  quality-first lower-bank comparator:
  - `top1 delta vs dense = +0.0004`
  - `amortized delta vs dense = +0.0158s`
- upper bank stays unchanged:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  remains the `quality-near systems promotion` default
- the old lower-bank bounded-backfill point is no longer an active default
  route:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`

## Decision
- close `INC-0134` positive/explanatory
- keep `TAU002` as the lower-bank default real-task route
- keep `SBI030` as the balanced lower-bank quality comparator
- keep `SBI080` as the quality-first lower-bank comparator
- move next to a focused lower-bank quality/systems frontier branch instead of
  another contract refresh
