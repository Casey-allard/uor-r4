/// SHACL fixture for observable:ReductionLength.
pub const TEST140_REDUCTION_LENGTH: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:reduction_len_1> a owl:NamedIndividual , observable:ReductionLength ;
    observable:value "8"^^xsd:decimal .
"#;
