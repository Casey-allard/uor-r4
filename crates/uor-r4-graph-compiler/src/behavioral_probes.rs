//! Unsupervised Intervention and Counterfactual Behavioral Probes
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§7, 11, 17;
//! `docs/formal_vocabulary.md` §3; GitHub Issue #128.
//!
//! This module provides the behavioral-probing layer needed to distinguish reusable
//! predictive structure from surface association:
//! - Content-addressed `InterventionRecord` defining baseline vs perturbed observations.
//! - Supported intervention kinds: `ContextAblation`, `SurfaceVariation`, `EntitySubstitution`,
//!   `TemporalChange`, and `GoalChange`.
//! - Declarative expectations: `Invariant` (nuisance variations must preserve output) vs
//!   `Sensitive` (causal interventions must alter output).
//! - Probe harness evaluating sensitivity and invariance scores with anti-memorization guards.

use serde::{Deserialize, Serialize};

/// Controlled intervention kind applied to observation context $x$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterventionKind {
    /// Ablating specific context text spans.
    ContextAblation,
    /// Paraphrase or surface-preserving variation (nuisance parameter).
    SurfaceVariation,
    /// Value or entity substitution.
    EntitySubstitution,
    /// Temporal sequence shift.
    TemporalChange,
    /// Action or goal specification change.
    GoalChange,
}

/// Declared expectation relation under intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpectedRelation {
    /// Output MUST remain invariant ($\Delta \le \epsilon$).
    Invariant,
    /// Output MUST be sensitive / change ($\Delta \ge \delta$).
    Sensitive,
    /// Relationship is unknown / unconstrained.
    Unknown,
}

/// A content-addressed intervention record describing a counterfactual probe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRecord {
    /// Content-addressed ID derived from observation and intervention payload.
    pub id: String,
    /// Baseline source observation text ($x$).
    pub source_observation: String,
    /// Type of controlled intervention applied.
    pub kind: InterventionKind,
    /// Character/byte span affected by intervention $(start, end)$.
    pub affected_span: (usize, usize),
    /// Declared expected behavior relation.
    pub expected_relation: ExpectedRelation,
    /// Baseline teacher output probabilities $P_\theta(\cdot | x)$.
    pub baseline_output: Vec<f32>,
    /// Counterfactual teacher output probabilities $P_\theta(\cdot | x_{\text{intervened}})$.
    pub intervention_output: Vec<f32>,
}

impl InterventionRecord {
    /// Create and validate a new intervention record. `None` when the record is
    /// not a product of these inputs: the affected span is out of bounds for the
    /// observation, or the baseline/intervention outputs are empty or of
    /// mismatched dimension (R5 — the absence of a product rather than a raised
    /// error).
    pub fn new(
        source_observation: impl Into<String>,
        kind: InterventionKind,
        affected_span: (usize, usize),
        expected_relation: ExpectedRelation,
        baseline_output: Vec<f32>,
        intervention_output: Vec<f32>,
    ) -> Option<Self> {
        let obs = source_observation.into();
        let (start, end) = affected_span;
        if start > end || end > obs.len() {
            return None;
        }
        if baseline_output.len() != intervention_output.len() || baseline_output.is_empty() {
            return None;
        }

        // Content-addressed ID: blake3 over the observation, intervention
        // kind/span, expected relation, and both output vectors, so any
        // difference in inputs or outputs yields a distinct id.
        let mut hasher = blake3::Hasher::new();
        hasher.update(obs.as_bytes());
        hasher.update(&[kind as u8]);
        hasher.update(&(start as u64).to_le_bytes());
        hasher.update(&(end as u64).to_le_bytes());
        hasher.update(&[expected_relation as u8]);
        for value in &baseline_output {
            hasher.update(&value.to_le_bytes());
        }
        for value in &intervention_output {
            hasher.update(&value.to_le_bytes());
        }
        let id = format!("probe_blake3:{}", hasher.finalize().to_hex());

        Some(Self {
            id,
            source_observation: obs,
            kind,
            affected_span,
            expected_relation,
            baseline_output,
            intervention_output,
        })
    }

    /// Compute L1 divergence between baseline and intervention outputs.
    pub fn output_divergence(&self) -> f32 {
        self.baseline_output
            .iter()
            .zip(self.intervention_output.iter())
            .map(|(b, i)| (b - i).abs())
            .sum::<f32>()
    }
}

/// Evaluation result metrics for a probe suite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehavioralProbeReport {
    pub total_probes: usize,
    pub invariant_passed: usize,
    pub sensitive_passed: usize,
    pub invariance_score: f32,
    pub sensitivity_score: f32,
    pub memorization_check_passed: bool,
}

/// Harness executing and auditing counterfactual behavioral probes.
pub struct BehavioralProbeHarness;

impl BehavioralProbeHarness {
    /// Evaluate a set of intervention records against expectation relations.
    /// Total: the anti-memorization guard's verdict is carried in the report's
    /// `memorization_check_passed` field rather than raised as an error (R5 — a
    /// validator reports the held property; a tripped guard is `false`, not a
    /// failure to produce a report).
    pub fn evaluate_suite(
        probes: &[InterventionRecord],
        invariance_tolerance: f32,
        sensitivity_threshold: f32,
    ) -> BehavioralProbeReport {
        if probes.is_empty() {
            return BehavioralProbeReport {
                total_probes: 0,
                invariant_passed: 0,
                sensitive_passed: 0,
                invariance_score: 1.0,
                sensitivity_score: 1.0,
                memorization_check_passed: true,
            };
        }

        let mut inv_count = 0;
        let mut inv_passed = 0;
        let mut sens_count = 0;
        let mut sens_passed = 0;

        for probe in probes {
            let div = probe.output_divergence();
            match probe.expected_relation {
                ExpectedRelation::Invariant => {
                    inv_count += 1;
                    if div <= invariance_tolerance {
                        inv_passed += 1;
                    }
                }
                ExpectedRelation::Sensitive => {
                    sens_count += 1;
                    if div >= sensitivity_threshold {
                        sens_passed += 1;
                    }
                }
                ExpectedRelation::Unknown => {}
            }
        }

        let invariance_score = if inv_count > 0 {
            inv_passed as f32 / inv_count as f32
        } else {
            1.0
        };

        let sensitivity_score = if sens_count > 0 {
            sens_passed as f32 / sens_count as f32
        } else {
            1.0
        };

        // Anti-memorization guard check: any Sensitive GoalChange/ContextAblation probe
        // whose divergence falls below the sensitivity threshold indicates the model is
        // memorizing surface form without understanding state dynamics.
        let memorization_check_passed = !probes.iter().any(|probe| {
            probe.expected_relation == ExpectedRelation::Sensitive
                && matches!(
                    probe.kind,
                    InterventionKind::GoalChange | InterventionKind::ContextAblation
                )
                && probe.output_divergence() < sensitivity_threshold
        });

        BehavioralProbeReport {
            total_probes: probes.len(),
            invariant_passed: inv_passed,
            sensitive_passed: sens_passed,
            invariance_score,
            sensitivity_score,
            memorization_check_passed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_record_creation_and_divergence() {
        let rec = InterventionRecord::new(
            "The temperature is 20C.",
            InterventionKind::SurfaceVariation,
            (0, 15),
            ExpectedRelation::Invariant,
            vec![0.8, 0.2],
            vec![0.81, 0.19],
        )
        .unwrap();

        assert!(rec.id.starts_with("probe_"));
        assert!((rec.output_divergence() - 0.02).abs() < 1e-4);
    }

    #[test]
    fn test_probe_harness_evaluation() {
        let p_inv = InterventionRecord::new(
            "Context text sample",
            InterventionKind::SurfaceVariation,
            (0, 7),
            ExpectedRelation::Invariant,
            vec![0.9, 0.1],
            vec![0.905, 0.095], // div = 0.01
        )
        .unwrap();

        let p_sens = InterventionRecord::new(
            "Context text sample",
            InterventionKind::GoalChange,
            (0, 7),
            ExpectedRelation::Sensitive,
            vec![0.9, 0.1],
            vec![0.1, 0.9], // div = 1.6
        )
        .unwrap();

        let report = BehavioralProbeHarness::evaluate_suite(&[p_inv, p_sens], 0.05, 0.5);

        assert_eq!(report.total_probes, 2);
        assert_eq!(report.invariance_score, 1.0);
        assert_eq!(report.sensitivity_score, 1.0);
        assert!(report.memorization_check_passed);
    }

    #[test]
    fn test_anti_memorization_guard_rejection() {
        // Sensitive goal change resulted in 0 divergence -> memorization failure!
        let p_mem = InterventionRecord::new(
            "Context text sample",
            InterventionKind::GoalChange,
            (0, 7),
            ExpectedRelation::Sensitive,
            vec![0.9, 0.1],
            vec![0.9, 0.1], // div = 0.0
        )
        .unwrap();

        let report = BehavioralProbeHarness::evaluate_suite(&[p_mem], 0.05, 0.5);
        assert!(!report.memorization_check_passed);
    }
}
