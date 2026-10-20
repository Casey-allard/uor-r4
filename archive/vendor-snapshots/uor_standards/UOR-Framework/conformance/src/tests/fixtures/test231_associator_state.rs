//! SHACL test 231: `interaction:AssociatorState` instance.

/// Instance graph for Test 231: AssociatorState with norm.
pub const TEST231_ASSOCIATOR_STATE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:test_associator a owl:NamedIndividual, interaction:AssociatorState ;
    interaction:associatorNorm "0.07" .
"#;
