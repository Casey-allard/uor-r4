/// SHACL test 93: SynthesisSignature with realisedEuler, realisedBetti, and
/// achievabilityStatus — Q1 scale (Amendment 39). Validates TS_3 monotonicity.
pub const TEST93_SYNTHESIS_SIGNATURE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_sig_before_93 a owl:NamedIndividual, observable:SynthesisSignature ;
    observable:realisedEuler       "14"^^xsd:integer ;
    observable:realisedBetti       "0"^^xsd:nonNegativeInteger ;
    observable:achievabilityStatus observable:Achievable .

observable:ex_sig_after_93 a owl:NamedIndividual, observable:SynthesisSignature ;
    observable:realisedEuler       "15"^^xsd:integer ;
    observable:realisedBetti       "0"^^xsd:nonNegativeInteger ;
    observable:achievabilityStatus observable:Achievable .
"#;
