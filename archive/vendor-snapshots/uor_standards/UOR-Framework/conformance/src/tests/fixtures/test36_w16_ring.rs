/// SHACL test 36: W16Ring and WittLevel chain — Amendment 26.
pub const TEST36_W16_RING: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:ex_q1ring_36 a owl:NamedIndividual, schema:W16Ring ;
    schema:W16bitWidth  "16"^^xsd:positiveInteger ;
    schema:W16capacity  "65536"^^xsd:positiveInteger .

schema:W8 schema:nextWittLevel schema:W16 .
"#;
