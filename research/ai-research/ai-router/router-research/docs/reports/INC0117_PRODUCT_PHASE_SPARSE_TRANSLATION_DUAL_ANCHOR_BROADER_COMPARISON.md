# Dual-Anchor Broader Comparison

## Summary
- Packet id: `inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet`
- Overall dense frontier claim: `upper-bank near-frontier; lower-bank systems-only`
- Read: Default dual-anchor broader comparison is now explicit: lower bank stays systems-only via CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, while upper bank stays quality-near systems promotion via CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000. CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 remains optional comparator-only.

## Default Packet
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Lower Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Baseline: `DENSE_Q01_T2500`
- Classification: `systems-only`
- Systems verdict: `promote`
- Quality read: `quality_negative`
- Top-1 tolerance gap abs: `0.00744`

## Upper Anchor
- Default routed route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Baseline: `DENSE_Q01_T40000`
- Classification: `quality-near systems promotion`
- Systems verdict: `promote`
- Quality read: `quality_near`
- Top-1 tolerance gap abs: `0.0014400000000000003`

## Optional Comparator
- Route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Inclusion mode: `optional`
- Classification: `quality-near systems promotion`

## Inheritance Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
