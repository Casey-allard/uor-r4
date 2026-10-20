//! SHACL test 233: `interaction:NegotiationTrace` instance.

/// Instance graph for Test 233: NegotiationTrace with convergence.
pub const TEST233_NEGOTIATION_TRACE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:test_negotiation a owl:NamedIndividual, interaction:NegotiationTrace ;
    interaction:negotiationSteps "12"^^xsd:nonNegativeInteger ;
    interaction:isConvergent true .
"#;
