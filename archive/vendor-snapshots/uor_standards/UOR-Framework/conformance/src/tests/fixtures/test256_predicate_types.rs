//! SHACL test 256: `predicate` namespace types.

/// Instance graph for Test 256: Predicate and dispatch types.
pub const TEST256_PREDICATE_TYPES: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix predicate: <https://uor.foundation/predicate/> .

predicate:ex_type_pred_256 a owl:NamedIndividual, predicate:TypePredicate .
predicate:ex_state_pred_256 a owl:NamedIndividual, predicate:StatePredicate .
predicate:ex_site_pred_256 a owl:NamedIndividual, predicate:SitePredicate .
predicate:ex_dispatch_rule_256 a owl:NamedIndividual, predicate:DispatchRule .
predicate:ex_dispatch_table_256 a owl:NamedIndividual, predicate:DispatchTable .
predicate:ex_guarded_trans_256 a owl:NamedIndividual, predicate:GuardedTransition .
predicate:ex_match_arm_256 a owl:NamedIndividual, predicate:MatchArm .
predicate:ex_match_expr_256 a owl:NamedIndividual, predicate:MatchExpression .
"#;
