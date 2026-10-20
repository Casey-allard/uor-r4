//! SHACL test 187: `carry:CarryProfile`.

/// Instance graph for Test 187: CarryProfile summarizing a carry chain.
pub const TEST187_CARRY_PROFILE: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix carry: <https://uor.foundation/carry/> .

carry:ex_profile_187 a owl:NamedIndividual, carry:CarryProfile ;
    carry:carryCount "3"^^xsd:nonNegativeInteger ;
    carry:maxPropagationLength "2"^^xsd:nonNegativeInteger ;
    carry:profileChain carry:ex_chain_185 .
"#;
