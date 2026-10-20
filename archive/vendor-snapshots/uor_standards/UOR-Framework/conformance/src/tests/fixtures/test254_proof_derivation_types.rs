//! SHACL test 254: `proof` derivation and strategy types, plus `op:GroupPresentation`.

/// Instance graph for Test 254: Proof strategy, derivation term, and related types.
pub const TEST254_PROOF_DERIVATION_TYPES: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix op:    <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:ex_group_pres_254 a owl:NamedIndividual, op:GroupPresentation .
proof:ex_proof_strategy_254 a owl:NamedIndividual, proof:ProofStrategy .
proof:ex_derivation_term_254 a owl:NamedIndividual, proof:DerivationTerm .
proof:ex_tactic_app_254 a owl:NamedIndividual, proof:TacticApplication .
proof:ex_lemma_inv_254 a owl:NamedIndividual, proof:LemmaInvocation .
proof:ex_induction_step_254 a owl:NamedIndividual, proof:InductionStep .
"#;
