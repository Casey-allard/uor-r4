//! Behavioral contract for `Calibration::new` and the four preset calibrations.
//!
//! The contract (target §1.7):
//!
//! - `Calibration::new(k_b_t, thermal_power, characteristic_energy)` returns
//!   `Err(CalibrationError::ThermalEnergy)` when `k_b_t` is NaN, ≤ 0, < 1e-30,
//!   or > 1e-15.
//! - Returns `Err(CalibrationError::ThermalPower)` when `thermal_power` is
//!   NaN, ≤ 0, or > 1e9.
//! - Returns `Err(CalibrationError::CharacteristicEnergy)` when
//!   `characteristic_energy` is NaN, ≤ 0, or > 1e3.
//! - Returns `Ok(Calibration)` when all three parameters are in range.
//! - The four preset constants — `X86_SERVER`, `ARM_MOBILE`,
//!   `CORTEX_M_EMBEDDED`, `CONSERVATIVE_WORST_CASE` — must all be in-range;
//!   a zero-valued sentinel (produced by the `match { Ok(c) => c, Err(_) =>
//!   ZERO_SENTINEL }` fallback in the foundation) is a defect.
//!
//! If these assertions fail, the foundation's calibration endpoint is
//! incorrect and must be fixed before release.

use uor_foundation::enforcement::{calibrations, CalibrationError};
use uor_foundation::DefaultHostTypes;

// Pin the test surface to the default-host (f64) backing. Phase 9 made
// `Calibration<H>` generic; the hand-written behavioral tests in this file
// exercise the f64 path because the in-tree presets are all
// `Calibration<DefaultHostTypes>`.
type Calibration = uor_foundation::enforcement::Calibration<DefaultHostTypes>;

// ─── ThermalEnergy rejections ────────────────────────────────────────────

#[test]
fn calibration_rejects_nan_thermal_energy() {
    let err = Calibration::new(f64::NAN, 1.0, 1e-15).expect_err("NaN k_b_t must be rejected");
    assert_eq!(err, CalibrationError::ThermalEnergy);
}

#[test]
fn calibration_rejects_zero_thermal_energy() {
    let err = Calibration::new(0.0, 1.0, 1e-15).expect_err("zero k_b_t must be rejected");
    assert_eq!(err, CalibrationError::ThermalEnergy);
}

#[test]
fn calibration_rejects_negative_thermal_energy() {
    let err = Calibration::new(-1e-21, 1.0, 1e-15).expect_err("negative k_b_t must be rejected");
    assert_eq!(err, CalibrationError::ThermalEnergy);
}

#[test]
fn calibration_rejects_thermal_energy_below_floor() {
    // 1e-31 < 1e-30 floor
    let err = Calibration::new(1e-31, 1.0, 1e-15)
        .expect_err("k_b_t below the 1e-30 floor must be rejected");
    assert_eq!(err, CalibrationError::ThermalEnergy);
}

#[test]
fn calibration_rejects_thermal_energy_above_ceiling() {
    // 1e-14 > 1e-15 ceiling
    let err = Calibration::new(1e-14, 1.0, 1e-15)
        .expect_err("k_b_t above the 1e-15 ceiling must be rejected");
    assert_eq!(err, CalibrationError::ThermalEnergy);
}

// ─── ThermalPower rejections ─────────────────────────────────────────────

#[test]
fn calibration_rejects_nan_thermal_power() {
    let err = Calibration::new(4.14e-21, f64::NAN, 1e-15)
        .expect_err("NaN thermal_power must be rejected");
    assert_eq!(err, CalibrationError::ThermalPower);
}

#[test]
fn calibration_rejects_zero_thermal_power() {
    let err =
        Calibration::new(4.14e-21, 0.0, 1e-15).expect_err("zero thermal_power must be rejected");
    assert_eq!(err, CalibrationError::ThermalPower);
}

#[test]
fn calibration_rejects_negative_thermal_power() {
    let err = Calibration::new(4.14e-21, -1.0, 1e-15)
        .expect_err("negative thermal_power must be rejected");
    assert_eq!(err, CalibrationError::ThermalPower);
}

#[test]
fn calibration_rejects_thermal_power_above_gigawatt() {
    // 2e9 > 1e9 ceiling
    let err = Calibration::new(4.14e-21, 2e9, 1e-15)
        .expect_err("thermal_power above 1e9 W must be rejected");
    assert_eq!(err, CalibrationError::ThermalPower);
}

// ─── CharacteristicEnergy rejections ─────────────────────────────────────

#[test]
fn calibration_rejects_nan_characteristic_energy() {
    let err = Calibration::new(4.14e-21, 1.0, f64::NAN)
        .expect_err("NaN characteristic_energy must be rejected");
    assert_eq!(err, CalibrationError::CharacteristicEnergy);
}

#[test]
fn calibration_rejects_zero_characteristic_energy() {
    let err = Calibration::new(4.14e-21, 1.0, 0.0)
        .expect_err("zero characteristic_energy must be rejected");
    assert_eq!(err, CalibrationError::CharacteristicEnergy);
}

#[test]
fn calibration_rejects_negative_characteristic_energy() {
    let err = Calibration::new(4.14e-21, 1.0, -1e-15)
        .expect_err("negative characteristic_energy must be rejected");
    assert_eq!(err, CalibrationError::CharacteristicEnergy);
}

#[test]
fn calibration_rejects_characteristic_energy_above_kilojoule() {
    // 1e4 > 1e3 ceiling
    let err = Calibration::new(4.14e-21, 1.0, 1e4)
        .expect_err("characteristic_energy above 1e3 J must be rejected");
    assert_eq!(err, CalibrationError::CharacteristicEnergy);
}

// ─── Admissible constructions ────────────────────────────────────────────

#[test]
fn calibration_accepts_room_temperature_server_envelope() {
    // k_B·T at 300K = 4.14e-21; thermal_power ~85W; characteristic_energy ~1e-15
    let cal = Calibration::new(4.14e-21, 85.0, 1e-15)
        .expect("physically plausible x86-server parameters must validate");
    let _ = cal;
}

#[test]
fn calibration_accepts_boundary_values_inclusive() {
    // Exact boundary values must validate (the ranges are inclusive of the endpoints).
    let _ = Calibration::new(1e-30, 1e-9, 1e-15).expect("1e-30 k_b_t floor is inclusive");
    let _ = Calibration::new(1e-15, 1e-9, 1e-15).expect("1e-15 k_b_t ceiling is inclusive");
    let _ = Calibration::new(4.14e-21, 1e9, 1e-15).expect("1e9 thermal_power ceiling is inclusive");
    let _ = Calibration::new(4.14e-21, 1.0, 1e3)
        .expect("1e3 characteristic_energy ceiling is inclusive");
}

// ─── Preset calibrations ─────────────────────────────────────────────────
//
// The four shipped presets must be valid — i.e. each must round-trip through
// Calibration::new with Ok. The foundation's emission uses a match
// `Ok(c) => c, Err(_) => ZERO_SENTINEL` pattern; a zero-sentinel would
// silently produce `min_wall_clock` = 0 for every UorTime. The accessors
// below catch that regression: an invalid preset produces k_b_t == 0.0
// which no physically meaningful calibration can have.

#[test]
fn x86_server_preset_is_physically_valid() {
    let cal = &calibrations::X86_SERVER;
    assert!(
        cal.k_b_t() > 0.0,
        "X86_SERVER preset must have non-zero k_b_t (else it's the ZERO_SENTINEL fallback)"
    );
    assert!(cal.thermal_power() > 0.0);
    assert!(cal.characteristic_energy() > 0.0);
    // Round-trip: constructing a Calibration from the preset's fields must succeed.
    Calibration::new(
        cal.k_b_t(),
        cal.thermal_power(),
        cal.characteristic_energy(),
    )
    .expect("X86_SERVER preset must round-trip through Calibration::new");
}

#[test]
fn arm_mobile_preset_is_physically_valid() {
    let cal = &calibrations::ARM_MOBILE;
    assert!(cal.k_b_t() > 0.0);
    assert!(cal.thermal_power() > 0.0);
    assert!(cal.characteristic_energy() > 0.0);
    Calibration::new(
        cal.k_b_t(),
        cal.thermal_power(),
        cal.characteristic_energy(),
    )
    .expect("ARM_MOBILE preset must round-trip");
}

#[test]
fn cortex_m_embedded_preset_is_physically_valid() {
    let cal = &calibrations::CORTEX_M_EMBEDDED;
    assert!(cal.k_b_t() > 0.0);
    assert!(cal.thermal_power() > 0.0);
    assert!(cal.characteristic_energy() > 0.0);
    Calibration::new(
        cal.k_b_t(),
        cal.thermal_power(),
        cal.characteristic_energy(),
    )
    .expect("CORTEX_M_EMBEDDED preset must round-trip");
}

#[test]
fn conservative_worst_case_preset_is_physically_valid() {
    let cal = &calibrations::CONSERVATIVE_WORST_CASE;
    assert!(cal.k_b_t() > 0.0);
    assert!(cal.thermal_power() > 0.0);
    assert!(cal.characteristic_energy() > 0.0);
    Calibration::new(
        cal.k_b_t(),
        cal.thermal_power(),
        cal.characteristic_energy(),
    )
    .expect("CONSERVATIVE_WORST_CASE preset must round-trip");
}
