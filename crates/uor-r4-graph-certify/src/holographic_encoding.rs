//! Holographic encoding contract: projection family semantics, ablation protocol,
//! partial reconstruction, and progressive-fidelity probes (PDF §5 / issue #126).

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivergenceMetric {
    KLDivergence,
    JensenShannonDivergence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AblationProtocol {
    pub baseline_projection_ids: Vec<String>,
    pub ablation_order: Vec<String>,
    pub semantics: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionMetadata {
    pub projection_id: String,
    pub depth: u8,
    pub membership_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionRecovery {
    pub projection_id: String,
    pub divergence_to_teacher: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressiveFidelityPoint {
    pub active_projection_count: usize,
    pub mean_membership_count: f64,
    pub divergence_to_teacher: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AblationPoint {
    pub removed_projection_id: String,
    pub divergence_to_teacher: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolographicProbeReport {
    pub partial_recovery: Vec<ProjectionRecovery>,
    pub distributed_evidence_mean_support: f64,
    pub progressive_fidelity: Vec<ProgressiveFidelityPoint>,
    pub ablation_curve: Vec<AblationPoint>,
    pub paraphrase_invariance: f64,
    pub perturbation_stability: f64,
    pub cross_context_reuse: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolographicEncodingCertificate {
    pub version: u32,
    pub certificate_cid: String,
    pub projection_set: Vec<ProjectionMetadata>,
    pub ablation_protocol: AblationProtocol,
    pub divergence_metric: DivergenceMetric,
    pub sample_count: u64,
    pub confidence_interval_95: (f64, f64),
    pub corpus_cids: Vec<String>,
    pub structural_encoding_facts: Vec<String>,
    pub teacher_agreement_claims: Vec<String>,
    pub probe_report: HolographicProbeReport,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub metadata: ProjectionMetadata,
    pub recovered_distribution: Vec<f64>,
}

impl HolographicEncodingCertificate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        projection_set: Vec<ProjectionMetadata>,
        ablation_protocol: AblationProtocol,
        divergence_metric: DivergenceMetric,
        sample_count: u64,
        confidence_interval_95: (f64, f64),
        corpus_cids: Vec<String>,
        structural_encoding_facts: Vec<String>,
        teacher_agreement_claims: Vec<String>,
        probe_report: HolographicProbeReport,
    ) -> Self {
        let mut cert = Self {
            version: 1,
            certificate_cid: String::new(),
            projection_set,
            ablation_protocol,
            divergence_metric,
            sample_count,
            confidence_interval_95,
            corpus_cids,
            structural_encoding_facts,
            teacher_agreement_claims,
            probe_report,
        };
        cert.certificate_cid = cert.compute_cid();
        cert
    }

    pub fn compute_cid(&self) -> String {
        let mut clone = self.clone();
        clone.certificate_cid.clear();
        let mut bytes = Vec::new();
        ciborium::into_writer(&clone, &mut bytes)
            .expect("holographic certificate CBOR serialization must succeed");
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        format!("kappa:blake3:{}", hasher.finalize().to_hex())
    }

    pub fn verify_cid(&self) -> bool {
        self.certificate_cid == self.compute_cid()
    }
}

pub struct HolographicEncodingEvaluator;

impl HolographicEncodingEvaluator {
    /// Validate a projection family for degeneracy. `None` when the family is
    /// non-degenerate; `Some(reason)` naming the first degeneracy found (empty
    /// set, single-node memorization, duplicate id/projection, empty or
    /// invalid recovery distribution, or inconsistent distribution length). A
    /// validator is total; the held property is the absence of a degeneracy
    /// rather than a raised error (R5).
    pub fn validate_projection_family(projections: &[Projection]) -> Option<String> {
        if projections.is_empty() {
            return Some("projection set is empty".to_string());
        }
        if projections.len() == 1 && projections[0].metadata.membership_ids.len() == 1 {
            return Some(format!(
                "single-node memorization in projection '{}'",
                projections[0].metadata.projection_id
            ));
        }

        let mut seen = HashSet::new();
        let mut seen_ids = HashSet::new();
        let mut expected_len = None::<usize>;
        for projection in projections {
            if !seen_ids.insert(projection.metadata.projection_id.clone()) {
                return Some(format!(
                    "duplicate projection id '{}'",
                    projection.metadata.projection_id
                ));
            }
            if projection.metadata.membership_ids.is_empty() {
                return Some(format!(
                    "projection '{}' has no members",
                    projection.metadata.projection_id
                ));
            }
            if !Self::valid_distribution(&projection.recovered_distribution) {
                return Some(format!(
                    "projection '{}' has an invalid recovery distribution",
                    projection.metadata.projection_id
                ));
            }
            match expected_len {
                Some(exp) if exp != projection.recovered_distribution.len() => {
                    return Some(format!(
                        "projection '{}' distribution length {} does not match expected {exp}",
                        projection.metadata.projection_id,
                        projection.recovered_distribution.len()
                    ));
                }
                None => expected_len = Some(projection.recovered_distribution.len()),
                _ => {}
            }
            let key = format!(
                "{:?}|{:?}",
                projection.metadata.membership_ids, projection.recovered_distribution
            );
            if !seen.insert(key) {
                return Some(format!(
                    "duplicate projection '{}'",
                    projection.metadata.projection_id
                ));
            }
        }
        None
    }

    /// Recover the mean behavior distribution across the active projections.
    /// `None` when the request is not a product of these projections: an active
    /// id is unknown or duplicated, or the projections disagree on distribution
    /// length (R5 — the absence of a product rather than a raised error).
    pub fn recover_behavior_distribution(
        projections: &[Projection],
        active_projection_ids: &[String],
    ) -> Option<Vec<f64>> {
        if active_projection_ids.is_empty() {
            return Some(Vec::new());
        }
        let mut acc = Vec::new();
        let mut used = 0usize;
        for projection_id in active_projection_ids {
            let mut matches = projections
                .iter()
                .filter(|p| p.metadata.projection_id == *projection_id);
            let projection = matches.next()?;
            if matches.next().is_some() {
                return None;
            }
            if acc.is_empty() {
                acc = vec![0.0; projection.recovered_distribution.len()];
            }
            if acc.len() != projection.recovered_distribution.len() {
                return None;
            }
            for (idx, value) in projection.recovered_distribution.iter().enumerate() {
                acc[idx] += *value;
            }
            used += 1;
        }
        if used == 0 {
            return Some(Vec::new());
        }
        for value in &mut acc {
            *value /= used as f64;
        }
        Some(Self::normalize(acc))
    }

    pub fn divergence(metric: DivergenceMetric, teacher: &[f64], graph: &[f64]) -> f64 {
        match metric {
            DivergenceMetric::KLDivergence => Self::kl_divergence(teacher, graph),
            DivergenceMetric::JensenShannonDivergence => Self::js_divergence(teacher, graph),
        }
    }

    pub fn partial_recovery(
        projections: &[Projection],
        teacher_distribution: &[f64],
        metric: DivergenceMetric,
    ) -> Vec<ProjectionRecovery> {
        projections
            .iter()
            .map(|projection| ProjectionRecovery {
                projection_id: projection.metadata.projection_id.clone(),
                divergence_to_teacher: Self::divergence(
                    metric,
                    teacher_distribution,
                    &projection.recovered_distribution,
                ),
            })
            .collect()
    }

    pub fn distributed_evidence_mean_support(
        projections: &[Projection],
        support_threshold: f64,
    ) -> f64 {
        if projections.is_empty() {
            return 0.0;
        }
        let Some(first) = projections.first() else {
            return 0.0;
        };
        if first.recovered_distribution.is_empty() {
            return 0.0;
        }

        let mut total_support = 0usize;
        for token_idx in 0..first.recovered_distribution.len() {
            let support = projections
                .iter()
                .filter(|projection| {
                    projection
                        .recovered_distribution
                        .get(token_idx)
                        .is_some_and(|v| *v >= support_threshold)
                })
                .count();
            total_support += support;
        }
        total_support as f64 / first.recovered_distribution.len() as f64
    }

    /// Progressive-fidelity curve as projections are added one at a time.
    /// `None` when a prefix cannot be recovered (see
    /// [`recover_behavior_distribution`]) — R5.
    pub fn progressive_fidelity(
        projections: &[Projection],
        teacher_distribution: &[f64],
        metric: DivergenceMetric,
    ) -> Option<Vec<ProgressiveFidelityPoint>> {
        let mut ids = Vec::new();
        let mut points = Vec::new();
        let mut membership_sum = 0usize;

        for projection in projections {
            ids.push(projection.metadata.projection_id.clone());
            membership_sum += projection.metadata.membership_ids.len();
            let recovered = Self::recover_behavior_distribution(projections, &ids)?;
            points.push(ProgressiveFidelityPoint {
                active_projection_count: ids.len(),
                mean_membership_count: membership_sum as f64 / ids.len() as f64,
                divergence_to_teacher: Self::divergence(metric, teacher_distribution, &recovered),
            });
        }
        Some(points)
    }

    /// Ablation curve as baseline projections are removed in protocol order.
    /// `None` when an ablation id is not in the active set, or a resulting
    /// prefix cannot be recovered (R5 — the absence of a product).
    pub fn ablation_curve(
        projections: &[Projection],
        teacher_distribution: &[f64],
        metric: DivergenceMetric,
        protocol: &AblationProtocol,
    ) -> Option<Vec<AblationPoint>> {
        let mut active: BTreeSet<String> =
            protocol.baseline_projection_ids.iter().cloned().collect();
        let mut curve = Vec::new();
        for removed_id in &protocol.ablation_order {
            if !active.remove(removed_id) {
                return None;
            }
            let active_ids = active.iter().cloned().collect::<Vec<_>>();
            let recovered = Self::recover_behavior_distribution(projections, &active_ids)?;
            curve.push(AblationPoint {
                removed_projection_id: removed_id.clone(),
                divergence_to_teacher: Self::divergence(metric, teacher_distribution, &recovered),
            });
        }
        Some(curve)
    }

    pub fn paraphrase_invariance(
        distribution_x: &[f64],
        distribution_paraphrase: &[f64],
        metric: DivergenceMetric,
    ) -> f64 {
        Self::divergence(metric, distribution_x, distribution_paraphrase)
    }

    pub fn perturbation_stability(
        distribution_x: &[f64],
        distribution_perturbed: &[f64],
        metric: DivergenceMetric,
    ) -> f64 {
        Self::divergence(metric, distribution_x, distribution_perturbed)
    }

    pub fn cross_context_reuse(
        context_a_memberships: &[u32],
        context_b_memberships: &[u32],
    ) -> f64 {
        let a: BTreeSet<u32> = context_a_memberships.iter().copied().collect();
        let b: BTreeSet<u32> = context_b_memberships.iter().copied().collect();
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        let intersection = a.intersection(&b).count();
        let union = a.union(&b).count();
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn valid_distribution(values: &[f64]) -> bool {
        if values.is_empty() {
            return false;
        }
        let sum: f64 = values.iter().sum();
        values.iter().all(|v| v.is_finite() && *v >= 0.0) && sum > 0.0
    }

    fn normalize(mut values: Vec<f64>) -> Vec<f64> {
        let sum: f64 = values.iter().sum();
        if sum <= 0.0 {
            return values;
        }
        for value in &mut values {
            *value /= sum;
        }
        values
    }

    fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
        if p.len() != q.len() || p.is_empty() {
            return f64::NAN;
        }
        let eps = 1e-12;
        p.iter()
            .zip(q.iter())
            .map(|(&pi, &qi)| {
                if pi <= 0.0 {
                    0.0
                } else {
                    let qi_c = qi.max(eps);
                    pi * (pi / qi_c).log2()
                }
            })
            .sum::<f64>()
    }

    fn js_divergence(p: &[f64], q: &[f64]) -> f64 {
        if p.len() != q.len() || p.is_empty() {
            return f64::NAN;
        }
        let m: Vec<f64> = p
            .iter()
            .zip(q.iter())
            .map(|(&pi, &qi)| (pi + qi) / 2.0)
            .collect();
        0.5 * Self::kl_divergence(p, &m) + 0.5 * Self::kl_divergence(q, &m)
    }
}
