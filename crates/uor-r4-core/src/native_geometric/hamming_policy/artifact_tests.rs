use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn altered(artifact: &Artifact, mutation: impl FnOnce(&mut Payload)) -> Result<Vec<u8>> {
    let mut payload = artifact.payload.clone();
    mutation(&mut payload);
    let checksum = checksum(&payload)?;
    serde_json::to_vec(&Envelope { payload, checksum }).map_err(|_| ArtifactError::Wire)
}

#[test]
fn source_bound_parameter_artifact_round_trip_and_finite_intervention() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let params = Params::seeded(973);
    let artifact = Artifact::new(params.clone(), &geometry)?;
    let encoded = artifact.encode()?;
    assert!(encoded.len() < MAX_ARTIFACT_BYTES);
    let restored = Artifact::decode(&encoded, &geometry)?;
    assert_eq!(restored.id(), artifact.id());
    assert_eq!(restored.params(), &params);
    assert_eq!(restored.encode()?, encoded);
    assert_eq!(restored.geometry_digest(), geometry.identity_digest());
    assert_ne!(implementation_digest(), [0; 32]);
    let mut intervention = params;
    intervention.flip(0, 0)?;
    let changed = Artifact::new(intervention, &geometry)?;
    assert_ne!(changed.id(), artifact.id());
    assert_eq!(changed.payload.fit_calls, 0);
    assert_eq!(changed.payload.optimizer_updates, 0);
    assert_eq!(changed.payload.training_status, STATUS);
    Ok(())
}

#[test]
fn resealed_version_configuration_geometry_and_source_changes_are_rejected() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let artifact = Artifact::new(Params::seeded(973), &geometry)?;
    let cases: Vec<(Vec<u8>, ArtifactError)> = vec![
        (
            altered(&artifact, |p| p.version += 1)?,
            ArtifactError::Version,
        ),
        (
            altered(&artifact, |p| p.schema.push('x'))?,
            ArtifactError::Version,
        ),
        (
            altered(&artifact, |p| p.configuration.refinements = 3)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.configuration.topology[2] = 9)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.configuration.emission_output[0] += 1)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.configuration.eos = 255)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.configuration.geometric_decoder.push('x'))?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.fit_calls = 1)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.optimizer_updates = 1)?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.training_status = "TRAINED".into())?,
            ArtifactError::Configuration,
        ),
        (
            altered(&artifact, |p| p.geometry_digest[0] ^= 1)?,
            ArtifactError::Geometry,
        ),
        (
            altered(&artifact, |p| p.source_digest[0] ^= 1)?,
            ArtifactError::Source,
        ),
    ];
    for (encoded, expected) in cases {
        assert_eq!(Artifact::decode(&encoded, &geometry).err(), Some(expected));
    }
    Ok(())
}

#[test]
fn parameter_checksum_unknown_fields_and_wire_limits_are_rejected() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let artifact = Artifact::new(Params::seeded(973), &geometry)?;
    for changed in [
        altered(&artifact, |p| {
            p.parameters.tables.pop();
        })?,
        altered(&artifact, |p| p.parameters.tables[0] ^= 1)?,
        altered(&artifact, |p| p.parameters.seed ^= 1)?,
        altered(&artifact, |p| p.parameter_digest[0] ^= 1)?,
    ] {
        assert_eq!(
            Artifact::decode(&changed, &geometry).err(),
            Some(ArtifactError::Parameters)
        );
    }
    let bytes = artifact.encode()?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    value["checksum"][0] = serde_json::json!(255 - artifact.id()[0]);
    assert_eq!(
        Artifact::decode(&serde_json::to_vec(&value)?, &geometry).err(),
        Some(ArtifactError::Checksum)
    );
    for path in ["envelope", "payload", "configuration", "parameters"] {
        let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let target = match path {
            "envelope" => &mut value,
            "payload" => &mut value["payload"],
            "configuration" => &mut value["payload"]["configuration"],
            _ => &mut value["payload"]["parameters"],
        };
        target["undeclared"] = serde_json::json!(true);
        assert_eq!(
            Artifact::decode(&serde_json::to_vec(&value)?, &geometry).err(),
            Some(ArtifactError::Wire)
        );
    }
    assert_eq!(
        Artifact::decode(&bytes[..bytes.len() - 1], &geometry).err(),
        Some(ArtifactError::Wire)
    );
    assert_eq!(
        Artifact::decode(&vec![b' '; MAX_ARTIFACT_BYTES + 1], &geometry).err(),
        Some(ArtifactError::Limit)
    );
    Ok(())
}
