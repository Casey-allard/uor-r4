//! Phase H: libm + transcendentals integration test.
//!
//! Asserts that the foundation's `transcendentals` module — the canonical
//! routing point for `xsd:decimal` observables — is reachable and produces
//! correct results via the always-on `libm` dependency (target §1.6).

use uor_foundation::enforcement::transcendentals;

#[test]
fn transcendentals_ln_agrees_with_known_values() {
    assert!((transcendentals::ln::<f64>(1.0) - 0.0).abs() < 1e-12);
    assert!((transcendentals::ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
    assert!((transcendentals::ln(2.0) - core::f64::consts::LN_2).abs() < 1e-12);
}

#[test]
fn transcendentals_exp_agrees_with_known_values() {
    assert!((transcendentals::exp::<f64>(0.0) - 1.0).abs() < 1e-12);
    assert!((transcendentals::exp::<f64>(1.0) - core::f64::consts::E).abs() < 1e-12);
}

#[test]
fn transcendentals_sqrt_agrees_with_known_values() {
    assert!((transcendentals::sqrt::<f64>(0.0) - 0.0).abs() < 1e-12);
    assert!((transcendentals::sqrt::<f64>(4.0) - 2.0).abs() < 1e-12);
    assert!((transcendentals::sqrt::<f64>(2.0) - core::f64::consts::SQRT_2).abs() < 1e-12);
}

#[test]
fn entropy_term_at_zero_probability_is_zero() {
    // Shannon's entropy term -p·ln(p) by continuous extension: 0 at p=0.
    assert_eq!(transcendentals::entropy_term_nats::<f64>(0.0), 0.0);
    // And for negative probabilities (outside the valid domain), also 0
    // by construction rather than NaN — the foundation keeps the function
    // total for safety.
    assert_eq!(transcendentals::entropy_term_nats::<f64>(-0.5), 0.0);
}

#[test]
fn entropy_term_at_uniform_half_equals_half_ln_two() {
    // H(1/2) = -0.5 · ln(0.5) = 0.5 · ln(2)
    let expected = 0.5 * core::f64::consts::LN_2;
    let got = transcendentals::entropy_term_nats::<f64>(0.5);
    assert!((got - expected).abs() < 1e-12);
}

#[test]
fn round_trip_ln_exp_identity() {
    for x in [0.1, 1.0, core::f64::consts::E, 10.0, 100.0] {
        let y = transcendentals::ln(x);
        let x_recovered = transcendentals::exp::<f64>(y);
        assert!((x - x_recovered).abs() / x < 1e-12);
    }
}
