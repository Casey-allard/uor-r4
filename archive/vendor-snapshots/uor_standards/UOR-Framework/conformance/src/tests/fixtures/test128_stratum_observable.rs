/// SHACL fixture for observable:StratumObservable.
pub const TEST128_STRATUM_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:stratum_obs_1> a owl:NamedIndividual , observable:StratumObservable ;
    observable:value "42"^^xsd:decimal .
"#;
