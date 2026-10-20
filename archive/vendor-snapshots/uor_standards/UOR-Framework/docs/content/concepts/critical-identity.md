# Critical Identity

## Statement

The critical identity is the foundational theorem of the UOR ring substrate:

> **neg(bnot(x)) = succ(x)** for all x ∈ R_n

This states that the composition of the two canonical involutions of Z/(2^n)Z
equals the successor operation.

## Mathematical Proof

For x ∈ Z/(2^n)Z:
1. `bnot(x) = (2^n - 1) - x` (bitwise complement)
2. `neg(bnot(x)) = -(2^n - 1 - x) mod 2^n = x + 1 mod 2^n`
3. `succ(x) = x + 1 mod 2^n`

Therefore `neg(bnot(x)) = succ(x)`. ∎

## Ontology Representation

The critical identity is represented by the named individual
{@ind https://uor.foundation/op/criticalIdentity}:

```turtle
op:criticalIdentity
    a           owl:NamedIndividual, op:Identity ;
    op:lhs      op:succ ;
    op:rhs      op:neg ;
    op:forAll   "x ∈ R_n" .
```

Properties involved:
- {@prop https://uor.foundation/op/lhs}: the left-hand side of the identity
- {@prop https://uor.foundation/op/rhs}: the right-hand side
- {@prop https://uor.foundation/op/forAll}: quantification domain

## Proof Representation

The proof is captured by the class {@class https://uor.foundation/proof/CriticalIdentityProof},
a subclass of {@class https://uor.foundation/proof/Proof}.

The property {@prop https://uor.foundation/proof/provesIdentity} links a
`CriticalIdentityProof` to the `op:criticalIdentity` individual:

```turtle
<https://uor.foundation/instance/proof-critical-id>
    a                       proof:CriticalIdentityProof ;
    proof:provesIdentity    op:criticalIdentity ;
    proof:valid             true .
```

## Significance

The critical identity reveals:
1. **Successor is not primitive** — it is derived from the two involutions
2. **Dihedral structure** — neg and bnot generate the full dihedral group D_{2^n}
   captured by {@class https://uor.foundation/op/DihedralGroup}
3. **Universal computation** — any computable function on R_n can be expressed
   in terms of neg, bnot, and their compositions via {@prop https://uor.foundation/op/composedOf}

## Composition

The successor operation {@ind https://uor.foundation/op/succ} is defined as the
composition of neg and bnot in that order:

```turtle
op:succ  op:composedOf  ( op:neg  op:bnot ) .
```

The list preserves application order: bnot is applied first, then neg.
