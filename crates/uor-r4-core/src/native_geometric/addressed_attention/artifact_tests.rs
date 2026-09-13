use super::super::circuit::{CompiledCircuit, CompiledHeads, Topology, GATE_LOGITS, HEAD_LOGITS};
use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn fixture() -> Result<Model> {
    let compiled = PrimitiveExport {
        circuit: CompiledCircuit::compile(
            Topology::seeded(super::super::policy::WIRING_SEED),
            &vec![0.0; GATE_LOGITS],
        )?,
        heads: CompiledHeads::compile(&vec![0.0; HEAD_LOGITS])?,
    };
    Model::new(
        compiled,
        Provenance {
            seed: 973,
            training_data_digest: [1; 32],
            training_config_digest: [2; 32],
            parameter_digest: [3; 32],
            source_digest: super::super::policy::implementation_digest(),
            parent: None,
        },
    )
}
fn reseal(bytes: &mut [u8]) {
    let n = bytes.len() - 32;
    let id = digest(b"uor-addressed-model-v1", &bytes[..n]);
    bytes[n..].copy_from_slice(&id);
}

#[test]
fn canonical_geometry_byte_endpoints_group_scores_and_predicates() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let native = crate::native_geometric::training::geometry(258, 256)?;
    assert_eq!(geometry.identity(), native.identity);
    assert_eq!(geometry.primes[0], 2);
    for byte in [0u8, 1, 254, 255] {
        let token = &native.tokens[usize::from(byte) + 2];
        assert_eq!(
            geometry.byte_phases(byte),
            [
                token.phases[0],
                token.phases[1],
                token.phases[2],
                token.phases[3]
            ]
        );
        assert_eq!(geometry.primes[usize::from(byte) + 2], token.prime);
    }
    for root in 0..120u16 {
        assert_eq!(geometry.product(root, geometry.identity())?, root);
        assert_eq!(
            geometry.product(root, geometry.inverses[usize::from(root)])?,
            geometry.identity()
        );
        let signature = geometry.signature(root)?;
        assert_eq!(signature[1] >> 56, 0);
        for key in 0..120u16 {
            let product = geometry.product(root, geometry.inverses[usize::from(key)])?;
            let rank = geometry.ranks[usize::from(product)];
            assert_eq!(geometry.score([root; 2], [key; 2])?, rank + rank);
            assert_eq!(
                (signature[usize::from(key) >> 6] >> (key & 63)) & 1,
                u64::from(rank > 0)
            );
        }
    }
    assert_eq!(geometry.product(120, 0), Err(ArtifactError::Domain));
    assert_eq!(geometry.product(0, u16::MAX), Err(ArtifactError::Domain));
    assert_eq!(geometry.score([0, 120], [0; 2]), Err(ArtifactError::Domain));
    assert_eq!(geometry.signature(120), Err(ArtifactError::Domain));
    assert_ne!(geometry.identity_digest(), [0; 32]);
    Ok(())
}

#[test]
fn canonical_model_roundtrip_and_strict_envelope_fail_closed() -> TestResult {
    let model = fixture()?;
    let bytes = model.encode();
    assert!(bytes.len() < MAX_ARTIFACT_BYTES);
    assert!(model.initialized_parameters_no_training());
    assert!(!model.paired_h4_implemented());
    assert!(!model.durable_writer_implemented());
    let restored = Model::decode(&bytes)?;
    assert_eq!(restored, model);
    assert_eq!(restored.encode(), bytes);
    assert_eq!(
        restored.geometry().identity_digest(),
        model.geometry().identity_digest()
    );
    assert_eq!(restored.compiled().encode(), model.compiled().encode());
    for cut in [0, 1, 8, 31, bytes.len() - 1] {
        assert!(Model::decode(&bytes[..cut]).is_err());
    }
    let mut changed = bytes.clone();
    changed.push(0);
    assert!(Model::decode(&changed).is_err());
    let mut changed = bytes.clone();
    changed[8] ^= 1;
    reseal(&mut changed);
    assert_eq!(Model::decode(&changed), Err(ArtifactError::Version));
    let mut changed = bytes.clone();
    changed[18] = 0;
    reseal(&mut changed);
    assert_eq!(Model::decode(&changed), Err(ArtifactError::Version));
    // Fixed metadata header: magic + six u16 tags + seed + four digests + parent.
    let geometry_length_offset = 8 + 12 + 8 + 128 + 1 + 32;
    let geometry_start = geometry_length_offset + 4;
    let mut changed = bytes.clone();
    changed[geometry_start + 42] ^= 1;
    reseal(&mut changed);
    assert_eq!(Model::decode(&changed), Err(ArtifactError::Geometry));
    let mut changed = bytes.clone();
    changed[geometry_length_offset..geometry_start].copy_from_slice(&u32::MAX.to_le_bytes());
    reseal(&mut changed);
    assert!(Model::decode(&changed).is_err());
    let mut changed = bytes.clone();
    changed[8 + 12 + 8 + 96] ^= 1;
    reseal(&mut changed);
    assert_eq!(Model::decode(&changed), Err(ArtifactError::Provenance));
    let mut changed = bytes.clone();
    let body_end = changed.len() - 32;
    changed.insert(body_end, 0);
    reseal(&mut changed);
    assert_eq!(Model::decode(&changed), Err(ArtifactError::Wire));
    assert_eq!(
        Model::decode(&vec![0; MAX_ARTIFACT_BYTES + 1]),
        Err(ArtifactError::Limit)
    );
    let mut provenance = model.provenance().clone();
    provenance.parent = Some(model.id());
    let child = Model::new(model.compiled().clone(), provenance)?;
    assert_ne!(child.id(), model.id());
    assert_eq!(
        Model::decode(&child.encode())?.provenance().parent,
        Some(model.id())
    );
    // Wiring is fixed by the machine contract, independently of parameter seed.
    let mut wrong_compiled = model.compiled().clone();
    wrong_compiled.circuit = CompiledCircuit::compile(
        Topology::seeded(super::super::policy::WIRING_SEED + 1),
        &vec![0.0; GATE_LOGITS],
    )?;
    assert_eq!(
        Model::new(wrong_compiled.clone(), model.provenance().clone()),
        Err(ArtifactError::Circuit(CircuitError::Topology))
    );
    let mut wrong_model = model.clone();
    wrong_model.compiled = wrong_compiled;
    let mut wrong_wire = wrong_model.encode();
    reseal(&mut wrong_wire);
    assert_eq!(
        Model::decode(&wrong_wire),
        Err(ArtifactError::Circuit(CircuitError::Topology))
    );
    Ok(())
}
