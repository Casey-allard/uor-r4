# Repo-Derived RH Elimination Program

Date: 2026-02-18

## Goal

Eliminate all RH-specific axioms/import assumptions and replace them with derived proof terms inside this repository.

## Current boundary (post-audit)

- No RH-specific axioms remain in audited active Lean files.
- The asymptotic theorem-term boundary is now derivable from the existing published pack input:
  `PublishedZeroOscillationPack -> ImportedAsymptoticSequenceTheoremTerm`.
- The remaining imported boundary is a concrete `PublishedZeroOscillationPack` term
  (or `ImportedPublishedResults` instance) for final instantiation.

## Sequential elimination steps

1. Provide a concrete `PublishedZeroOscillationPack` term (or `ImportedPublishedResults` instance).
2. Instantiate endpoint theorem via either path:
   `endpoint_to_rh_of_imported_published_pack` or
   `endpoint_to_rh_from_published_pack_via_provider`.
3. Keep canonical manifest + status/audit artifacts synchronized after each instantiation attempt.

## Completion criterion

- `formal_axiom_audit.py` reports `axiom_count = 0` on active Lean files.
- A concrete `PublishedZeroOscillationPack` term is supplied,
  enabling a fully instantiated endpoint=>RH theorem inside the repo.
