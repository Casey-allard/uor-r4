//! SHACL test 213: `reduction:ReductionTransaction` instance.

/// Instance graph for Test 213: ReductionTransaction with transactionPolicy.
pub const TEST213_REDUCTION_TRANSACTION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:tx_example a owl:NamedIndividual, reduction:ReductionTransaction ;
    reduction:transactionPolicy "AllOrNothing"^^xsd:string ;
    reduction:transactionOutcome "committed"^^xsd:string .
"#;
