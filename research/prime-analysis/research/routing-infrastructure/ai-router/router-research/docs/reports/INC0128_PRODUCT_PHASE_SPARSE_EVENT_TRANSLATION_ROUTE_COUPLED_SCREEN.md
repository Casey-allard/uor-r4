# INC0128 Product Phase Sparse Event Translation Route-Coupled Screen

## Summary
- `INC-0128` closed positive/explanatory at screen stage.
- The translated sparse-event branch is no longer audit-only once train-gate
  pruning is wired into the translated train bank.
- The current prune coupling is not promotable yet.

## Key Read
- Soft sparse plus prune at threshold `0.02` is inert in practice:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_EGP020_CPX8_Q01_T2500`
  - keep fraction `1.0`
  - top-1 `0.0444`
  - candidate fraction `0.189016`
- Near-hard plus prune at threshold `0.02` is downstream-live but too blunt:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_EGP020_CPX8_Q01_T2500`
  - keep fraction `0.7452`
  - effective train items `1863`
  - top-1 `0.0212`
  - candidate fraction `0.131042`
  - online `0.1307s`

## Decision
- Keep the current translated sparse-event references unchanged.
- Close the old “audit-only” objection to translated sparse-event work.
- Move next to a narrow threshold map on the same coupling law instead of
  claiming a translated sparse-event promotion from this screen.
