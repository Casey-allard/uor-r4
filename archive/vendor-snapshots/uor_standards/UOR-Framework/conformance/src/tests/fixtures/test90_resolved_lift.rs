/// SHACL test 90: Resolved lift — SynthesizedType at Q1 with basisSize increment
/// (Amendment 39). Validates QLS_3.
pub const TEST90_RESOLVED_LIFT: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix type:   <https://uor.foundation/type/> .
@prefix schema: <https://uor.foundation/schema/> .

type:ex_synthesized_90 a owl:NamedIndividual, type:SynthesizedType ;
    type:synthesisResult type:ex_result_90 .

type:ex_result_90 a owl:NamedIndividual, type:TypeSynthesisResult .

type:ex_basis_90 a owl:NamedIndividual, type:MinimalConstraintBasis ;
    type:basisSize "9"^^xsd:nonNegativeInteger .

type:ex_lift_90 a owl:NamedIndividual, type:WittLift ;
    type:liftBase        type:ex_synthesized_90 ;
    type:liftTargetLevel schema:Q1 .
"#;
