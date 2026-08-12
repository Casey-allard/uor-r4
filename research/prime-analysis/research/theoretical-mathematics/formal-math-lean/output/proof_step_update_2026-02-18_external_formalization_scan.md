# Proof Step Update: External Formalization Scan (2026-02-18)

## Goal

Check whether the remaining final-slot theorem can be imported as an existing Lean4 proof artifact.

## Findings

1. Mathlib currently provides a **definition** of `RiemannHypothesis` (in `Mathlib.NumberTheory.LSeries.RiemannZeta`) rather than a completed proof term.
2. The `AlexKontorovich/Lean-RH` repository presents a Lean formalization and explicitly notes assumptions around real/complex log/exp in its standalone files.
3. In current repo state, no directly importable Lean4 theorem term was located that can instantiate:
   - `PublishedZeroOscillationPack.right_half_zero_forces_lower_envelope`

## Practical conclusion

- The in-repo final blocker remains unchanged: one concrete theorem term for the Pintz-style zero->oscillation transfer.
- The repository is now prepared so that once this theorem term is available, `RHStatement` closes immediately via `rh_from_pintz2017_formalized`.

## Sources

- https://leanprover-community.github.io/mathlib4_docs/Mathlib/NumberTheory/LSeries/RiemannZeta.html
- https://github.com/AlexKontorovich/Lean-RH
