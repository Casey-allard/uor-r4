//! Paired statistics for the #845 measurement — the #843 harness arithmetic
//! (normal-approximation one-sided lower bounds, degenerate-case p-values,
//! standard-direction Holm step-down) generalized to f64 paired differences.
#![allow(dead_code)]

/// One-sided 95% normal quantile.
pub const Z95: f64 = 1.645;

/// One-sided 95% lower confidence bound on a paired difference:
/// (mean, standard error, lower bound). Deterministic; no bootstrap, no RNG.
pub fn paired_lower_bound(differences: &[f64]) -> (f64, f64, f64) {
    let n = differences.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let sum: f64 = differences.iter().sum();
    let mean = sum / n as f64;
    let variance: f64 = differences
        .iter()
        .map(|d| {
            let centred = d - mean;
            centred * centred
        })
        .sum::<f64>()
        / n as f64;
    let standard_error = (variance / n as f64).sqrt();
    (mean, standard_error, mean - Z95 * standard_error)
}

/// Standard normal upper tail.
fn upper_tail(z: f64) -> f64 {
    0.5 * libm::erfc(z / core::f64::consts::SQRT_2)
}

/// p-value for `H0: mean difference <= threshold`. A zero standard error is
/// degenerate: 0 when the point estimate clears the threshold, 1 otherwise.
pub fn p_value(mean: f64, standard_error: f64, threshold: f64) -> f64 {
    if standard_error == 0.0 {
        return if mean > threshold { 0.0 } else { 1.0 };
    }
    upper_tail((mean - threshold) / standard_error)
}

/// Holm–Bonferroni step-down in the standard direction at alpha = 0.05.
pub fn holm_pass(p_values: &[f64]) -> Vec<bool> {
    let m = p_values.len();
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|a, b| {
        p_values[*a]
            .partial_cmp(&p_values[*b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut pass = vec![false; m];
    for (rank, index) in order.iter().enumerate() {
        let level = 0.05 / (m - rank) as f64;
        if p_values[*index] <= level {
            pass[*index] = true;
        } else {
            break;
        }
    }
    pass
}
