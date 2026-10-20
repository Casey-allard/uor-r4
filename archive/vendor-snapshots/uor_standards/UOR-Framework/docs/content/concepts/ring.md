# Ring

## Definition

A **ring** in the UOR framework is the algebraic substrate Z/(2^n)Z — integers
modulo 2^n. The ontology class {@class https://uor.foundation/schema/Ring}
represents this structure.

## Mathematical Basis

For Witt level n, the ring R_n = Z/(2^n)Z has 2^n elements: {0, 1, ..., 2^n - 1}.

The ring is equipped with:
- Standard addition and multiplication (mod 2^n)
- Two canonical **involutions** (self-inverse operations):
  - {@ind https://uor.foundation/op/neg}: additive negation, `neg(x) = 2^n - x mod 2^n`
  - {@ind https://uor.foundation/op/bnot}: bitwise complement, `bnot(x) = 2^n - 1 - x`

These two involutions generate the dihedral group D_{2^n}, captured by
{@class https://uor.foundation/op/DihedralGroup}.

## Ontology Representation

`schema:Ring` has the following properties:

| Property | Type | Description |
|----------|------|-------------|
| {@prop https://uor.foundation/schema/ringWittLength} | xsd:nonNegativeInteger | The Witt level n |
| {@prop https://uor.foundation/schema/modulus} | xsd:nonNegativeInteger | 2^n |
| {@prop https://uor.foundation/schema/generator} | op:Operation | The primary generator (op:neg) |
| {@prop https://uor.foundation/schema/negation} | op:Operation | Ring negation (op:neg) |
| {@prop https://uor.foundation/schema/complement} | op:Operation | Bitwise complement (op:bnot) |

## Example: The Byte Ring

For n=8, R_8 = Z/256Z is the **byte ring** — the algebraic structure underlying
every byte of digital information:

```turtle
<https://uor.foundation/instance/ring-R8>
    a                   schema:Ring ;
    schema:ringWittLength "8"^^xsd:nonNegativeInteger ;
    schema:modulus      "256"^^xsd:nonNegativeInteger ;
    schema:generator    op:neg ;
    schema:negation     op:neg ;
    schema:complement   op:bnot .
```

## The Critical Identity

The ring's two involutions satisfy the critical identity
(see {@class https://uor.foundation/proof/CriticalIdentityProof}):

```
neg(bnot(x)) = succ(x)  for all x ∈ R_n
```

This means the successor operation {@ind https://uor.foundation/op/succ}
is the composition of negation and complement:
{@prop https://uor.foundation/op/composedOf} `[op:neg, op:bnot]`.
