//! SHACL test 259: `failure` namespace types.

/// Instance graph for Test 259: Failure and computation result types.
pub const TEST259_FAILURE_TYPES: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix failure: <https://uor.foundation/failure/> .

failure:ex_result_259 a owl:NamedIndividual, failure:ComputationResult .
failure:ex_guard_259 a owl:NamedIndividual, failure:GuardFailure .
failure:ex_contradiction_259 a owl:NamedIndividual, failure:ConstraintContradiction .
failure:ex_exhaustion_259 a owl:NamedIndividual, failure:SiteExhaustion .
failure:ex_lift_obstruction_259 a owl:NamedIndividual, failure:LiftObstructionFailure .
failure:ex_partial_259 a owl:NamedIndividual, failure:PartialComputation .
failure:ex_total_259 a owl:NamedIndividual, failure:TotalComputation .
failure:ex_recovery_259 a owl:NamedIndividual, failure:Recovery .
failure:ex_propagation_259 a owl:NamedIndividual, failure:FailurePropagation .
"#;
