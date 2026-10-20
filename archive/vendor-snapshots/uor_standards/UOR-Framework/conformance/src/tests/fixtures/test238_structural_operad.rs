//! SHACL test 238: `operad:StructuralOperad` instance.

/// Instance graph for Test 238: StructuralOperad with description.
pub const TEST238_STRUCTURAL_OPERAD: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix operad: <https://uor.foundation/operad/> .

operad:test_operad a owl:NamedIndividual, operad:StructuralOperad ;
    operad:operadDescription "Composition structure on 8 structural types" .
"#;
