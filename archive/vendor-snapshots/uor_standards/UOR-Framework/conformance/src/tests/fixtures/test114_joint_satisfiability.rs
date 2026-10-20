/// SHACL test 114: Nerve joint satisfiability decision procedure —
/// demonstrates jsat_RR, jsat_CR, jsat_CC identity grounding (Amendment 44).
pub const TEST114_JOINT_SATISFIABILITY: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:jsat_RR a owl:NamedIndividual, op:Identity ;
    rdfs:label "jsat_RR" ;
    op:lhs "jointSat(Res(m1,r1), Res(m2,r2))" ;
    op:rhs "gcd(m1,m2) | (r1 - r2)" ;
    op:forAll "ResidueConstraint pairs over R_n" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:jsat_CR a owl:NamedIndividual, op:Identity ;
    rdfs:label "jsat_CR" ;
    op:lhs "jointSat(Carry(p), Res(m,r))" ;
    op:rhs "pin-site intersection residue-class compatible" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:jsat_CC a owl:NamedIndividual, op:Identity ;
    rdfs:label "jsat_CC" ;
    op:lhs "jointSat(Carry(p1), Carry(p2))" ;
    op:rhs "p1 AND p2 conflict-free" ;
    op:verificationDomain op:Enumerative ;
    op:universallyValid "true"^^xsd:boolean .

proof:prf_jsat_RR a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:jsat_RR ;
    proof:universalScope "true"^^xsd:boolean .
"#;
