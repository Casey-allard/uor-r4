//! Ontology artifact validators (EBNF, JSON-LD, JSON Schema, OWL, OWL XML, RDF, SHACL, inventory, generated crate).

pub mod crate_;
pub mod ebnf;
pub mod enforcement;
pub mod inventory;
pub mod json_schema;
pub mod jsonld;
pub mod owl;
pub mod owl_xml;
pub mod rdf;
pub mod required_property_coverage;
pub mod shacl;
pub mod shacl_shapes;
pub mod standards;
