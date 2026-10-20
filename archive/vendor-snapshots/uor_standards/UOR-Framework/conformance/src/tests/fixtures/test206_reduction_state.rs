//! SHACL test 206: `reduction:ReductionState` instance.

/// Instance graph for Test 206: ReductionState with currentStage and phaseAngle.
pub const TEST206_REDUCTION_STATE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:state_at_stage_2 a owl:NamedIndividual, reduction:ReductionState ;
    reduction:currentStage reduction:stage_factorize ;
    reduction:phaseAngle "Omega^2" ;
    reduction:pinnedMask "110000" ;
    reduction:freeRank "4"^^xsd:nonNegativeInteger .
"#;
