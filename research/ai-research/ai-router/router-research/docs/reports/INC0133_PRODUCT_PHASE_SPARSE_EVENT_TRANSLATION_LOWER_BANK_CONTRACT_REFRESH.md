# INC0133 Product Phase Sparse Event Translation Lower-Bank Contract Refresh

## Summary
- Contract id: `inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh`
- Mode: `single_contract_refresh_from_inc0132`
- Read: INC-0132 moved the real lower-bank science: CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500 is now the explicit default systems reference, while the two soft-bias routes are explicit comparator-only options. INC-0133 applies that result once across broader, task-side, and downstream contracts so later branches stop inheriting the stale lower-bank backfill default.

## Lower-Bank Contract
- Default: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- Balanced comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- Quality-first comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- Historical-only comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`

## Upper-Bank Contract
- Default: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Optional comparator: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`

## Refreshed Surfaces
- Broader default routes: `DENSE_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500, DENSE_Q01_T40000, CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Broader optional comparators: `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500, CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Task-side read: Task-side dual-anchor inheritance now uses CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500 as the default lower-bank systems route, keeps the two soft-bias lower-bank routes as comparator-only, and leaves the upper-bank task-side default at CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000.
- Downstream read: Downstream real-task inheritance now uses CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500 as the lower-bank default, keeps CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500 and CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500 optional, and leaves the upper-bank default at CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000.

## Inheritance Rules
- Use `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the only default lower-bank routed route on current dual-anchor surfaces.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` and `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500` as explicit lower-bank comparators rather than defaults.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` out of default inheritance; only use it when a branch explicitly needs the stale historical comparator.
- Leave the upper-bank promoted default at `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` and the upper-bank optional comparator at `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`.
