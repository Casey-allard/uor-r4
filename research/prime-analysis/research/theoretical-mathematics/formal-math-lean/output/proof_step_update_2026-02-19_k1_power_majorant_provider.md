# Proof Step Update (2026-02-19): K1 Power-Majorant Provider Wiring

## Objective
Make the final insertion path for the remaining source term operationally simple: one provider instance should now close the bridge if the theorem term is supplied in power-majorant form.

## Lean updates
File:
- `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Added:
- `ZeroToCosSinPowerMajorantProvider`
- `k1_term_from_zero_to_cos_sin_power_majorant_provider`
- `rh_from_zero_to_cos_sin_power_majorant_provider`

## Effect
- No theorem-logic changes to the remaining blocker count.
- The drop-in point for the final non-circular source theorem term is now explicit and minimal for the power-majorant route.

## Verification
- `lake build PrimeRiemannBridgeSpinningTopFrontier` succeeded.
- formal audit still reports:
  - `axiom_count = 0`
  - `proof_remaining_item_count = 1`
