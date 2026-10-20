//! SHACL test 235: `monoidal:MonoidalProduct` instance.

/// Instance graph for Test 235: MonoidalProduct with sequential composition.
pub const TEST235_MONOIDAL_PRODUCT: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix monoidal: <https://uor.foundation/monoidal/> .

monoidal:test_product a owl:NamedIndividual, monoidal:MonoidalProduct ;
    monoidal:leftComponent "computation_A" ;
    monoidal:rightComponent "computation_B" ;
    monoidal:composedEndpoint "A_then_B" ;
    monoidal:monoidalGrounding "max(sigma_A, sigma_B)" .
"#;
