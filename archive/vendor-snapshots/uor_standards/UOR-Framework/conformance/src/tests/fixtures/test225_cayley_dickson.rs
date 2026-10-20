//! SHACL test 225: `division:CayleyDicksonConstruction` instance.

/// Instance graph for Test 225: CayleyDicksonConstruction with source, target, adjoined element.
pub const TEST225_CAYLEY_DICKSON: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix division: <https://uor.foundation/division/> .

division:test_doubling a owl:NamedIndividual, division:CayleyDicksonConstruction ;
    division:cayleyDicksonSource division:RealAlgebra ;
    division:cayleyDicksonTarget division:ComplexAlgebra ;
    division:adjoinedElement "i" ;
    division:conjugationRule "i² = −1" .
"#;
