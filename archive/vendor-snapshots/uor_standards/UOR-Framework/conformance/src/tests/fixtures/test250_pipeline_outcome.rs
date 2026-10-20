//! SHACL test 250: `reduction:PipelineSuccess` and `reduction:PipelineFailureReason`.

/// Instance graph for Test 250: PipelineSuccess and PipelineFailureReason.
pub const TEST250_PIPELINE_OUTCOME: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_success_250 a owl:NamedIndividual, reduction:PipelineSuccess ;
    reduction:finalGrounding "1.0" .

reduction:ex_failure_250 a owl:NamedIndividual, reduction:PipelineFailureReason ;
    reduction:failureKind "DispatchMiss" ;
    reduction:failureDetail "No resolver found for query" ;
    reduction:failureStage "Declare" .
"#;
