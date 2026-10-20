//! SHACL test 243: `interaction:InteractionComposition`.

/// Instance graph for Test 243: InteractionComposition.
pub const TEST243_INTERACTION_COMPOSITION: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:ex_composition_243 a owl:NamedIndividual, interaction:InteractionComposition ;
    interaction:reificationDepth "1"^^xsd:nonNegativeInteger .
"#;
