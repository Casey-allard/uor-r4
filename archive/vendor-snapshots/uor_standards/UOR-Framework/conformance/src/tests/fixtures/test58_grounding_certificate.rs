/// SHACL test 58: Grounding certificate — GroundingCertificate with
/// certifiedGrounding, groundingWitness (Amendment 33).
pub const TEST58_GROUNDING_CERTIFICATE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix state:      <https://uor.foundation/state/> .

# 1. GroundingCertificate
cert:ex_scert_58 a owl:NamedIndividual, cert:GroundingCertificate ;
    cert:certifiedGrounding state:ex_sc_58 ;
    cert:groundingWitness state:ex_sw_58 .

# 2. Referenced GroundedContext and GroundingWitness
state:ex_sc_58 a owl:NamedIndividual, state:GroundedContext .
state:ex_sw_58 a owl:NamedIndividual, state:GroundingWitness .
"#;
