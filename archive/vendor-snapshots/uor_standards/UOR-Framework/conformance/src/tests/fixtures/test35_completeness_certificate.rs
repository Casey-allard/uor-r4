/// SHACL test 35: CompletenessCertificate with audit trail — Amendment 25.
pub const TEST35_COMPLETENESS_CERTIFICATE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_certificate_35 a owl:NamedIndividual, cert:CompletenessCertificate ;
    cert:certifiedType  type:ex_complete_35 ;
    cert:verified       "true"^^xsd:boolean ;
    cert:auditTrail     cert:ex_trail_35 .

type:ex_complete_35 a owl:NamedIndividual, type:CompleteType .

cert:ex_trail_35 a owl:NamedIndividual, cert:CompletenessAuditTrail ;
    cert:witnessCount "5"^^xsd:nonNegativeInteger .
"#;
