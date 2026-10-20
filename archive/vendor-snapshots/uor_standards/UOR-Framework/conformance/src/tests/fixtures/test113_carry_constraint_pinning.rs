/// SHACL test 113: Carry constraint site-binding map identities —
/// demonstrates CC_PINS and CC_COST_SITE identity grounding (Amendment 44).
pub const TEST113_CARRY_CONSTRAINT_PINNING: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:CC_PINS a owl:NamedIndividual, op:Identity ;
    rdfs:label "CC_PINS" ;
    op:lhs "pinsSites(CarryConstraint(p))" ;
    op:rhs "{k : p(k)=1} union {first-zero(p)}" ;
    op:forAll "bit-pattern p in CarryConstraint" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:CC_COST_SITE a owl:NamedIndividual, op:Identity ;
    rdfs:label "CC_COST_SITE" ;
    op:lhs "|pinsSites(CarryConstraint(p))|" ;
    op:rhs "popcount(p) + 1" ;
    op:forAll "bit-pattern p in CarryConstraint" ;
    op:verificationDomain op:Enumerative ;
    op:universallyValid "true"^^xsd:boolean .

proof:prf_CC_PINS a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:CC_PINS ;
    proof:universalScope "true"^^xsd:boolean .

proof:prf_CC_COST_SITE a owl:NamedIndividual, proof:ComputationCertificate ;
    proof:provesIdentity op:CC_COST_SITE ;
    proof:atWittLevel <https://uor.foundation/schema/W8> .
"#;
