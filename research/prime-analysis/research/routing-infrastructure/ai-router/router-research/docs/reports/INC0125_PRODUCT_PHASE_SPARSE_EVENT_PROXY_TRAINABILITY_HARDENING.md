# INC-0125 Product Phase Sparse Event Proxy Trainability Hardening

## Summary
- Harder proxy training did not kill the sparse-event branch.
- The fixed near-hard controller `H4XH4_FIELD_A150_EVT_T070_TAU002` remained
  healthy, materially sparse, and best on confirm-stage proxy MSE.
- The true hard gate `H4XH4_FIELD_A150_HARD_T062` remained mostly-on, so this
  branch does not justify a binary hard-event claim.

## Confirm Read
- `H4XH4_FIELD_A150_EVT_T070_TAU002`
  - `mse=0.003859`
  - `total_sec=10.213`
  - `event_gate_mean=0.020038`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_EVT_T070`
  - `mse=0.003895`
  - `total_sec=10.184`
  - `event_gate_mean=0.318959`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_HARD_T062`
  - `mse=0.003892`
  - `total_sec=11.919`
  - `event_gate_mean=0.840375`
  - `event_gate_active_frac=0.840375`
- `R0`
  - fails the route-health gate on this harder load
  - should not be used as the acceptance surface for this branch

## Interpretation
- The project now has a hardened near-hard proxy result, not just a toy
  screen-stage hint.
- Soft sparse and near-hard both remain trainable on the fixed product route
  law.
- True hard firing still looks like a mostly-on controller rather than a clean
  sparse regime.
- The next real question is no longer whether sparse event proxy trainability
  exists. It is why the near-hard controller survives here but failed in the
  translated carry-forward branch.

## Artifacts
- `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json`
- `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json`
- `docs/governance/gates/gate_20260312_184338.md`
- `docs/governance/gates/gate_20260312_184913.md`
