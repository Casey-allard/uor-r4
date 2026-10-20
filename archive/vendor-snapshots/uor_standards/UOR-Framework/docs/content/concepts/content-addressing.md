# Content Addressing

## Definition

**Content addressing** is the principle that an object's identity is determined by
its content, not by an external location or name. In the UOR framework, this is
formalized through the {@class https://uor.foundation/u/Element} class.

## Mathematical Basis

A content address is a canonical representation of an object derived from its bytes.
The UOR framework uses the ring structure (see [Ring](ring.html)) to define a
**canonical form** — a unique representative for each equivalence class of objects.

The {@class https://uor.foundation/resolver/CanonicalFormResolver} computes this
canonical form by factorizing an object's representation in the dihedral group
D_{2^n}.

## Ontology Representation

The addressing namespace `u/` provides two foundational classes:

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/u/Element} | Universal content address |

Properties in the `u/` namespace:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/u/wittLength} | The Witt level n of this element |
| {@prop https://uor.foundation/u/length} | Length in bytes |
| {@prop https://uor.foundation/u/digest} | Content hash (algorithm-prefixed) |
| {@prop https://uor.foundation/u/digestAlgorithm} | Hash algorithm ('blake3' or 'sha256') |
| {@prop https://uor.foundation/u/canonicalBytes} | Canonical byte pre-image for hashing |

## Cryptographic Primitive Pinning (Amendment 43)

The {@prop https://uor.foundation/u/digest} property is pinned to two allowed hash algorithms:
BLAKE3 (primary) and SHA-256 (secondary). The digest value is prefixed with the algorithm
identifier and a colon, e.g. `blake3:af13...` or `sha256:e3b0...`.

The hash input is a deterministic byte string stored in
{@prop https://uor.foundation/u/canonicalBytes}. This canonical form consists of a 4-byte
header (magic `UR` + Witt level + reserved byte) followed by the datum value in
little-endian byte order. At Witt level k, the total length is `4 + (k + 1)` bytes.

The {@prop https://uor.foundation/u/digestAlgorithm} property records which algorithm was
used, ensuring that the algorithm prefix in the digest can be cross-checked.

## Resolution

Given a content address, the {@class https://uor.foundation/resolver/Resolver}
hierarchy performs resolution:

1. **DihedralFactorizationResolver** — factorizes in D_{2^n}
2. **CanonicalFormResolver** — computes the canonical form
3. **EvaluationResolver** — evaluates the canonical form

The result is a {@class https://uor.foundation/partition/Partition} decomposing
the address into irreducible, reducible, unit, and exterior components.

## Schema Integration

The {@class https://uor.foundation/schema/Datum} class represents raw byte content,
while {@class https://uor.foundation/schema/Term} represents symbolic content.
These two are `owl:disjointWith` — a datum and a term are fundamentally different
kinds of things.

A {@class https://uor.foundation/schema/Literal} (a subclass of `Term`) can
**denote** a `Datum` via the {@prop https://uor.foundation/schema/denotes} property,
bridging the symbolic and data layers without conflating them.
