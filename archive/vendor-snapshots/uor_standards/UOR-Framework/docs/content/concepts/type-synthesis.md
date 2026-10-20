# Type Synthesis

## Definition

**Type synthesis** is the process of running the ψ-pipeline in inverse mode:
given a target topological signature (Euler characteristic χ* and Betti profile
β\*), the {@class https://uor.foundation/resolver/TypeSynthesisResolver} searches
the space of constraint combinations and produces a
{@class https://uor.foundation/type/SynthesizedType} that achieves the target.

A {@class https://uor.foundation/type/TypeSynthesisGoal} carries the target
signature via {@prop https://uor.foundation/type/targetEulerCharacteristic}
(functional, one χ\* value) and
{@prop https://uor.foundation/type/targetBettiNumber} (non-functional, one
assertion per Betti degree).

## Output Classes

A successful synthesis run produces:

- {@class https://uor.foundation/type/TypeSynthesisResult} — the abstract result
  record linking goal to synthesized type.
- {@class https://uor.foundation/type/SynthesizedType} — a
  {@class https://uor.foundation/type/ConstrainedType} distinguished by a
  {@prop https://uor.foundation/type/synthesisResult} link.
- {@class https://uor.foundation/type/MinimalConstraintBasis} — the smallest set
  of constraints sufficient to realise the target signature, with
  {@prop https://uor.foundation/type/basisSize} giving the cardinality.

## Search State

The resolver maintains a
{@class https://uor.foundation/resolver/ConstraintSearchState} tracking explored
combinations ({@prop https://uor.foundation/resolver/exploredCount}) and the
current candidate ({@prop https://uor.foundation/resolver/currentCandidate}).
Each step in the construction is recorded as a
{@class https://uor.foundation/derivation/SynthesisStep} ordered by
{@prop https://uor.foundation/derivation/stepIndex}.

Intermediate topological signatures are captured in
{@class https://uor.foundation/observable/SynthesisSignature} individuals with
{@prop https://uor.foundation/observable/realisedEuler} and
{@prop https://uor.foundation/observable/realisedBetti} assertions.

## Identity Algebra (TS\_ series)

| Identity | Statement |
|----------|-----------|
| TS\_1 | ∃ SynthesizedType T realising target iff χ\* ≤ n and β₀\*=1, βₖ\*=0 k≥1 |
| TS\_2 | basisSize(T) = n for an n-site target |
| TS\_3 | χ(T + constraint) ≥ χ(T) for any synthesis candidate |
| TS\_4 | TypeSynthesisResolver takes ≤ n steps for a realisable n-site target |
| TS\_5 | SynthesizedType achieves IT\_7d iff CompletenessResolver certifies CompleteType |
| TS\_6 | Expected steps with Jacobian-guided search is O(n log n) |
| TS\_7 | β₀(non-empty ConstrainedType) ≥ 1 |

## Related

- {@class https://uor.foundation/type/TypeSynthesisGoal}
- {@class https://uor.foundation/resolver/TypeSynthesisResolver}
- {@class https://uor.foundation/type/SynthesizedType}
- [Type Completeness](type-completeness.html)
- [ψ Pipeline](../guides/psi-pipeline.html)
