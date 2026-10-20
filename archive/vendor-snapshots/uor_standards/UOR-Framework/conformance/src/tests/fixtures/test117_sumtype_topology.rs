/// SHACL test 117: SumType topological identity algebra —
/// demonstrates ST_3, ST_4, ST_5 identity grounding (Amendment 44).
pub const TEST117_SUMTYPE_TOPOLOGY: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:ST_3 a owl:NamedIndividual, op:Identity ;
    rdfs:label "ST_3" ;
    op:lhs "chi(N(C(A+B)))" ;
    op:rhs "chi(N(C(A))) + chi(N(C(B)))" ;
    op:forAll "disjoint SumType A + B" ;
    op:verificationDomain op:IndexTheoretic ;
    op:universallyValid "true"^^xsd:boolean .

op:ST_4 a owl:NamedIndividual, op:Identity ;
    rdfs:label "ST_4" ;
    op:lhs "beta_k(N(C(A+B)))" ;
    op:rhs "beta_k(N(C(A))) + beta_k(N(C(B)))" ;
    op:forAll "disjoint SumType A + B, k >= 0" ;
    op:verificationDomain op:Topological ;
    op:universallyValid "true"^^xsd:boolean .

op:ST_5 a owl:NamedIndividual, op:Identity ;
    rdfs:label "ST_5" ;
    op:lhs "CompleteType(A + B)" ;
    op:rhs "CompleteType(A) and CompleteType(B) and Q(A)=Q(B)" ;
    op:forAll "SumType A + B" ;
    op:verificationDomain op:IndexTheoretic ;
    op:universallyValid "true"^^xsd:boolean .
"#;
