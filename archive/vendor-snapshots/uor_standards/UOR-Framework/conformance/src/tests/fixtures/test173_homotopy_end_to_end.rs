//! SHACL test 173: End-to-end homotopy pipeline.

/// Instance graph for Test 173: CechNerve -> KanComplex -> PostnikovTruncation -> HomotopyGroup -> HigherMonodromy.
pub const TEST173_HOMOTOPY_END_TO_END: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix homology:   <https://uor.foundation/homology/> .
@prefix observable: <https://uor.foundation/observable/> .

resolver:ex_nerve_173 a owl:NamedIndividual, resolver:CechNerve, homology:KanComplex .

homology:ex_trunc_173 a owl:NamedIndividual, homology:PostnikovTruncation ;
    homology:truncationLevel  "1"^^xsd:nonNegativeInteger ;
    homology:truncationSource resolver:ex_nerve_173 .

observable:ex_pi1_173 a owl:NamedIndividual, observable:HomotopyGroup ;
    observable:homotopyDimension "1"^^xsd:nonNegativeInteger .

observable:ex_hm_173 a owl:NamedIndividual, observable:HigherMonodromy ;
    observable:higherMonodromyDimension "2"^^xsd:nonNegativeInteger .
"#;
