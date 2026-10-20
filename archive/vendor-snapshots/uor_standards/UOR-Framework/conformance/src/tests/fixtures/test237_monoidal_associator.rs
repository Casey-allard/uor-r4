//! SHACL test 237: `monoidal:MonoidalAssociator` instance.

/// Instance graph for Test 237: MonoidalAssociator witness.
pub const TEST237_MONOIDAL_ASSOCIATOR: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix monoidal: <https://uor.foundation/monoidal/> .

monoidal:test_assoc a owl:NamedIndividual, monoidal:MonoidalAssociator ;
    monoidal:associatorLeftTriple "(A tensor B) tensor C" ;
    monoidal:associatorRightTriple "A tensor (B tensor C)" ;
    monoidal:associatorWitness "canonical isomorphism" .
"#;
