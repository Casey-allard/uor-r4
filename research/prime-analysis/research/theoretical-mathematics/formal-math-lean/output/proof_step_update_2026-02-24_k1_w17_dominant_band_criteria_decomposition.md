# Proof Step Update — 2026-02-24 (K1-W17 Dominant-Band Criteria Decomposition)

## Executed
Reduced the final gate by deriving anchor behavior and wiring a core-lock path to RH endpoint.

## Files changed
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`

## New formal math and wiring

### A) Anchor derivation theorem
In `PrimeRiemannBridgeOscillatoryReduction.lean`:
- `spinningTopR6AnchorOfTrivialCoreSuperposition`

This proves the anchor is forced (`τ = 1`, `φ = 0`) if trivial core equals the R6 superposition pointwise.

### B) Core-lock assumption layer
In `PrimeRiemannBridgeOscillatoryReduction.lean`:
- `ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions`
- `spinningTopR6DominantBandCriteriaAssumptionsOfCoreLock`
- `spinningTopR6DominantBandAssumptionsOfCoreLock`

So anchor is no longer an independent postulate on this route; it is theorem-derived from core-superposition identity.

### C) Provider endpoints
In `PrimeRiemannBridgeOscillatoryReduction.lean`:
- `ConcreteSpinningTopR6DominantBandCoreLockProvider`
- `concreteSpinningTopR6DominantBandCriteriaProviderOfCoreLock`
- `endpoint_to_rh_from_spinning_top_r6_dominant_band_core_lock_instance`
- `rh_from_spinning_top_r6_dominant_band_core_lock_instance`

### D) Imported-instance equivalence locks
In `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`:
- `rh_iff_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider`
- `nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized`

## Build status
- `lake build PrimeRiemannBridgeOscillatoryReduction` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅
- `lake build` ✅

## Quantitative screen (cached data, math-first)
- Added scan artifact:
  - `/Users/adminamn/Documents/New project/research/output/k1_w17_dominant_band_scan_grid_2026-02-24.json`
  - `/Users/adminamn/Documents/New project/research/output/k1_w17_dominant_band_scan_grid_2026-02-24.md`
- Best observed row in tested grid:
  - `beta=0.65`, `x∈[1e7,1e20]`
  - `offdiag_l1/dom = 1.243234`
  - `tail_sup/amp_total = 1.479961`
- Interpretation: still outside strong dominant-band regime (`<< 1` not met), so current numerical evidence does not yet support immediate unconditional collapse.

## Remaining open math (same endpoint, fewer independent assumptions)
The unresolved content is now concentrated in:
1. `trivial_core_superposition_of_model`
2. `dominant_band_collapse_of_model`

Closing these non-circularly closes the final dominant-band gate by theorem.
