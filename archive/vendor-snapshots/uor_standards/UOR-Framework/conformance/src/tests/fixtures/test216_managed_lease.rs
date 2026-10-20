//! SHACL test 216: `reduction:ManagedLease` instance.

/// Instance graph for Test 216: ManagedLease with lifecycle.
pub const TEST216_MANAGED_LEASE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ml_example a owl:NamedIndividual, reduction:ManagedLease ;
    reduction:managedLeaseId "lease-001"^^xsd:string ;
    reduction:leaseLifecycle reduction:Active .
"#;
