//! SHACL test 226: `division:MultiplicationTable` instance.

/// Instance graph for Test 226: MultiplicationTable.
pub const TEST226_MULTIPLICATION_TABLE: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix division: <https://uor.foundation/division/> .

division:test_table a owl:NamedIndividual, division:MultiplicationTable .
"#;
