//! SHACL test 275: `reduction:ReductionAdvance` instance.

/// Instance graph for Test 275: ReductionAdvance with advanceFrom and advanceTo.
pub const TEST275_REDUCTION_ADVANCE: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_step_from_275 a owl:NamedIndividual, reduction:ReductionStep ;
    reduction:stageIndex "0"^^xsd:nonNegativeInteger ;
    reduction:stageName "Initialization" .

reduction:ex_step_to_275 a owl:NamedIndividual, reduction:ReductionStep ;
    reduction:stageIndex "1"^^xsd:nonNegativeInteger ;
    reduction:stageName "Reduction" .

reduction:ex_advance_275 a owl:NamedIndividual, reduction:ReductionAdvance ;
    reduction:advanceFrom reduction:ex_step_from_275 ;
    reduction:advanceTo   reduction:ex_step_to_275 .
"#;
