//! SHACL test 170: `homology:KanComplex` and `homology:HornFiller`.

/// Instance graph for Test 170: KanComplex with HornFiller.
pub const TEST170_KAN_COMPLEX: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix homology: <https://uor.foundation/homology/> .

homology:ex_kan_170 a owl:NamedIndividual, homology:KanComplex ;
    homology:kanWitness homology:ex_horn_170 .

homology:ex_horn_170 a owl:NamedIndividual, homology:HornFiller ;
    homology:hornDimension "2"^^xsd:nonNegativeInteger ;
    homology:hornPosition  "1"^^xsd:nonNegativeInteger .
"#;
