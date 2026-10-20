//! SHACL test 205: `reduction:ReductionStep` instance.

/// Instance graph for Test 205: ReductionStep with stageIndex and stageName.
pub const TEST205_REDUCTION_STEP: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:stage_initialization a owl:NamedIndividual, reduction:ReductionStep ;
    reduction:stageIndex "0"^^xsd:nonNegativeInteger ;
    reduction:stageName "Initialization" ;
    reduction:expectedPhase "Omega^0" .
"#;
