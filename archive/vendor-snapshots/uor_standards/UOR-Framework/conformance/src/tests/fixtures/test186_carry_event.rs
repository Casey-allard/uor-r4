//! SHACL test 186: `carry:CarryEvent`.

/// Instance graph for Test 186: CarryEvent at a specific site.
pub const TEST186_CARRY_EVENT: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix carry: <https://uor.foundation/carry/> .

carry:ex_event_186 a owl:NamedIndividual, carry:CarryEvent ;
    carry:eventKind "Generate" ;
    carry:sitePosition "3"^^xsd:nonNegativeInteger .
"#;
