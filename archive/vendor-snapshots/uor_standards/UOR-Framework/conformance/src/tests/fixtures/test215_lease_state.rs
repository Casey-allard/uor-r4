//! SHACL test 215: `reduction:LeaseState` instance.

/// Instance graph for Test 215: LeaseState with leasePhase.
pub const TEST215_LEASE_STATE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ls_example a owl:NamedIndividual, reduction:LeaseState ;
    reduction:leasePhase "Active"^^xsd:string .
"#;
