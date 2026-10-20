//! SHACL test 194: `homology:SimplicialComplex` (nerve operations).

/// Instance graph for Test 194: SimplicialComplex nerve operation instance.
pub const TEST194_NERVE_OPERATIONS: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix homology: <https://uor.foundation/homology/> .

homology:ex_nerve_194 a owl:NamedIndividual, homology:SimplicialComplex ;
    homology:dimension "3"^^xsd:nonNegativeInteger .
"#;
