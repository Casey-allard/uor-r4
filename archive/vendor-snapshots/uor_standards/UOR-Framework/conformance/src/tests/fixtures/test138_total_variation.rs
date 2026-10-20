/// SHACL fixture for observable:TotalVariation.
pub const TEST138_TOTAL_VARIATION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:total_var_1> a owl:NamedIndividual , observable:TotalVariation ;
    observable:value "6"^^xsd:decimal .
"#;
