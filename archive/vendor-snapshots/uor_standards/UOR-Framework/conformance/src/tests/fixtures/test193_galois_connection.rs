//! SHACL test 193: `type:GaloisConnection`.

/// Instance graph for Test 193: GaloisConnection with upper and lower adjoint.
pub const TEST193_GALOIS_CONNECTION: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix type:  <https://uor.foundation/type/> .

type:ex_galois_193 a owl:NamedIndividual, type:GaloisConnection ;
    type:upperAdjoint "closure(T)" ;
    type:lowerAdjoint "interior(T)" .
"#;
