//! Offline log-linear blend arithmetic in explicitly base-two units.
pub fn logsum2(values: &[f64]) -> f64 {
    assert!(!values.is_empty());
    let m = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    m + values.iter().map(|x| (x - m).exp2()).sum::<f64>().log2()
}
pub fn blend_bits(a: &[f64], c: &[f64], target: usize, beta: f64, stop_bits: f64) -> f64 {
    assert_eq!(a.len(), c.len());
    assert!((0.0..=2.0).contains(&beta));
    let z: Vec<f64> = a
        .iter()
        .zip(c)
        .map(|(a, c)| (1.0 - beta) * a + beta * c)
        .collect();
    stop_bits + logsum2(&z) - z[target]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blend_preserves_stop_mass_and_both_endpoints() {
        let a = [0.75f64.log2(), 0.25f64.log2()];
        let c = [0.2f64.log2(), 0.8f64.log2()];
        let stop = -0.9f64.log2();
        assert!((blend_bits(&a, &c, 1, 0.0, stop) + 0.225f64.log2()).abs() < 1e-12);
        assert!((blend_bits(&a, &c, 1, 1.0, stop) + 0.72f64.log2()).abs() < 1e-12);
        let mass: f64 = (0..2)
            .map(|t| (-blend_bits(&a, &c, t, 0.45, stop)).exp2())
            .sum();
        assert!((mass - 0.9).abs() < 1e-12);
    }
    #[test]
    fn blend_is_invariant_to_a_common_logit_offset() {
        let c = [-3.0, -0.19264507794239588];
        assert!(
            (blend_bits(&[1000., 1002.], &c, 1, 0.35, 0.0)
                - blend_bits(&[0., 2.], &c, 1, 0.35, 0.0))
            .abs()
                < 1e-10
        );
    }
}

#[cfg(test)]
#[test]
fn count_temperature_can_sharpen_as_well_as_flatten() {
    let c = [0.2f64.log2(), 0.8f64.log2()];
    let p = (-blend_bits(&[0.0, 0.0], &c, 1, 2.0, 0.0)).exp2();
    assert!((p - 16.0 / 17.0).abs() < 1e-12);
}

/// Construct endpoints whose interpolation independently varies residual strength.
pub fn residual_endpoints(
    a: &[f64],
    c: &[f64],
    alpha: f64,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    if a.len() != c.len() || a.is_empty() || !alpha.is_finite() || !(0.0..=2.0).contains(&alpha) {
        return Err("invalid residual endpoint dimensions or exponent".into());
    }
    let scaled: Vec<f64> = c.iter().map(|x| alpha * x).collect();
    let normalizer = logsum2(&scaled);
    let right: Vec<f64> = scaled.iter().map(|x| x - normalizer).collect();
    let left = a.iter().zip(&right).map(|(a, c)| a + c).collect();
    Ok((left, right))
}
#[cfg(test)]
#[test]
fn residual_strength_does_not_reduce_count_exponent() {
    let a = [0.7f64.log2(), 0.3f64.log2()];
    let c = [0.2f64.log2(), 0.8f64.log2()];
    let (left, right) = residual_endpoints(&a, &c, 1.1).unwrap();
    for gamma in [0.0, 0.15, 1.0] {
        let z: Vec<f64> = (0..2).map(|i| 1.1 * c[i] + gamma * a[i]).collect();
        assert!(
            (blend_bits(&left, &right, 1, 1.0 - gamma, 0.0) - (logsum2(&z) - z[1])).abs() < 1e-12
        );
    }
    assert!(residual_endpoints(&a, &c, f64::NAN).is_err());
}
