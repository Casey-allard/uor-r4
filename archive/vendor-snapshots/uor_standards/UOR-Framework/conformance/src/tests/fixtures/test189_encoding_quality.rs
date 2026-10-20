//! SHACL test 189: `carry:EncodingQuality`.

/// Instance graph for Test 189: EncodingQuality metric for an encoding.
pub const TEST189_ENCODING_QUALITY: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix carry: <https://uor.foundation/carry/> .

carry:ex_quality_189 a owl:NamedIndividual, carry:EncodingQuality ;
    carry:meanDelta "1.42"^^xsd:decimal ;
    carry:discriminationRatio "0.87"^^xsd:decimal ;
    carry:isOptimalEncoding "false"^^xsd:boolean .
"#;
