//! SHACL test 246: `reduction:PhaseRotationScheduler` and `reduction:TargetConvergenceAngle`.

/// Instance graph for Test 246: PhaseRotationScheduler and TargetConvergenceAngle.
pub const TEST246_PHASE_ROTATION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_scheduler_246 a owl:NamedIndividual, reduction:PhaseRotationScheduler ;
    reduction:rotationSchedule "0, 30, 60, 90, 120, 150" ;
    reduction:baseAngle "30" .

reduction:ex_target_246 a owl:NamedIndividual, reduction:TargetConvergenceAngle ;
    reduction:targetAngle "180" .
"#;
