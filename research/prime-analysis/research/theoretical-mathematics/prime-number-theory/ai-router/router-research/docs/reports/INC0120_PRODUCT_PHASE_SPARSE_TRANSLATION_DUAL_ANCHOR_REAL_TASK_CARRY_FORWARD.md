# Dual-Anchor Real-Task Carry-Forward

## Summary
- Contract: `dual_anchor_real_task_default_comparison`
- Surface: `lm_proxy_real_task`
- Read: The explicit LM-proxy real-task comparison now carries forward CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the lower-bank systems-only default and CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the promoted upper-bank real-task default. CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 remains available only when a downstream real-task branch explicitly needs a pruning or systems comparator.

## Default Downstream Real-Task Routes
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Lower-Bank Default
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Classification: `systems-only`
- Recommendation: `carry_as_systems_only_default`

## Upper-Bank Default
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Classification: `quality-near systems promotion`
- Recommendation: `carry_as_promoted_real_task_default`

## Optional Comparator
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Recommendation: `optional_only`

## Inheritance Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
- Task-side comparisons should update docs/reports/REAL_TASK_COMPARISON.md from this exact inherited packet instead of rebuilding lower-bank or upper-bank sparse route forks.
