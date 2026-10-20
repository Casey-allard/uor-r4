# Type System

## Definition

The UOR type system provides a structured way to classify objects in the ring.
The base class is {@class https://uor.foundation/type/TypeDefinition}.

## Type Hierarchy

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/type/TypeDefinition} | Base type |
| {@class https://uor.foundation/type/PrimitiveType} | Atomic type (e.g., u8, u16) |
| {@class https://uor.foundation/type/ProductType} | Cartesian product of types |
| {@class https://uor.foundation/type/SumType} | Discriminated union of types |
| {@class https://uor.foundation/type/ConstrainedType} | Type with additional constraints |

## Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| {@prop https://uor.foundation/type/bitWidth} | PrimitiveType | xsd:nonNegativeInteger | Bit width |
| {@prop https://uor.foundation/type/component} | ProductType | TypeDefinition | Component types |
| {@prop https://uor.foundation/type/baseType} | SumType/ConstrainedType | TypeDefinition | Base type |
| {@prop https://uor.foundation/type/contentAddress} | TypeDefinition | u:Element | Content address |

## Example: Primitive Types

```turtle
<https://uor.foundation/instance/type-u8>
    a               type:PrimitiveType ;
    type:bitWidth   "8"^^xsd:nonNegativeInteger .

<https://uor.foundation/instance/type-u64>
    a               type:PrimitiveType ;
    type:bitWidth   "64"^^xsd:nonNegativeInteger .
```

## Integration with State

Types are used in {@class https://uor.foundation/state/Binding} to record the
type of bound values:

```turtle
<https://uor.foundation/instance/binding-x>
    a               state:Binding ;
    state:boundType <https://uor.foundation/instance/type-u8> .
```

## Integration with Partition

Types serve as the source for partition computations via
{@prop https://uor.foundation/partition/sourceType}.

## Constrained Types

A {@class https://uor.foundation/type/ConstrainedType} wraps a base type with
additional constraints that restrict the set of valid values. Constraints
are combined using {@class https://uor.foundation/type/Conjunction}:

```turtle
<https://uor.foundation/instance/constrained-odd-shallow>
    a                   type:ConstrainedType ;
    type:baseType       <https://uor.foundation/instance/type-u8> ;
    type:constraint     "residue(2,1) AND depth(1,2)" .
```

This constrained type accepts only values in Z/256Z that are odd (residue 1
mod 2) and have valuation depth between 1 and 2. Each constraint pins
specific sites of the Z/2Z fibration:

- {@ind https://uor.foundation/type/residueConstraintKind} (mod 2, residue 1)
  pins site 0.
- {@ind https://uor.foundation/type/depthConstraintKind} (depth 1–2) pins
  sites 0 and 1.

## Types and Free Rank

Each constraint pins sites, and the type's total free rank tracks how
many sites remain free. A fully resolved type has zero free sites — every
site is pinned by at least one constraint. See
[Free Rank](free-rank.html) for the accounting and
[Constraint Algebra](constraint-algebra.html) for how constraints compose.
