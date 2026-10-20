/// SHACL test 118: TypeSynthesis reachability domain completeness —
/// demonstrates TS_8, TS_9, TS_10 identity grounding (Amendment 44).
pub const TEST118_SYNTHESIS_REACHABILITY: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:TS_8 a owl:NamedIndividual, op:Identity ;
    rdfs:label "TS_8" ;
    op:lhs "min constraints for beta_1 = k" ;
    op:rhs "2k + 1" ;
    op:verificationDomain op:Pipeline ;
    op:universallyValid "true"^^xsd:boolean .

op:TS_9 a owl:NamedIndividual, op:Identity ;
    rdfs:label "TS_9" ;
    op:lhs "TypeSynthesisResolver terminates" ;
    op:rhs "within 2^n steps" ;
    op:verificationDomain op:Pipeline ;
    op:universallyValid "true"^^xsd:boolean .

op:TS_10 a owl:NamedIndividual, op:Identity ;
    rdfs:label "TS_10" ;
    op:lhs "ForbiddenSignature(sigma)" ;
    op:rhs "no ConstrainedType with <= n constraints realises sigma" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

proof:prf_TS_8 a owl:NamedIndividual, proof:InductiveProof ;
    proof:provesIdentity op:TS_8 ;
    proof:universalScope "true"^^xsd:boolean ;
    proof:baseCase <https://uor.foundation/proof/prf_HA_1> ;
    proof:validForKAtLeast "1"^^xsd:integer .
"#;
