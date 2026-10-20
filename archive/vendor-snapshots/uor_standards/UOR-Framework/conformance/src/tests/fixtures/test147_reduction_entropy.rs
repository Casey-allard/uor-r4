/// SHACL fixture for observable:ReductionEntropy.
pub const TEST147_REDUCTION_ENTROPY: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:reduction_entropy_1> a owl:NamedIndividual , observable:ReductionEntropy ;
    observable:value "0.693"^^xsd:decimal .
"#;
