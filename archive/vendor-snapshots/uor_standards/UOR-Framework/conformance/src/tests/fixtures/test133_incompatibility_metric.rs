/// SHACL fixture for observable:IncompatibilityMetric.
pub const TEST133_INCOMPATIBILITY_METRIC: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:incompat_1> a owl:NamedIndividual , observable:IncompatibilityMetric ;
    observable:value "0.5"^^xsd:decimal .
"#;
