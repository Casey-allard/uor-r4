/// SHACL test 24: Verification domain vocabulary — typed verification properties.
pub const TEST24_VERIFICATION_DOMAIN: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Vocabulary individuals
op:Enumerative a op:VerificationDomain .
op:Algebraic a op:VerificationDomain .
op:Geometric a op:VerificationDomain .
op:Analytical a op:VerificationDomain .
op:Thermodynamic a op:VerificationDomain .
op:Topological a op:VerificationDomain .
op:Pipeline a op:VerificationDomain .
op:IndexTheoretic a op:VerificationDomain .


# Spot-check: R_A1 (verifiable, Enumerative)
op:R_A1 a op:Identity ;
    op:lhs "add(x, add(y, z))" ;
    op:rhs "add(add(x, y), z)" ;
    op:forAll "x, y, z ∈ R_n" ;
    op:verificationDomain op:Enumerative .

# Spot-check: TH_1 (derivable, Thermodynamic)
op:TH_1 a op:Identity ;
    op:lhs "S(site_state)" ;
    op:rhs "free_count × ln(2)" ;
    op:forAll "site state" ;
    op:verificationDomain op:Thermodynamic .

# Spot-check: IT_7a (derivable, 3 domains)
op:IT_7a a op:Identity ;
    op:lhs "Σ κ_k - χ(N(C))" ;
    op:rhs "= S_residual / ln 2" ;
    op:forAll "constraint configuration C" ;
    op:verificationDomain op:IndexTheoretic, op:Analytical, op:Topological .

# Spot-check: phi_1 (verifiable, Pipeline)
op:phi_1 a op:Identity ;
    op:lhs "φ₁(neg, ResidueConstraint(m,r))" ;
    op:rhs "ResidueConstraint(m, m-r)" ;
    op:forAll "ring op, constraint" ;
    op:verificationDomain op:Pipeline .
"#;
