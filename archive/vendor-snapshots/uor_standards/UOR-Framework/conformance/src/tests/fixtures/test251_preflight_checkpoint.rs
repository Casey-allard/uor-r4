//! SHACL test 251: `reduction:PreflightCheck` and `reduction:LeaseCheckpoint`.

/// Instance graph for Test 251: PreflightCheck and LeaseCheckpoint.
pub const TEST251_PREFLIGHT_CHECKPOINT: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_preflight_251 a owl:NamedIndividual, reduction:PreflightCheck ;
    reduction:preflightKind "FeasibilityCheck" ;
    reduction:preflightResult "pass" .

reduction:ex_checkpoint_251 a owl:NamedIndividual, reduction:LeaseCheckpoint ;
    reduction:checkpointEpoch "5"^^xsd:nonNegativeInteger ;
    reduction:leaseRemainingBudget "3"^^xsd:nonNegativeInteger .
"#;
