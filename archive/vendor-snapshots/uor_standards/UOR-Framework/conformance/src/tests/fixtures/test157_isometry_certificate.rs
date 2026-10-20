/// SHACL fixture for cert:IsometryCertificate.
pub const TEST157_ISOMETRY_CERTIFICATE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

<urn:test:isometry_cert_1> a owl:NamedIndividual , cert:IsometryCertificate ;
    cert:verified "true"^^xsd:boolean .
"#;
