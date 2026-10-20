/// SHACL fixture for observable:WindingNumber.
pub const TEST139_WINDING_NUMBER: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:winding_1> a owl:NamedIndividual , observable:WindingNumber ;
    observable:value "2"^^xsd:decimal .
"#;
