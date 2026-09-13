//! Offline streaming diagnostics for repeated gradient vectors.
//! These descriptive estimates are not confidence intervals or a learning gate.
use super::engine::{EngineError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GradientStats {
    pub len: usize,
    pub count: usize,
    pub nonzero_count: usize,
    pub mean_l2: f64,
    pub mean_sample_l2: f64,
    pub trace_sample_covariance: f64,
    pub standard_error_l2: f64,
    /// May be negative: no truncation of the unbiased signal estimate.
    pub unbiased_signal_norm_squared: f64,
    /// Zero vectors have no direction and are excluded from both pair counts.
    pub mean_pairwise_cosine: Option<f64>,
}

/// Storage is two vectors of `len` f64 values, independent of replicate count.
#[derive(Debug, Clone)]
pub struct Stats {
    sum: Vec<f64>,
    unit_sum: Vec<f64>,
    sum_sqnorm: f64,
    sum_norm: f64,
    count: usize,
    nonzero_count: usize,
}

impl Stats {
    pub fn new(len: usize) -> Result<Self> {
        if len == 0 {
            return Err(EngineError::Invalid("gradient statistics empty vector"));
        }
        fn zeros(len: usize) -> Result<Vec<f64>> {
            let mut values = Vec::new();
            values
                .try_reserve_exact(len)
                .map_err(|_| EngineError::Invalid("gradient statistics allocation"))?;
            values.resize(len, 0.0);
            Ok(values)
        }
        Ok(Self {
            sum: zeros(len)?,
            unit_sum: zeros(len)?,
            sum_sqnorm: 0.0,
            sum_norm: 0.0,
            count: 0,
            nonzero_count: 0,
        })
    }

    /// Validate all prospective scalar/vector sums before changing this stream.
    pub fn add(&mut self, sample: &[f64]) -> Result<()> {
        if sample.len() != self.sum.len() {
            return Err(EngineError::Invalid("gradient statistics length mismatch"));
        }
        let count = self
            .count
            .checked_add(1)
            .ok_or(EngineError::Invalid("gradient statistics count overflow"))?;
        // Counts are converted to f64 in the descriptive formulae below.
        if count as u128 > (1u128 << 53) {
            return Err(EngineError::Invalid("gradient statistics count precision"));
        }
        let mut sqnorm = 0.0;
        let mut has_nonzero = false;
        for &value in sample {
            if !value.is_finite() {
                return Err(EngineError::Invalid("gradient statistics nonfinite sample"));
            }
            has_nonzero |= value != 0.0;
            sqnorm += value * value;
        }
        if !sqnorm.is_finite() || (has_nonzero && sqnorm == 0.0) {
            return Err(EngineError::Invalid("gradient statistics norm range"));
        }
        let norm = sqnorm.sqrt();
        let sum_sqnorm = self.sum_sqnorm + sqnorm;
        let sum_norm = self.sum_norm + norm;
        if !sum_sqnorm.is_finite() || !sum_norm.is_finite() {
            return Err(EngineError::Invalid("gradient statistics scalar overflow"));
        }
        for ((&value, &sum), &unit_sum) in sample.iter().zip(&self.sum).zip(&self.unit_sum) {
            let unit = if has_nonzero { value / norm } else { 0.0 };
            if !(sum + value).is_finite() || !(unit_sum + unit).is_finite() {
                return Err(EngineError::Invalid("gradient statistics vector overflow"));
            }
        }
        for ((&value, sum), unit_sum) in sample.iter().zip(&mut self.sum).zip(&mut self.unit_sum) {
            *sum += value;
            if has_nonzero {
                *unit_sum += value / norm;
            }
        }
        self.sum_sqnorm = sum_sqnorm;
        self.sum_norm = sum_norm;
        self.count = count;
        if has_nonzero {
            self.nonzero_count += 1;
        }
        Ok(())
    }

    pub fn finish(&self) -> Result<GradientStats> {
        if self.count < 2 {
            return Err(EngineError::Invalid(
                "gradient statistics needs two samples",
            ));
        }
        let n = self.count as f64;
        let mean_sqnorm: f64 = self.sum.iter().map(|value| (value / n).powi(2)).sum();
        let centered_sum = self.sum_sqnorm - n * mean_sqnorm;
        // The exact centered sum is nonnegative. Allow only relative floating
        // cancellation at this subtraction; never clamp the signal estimate.
        let tolerance = 64.0 * f64::EPSILON * self.sum_sqnorm;
        if !mean_sqnorm.is_finite() || !centered_sum.is_finite() || centered_sum < -tolerance {
            return Err(EngineError::Invalid("gradient statistics covariance range"));
        }
        let variance = centered_sum.max(0.0) / (n - 1.0);
        let mean_pairwise_cosine = if self.nonzero_count < 2 {
            None
        } else {
            let m = self.nonzero_count as f64;
            let unit_sqnorm: f64 = self.unit_sum.iter().map(|value| value * value).sum();
            let cosine = (unit_sqnorm - m) / (m * (m - 1.0));
            if !cosine.is_finite() || !(-1.0 - 1e-12..=1.0 + 1e-12).contains(&cosine) {
                return Err(EngineError::Invalid("gradient statistics cosine range"));
            }
            Some(cosine.clamp(-1.0, 1.0))
        };
        Ok(GradientStats {
            len: self.sum.len(),
            count: self.count,
            nonzero_count: self.nonzero_count,
            mean_l2: mean_sqnorm.sqrt(),
            mean_sample_l2: self.sum_norm / n,
            trace_sample_covariance: variance,
            standard_error_l2: (variance / n).sqrt(),
            unbiased_signal_norm_squared: mean_sqnorm - variance / n,
            mean_pairwise_cosine,
        })
    }
}

#[cfg(test)]
#[path = "stability_stats_tests.rs"]
mod tests;
