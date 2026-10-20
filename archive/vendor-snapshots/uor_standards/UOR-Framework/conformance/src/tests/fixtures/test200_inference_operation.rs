//! SHACL test 200: `op:InferenceOperation` inference identities.

/// Instance graph for Test 200: InferenceOperation identity PI_1.
pub const TEST200_INFERENCE_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Covers: op:InferenceOperation
op:infer a owl:NamedIndividual, op:InferenceOperation ;
    op:arity "2"^^xsd:integer ;
    op:hasGeometricCharacter op:ResolutionTraversal ;
    op:commutative "false"^^xsd:boolean ;
    op:associative "false"^^xsd:boolean ;
    op:operatorSignature "Symbol × Context → ResolvedType" .
"#;
