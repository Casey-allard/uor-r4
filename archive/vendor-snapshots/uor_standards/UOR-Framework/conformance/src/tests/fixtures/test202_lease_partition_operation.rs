//! SHACL test 202: `op:LeasePartitionOperation` lease identities.

/// Instance graph for Test 202: LeasePartitionOperation identity PL_1.
pub const TEST202_LEASE_PARTITION_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Covers: op:LeasePartitionOperation
op:partition_op a owl:NamedIndividual, op:LeasePartitionOperation ;
    op:arity "2"^^xsd:integer ;
    op:hasGeometricCharacter op:SitePartition ;
    op:commutative "false"^^xsd:boolean ;
    op:associative "false"^^xsd:boolean ;
    op:operatorSignature "SharedContext × N → ContextLease^k" .
"#;
