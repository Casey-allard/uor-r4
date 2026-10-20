//! Phase B: InhabitanceResolver free-function path and dispatch-table tests.
//!
//! The v0.2.1 `Resolver::new().certify(&input)` unit-struct path is deleted.
//! The free-function path `enforcement::resolver::inhabitance::certify(...)` is
//! the only verdict surface.

use uor_foundation::enforcement::resolver::inhabitance;
use uor_foundation::enforcement::{
    ConstrainedTypeInput, InhabitanceImpossibilityWitness, INHABITANCE_DISPATCH_TABLE,
};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16};

#[test]
fn inhabitance_dispatch_table_has_three_rules() {
    assert_eq!(INHABITANCE_DISPATCH_TABLE.len(), 3);
    let priorities: Vec<u32> = INHABITANCE_DISPATCH_TABLE
        .iter()
        .map(|r| r.priority)
        .collect();
    assert_eq!(priorities, vec![0, 1, 2]);
}

#[test]
fn inhabitance_dispatch_predicate_iris_match_ontology() {
    let preds: Vec<&str> = INHABITANCE_DISPATCH_TABLE
        .iter()
        .map(|r| r.predicate_iri)
        .collect();
    assert!(preds.contains(&"https://uor.foundation/predicate/Is2SatShape"));
    assert!(preds.contains(&"https://uor.foundation/predicate/IsHornShape"));
    assert!(preds.contains(&"https://uor.foundation/predicate/IsResidualFragment"));
}

#[test]
fn inhabitance_dispatch_target_resolvers_match_ontology() {
    let targets: Vec<&str> = INHABITANCE_DISPATCH_TABLE
        .iter()
        .map(|r| r.target_resolver_iri)
        .collect();
    assert!(targets.contains(&"https://uor.foundation/resolver/TwoSatDecider"));
    assert!(targets.contains(&"https://uor.foundation/resolver/HornSatDecider"));
    assert!(targets.contains(&"https://uor.foundation/resolver/ResidualVerdictResolver"));
}

#[test]
fn inhabitance_resolver_vacuous_satisfies_empty_input() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let result = inhabitance::certify::<_, _, Fnv1aHasher16, 32>(&input);
    assert!(
        result.is_ok(),
        "inhabitance::certify must return Ok for vacuous (no-constraint) inputs"
    );
    let _unused: InhabitanceImpossibilityWitness = InhabitanceImpossibilityWitness::default();
}
