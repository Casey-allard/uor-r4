//! SHACL test 175: Extended psi-pipeline round-trip.

/// Instance graph for Test 175: psi-pipeline CechNerve -> KanComplex -> HomotopyGroup.
pub const TEST175_HOMOTOPY_PIPELINE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix homology:   <https://uor.foundation/homology/> .
@prefix observable: <https://uor.foundation/observable/> .

homology:ex_kan_175 a owl:NamedIndividual, homology:KanComplex .

homology:ex_trunc_175 a owl:NamedIndividual, homology:PostnikovTruncation ;
    homology:truncationLevel  "1"^^xsd:nonNegativeInteger ;
    homology:truncationSource homology:ex_kan_175 ;
    homology:kInvariant       homology:ex_kinv_175 .

homology:ex_kinv_175 a owl:NamedIndividual, homology:KInvariant ;
    homology:kInvariantTrivial "false"^^xsd:boolean .

observable:ex_pi_175 a owl:NamedIndividual, observable:HomotopyGroup ;
    observable:homotopyDimension "1"^^xsd:nonNegativeInteger .
"#;
