//! SHACL test 210: `reduction:GuardExpression` instance.

/// Instance graph for Test 210: GuardExpression with guardPredicates.
pub const TEST210_GUARD_EXPRESSION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:guard_example a owl:NamedIndividual, reduction:GuardExpression ;
    reduction:guardPredicates reduction:true_predicate .
"#;
