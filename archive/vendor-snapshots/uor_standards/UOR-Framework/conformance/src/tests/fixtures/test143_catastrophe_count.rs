/// SHACL fixture for observable:CatastropheCount.
pub const TEST143_CATASTROPHE_COUNT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:cat_count_1> a owl:NamedIndividual , observable:CatastropheCount ;
    observable:value "2"^^xsd:decimal .
"#;
