/// SHACL test 92: SynthesisCheckpoint with checkpointStep and checkpointState
/// — Amendment 38 new class exercised at Q1 scale (Amendment 39).
pub const TEST92_SYNTHESIS_CHECKPOINT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

derivation:ex_step_92 a owl:NamedIndividual, derivation:SynthesisStep ;
    derivation:stepIndex      "3"^^xsd:nonNegativeInteger ;
    derivation:signatureAfter observable:ex_sig_92 .

observable:ex_sig_92 a owl:NamedIndividual, observable:SynthesisSignature ;
    observable:realisedEuler "12"^^xsd:integer .

resolver:ex_css_92 a owl:NamedIndividual, resolver:ConstraintSearchState ;
    resolver:exploredCount "42"^^xsd:nonNegativeInteger .

derivation:ex_checkpoint_92 a owl:NamedIndividual, derivation:SynthesisCheckpoint ;
    derivation:checkpointStep  derivation:ex_step_92 ;
    derivation:checkpointState resolver:ex_css_92 .
"#;
