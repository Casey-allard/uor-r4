# INC0130 Product Phase Sparse Event Translation Route-Coupled Soft-Bias Screen

## Summary
- `INC-0130` closed positive/explanatory at screen stage.
- Soft score bias is a genuinely downstream-live translated sparse-event
  surface.
- The branch split cleanly into a balanced quality-lift point and a
  quality-first point.

## Key Read
- Every `SBI` route changed downstream retrieval behavior without deleting
  train items:
  - `event_gate_retrieval_surface_active_mean=1.0`
  - `cand_frac=0.189016` on every soft-bias point
- Balanced lower-bank quality lift:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - `top1=0.0464`
  - `online=0.0996s`
  - `amortized=0.1330s`
- Quality-first point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - `top1=0.0548`
  - `online=0.1954s`
  - `amortized=0.2443s`

## Decision
- Keep uncoupled near-hard as the lower-bank sparse-event systems reference.
- Carry `SBI030` forward as the balanced soft-bias candidate.
- Keep `SBI080` only as a quality-first comparator until prewarmed confirm
  says it is real enough to matter.
