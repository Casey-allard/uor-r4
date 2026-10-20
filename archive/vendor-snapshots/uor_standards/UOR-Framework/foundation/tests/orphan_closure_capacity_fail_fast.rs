//! Phase 1a test: `primitive_simplicial_nerve_betti` returns
//! `NERVE_CAPACITY_EXCEEDED` when inputs exceed the stack-fittable caps,
//! rather than silently truncating.

use uor_foundation::enforcement::{
    primitive_simplicial_nerve_betti, GenericImpossibilityWitness, NERVE_CONSTRAINTS_CAP,
    NERVE_SITES_CAP,
};
use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef};

/// 9 constraints — one over the cap. Capacity-guard must fail fast.
struct TooManyConstraints;

impl ConstrainedTypeShape for TooManyConstraints {
    const IRI: &'static str = "https://test.local/TooManyConstraints";
    const SITE_COUNT: usize = 4;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Site { position: 3 },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Site { position: 3 },
        ConstraintRef::Site { position: 0 },
    ];
    const CYCLE_SIZE: u64 = 1;
}

/// 9 sites — one over the cap.
struct TooManySites;

impl ConstrainedTypeShape for TooManySites {
    const IRI: &'static str = "https://test.local/TooManySites";
    const SITE_COUNT: usize = 9;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Site { position: 0 }];
    const CYCLE_SIZE: u64 = 1;
}

/// Exactly at the caps — must still succeed.
struct AtCaps;

impl ConstrainedTypeShape for AtCaps {
    const IRI: &'static str = "https://test.local/AtCaps";
    const SITE_COUNT: usize = 8;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Site { position: 3 },
        ConstraintRef::Site { position: 4 },
        ConstraintRef::Site { position: 5 },
        ConstraintRef::Site { position: 6 },
        ConstraintRef::Site { position: 7 },
    ];
    const CYCLE_SIZE: u64 = 1;
}

fn has_identity(err: &GenericImpossibilityWitness, expected: &str) -> bool {
    err.identity() == Some(expected)
}

// Compile-time sanity: the test shapes do exceed the caps (otherwise
// the capacity-exceeded branch wouldn't be triggered at runtime).
const _: () = {
    assert!(TooManyConstraints::CONSTRAINTS.len() > NERVE_CONSTRAINTS_CAP);
    assert!(TooManySites::SITE_COUNT > NERVE_SITES_CAP);
};

#[test]
fn oversized_constraints_fails_fast() {
    let result = primitive_simplicial_nerve_betti::<TooManyConstraints>();
    match result {
        Ok(_) => panic!("expected NERVE_CAPACITY_EXCEEDED, got Ok"),
        Err(w) => assert!(
            has_identity(&w, "NERVE_CAPACITY_EXCEEDED"),
            "identity was {:?}",
            w.identity()
        ),
    }
}

#[test]
fn oversized_sites_fails_fast() {
    let result = primitive_simplicial_nerve_betti::<TooManySites>();
    match result {
        Ok(_) => panic!("expected NERVE_CAPACITY_EXCEEDED, got Ok"),
        Err(w) => assert!(
            has_identity(&w, "NERVE_CAPACITY_EXCEEDED"),
            "identity was {:?}",
            w.identity()
        ),
    }
}

#[test]
fn at_caps_succeeds() {
    assert_eq!(AtCaps::SITE_COUNT, NERVE_SITES_CAP);
    assert_eq!(AtCaps::CONSTRAINTS.len(), NERVE_CONSTRAINTS_CAP);
    let result = primitive_simplicial_nerve_betti::<AtCaps>();
    assert!(result.is_ok(), "at-caps input should succeed");
}
