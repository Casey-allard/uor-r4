/// SHACL fixture for observable:Commutator.
pub const TEST144_COMMUTATOR: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:commutator_1> a owl:NamedIndividual , observable:Commutator ;
    observable:value "0"^^xsd:decimal .
"#;
