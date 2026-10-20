# About UOR Foundation

The UOR Foundation develops the Universal Object Reference specification —
a formal ontology for content-addressed, symmetric, multi-metric object spaces.

## Mission

To provide a rigorous, open, machine-verifiable specification for the algebraic
structure of digital information, enabling interoperability across systems that
represent, resolve, and transform computable objects.

## What Is UOR?

UOR is a mathematical framework built on ring theory. Every computable object
lives in the ring Z/(2^n)Z — integers modulo a power of 2. The framework
specifies how objects are uniquely identified (content-addressed), decomposed
(factorized under the dihedral group), and verified (certified by algebraic proofs).

The framework is encoded as an OWL 2 DL ontology with {@count:namespaces} namespaces,
{@count:classes} classes, {@count:properties} properties, and {@count:individuals}
named individuals. It is published in 7 serialization formats
and as a Rust trait crate (`uor-foundation` on crates.io).

## Conformance-First Design

Every claim this website makes is validated by the conformance suite — a set of
automated checks that verify the ontology's structure, the website's content, and
the generated artifacts. The website itself is generated directly from the ontology
source, ensuring perfect alignment between specification and documentation.

## License

All UOR Foundation specifications and implementations are published under the
MIT license.

## Repository

The canonical implementation is at
[github.com/UOR-Foundation/UOR-Framework](https://github.com/UOR-Foundation/UOR-Framework).
