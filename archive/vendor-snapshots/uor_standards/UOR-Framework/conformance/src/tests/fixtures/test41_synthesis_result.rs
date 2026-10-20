/// SHACL test 41: Full synthesis round-trip — Amendment 28.
pub const TEST41_SYNTHESIS_RESULT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix cert:       <https://uor.foundation/cert/> .

type:ex_goal_41 a owl:NamedIndividual, type:TypeSynthesisGoal ;
    type:targetEulerCharacteristic  "8"^^xsd:integer .

resolver:ex_tsr_41 a owl:NamedIndividual, resolver:TypeSynthesisResolver ;
    resolver:synthesisGoal  type:ex_goal_41 .

resolver:ex_css_41 a owl:NamedIndividual, resolver:ConstraintSearchState ;
    resolver:exploredCount  "4"^^xsd:nonNegativeInteger .

type:ex_result_41 a owl:NamedIndividual, type:TypeSynthesisResult .

type:ex_synthesized_41 a owl:NamedIndividual, type:SynthesizedType ;
    type:synthesisResult  type:ex_result_41 .

type:ex_basis_41 a owl:NamedIndividual, type:MinimalConstraintBasis ;
    type:basisSize  "8"^^xsd:nonNegativeInteger .

observable:ex_sig_41 a owl:NamedIndividual, observable:SynthesisSignature ;
    observable:realisedEuler  "8"^^xsd:integer .

derivation:ex_step_41 a owl:NamedIndividual, derivation:SynthesisStep ;
    derivation:stepIndex      "0"^^xsd:nonNegativeInteger ;
    derivation:signatureAfter observable:ex_sig_41 .

cert:ex_cert_41 a owl:NamedIndividual, cert:CompletenessCertificate .
"#;
