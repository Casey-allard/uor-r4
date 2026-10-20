//! SHACL test 229: `interaction:InteractionContext` instance.

/// Instance graph for Test 229: InteractionContext with shared sites.
pub const TEST229_INTERACTION_CONTEXT: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:test_context a owl:NamedIndividual, interaction:InteractionContext ;
    interaction:entityA "entity_alpha" ;
    interaction:entityB "entity_beta" ;
    interaction:sharedSiteMask "0b1100" ;
    interaction:commutatorNorm "0.42"^^xsd:decimal .
"#;
