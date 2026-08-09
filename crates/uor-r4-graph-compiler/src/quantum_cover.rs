//! Von Neumann maximum-entropy quantum density operator and quantum
//! cover induction (issue #108; spec:
//! `docs/r4_furey_quantum_geometric_plan.md` §Phase 5).
//!
//! Compiler-side only: f32/f64 math is confined to cover evaluation and
//! residual quantization; nothing here runs in the deployed integer
//! kernel. The density operators used for cover induction are diagonal
//! (eigenvalues = normalized next-token distributions), where the von
//! Neumann entropy S(ρ) = −Tr(ρ ln ρ) coincides with the Shannon
//! entropy of the distribution in nats — the value of the module is
//! the operator framing, the maximum-entropy equipartition bound, and
//! the nats-based gain criterion, not a different number.

use crate::induction::{DEFAULT_SPLIT_ENTROPY_GAIN_BITS, Observation};
use std::collections::BTreeMap;
use uor_r4_graph_format::ScoreQ;

/// Diagonal density operator ρ: eigenvalues are a normalized
/// distribution (trace 1). The maximum-entropy operator (1/n)I is the
/// uniform special case.
#[derive(Debug, Clone, PartialEq)]
pub struct DensityOperator {
    eigenvalues: Vec<f32>,
}

impl DensityOperator {
    /// The maximum-entropy operator ρ = (1/n)I_{n×n} in `dimension`
    /// dimensions. `None` when `dimension` is zero: an empty distribution is
    /// not a density operator (R5 — the absence of a product, not an error).
    pub fn max_entropy(dimension: usize) -> Option<Self> {
        if dimension == 0 {
            return None;
        }
        let p = 1.0 / dimension as f32;
        Some(Self {
            eigenvalues: vec![p; dimension],
        })
    }

    /// ρ from finite non-negative weights, normalized to trace 1. `None` when
    /// the weights are empty or do not sum to a finite positive value (no
    /// density operator exists for them).
    pub fn from_weights(weights: &[f32]) -> Option<Self> {
        if weights.is_empty() {
            return None;
        }
        let sum: f32 = weights.iter().sum();
        if !sum.is_finite() || sum <= 0.0 {
            return None;
        }
        Some(Self {
            eigenvalues: weights.iter().map(|&w| w / sum).collect(),
        })
    }

    /// ρ from integer occurrence counts, normalized to trace 1. `None` when the
    /// counts are empty or sum to zero.
    pub fn from_counts(counts: &[u64]) -> Option<Self> {
        if counts.is_empty() {
            return None;
        }
        let sum: u64 = counts.iter().sum();
        if sum == 0 {
            return None;
        }
        Some(Self {
            eigenvalues: counts.iter().map(|&c| c as f32 / sum as f32).collect(),
        })
    }

    /// Matrix dimension n.
    pub fn dimension(&self) -> usize {
        self.eigenvalues.len()
    }

    /// Tr(ρ): 1 up to float rounding.
    pub fn trace(&self) -> f32 {
        self.eigenvalues.iter().sum()
    }

    /// Von Neumann entropy S(ρ) = −Tr(ρ ln ρ) in nats. Zero-eigenvalue
    /// terms contribute 0 (the p ln p → 0 limit).
    pub fn von_neumann_entropy(&self) -> f32 {
        self.eigenvalues
            .iter()
            .map(|&p| if p > 1e-9 { -p * p.ln() } else { 0.0 })
            .sum()
    }

    /// Maximum entropy bound S_max = ln n for dimension n (attained by
    /// the maximum-entropy operator).
    pub fn max_entropy_bound(dimension: usize) -> f32 {
        (dimension as f32).ln()
    }

    /// Equipartition ratio S(ρ) / S_max ∈ [0, 1]: 1 is the
    /// maximum-entropy (fully noisy) operator, 0 a pure state.
    /// Dimension ≤ 1 carries no entropy and reports 0.
    pub fn equipartition(&self) -> f32 {
        let bound = Self::max_entropy_bound(self.dimension());
        if bound <= 0.0 {
            0.0
        } else {
            (self.von_neumann_entropy() / bound).clamp(0.0, 1.0)
        }
    }
}

/// Next-token counts over a member list (observation order preserved,
/// tokens ascending).
fn next_token_counts(observations: &[Observation], members: &[usize]) -> BTreeMap<u32, u64> {
    let mut counts = BTreeMap::new();
    for &index in members {
        *counts.entry(observations[index].next).or_insert(0) += 1;
    }
    counts
}

/// Von Neumann entropy of the next-token density operator over a member
/// list, in nats.
fn member_entropy_nats(observations: &[Observation], members: &[usize]) -> f64 {
    let counts = next_token_counts(observations, members);
    let eigenvalues: Vec<u64> = counts.values().copied().collect();
    match DensityOperator::from_counts(&eigenvalues) {
        Some(rho) => f64::from(rho.von_neumann_entropy()),
        None => 0.0,
    }
}

/// Quantum entropy gain of a candidate region partition, in nats:
/// S(ρ_parent) − Σ_c (|c|/|parent|)·S(ρ_c), with ρ the next-token
/// density operator. The nats analogue of
/// `cover::entropy_reduction` (which reports bits); used as the
/// entropy-gain criterion for quantum cover induction.
pub fn quantum_entropy_gain(
    observations: &[Observation],
    members: &[usize],
    children: &[Vec<usize>],
) -> f64 {
    let parent = member_entropy_nats(observations, members);
    let total = members.len() as f64;
    if total == 0.0 {
        return 0.0;
    }
    let mut expected_child = 0.0f64;
    for child in children {
        if child.is_empty() {
            continue;
        }
        let weight = child.len() as f64 / total;
        expected_child += weight * member_entropy_nats(observations, child);
    }
    parent - expected_child
}

/// Entropy-gain criterion for quantum cover induction: a candidate
/// partition is accepted when its quantum entropy gain clears
/// `min_entropy_gain_nats`. The default converts the compiler's
/// classical floor (bits) to nats so both criteria accept at the same
/// information threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumCoverConfig {
    /// Minimum quantum entropy gain for accepting a split, in nats.
    pub min_entropy_gain_nats: f64,
}

impl Default for QuantumCoverConfig {
    fn default() -> Self {
        Self {
            min_entropy_gain_nats: DEFAULT_SPLIT_ENTROPY_GAIN_BITS * std::f64::consts::LN_2,
        }
    }
}

impl QuantumCoverConfig {
    /// Accept the partition when its quantum entropy gain clears the
    /// configured floor.
    pub fn accept_partition(&self, gain_nats: f64) -> bool {
        gain_nats >= self.min_entropy_gain_nats
    }
}

/// Quantize emission log-residuals into a `ScoreQ` (Q16.16) integer
/// table for the deployed runtime. Delegates to
/// `ScoreQ::from_logprob` (saturating; NaN maps to zero) — this is a
/// compiler-side convenience; the runtime never converts floats.
pub fn quantize_residuals(residuals: &[f32]) -> Vec<ScoreQ> {
    residuals.iter().map(|&r| ScoreQ::from_logprob(r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::transformerless::compiler::SIG_BYTES;

    #[test]
    fn max_entropy_operator_matches_analytical_entropy() {
        let n = 8usize;
        let rho = DensityOperator::max_entropy(n).expect("dimension non-zero");
        assert!((rho.trace() - 1.0).abs() < 1e-6, "trace is one");
        let s = rho.von_neumann_entropy();
        let expected = (n as f32).ln();
        assert!(
            (s - expected).abs() < 1e-6,
            "S((1/n)I) = ln n: got {s}, want {expected}"
        );
        assert!((rho.equipartition() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pure_state_has_zero_entropy() {
        let rho = DensityOperator::from_weights(&[1.0, 0.0, 0.0]).expect("valid weights");
        assert_eq!(rho.von_neumann_entropy(), 0.0);
        assert_eq!(rho.equipartition(), 0.0);
    }

    #[test]
    fn two_point_distribution_matches_hand_computed() {
        // p = (0.25, 0.75): S = -(0.25 ln 0.25 + 0.75 ln 0.75) nats.
        let rho = DensityOperator::from_weights(&[0.25, 0.75]).expect("valid weights");
        let expected = -(0.25f32 * 0.25f32.ln() + 0.75f32 * 0.75f32.ln());
        assert!((rho.von_neumann_entropy() - expected).abs() < 1e-6);
    }

    #[test]
    fn invalid_distributions_are_none_not_panics() {
        assert!(DensityOperator::max_entropy(0).is_none());
        assert!(DensityOperator::from_weights(&[]).is_none());
        assert!(DensityOperator::from_weights(&[0.0, 0.0]).is_none());
        assert!(DensityOperator::from_counts(&[0, 0]).is_none());
    }

    #[test]
    fn gain_is_positive_for_informative_split_and_zero_for_noise() {
        let observations: Vec<Observation> = (0..100u32)
            .map(|i| Observation {
                position: i,
                sample: [0u8; 32],
                vector: Vec::new(),
                sig: [0u8; SIG_BYTES],
                prev: 0,
                next: if i < 50 { 1 } else { 2 },
            })
            .collect();
        let members: Vec<usize> = (0..100).collect();
        let informative = vec![(0..50).collect::<Vec<_>>(), (50..100).collect::<Vec<_>>()];
        let gain = quantum_entropy_gain(&observations, &members, &informative);
        // Parent S = ln 2 nats, each child is pure: gain = ln 2.
        assert!((gain - std::f64::consts::LN_2).abs() < 1e-4, "gain {gain}");
        assert!(QuantumCoverConfig::default().accept_partition(gain));
        // A split that mixes the halves leaves the entropy untouched.
        let noisy = vec![
            (0..100).step_by(2).collect::<Vec<_>>(),
            (1..100).step_by(2).collect::<Vec<_>>(),
        ];
        let noise_gain = quantum_entropy_gain(&observations, &members, &noisy);
        assert!(noise_gain.abs() < 1e-4, "noise gain {noise_gain}");
        assert!(!QuantumCoverConfig::default().accept_partition(noise_gain));
    }

    #[test]
    fn quantization_matches_scoreq_from_logprob() {
        let residuals = [-2.5, -0.0, 1.25, f32::NAN, 1e9, -1e9];
        let table = quantize_residuals(&residuals);
        assert_eq!(table.len(), residuals.len());
        for (&q, &r) in table.iter().zip(residuals.iter()) {
            assert_eq!(q, ScoreQ::from_logprob(r));
        }
        // Round-trip within Q16.16 resolution for in-range values.
        let back = table[2].to_logprob();
        assert!((back - 1.25).abs() < 1e-4);
    }
}
