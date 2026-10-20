/// SHACL test 109: Identity with validityKind — an op:Identity individual
/// demonstrating the validityKind annotation on a universally valid identity.
pub const TEST109_VALIDITY_SCOPE: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix op:     <https://uor.foundation/op/> .
@prefix proof:  <https://uor.foundation/proof/> .

# Identity with validity scope annotation
op:ex_identity_109 a owl:NamedIndividual, op:Identity ;
    op:universallyValid  "true"^^xsd:boolean ;
    rdfs:label "Universally valid identity" .

# Proof covering the identity
proof:ex_proof_109 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:atWittLevel schema:Q0 ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:criticalIdentity op:ex_identity_109 ;
    rdfs:label "Proof of universal validity" .
"#;
