# Partition

## Definition

A **partition** in the UOR framework is a decomposition of the ring R_n into
disjoint components. The class {@class https://uor.foundation/partition/Partition}
represents this decomposition.

## Four Components

Every partition of R_n has exactly four component sets:

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/partition/IrreducibleSet} | Elements with no non-trivial factorization |
| {@class https://uor.foundation/partition/ReducibleSet} | Elements that factor into smaller pieces |
| {@class https://uor.foundation/partition/UnitGroup} | Invertible elements (units of the ring) |
| {@class https://uor.foundation/partition/Complement} | Elements outside the kernel |

These four sets are mutually `owl:disjointWith` and their cardinalities sum to 2^n.

## Ontology Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| {@prop https://uor.foundation/partition/irreducibles} | Partition | IrreducibleSet | Link to irreducible set |
| {@prop https://uor.foundation/partition/reducibles} | Partition | ReducibleSet | Link to reducible set |
| {@prop https://uor.foundation/partition/units} | Partition | UnitGroup | Link to unit set |
| {@prop https://uor.foundation/partition/exterior} | Partition | Complement | Link to complement set |
| {@prop https://uor.foundation/partition/cardinality} | Component | xsd:nonNegativeInteger | Element count |
| {@prop https://uor.foundation/partition/density} | Component | xsd:string | Density as fraction |
| {@prop https://uor.foundation/partition/member} | Component | partition:Component | Member element |
| {@prop https://uor.foundation/partition/sourceType} | Partition | type:TypeDefinition | Source type |
| {@prop https://uor.foundation/partition/wittLength} | Partition | xsd:nonNegativeInteger | Ring Witt level |

## Example: R_4

For R_4 = Z/16Z (n=4, 16 elements):

```turtle
<https://uor.foundation/instance/partition-R4>
    a                   partition:Partition ;
    schema:ringWittLength "4"^^xsd:nonNegativeInteger ;
    partition:irreducibles  <...irred-set-R4> ;
    partition:reducibles    <...red-set-R4> ;
    partition:units         <...unit-set-R4> ;
    partition:exterior      <...ext-set-R4> .
```

## How Elements Are Classified

An element x ∈ R_n is classified by its factorization behavior:

- **Irreducible**: x has no non-trivial factorization — its only
  factorizations involve units. These are the "primes" of the ring.
- **Reducible**: x factors into two or more non-units. These elements
  decompose further under the dihedral factorization.
- **Unit**: x is invertible in R_n (i.e., gcd(x, 2^n) = 1). Units are
  exactly the odd elements.
- **Exterior**: x lies outside the multiplicative kernel — typically the
  zero element and nilpotents.

## Worked Example: R_4

For R_4 = Z/16Z with 16 elements (0–15):

| Component | Elements | {@prop https://uor.foundation/partition/cardinality} | {@prop https://uor.foundation/partition/density} |
|-----------|----------|-------------|---------|
| {@class https://uor.foundation/partition/UnitGroup} | {1, 3, 5, 7, 9, 11, 13, 15} | 8 | 1/2 |
| {@class https://uor.foundation/partition/IrreducibleSet} | {2} | 1 | 1/16 |
| {@class https://uor.foundation/partition/ReducibleSet} | {4, 6, 8, 10, 12, 14} | 6 | 3/8 |
| {@class https://uor.foundation/partition/Complement} | {0} | 1 | 1/16 |

The 8 units are exactly the odd numbers (invertible mod 16). The sole
irreducible is 2, the generator of the maximal ideal. All even non-zero
non-powers-of-two elements are reducible (e.g., 6 = 2 × 3). Zero is
exterior.

## Role in Resolution

The {@class https://uor.foundation/resolver/DihedralFactorizationResolver}
produces a `Partition` as its output. The partition is then used by:
- {@class https://uor.foundation/observable/Observable} to measure properties
- {@class https://uor.foundation/cert/Certificate} to certify correctness
- {@class https://uor.foundation/morphism/Transform} to apply transformations

See [Factorization](factorization.html) for the dihedral decomposition that
produces the partition.
