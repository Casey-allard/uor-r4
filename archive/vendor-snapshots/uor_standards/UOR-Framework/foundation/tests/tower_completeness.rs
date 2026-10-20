//! Phase B: TowerCompleteness / IncrementalCompleteness free-function verdicts.
//!
//! The v0.2.1 `Resolver::new().certify(&input)` unit-struct path is deleted
//! per target §4.1 W12. The only verdict surface is the module-per-resolver
//! free-function path:
//!
//! ```rust,ignore
//! use uor_foundation::enforcement::resolver::tower_completeness;
//! let validated = uor_foundation_test_helpers::validated_runtime(my_input);
//! let verdict = tower_completeness::certify::<_, _, MyHasher>(&validated)?;
//! ```
//!
//! v0.2.2 closure (target §4.2): resolvers consume `&Validated<T, P>` and
//! return `Result<Certified<C>, Certified<ImpossibilityWitness>>`.

use uor_foundation::enforcement::resolver::{incremental_completeness, tower_completeness};
use uor_foundation::enforcement::ConstrainedTypeInput;
use uor_foundation::WittLevel;
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16};

#[test]
fn tower_completeness_certifies_vacuous_input_via_free_function() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let result = tower_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input);
    assert!(result.is_ok(), "vacuous input must certify");
}

#[test]
fn incremental_completeness_certify_signature() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let result = incremental_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input);
    assert!(result.is_ok());
}

#[test]
fn tower_completeness_certify_at_w16_returns_w16_target_level() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let cert = tower_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W16)
        .expect("w16 certifies");
    assert_eq!(cert.certificate().target_level().witt_length(), 16);
}

#[test]
fn tower_completeness_certify_at_w24_returns_w24_target_level() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let cert =
        tower_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::new(24))
            .expect("w24 certifies");
    assert_eq!(cert.certificate().target_level().witt_length(), 24);
}

#[test]
fn tower_completeness_certify_default_uses_w32() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let cert =
        tower_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("w32 certifies");
    assert_eq!(cert.certificate().target_level().witt_length(), 32);
}
