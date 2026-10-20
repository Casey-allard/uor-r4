//! `#![no_std]` compile-time verification for `uor-foundation`.
//!
//! Run with: `cargo test -p uor-foundation --no-default-features --test no_std`.
//!
//! This test intentionally uses only core types and the foundation's
//! `#![no_std]`-compatible surface. If a change to the foundation crate
//! pulls in `std`, this test fails to compile.

#![no_std]

extern crate uor_foundation;

use uor_foundation::enforcement::resolver::{inhabitance, tower_completeness};
use uor_foundation::enforcement::ConstrainedTypeInput;
use uor_foundation::pipeline::{
    decide_horn_sat, decide_two_sat, fragment_classify, ConstrainedTypeShape, ConstraintRef,
    FragmentKind,
};
use uor_foundation_test_helpers::Fnv1aHasher16;

#[test]
fn no_std_pipeline_deciders_are_core_only() {
    // Both deciders accept core slices and return core booleans.
    assert!(decide_two_sat(&[], 0));
    assert!(decide_horn_sat(&[], 0));
    let residue = &[ConstraintRef::Residue {
        modulus: 256,
        residue: 255,
    }];
    assert_eq!(fragment_classify(residue), FragmentKind::Residual);
}

#[test]
fn no_std_resolver_free_functions_reachable() {
    // Phase B: the only verdict surface is the module-per-resolver
    // free-function path. The unit-struct `Certify` façades are deleted;
    // downstream reaches the resolvers through `enforcement::resolver::*`.
    let input = uor_foundation_test_helpers::validated_runtime(ConstrainedTypeInput::default());
    let _ = inhabitance::certify::<_, _, Fnv1aHasher16, 32>(&input);
    let _ = tower_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input);
    let _: &str = <ConstrainedTypeInput as ConstrainedTypeShape>::IRI;
}
