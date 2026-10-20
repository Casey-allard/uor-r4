//! SHACL test 236: `monoidal:MonoidalUnit` instance.

/// Instance graph for Test 236: MonoidalUnit identity computation.
pub const TEST236_MONOIDAL_UNIT: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix monoidal: <https://uor.foundation/monoidal/> .

monoidal:test_unit a owl:NamedIndividual, monoidal:MonoidalUnit ;
    monoidal:unitWitness "I tensor A iso A iso A tensor I" .
"#;
