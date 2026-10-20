/// SHACL test 87: SpectralSequencePage convergence at E2 — Q1 scale (Amendment 39).
pub const TEST87_SPECTRAL_CONVERGENCE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_page1_87 a owl:NamedIndividual, observable:SpectralSequencePage ;
    observable:pageIndex          "1"^^xsd:nonNegativeInteger ;
    observable:differentialIsZero "false"^^xsd:boolean .

observable:ex_page2_87 a owl:NamedIndividual, observable:SpectralSequencePage ;
    observable:pageIndex          "2"^^xsd:nonNegativeInteger ;
    observable:differentialIsZero "true"^^xsd:boolean ;
    observable:convergedAt        "2"^^xsd:nonNegativeInteger .
"#;
