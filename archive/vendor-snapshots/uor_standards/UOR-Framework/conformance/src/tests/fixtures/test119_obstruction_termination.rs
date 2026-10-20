/// SHACL test 119: ObstructionChain termination guarantee —
/// demonstrates QT_8 and QT_9 identity grounding (Amendment 44).
pub const TEST119_OBSTRUCTION_TERMINATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:QT_8 a owl:NamedIndividual, op:Identity ;
    rdfs:label "QT_8" ;
    op:lhs "ObstructionChain length from Q_j to Q_k" ;
    op:rhs "<= (k-j) * C(basisSize(Q_j), 3)" ;
    op:verificationDomain op:IndexTheoretic ;
    op:universallyValid "true"^^xsd:boolean .

op:QT_9 a owl:NamedIndividual, op:Identity ;
    rdfs:label "QT_9" ;
    op:lhs "TowerCompletenessResolver terminates" ;
    op:rhs "within QT_8 bound" ;
    op:verificationDomain op:Pipeline ;
    op:universallyValid "true"^^xsd:boolean .
"#;
