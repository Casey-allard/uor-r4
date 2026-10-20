//! SHACL test 267: `CompileUnitBuilder` and declaration builder types.

/// Instance graph for Test 267: All 9 builder types and ShapeViolationReport.
pub const TEST267_COMPILE_UNIT_BUILDER: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix conformance: <https://uor.foundation/conformance/> .

conformance:ex_cu_builder_267 a owl:NamedIndividual, conformance:CompileUnitBuilder .
conformance:ex_effect_decl_267 a owl:NamedIndividual, conformance:EffectDeclaration .
conformance:ex_grounding_decl_267 a owl:NamedIndividual, conformance:GroundingDeclaration .
conformance:ex_dispatch_decl_267 a owl:NamedIndividual, conformance:DispatchDeclaration .
conformance:ex_lease_decl_267 a owl:NamedIndividual, conformance:LeaseDeclaration .
conformance:ex_stream_decl_267 a owl:NamedIndividual, conformance:StreamDeclaration .
conformance:ex_predicate_decl_267 a owl:NamedIndividual, conformance:PredicateDeclaration .
conformance:ex_parallel_decl_267 a owl:NamedIndividual, conformance:ParallelDeclaration .
conformance:ex_ql_decl_267 a owl:NamedIndividual, conformance:WittLevelDeclaration .
conformance:ex_violation_267 a owl:NamedIndividual, conformance:ShapeViolationReport .
"#;
