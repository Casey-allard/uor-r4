//! SHACL test 234: `interaction:InteractionNerve` instance.

/// Instance graph for Test 234: InteractionNerve with Betti numbers.
pub const TEST234_INTERACTION_NERVE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:test_nerve a owl:NamedIndividual, interaction:InteractionNerve ;
    interaction:nerveDimension "4"^^xsd:nonNegativeInteger ;
    interaction:nerveBettiNumbers "[1,3,2,1,0]" .
"#;
