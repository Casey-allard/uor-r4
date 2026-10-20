//! SHACL test 188: `carry:EncodingConfiguration`.

/// Instance graph for Test 188: EncodingConfiguration for a symbol set.
pub const TEST188_ENCODING_CONFIGURATION: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix carry: <https://uor.foundation/carry/> .

carry:ex_enc_188 a owl:NamedIndividual, carry:EncodingConfiguration ;
    carry:symbolSetSize "26"^^xsd:positiveInteger ;
    carry:quantizationBits "5"^^xsd:positiveInteger ;
    carry:encodingMap "a=0,b=1,...,z=25" .
"#;
