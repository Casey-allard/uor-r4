/// SHACL test 122: Session saturation lifecycle bridge —
/// demonstrates SR_6 and SR_7 identity grounding (Amendment 44).
pub const TEST122_SESSION_GROUNDING: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:SR_6 a owl:NamedIndividual, op:Identity ;
    rdfs:label "SR_6" ;
    op:lhs "freeRank(q) after saturation" ;
    op:rhs "sites of q not in BindingAccumulator" ;
    op:forAll "saturated Session, new RelationQuery q" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .

op:SR_7 a owl:NamedIndividual, op:Identity ;
    rdfs:label "SR_7" ;
    op:lhs "sigma after re-entry with query q" ;
    op:rhs "min(sigma, 1 - freeRank(q)/n)" ;
    op:forAll "SessionResolver, new query q" ;
    op:verificationDomain op:Algebraic ;
    op:universallyValid "true"^^xsd:boolean .
"#;
