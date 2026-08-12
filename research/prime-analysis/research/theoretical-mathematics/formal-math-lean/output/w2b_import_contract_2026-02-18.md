# W2b Import Contract (2026-02-18)

## Objective
Close the final open lemma `W2b` by supplying one concrete Lean instance of:
- `ImportedLinearPhaseOnlyResults.linear_phase_of_model_import`

## Exact mathematical payload required
For every admissible tuple
- `E : Real -> Real`
- `VonKochPrimeErrorCriterion E`
- right-half nontrivial zeta zero `s` with `(1/2) < Re(s)`
- decomposition parameters `A, beta, phase, R` with
  - `0 < A`, `(1/2) < beta`
  - `E x = A * x^beta * cos(phase x) + R x`

prove existence of constants `tau, phi` such that
- `0 < tau`
- `phase x = tau * log x + phi` for all `x > 0`.

## Current closure path once instance exists
- `PrimeRiemannBridgeW2bImportedInstance.rh_from_concrete_w2b`
- `PrimeRiemannBridgeW2bFinalSlot.rh_from_w2b_final_slot`
- `PrimeRiemannBridgeOscillatoryReduction.rh_from_imported_linear_phase_only_results_instance`
- `PrimeRiemannBridgeOscillatoryReduction.rh_from_imported_linear_phase_only_via_witness`
- `PrimeRiemannBridgeW2bImportedInstance.rh_from_pintz2017_via_concrete_w2b` (identifies W2b with the existing Pintz theorem-term slot)
- `PrimeRiemannBridgeW2bImportedInstance.rh_from_pintz2017_via_w2b_linear_phase_slot` (explicit `Pintz2017 -> ImportedLinearPhaseOnlyResults -> W2b` chain)
- `PrimeRiemannBridgeW2bImportedInstance.endpoint_to_rh_from_pintz2017_via_w2b_linear_phase_slot`
- `PrimeRiemannBridgeConcretePackInstantiation.rh_from_ingham1932_formalized`
- `PrimeRiemannBridgeW2bImportedInstance.rh_from_ingham1932_via_w2b_linear_phase_slot`
- `PrimeRiemannBridgeInghamImportedSlot.rh_from_ingham_imported_slot`
- `PrimeRiemannBridgeInghamImportedSlot.rh_from_ingham_imported_slot_via_w2b`
- `PrimeRiemannBridgeInghamImportedSlot.rh_from_asymptotic_imported_slot`
- `PrimeRiemannBridgeInghamImportedSlot.rh_from_asymptotic_imported_provider`

## Candidate source anchors
- Ingham (1932) boundary:
  - https://openlibrary.org/books/OL14108521M/The_distribution_of_prime_numbers
- Pintz-transfer anchor currently locked in the repo:
  - https://doi.org/10.1134/S0081543817010163
- Recent arXiv candidates for explicit-formula/error-term linkage:
  - https://arxiv.org/abs/2507.13780
  - https://arxiv.org/abs/2505.23795
- Formalization template reference (Isabelle AFP):
  - https://www.isa-afp.org/entries/Prime_Number_Theorem.html

## Selected primary source
- https://openlibrary.org/books/OL14108521M/The_distribution_of_prime_numbers
- Reason: this matches the final theorem payload signature directly and now maps to a dedicated Lean boundary class (`Ingham1932ZeroToOmegaFormalized`).

## Verification checklist
1. `~/.elan/bin/lake build PrimeRiemannBridgeW2bImportedInstance`
2. `~/.elan/bin/lake build`
3. `python3 research/formal_axiom_audit.py ...` reports `axiom_count = 0`
