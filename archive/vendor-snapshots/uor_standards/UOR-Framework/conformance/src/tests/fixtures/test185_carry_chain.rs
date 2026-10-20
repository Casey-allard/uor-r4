//! SHACL test 185: `carry:CarryChain`.

/// Instance graph for Test 185: CarryChain with generate/propagate/kill masks.
pub const TEST185_CARRY_CHAIN: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix carry: <https://uor.foundation/carry/> .

carry:ex_chain_185 a owl:NamedIndividual, carry:CarryChain ;
    carry:chainLength "8"^^xsd:nonNegativeInteger ;
    carry:generateMask "00100010" ;
    carry:propagateMask "01000100" ;
    carry:killMask "10011001" .
"#;
