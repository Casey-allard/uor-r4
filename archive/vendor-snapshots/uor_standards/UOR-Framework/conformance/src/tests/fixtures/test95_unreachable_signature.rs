/// SHACL test 95: Unreachable signature rejection — TypeSynthesisGoal with
/// invalid target, RefinementSuggestion returned (Amendment 39). Validates TS_7.
pub const TEST95_UNREACHABLE_SIGNATURE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

type:ex_goal_95 a owl:NamedIndividual, type:TypeSynthesisGoal ;
    type:targetEulerCharacteristic "0"^^xsd:integer ;
    type:targetBettiNumber         "0"^^xsd:nonNegativeInteger ;
    type:targetForbidden           "true"^^xsd:boolean .

observable:ex_sig_95 a owl:NamedIndividual, observable:SynthesisSignature ;
    observable:achievabilityStatus observable:Forbidden ;
    observable:isForbidden         "true"^^xsd:boolean .

resolver:ex_suggestion_95 a owl:NamedIndividual, resolver:RefinementSuggestion .
"#;
