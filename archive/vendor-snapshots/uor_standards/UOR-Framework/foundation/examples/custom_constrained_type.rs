//! v0.2.2 Phase Q.3 example: downstream defines a custom `ConstrainedTypeShape`,
//! admits it through the foundation's runtime admission path, then exercises
//! a Phase D resolver against it.
//!
//! This is the canonical flow for a downstream author who wants to carry their
//! own type through the reduction pipeline:
//! 1. Impl `ConstrainedTypeShape` on a zero-sized marker type, declaring the
//!    `(IRI, SITE_COUNT, CONSTRAINTS)` triple.
//! 2. Call `validate_constrained_type(shape)` to get a `Validated<Shape, Runtime>`.
//! 3. Feed the validated shape to a Phase D resolver's `certify`/`certify_at`.
//!
//! The admission function runs `preflight_feasibility` and
//! `preflight_package_coherence` before wrapping the shape in `Validated<_>`,
//! catching unsatisfiable constraint systems at the admission boundary.

use uor_foundation::enforcement::resolver;
use uor_foundation::pipeline::{validate_constrained_type, ConstrainedTypeShape, ConstraintRef};
use uor_foundation_test_helpers::Fnv1aHasher16;

/// A downstream-declared constrained type: 4 sites, residue-3-mod-7 constraint.
///
/// Sealing lives on `Validated<T>` / `Grounded<'static, T>` construction — downstream
/// is free to impl `ConstrainedTypeShape` but cannot mint `Validated<Self>`
/// except through a foundation admission function.
pub struct ModSeven4Site;

impl ConstrainedTypeShape for ModSeven4Site {
    const IRI: &'static str = "https://example.org/ModSeven4Site";
    const SITE_COUNT: usize = 4;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Residue {
        modulus: 7,
        residue: 3,
    }];
    const CYCLE_SIZE: u64 = 1;
}

/// A second shape with distinct structure — demonstrates input-variation
/// oracle discrimination on the Phase J primitives.
pub struct HammingBounded;

impl ConstrainedTypeShape for HammingBounded {
    const IRI: &'static str = "https://example.org/HammingBounded";
    const SITE_COUNT: usize = 8;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Hamming { bound: 1024 }];
    const CYCLE_SIZE: u64 = 1;
}

fn main() {
    // Admit both shapes through the foundation's runtime admission path.
    let mod7 = validate_constrained_type(ModSeven4Site).expect("ModSeven4Site is admissible");
    let hamm = validate_constrained_type(HammingBounded).expect("HammingBounded is admissible");

    // Run each through the homotopy resolver at the canonical W32 level.
    // homotopy composes SimplicialNerve, folding Betti numbers of the
    // constraint nerve into the content fingerprint. Different (site_count,
    // constraint_count) tuples produce different Betti tuples → different
    // fingerprints.
    let cert_mod7 = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&mod7)
        .expect("homotopy certifies ModSeven4Site");
    let cert_hamm = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&hamm)
        .expect("homotopy certifies HammingBounded");

    assert_ne!(
        cert_mod7.certificate().content_fingerprint(),
        cert_hamm.certificate().content_fingerprint(),
        "distinct shapes must yield distinct fingerprints"
    );

    println!(
        "ModSeven4Site homotopy fingerprint width: {} bytes",
        cert_mod7.certificate().content_fingerprint().width_bytes()
    );
    println!(
        "HammingBounded homotopy fingerprint width: {} bytes",
        cert_hamm.certificate().content_fingerprint().width_bytes()
    );

    // Invalid shape: residue >= modulus violates feasibility.
    /// A deliberately-invalid shape used to demonstrate admission rejection.
    pub struct Invalid;
    impl ConstrainedTypeShape for Invalid {
        const IRI: &'static str = "https://example.org/Invalid";
        const SITE_COUNT: usize = 1;
        const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Residue {
            modulus: 5,
            residue: 10,
        }];
        const CYCLE_SIZE: u64 = 1;
    }

    match validate_constrained_type(Invalid) {
        Ok(_) => panic!("expected admission rejection"),
        Err(violation) => {
            println!(
                "Invalid shape rejected as expected: constraint_iri={}",
                violation.constraint_iri
            );
        }
    }
}
