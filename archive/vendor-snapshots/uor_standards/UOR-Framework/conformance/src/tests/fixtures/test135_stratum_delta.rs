/// SHACL fixture for observable:StratumDelta.
pub const TEST135_STRATUM_DELTA: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:stratum_delta_1> a owl:NamedIndividual , observable:StratumDelta ;
    observable:value "1"^^xsd:decimal .
"#;
