/// SHACL test 31: Witt level -- WittLevel individuals W8-W32 with
/// bitsWidth/cycleSize/nextWittLevel. W32 is terminal (no nextWittLevel).
pub const TEST31_WITT_LEVEL: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:W8 a owl:NamedIndividual, schema:WittLevel ;
    schema:bitsWidth    "8"^^xsd:positiveInteger ;
    schema:cycleSize    "256"^^xsd:positiveInteger ;
    schema:nextWittLevel    schema:W16 .

schema:W16 a owl:NamedIndividual, schema:WittLevel ;
    schema:bitsWidth    "16"^^xsd:positiveInteger ;
    schema:cycleSize    "65536"^^xsd:positiveInteger ;
    schema:nextWittLevel    schema:W24 .

schema:W24 a owl:NamedIndividual, schema:WittLevel ;
    schema:bitsWidth    "24"^^xsd:positiveInteger ;
    schema:cycleSize    "16777216"^^xsd:positiveInteger ;
    schema:nextWittLevel    schema:W32 .

schema:W32 a owl:NamedIndividual, schema:WittLevel ;
    schema:bitsWidth    "32"^^xsd:positiveInteger ;
    schema:cycleSize    "4294967296"^^xsd:positiveInteger .
"#;
