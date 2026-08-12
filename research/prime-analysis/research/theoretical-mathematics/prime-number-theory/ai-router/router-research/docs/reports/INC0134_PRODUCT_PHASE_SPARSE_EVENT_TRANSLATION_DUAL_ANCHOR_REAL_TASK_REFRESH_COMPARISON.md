# INC0134 Product Phase Sparse Event Translation Dual-Anchor Real-Task Refresh Comparison

## Summary
- Surface: `lm_proxy_real_task_refreshed`
- Read: Refreshed LM-proxy dual-anchor real-task comparison now uses CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500 as the lower-bank systems-only default, keeps CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500 as the balanced lower-bank quality comparator, keeps CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500 as the lower-bank quality-first comparator, and leaves CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the upper-bank quality-near systems default. CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 is no longer an active default route.

## Default Real-Task Routes
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Lower-Bank Default
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- Top-1: `0.0446`
- Cand frac: `0.193328`
- Amortized: `0.0899s`
- Top-1 delta vs dense: `-0.0074`
- Recommendation: `carry_as_systems_only_default`

## Lower-Bank Comparators
- Balanced: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - top-1 delta vs default: `0.0018`
  - amortized delta vs default: `0.0043s`
  - recommendation: `optional_balanced_quality_comparator`
- Quality-first: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - top-1 delta vs dense: `0.0004`
  - amortized delta vs dense: `0.0158s`
  - recommendation: `optional_quality_first_comparator`

## Upper-Bank Default
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Top-1 delta vs dense: `-0.0015`
- Cand frac delta vs dense: `-0.816236`
- Amortized delta vs dense: `-13.3441s`
- Recommendation: `carry_as_promoted_real_task_default`

## Inheritance Rules
- Use `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the only default lower-bank routed route on current dual-anchor surfaces.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` and `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500` as explicit lower-bank comparators rather than defaults.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` out of default inheritance; only use it when a branch explicitly needs the stale historical comparator.
- Leave the upper-bank promoted default at `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` and the upper-bank optional comparator at `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`.
