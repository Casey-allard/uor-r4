use uor_r4_graph_certify::certificate::{
    Certificate, ClaimKind, EmpiricalClaim, ProtocolAttestation,
};

#[test]
fn test_certificate_cid_computation_and_verification() {
    let claim = EmpiricalClaim {
        name: "held_out_accuracy".to_string(),
        sample_size: 10000,
        metric_value: 0.942,
        confidence_interval_95: (0.935, 0.949),
        slice_label: "test_set_v1".to_string(),
        claim_kind: ClaimKind::Empirical,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert = Certificate::new(
        "kappa:blake3:source_123",
        "kappa:blake3:corpus_456",
        "kappa:blake3:graph_789",
        "kappa:blake3:metric_abc",
        "kappa:blake3:op_def",
        "kappa:blake3:benchmark_ghi",
        vec![claim],
        attestation,
    );

    assert!(cert.verify_cid(), "Certificate CID verification must pass");
    assert!(
        cert.certificate_cid.starts_with("kappa:blake3:"),
        "Certificate CID must carry kappa scheme prefix"
    );
    assert!(
        cert.verify_attestation().is_none(),
        "Structural attestation checks must pass"
    );
}

#[test]
fn test_certificate_cbor_roundtrip() {
    let claim = EmpiricalClaim {
        name: "zero_alloc_step".to_string(),
        sample_size: 50000,
        metric_value: 0.0,
        confidence_interval_95: (0.0, 0.0),
        slice_label: "census_suite".to_string(),
        claim_kind: ClaimKind::Performance,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert = Certificate::new(
        "kappa:blake3:source_test",
        "kappa:blake3:corpus_test",
        "kappa:blake3:graph_test",
        "kappa:blake3:metric_test",
        "kappa:blake3:op_test",
        "kappa:blake3:benchmark_test",
        vec![claim],
        attestation,
    );

    let cbor_bytes = cert.to_cbor_bytes();
    assert!(!cbor_bytes.is_empty());

    let decoded = Certificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");
    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}

#[test]
fn test_certificate_attestation_failure() {
    let cert = Certificate::new(
        "kappa:blake3:source",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![],
        ProtocolAttestation {
            deterministic_canonical_mode: true,
            zero_allocation_verified: false, // Intentionally false!
            no_multiply_verified: true,
            theorem_7_reverse_index_verified: true,
        },
    );

    assert!(
        cert.verify_attestation().is_some(),
        "Attestation check must fail when zero_allocation_verified is false"
    );
}

#[test]
fn test_deterministic_certificate_rebuild() {
    let claim = EmpiricalClaim {
        name: "gate_e_rebuild".to_string(),
        sample_size: 1000,
        metric_value: 1.0,
        confidence_interval_95: (1.0, 1.0),
        slice_label: "ci_rebuild".to_string(),
        claim_kind: ClaimKind::Structural,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert1 = Certificate::new(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![claim.clone()],
        attestation.clone(),
    );

    let cert2 = Certificate::new(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![claim],
        attestation,
    );

    assert_eq!(
        cert1, cert2,
        "Certificates built from identical inputs must be equal"
    );
    assert_eq!(
        cert1.compute_cid(),
        cert2.compute_cid(),
        "Certificate CIDs must match"
    );
    assert_eq!(
        cert1.to_cbor_bytes(),
        cert2.to_cbor_bytes(),
        "Certificate CBOR bytes must be byte-identical"
    );
}

#[test]
fn test_parallel_claim_fragment_assembly_is_canonical_and_thread_invariant() {
    let mut fragments = vec![
        EmpiricalClaim {
            name: "z-claim".to_string(),
            sample_size: 10,
            metric_value: 0.1,
            confidence_interval_95: (0.0, 0.2),
            slice_label: "s1".to_string(),
            claim_kind: ClaimKind::Empirical,
        },
        EmpiricalClaim {
            name: "a-claim".to_string(),
            sample_size: 20,
            metric_value: 0.9,
            confidence_interval_95: (0.8, 1.0),
            slice_label: "s2".to_string(),
            claim_kind: ClaimKind::Structural,
        },
        EmpiricalClaim {
            name: "a-claim".to_string(),
            sample_size: 20,
            metric_value: 0.9,
            confidence_interval_95: (0.8, 1.0),
            slice_label: "s2".to_string(),
            claim_kind: ClaimKind::Structural,
        },
    ];
    fragments.swap(0, 1);

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert_seq = Certificate::new_from_claim_fragments(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        &fragments,
        attestation.clone(),
        1,
    );
    let cert_par = Certificate::new_from_claim_fragments(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        &fragments,
        attestation,
        4,
    );

    assert_eq!(cert_seq.claims.len(), 2);
    assert_eq!(cert_seq, cert_par);
    assert_eq!(cert_seq.claims[0].name, "a-claim");
}

#[test]
fn test_parallel_claim_fragment_threads_zero_matches_sequential() {
    let fragments = vec![
        EmpiricalClaim {
            name: "x".to_string(),
            sample_size: 1,
            metric_value: 0.1,
            confidence_interval_95: (0.0, 0.2),
            slice_label: "slice".to_string(),
            claim_kind: ClaimKind::Empirical,
        },
        EmpiricalClaim {
            name: "y".to_string(),
            sample_size: 2,
            metric_value: 0.2,
            confidence_interval_95: (0.1, 0.3),
            slice_label: "slice".to_string(),
            claim_kind: ClaimKind::Structural,
        },
    ];
    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert_seq = Certificate::new_from_claim_fragments(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        &fragments,
        attestation.clone(),
        1,
    );
    let cert_auto = Certificate::new_from_claim_fragments(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        &fragments,
        attestation,
        0,
    );

    assert_eq!(cert_seq, cert_auto);
}
