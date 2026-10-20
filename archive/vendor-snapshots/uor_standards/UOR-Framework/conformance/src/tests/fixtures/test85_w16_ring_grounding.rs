/// SHACL test 85: Q1Ring individual grounding at quantum level Q1 (Amendment 39).
pub const TEST85_W16_RING_GROUNDING: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:ex_q1ring_85 a owl:NamedIndividual, schema:Q1Ring ;
    schema:Q1bitWidth     "16"^^xsd:positiveInteger ;
    schema:Q1capacity     "65536"^^xsd:positiveInteger ;
    schema:atWittLevel schema:Q1 .
"#;
