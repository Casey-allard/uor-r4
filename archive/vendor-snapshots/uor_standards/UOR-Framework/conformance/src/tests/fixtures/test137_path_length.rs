/// SHACL fixture for observable:PathLength.
pub const TEST137_PATH_LENGTH: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:path_len_1> a owl:NamedIndividual , observable:PathLength ;
    observable:value "12"^^xsd:decimal .
"#;
