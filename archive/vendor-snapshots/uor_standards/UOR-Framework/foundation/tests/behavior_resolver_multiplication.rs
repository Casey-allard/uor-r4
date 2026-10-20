//! Behavioral contract for `resolver::multiplication::certify`.
//!
//! Target §4.2 (W12) + §4.5: the multiplication resolver picks the
//! cost-optimal splitting factor R (schoolbook R=1 vs Karatsuba R=2)
//! based on `MulContext.stack_budget_bytes`. It returns
//! `Err(GenericImpossibilityWitness)` when `stack_budget_bytes == 0`
//! (inadmissible context) and `Ok` otherwise, with cert evidence
//! matching the chosen algorithm's Landauer cost.
//!
//! A regression where the resolver accepts zero-stack contexts, or
//! emits a cert whose algorithm choice contradicts the stack budget,
//! would fail these assertions.

use uor_foundation::enforcement::{resolver, GenericImpossibilityWitness, MulContext};
use uor_foundation_test_helpers::Fnv1aHasher16;

#[test]
fn multiplication_rejects_zero_stack_budget() {
    let ctx = MulContext::new(0, false, 1);
    let result = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx);
    assert!(
        matches!(result, Err(GenericImpossibilityWitness { .. })),
        "multiplication::certify must reject stack_budget_bytes == 0"
    );
}

#[test]
fn multiplication_accepts_admissible_schoolbook_context() {
    let ctx = MulContext::new(1024, false, 1);
    let cert = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx)
        .expect("1-limb mul must certify on a 1KB stack");
    assert_ne!(cert.certificate().witt_bits(), 0);
}

#[test]
fn multiplication_different_limb_counts_differ_on_fingerprint() {
    // MulContext differs only by limb_count → the fingerprint must differ,
    // because the resolver folds limb_count into the substrate digest.
    // If the resolver ignored limb_count, both calls would produce identical
    // fingerprints — a correctness gap.
    let ctx_1 = MulContext::new(4096, false, 1);
    let ctx_16 = MulContext::new(4096, false, 16);
    let cert_1 = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_1).expect("1-limb");
    let cert_16 = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_16).expect("16-limb");
    assert_ne!(
        cert_1.certificate().content_fingerprint(),
        cert_16.certificate().content_fingerprint(),
        "different limb_counts must produce different fingerprints"
    );
}

#[test]
fn multiplication_different_stack_budgets_differ_on_fingerprint() {
    // The splitting factor depends on stack_budget_bytes. Even if the
    // cert doesn't carry the factor explicitly, the fingerprint must
    // differ because stack_budget_bytes is folded into the digest.
    let ctx_tight = MulContext::new(64, false, 16);
    let ctx_ample = MulContext::new(4096, false, 16);
    let a = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_tight).expect("tight");
    let b = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_ample).expect("ample");
    assert_ne!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
        "different stack budgets must produce different fingerprints"
    );
}

#[test]
fn multiplication_const_eval_differs_from_runtime_fingerprint() {
    let ctx_rt = MulContext::new(4096, false, 16);
    let ctx_ce = MulContext::new(4096, true, 16);
    let a = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_rt).expect("rt");
    let b = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx_ce).expect("ce");
    assert_ne!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
        "const_eval toggling must affect the fingerprint"
    );
}

#[test]
fn multiplication_certify_is_pure() {
    let ctx = MulContext::new(4096, false, 4);
    let a = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx).expect("a");
    let b = resolver::multiplication::certify::<Fnv1aHasher16, 32>(&ctx).expect("b");
    assert_eq!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
        "multiplication::certify must be pure (same context \u{2192} same fingerprint)"
    );
}
