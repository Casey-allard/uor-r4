# W2 Contract: Linear-Phase Witness (2026-02-18)

## Open lemma

`W2` remains the only open lemma in the RH witness route.

Lean target interface:
- `ImportedLinearPhaseWitnessStepResults` in
  `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`

## Required sub-lemmas

1. `W2a` (`zero_to_cos_sin_phase`):
   For each right-half zero, derive
   \(E(x)=x^\beta(a\cos(\tau\log x)+b\sin(\tau\log x))+R(x)\)
   with \(\beta>1/2\), \(\tau>0\), nontrivial amplitude `(a,b)`, and
   \(R(x)/x^\beta \to 0\).

2. `W2b` (`cos_sin_to_single_cos`):
   Convert nontrivial `a cos t + b sin t` into
   `A cos(t + φ)` with `A > 0`.

## Composition already implemented

- `importedLinearPhaseWitnessResultsOfStepResults` composes `W2a + W2b`
  into the final witness interface:
  `ImportedLinearPhaseWitnessResults`.

## Closure consequence

After `W2` is instantiated, RH closes directly through:
- `rh_from_imported_linear_phase_witness_instance`
