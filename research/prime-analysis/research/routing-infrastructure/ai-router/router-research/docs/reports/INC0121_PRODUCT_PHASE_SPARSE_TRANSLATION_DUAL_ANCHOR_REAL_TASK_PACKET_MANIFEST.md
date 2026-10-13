# Dual-Anchor Real-Task Packet Manifest

## Summary
- Packet id: `inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest`
- Mode: `downstream_dual_anchor_real_task_default_packet`
- Read: This packet freezes the downstream LM-proxy real-task inheritance set. Later downstream branches should start from these four default routes and only reintroduce the upper-bank bounded-backfill comparator when the branch explicitly needs a pruning or systems side-by-side comparison.

## Default Routes
- `DENSE_Q01_T2500` (lower_bank, dense_baseline) from `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` (lower_bank, default_routed_systems_reference) from `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- `DENSE_Q01_T40000` (upper_bank, dense_baseline) from `inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm`
- `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` (upper_bank, promoted_dense_near_reference) from `inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm`

## Optional Comparators
- `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` (upper_bank, supporting_comparator) from `inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm`

## Excluded By Default
- `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` (lower_bank, nondefault_pruning_quality_reference)
- `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` (upper_bank, supporting_comparator)

## Inheritance Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
- Task-side comparisons should update docs/reports/REAL_TASK_COMPARISON.md from this exact inherited packet instead of rebuilding lower-bank or upper-bank sparse route forks.
- Downstream real-task branches should inherit this exact packet manifest rather than reconstructing route ids and args from older broader or task-side artifacts.
