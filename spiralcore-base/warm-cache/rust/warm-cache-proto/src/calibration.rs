//! Derive B_l, R_min and W from measured offsets and the Bell margin.
use std::f64::consts::TAU;

pub const BELL_MARGIN: f64 = TAU / 15.0;

#[derive(Clone, Debug, PartialEq)]
pub struct TimeCalibration {
    pub phase_bits: u8,
    pub r_min: f64,
    pub w: u32,
    pub margin: f64,
    pub sigma_allow: f64,
}

impl TimeCalibration {
    pub fn from_offset(spread: f64, margin: f64, drift_per_cycle: f64) -> Self {
        let spread = spread.max(1e-9);
        let sigma_allow = (margin / 2.0).min(spread);
        let delta = sigma_allow.max(1e-12);
        let phase_bits = ((TAU / delta).log2().ceil() as u8).saturating_add(1);
        let uniform = (sigma_allow.sin() / sigma_allow).clamp(0.0, 1.0);
        let von_mises = (-(sigma_allow * sigma_allow) / 2.0).exp();
        let r_min = uniform.min(von_mises);
        let w = if drift_per_cycle > 0.0 && sigma_allow < margin / 2.0 {
            (((margin / 2.0 - sigma_allow) / drift_per_cycle).ceil() as u32).max(1)
        } else { 1 };
        Self { phase_bits, r_min, w, margin, sigma_allow }
    }

    pub fn default_bell(spread: f64, drift_per_cycle: f64) -> Self {
        Self::from_offset(spread, BELL_MARGIN, drift_per_cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_is_derived_from_offsets() {
        let c = TimeCalibration::default_bell(0.1259, 0.0);
        assert!(c.phase_bits >= 4);
        assert!(c.r_min > 0.99 && c.r_min <= 1.0);
        assert_eq!(c.w, 1);
        let c2 = TimeCalibration::default_bell(0.05, 0.01);
        assert!(c2.phase_bits > c.phase_bits);
        assert!(c2.r_min > 0.99);
        assert!(c2.w >= 1);
    }
}
