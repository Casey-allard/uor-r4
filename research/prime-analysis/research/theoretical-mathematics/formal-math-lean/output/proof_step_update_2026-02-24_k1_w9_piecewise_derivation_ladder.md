# Proof Step Update — 2026-02-24 (K1-W9 Piecewise Derivation Ladder)

## What changed

Updated `PrimeRiemannBridgeOscillatoryReduction.lean` to make the remaining finite-mode residual blocker explicitly derivable in smaller internal pieces:

1. Introduced a concrete split witness record:
   - `FiniteModeResidualPhaseSplitWitness`

2. Refactored the pieces assumptions so they are compositional and witness-based:
   - `ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions.phase_split_of_model` now returns a witness object.
   - `...residual_majorant_of_phase_split` now requires majorization only for the selected witness (not every arbitrary split).

3. Updated bridge assembly:
   - `finiteModeResidualMajorantAssumptionsOfPieces` now composes from the selected witness + its majorant.

4. Added internal derivation adapters (no external import required):
   - `finiteModeResidualMajorantPiecesAssumptionsOfFiniteDecayingCorrections`
   - `finiteModeResidualMajorantPiecesAssumptionsOfSingleDecayingCorrection`

5. Added provider instances for those adapters:
   - `concreteFiniteModeResidualMajorantPiecesProviderOfFiniteDecayingCorrections`
   - `concreteFiniteModeResidualMajorantPiecesProviderOfSingleDecayingCorrection`

6. Added RH endpoint wrappers through the pieces route:
   - `endpoint_to_rh_from_finite_decaying_phase_corrections_via_pieces_instance`
   - `rh_from_finite_decaying_phase_corrections_via_pieces_instance`
   - `endpoint_to_rh_from_single_decaying_phase_correction_via_pieces_instance`
   - `rh_from_single_decaying_phase_correction_via_pieces_instance`

## Build status

- `lake build PrimeRiemannBridgeOscillatoryReduction` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅
- `lake build` ✅

## Effect on the final blocker

The previous two open piece-fields have been normalized to a witness-first ladder that can be attacked sequentially.

Current minimal mathematical target remains analytic (non-circular): provide a concrete provider for one of:

- `ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions.finite_decaying_phase_correction_of_model`, or
- `ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions.single_decaying_phase_correction_of_model`.

Once one of these is instantiated non-circularly, the pieces/majorant chain to RH is already wired and compiling.
