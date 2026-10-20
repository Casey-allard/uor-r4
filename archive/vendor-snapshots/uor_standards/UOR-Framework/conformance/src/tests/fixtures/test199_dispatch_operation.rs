//! SHACL test 199: `op:DispatchOperation` dispatch determinism identity.

/// Instance graph for Test 199: DispatchOperation identity DD_1.
pub const TEST199_DISPATCH_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:DD_1 a owl:NamedIndividual, op:Identity ;
    op:lhs "δ(q, R)" ;
    op:rhs "δ(q, R)" ;
    op:verificationDomain op:ComposedAlgebraic ;
    op:universallyValid "true"^^xsd:boolean .
"#;
