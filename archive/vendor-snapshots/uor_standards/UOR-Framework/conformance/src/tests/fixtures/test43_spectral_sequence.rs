/// SHACL test 43: SpectralSequencePage convergence — Amendment 29.
pub const TEST43_SPECTRAL_SEQUENCE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_page1_43 a owl:NamedIndividual, observable:SpectralSequencePage ;
    observable:pageIndex          "1"^^xsd:nonNegativeInteger ;
    observable:differentialIsZero "false"^^xsd:boolean .

observable:ex_page2_43 a owl:NamedIndividual, observable:SpectralSequencePage ;
    observable:pageIndex          "2"^^xsd:nonNegativeInteger ;
    observable:differentialIsZero "true"^^xsd:boolean ;
    observable:convergedAt        "2"^^xsd:nonNegativeInteger .
"#;
