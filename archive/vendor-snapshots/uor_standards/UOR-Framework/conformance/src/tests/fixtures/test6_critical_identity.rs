//! Test 6: Critical identity — neg(bnot(x)) = succ(x).
//!
//! Validates: `op:criticalIdentity` individual + `proof:CriticalIdentityProof`
//! + `proof:provesIdentity` property linking the proof to the identity.

/// Instance graph for Test 6: Critical identity.
pub const TEST6_CRITICAL_IDENTITY: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix op:     <https://uor.foundation/op/> .
@prefix proof:  <https://uor.foundation/proof/> .

# The critical identity: neg(bnot(x)) = succ(x) for all x ∈ R_n
op:criticalIdentity
    a                   owl:NamedIndividual, op:Identity ;
    op:lhs              op:succ ;
    op:rhs              op:neg ;
    op:forAll           "x ∈ R_n" .

# The proof of the critical identity
<https://uor.foundation/instance/proof-critical-id>
    a                       owl:NamedIndividual, proof:CriticalIdentityProof, proof:Proof ;
    proof:provesIdentity    op:criticalIdentity ;
    proof:witness           <https://uor.foundation/instance/witness-succ> ;
    proof:criticalIdentity  "neg(bnot(x)) = succ(x) for all x in R_n" ;
    proof:verified          true .

# op:succ is the composition
op:succ
    a               owl:NamedIndividual, op:UnaryOp, op:Operation ;
    op:composedOf   ( op:neg op:bnot ) .

op:neg
    a               owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation .

op:bnot
    a               owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation .

# Witness data — linked from the proof via proof:witness
<https://uor.foundation/instance/witness-succ>
    a                       owl:NamedIndividual, proof:WitnessData .
"#;
