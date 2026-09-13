use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn reseal(bytes: &mut [u8]) {
    let end = bytes.len() - 32;
    let checksum = wire_digest(&bytes[..end]);
    bytes[end..].copy_from_slice(&checksum);
}

#[test]
fn causal_checkpoint_preserves_bits_and_refuses_cross_codec_or_source() -> TestResult {
    let mut values = vec![0.0; PARAMETER_COUNT];
    values[0] = -0.0;
    values[1] = f64::from_bits(1);
    values[PARAMETER_COUNT - 1] = -0.25;
    let parameters = Parameters::from_values(7341, values)?;
    let checkpoint = Checkpoint::new(parameters.clone(), [1; 32], [2; 32], 973, 64)?;
    let bytes = checkpoint.encode()?;
    assert_eq!(bytes.len(), CHECKPOINT_BYTES);
    let restored = Checkpoint::decode(&bytes, [1; 32], [2; 32], 973)?;
    assert_eq!(restored.completed_updates, 64);
    assert_eq!(restored.parameters.seed(), 7341);
    assert_eq!(restored.parameters.digest(), parameters.digest());
    assert!(restored
        .parameters
        .values()
        .iter()
        .zip(parameters.values())
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert_eq!(restored.encode()?, bytes);
    let old =
        super::super::learner::Checkpoint::new(parameters, [1; 32], [2; 32], 973, 64)?.encode()?;
    assert!(Checkpoint::decode(&old, [1; 32], [2; 32], 973).is_err());
    assert!(super::super::learner::Checkpoint::decode(&bytes, [1; 32], [2; 32], 973).is_err());
    // Even an intentionally recomputed checksum cannot turn old metadata into
    // this format or disguise a changed estimator/SGD implementation identity.
    let mut old_resealed = old;
    reseal(&mut old_resealed);
    assert!(Checkpoint::decode(&old_resealed, [1; 32], [2; 32], 973).is_err());
    for offset in [8usize, 10, 42] {
        let mut changed = bytes.clone();
        changed[offset] ^= 1;
        reseal(&mut changed);
        assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    }
    Ok(())
}

#[test]
fn causal_checkpoint_rejects_resealed_bad_values_and_experiment_identity() -> TestResult {
    let parameters = Parameters::from_values(7341, vec![0.0; PARAMETER_COUNT])?;
    let checkpoint = Checkpoint::new(parameters, [1; 32], [2; 32], 973, 0)?;
    let bytes = checkpoint.encode()?;
    assert!(Checkpoint::decode(&bytes, [3; 32], [2; 32], 973).is_err());
    assert!(Checkpoint::decode(&bytes, [1; 32], [3; 32], 973).is_err());
    assert!(Checkpoint::decode(&bytes, [1; 32], [2; 32], 974).is_err());
    assert!(Checkpoint::decode(&bytes[..bytes.len() - 1], [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    changed.push(0);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let parameter_start = 8 + 2 + 32 * 4 + 8 * 3 + 4 + 32;
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut changed = bytes.clone();
        changed[parameter_start..parameter_start + 8]
            .copy_from_slice(&invalid.to_bits().to_le_bytes());
        reseal(&mut changed);
        assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    }
    let mut changed = bytes.clone();
    changed[parameter_start + 8] ^= 1;
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    let step_offset = 8 + 2 + 32 * 4 + 8 * 2;
    changed[step_offset..step_offset + 8].copy_from_slice(&65u64.to_le_bytes());
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    let mut changed = bytes.clone();
    let count_offset = 8 + 2 + 32 * 4 + 8 * 3;
    changed[count_offset..count_offset + 4]
        .copy_from_slice(&((PARAMETER_COUNT - 1) as u32).to_le_bytes());
    reseal(&mut changed);
    assert!(Checkpoint::decode(&changed, [1; 32], [2; 32], 973).is_err());
    assert!(Checkpoint::new(checkpoint.parameters.clone(), [0; 32], [2; 32], 973, 0).is_err());
    assert!(Checkpoint::new(checkpoint.parameters, [1; 32], [2; 32], 973, 65).is_err());
    Ok(())
}
