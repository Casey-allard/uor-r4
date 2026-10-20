//! SHACL test 203: `op:SessionCompositionOperation` session composition identities.

/// Instance graph for Test 203: SessionCompositionOperation identity PK_1.
pub const TEST203_SESSION_COMPOSITION_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Covers: op:SessionCompositionOperation
op:compose_op a owl:NamedIndividual, op:SessionCompositionOperation ;
    op:arity "2"^^xsd:integer ;
    op:hasGeometricCharacter op:SessionMerge ;
    op:commutative "true"^^xsd:boolean ;
    op:associative "true"^^xsd:boolean ;
    op:operatorSignature "Session × Session → Session" .
"#;
