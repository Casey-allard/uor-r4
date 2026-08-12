# Final Pack Instantiation Contract (2026-02-18)

## Objective

Provide one concrete theorem term that inhabits:

- `PublishedZeroOscillationPack`

in:

- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeConcretePackInstantiation.lean`

## Exact Proof Field Required

The required nontrivial field is:

```lean
right_half_zero_forces_lower_envelope :
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β : Real, (1 / 2 : Real) < β ∧
          (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)
```

The metadata fields must lock to:

- `source_tag = "PINTZ-2017-OSCILLATION"`
- `source_url = "https://doi.org/10.1134/S0081543817010163"`
- `theorem_ref = "Thm-2-zero-to-oscillation-transfer"`

## Fastest Integration Paths

1. Provide an `ImportedPublishedResults` instance and use existing bridge:
   - `concreteProviderOfImportedResults`
2. Provide an `ImportedZeroOscillationResults` instance and use:
   - `publishedPackOfImportedZeroOscillation`
3. Provide an `ImportedAnalyticBridgeResults` instance and use:
   - `importedZeroOscillationOfAnalyticBridge`
4. Provide an `ImportedLinearPhaseOnlyResults` instance and use:
   - `endpoint_to_rh_from_imported_linear_phase_only_instance`
   - `rh_from_imported_linear_phase_only_instance`
5. Provide an `ImportedLinearPhaseWitnessResults` instance and use:
   - `endpoint_to_rh_from_imported_linear_phase_witness_instance`
   - `rh_from_imported_linear_phase_witness_instance`

## Reduced Mathematical Obligation

After deriving `zero_to_global_decomposition_of_vonkoch` in
`PrimeRiemannBridgeOscillatoryReduction.lean`, the remaining external theorem
term in the oscillatory route is only:

- `linear_phase_of_model` (encapsulated by `ImportedLinearPhaseOnlyResults`)
- or equivalently one explicit linear-phase witness with vanishing normalized remainder
  (encapsulated by `ImportedLinearPhaseWitnessResults`)

## Acceptance Check

- `lake build PrimeRiemannBridgeConcretePackInstantiation`
- `lake build`
- `python3 /Users/adminamn/Documents/New\ project/research/formal_axiom_audit.py --lean-files /Users/adminamn/Documents/New\ project/research/formal/lean/*.lean --output-json /Users/adminamn/Documents/New\ project/research/output/formal_axiom_audit_2026-02-18.json --output-md /Users/adminamn/Documents/New\ project/research/output/formal_axiom_audit_2026-02-18.md`
