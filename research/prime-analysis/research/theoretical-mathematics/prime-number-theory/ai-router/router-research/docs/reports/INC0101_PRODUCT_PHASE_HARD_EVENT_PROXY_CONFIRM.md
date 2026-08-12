# INC-0101 Confirm

- Config:
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_confirm.json`
- Analysis:
  - `results/analysis/inc0101_product_phase_hard_event_proxy_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260312_114258.md`

## Read
- `H4XH4_FIELD_A150_EVT_T070_TAU002` held across 4 seeds as the strongest
  discrete-leaning event point:
  - `mse=0.0038642`
  - `total_sec=6.040`
  - `shell_pmax=0.5702`
  - `event_gate_mean=0.02055`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_HARD_T062` also held health, but not as a genuinely sparse
  point:
  - `mse=0.0039054`
  - `total_sec=6.169`
  - `event_gate_mean=0.8439`
  - `event_gate_active_frac=0.8439`
- The continuous and soft sparse references remained healthy:
  - `H4XH4_FIELD_A150`: `mse=0.0039004`, `total_sec=6.751`
  - `H4XH4_FIELD_A150_EVT_T070`: `mse=0.0038966`, `total_sec=5.981`

## Decision
- Close the branch positive/narrow on near-hard event activation.
- Promote `H4XH4_FIELD_A150_EVT_T070_TAU002` as the proxy near-hard event
  reference.
- Carry the near-hard point, not the mostly-on hard point, into translated
  evaluation next.
