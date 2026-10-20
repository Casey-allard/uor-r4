/// and verificationPathNote.
pub const TEST23_IDENTITY_GROUNDING: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:R_A1 a op:Identity ;
    op:lhs "add(x, add(y, z))" ;
    op:rhs "add(add(x, y), z)" ;
    op:forAll "x, y, z ∈ R_n" ;
    op:verificationDomain op:Enumerative ;
    op:verificationPathNote "Ring associativity — direct computation" .

op:C_1 a op:Identity ;
    op:lhs "pins(compose(A, B))" ;
    op:rhs "pins(A) ∪ pins(B)" ;
    op:forAll "constraints A, B" ;
    op:verificationDomain op:Algebraic ;
    op:verificationPathNote "Constraint composition — set union lemma" .

op:F_1 a op:Identity ;
    op:lhs "pinned site" ;
    op:rhs "cannot be unpinned" ;
    op:forAll "SiteIndex" ;
    op:verificationDomain op:Algebraic ;
    op:verificationPathNote "Site monotonicity — lattice argument" .

op:DC_1 a op:Identity ;
    op:lhs "∂_R f(x)" ;
    op:rhs "f(succ(x)) - f(x)" ;
    op:forAll "f : R_n → R_n, x ∈ R_n" ;
    op:verificationDomain op:Analytical ;
    op:verificationPathNote "Discrete derivative — finite difference definition" .

op:psi_1 a op:Identity ;
    op:lhs "ψ₁(κ_k, constraint_k)" ;
    op:rhs "site binding state" ;
    op:forAll "curvature κ_k, constraint_k" ;
    op:verificationDomain op:Topological ;
    op:verificationPathNote "Curvature → Site — index bridge" .
"#;
