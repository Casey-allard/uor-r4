//! SHACL test 247: `reduction:ComplexConjugateRollback` and `reduction:ReductionRule`.

/// Instance graph for Test 247: ComplexConjugateRollback and ReductionRule.
pub const TEST247_ROLLBACK_TRANSITION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_rollback_247 a owl:NamedIndividual, reduction:ComplexConjugateRollback ;
    reduction:rollbackTarget "stage_2" .

reduction:ex_rule_247 a owl:NamedIndividual, reduction:ReductionRule ;
    reduction:transitionGuard reduction:ex_guard_247 ;
    reduction:transitionEffect reduction:ex_effect_247 .
"#;
