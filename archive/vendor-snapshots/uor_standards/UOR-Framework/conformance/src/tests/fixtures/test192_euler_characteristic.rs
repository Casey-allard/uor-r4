//! SHACL test 192: `observable:EulerCharacteristicObservable`.

/// Instance graph for Test 192: EulerCharacteristicObservable.
pub const TEST192_EULER_CHARACTERISTIC: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_euler_192 a owl:NamedIndividual, observable:EulerCharacteristicObservable ;
    observable:alternatingSum "sum((-1)^k * beta_k)" .
"#;
