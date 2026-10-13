# Proof Step Update (2026-02-19): Asymptotic Strict-Tail Bridge to K1/Pintz

## Objective
Convert the near-strict finite-tail route into a stricter asymptotic theorem shape that is less tied to finite-grid threshold behavior.

## Lean additions
Updated:
- `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`
- `research/formal/lean/PrimeRiemannBridgeNearStrictTailToPintz.lean`

New source theorem contract:
1. `ZeroToCosSinAsymptoticStrictTailPowerTerm`
   - Requires eventual bound `|R(x)/x^β| <= ε * amplitude` for every `ε>0`.

New reduction theorem:
2. `zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term`
   - Chooses `ε = 1/2` to derive the existing near-strict (`ρ<1`) term non-circularly.

New K1/RH closures:
3. `k1_term_of_asymptotic_strict_tail_power_term`
4. `rh_from_asymptotic_strict_tail_power_term`
5. `ZeroToCosSinAsymptoticStrictTailPowerProvider` + provider closures.

New Pintz-slot closures:
6. `pintz_term_of_asymptotic_strict_tail_power_term`
7. `pintz2017_formalized_of_asymptotic_strict_tail_power_term`
8. `AsymptoticStrictTailPowerPintzProvider` + provider RH closure theorems.

## Why this matters
- This creates a theorem-layer target aligned to the remaining K1 source gap that is asymptotic and discretization-stable in form.
- It does not finish RH, but it makes the remaining missing witness more explicit: prove one non-circular asymptotic strict-tail theorem term.

## Build verification
- `lake build PrimeRiemannBridgeSpinningTopFrontier PrimeRiemannBridgeNearStrictTailToPintz PrimeRiemannBridgeFinalTargetEquivalence` ✅
- `python3 research/formal_axiom_audit.py ...` => `axiom_count = 0` ✅

## Status impact
- Remaining open kernel remains one item: `K1-SOURCE`.
- Frontier is now tightened from finite near-strict observation to explicit asymptotic strict-tail theorem shape.
