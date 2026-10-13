# Promoted Upper-Bank Carry-Forward Contract

## Summary
- Contract: `promoted_upper_bank_single_reference`
- Selection mode carried forward from `INC-0114`: `tie_break_within_tolerance`
- Lower-bank mode: `systems_only_anchor`
- Upper-bank mode: `single_promoted_reference`
- Read: The default broader-comparison packet now carries CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the lower-bank systems-only routed point and CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank dense-near routed reference. CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 remains available only when an explicit pruning/systems comparator is needed, and the lower-bank soft sparse point stays out of the default packet because the lower-bank dense read remains systems-only.

## Default Broader Packet
- Route ids: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Lower-bank default routed point: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Upper-bank promoted routed point: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`

## Comparator Handling
- Optional comparator ids: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Excluded-by-default ids: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`

## Anchors
- Lower dense `DENSE_Q01_T2500`: `top1=0.052000`, `cand_frac=1.000000`, `amortized=0.125365s`
- Lower systems `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`: `top1=0.044400`, `cand_frac=0.189366`, `amortized=0.105716s`
- Lower nondefault `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`: `top1=0.044600`, `cand_frac=0.193328`, `amortized=0.152708s`
- Upper dense `DENSE_Q01_T40000`: `top1=0.048850`, `cand_frac=1.000000`, `amortized=16.770385s`
- Upper promoted `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`: `top1=0.047325`, `cand_frac=0.183764`, `amortized=3.426262s`
- Upper supporting `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`: `top1=0.047287`, `cand_frac=0.182003`, `amortized=3.470096s`

## Inclusion Rules
- Always carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 as the sole upper-bank routed representative in broader comparisons.
- Only add CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 when an explicit upper-bank pruning or systems comparator is required.
- Keep CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 as the default lower-bank routed representative because the lower-bank dense story remains systems-only.
- Do not carry CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.
