# INC-0102: Product Phase Near-Hard Event Translation Pilot

## Status
Screen completed negative on 2026-03-12.

## Trigger
`INC-0101` closed positive/narrow on the proxy harness:
- the sharpened near-hard controller
  `H4XH4_FIELD_A150_EVT_T070_TAU002` held across 4 seeds
- it reduced event update mass to about `2%` of dense EMA pressure while
  preserving route health and slightly improving proxy MSE
- the true hard controller stayed healthy only in a mostly-on regime and did
  not justify the same discreteness claim

That made the next honest question downstream carry-forward: does the
confirmed near-hard point survive on the translated chart-resident retrieval
stack?

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0100` soft sparse translated point frozen as the
  translated sparse-event reference
- carry only the confirmed near-hard proxy point from `INC-0101`
- treat translated retrieval as the downstream systems harness, not as a
  retuning surface
- do not reopen bank, cache, or packet mapping inside this branch

## Implementation
- Added tracked prewarm and screen packets:
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_screen.json`
- Compared:
  - dense exact baseline `DENSE_Q01_T2500`
  - continuous translated product reference
    `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - translated soft sparse reference
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - translated near-hard candidate
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_screen.json`
- Analyses:
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json`
- Reports:
  - `docs/reports/INC0102_PRODUCT_PHASE_NEAR_HARD_EVENT_TRANSLATION_SCREEN.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_115408.md`
  - `docs/governance/gates/gate_20260312_115418.md`

## Screen Read
- The translated near-hard candidate preserved the same retrieval signal as the
  soft sparse and continuous translated references:
  - `top1=0.0444`
  - `cand_frac=0.189016`
- But it gave back the systems story instead of improving it:
  - continuous translated product:
    - `online=0.08139s`
    - `amortized=0.11841s`
  - translated soft sparse reference:
    - `online=0.11701s`
    - `amortized=0.14575s`
  - translated near-hard candidate:
    - `online=0.21561s`
    - `amortized=0.26344s`
    - `event_gate_mean=0.02097`
    - `event_gate_active_frac=0.0`
- Dense exact remained above all routed points on top-1 but below the
  translated near-hard point on amortized cost:
  - `top1=0.0516`
  - `amortized=0.25111s`

## Reading
- The confirmed near-hard proxy point does not carry its benefit into the
  translated retrieval harness.
- Downstream, the sharper controller keeps the same routed retrieval signal but
  becomes slower than both:
  - the continuous translated product reference
  - the translated soft sparse reference
- This means the translated sparse-event story remains explicitly soft.
- The near-hard controller is therefore a proxy-only mechanism result for now,
  not a new translated systems reference.

## Decision
- Close `INC-0102` negative at screen stage.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` as the translated
  sparse-event reference.
- Do not promote near-hard event carry-forward into confirm.
- Move next to bounded translated quality recovery on the fixed soft sparse
  translated point (`INC-0103`), not to more event sharpening.
