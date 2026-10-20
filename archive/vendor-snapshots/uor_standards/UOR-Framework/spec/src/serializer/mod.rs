//! Serializers for the UOR Foundation ontology.
//!
//! Eight serialization formats are supported:
//! - **EBNF** ([`ebnf`]) — the UOR Term Language grammar, output to `public/uor.term.ebnf`
//! - **Conformance EBNF** ([`conformance_ebnf`]) — the v0.2.1 conformance declaration grammar, output to `public/uor.conformance.ebnf`
//! - **JSON-LD** ([`jsonld`]) — the canonical format, output to `public/uor.foundation.jsonld`
//! - **JSON Schema** ([`json_schema`]) — type definitions, output to `public/uor.foundation.schema.json`
//! - **N-Triples** ([`ntriples`]) — for streaming/bulk processing, output to `public/uor.foundation.nt`
//! - **OWL RDF/XML** ([`owl_xml`]) — ontology interchange, output to `public/uor.foundation.owl`
//! - **SHACL** ([`shacl`]) — validation shapes, output to `public/uor.shapes.ttl`
//! - **Turtle** ([`turtle`]) — for RDF tooling, output to `public/uor.foundation.ttl`

pub mod conformance_ebnf;
pub mod ebnf;
pub mod json_schema;
pub mod jsonld;
pub mod ntriples;
pub mod owl_xml;
pub mod prefixes;
pub mod shacl;
pub mod turtle;
