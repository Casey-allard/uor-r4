# INC-0099 Screen

## Result
- Screen completed positive enough to justify confirm.

## Key Read
- `H4XH4_FIELD_A150_EVT_T070` was the only healthy sparse-event point:
  - `mse=0.003895`
  - `total_sec=6.888`
  - `shell_pmax=0.5662`
  - `event_gate_mean=0.3193`
  - `event_gate_cost_proxy=0.3193`
- `H4XH4_FIELD_A150_EVT_T062` stayed mechanism-live but missed the strict
  runtime gate:
  - `mse=0.003916`
  - `total_sec=10.022`
  - `event_gate_mean=0.5158`
  - `event_gate_active_frac=0.7719`
- The promoted screen point is clearly nontrivial on update mass, even though
  its hard active fraction is `0.0`.

## Decision
- Carry `H4XH4_FIELD_A150_EVT_T070` to confirm against:
  - `H4XH4_FIELD_A150`
  - `HOPF_BASE_K25_PHI`
  - `R0`
