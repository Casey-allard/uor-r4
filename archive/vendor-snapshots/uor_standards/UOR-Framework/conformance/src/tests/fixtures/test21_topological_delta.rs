/// SHACL test 21: Topological delta — before/after Betti numbers and Euler characteristic.
pub const TEST21_TOPOLOGICAL_DELTA: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix morphism:   <https://uor.foundation/morphism/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix homology:   <https://uor.foundation/homology/> .

morphism:K_before a homology:SimplicialComplex .

morphism:K_after a homology:SimplicialComplex .

morphism:betti_before a observable:BettiNumber ;
    homology:bettiNumber "1"^^xsd:integer ;
    homology:homologyDegree "0"^^xsd:integer .

morphism:betti_after a observable:BettiNumber ;
    homology:bettiNumber "2"^^xsd:integer ;
    homology:homologyDegree "0"^^xsd:integer .

morphism:delta1 a morphism:TopologicalDelta ;
    morphism:bettisBefore morphism:betti_before ;
    morphism:bettisAfter morphism:betti_after ;
    morphism:eulerBefore "1"^^xsd:integer ;
    morphism:eulerAfter "0"^^xsd:integer ;
    morphism:nerveBefore morphism:K_before ;
    morphism:nerveAfter morphism:K_after .
"#;
