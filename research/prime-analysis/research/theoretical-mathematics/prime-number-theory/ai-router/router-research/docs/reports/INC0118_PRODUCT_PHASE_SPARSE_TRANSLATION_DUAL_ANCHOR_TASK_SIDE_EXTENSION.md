# Dual-Anchor Task-Side Extension

## Summary
- Packet id: `inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet`
- Task-side surface: `real_task_comparison`
- Read: Task-side dual-anchor inheritance is now explicit: lower bank stays systems-only by default via CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, upper bank stays quality-near systems promotion by default via CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000, and CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 remains optional comparator-only.

## Default Task-Side Routes
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Lower Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Baseline: `DENSE_Q01_T2500`
- Classification: `systems-only`
- Quality read: `quality_negative`

## Upper Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Baseline: `DENSE_Q01_T40000`
- Classification: `quality-near systems promotion`
- Quality read: `quality_near`

## Optional Comparator
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Inclusion mode: `optional`
- Classification: `quality-near systems promotion`

## Inheritance Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
- Task-side comparisons should update docs/reports/REAL_TASK_COMPARISON.md from this exact inherited packet instead of rebuilding lower-bank or upper-bank sparse route forks.
