//! SHACL test 264: `conformance` namespace types.

/// Instance graph for Test 264: Conformance shape and validation types.
pub const TEST264_CONFORMANCE_TYPES: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix conformance: <https://uor.foundation/conformance/> .

conformance:ex_shape_264 a owl:NamedIndividual, conformance:Shape .
conformance:ex_prop_constraint_264 a owl:NamedIndividual, conformance:PropertyConstraint .
conformance:ex_ql_shape_264 a owl:NamedIndividual, conformance:WittLevelShape .
conformance:ex_effect_shape_264 a owl:NamedIndividual, conformance:EffectShape .
conformance:ex_parallel_shape_264 a owl:NamedIndividual, conformance:ParallelShape .
conformance:ex_stream_shape_264 a owl:NamedIndividual, conformance:StreamShape .
conformance:ex_dispatch_shape_264 a owl:NamedIndividual, conformance:DispatchShape .
conformance:ex_lease_shape_264 a owl:NamedIndividual, conformance:LeaseShape .
conformance:ex_grounding_shape_264 a owl:NamedIndividual, conformance:GroundingShape .
conformance:ex_validation_result_264 a owl:NamedIndividual, conformance:ValidationResult .
conformance:ex_predicate_shape_264 a owl:NamedIndividual, conformance:PredicateShape .
"#;
