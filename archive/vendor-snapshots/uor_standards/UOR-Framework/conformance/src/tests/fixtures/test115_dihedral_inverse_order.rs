/// SHACL test 115: Dihedral inverse and order identities —
/// demonstrates D_8 and D_9 identity grounding (Amendment 44).
pub const TEST115_DIHEDRAL_INVERSE_ORDER: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:D_8 a owl:NamedIndividual, op:Identity ;
    rdfs:label "D_8" ;
    op:lhs "(r^a s^p)^(-1)" ;
    op:rhs "r^(-(−1)^p a mod 2^n) s^p" ;
    op:forAll "a in 0..2^n, p in {0,1}" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:D_9 a owl:NamedIndividual, op:Identity ;
    rdfs:label "D_9" ;
    op:lhs "ord(r^k s^1)" ;
    op:rhs "2" ;
    op:forAll "k in Z/(2^n)Z" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

proof:prf_D_8 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:D_8 ;
    proof:universalScope "true"^^xsd:boolean .

proof:prf_D_9 a owl:NamedIndividual, proof:AxiomaticDerivation ;
    proof:provesIdentity op:D_9 ;
    proof:universalScope "true"^^xsd:boolean .
"#;
