# INC-0099 Confirm

## Result
- Confirm completed positive/narrow.

## Confirm Means
- `H4XH4_FIELD_A150_EVT_T070`
  - `mse=0.0038966`
  - `total_sec=6.558`
  - `shell_pmax=0.5702`
  - `eval_shells=2.0`
  - `event_gate_mean=0.3191`
  - `event_gate_cost_proxy=0.3191`
  - `event_gate_active_frac=0.0`
  - health: passed
- `H4XH4_FIELD_A150`
  - `mse=0.0039004`
  - `total_sec=7.427`
  - `shell_pmax=0.5702`
  - `event_gate_mean=1.0000`
  - health: passed

## Reading
- The sparse-event controller survives 4 seeds on the fixed product route law.
- The positive point is soft-sparse, not hard-firing:
  - the hard active fraction stays `0.0`
  - the meaningful sparsity signal is the soft update-mass proxy
    `event_gate_mean≈0.319`
- Route health and shell structure stay matched to the continuous product
  reference while update mass falls by about `68%`.

## Decision
- Close `INC-0099` positive/narrow.
- Promote `H4XH4_FIELD_A150_EVT_T070` as the sparse-event proxy reference.
- Move next to translated carry-forward of the fixed sparse point (`INC-0100`).
