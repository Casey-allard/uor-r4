# Imported Asymptotic Theorem-Term Contract (2026-02-18)

## Required object
- Lean type:
  `PrimeRiemannBridgeAsymptoticImportedBoundary.ImportedAsymptoticSequenceTheoremTerm`

## Required field
- `zero_to_sequence_asymptotic`

This field must provide exactly the theorem term consumed by:
- `PrimeRiemannBridgeAsymptoticImportedBoundary.endpoint_to_rh_of_imported_asymptotic_theorem_term`

## Endpoint usage
Once a term `t : ImportedAsymptoticSequenceTheoremTerm` is available, RH endpoint follows via:
- `endpoint_to_rh_of_imported_asymptotic_theorem_term t`

## Citation lock behavior
Citation lock strings are fixed in-repo by:
- `assumptionsOfImportedTheoremTerm`
inside:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeAsymptoticImportedBoundary.lean`

No external caller needs to supply citation metadata; only the theorem term.
