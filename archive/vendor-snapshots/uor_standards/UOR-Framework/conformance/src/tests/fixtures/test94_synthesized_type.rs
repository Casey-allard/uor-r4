/// SHACL test 94: SynthesizedType with TypeSynthesisResult and MinimalConstraintBasis
/// — Q1 scale (Amendment 39). Validates TS_2 (basisSize = n) and TS_5 (duality).
pub const TEST94_SYNTHESIZED_TYPE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .
@prefix cert: <https://uor.foundation/cert/> .

type:ex_synthesized_94 a owl:NamedIndividual, type:SynthesizedType ;
    type:synthesisResult type:ex_result_94 .

type:ex_result_94 a owl:NamedIndividual, type:TypeSynthesisResult .

type:ex_basis_94 a owl:NamedIndividual, type:MinimalConstraintBasis ;
    type:basisSize "16"^^xsd:nonNegativeInteger .

cert:ex_cert_94 a owl:NamedIndividual, cert:CompletenessCertificate .
"#;
