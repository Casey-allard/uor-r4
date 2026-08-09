//! Source-bias amplification, rare-group erasure, and provenance-deletion harness
//! (Phase 3 / PDF §14 / Plan §9.21).

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::transitions::TransitionGraph;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceBiasMetric {
    pub slice_label: String,
    pub teacher_bias_ratio: f64,
    pub graph_bias_ratio: f64,
    pub amplification_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RareGroupRetentionMetric {
    pub group_label: String,
    pub original_sample_count: usize,
    pub retained_region_count: usize,
    pub retention_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceDeletionWitness {
    pub deleted_slice_id: String,
    pub tombstoned_provenance_nodes: usize,
    pub graph_integrity_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FairnessAndProvenanceCertificate {
    pub version: u32,
    pub certificate_cid: String,
    pub bias_metrics: Vec<SliceBiasMetric>,
    pub rare_group_retention: Vec<RareGroupRetentionMetric>,
    pub provenance_deletion_witness: ProvenanceDeletionWitness,
    pub bias_amplification_passed: bool,
    pub rare_group_erasure_passed: bool,
}

impl FairnessAndProvenanceCertificate {
    pub fn new(
        bias_metrics: Vec<SliceBiasMetric>,
        rare_group_retention: Vec<RareGroupRetentionMetric>,
        provenance_deletion_witness: ProvenanceDeletionWitness,
    ) -> Self {
        let max_amp = bias_metrics
            .iter()
            .map(|m| m.amplification_factor)
            .fold(1.0f64, f64::max);
        let bias_amplification_passed = max_amp <= 1.05;

        let min_retention = rare_group_retention
            .iter()
            .map(|r| r.retention_rate)
            .fold(1.0f64, f64::min);
        let rare_group_erasure_passed = min_retention >= 0.95;

        let mut cert = FairnessAndProvenanceCertificate {
            version: 1,
            certificate_cid: String::new(),
            bias_metrics,
            rare_group_retention,
            provenance_deletion_witness,
            bias_amplification_passed,
            rare_group_erasure_passed,
        };
        cert.certificate_cid = cert.compute_cid();
        cert
    }

    /// Compute self-referential BLAKE3 CID over certificate payload.
    pub fn compute_cid(&self) -> String {
        let mut clone = self.clone();
        clone.certificate_cid.clear();

        let mut bytes = Vec::new();
        ciborium::into_writer(&clone, &mut bytes)
            .expect("fairness/provenance certificate CBOR serialization must succeed");

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        format!("kappa:blake3:{}", hasher.finalize().to_hex())
    }

    pub fn verify_cid(&self) -> bool {
        self.certificate_cid == self.compute_cid()
    }

    /// Serialize to CBOR. Infallible: ciborium serialization of this
    /// derive-Serialize certificate into an in-memory buffer cannot fail — a
    /// failure would be a serialization defect, not a property of the data
    /// (R5 — self-produced bytes are an invariant, not a reported condition).
    pub fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .expect("FairnessAndProvenanceCertificate CBOR serialization is infallible");
        buf
    }

    /// Parse from CBOR and check the certificate's self-CID. `None` when the
    /// bytes are not a valid CBOR encoding, or when the recomputed CID does not
    /// match the embedded one: in either case the bytes are not a valid,
    /// self-consistent certificate — the absence of a product (R5).
    pub fn from_cbor_bytes(bytes: &[u8]) -> Option<Self> {
        let cert: FairnessAndProvenanceCertificate = ciborium::from_reader(bytes).ok()?;
        if !cert.verify_cid() {
            return None;
        }
        Some(cert)
    }
}

pub struct FairnessEvaluator;

impl FairnessEvaluator {
    pub fn evaluate_bias_and_erasure(
        teacher_slice_counts: &[(String, usize)],
        graph_slice_counts: &[(String, usize)],
    ) -> (Vec<SliceBiasMetric>, Vec<RareGroupRetentionMetric>) {
        let teacher_total: usize = teacher_slice_counts.iter().map(|(_, c)| c).sum();
        let graph_total: usize = graph_slice_counts.iter().map(|(_, c)| c).sum();

        let mut bias_metrics = Vec::new();
        let mut rare_group_retention = Vec::new();

        for (label, t_count) in teacher_slice_counts {
            let g_count = graph_slice_counts
                .iter()
                .find(|(l, _)| l == label)
                .map(|(_, c)| *c)
                .unwrap_or(0);

            let t_ratio = if teacher_total > 0 {
                *t_count as f64 / teacher_total as f64
            } else {
                0.0
            };
            let g_ratio = if graph_total > 0 {
                g_count as f64 / graph_total as f64
            } else {
                0.0
            };

            let amp = if t_ratio > 0.0 {
                g_ratio / t_ratio
            } else {
                1.0
            };

            bias_metrics.push(SliceBiasMetric {
                slice_label: label.clone(),
                teacher_bias_ratio: t_ratio,
                graph_bias_ratio: g_ratio,
                amplification_factor: amp,
            });

            if *t_count < 50 {
                let retention_rate = if *t_count > 0 {
                    g_count as f64 / *t_count as f64
                } else {
                    1.0
                };
                rare_group_retention.push(RareGroupRetentionMetric {
                    group_label: label.clone(),
                    original_sample_count: *t_count,
                    retained_region_count: g_count,
                    retention_rate,
                });
            }
        }

        (bias_metrics, rare_group_retention)
    }

    /// Verify that every provenance edge attributed to `deleted_slice_id` has
    /// been tombstoned in `graph` (weight forced to 0). `Some(witness)` when the
    /// deletion holds; `None` when it does not — a tombstoned edge id is out of
    /// bounds, or a referenced edge still carries a non-zero weight. The
    /// deletion is verified or it is not; an unverified deletion yields no
    /// witness rather than a raised error (R5).
    pub fn verify_provenance_deletion(
        deleted_slice_id: &str,
        graph: &TransitionGraph,
        tombstoned_edge_ids: &[usize],
    ) -> Option<ProvenanceDeletionWitness> {
        for &edge_id in tombstoned_edge_ids {
            let edge = graph.edges.get(edge_id)?;
            if edge.weight != 0 {
                return None;
            }
        }

        Some(ProvenanceDeletionWitness {
            deleted_slice_id: deleted_slice_id.to_string(),
            tombstoned_provenance_nodes: tombstoned_edge_ids.len(),
            graph_integrity_verified: true,
        })
    }
}
