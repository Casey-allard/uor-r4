use super::*;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "actual={actual}, expected={expected}"
    );
}

#[test]
fn stability_orthogonal_vectors_have_known_covariance() -> Result<()> {
    let mut stream = Stats::new(2)?;
    stream.add(&[1.0, 0.0])?;
    stream.add(&[0.0, 1.0])?;
    let stats = stream.finish()?;
    assert_eq!((stats.len, stats.count, stats.nonzero_count), (2, 2, 2));
    near(stats.mean_l2, 0.5_f64.sqrt());
    near(stats.mean_sample_l2, 1.0);
    near(stats.trace_sample_covariance, 1.0);
    near(stats.standard_error_l2, 0.5_f64.sqrt());
    near(stats.unbiased_signal_norm_squared, 0.0);
    assert_eq!(stats.mean_pairwise_cosine, Some(0.0));
    Ok(())
}

#[test]
fn stability_identical_and_opposite_vectors_preserve_signed_signal() -> Result<()> {
    let mut identical = Stats::new(2)?;
    identical.add(&[2.0, 0.0])?;
    identical.add(&[2.0, 0.0])?;
    let stats = identical.finish()?;
    near(stats.mean_l2, 2.0);
    near(stats.mean_sample_l2, 2.0);
    near(stats.trace_sample_covariance, 0.0);
    near(stats.standard_error_l2, 0.0);
    near(stats.unbiased_signal_norm_squared, 4.0);
    assert_eq!(stats.mean_pairwise_cosine, Some(1.0));
    let mut opposite = Stats::new(1)?;
    opposite.add(&[1.0])?;
    opposite.add(&[-1.0])?;
    let stats = opposite.finish()?;
    near(stats.mean_l2, 0.0);
    near(stats.trace_sample_covariance, 2.0);
    near(stats.standard_error_l2, 1.0);
    near(stats.unbiased_signal_norm_squared, -1.0);
    assert_eq!(stats.mean_pairwise_cosine, Some(-1.0));
    Ok(())
}

#[test]
fn stability_zero_vectors_count_for_variance_but_not_directions() -> Result<()> {
    let mut stream = Stats::new(1)?;
    stream.add(&[0.0])?;
    stream.add(&[0.0])?;
    let zeros = stream.finish()?;
    near(zeros.mean_l2, 0.0);
    near(zeros.trace_sample_covariance, 0.0);
    assert_eq!(zeros.mean_pairwise_cosine, None);
    stream.add(&[3.0])?;
    let one = stream.finish()?;
    assert_eq!((one.count, one.nonzero_count), (3, 1));
    near(one.mean_l2, 1.0);
    near(one.mean_sample_l2, 1.0);
    near(one.trace_sample_covariance, 3.0);
    near(one.standard_error_l2, 1.0);
    near(one.unbiased_signal_norm_squared, 0.0);
    assert_eq!(one.mean_pairwise_cosine, None);
    stream.add(&[-3.0])?;
    assert_eq!(stream.finish()?.mean_pairwise_cosine, Some(-1.0));
    Ok(())
}

#[test]
fn stability_rejected_samples_are_transactional() -> Result<()> {
    assert!(Stats::new(0).is_err());
    let mut stream = Stats::new(2)?;
    assert!(stream.finish().is_err());
    stream.add(&[1.0, 2.0])?;
    assert!(stream.finish().is_err());
    stream.add(&[3.0, 4.0])?;
    let before = stream.finish()?;
    for sample in [
        &[1.0][..],
        &[f64::NAN, 1.0],
        &[1.0, f64::INFINITY],
        &[f64::MAX, 0.0],
        &[f64::MIN_POSITIVE, 0.0],
    ] {
        assert!(stream.add(sample).is_err());
        assert_eq!(stream.finish()?, before);
    }
    Ok(())
}
