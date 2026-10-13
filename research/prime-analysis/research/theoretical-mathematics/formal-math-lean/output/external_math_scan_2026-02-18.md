# External Math Scan (2026-02-18)

## Scope

Assess whether the final Lean slot can be closed by importing already-established formal RH content, and identify current research directions relevant to zero-oscillation / prime-error transfer.

## Primary Sources and Findings

1. Clay Mathematics Institute RH page ([link](https://www.claymath.org/millennium/riemann-hypothesis/))
- Status is explicitly listed as **Unsolved**.
- Clay reports large finite verification of zeros but no full proof.
- Implication: no accepted unconditional theorem can be imported as a completed RH proof.

2. mathlib4 docs for `Mathlib.NumberTheory.LSeries.RiemannZeta` ([link](https://leanprover-community.github.io/mathlib4_docs/Mathlib/NumberTheory/LSeries/RiemannZeta.html))
- `RiemannHypothesis` is present as a proposition definition.
- Docs note that constructing a term of this type is the unsolved target.
- Implication: mainline Lean mathlib does not provide an unconditional RH proof term.

3. Lean-RH repository README ([link](https://github.com/AlexKontorovich/Lean-RH))
- States a formalization of RH formulation while assuming certain analytic facts in standalone files.
- Implication: useful structure and proof engineering references, but not a drop-in unconditional closure for this repo.

4. Current arXiv feed (generated locally in `research/data/latest_math_arxiv_2026-02-18.md`)
- Recent topics include zeta zero moments, argument bounds, and critical-line-focused analyses.
- Implication: active progress exists on components, but no accepted complete RH proof artifact to import.

## Repo Frontier Mapping

- In-repo theorem `rh_iff_ingham_imported_payload` now states the final payload is equivalent to RH in this formalization.
- Therefore the remaining blocker is mathematical, not pipeline/integration.

## Practical Next Research Move

- Shift from "find drop-in proof" to "derive a new intermediate theorem strictly weaker than RH but stronger than current imports," then formalize that bridge to shrink the final gap with explicit non-circular assumptions.
