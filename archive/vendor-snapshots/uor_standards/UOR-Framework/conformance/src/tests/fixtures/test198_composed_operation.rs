//! SHACL test 198: `op:ComposedOperation` instance.

/// Instance graph for Test 198: ComposedOperation with composedOfOps.
pub const TEST198_COMPOSED_OPERATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# op:ComposedOperation parent class covered via subclass instantiation
op:dispatch a owl:NamedIndividual, op:DispatchOperation ;
    op:arity "2"^^xsd:integer ;
    op:hasGeometricCharacter op:ConstraintSelection ;
    op:commutative "false"^^xsd:boolean ;
    op:associative "false"^^xsd:boolean ;
    op:operatorSignature "Query × ResolverRegistry → Resolver" .
"#;
