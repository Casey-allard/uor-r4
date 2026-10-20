//! SHACL test 207: `reduction:PhaseGateAttestation` instance.

/// Instance graph for Test 207: PhaseGateAttestation with gateStage and gateResult.
pub const TEST207_PHASE_GATE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:gate_initialization a owl:NamedIndividual, reduction:PhaseGateAttestation ;
    reduction:gateStage reduction:stage_initialization ;
    reduction:gateExpectedPhase "Omega^0" ;
    reduction:gateResult "true"^^xsd:boolean .
"#;
