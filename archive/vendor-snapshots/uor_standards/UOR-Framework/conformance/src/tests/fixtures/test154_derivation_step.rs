/// SHACL fixture for derivation:DerivationStep.
pub const TEST154_DERIVATION_STEP: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix derivation: <https://uor.foundation/derivation/> .

<urn:test:derivation_step_1> a owl:NamedIndividual , derivation:DerivationStep .
"#;
