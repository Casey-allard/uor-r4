/// SHACL fixture for observable:PathObservable.
pub const TEST130_PATH_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:path_obs_1> a owl:NamedIndividual , observable:PathObservable ;
    observable:value "3"^^xsd:decimal .
"#;
