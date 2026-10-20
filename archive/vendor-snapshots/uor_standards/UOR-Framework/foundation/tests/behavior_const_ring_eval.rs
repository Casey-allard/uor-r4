//! Behavioral contract for `const_ring_eval_w{n}` helpers.
//!
//! Target §4.4 / §4.5: every shipped WittLevel `W_n` has a
//! `const_ring_eval_w{n}` helper that evaluates the ring operations
//! `Add, Sub, Mul, And, Or, Xor, Neg, BNot, Succ` under `Z/(2^bits)Z`
//! semantics.
//!
//! This test file asserts correctness with known inputs/outputs for the
//! native-backed levels (W8 through W128). A regression where the wrap
//! semantics break (e.g., Mul doesn't mask, Neg computes `-x` without
//! modular reduction, Add loses carry) fails here.

use uor_foundation::enforcement::{
    const_ring_eval_w104, const_ring_eval_w112, const_ring_eval_w120, const_ring_eval_w128,
    const_ring_eval_w16, const_ring_eval_w24, const_ring_eval_w32, const_ring_eval_w40,
    const_ring_eval_w48, const_ring_eval_w56, const_ring_eval_w64, const_ring_eval_w72,
    const_ring_eval_w8, const_ring_eval_w80, const_ring_eval_w88, const_ring_eval_w96,
};
use uor_foundation::PrimitiveOp;

// ─── W8: Z/256Z ─────────────────────────────────────────────────────────

#[test]
fn w8_add_wraps_modulo_256() {
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Add, 200, 100), 44);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Add, 0, 0), 0);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Add, 255, 1), 0);
}

#[test]
fn w8_sub_is_modular() {
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Sub, 0, 1), 255);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Sub, 100, 50), 50);
}

#[test]
fn w8_mul_masks_to_u8() {
    // 15 * 15 = 225 (fits in u8)
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Mul, 15, 15), 225);
    // 16 * 16 = 256 -> 0 (wraps)
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Mul, 16, 16), 0);
}

#[test]
fn w8_bitwise_ops_are_exact() {
    assert_eq!(const_ring_eval_w8(PrimitiveOp::And, 0b1010, 0b1100), 0b1000);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Or, 0b1010, 0b1100), 0b1110);
    assert_eq!(const_ring_eval_w8(PrimitiveOp::Xor, 0b1010, 0b1100), 0b0110);
}

#[test]
fn w8_unary_ops_identity_neg_bnot_succ() {
    // The critical identity: neg(bnot(x)) = succ(x) for all x in Z/2^nZ.
    for x in 0u8..=255 {
        let bnot_x = const_ring_eval_w8(PrimitiveOp::Bnot, x, 0);
        let neg_bnot = const_ring_eval_w8(PrimitiveOp::Neg, bnot_x, 0);
        let succ_x = const_ring_eval_w8(PrimitiveOp::Succ, x, 0);
        assert_eq!(
            neg_bnot, succ_x,
            "identity neg(bnot({x})) = succ({x}) must hold at W8"
        );
    }
}

// ─── W16: Z/65536Z ──────────────────────────────────────────────────────

#[test]
fn w16_mul_and_wrap() {
    // 300 * 300 = 90000 mod 65536 = 24464
    assert_eq!(const_ring_eval_w16(PrimitiveOp::Mul, 300, 300), 24464);
    // 65535 + 1 = 0
    assert_eq!(const_ring_eval_w16(PrimitiveOp::Add, 65535, 1), 0);
}

#[test]
fn w16_identity_holds_at_boundaries() {
    for &x in &[0u16, 1, 2, 12345, 65535] {
        let bnot_x = const_ring_eval_w16(PrimitiveOp::Bnot, x, 0);
        let neg_bnot = const_ring_eval_w16(PrimitiveOp::Neg, bnot_x, 0);
        let succ_x = const_ring_eval_w16(PrimitiveOp::Succ, x, 0);
        assert_eq!(neg_bnot, succ_x, "W16 identity at x={x}");
    }
}

// ─── W24: masked u32 ────────────────────────────────────────────────────

#[test]
fn w24_masks_to_24_bits() {
    // 2^24 = 16777216 -> result must be masked to 24 bits.
    // 0xFFFFFF + 1 = 0x1000000 mod 2^24 = 0
    assert_eq!(
        const_ring_eval_w24(PrimitiveOp::Add, 0xFF_FFFF, 1),
        0,
        "W24 add must mask high bits above bit 23"
    );
    // Mul: 0x800000 * 2 = 0x1000000 mod 2^24 = 0
    assert_eq!(
        const_ring_eval_w24(PrimitiveOp::Mul, 0x80_0000, 2),
        0,
        "W24 mul must mask"
    );
}

// ─── W32, W64, W128: exact native widths ───────────────────────────────

#[test]
fn w32_exact_u32_arithmetic() {
    assert_eq!(const_ring_eval_w32(PrimitiveOp::Add, u32::MAX, 1), 0);
    assert_eq!(
        const_ring_eval_w32(PrimitiveOp::Xor, 0xFF00_FF00, 0x0F0F_0F0F),
        0xF00F_F00F
    );
}

#[test]
fn w64_exact_u64_arithmetic() {
    assert_eq!(const_ring_eval_w64(PrimitiveOp::Add, u64::MAX, 1), 0);
    assert_eq!(const_ring_eval_w64(PrimitiveOp::Mul, 2, 3), 6);
}

#[test]
fn w128_exact_u128_arithmetic() {
    assert_eq!(const_ring_eval_w128(PrimitiveOp::Add, u128::MAX, 1), 0);
    assert_eq!(const_ring_eval_w128(PrimitiveOp::Sub, 0, 1), u128::MAX);
}

// ─── Non-native widths (W40..W120) mask correctly ──────────────────────

#[test]
fn non_native_widths_mask_correctly() {
    // W40 = masked u64, bit width 40, max = 2^40 - 1 = 0xFF_FFFF_FFFF
    let max_w40: u64 = (1u64 << 40) - 1;
    assert_eq!(const_ring_eval_w40(PrimitiveOp::Add, max_w40, 1), 0);

    // W48
    let max_w48: u64 = (1u64 << 48) - 1;
    assert_eq!(const_ring_eval_w48(PrimitiveOp::Add, max_w48, 1), 0);

    // W56
    let max_w56: u64 = (1u64 << 56) - 1;
    assert_eq!(const_ring_eval_w56(PrimitiveOp::Add, max_w56, 1), 0);

    // W72 (u128-backed, masked to 72)
    let max_w72: u128 = (1u128 << 72) - 1;
    assert_eq!(const_ring_eval_w72(PrimitiveOp::Add, max_w72, 1), 0);

    // W80, W88, W96, W104, W112, W120
    type EvalFn = fn(PrimitiveOp, u128, u128) -> u128;
    let cases: &[(u128, EvalFn)] = &[
        ((1u128 << 80) - 1, const_ring_eval_w80),
        ((1u128 << 88) - 1, const_ring_eval_w88),
        ((1u128 << 96) - 1, const_ring_eval_w96),
        ((1u128 << 104) - 1, const_ring_eval_w104),
        ((1u128 << 112) - 1, const_ring_eval_w112),
        ((1u128 << 120) - 1, const_ring_eval_w120),
    ];
    for (max, f) in cases {
        assert_eq!(f(PrimitiveOp::Add, *max, 1), 0);
    }
}

// ─── Purity: same inputs → same outputs ──────────────────────────────────

#[test]
fn all_ops_are_pure() {
    for _ in 0..100 {
        assert_eq!(
            const_ring_eval_w32(PrimitiveOp::Mul, 12345, 67890),
            12345u32.wrapping_mul(67890)
        );
    }
}
