use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn reseal(bytes: &mut [u8]) {
    let end = bytes.len() - 32;
    let checksum = wire_digest(&bytes[..end]);
    bytes[end..].copy_from_slice(&checksum);
}

#[test]
fn sgd_preserves_parent_and_updates_unclipped_with_descent_sign() -> TestResult {
    let parameters = Parameters::from_values(7341, vec![0.0; PARAMETER_COUNT])?;
    let before = parameters.digest();
    let mut gradient = vec![0.0; PARAMETER_COUNT];
    gradient[0] = 2.0;
    gradient[1] = -4.0;
    gradient[PARAMETER_COUNT - 1] = 100.0;
    let next = sgd(&parameters, &gradient, 0.05)?;
    assert_eq!(parameters.digest(), before);
    assert_eq!(next.seed(), parameters.seed());
    assert_eq!(next.values()[0], -0.1);
    assert_eq!(next.values()[1], 0.2);
    assert_eq!(next.values()[PARAMETER_COUNT - 1], -5.0);
    assert!(
        gradient
            .iter()
            .zip(next.values())
            .map(|(g, change)| g * change)
            .sum::<f64>()
            < 0.0
    );
    assert!(sgd(&parameters, &gradient[..PARAMETER_COUNT - 1], 0.05).is_err());
    for rate in [0.0, -0.1, f64::NAN, f64::INFINITY] {
        assert!(sgd(&parameters, &gradient, rate).is_err());
    }
    gradient[2] = f64::NAN;
    assert!(sgd(&parameters, &gradient, 0.05).is_err());
    gradient[2] = f64::MAX;
    assert!(sgd(&parameters, &gradient, 2.0).is_err());
    assert_eq!(parameters.digest(), before);
    Ok(())
}

#[test]
fn checkpoint_bitwise_roundtrip_and_resealed_invalid_payloads_rejected() -> TestResult {
    let mut values = vec![0.0; PARAMETER_COUNT];
    values[0] = -0.0;
    values[1] = f64::from_bits(1);
    values[PARAMETER_COUNT - 1] = -0.25;
    let parameters = Parameters::from_values(7341, values)?;
    let checkpoint = Checkpoint::new(parameters, [1; 32], [2; 32], 973, 64)?;
    let bytes = checkpoint.encode()?;
    assert_eq!(bytes.len(), CHECKPOINT_BYTES);
    let restored = Checkpoint::decode(&bytes, [1; 32], [2; 32], 973)?;
    assert_eq!(restored.completed_updates, 64);
    assert_eq!(restored.parameters.seed(), 7341);
    assert_eq!(restored.parameters.digest(), checkpoint.parameters.digest());
    assert!(restored
        .parameters
        .values()
        .iter()
        .zip(checkpoint.parameters.values())
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert_eq!(restored.encode()?, bytes);
    assert!(Checkpoint::decode(&bytes, [3; 32], [2; 32], 973).is_err());
    assert!(Checkpoint::decode(&bytes, [1; 32], [3; 32], 973).is_err());
    assert!(Checkpoint::decode(&bytes, [1; 32], [2; 32], 974).is_err());
    assert!(Checkpoint::decode(&bytes[..bytes.len() - 1], [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    changed.push(0);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    changed[10] ^= 1;
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let parameter_start = 8 + 2 + 32 * 4 + 8 * 3 + 4 + 32;
    let mut changed = bytes.clone();
    changed[parameter_start..parameter_start + 8]
        .copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    changed[parameter_start + 8] ^= 1;
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    let step_offset = 8 + 2 + 32 * 4 + 8 * 2;
    changed[step_offset..step_offset + 8].copy_from_slice(&65u64.to_le_bytes());
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    Ok(())
}

#[test]
fn terminal_eos_batch_scores_every_position_and_keeps_parameters_frozen() -> TestResult {
    let parameters = Parameters::seeded(7341)?;
    let geometry = BoundGeometry::canonical()?;
    let before = parameters.digest();
    for document in [&[][..], &[1u16][..], &[256, 1, 256][..], &[257, 256][..]] {
        assert!(batch(&parameters, &geometry, document, 973, 0).is_err());
    }
    assert!(batch(&parameters, &geometry, &[256; 65], 973, 0).is_err());
    let result = batch(&parameters, &geometry, &[u16::from(b'A'), 256], 973, 0)?;
    assert_eq!(result.counts.scored_positions, 8);
    assert_eq!(
        result.counts.gate_events,
        result.counts.circuit_calls * super::super::circuit::GATES as u64
    );
    assert!(result.mean_ce.is_finite() && result.mean_ce > 0.0);
    assert_eq!(result.gradient.len(), PARAMETER_COUNT);
    assert!(result.gradient.iter().all(|v| v.is_finite()));
    assert!(result.gradient.iter().any(|v| *v != 0.0));
    assert_eq!(before, parameters.digest());
    Ok(())
}
