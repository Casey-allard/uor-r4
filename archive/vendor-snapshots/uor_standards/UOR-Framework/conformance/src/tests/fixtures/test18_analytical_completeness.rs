/// SHACL test 18: Part III classes and analytical identities.
pub const TEST18_ANALYTICAL_COMPLETENESS: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs:      <http://www.w3.org/2000/01/rdf-schema#> .
@prefix op:        <https://uor.foundation/op/> .
@prefix obs:       <https://uor.foundation/observable/> .
@prefix resolver:  <https://uor.foundation/resolver/> .

obs:Jacobian a owl:Class ;
    rdfs:subClassOf obs:CurvatureObservable .

obs:TopologicalObservable a owl:Class ;
    rdfs:subClassOf obs:Observable .

obs:BettiNumber a owl:Class ;
    rdfs:subClassOf obs:TopologicalObservable .

obs:SpectralGap a owl:Class ;
    rdfs:subClassOf obs:TopologicalObservable .

resolver:CechNerve a owl:Class .

op:DC_1 a op:Identity ;
    op:lhs "∂_R f(x)" ;
    op:rhs "f(succ(x)) - f(x)" ;
    op:forAll "f : R_n → R_n, x ∈ R_n" .

op:HA_1 a op:Identity ;
    op:lhs "N(C)" ;
    op:rhs "simplicial complex on constraints" ;
    op:forAll "constraint set C" .

op:IT_7a a op:Identity ;
    op:lhs "Σ κ_k - χ(N(C))" ;
    op:rhs "= S_residual / ln 2" ;
    op:forAll "constraint configuration C" .

op:IT_7d a op:Identity ;
    op:lhs "resolution complete" ;
    op:rhs "⟺ χ(N(C)) = n and all β_k = 0" ;
    op:forAll "constraint nerve N(C)" .
"#;
