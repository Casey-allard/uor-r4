# RDF 1.1 and Turtle 1.1 Standards

## RDF 1.1

The ontology is a valid RDF 1.1 graph. Requirements:

- All IRIs are absolute (no relative IRIs in the serialized graph).
- Language tags conform to BCP 47.
- Typed literals use XSD or known datatypes.
- The graph is acyclic with respect to `rdf:type` chains (no instance is its own class).

## Turtle 1.1

The `public/uor.foundation.ttl` file must conform to Turtle 1.1:

- File begins with `@prefix` declarations for all 33 namespace prefixes + standard prefixes.
- All triples end with ` .`
- Predicate-object lists use `;` as separator.
- `rdf:List` for ordered collections uses `( elem1 elem2 )` syntax.
- Blank nodes use `_:` or `[]` syntax.

### Required Prefixes

```turtle
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:       <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix sh:         <http://www.w3.org/ns/shacl#> .
@prefix uor:        <https://uor.foundation/> .
@prefix u:          <https://uor.foundation/u/> .
@prefix schema:     <https://uor.foundation/schema/> .
@prefix op:         <https://uor.foundation/op/> .
@prefix query:      <https://uor.foundation/query/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix homology:   <https://uor.foundation/homology/> .
@prefix cohomology: <https://uor.foundation/cohomology/> .
@prefix proof:      <https://uor.foundation/proof/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix morphism:   <https://uor.foundation/morphism/> .
@prefix state:      <https://uor.foundation/state/> .
```

## N-Triples

The `public/uor.foundation.nt` file must conform to N-Triples 1.1:

- One triple per line.
- Each line ends with ` .`
- Subject and predicate are absolute IRIs enclosed in `< >`.
- Objects are IRIs, blank nodes, or typed/plain literals.
- No prefix declarations (N-Triples uses full IRIs only).

## References

- [RDF 1.1 Concepts](https://www.w3.org/TR/rdf11-concepts/)
- [Turtle 1.1 W3C Specification](https://www.w3.org/TR/turtle/)
- [N-Triples W3C Specification](https://www.w3.org/TR/n-triples/)
