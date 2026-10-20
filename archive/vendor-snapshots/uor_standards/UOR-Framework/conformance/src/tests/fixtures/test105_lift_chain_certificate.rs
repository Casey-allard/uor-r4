/// SHACL test 105: LiftChainCertificate — a completeness certificate
/// certifying that a flat lift chain is valid end-to-end.
pub const TEST105_LIFT_CHAIN_CERTIFICATE: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix cert:   <https://uor.foundation/cert/> .

# Certified lift chain
type:ex_lift_105 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel   schema:Q0 ;
    type:targetLevel   schema:Q3 ;
    type:liftedType    type:ex_flat_105 ;
    type:isFlat        "true"^^xsd:boolean .

type:ex_flat_105 a owl:NamedIndividual, type:FlatType ;
    rdfs:label "Certified flat type" .

# Completeness certificate for the lift chain
cert:ex_cert_105 a owl:NamedIndividual, cert:CompletenessAuditTrail ;
    cert:certifiedType  type:ex_flat_105 ;
    rdfs:label "Lift chain certificate Q0-Q3" .
"#;
