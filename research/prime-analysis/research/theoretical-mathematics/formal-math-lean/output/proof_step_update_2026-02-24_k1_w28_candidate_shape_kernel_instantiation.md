# Proof Step Update — 2026-02-24 (K1-W28 Candidate-Shape Kernel Instantiation)

## What was done
Completed the next immediate formal step on the L1 gate:

1. Derived a non-circular theorem term mapping a candidate-shape contract to the L1 kernel term.
2. Instantiated `R6DualBandTailRepresentationKernelProvider` from that candidate-shape provider contract.

## Lean changes
File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Added theorem contract and derivation:
- `R6DualBandTailRepresentationCandidateShapeTerm`
- `r6_dual_band_tail_representation_kernel_of_candidate_shape_term`

Added provider instantiation:
- `R6DualBandTailRepresentationCandidateShapeProvider`
- `r6DualBandTailRepresentationKernelProviderOfCandidateShape`
- `rh_from_r6_dual_band_tail_representation_candidate_shape_provider`

## Build verification
- `lake build PrimeRiemannBridgeSpinningTopFrontier` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅

## Interpretation
- The requested immediate step is now complete at the theorem-interface level:
  candidate-shape theorem term -> kernel provider -> RH endpoint path is formalized and compiled.
- Remaining open item is now strictly one payload object:
  a concrete non-circular instance of
  `R6DualBandTailRepresentationCandidateShapeProvider.theorem_term`.

## Next step
- Construct and prove (non-circular) a concrete theorem term for
  `R6DualBandTailRepresentationCandidateShapeTerm` from explicit-formula assumptions,
  then instantiate `R6DualBandTailRepresentationCandidateShapeProvider` directly.
