/// SHACL test 121: GluingObstruction resolver feedback —
/// demonstrates GO_1 identity grounding (Amendment 44).
pub const TEST121_GLUING_FEEDBACK: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:GO_1 a owl:NamedIndividual, op:Identity ;
    rdfs:label "GO_1" ;
    op:lhs "pinsSites(killing constraint for obstruction c)" ;
    op:rhs "superset of pinsSites(C_i) cap pinsSites(C_j)" ;
    op:forAll "GluingObstruction c, cycle pair (C_i, C_j)" ;
    op:verificationDomain op:Topological ;
    op:universallyValid "true"^^xsd:boolean .
"#;
