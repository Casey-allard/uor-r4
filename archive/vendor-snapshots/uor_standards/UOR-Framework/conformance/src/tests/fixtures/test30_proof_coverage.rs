/// SHACL test 30: Proof coverage — ComputationCertificate and AxiomaticDerivation proof individuals.
pub const TEST30_PROOF_COVERAGE: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix op:     <https://uor.foundation/op/> .
@prefix proof:  <https://uor.foundation/proof/> .
@prefix schema: <https://uor.foundation/schema/> .

# CriticalIdentityProof (subclass of ComputationCertificate)
proof:prf_criticalIdentity a owl:NamedIndividual, proof:CriticalIdentityProof ;
    proof:provesIdentity op:criticalIdentity ;
    proof:atWittLevel schema:Q0 ;
    proof:verified       "true"^^xsd:boolean .

# Axiomatic dual for criticalIdentity
proof:prf_criticalIdentity_axiomatic a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:criticalIdentity ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:verified       "true"^^xsd:boolean .

# ComputationCertificate example
proof:prf_phi_1 a owl:NamedIndividual, proof:ComputationCertificate ;
    proof:provesIdentity op:phi_1 ;
    proof:atWittLevel schema:Q0 ;
    proof:verified       "true"^^xsd:boolean .

# AxiomaticDerivation example
proof:prf_R_A1 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:R_A1 ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:verified       "true"^^xsd:boolean .

# Amendment 30: Monodromy proof individuals (MN_1 and MN_7 boundary cases)
proof:prf_MN_1 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:MN_1 ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:verified       "true"^^xsd:boolean .

proof:prf_MN_7 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:MN_7 ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:verified       "true"^^xsd:boolean .
"#;
