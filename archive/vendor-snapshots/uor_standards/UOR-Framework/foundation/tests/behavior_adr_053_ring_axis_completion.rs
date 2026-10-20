//! ADR-053 ring-axis completion: Div / Mod / Pow primitives at every
//! shipped WittLevel.
//!
//! Asserts the const-fn `const_ring_eval_w{n}` helpers compute Euclidean
//! division, remainder, and modular exponentiation correctly under
//! `Z/(2^bits)Z` semantics. This file complements
//! `behavior_const_ring_eval.rs` which pins the original Ring/Hypercube
//! axis primitives.
//!
//! ADR-050 width-parametric kernel: the same fold-rule must hold across
//! native-backed widths (W8 through W128) and Limbs-backed widths
//! (W160 through W32768). Cross-width truncation is forbidden — any
//! regression where Div, Mod, or Pow narrows the operand to u64 fails
//! the cross-Witt invariants below.

use uor_foundation::enforcement::{
    const_ring_eval_w128, const_ring_eval_w16, const_ring_eval_w32, const_ring_eval_w64,
    const_ring_eval_w8,
};
use uor_foundation::PrimitiveOp;

// ── ADR-053 DV_1: div right-identity ─────────────────────────────────

#[test]
fn dv1_div_right_identity_holds_at_native_widths() {
    for &x in &[0u8, 1, 7, 42, 100, 255] {
        assert_eq!(
            const_ring_eval_w8(PrimitiveOp::Div, x, 1),
            x,
            "DV_1 failure: div({x}, 1) ≠ {x} at W8"
        );
    }
    assert_eq!(const_ring_eval_w16(PrimitiveOp::Div, 0xFFFF, 1), 0xFFFF);
    assert_eq!(
        const_ring_eval_w32(PrimitiveOp::Div, 0xDEAD_BEEF, 1),
        0xDEAD_BEEF
    );
    assert_eq!(const_ring_eval_w64(PrimitiveOp::Div, u64::MAX, 1), u64::MAX);
    assert_eq!(
        const_ring_eval_w128(PrimitiveOp::Div, u128::MAX, 1),
        u128::MAX
    );
}

// ── ADR-053 DV_2: 0 div b = 0 for b ≠ 0 ─────────────────────────────

#[test]
fn dv2_div_left_absorbing_holds_at_native_widths() {
    for &b in &[1u8, 2, 7, 42, 255] {
        assert_eq!(const_ring_eval_w8(PrimitiveOp::Div, 0, b), 0);
    }
}

// ── ADR-053 DV_3: div recovery for in-range mul ─────────────────────

#[test]
fn dv3_div_of_multiplication_recovers_multiplicand() {
    // For x = 7, b = 9, mul = 63 which fits in u8.
    let mul = const_ring_eval_w8(PrimitiveOp::Mul, 7, 9);
    assert_eq!(mul, 63);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Div, mul, 9), 7);
}

// ── ADR-053 DV_4: Euclidean compatibility (a = q·b + r) ──────────────

#[test]
fn dv4_euclidean_compatibility_at_native_widths() {
    for &a in &[0u16, 1, 42, 1000, 12345, 0xFFFE] {
        for &b in &[1u16, 7, 13, 100, 255] {
            let q = const_ring_eval_w16(PrimitiveOp::Div, a, b);
            let r = const_ring_eval_w16(PrimitiveOp::Mod, a, b);
            let prod = const_ring_eval_w16(PrimitiveOp::Mul, q, b);
            let sum = const_ring_eval_w16(PrimitiveOp::Add, prod, r);
            assert_eq!(sum, a, "DV_4 failure: q·b+r={sum} ≠ a={a} for div({a},{b})");
        }
    }
}

// ── ADR-053 Mod range: r < b ─────────────────────────────────────────

#[test]
fn mod_result_is_strictly_less_than_divisor() {
    for &a in &[0u8, 1, 42, 100, 255] {
        for &b in &[1u8, 7, 13, 100, 255] {
            let r = const_ring_eval_w8(PrimitiveOp::Mod, a, b);
            assert!(r < b, "mod({a},{b}) = {r} should be < {b}");
        }
    }
}

// ── ADR-053 PW_1: pow(a, 0) = 1 ──────────────────────────────────────

#[test]
fn pw1_zero_exponent_yields_one() {
    for &a in &[0u8, 1, 2, 7, 42, 255] {
        assert_eq!(const_ring_eval_w8(PrimitiveOp::Pow, a, 0), 1);
    }
}

// ── ADR-053 PW_2: pow(a, 1) = a ──────────────────────────────────────

#[test]
fn pw2_unit_exponent_yields_base() {
    for &a in &[0u8, 1, 2, 7, 42, 255] {
        assert_eq!(const_ring_eval_w8(PrimitiveOp::Pow, a, 1), a);
    }
}

// ── ADR-053 PW_3: pow(a, b+c) = pow(a, b) · pow(a, c) ────────────────

#[test]
fn pw3_additive_exponent_decomposition_at_w16() {
    // 3^5 = 243 fits in u16, 3^7 = 2187 fits in u16.
    let lhs = const_ring_eval_w16(PrimitiveOp::Pow, 3, 12);
    let pb = const_ring_eval_w16(PrimitiveOp::Pow, 3, 5);
    let pc = const_ring_eval_w16(PrimitiveOp::Pow, 3, 7);
    let rhs = const_ring_eval_w16(PrimitiveOp::Mul, pb, pc);
    assert_eq!(
        lhs, rhs,
        "PW_3 failure: pow(3,12) ≠ pow(3,5)·pow(3,7) at W16"
    );
}

// ── ADR-050: Pow under modular reduction (overflow path) ─────────────

#[test]
fn pow_reduces_under_ring_modulus_at_w8() {
    // 2^8 = 256, which wraps to 0 at W8.
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Pow, 2, 8), 0);
    // 3^5 = 243, fits in u8.
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Pow, 3, 5), 243);
    // 5^4 = 625; 625 mod 256 = 113.
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Pow, 5, 4), 113);
}

// ── ADR-050: wide-width semantics ─────────────────────────────────────
//
// The byte-level kernel in `pipeline::byte_arith_be` is the runtime
// dispatch path for widths > 8 bytes (the Limbs-backed Witt levels W160
// through W32768). It is exercised through the catamorphism in the
// integration tests `behavior_pipeline_determinism` and via the trace
// replay round-trip in `uor-foundation-verify`; the const-fn helpers
// `const_ring_eval_w{n}` for Limbs widths are not publicly accessible
// because `Limbs::from_words` is `pub(crate)` (per ADR-018, applications
// drive width selection through `<MyBounds as HostBounds>::CONST`, not
// by constructing Limbs directly).

// ── ADR-050: Div by zero in const-eval returns 0 (safe-divisor) ──────

#[test]
fn div_by_zero_returns_zero_in_const_path() {
    // Const-fn helpers cannot raise errors; the catamorphism is the layer
    // that surfaces a ShapeViolation. The const helper returns 0 to keep
    // `const fn` total.
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Div, 42, 0), 0);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Mod, 42, 0), 0);
}

// ── ADR-050: Identical results across widths for small values ────────

#[test]
fn small_values_yield_identical_results_across_widths() {
    // div(100, 7) = 14 regardless of width.
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Div, 100, 7), 14);
    assert_eq!(const_ring_eval_w16(PrimitiveOp::Div, 100, 7), 14);
    assert_eq!(const_ring_eval_w32(PrimitiveOp::Div, 100, 7), 14);
    assert_eq!(const_ring_eval_w64(PrimitiveOp::Div, 100, 7), 14);
    // pow(3, 4) = 81 regardless of width.
    assert_eq!(const_ring_eval_w16(PrimitiveOp::Pow, 3, 4), 81);
    assert_eq!(const_ring_eval_w32(PrimitiveOp::Pow, 3, 4), 81);
    assert_eq!(const_ring_eval_w64(PrimitiveOp::Pow, 3, 4), 81);
}

// ── ADR-050: u128 Euclidean compatibility at W128 ─────────────────────

#[test]
fn w128_euclidean_compatibility() {
    let a: u128 = 0xDEAD_BEEF_CAFE_BABE_1234_5678_9ABC_DEFFu128;
    let b: u128 = 0xFFFF_FFFF_FFFFu128;
    let q = const_ring_eval_w128(PrimitiveOp::Div, a, b);
    let r = const_ring_eval_w128(PrimitiveOp::Mod, a, b);
    let qb = const_ring_eval_w128(PrimitiveOp::Mul, q, b);
    let recovered = const_ring_eval_w128(PrimitiveOp::Add, qb, r);
    assert_eq!(recovered, a, "DV_4 (Euclidean compatibility) fails at W128");
    assert!(r < b, "Mod result must be strictly less than divisor");
}

// ── ADR-053: Pow agrees with iterated Mul ─────────────────────────────

#[test]
fn pow_agrees_with_iterated_mul_at_w32() {
    // 5^6 via iterated multiplication.
    let mut acc: u32 = 1;
    for _ in 0..6 {
        acc = const_ring_eval_w32(PrimitiveOp::Mul, acc, 5);
    }
    let p = const_ring_eval_w32(PrimitiveOp::Pow, 5, 6);
    assert_eq!(p, acc, "pow(5,6) should match iterated Mul at W32");
}

// ── ADR-053 + ADR-050: Catamorphism integration over Term::Application ──
//
// These exercise `pipeline::evaluate_term_tree` (the runtime catamorphism's
// entry point) with the new Div/Mod/Pow primitives. The fold-rule routes
// through the u64 hot path for widths ≤ 8 bytes, and through the byte-level
// `byte_arith_be` kernel for wider operands.

mod catamorphism {
    use uor_foundation::enforcement::{Hasher, Term, TermList};
    use uor_foundation::pipeline::{evaluate_term_tree, NullResolverTuple, TermValue};
    use uor_foundation::{PipelineFailure, PrimitiveOp, WittLevel};
    use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

    #[derive(Debug, Clone, Copy, Default)]
    struct ZeroHasher;
    impl Hasher for ZeroHasher {
        const OUTPUT_BYTES: usize = 4;
        fn initial() -> Self {
            Self
        }
        fn fold_byte(self, _: u8) -> Self {
            self
        }
        fn finalize(self) -> [u8; 32] {
            [0u8; 32]
        }
    }

    fn eval<'a>(arena: &'a [Term<'a, N>]) -> Result<TermValue<'a, N>, PipelineFailure> {
        evaluate_term_tree::<ZeroHasher, NullResolverTuple, N, 32>(
            arena,
            TermValue::empty(),
            &NullResolverTuple,
        )
    }

    #[test]
    fn application_div_evaluates() {
        let arena = [
            uor_foundation::pipeline::literal_u64(100, WittLevel::W8),
            uor_foundation::pipeline::literal_u64(7, WittLevel::W8),
            Term::Application {
                operator: PrimitiveOp::Div,
                args: TermList { start: 0, len: 2 },
            },
        ];
        let r = eval(&arena).expect("div evaluates");
        assert_eq!(r.bytes(), &[14u8][..]);
    }

    #[test]
    fn application_mod_evaluates() {
        let arena = [
            uor_foundation::pipeline::literal_u64(100, WittLevel::W8),
            uor_foundation::pipeline::literal_u64(7, WittLevel::W8),
            Term::Application {
                operator: PrimitiveOp::Mod,
                args: TermList { start: 0, len: 2 },
            },
        ];
        let r = eval(&arena).expect("mod evaluates");
        assert_eq!(r.bytes(), &[2u8][..]);
    }

    #[test]
    fn application_pow_evaluates() {
        let arena = [
            uor_foundation::pipeline::literal_u64(3, WittLevel::W16),
            uor_foundation::pipeline::literal_u64(5, WittLevel::W16),
            Term::Application {
                operator: PrimitiveOp::Pow,
                args: TermList { start: 0, len: 2 },
            },
        ];
        let r = eval(&arena).expect("pow evaluates");
        // 3^5 = 243; W16 → two-byte big-endian payload.
        assert_eq!(r.bytes(), &[0x00, 243u8][..]);
    }

    #[test]
    fn application_div_by_zero_returns_shape_violation() {
        let arena = [
            uor_foundation::pipeline::literal_u64(42, WittLevel::W8),
            uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
            Term::Application {
                operator: PrimitiveOp::Div,
                args: TermList { start: 0, len: 2 },
            },
        ];
        let err = eval(&arena).expect_err("div(_, 0) must raise ShapeViolation");
        match err {
            PipelineFailure::ShapeViolation { report } => {
                assert_eq!(
                    report.constraint_iri,
                    "https://uor.foundation/op/Div/nonZeroDivisor"
                );
            }
            other => panic!("expected ShapeViolation, got {other:?}"),
        }
    }

    #[test]
    fn application_mod_by_zero_returns_shape_violation() {
        let arena = [
            uor_foundation::pipeline::literal_u64(42, WittLevel::W8),
            uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
            Term::Application {
                operator: PrimitiveOp::Mod,
                args: TermList { start: 0, len: 2 },
            },
        ];
        let err = eval(&arena).expect_err("mod(_, 0) must raise ShapeViolation");
        match err {
            PipelineFailure::ShapeViolation { report } => {
                assert_eq!(
                    report.constraint_iri,
                    "https://uor.foundation/op/Mod/nonZeroDivisor"
                );
            }
            other => panic!("expected ShapeViolation, got {other:?}"),
        }
    }
}
