# Type Synthesis Guide

Type synthesis runs the ψ-pipeline in inverse mode: you specify a desired
topological signature and the
{@class https://uor.foundation/resolver/TypeSynthesisResolver} constructs a
{@class https://uor.foundation/type/SynthesizedType} that achieves it.

## Step 1 — Declare a TypeSynthesisGoal

Create a {@class https://uor.foundation/type/TypeSynthesisGoal} individual with
the target signature:

```turtle
@prefix type:     <https://uor.foundation/type/> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .

type:my_goal a owl:NamedIndividual, type:TypeSynthesisGoal ;
    type:targetEulerCharacteristic  "4"^^xsd:integer ;
    type:targetBettiNumber          "1"^^xsd:nonNegativeInteger ;
    type:targetBettiNumber          "0"^^xsd:nonNegativeInteger .
```

The {@prop https://uor.foundation/type/targetEulerCharacteristic} is functional
(one χ\* value). {@prop https://uor.foundation/type/targetBettiNumber} is
non-functional — one assertion per Betti degree in ascending order.

## Step 2 — Attach a TypeSynthesisResolver

```turtle
@prefix resolver: <https://uor.foundation/resolver/> .

resolver:my_tsr a owl:NamedIndividual, resolver:TypeSynthesisResolver ;
    resolver:synthesisGoal  type:my_goal .
```

The resolver maintains a
{@class https://uor.foundation/resolver/ConstraintSearchState} internally,
recording {@prop https://uor.foundation/resolver/exploredCount} and
{@prop https://uor.foundation/resolver/currentCandidate} as it searches.

## Step 3 — Inspect the Result

On success the resolver produces:

1. A {@class https://uor.foundation/type/TypeSynthesisResult} linked from the
   {@class https://uor.foundation/type/SynthesizedType} via
   {@prop https://uor.foundation/type/synthesisResult}.
2. A {@class https://uor.foundation/type/MinimalConstraintBasis} with
   {@prop https://uor.foundation/type/basisSize} = n (the minimal constraint
   count).
3. A {@class https://uor.foundation/observable/SynthesisSignature} per
   {@class https://uor.foundation/derivation/SynthesisStep} showing the Euler
   and Betti progress (via `derivation:signatureBefore` / `derivation:signatureAfter`).

## Step 4 — Certify the Result

Feed the SynthesizedType into a
{@class https://uor.foundation/resolver/CompletenessResolver} to verify it
satisfies IT\_7d and obtain a
{@class https://uor.foundation/cert/CompletenessCertificate}. Identity TS\_5
guarantees the SynthesizedType achieves completeness iff the resolver certifies
it.

## Complexity

Identity TS\_4 bounds the number of synthesis steps at ≤ n for a realisable
n-site target. With Jacobian-guided exploration (TS\_6), the expected step
count is O(n log n).

## Related

- [Type Synthesis Concept](../concepts/type-synthesis.html)
- [Type Completeness](../concepts/type-completeness.html)
- [ψ Pipeline](psi-pipeline.html)
