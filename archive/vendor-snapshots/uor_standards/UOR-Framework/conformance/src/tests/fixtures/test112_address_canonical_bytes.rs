/// SHACL test 112: Address at Q1 with canonical byte encoding —
/// demonstrates the 6-byte canonical form (4-byte header + 2-byte LE payload)
/// and the secondary SHA-256 algorithm for an Element (Amendment 43).
pub const TEST112_ADDRESS_CANONICAL_BYTES: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix u:    <https://uor.foundation/u/> .

# Element for datum value 42 at Q1 (16-bit ring)
# Canonical bytes: header 55 52 01 00 + LE payload 2a 00 = 6 bytes
u:ex_addr_112 a owl:NamedIndividual, u:Element ;
    rdfs:label "Address for 42 at Q1 (SHA-256)" ;
    u:glyph "\u2800\u2819\u2840" ;
    u:length "3"^^xsd:nonNegativeInteger ;
    u:quantum "2"^^xsd:positiveInteger ;
    u:digest "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" ;
    u:digestAlgorithm "sha256" ;
    u:canonicalBytes "5552010000002a00"^^xsd:hexBinary .
"#;
