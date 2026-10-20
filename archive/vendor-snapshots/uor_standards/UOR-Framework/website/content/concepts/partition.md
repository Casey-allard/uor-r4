# The Partition Decomposition

The {@ns partition} namespace decomposes the address space into disjoint subsets.
Every ring element is classified as irreducible, reducible, a unit, or exterior. This
four-way decomposition is the structural backbone of the Resolve stage in the
[PRISM](../pipeline/) pipeline.

## Four-Component Partition

The {@class https://uor.foundation/partition/Partition} class models a decomposition
of the universal address space into mutually exclusive subsets. Every element of the
ring Z/(2^n)Z falls into exactly one of four components:

- **Irreducible** -- elements that cannot be factored further under the dihedral group.
- **Reducible** -- elements that decompose into products of irreducibles.
- **Unit** -- invertible elements (those with a multiplicative inverse in the ring).
- **Exterior** -- elements outside the primary factorization domain.

This classification mirrors the role of prime factorization in the integers, lifted
to the modular ring setting.

## Products and Coproducts

The {@class https://uor.foundation/partition/PartitionProduct} class represents the
Cartesian product of two partitions -- the full joint space. The
{@class https://uor.foundation/partition/PartitionCoproduct} class represents their
disjoint union -- an exclusive choice between classification domains.

The {@class https://uor.foundation/partition/AncillaSite} class provides auxiliary
sites used in reversible computation strategies, where ancilla bits enable
information-preserving transformations.

## Exhaustive Coverage

The {@prop https://uor.foundation/partition/isExhaustive} property asserts that a
partition covers the entire space -- every element is classified, none are left out.
This is a formal completeness claim implemented in OWL, ensuring that the partition
is not merely partial but truly decomposes the whole ring.

## Free Rank Connection

The partition structure relates directly to the {@concept site} bundle semantics.
Each partition component corresponds to a family of sites over the address space.
The free rank -- the count of unconstrained sites remaining -- decreases as
resolution proceeds. When every site is pinned (assigned to a partition component),
resolution is complete.

## Connection to the PRISM Pipeline

Partition sits at the boundary between {@concept content-addressing} (Define) and
{@concept resolution} (Resolve). It decomposes the kernel-defined address space into
the structural categories that resolvers use to factorize elements. The four-component
classification feeds into the type system, where each component carries different
algebraic properties.
