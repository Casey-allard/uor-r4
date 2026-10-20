/// SHACL test 82: WittLevel with levelSuccessor (Amendment 37, Gap 5).
pub const TEST82_LEVEL_SUCCESSOR: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:ex_ql_82 a owl:NamedIndividual, schema:WittLevel ;
    schema:bitsWidth "16"^^xsd:positiveInteger ;
    schema:cycleSize "65536"^^xsd:positiveInteger ;
    schema:wittLevelPredecessor schema:W8 .
"#;
