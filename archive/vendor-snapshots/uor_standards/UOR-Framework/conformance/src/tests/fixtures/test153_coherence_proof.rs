/// SHACL fixture for proof:CoherenceProof.
pub const TEST153_COHERENCE_PROOF: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix proof: <https://uor.foundation/proof/> .

<urn:test:coherence_proof_1> a owl:NamedIndividual , proof:CoherenceProof ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:verified "true"^^xsd:boolean .
"#;
