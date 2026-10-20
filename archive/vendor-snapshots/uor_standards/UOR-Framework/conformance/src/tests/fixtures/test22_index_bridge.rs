/// SHACL test 22: Index bridge — all 12 phi/psi inter-algebra identities with verification domain.
pub const TEST22_INDEX_BRIDGE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:phi_1 a op:Identity ;
    op:lhs "φ₁(neg, ResidueConstraint(m,r))" ;
    op:rhs "ResidueConstraint(m, m-r)" ;
    op:forAll "ring op, constraint" ;
    op:verificationDomain op:Pipeline .

op:phi_2 a op:Identity ;
    op:lhs "φ₂(compose(A,B))" ;
    op:rhs "φ₂(A) ∪ φ₂(B)" ;
    op:forAll "constraints A, B" ;
    op:verificationDomain op:Pipeline .

op:phi_3 a op:Identity ;
    op:lhs "φ₃(closed site state)" ;
    op:rhs "unique 4-component partition" ;
    op:forAll "closed FreeRank" ;
    op:verificationDomain op:Pipeline .

op:phi_4 a op:Identity ;
    op:lhs "φ₄(T, x)" ;
    op:rhs "φ₃(φ₂(φ₁(T, x)))" ;
    op:forAll "T ∈ T_n, x ∈ R_n" ;
    op:verificationDomain op:Pipeline .

op:phi_5 a op:Identity ;
    op:lhs "φ₅(neg)" ;
    op:rhs "preserves d_R, may change d_H" ;
    op:forAll "op ∈ Operation" ;
    op:verificationDomain op:Pipeline .

op:phi_6 a op:Identity ;
    op:lhs "φ₆(state, observables)" ;
    op:rhs "RefinementSuggestion" ;
    op:forAll "ResolutionState" ;
    op:verificationDomain op:Pipeline .

op:psi_1 a op:Identity ;
    op:lhs "ψ₁(κ_k, constraint_k)" ;
    op:rhs "site binding state" ;
    op:forAll "curvature κ_k, constraint_k" ;
    op:verificationDomain op:Topological .

op:psi_2 a op:Identity ;
    op:lhs "ψ₂(β_k)" ;
    op:rhs "homological hole count" ;
    op:forAll "Betti number β_k" ;
    op:verificationDomain op:Topological .

op:psi_3 a op:Identity ;
    op:lhs "ψ₃(Σ κ_k)" ;
    op:rhs "S_residual / ln 2" ;
    op:forAll "curvature sum" ;
    op:verificationDomain op:Topological .

op:psi_4 a op:Identity ;
    op:lhs "ψ₄(χ(N(C)))" ;
    op:rhs "n iff resolution complete" ;
    op:forAll "Euler characteristic of nerve" ;
    op:verificationDomain op:Topological .

op:psi_5 a op:Identity ;
    op:lhs "ψ₅(J_f)" ;
    op:rhs "local curvature field" ;
    op:forAll "Jacobian J_f" ;
    op:verificationDomain op:Topological .

op:psi_6 a op:Identity ;
    op:lhs "ψ₆(∂²)" ;
    op:rhs "0" ;
    op:forAll "boundary operator ∂" ;
    op:verificationDomain op:Topological .
"#;
