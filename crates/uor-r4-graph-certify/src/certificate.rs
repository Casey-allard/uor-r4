//! Machine-checkable Certificate schema for empirical and structural claims
//! emitted by the R4 holographic graph compiler.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_graph_compiler::executor::RayonExecutor;
use uor_r4_graph_compiler::executor::{CompilerExecutor, SequentialExecutor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimKind {
    Structural,
    Empirical,
    Performance,
    Safety,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpiricalClaim {
    pub name: String,
    pub sample_size: u64,
    pub metric_value: f64,
    pub confidence_interval_95: (f64, f64),
    pub slice_label: String,
    pub claim_kind: ClaimKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProtocolAttestation {
    pub deterministic_canonical_mode: bool,
    pub zero_allocation_verified: bool,
    pub no_multiply_verified: bool,
    pub theorem_7_reverse_index_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    pub version: u32,
    pub certificate_cid: String,
    pub source_cid: String,
    pub corpus_cid: String,
    pub graph_cid: String,
    pub metric_cid: String,
    pub op_cid: String,
    pub benchmark_cid: String,
    pub claims: Vec<EmpiricalClaim>,
    pub attestation: ProtocolAttestation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateError {
    SerializationError(String),
    DeserializationError(String),
    CidMismatch { expected: String, actual: String },
    AttestationFailed(String),
}

impl Certificate {
    // Args mirror the schema's CID fields one-to-one (issue #20); a params
    // struct can replace them if the schema grows further.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_cid: impl Into<String>,
        corpus_cid: impl Into<String>,
        graph_cid: impl Into<String>,
        metric_cid: impl Into<String>,
        op_cid: impl Into<String>,
        benchmark_cid: impl Into<String>,
        claims: Vec<EmpiricalClaim>,
        attestation: ProtocolAttestation,
    ) -> Self {
        let mut cert = Certificate {
            version: 1,
            certificate_cid: String::new(),
            source_cid: source_cid.into(),
            corpus_cid: corpus_cid.into(),
            graph_cid: graph_cid.into(),
            metric_cid: metric_cid.into(),
            op_cid: op_cid.into(),
            benchmark_cid: benchmark_cid.into(),
            claims,
            attestation,
        };
        cert.certificate_cid = cert.compute_cid();
        cert
    }

    /// Canonical constructor for claims assembled from parallel shard fragments.
    ///
    /// Claims are copied in parallel and then canonically sorted/deduplicated by
    /// stable content keys before CID materialization.
    #[allow(clippy::too_many_arguments)]
    pub fn new_from_claim_fragments(
        source_cid: impl Into<String>,
        corpus_cid: impl Into<String>,
        graph_cid: impl Into<String>,
        metric_cid: impl Into<String>,
        op_cid: impl Into<String>,
        benchmark_cid: impl Into<String>,
        claim_fragments: &[EmpiricalClaim],
        attestation: ProtocolAttestation,
        threads: usize,
    ) -> Result<Self, CertificateError> {
        let mut claims = Self::claims_from_fragments_with_threads(claim_fragments, threads)
            .map_err(|e| {
                CertificateError::SerializationError(format!("claim-fragment assembly failed: {e}"))
            })?;
        Self::canonical_sort_and_dedup_claims(&mut claims);
        Ok(Self::new(
            source_cid,
            corpus_cid,
            graph_cid,
            metric_cid,
            op_cid,
            benchmark_cid,
            claims,
            attestation,
        ))
    }

    /// Compute self-referential BLAKE3 CID (hex format) over certificate content.
    pub fn compute_cid(&self) -> String {
        let mut clone = self.clone();
        clone.certificate_cid.clear();

        let mut bytes = Vec::new();
        ciborium::into_writer(&clone, &mut bytes)
            .expect("certificate CBOR serialization must succeed");

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        format!("kappa:blake3:{}", hasher.finalize().to_hex())
    }

    fn claims_from_fragments_with_threads(
        claim_fragments: &[EmpiricalClaim],
        threads: usize,
    ) -> Result<Vec<EmpiricalClaim>, String> {
        let indices: Vec<usize> = (0..claim_fragments.len()).collect();
        if threads == 1 {
            return SequentialExecutor::new()
                .map(&indices, |&idx| Ok(claim_fragments[idx].clone()))
                .map_err(|e| e.to_string());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            RayonExecutor::new(threads)
                .and_then(|executor| {
                    executor.map(&indices, |&idx| Ok(claim_fragments[idx].clone()))
                })
                .map_err(|e| e.to_string())
        }
        #[cfg(target_arch = "wasm32")]
        {
            SequentialExecutor::new()
                .map(&indices, |&idx| Ok(claim_fragments[idx].clone()))
                .map_err(|e| e.to_string())
        }
    }

    fn canonical_sort_and_dedup_claims(claims: &mut Vec<EmpiricalClaim>) {
        claims.sort_by_key(Self::claim_key);
        claims.dedup_by_key(|c| Self::claim_key(c));
    }

    fn claim_key(claim: &EmpiricalClaim) -> (String, String, u8, u64, u64, u64, u64) {
        (
            claim.name.clone(),
            claim.slice_label.clone(),
            Self::claim_kind_order(claim.claim_kind),
            claim.sample_size,
            claim.metric_value.to_bits(),
            claim.confidence_interval_95.0.to_bits(),
            claim.confidence_interval_95.1.to_bits(),
        )
    }

    fn claim_kind_order(kind: ClaimKind) -> u8 {
        match kind {
            ClaimKind::Structural => 0,
            ClaimKind::Empirical => 1,
            ClaimKind::Performance => 2,
            ClaimKind::Safety => 3,
        }
    }

    /// Verify self-referential BLAKE3 CID.
    pub fn verify_cid(&self) -> bool {
        let computed = self.compute_cid();
        self.certificate_cid == computed
    }

    /// Validate structural attestation requirements (Gate K).
    pub fn verify_attestation(&self) -> Result<(), CertificateError> {
        if !self.verify_cid() {
            return Err(CertificateError::CidMismatch {
                expected: self.compute_cid(),
                actual: self.certificate_cid.clone(),
            });
        }
        if !self.attestation.deterministic_canonical_mode {
            return Err(CertificateError::AttestationFailed(
                "deterministic canonical mode not verified".to_string(),
            ));
        }
        if !self.attestation.zero_allocation_verified {
            return Err(CertificateError::AttestationFailed(
                "zero allocation check failed".to_string(),
            ));
        }
        if !self.attestation.no_multiply_verified {
            return Err(CertificateError::AttestationFailed(
                "no multiply check failed".to_string(),
            ));
        }
        if !self.attestation.theorem_7_reverse_index_verified {
            return Err(CertificateError::AttestationFailed(
                "Theorem 7 reverse index check failed".to_string(),
            ));
        }
        Ok(())
    }

    /// Serialize certificate to CBOR bytes.
    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| CertificateError::SerializationError(e.to_string()))?;
        Ok(buf)
    }

    /// Deserialize certificate from CBOR bytes.
    pub fn from_cbor_bytes(bytes: &[u8]) -> Result<Self, CertificateError> {
        let cert: Certificate = ciborium::from_reader(bytes)
            .map_err(|e| CertificateError::DeserializationError(e.to_string()))?;
        if !cert.verify_cid() {
            return Err(CertificateError::CidMismatch {
                expected: cert.compute_cid(),
                actual: cert.certificate_cid,
            });
        }
        Ok(cert)
    }
}
