/// SHACL test 54: Grounded context — GroundedContext with groundingDegree,
/// contextTemperature, isGrounded, groundingPhase (Amendment 33).
pub const TEST54_GROUNDED_CONTEXT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix state:      <https://uor.foundation/state/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. GroundedContext with properties
state:ex_sc_54 a owl:NamedIndividual, state:GroundedContext ;
    state:groundingDegree "0.95"^^xsd:decimal ;
    state:contextTemperature "42"^^xsd:integer ;
    state:isGrounded "true"^^xsd:boolean ;
    state:groundingPhase observable:FullGrounding .
"#;
