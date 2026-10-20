//! SHACL test 201: `op:AccumulationOperation` accumulation identities.

/// Instance graph for Test 201: AccumulationOperation identity PA_1.
pub const TEST201_ACCUMULATION_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Covers: op:AccumulationOperation
op:accumulate a owl:NamedIndividual, op:AccumulationOperation ;
    op:arity "2"^^xsd:integer ;
    op:hasGeometricCharacter op:SitePinning ;
    op:commutative "false"^^xsd:boolean ;
    op:associative "true"^^xsd:boolean ;
    op:operatorSignature "Binding × Context → Context" .
"#;
