/// SHACL test 91: TypeSynthesisGoal with Q1 target signature (Amendment 39).
/// Validates TS_1 existence criterion at 16-bit scale.
pub const TEST91_SYNTHESIS_GOAL_W16: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix resolver: <https://uor.foundation/resolver/> .

type:ex_goal_91 a owl:NamedIndividual, type:TypeSynthesisGoal ;
    type:targetEulerCharacteristic "16"^^xsd:integer ;
    type:targetBettiNumber         "0"^^xsd:nonNegativeInteger ;
    type:targetBettiNumber         "0"^^xsd:nonNegativeInteger .

resolver:ex_tsr_91 a owl:NamedIndividual, resolver:TypeSynthesisResolver ;
    resolver:synthesisGoal type:ex_goal_91 .
"#;
