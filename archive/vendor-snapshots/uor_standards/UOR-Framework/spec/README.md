# uor-ontology

Internal crate encoding the complete [UOR Foundation](https://uor.foundation/)
ontology as typed Rust data structures, with serializers for JSON-LD, Turtle,
N-Triples, OWL RDF/XML, JSON Schema, SHACL, and EBNF.

**This crate is not published to crates.io** (`publish = false`). For the
published Rust trait library, see
[`uor-foundation`](https://crates.io/crates/uor-foundation).

## Contents

- 33 namespaces in dependency order
- 396 OWL classes
- 837 OWL properties (836 namespace-level + 1 global annotation)
- 1432 named individuals
- Seven serialization formats: JSON-LD 1.1, Turtle 1.1, N-Triples, OWL 2 RDF/XML, JSON Schema (Draft 2020-12), SHACL Shapes, EBNF

## Usage

```rust
use uor_ontology::{Ontology, iris};

let ontology = Ontology::full();
assert_eq!(ontology.namespaces.len(), 33);
assert_eq!(ontology.class_count(), 234);

// Look up a class by IRI
let address = ontology.find_class("https://uor.foundation/u/Address");
assert!(address.is_some());

// Serialize to JSON-LD (requires `serializers` feature, enabled by default)
let json_ld = uor_ontology::serializer::jsonld::to_json_ld(ontology);

// Serialize to Turtle
let turtle = uor_ontology::serializer::turtle::to_turtle(ontology);
```

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | yes | Adds `Serialize` derive to all model types |
| `serializers` | yes | All seven serializers (pulls in `serde_json`) |

## License

MIT — see [LICENSE](https://github.com/UOR-Foundation/UOR-Framework/blob/main/LICENSE).
