/// SHACL test 16: Representative algebraic identities from each algebra.
pub const TEST16_ALGEBRAIC_IDENTITIES: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

op:R_A1 a op:Identity ;
    op:lhs "add(x, add(y, z))" ;
    op:rhs "add(add(x, y), z)" ;
    op:forAll "x, y, z ∈ R_n" .

op:B_1 a op:Identity ;
    op:lhs "xor(x, xor(y, z))" ;
    op:rhs "xor(xor(x, y), z)" ;
    op:forAll "x, y, z ∈ R_n" .

op:C_1 a op:Identity ;
    op:lhs "pins(compose(A, B))" ;
    op:rhs "pins(A) ∪ pins(B)" ;
    op:forAll "constraints A, B" .

op:F_1 a op:Identity ;
    op:lhs "pinned site" ;
    op:rhs "cannot be unpinned" ;
    op:forAll "SiteIndex" .

op:RE_1 a op:Identity ;
    op:lhs "Π_D(T)" ;
    op:rhs "Π_C(T) = Π_E(T)" ;
    op:forAll "T ∈ T_n" .

op:OB_M1 a op:Identity ;
    op:lhs "d_R(x, z)" ;
    op:rhs "≤ d_R(x, y) + d_R(y, z)" ;
    op:forAll "x, y, z ∈ R_n" .

op:T_C1 a op:Identity ;
    op:lhs "compose(id, f)" ;
    op:rhs "f" ;
    op:forAll "f ∈ Transform" .

op:AD_1 a op:Identity ;
    op:lhs "addresses(glyph(d))" ;
    op:rhs "d" ;
    op:forAll "d ∈ R_n" .

op:TH_1 a op:Identity ;
    op:lhs "S(state)" ;
    op:rhs "freeRank × ln 2" ;
    op:forAll "state ∈ FreeRank" .

op:CA_1 a op:Identity ;
    op:lhs "add(x,y)_k" ;
    op:rhs "xor(x_k, xor(y_k, c_k(x,y)))" ;
    op:forAll "x, y ∈ R_n, 0 ≤ k < n" .
"#;
