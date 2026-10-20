//! Behavioral contract for the impossibility-witness Certificate impls.
//!
//! Workstream C (v0.2.2 closure, target §4.2): resolver `certify`
//! functions return `Result<Certified<SuccessCert>, Certified<ImpossibilityWitness>>`.
//! The error side requires `ImpossibilityWitness: Certificate`.
//!
//! These tests exercise:
//!
//! 1. `GenericImpossibilityWitness: Certificate` (type-level witness).
//! 2. `InhabitanceImpossibilityWitness: Certificate` (type-level witness).
//! 3. Their IRIs are the canonical `cert:*ImpossibilityCertificate` strings.
//! 4. The `multiplication::certify` exception path (still returning a bare
//!    witness per §4.2 MulContext exemption) exercises the
//!    `GenericImpossibilityWitness::default()` failure construction.
//! 5. A round-trip through the `Debug` + `Display` + `core::error::Error`
//!    surfaces for both witnesses (required for `?` in std consumers).

use uor_foundation::enforcement::{
    resolver, Certificate, GenericImpossibilityWitness, InhabitanceImpossibilityWitness, MulContext,
};
use uor_foundation_test_helpers::{mul_context, Fnv1aHasher16};

// ─── Type-level: both witnesses implement Certificate ───────────────────

const fn assert_certificate<C: Certificate>() {}

#[test]
fn generic_impossibility_witness_implements_certificate() {
    // Compile-time witness: if `Certificate` bound fails, this function
    // fails to compile. The concrete call exercises the impl at link
    // time.
    assert_certificate::<GenericImpossibilityWitness>();
    // IRI pin: must be the `cert:GenericImpossibilityCertificate` class
    // added to the ontology in Workstream C.
    assert_eq!(
        <GenericImpossibilityWitness as Certificate>::IRI,
        "https://uor.foundation/cert/GenericImpossibilityCertificate",
        "GenericImpossibilityWitness IRI must point at the ontology cert class"
    );
}

#[test]
fn inhabitance_impossibility_witness_implements_certificate() {
    assert_certificate::<InhabitanceImpossibilityWitness>();
    assert_eq!(
        <InhabitanceImpossibilityWitness as Certificate>::IRI,
        "https://uor.foundation/cert/InhabitanceImpossibilityCertificate",
    );
}

// ─── Real Err path: multiplication::certify on zero stack budget ────────

#[test]
fn multiplication_certify_emits_witness_on_zero_stack_budget() {
    // The MulContext exemption path (target §4.2) returns a bare
    // `GenericImpossibilityWitness` — but the witness is a Certificate,
    // so downstream can wrap in `Certified<>` through their own paths
    // or match directly on the witness.
    let ctx: MulContext = mul_context(0, false, 4);
    let result = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx);
    assert!(
        result.is_err(),
        "stack_budget_bytes == 0 must return Err(GenericImpossibilityWitness)"
    );
    let witness = result.expect_err("err asserted");
    // Equality: two default witnesses must compare equal.
    assert_eq!(witness, GenericImpossibilityWitness::default());
}

// ─── Witnesses participate in Display / Error chains ────────────────────

#[test]
fn generic_impossibility_witness_displays() {
    use core::fmt::Write;
    let witness = GenericImpossibilityWitness::default();
    let mut buf = String::new();
    write!(&mut buf, "{}", witness).expect("Display must succeed");
    assert!(
        !buf.is_empty(),
        "GenericImpossibilityWitness::Display must produce a non-empty label"
    );
}

#[test]
fn inhabitance_impossibility_witness_displays() {
    use core::fmt::Write;
    let witness = InhabitanceImpossibilityWitness::default();
    let mut buf = String::new();
    write!(&mut buf, "{}", witness).expect("Display must succeed");
    assert!(!buf.is_empty());
}

#[test]
fn witnesses_implement_core_error_trait() {
    // v0.2.2 T6.22: error trait completeness. Both witnesses flow
    // through `Box<dyn core::error::Error>` chains alongside
    // PipelineFailure. Compile-time assertion via trait bound + runtime
    // assertion that `Debug` produces a non-empty label (the Error impl
    // is defaulted on Display/Debug).
    fn require_error<E: core::error::Error>() {}
    require_error::<GenericImpossibilityWitness>();
    require_error::<InhabitanceImpossibilityWitness>();
    let witness = GenericImpossibilityWitness::default();
    assert!(
        !format!("{witness:?}").is_empty(),
        "Debug impl must produce a non-empty label for Error chains"
    );
}
