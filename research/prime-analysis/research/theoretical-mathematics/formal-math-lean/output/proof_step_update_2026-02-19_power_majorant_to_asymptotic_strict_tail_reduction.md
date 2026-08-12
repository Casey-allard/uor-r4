# Proof Step Update (2026-02-19): Power-Majorant -> Asymptotic Strict-Tail Reduction

## Objective
Tighten the single remaining kernel route by proving the new asymptotic strict-tail theorem shape is implied by the existing power-majorant source shape, and by wiring this reduction through the published/Pintz boundaries.

## Lean changes
Updated:
- `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`
- `research/formal/lean/PrimeRiemannBridgeNearStrictTailToPintz.lean`
- `research/formal/lean/PrimeRiemannBridgeFinalTargetEquivalence.lean`

### A) Core reduction theorem
Added in `SpinningTopFrontier`:
1. `zero_to_cos_sin_asymptotic_strict_tail_power_of_power_majorant_term`
   - Uses `|R/x^beta| <= C*x^{-eta}` with `eta>0` and `a,b` nontrivial to derive
     `forall epsilon>0, eventually |R/x^beta| <= epsilon*sqrt(a^2+b^2)`.

2. `zero_to_cos_sin_power_majorant_of_asymptotic_strict_tail_power_term`
   - Explicit reverse projection via the existing asymptotic->near-strict->power chain.

### B) Published boundary refinement
Added in `SpinningTopFrontier`:
1. `PublishedZeroToCosSinPowerMajorantPack`
2. `publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack`
3. Provider closure:
   - `PublishedZeroToCosSinPowerMajorantProvider`
   - `rh_from_published_zero_to_cos_sin_power_majorant_provider`

### C) Pintz endpoint wiring
Added in `NearStrictTailToPintz`:
1. `rh_from_published_zero_to_cos_sin_power_majorant_pack_via_pintz`
2. `rh_from_published_zero_to_cos_sin_power_majorant_provider_via_pintz`

### D) Final equivalence lock extension
Added in `FinalTargetEquivalence`:
1. `rh_iff_nonempty_published_zero_to_cos_sin_power_majorant_provider`

## Why this matters
- It collapses route ambiguity: power-majorant and asymptotic strict-tail published boundaries now compose directly and consistently to the same RH endpoint layers.
- It keeps blocker count fixed: no new kernels were created; `K1-SOURCE` remains the single open item.

## Verification
- `lake build PrimeRiemannBridgeSpinningTopFrontier PrimeRiemannBridgeNearStrictTailToPintz PrimeRiemannBridgeFinalTargetEquivalence` ✅
- `python3 research/formal_axiom_audit.py ...` => `axiom_count = 0` ✅

## Status impact
- Remaining kernel unchanged: `K1-SOURCE` only.
- Reduction chain is sharper and easier to target for a final non-circular source theorem term insertion.
