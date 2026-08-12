# Lean Formalization Scaffold

This folder contains the machine-checking scaffold for the O1-O5 chain.

## Files

- `PrimeRiemannBridge.lean`: theorem spine with compiled O1-O5/L1-L3 chain and no axioms.
- `../../output/proof_constant_provenance_2026-02-17.json`: constant source map to mirror in Lean hypotheses.

## How to use next

1. Migrate from rational surrogate bounds to Real/log/sqrt/floor/zeta-zero statements (mathlib-backed).
2. Replace contract-level transfer assumptions with fully derived lemmas from O1/O2/O3/O4 theorem packs.
3. Encode citation-locked external theorem statements as formal imports/references.
4. Upgrade endpoint implication from project proxy class to formal RH-equivalent statement.

## Current Frontier (2026-02-24)

- Active bridge file: `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`
- Active source contract: `R6DualBandTailRepresentationCandidateShapeTerm`
- Latest checkpoint: `/Users/adminamn/Documents/New project/research/output/proof_resume_checkpoint_2026-02-24_k1_w28_candidate_shape_kernel_instantiation.json`
- Remaining math gate: construct a concrete non-circular theorem term for `R6DualBandTailRepresentationCandidateShapeProvider.theorem_term`.
