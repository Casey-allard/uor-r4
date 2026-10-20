/// SHACL test 111: Address with cryptographic primitive pinning —
/// demonstrates u:digestAlgorithm, u:canonicalBytes, and algorithm-prefixed
/// u:digest on a Q0 element (Amendment 43).
pub const TEST111_ADDRESS_CRYPTO_PINNING: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix u:    <https://uor.foundation/u/> .

# Element for datum value 42 at Q0 (8-bit ring)
u:ex_addr_111 a owl:NamedIndividual, u:Element ;
    rdfs:label "Address for 42 at Q0" ;
    u:glyph "\u2819\u2840" ;
    u:length "2"^^xsd:nonNegativeInteger ;
    u:quantum "1"^^xsd:positiveInteger ;
    u:digest "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262" ;
    u:digestAlgorithm "blake3" ;
    u:canonicalBytes "552000002a"^^xsd:hexBinary .
"#;
