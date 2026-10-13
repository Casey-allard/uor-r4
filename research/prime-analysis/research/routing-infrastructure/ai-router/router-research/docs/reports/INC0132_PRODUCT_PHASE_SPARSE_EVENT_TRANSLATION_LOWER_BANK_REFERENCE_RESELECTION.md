# INC0132 Product Phase Sparse Event Translation Lower-Bank Reference Reselection

## Summary
- Carry-forward contract: `single_lower_bank_default_plus_two_comparators`
- Selection mode: `near_hard_promoted_over_stale_backfill`
- Default lower-bank route: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- Balanced comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- Quality-first comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- Historical comparator status: `stale_historical_comparator`

## Key Read
- Default systems reference `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - top1 `0.0446`
  - cand_frac `0.193328`
  - amortized `0.0899s`
- Balanced quality comparator `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - top1 delta vs default `0.0018`
  - amortized delta vs default `0.0043s`
- Quality-first comparator `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - top1 delta vs default `0.0078`
  - amortized delta vs default `0.0517s`
  - top1 delta vs dense `0.0004`
  - amortized delta vs dense `0.0158s`
- Historical backfill packet-scope amortized delta `1.8934s`
- Historical backfill packet-scope amortized ratio `18.910x`

## Interpretation
- CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500 becomes the explicit lower-bank default because it keeps the fastest stable translated sparse-event systems profile after the focused prewarmed confirm. CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500 carries the smallest quality lift over the default, while CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500 remains the quality-first comparator. CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 is demoted because its focused amortized time inflated by 1.893355s (18.910x historical), making it unsafe as the default carry-forward route.
