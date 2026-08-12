# Sparse Gated Shell Screen

- stage: `screen`
- config: `configs/proxy_transfer_inc0097_product_phase_sparse_gated_shell_screen.json`
- authoritative gate note: `docs/governance/gates/gate_20260312_095747.md`

## Summary
- `H4XH4_FIELD_A150` remained the only healthy product route.
  - `mse=0.003899`
  - `total_sec=7.956`
  - `shell_pmax=0.5662`
- `H4XH4_FIELD_A150_G030` changed shells on about `6.4%` of routed points but
  collapsed shell balance.
  - `mse=0.003915`
  - `total_sec=7.127`
  - `shell_pmax=0.9846`
  - `product_shell_gate_score_mean=0.02461`
  - `product_shell_multiplier_mean=1.0706`
- `H4XH4_FIELD_A150_B035` also changed shells sparsely, but one seed collapsed
  fully to a single shell.
  - `mse=0.003908`
  - `total_sec=7.463`
  - `eval_shells=1.5`
  - `shell_pmax=0.9860`
  - `product_shell_active_frac=0.0732`
  - `product_shell_states_used=2.5`

## Reading
- Sparse shell control is real, but the pilot fails the route-health gate.
- The failure is shell concentration, not dead phase:
  - field-shift metrics remain nonzero on both sparse candidates
  - the product phase law itself stays live
- No sparse or banded candidate beat the continuous product reference on the
  branch contract.

## Decision
- Close `INC-0097` negative at screen.
- Keep `H4XH4_FIELD_A150` as the fixed product-phase geometry reference.
- Move next to translated route-cost decomposition rather than more shell-law
  threshold tuning.
