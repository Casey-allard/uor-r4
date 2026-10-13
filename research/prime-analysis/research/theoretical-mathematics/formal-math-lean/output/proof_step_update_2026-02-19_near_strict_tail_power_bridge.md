# Proof Step Update (2026-02-19): Near-Strict Tail Power Bridge

## Lean work added
Updated:
- `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

New formal layer:
1. `ZeroToCosSinNearStrictTailPowerTerm`
   - Strengthens the power-majorant source shape by adding an eventual strict tail-amplitude dominance condition (`ρ < 1`).
2. `zero_to_cos_sin_power_majorant_of_near_strict_tail_power_term`
   - Projects this stronger term into existing `ZeroToCosSinPhasePowerMajorantTerm`.
3. Closure theorems:
   - `zero_to_cos_sin_phase_of_near_strict_tail_power_term`
   - `k1_term_of_near_strict_tail_power_term`
   - `rh_from_near_strict_tail_power_term`
4. Provider interface:
   - `ZeroToCosSinNearStrictTailPowerProvider`
   - `k1_term_from_near_strict_tail_power_provider`
   - `rh_from_near_strict_tail_power_provider`

## Why this matters
- It formalizes the empirical near-strict tail regime as a named source-term contract.
- It does not change the final blocker taxonomy: still one missing non-circular theorem-term witness.

## Build verification
- `~/.elan/bin/lake build PrimeRiemannBridgeSpinningTopFrontier` ✅
- `~/.elan/bin/lake build PrimeRiemannBridgeFinalTargetEquivalence` ✅

## Status impact
- Formal route is strengthened and cleaner for plugging a future theorem-term witness.
- Open frontier remains `K1-SOURCE`.
