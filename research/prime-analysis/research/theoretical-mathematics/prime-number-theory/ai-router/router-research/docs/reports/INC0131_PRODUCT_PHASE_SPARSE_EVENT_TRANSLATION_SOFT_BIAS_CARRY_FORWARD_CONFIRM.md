# INC0131 Product Phase Sparse Event Translation Soft-Bias Carry-Forward Confirm

## Summary
- `INC-0131` closed positive/explanatory at confirm.
- The lower-bank soft-bias surface survived explicit prewarm and 4-seed
  confirm.
- The branch now splits cleanly into three roles:
  - systems reference = uncoupled near-hard
  - balanced quality comparator = `SBI030`
  - quality-first comparator = `SBI080`

## Key Read
- Lower-bank systems reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - `top1=0.0446`
  - `cand_frac=0.193328`
  - `amortized=0.0899s`
- Balanced quality lift:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - `top1=0.0464`
  - `cand_frac=0.193328`
  - `amortized=0.0942s`
- Quality-first point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - `top1=0.0524`
  - `cand_frac=0.193328`
  - `amortized=0.1416s`
- Dense reference:
  - `DENSE_Q01_T2500`
  - `top1=0.0520`
  - `amortized=0.1258s`

## Decision
- Keep uncoupled near-hard as the lower-bank sparse-event translated systems
  reference.
- Carry `SBI030` as the balanced quality-improving comparator.
- Carry `SBI080` only as a quality-first comparator.
- Do not keep the old lower-bank bounded-backfill point as the active default
  on the strength of older packets alone; it now needs explicit reselection.
