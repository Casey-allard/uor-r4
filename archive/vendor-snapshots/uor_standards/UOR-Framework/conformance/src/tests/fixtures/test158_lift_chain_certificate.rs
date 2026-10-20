/// SHACL fixture for cert:LiftChainCertificate.
pub const TEST158_LIFT_CHAIN_CERTIFICATE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

<urn:test:lift_chain_cert_1> a owl:NamedIndividual , cert:LiftChainCertificate ;
    cert:verified "true"^^xsd:boolean ;
    cert:chainStepCount "3"^^xsd:nonNegativeInteger .
"#;
