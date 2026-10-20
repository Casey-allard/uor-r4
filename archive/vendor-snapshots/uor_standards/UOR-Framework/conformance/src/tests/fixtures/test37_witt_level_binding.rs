/// SHACL test 37: WittLevelBinding and universallyValid — Amendment 26.
pub const TEST37_WITT_LEVEL_BINDING: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix op:     <https://uor.foundation/op/> .
@prefix schema: <https://uor.foundation/schema/> .

op:criticalIdentity op:universallyValid    "true"^^xsd:boolean ;
    op:verifiedAtLevel op:ex_binding_q0_37 ;
    op:verifiedAtLevel op:ex_binding_q1_37 .

op:ex_binding_q0_37 a owl:NamedIndividual, op:WittLevelBinding ;
    op:bindingLevel schema:W8 .

op:ex_binding_q1_37 a owl:NamedIndividual, op:WittLevelBinding ;
    op:bindingLevel schema:W16 .
"#;
