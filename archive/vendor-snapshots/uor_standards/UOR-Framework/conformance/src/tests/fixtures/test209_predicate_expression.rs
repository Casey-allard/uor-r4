//! SHACL test 209: `reduction:PredicateExpression` instance.

/// Instance graph for Test 209: PredicateExpression with predicateField.
pub const TEST209_PREDICATE_EXPRESSION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:pred_example a owl:NamedIndividual, reduction:PredicateExpression ;
    reduction:predicateField "freeRank"^^xsd:string ;
    reduction:predicateOperator ">"^^xsd:string ;
    reduction:predicateValue "0"^^xsd:string .
"#;
