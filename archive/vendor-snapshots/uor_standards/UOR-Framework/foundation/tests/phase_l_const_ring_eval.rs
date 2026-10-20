//! Phase L.2/L.3 (target §4.5 + §9 criterion 5): const-eval emission for
//! every shipped WittLevel.
//!
//! Asserts that `const_ring_eval_w{n}` helpers exist and are invocable in
//! const context for:
//! - Native-backed levels (W8..W128): scalar `u*` operands
//! - Limbs-backed levels (W160+): `Limbs<N>` operands
//!
//! The helpers are const fn; whether `rustc` completes any particular
//! compile-time evaluation is a function of the invocation, not the
//! helper's existence (target §4.5 Q2 practicality table). The mere
//! presence at each level is the conformance property.

use uor_foundation::enforcement::{
    const_ring_eval_w1024, const_ring_eval_w12288, const_ring_eval_w128, const_ring_eval_w16,
    const_ring_eval_w160, const_ring_eval_w16384, const_ring_eval_w192, const_ring_eval_w2048,
    const_ring_eval_w224, const_ring_eval_w256, const_ring_eval_w32, const_ring_eval_w32768,
    const_ring_eval_w384, const_ring_eval_w4096, const_ring_eval_w448, const_ring_eval_w512,
    const_ring_eval_w520, const_ring_eval_w528, const_ring_eval_w64, const_ring_eval_w8,
    const_ring_eval_w8192,
};
use uor_foundation::PrimitiveOp;

#[test]
fn const_ring_eval_w8_native_add_wraps_mod_256() {
    const R: u8 = const_ring_eval_w8(PrimitiveOp::Add, 200, 100);
    assert_eq!(R, 44);
}

#[test]
fn const_ring_eval_w16_native_mul() {
    const R: u16 = const_ring_eval_w16(PrimitiveOp::Mul, 300, 300);
    // 90000 mod 65536 = 24464
    assert_eq!(R, 24464);
}

#[test]
fn const_ring_eval_w32_native_xor() {
    const R: u32 = const_ring_eval_w32(PrimitiveOp::Xor, 0xFF00_FF00, 0x0F0F_0F0F);
    assert_eq!(R, 0xF00F_F00F);
}

#[test]
fn const_ring_eval_w64_native_add_wraps() {
    const R: u64 = const_ring_eval_w64(PrimitiveOp::Add, u64::MAX, 1);
    assert_eq!(R, 0);
}

#[test]
fn const_ring_eval_w128_native_add_wraps() {
    const R: u128 = const_ring_eval_w128(PrimitiveOp::Add, u128::MAX, 1);
    assert_eq!(R, 0);
}

/// All 16 Limbs-backed levels are addressable as const fn symbols. We can't
/// easily call them in const context without `Limbs<N>::from_words` being
/// public, but we can verify they are reachable via type-level witness.
#[test]
fn const_ring_eval_limbs_helpers_are_addressable() {
    type L3 = uor_foundation::enforcement::Limbs<3>;
    type L4 = uor_foundation::enforcement::Limbs<4>;
    type L6 = uor_foundation::enforcement::Limbs<6>;
    type L7 = uor_foundation::enforcement::Limbs<7>;
    type L8 = uor_foundation::enforcement::Limbs<8>;
    type L9 = uor_foundation::enforcement::Limbs<9>;
    type L16 = uor_foundation::enforcement::Limbs<16>;
    type L32 = uor_foundation::enforcement::Limbs<32>;
    type L64 = uor_foundation::enforcement::Limbs<64>;
    type L128 = uor_foundation::enforcement::Limbs<128>;
    type L192 = uor_foundation::enforcement::Limbs<192>;
    type L256 = uor_foundation::enforcement::Limbs<256>;
    type L512 = uor_foundation::enforcement::Limbs<512>;

    // Type-level witnesses that each helper accepts the expected Limbs<N>.
    let _a: fn(PrimitiveOp, L3, L3) -> L3 = const_ring_eval_w160;
    let _a: fn(PrimitiveOp, L3, L3) -> L3 = const_ring_eval_w192;
    let _a: fn(PrimitiveOp, L4, L4) -> L4 = const_ring_eval_w224;
    let _a: fn(PrimitiveOp, L4, L4) -> L4 = const_ring_eval_w256;
    let _a: fn(PrimitiveOp, L6, L6) -> L6 = const_ring_eval_w384;
    let _a: fn(PrimitiveOp, L7, L7) -> L7 = const_ring_eval_w448;
    let _a: fn(PrimitiveOp, L8, L8) -> L8 = const_ring_eval_w512;
    let _a: fn(PrimitiveOp, L9, L9) -> L9 = const_ring_eval_w520;
    let _a: fn(PrimitiveOp, L9, L9) -> L9 = const_ring_eval_w528;
    let _a: fn(PrimitiveOp, L16, L16) -> L16 = const_ring_eval_w1024;
    let _a: fn(PrimitiveOp, L32, L32) -> L32 = const_ring_eval_w2048;
    let _a: fn(PrimitiveOp, L64, L64) -> L64 = const_ring_eval_w4096;
    let _a: fn(PrimitiveOp, L128, L128) -> L128 = const_ring_eval_w8192;
    let _a: fn(PrimitiveOp, L192, L192) -> L192 = const_ring_eval_w12288;
    let _a: fn(PrimitiveOp, L256, L256) -> L256 = const_ring_eval_w16384;
    let _a: fn(PrimitiveOp, L512, L512) -> L512 = const_ring_eval_w32768;
}
