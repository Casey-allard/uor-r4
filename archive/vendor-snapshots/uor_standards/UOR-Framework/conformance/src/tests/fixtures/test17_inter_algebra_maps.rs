/// SHACL test 17: Inter-algebra map identities (phi_1 through phi_6).
pub const TEST17_INTER_ALGEBRA_MAPS: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:phi_1 a op:Identity ;
    op:lhs "φ₁(neg, ResidueConstraint(m,r))" ;
    op:rhs "ResidueConstraint(m, m-r)" ;
    op:forAll "ring op, constraint" .

op:phi_2 a op:Identity ;
    op:lhs "φ₂(compose(A,B))" ;
    op:rhs "φ₂(A) ∪ φ₂(B)" ;
    op:forAll "constraints A, B" .

op:phi_3 a op:Identity ;
    op:lhs "φ₃(closed site state)" ;
    op:rhs "unique 4-component partition" ;
    op:forAll "closed FreeRank" .

op:phi_4 a op:Identity ;
    op:lhs "φ₄(T, x)" ;
    op:rhs "φ₃(φ₂(φ₁(T, x)))" ;
    op:forAll "T ∈ T_n, x ∈ R_n" .

op:phi_5 a op:Identity ;
    op:lhs "φ₅(neg)" ;
    op:rhs "preserves d_R, may change d_H" ;
    op:forAll "op ∈ Operation" .

op:phi_6 a op:Identity ;
    op:lhs "φ₆(state, observables)" ;
    op:rhs "RefinementSuggestion" ;
    op:forAll "ResolutionState" .
"#;
