/// SHACL fixture for observable:StratumValue.
pub const TEST134_STRATUM_VALUE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:stratum_val_1> a owl:NamedIndividual , observable:StratumValue ;
    observable:value "3"^^xsd:decimal .
"#;
