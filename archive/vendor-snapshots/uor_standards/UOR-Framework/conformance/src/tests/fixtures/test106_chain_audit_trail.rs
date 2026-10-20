/// SHACL test 106: ChainAuditTrail — an audit trail recording the
/// completeness resolution steps across multiple quantum levels.
pub const TEST106_CHAIN_AUDIT_TRAIL: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix cert:   <https://uor.foundation/cert/> .
@prefix resolver: <https://uor.foundation/resolver/> .

# Completeness candidate under audit
type:ex_candidate_106 a owl:NamedIndividual, type:CompletenessCandidate ;
    rdfs:label "Candidate under chain audit" .

# Audit trail with multi-level resolution
cert:ex_audit_106 a owl:NamedIndividual, cert:CompletenessAuditTrail ;
    cert:certifiedType  type:ex_candidate_106 ;
    rdfs:label "Chain audit trail Q0-Q2" .

# Resolver that produced the audit
resolver:ex_resolver_106 a owl:NamedIndividual, resolver:CompletenessResolver ;
    rdfs:label "Resolver for chain audit" .
"#;
