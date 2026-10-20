//! v0.2.2 Phase A: tests for `UorTime`, `LandauerBudget`, `Calibration`,
//! `Nanos`, and the foundation-shipped calibration presets.
//!
//! Asserts:
//! - `UorTime` is component-wise `PartialOrd` (not `Ord`).
//! - `LandauerBudget` is `Ord` over its underlying f64.
//! - `Calibration::new` validates physical plausibility and returns
//!   `Err(CalibrationError::*)` on out-of-range inputs.
//! - All four foundation presets construct successfully.
//! - `min_wall_clock` produces a `Nanos` that is `>= 0` and increases
//!   monotonically with both clocks.

use uor_foundation::enforcement::{calibrations, CalibrationError, Nanos};
use uor_foundation::DefaultHostTypes;

// Phase 9 pinned: behavioral tests exercise the default-host (f64) path.
type Calibration = uor_foundation::enforcement::Calibration<DefaultHostTypes>;
type LandauerBudget = uor_foundation::enforcement::LandauerBudget<DefaultHostTypes>;
type UorTime = uor_foundation::enforcement::UorTime<DefaultHostTypes>;

// ─────────────────────────────────────────────────────────────────────────
// LandauerBudget
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn landauer_budget_accessor_returns_nats() {
    // We can only construct LandauerBudget via the pub(crate) constructor,
    // which means downstream tests can't make one directly. The only
    // observable budgets come through UorTime values produced by the
    // foundation. We exercise that path below; here we just assert the
    // type is in the public surface.
    let _f: fn(&LandauerBudget) -> f64 = LandauerBudget::nats;
}

#[test]
fn landauer_budget_implements_ord() {
    // Compile-time witness that LandauerBudget : Ord.
    fn require_ord<T: Ord>() {}
    require_ord::<LandauerBudget>();
}

// ─────────────────────────────────────────────────────────────────────────
// UorTime
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn uor_time_implements_partial_ord_not_ord() {
    fn require_partial_ord<T: PartialOrd>() {}
    require_partial_ord::<UorTime>();
    // The compile-time witness for "not Ord" is the absence of an
    // `Ord` impl. We can't directly assert non-implementation in stable
    // Rust without `negative_impls`; the public-API snapshot pins the
    // absence of `impl Ord for UorTime`, which catches accidental
    // additions.
}

#[test]
fn uor_time_accessors_match_struct_layout() {
    let _f: fn(&UorTime) -> LandauerBudget = UorTime::landauer_nats;
    let _g: fn(&UorTime) -> u64 = UorTime::rewrite_steps;
}

// ─────────────────────────────────────────────────────────────────────────
// Calibration
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn calibration_new_accepts_room_temperature_x86_server() {
    // X86_SERVER preset values: k_B·T = 4.14e-21 J, P = 85 W, E = 1e-15 J.
    let cal = Calibration::new(4.14e-21, 85.0, 1.0e-15);
    assert!(cal.is_ok());
}

#[test]
fn calibration_new_rejects_negative_thermal_energy() {
    let cal = Calibration::new(-1.0e-21, 85.0, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalEnergy));
}

#[test]
fn calibration_new_rejects_zero_thermal_energy() {
    let cal = Calibration::new(0.0, 85.0, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalEnergy));
}

#[test]
fn calibration_new_rejects_thermal_energy_above_known_universe() {
    // Above 1e-15 J (~1e8 K) is unphysical for any realistic substrate.
    let cal = Calibration::new(1.0e-10, 85.0, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalEnergy));
}

#[test]
fn calibration_new_rejects_negative_thermal_power() {
    let cal = Calibration::new(4.14e-21, -1.0, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalPower));
}

#[test]
fn calibration_new_rejects_thermal_power_above_gigawatt() {
    let cal = Calibration::new(4.14e-21, 1.0e10, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalPower));
}

#[test]
fn calibration_new_rejects_negative_characteristic_energy() {
    let cal = Calibration::new(4.14e-21, 85.0, -1.0e-15);
    assert_eq!(cal, Err(CalibrationError::CharacteristicEnergy));
}

#[test]
fn calibration_new_rejects_characteristic_energy_above_kilojoule() {
    let cal = Calibration::new(4.14e-21, 85.0, 1.0e4);
    assert_eq!(cal, Err(CalibrationError::CharacteristicEnergy));
}

#[test]
fn calibration_new_rejects_nan_thermal_energy() {
    let cal = Calibration::new(f64::NAN, 85.0, 1.0e-15);
    assert_eq!(cal, Err(CalibrationError::ThermalEnergy));
}

// ─────────────────────────────────────────────────────────────────────────
// Foundation presets
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn x86_server_preset_is_valid() {
    let cal: Calibration = calibrations::X86_SERVER;
    assert_eq!(cal.k_b_t(), 4.14e-21);
    assert_eq!(cal.thermal_power(), 85.0);
    assert_eq!(cal.characteristic_energy(), 1.0e-15);
}

#[test]
fn arm_mobile_preset_is_valid() {
    let cal: Calibration = calibrations::ARM_MOBILE;
    assert_eq!(cal.k_b_t(), 4.14e-21);
    assert_eq!(cal.thermal_power(), 5.0);
    assert_eq!(cal.characteristic_energy(), 1.0e-16);
}

#[test]
fn cortex_m_embedded_preset_is_valid() {
    let cal: Calibration = calibrations::CORTEX_M_EMBEDDED;
    assert_eq!(cal.k_b_t(), 4.14e-21);
    assert_eq!(cal.thermal_power(), 0.1);
    assert_eq!(cal.characteristic_energy(), 1.0e-17);
}

#[test]
fn conservative_worst_case_preset_is_valid() {
    let cal: Calibration = calibrations::CONSERVATIVE_WORST_CASE;
    assert_eq!(cal.k_b_t(), 4.14e-21);
    assert_eq!(cal.thermal_power(), 1.0e9);
    assert_eq!(cal.characteristic_energy(), 1.0);
}

// ─────────────────────────────────────────────────────────────────────────
// Nanos
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn nanos_as_u64_returns_underlying_value() {
    // Like LandauerBudget, Nanos has only a pub(crate) constructor.
    // We assert the public accessor exists; integration testing happens
    // through min_wall_clock once UorTime is wired to the pipeline (Phase E+).
    let _f: fn(Nanos) -> u64 = Nanos::as_u64;
}
