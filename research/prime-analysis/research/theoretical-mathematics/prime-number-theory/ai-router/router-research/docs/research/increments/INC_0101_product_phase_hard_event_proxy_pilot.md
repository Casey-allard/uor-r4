# INC-0101: Product Phase Hard Event Proxy Pilot

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0100` closed positive on translated carry-forward of the fixed
soft-sparse controller:
- the sparse-event routed point preserved translated retrieval signal relative
  to the continuous routed product reference
- it materially improved runtime versus that continuous routed reference
- it reached an almost exact lower-bank `Q01` amortized tie with dense exact
- the active event fraction still remained `0.0`

That made the next honest question event discreteness on the proxy harness:
can a hard or near-hard controller stay healthy, or does the branch remain
strictly soft-sparse?

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0099` soft sparse controller as the proxy reference
- keep the confirmed `INC-0100` translated sparse-event point frozen as the
  downstream systems reference, not as a tuning target
- reopen mechanism work only through hard or near-hard event activation on the
  proxy harness
- do not reopen translated bank/cache/packet mapping inside this branch

## Implementation
- Extended the proxy sparse-event controller in `tasks/router_proxy_eval.py`:
  - `event_gate_mode={off,soft_error,hard_error}`
- Added regression coverage for:
  - hard gate threshold behavior
  - hard gate parse surface
  - proxy CLI help exposure
- Added tracked screen/confirm packets:
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_screen.json`
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_confirm.json`
- Compared:
  - continuous product reference `H4XH4_FIELD_A150`
  - soft sparse reference `H4XH4_FIELD_A150_EVT_T070`
  - sharpened near-hard controller `H4XH4_FIELD_A150_EVT_T070_TAU002`
  - true hard controller `H4XH4_FIELD_A150_HARD_T062`
  - controls `HOPF_BASE_K25_PHI` and `R0`

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_screen.json`
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_confirm.json`
- Analyses:
  - `results/analysis/inc0101_product_phase_hard_event_proxy_screen.json`
  - `results/analysis/inc0101_product_phase_hard_event_proxy_confirm.json`
- Reports:
  - `docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_SCREEN.md`
  - `docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_113952.md`
  - `docs/governance/gates/gate_20260312_114258.md`

## Screen Read
- Both event-discreteness candidates passed the 2-seed proxy health gate:
  - `H4XH4_FIELD_A150_EVT_T070_TAU002`
  - `H4XH4_FIELD_A150_HARD_T062`
- The sharpened near-hard controller screened as the strongest event point:
  - `mse=0.0038644`
  - `total_sec=8.130`
  - `shell_pmax=0.5662`
  - `event_gate_mean=0.0206`
  - `event_gate_active_frac=0.0`
- The true hard controller stayed healthy, but only in a mostly-on regime:
  - `mse=0.0039070`
  - `total_sec=8.158`
  - `event_gate_mean=0.8448`
  - `event_gate_active_frac=0.8448`
- That was enough to justify confirm because the branch was no longer failing
  on shell health.

## Confirm Read
- `H4XH4_FIELD_A150_EVT_T070_TAU002` held across 4 seeds:
  - `mse=0.0038642`
  - `total_sec=6.040`
  - `shell_pmax=0.5702`
  - `eval_shells=2.0`
  - `event_gate_mean=0.02055`
  - `event_gate_active_frac=0.0`
- The original soft sparse reference also held:
  - `mse=0.0038966`
  - `total_sec=5.981`
  - `event_gate_mean=0.3191`
  - `event_gate_active_frac=0.0`
- The continuous product reference remained healthy:
  - `mse=0.0039004`
  - `total_sec=6.751`
- The true hard controller also passed health, but not as a genuinely sparse
  point:
  - `mse=0.0039054`
  - `total_sec=6.169`
  - `event_gate_mean=0.8439`
  - `event_gate_active_frac=0.8439`

## Reading
- Near-hard event activation is now positive on the fixed product route law.
- The best discrete-leaning point is not binary hard firing; it is the
  sharpened soft controller `H4XH4_FIELD_A150_EVT_T070_TAU002`.
- That near-hard point cuts update mass to about `2%` of dense EMA pressure
  while preserving route health and slightly improving proxy MSE.
- True hard thresholding can stay healthy on this proxy contract, but only in
  a regime that remains mostly on, not genuinely sparse.
- The branch therefore does not yet justify a strong “hard event” claim.
- What it does justify is narrower and still important:
  - event discreteness can be pushed materially further than `INC-0099`
  - the stable carry-forward point is near-hard, not binary hard

## Decision
- Close `INC-0101` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_EVT_T070_TAU002` as the near-hard proxy event
  reference on the fixed product route law.
- Keep the translated sparse-event reference from `INC-0100` frozen until the
  near-hard point is tested downstream.
- Do not overclaim binary hard-event trainability from this branch.
- Move next to translated carry-forward of the near-hard proxy point
  (`INC-0102`), not to more hard-threshold shaving.
