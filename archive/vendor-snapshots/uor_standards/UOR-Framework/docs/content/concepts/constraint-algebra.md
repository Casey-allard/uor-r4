# Constraint Algebra

## Definition

The **constraint algebra** provides composable predicates that refine types by
pinning site indices. A {@class https://uor.foundation/type/Constraint}
is a predicate that, when applied to a type, determines the value of one or
more sites in the iterated Z/2Z fibration.

## Constraint Hierarchy

Four concrete constraint kinds are provided, mutually disjoint:

| Class | Description |
|-------|-------------|
| {@ind https://uor.foundation/type/residueConstraintKind} | Membership in a residue class: x = r (mod m) |
| {@ind https://uor.foundation/type/carryConstraintKind} | Carry propagation pattern in ring arithmetic |
| {@ind https://uor.foundation/type/depthConstraintKind} | Bounds on factorization depth |
| {@class https://uor.foundation/type/Conjunction} | Composition of two or more simpler constraints |

A {@class https://uor.foundation/type/ConstrainedType} links to its constraints via
{@prop https://uor.foundation/type/hasConstraint}.

## Constraint Properties

v0.2.2 Phase D parametrized the constraint surface: instead of seven
disjoint subclasses, every constraint is now a `BoundConstraint<O, B>`
selecting one (`Observable`, `BoundShape`) pair from the closed catalogue.
The legacy names survive as Rust type aliases (`ResidueConstraint`,
`HammingConstraint`, ...) over the parametric carrier.

| Constraint kind | Observable | Bound shape |
|---|---|---|
| `ResidueConstraint` | `observable:ValueModObservable` | `type:ResidueClassBound` |
| `HammingConstraint` | `observable:HammingMetric` | `type:LessEqBound` |
| `DepthConstraint` | `derivation:DerivationDepthObservable` | `type:LessEqBound` |
| `CarryConstraint` | `carry:CarryDepthObservable` | `type:LessEqBound` |
| `SiteConstraint` | `partition:FreeRankObservable` | `type:LessEqBound` |
| `AffineConstraint` | `observable:ValueModObservable` | `type:AffineEqualBound` |

Each kind's parameters are passed via `BoundArguments`. The legacy
property triples are retained on `BoundConstraint` for ergonomic access
(modulus, residue, hammingBound, minDepth, maxDepth, etc.):

| Property | Stored on | Range | Description |
|----------|-----------|-------|-------------|
| {@prop https://uor.foundation/type/modulus} | BoundConstraint | xsd:positiveInteger | The modulus m (residue / affine kinds) |
| {@prop https://uor.foundation/type/residue} | BoundConstraint | xsd:nonNegativeInteger | The residue r |
| {@prop https://uor.foundation/type/carryPattern} | BoundConstraint | xsd:string | Binary carry pattern |
| {@prop https://uor.foundation/type/minDepth} | BoundConstraint | xsd:nonNegativeInteger | Minimum depth |
| {@prop https://uor.foundation/type/maxDepth} | BoundConstraint | xsd:nonNegativeInteger | Maximum depth |
| {@prop https://uor.foundation/type/composedFrom} | Conjunction | BoundConstraint | Component constraints |

## Metric Axes

Every constraint operates along a {@class https://uor.foundation/type/MetricAxis},
classified by its geometric effect. The three axes form the tri-metric coordinate
system of UOR:

| Individual | Description |
|------------|-------------|
| {@ind https://uor.foundation/type/verticalAxis} | Ring/additive: residue classes, divisibility |
| {@ind https://uor.foundation/type/horizontalAxis} | Hamming/bitwise: bit positions, carry patterns |
| {@ind https://uor.foundation/type/diagonalAxis} | Incompatibility: the gap between ring and Hamming |

The property {@prop https://uor.foundation/type/metricAxis} assigns each constraint
to its axis. The property {@prop https://uor.foundation/type/crossingCost} records
how many axis boundaries a constraint must traverse.

## Site Pinning

The property {@prop https://uor.foundation/type/pinsSites} declares which
{@class https://uor.foundation/partition/SiteIndex} instances a constraint
pins when applied. A {@class https://uor.foundation/type/Conjunction}
pins the union of sites pinned by its components.

## Example: Residue + Depth

In v0.2.2 Phase D, both kinds are `BoundConstraint` instances. The Rust
call-site syntax stays compatible via the type aliases:

```rust
use uor_foundation::enforcement::{ResidueConstraint, DepthConstraint};
let odd = ResidueConstraint::new(2, 1);
let shallow = DepthConstraint::new(0, 2);
```

The Turtle representation uses the parametric form:

```turtle
<https://uor.foundation/instance/constraint-odd>
    a                   type:BoundConstraint ;
    type:boundObservable observable:ValueModObservable ;
    type:boundShape      type:ResidueClassBound ;
    type:boundArguments  "modulus=2;residue=1" ;
    type:metricAxis      type:verticalAxis .

<https://uor.foundation/instance/constraint-shallow>
    a                   type:BoundConstraint ;
    type:boundObservable derivation:DerivationDepthObservable ;
    type:boundShape      type:LessEqBound ;
    type:boundArguments  "min_depth=0;max_depth=2" ;
    type:metricAxis      type:verticalAxis .
```

Each constraint pins specific sites tracked by the
{@class https://uor.foundation/partition/FreeRank} — see
[Free Rank](free-rank.html) for how pinned sites accumulate toward
resolution closure.
