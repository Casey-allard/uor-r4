use uor_r4_core::transformerless::runtime::OpKernel;
use uor_r4_graph_certify::performance_certificate::{
    CpuPortabilityRecord, PerformanceCertificate, PerformanceProfiler,
    RuntimePerformanceCertificate,
};

#[test]
fn test_performance_profiler_and_threshold_verification() {
    let ops = OpKernel {
        adds: 100,
        compares: 64,
        candidate_scans: 32,
        ..Default::default()
    };

    let cert = PerformanceProfiler::profile(2048, 4, ops, 4096);

    assert_eq!(cert.metrics.bytes_read, 2048);
    assert_eq!(cert.metrics.cache_miss_estimate, (2048 / 64) + 4);
    assert_eq!(cert.metrics.branch_miss_estimate, (64 / 32) + (32 / 16));
    assert!(cert.baseline_threshold_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_performance_certificate_threshold_failure() {
    let ops = OpKernel::default();
    let cert = PerformanceProfiler::profile(8192, 10, ops, 4096); // 8192 > 4096 limit

    assert!(!cert.baseline_threshold_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_performance_certificate_cbor_roundtrip() {
    let ops = OpKernel::default();
    let cert = PerformanceProfiler::profile(1024, 2, ops, 2048);

    let cbor_bytes = cert.to_cbor_bytes().expect("serialize CBOR");
    let decoded = PerformanceCertificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");

    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}

#[test]
fn test_runtime_performance_certificate_cid_is_content_addressed() {
    let cert = RuntimePerformanceCertificate::new();
    assert!(cert.verify_cid());

    let mut tampered = cert.clone();
    tampered.steady_state_allocations = 1;
    assert!(!tampered.verify_cid());
}

#[test]
fn test_runtime_performance_certificate_requires_all_declared_zero_links() {
    let mut cert = RuntimePerformanceCertificate::new();
    cert.declared_zero_fields.zero_fma_link.clear();
    cert.certificate_cid = cert.compute_cid();

    assert!(!cert.verify_evidence_links());
}

#[test]
fn test_cpu_portability_defaults_to_current_architecture_tier() {
    let record = CpuPortabilityRecord::default();
    assert_eq!(
        record.target_tier,
        format!("{}-scalar-portable", std::env::consts::ARCH)
    );
    assert!(!record.minimum_isa_requirements.is_empty());
    assert!(record.scalar_fallback_confirmed);
}
