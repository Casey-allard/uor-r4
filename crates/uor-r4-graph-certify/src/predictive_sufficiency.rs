//! Predictive-sufficiency divergence measurement and rate-distortion reporting across
//! graph depths (Phase 3 / Plan §9.14).

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::runtime::OpKernel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphDepth {
    BroadCloud,
    IntermediateCloud,
    FullCloud,
    ResidualAugmented,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivergencePoint {
    pub depth: GraphDepth,
    pub kl_divergence: f64,
    pub cross_entropy: f64,
    pub top1_accuracy: f64,
    /// Whether the teacher's most likely class is among the graph's five most likely classes.
    pub top5_recall: f64,
    pub bytes_footprint: usize,
    pub op_budget: OpKernel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RateDistortionReport {
    pub points: Vec<DivergencePoint>,
}

impl RateDistortionReport {
    pub fn new(points: Vec<DivergencePoint>) -> Self {
        RateDistortionReport { points }
    }

    /// Returns list of (Rate in bytes+ops, Distortion in KL-divergence bits).
    pub fn compute_rate_distortion_curve(&self) -> Vec<(f64, f64)> {
        const OPS_TO_BYTES_WEIGHT: f64 = 0.1;

        self.points
            .iter()
            .map(|p| {
                let total_ops = (p.op_budget.adds
                    + p.op_budget.xors
                    + p.op_budget.shifts
                    + p.op_budget.compares
                    + p.op_budget.table_reads
                    + p.op_budget.candidate_scans) as f64;
                let rate = (p.bytes_footprint as f64) + OPS_TO_BYTES_WEIGHT * total_ops;
                let distortion = p.kl_divergence;
                (rate, distortion)
            })
            .collect()
    }

    /// Serialize to CBOR. Infallible: the report's fields are plain scalars,
    /// so ciborium serialization into an in-memory buffer cannot fail — a
    /// failure would be a serialization defect, not a property of the data
    /// (R5 — self-produced bytes are an invariant, not a reported condition).
    pub fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .expect("RateDistortionReport CBOR serialization is infallible for its scalar fields");
        buf
    }

    /// Parse from CBOR. `None` when the bytes are not a valid CBOR encoding of
    /// this report (R5 — the absence of a product rather than a raised error).
    pub fn from_cbor_bytes(bytes: &[u8]) -> Option<Self> {
        ciborium::from_reader(bytes).ok()
    }
}

pub struct PredictiveSufficiencyEvaluator;

impl PredictiveSufficiencyEvaluator {
    /// Compute KL-divergence D_KL(P || Q) = sum(P(i) * log2(P(i) / Q(i)))
    pub fn compute_kl_divergence(p: &[f64], q: &[f64]) -> f64 {
        if p.len() != q.len() || p.is_empty() {
            return f64::NAN;
        }
        let eps = 1e-12;
        p.iter()
            .zip(q.iter())
            .map(|(&pi, &qi)| {
                if pi <= 0.0 {
                    return 0.0;
                }
                let qi_c = qi.max(eps);
                pi * (pi / qi_c).log2()
            })
            .sum::<f64>()
    }

    /// Compute Cross-Entropy H(P, Q) = -sum(P(i) * log2(Q(i)))
    pub fn compute_cross_entropy(p: &[f64], q: &[f64]) -> f64 {
        if p.len() != q.len() || p.is_empty() {
            return f64::NAN;
        }
        let eps = 1e-12;
        p.iter()
            .zip(q.iter())
            .map(|(&pi, &qi)| {
                if pi <= 0.0 {
                    return 0.0;
                }
                let qi_c = qi.max(eps);
                -pi * qi_c.log2()
            })
            .sum::<f64>()
    }

    pub fn evaluate_depth(
        teacher_probs: &[f64],
        graph_probs: &[f64],
        depth: GraphDepth,
        bytes: usize,
        ops: OpKernel,
    ) -> DivergencePoint {
        let kl = Self::compute_kl_divergence(teacher_probs, graph_probs);
        let ce = Self::compute_cross_entropy(teacher_probs, graph_probs);

        let top1_acc = if !teacher_probs.is_empty() && !graph_probs.is_empty() {
            let argmax = |xs: &[f64]| -> usize {
                xs.iter()
                    .enumerate()
                    .filter(|(_, v)| v.is_finite())
                    .max_by(|a, b| a.1.total_cmp(b.1))
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            };

            let max_t = argmax(teacher_probs);
            let max_g = argmax(graph_probs);
            if max_t == max_g {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        let top5_recall = if !teacher_probs.is_empty() && !graph_probs.is_empty() {
            let teacher_top = teacher_probs
                .iter()
                .enumerate()
                .filter(|(_, v)| v.is_finite())
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i);
            let mut graph_top5 = graph_probs
                .iter()
                .enumerate()
                .filter(|(_, v)| v.is_finite())
                .collect::<Vec<_>>();
            graph_top5.sort_by(|a, b| b.1.total_cmp(a.1));

            teacher_top
                .map(|teacher_idx| {
                    if graph_top5
                        .iter()
                        .take(5)
                        .any(|(index, _)| *index == teacher_idx)
                    {
                        1.0
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };

        DivergencePoint {
            depth,
            kl_divergence: kl,
            cross_entropy: ce,
            top1_accuracy: top1_acc,
            top5_recall,
            bytes_footprint: bytes,
            op_budget: ops,
        }
    }
}
