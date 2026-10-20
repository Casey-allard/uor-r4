/// SHACL test 120: Sheaf coefficient ring grounding —
/// demonstrates COEFF_1 identity grounding (Amendment 44).
pub const TEST120_COEFFICIENT_RING: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:COEFF_1 a owl:NamedIndividual, op:Identity ;
    rdfs:label "COEFF_1" ;
    op:lhs "standard coefficient ring for psi-pipeline" ;
    op:rhs "Z/2Z" ;
    op:forAll "CechNerve N(C) at any quantum level" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .
"#;
