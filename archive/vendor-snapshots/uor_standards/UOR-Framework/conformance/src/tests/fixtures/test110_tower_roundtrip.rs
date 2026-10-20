/// SHACL test 110: Full tower round-trip — a complete lift from Q0 to Q3
/// with resolver, certificate, spectral page, and proof, demonstrating
/// end-to-end tower integrity.
pub const TEST110_TOWER_ROUNDTRIP: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix cert:   <https://uor.foundation/cert/> .
@prefix proof:  <https://uor.foundation/proof/> .
@prefix resolver: <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. WittLift Q0→Q3
type:ex_lift_110 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel   schema:Q0 ;
    type:targetLevel   schema:Q3 ;
    type:liftedType    type:ex_flat_110 ;
    type:isFlat        "true"^^xsd:boolean .

type:ex_flat_110 a owl:NamedIndividual, type:FlatType ;
    rdfs:label "Round-trip flat type" .

# 2. Spectral sequence page confirming convergence
observable:ex_page_110 a owl:NamedIndividual, observable:SpectralSequencePage ;
    rdfs:label "E2 page converged" .

# 3. Resolver
resolver:ex_resolver_110 a owl:NamedIndividual, resolver:IncrementalCompletenessResolver ;
    resolver:resolvedLift type:ex_lift_110 ;
    rdfs:label "Tower round-trip resolver" .

# 4. Completeness certificate
cert:ex_cert_110 a owl:NamedIndividual, cert:CompletenessAuditTrail ;
    cert:certifiedType  type:ex_flat_110 ;
    rdfs:label "Round-trip certificate" .

# 5. Proof of tower integrity
proof:ex_proof_110 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:atWittLevel schema:Q3 ;
    proof:universalScope "true"^^xsd:boolean ;
    rdfs:label "Tower round-trip proof" .
"#;
