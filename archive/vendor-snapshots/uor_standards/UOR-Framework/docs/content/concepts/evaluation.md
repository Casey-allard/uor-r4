# Evaluation

## Definition

**Evaluation** is the process of computing concrete results from canonical
forms. The {@class https://uor.foundation/resolver/EvaluationResolver}
implements this process: it takes a resolved type and evaluates it by
applying operations to enumerate and classify ring elements.

## The Resolver

The {@class https://uor.foundation/resolver/EvaluationResolver} is the third
and final stage of the resolution pipeline, following factorization and
canonical form computation:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/resolver/inputType} | The type declaration to evaluate |
| {@prop https://uor.foundation/resolver/outputType} | The resulting partition |
| {@prop https://uor.foundation/resolver/strategy} | Direct enumeration and classification |
| {@prop https://uor.foundation/resolver/hasComplexityClass} | Computational complexity |

## Evaluation Process

1. Accept a {@class https://uor.foundation/type/TypeDefinition} as input
2. Apply the type's constraints ({@prop https://uor.foundation/type/hasConstraint})
   to the ring elements
3. Classify each element into the four partition components:
   {@class https://uor.foundation/partition/IrreducibleSet},
   {@class https://uor.foundation/partition/ReducibleSet},
   {@class https://uor.foundation/partition/UnitGroup}, or
   {@class https://uor.foundation/partition/Complement}
4. Produce the final {@class https://uor.foundation/partition/Partition}

## Boolean SAT Case Study

Evaluation provides a direct path to Boolean satisfiability. Given a
{@class https://uor.foundation/type/ConstrainedType} with Boolean constraints,
the evaluation resolver enumerates satisfying assignments:

- Each {@class https://uor.foundation/partition/SiteIndex} corresponds to
  a Boolean variable
- A {@ind https://uor.foundation/type/residueConstraintKind} with modulus 2
  fixes a variable's value
- A {@class https://uor.foundation/type/Conjunction} represents a clause
  over multiple variables
- The {@class https://uor.foundation/partition/IrreducibleSet} of the resulting
  partition contains the satisfying assignments

The {@class https://uor.foundation/partition/FreeRank} tracks how many
variables have been determined. When
{@prop https://uor.foundation/partition/isClosed} is true, all variables are
fixed and the formula is either satisfied or unsatisfiable.

## Derivation Output

Evaluation produces a {@class https://uor.foundation/derivation/Derivation}
with a fully evaluated {@prop https://uor.foundation/derivation/result}:

```turtle
<https://uor.foundation/instance/eval-derivation>
    a                       derivation:Derivation ;
    derivation:originalTerm <term-constrained-bool> ;
    derivation:canonicalTerm <term-canonical-bool> ;
    derivation:result       <datum-satisfying-assignment> .
```

## Certification

The evaluation result can be certified by a
{@class https://uor.foundation/cert/Certificate}, attesting that the partition
was computed correctly and the classification of every element is sound.

## Pipeline Position

Evaluation is the final resolver in the three-stage pipeline:
1. {@class https://uor.foundation/resolver/DihedralFactorizationResolver} -- orbit decomposition
2. {@class https://uor.foundation/resolver/CanonicalFormResolver} -- term normalization
3. **EvaluationResolver** -- concrete classification

Evaluation triggers state transitions: each classification step updates the
{@class https://uor.foundation/state/Frame} and produces a
{@class https://uor.foundation/state/Transition} — see
[State Model](state-model.html) for the context/binding/frame lifecycle.
