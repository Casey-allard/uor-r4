# Composition

## Definition

**Composition** is the categorical backbone of the UOR transform system. It turns
the collection of transforms into a category with identity morphisms and
associative composition. The class {@class https://uor.foundation/morphism/Composition}
represents a transform formed by sequentially applying two or more transforms.

## Composition Primitive

A {@class https://uor.foundation/morphism/Composition} is a subclass of
{@class https://uor.foundation/morphism/Transform}. Its component transforms
are declared via {@prop https://uor.foundation/morphism/compositionComponents},
and the resulting transform is given by
{@prop https://uor.foundation/morphism/compositionResult}:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/morphism/compositionComponents} | Transform | Component transforms |
| {@prop https://uor.foundation/morphism/compositionResult} | Transform | Resulting transform |
| {@prop https://uor.foundation/morphism/compositionOrder} | xsd:nonNegativeInteger | Number of components |

The property {@prop https://uor.foundation/morphism/composesWith} declares
composability between transforms: the target of the first must match the source
of the second.

## Identity

The class {@class https://uor.foundation/morphism/Identity} represents the
identity transform on a type. The property
{@prop https://uor.foundation/morphism/identityOn} declares which type
the identity acts on. Composing any transform with its identity is a no-op.

## Composition Laws

A {@class https://uor.foundation/morphism/CompositionLaw} records how
specific operations compose:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/morphism/lawComponents} | Operation | Operations being composed |
| {@prop https://uor.foundation/morphism/lawResult} | Operation | The resulting operation |
| {@prop https://uor.foundation/morphism/isAssociative} | xsd:boolean | Whether the law is associative |
| {@prop https://uor.foundation/morphism/isCommutative} | xsd:boolean | Whether the law is commutative |

## The Critical Composition

The named individual {@ind https://uor.foundation/morphism/criticalComposition}
encodes the operational form of the critical identity theorem:

> **neg compose bnot = succ**

This law is non-associative and non-commutative: the order of composition
matters, and the law does not extend to arbitrary triples.

```turtle
morphism:criticalComposition
    a                           morphism:CompositionLaw ;
    morphism:lawComponents      op:neg, op:bnot ;
    morphism:lawResult          op:succ ;
    morphism:isAssociative      false ;
    morphism:isCommutative      false .
```

## Structure Preservation

The property {@prop https://uor.foundation/morphism/preservedInvariant} records
which identity a transform preserves (linking to the op:Identity that the
transform commutes with). This connects composition to the certification layer: a
{@class https://uor.foundation/morphism/Isometry} preserves metric structure
via {@prop https://uor.foundation/morphism/preservesMetric}.

The critical composition `neg ∘ bnot = succ` is the rewrite rule that drives
the {@class https://uor.foundation/resolver/DihedralFactorizationResolver} —
see [Factorization](factorization.html) for how this law decomposes ring
elements under the dihedral group action.
