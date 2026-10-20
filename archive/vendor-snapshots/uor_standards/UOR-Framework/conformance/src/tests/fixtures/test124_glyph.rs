/// SHACL fixture for u:Element.
pub const TEST124_GLYPH: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix u:   <https://uor.foundation/u/> .

<urn:test:element_1> a owl:NamedIndividual , u:Element ;
    u:wittLength "8"^^xsd:positiveInteger ;
    u:length     "1"^^xsd:nonNegativeInteger .
"#;
