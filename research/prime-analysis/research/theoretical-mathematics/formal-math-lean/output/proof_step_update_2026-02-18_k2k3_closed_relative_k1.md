# Proof Step Update (2026-02-18): K2/K3 Closed Relative to K1

## What was proved

In `PrimeRiemannBridgeSpinningTopFrontier.lean`:

- `tendsto_exp_affine_nat_div_pos` (helper)
- `omega_abs_from_eventual_sequence` (helper)
- `ingham_payload_of_zero_to_cos_sin_phase`
- `ingham_payload_of_linear_phase_witness_step_results_non_circular`

## Mathematical effect

This proves that from the single K1-style decomposition assumption (cos/sin main term + vanishing normalized remainder), the final Ingham payload follows directly.

Therefore:
- K2 and K3 are now theorem-closed relative to K1.
- K4 remains already closed by existing envelope promotion lemmas.
- Only K1 remains genuinely open.

## Verification

- `lake build` passed.
- `formal_axiom_audit_2026-02-18.json` remains at `axiom_count = 0`.
