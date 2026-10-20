/// SHACL test 116: Constraint language expressiveness boundary —
/// demonstrates EXP_1, EXP_2, EXP_3 identity grounding (Amendment 44).
pub const TEST116_CONSTRAINT_EXPRESSIVENESS: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:EXP_1 a owl:NamedIndividual, op:Identity ;
    rdfs:label "EXP_1" ;
    op:lhs "carrier(C) is monotone" ;
    op:rhs "all residues of C = modulus - 1, no Carry/Depth" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:EXP_2 a owl:NamedIndividual, op:Identity ;
    rdfs:label "EXP_2" ;
    op:lhs "count of monotone ConstrainedTypes at Q_n" ;
    op:rhs "2^n" ;
    op:verificationDomain op:Enumerative ;
    op:universallyValid "true"^^xsd:boolean .

op:EXP_3 a owl:NamedIndividual, op:Identity ;
    rdfs:label "EXP_3" ;
    op:lhs "carrier(SumType(A,B))" ;
    op:rhs "coproduct(carrier(A), carrier(B))" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .
"#;
