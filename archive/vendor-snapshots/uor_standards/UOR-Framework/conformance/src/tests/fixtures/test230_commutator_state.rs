//! SHACL test 230: `interaction:CommutatorState` instance.

/// Instance graph for Test 230: CommutatorState with commutator value.
pub const TEST230_COMMUTATOR_STATE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:test_commutator a owl:NamedIndividual, interaction:CommutatorState ;
    interaction:commutatorValue "0.15"^^xsd:decimal .
"#;
