//! Phase 9a test: `DecimalTranscendental` is implemented for `f64` and
//! `f32` and bounded onto `HostTypes::Decimal`. Verifies the closed
//! arithmetic + transcendental + bit-pattern round-trip surface.

use uor_foundation::{DecimalTranscendental, DefaultHostTypes, HostTypes};

fn assert_decimal_transcendental<T: DecimalTranscendental>() {
    // The presence of this monomorphisation is the bound check.
    let _ = T::from_u32(1);
}

#[test]
fn default_host_types_decimal_implements_decimal_transcendental() {
    assert_decimal_transcendental::<<DefaultHostTypes as HostTypes>::Decimal>();
}

#[test]
fn f64_round_trip_bits() {
    let value = std::f64::consts::LN_2;
    let bits = value.to_bits();
    let restored = <f64 as DecimalTranscendental>::from_bits(bits);
    assert_eq!(value.to_bits(), restored.to_bits());
}

#[test]
fn f64_arithmetic_closure() {
    let a = <f64 as DecimalTranscendental>::from_u32(2);
    let b = <f64 as DecimalTranscendental>::from_u32(3);
    let sum = a + b;
    assert_eq!(sum, 5.0);
    let prod = a * b;
    assert_eq!(prod, 6.0);
}

#[test]
fn f64_transcendentals() {
    let one = <f64 as DecimalTranscendental>::from_u32(1);
    let ln_e = <f64 as DecimalTranscendental>::ln(one.exp());
    // ln(e^1) = 1 within fp tolerance.
    assert!((ln_e - one).abs() < 1e-12);
    let sqrt4 = <f64 as DecimalTranscendental>::sqrt(<f64 as DecimalTranscendental>::from_u32(4));
    assert!((sqrt4 - 2.0).abs() < 1e-12);
}

#[test]
fn f32_round_trip_via_f64() {
    let value = 0.5_f32;
    let bits64 = (value as f64).to_bits();
    let restored = <f32 as DecimalTranscendental>::from_bits(bits64);
    assert!((restored - value).abs() < f32::EPSILON);
}

#[test]
fn entropy_term_zero() {
    let zero: f64 = Default::default();
    assert_eq!(<f64 as DecimalTranscendental>::entropy_term_nats(zero), 0.0);
}

#[test]
fn entropy_term_unit() {
    let one = <f64 as DecimalTranscendental>::from_u32(1);
    // 1 * ln(1) = 0
    assert!(one.entropy_term_nats().abs() < 1e-12);
}
