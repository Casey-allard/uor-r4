/// SHACL test 40: TypeSynthesisGoal and TypeSynthesisResolver — Amendment 28.
pub const TEST40_TYPE_SYNTHESIS_GOAL: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix resolver: <https://uor.foundation/resolver/> .

type:ex_goal_40 a owl:NamedIndividual, type:TypeSynthesisGoal ;
    type:targetEulerCharacteristic  "8"^^xsd:integer ;
    type:targetBettiNumber          "0"^^xsd:nonNegativeInteger ;
    type:targetBettiNumber          "0"^^xsd:nonNegativeInteger .

resolver:ex_tsr_40 a owl:NamedIndividual, resolver:TypeSynthesisResolver ;
    resolver:synthesisGoal  type:ex_goal_40 .
"#;
