# Dual-Anchor Real-Task Downstream Extension

## Summary
- Packet id: `inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest`
- Downstream surface: `lm_proxy_real_task_downstream`
- Read: Downstream LM-proxy real-task inheritance is now explicit from the exact packet manifest: lower bank stays systems-only by default via CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, upper bank stays quality-near systems promotion by default via CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000, and CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 remains optional comparator-only.

## Default Downstream Routes
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Lower Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Baseline: `DENSE_Q01_T2500`
- Classification: `systems-only`
- Recommendation: `carry_as_systems_only_default`

## Upper Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Baseline: `DENSE_Q01_T40000`
- Classification: `quality-near systems promotion`
- Recommendation: `carry_as_promoted_real_task_default`

## Optional Comparator
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Classification: `quality-near systems promotion`
- Recommendation: `optional_only`

## Branch Requirements
- Start the next downstream real-task branch from the exact INC-0121 packet manifest.
- Record any optional comparator reintroduction explicitly.
- Keep packet scope fixed while testing the next downstream real-task question.

## Inheritance Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
- Task-side comparisons should update docs/reports/REAL_TASK_COMPARISON.md from this exact inherited packet instead of rebuilding lower-bank or upper-bank sparse route forks.
- Downstream real-task branches should inherit this exact packet manifest rather than reconstructing route ids and args from older broader or task-side artifacts.
- Downstream real-task branches should start from this exact manifest-backed extension artifact and must record any optional comparator reintroduction explicitly.
