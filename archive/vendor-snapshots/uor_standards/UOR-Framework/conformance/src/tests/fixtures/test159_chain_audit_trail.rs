/// SHACL fixture for cert:ChainAuditTrail.
pub const TEST159_CHAIN_AUDIT_TRAIL: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

<urn:test:chain_audit_1> a owl:NamedIndividual , cert:ChainAuditTrail .
"#;
