//! SHACL test 244: `reduction:SubleaseTransfer`.

/// Instance graph for Test 244: SubleaseTransfer.
pub const TEST244_SUBLEASE_TRANSFER: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_sublease_244 a owl:NamedIndividual, reduction:SubleaseTransfer ;
    reduction:sourceLeaseRef "lease_alpha" ;
    reduction:targetLeaseRef "lease_beta" ;
    reduction:transferredBudget "5"^^xsd:nonNegativeInteger ;
    reduction:transferCompleted "true"^^xsd:boolean .
"#;
