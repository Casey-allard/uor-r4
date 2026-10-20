//! Shared standard RDF/OWL prefix table used by the Turtle, JSON-LD, and
//! SHACL serializers.
//!
//! Order matters for byte-stable output: Turtle's `@prefix` declarations
//! are emitted in this order, and the JSON-LD `@context` follows the same
//! ordering for determinism.

/// Standard prefix entries: (prefix, full IRI).
pub const STANDARD_PREFIXES: &[(&str, &str)] = &[
    ("owl", "http://www.w3.org/2002/07/owl#"),
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("sh", "http://www.w3.org/ns/shacl#"),
    ("uor", "https://uor.foundation/"),
];
