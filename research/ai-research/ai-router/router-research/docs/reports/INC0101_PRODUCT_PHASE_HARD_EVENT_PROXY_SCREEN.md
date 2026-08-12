# INC-0101 Screen

- Config:
  - `configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_screen.json`
- Analysis:
  - `results/analysis/inc0101_product_phase_hard_event_proxy_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260312_113952.md`

## Read
- Both event-discreteness candidates passed the 2-seed proxy health gate.
- `H4XH4_FIELD_A150_EVT_T070_TAU002` screened as the strongest event point:
  - `mse=0.0038644`
  - `total_sec=8.130`
  - `shell_pmax=0.5662`
  - `event_gate_mean=0.0206`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_HARD_T062` stayed healthy, but remained mostly on:
  - `mse=0.0039070`
  - `total_sec=8.158`
  - `event_gate_mean=0.8448`
  - `event_gate_active_frac=0.8448`

## Decision
- Carry both the near-hard and true hard points to confirm.
- Treat the sharpened soft controller as the screen leader for event
  discreteness.
