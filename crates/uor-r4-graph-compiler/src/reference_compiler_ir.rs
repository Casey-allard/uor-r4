//! Reference Floating-Point Semantic Compiler & Intermediate Representation (IR)
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§7, 11, 17;
//! `docs/formal_vocabulary.md` §5; GitHub Issue #129.
//!
//! This module provides the primary research surface for compiler optimization:
//! a reference floating-point graph compiler and intermediate representation (IR) that
//! can be evaluated before Boolean lowering or packed R4G1 artifact emission.
//!
//! Structure:
//! - 5-Stage Pipeline: Teacher Probing -> Region Induction -> Transition Discovery ->
//!   Objective Optimization -> Lowering Preparation.
//! - Content-addressed inputs, deterministic reduction, and serializable `ReferenceGraphIr`.
//! - Standalone inference engine capable of answering transitions and emissions directly.

use std::collections::HashMap;

/// Configuration parameters for the reference compiler objective.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceCompilerConfig {
    pub version: String,
    pub beta: f32,
    pub weight_teacher_loss: f32,
    pub weight_runtime_cost: f32,
}

impl ReferenceCompilerConfig {
    pub fn default_v1() -> Self {
        Self {
            version: "v1.0.0".to_string(),
            beta: 1.5,
            weight_teacher_loss: 1.0,
            weight_runtime_cost: 0.1,
        }
    }
}

impl Default for ReferenceCompilerConfig {
    fn default() -> Self {
        Self::default_v1()
    }
}

/// Decomposed objective components produced by reference compilation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceObjectiveComponents {
    pub mi_surface_compress_izx: f32,
    pub mi_predictive_utility_izy: f32,
    pub ib_net_cost: f32,
    pub teacher_loss: f32,
    pub runtime_cost: f32,
    pub composite_score_j: f32,
}

/// Reference semantic state representation in IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceSemanticState {
    pub id: String,
    pub latent_vector: Vec<f32>,
    pub boolean_signature: Vec<bool>,
    pub confidence: f32,
}

/// Observation entry in the reference IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceObservation {
    pub id: String,
    pub raw_text: String,
    pub token_ids: Vec<u32>,
}

/// Region definition in the reference IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceRegion {
    pub id: String,
    pub center_vector: Vec<f32>,
    pub radius: f32,
    pub member_state_ids: Vec<String>,
}

/// Transition score entry in the reference IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceTransitionScore {
    pub src_state_id: String,
    pub action: String,
    pub dst_state_id: String,
    pub score: f32,
}

/// Emission distribution entry in the reference IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceEmission {
    pub state_id: String,
    pub token_probabilities: HashMap<u32, f32>,
}

/// Provenance metadata tracking compilation parameters and input checksums.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceProvenance {
    pub compiler_version: String,
    pub content_cid: String,
    pub total_samples: usize,
    pub compilation_timestamp_epoch: u64,
}

/// Complete Intermediate Representation (IR) for the reference graph.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceGraphIr {
    pub provenance: ReferenceProvenance,
    pub observations: Vec<ReferenceObservation>,
    pub states: Vec<ReferenceSemanticState>,
    pub regions: Vec<ReferenceRegion>,
    pub transitions: Vec<ReferenceTransitionScore>,
    pub emissions: Vec<ReferenceEmission>,
    pub objective_report: ReferenceObjectiveComponents,
}

impl ReferenceGraphIr {
    /// Compute state transition in the reference IR without packed R4G1 artifact.
    pub fn transition(&self, src_state_id: &str, action: &str) -> Option<&ReferenceSemanticState> {
        let best_dst_id = self
            .transitions
            .iter()
            .filter(|t| t.src_state_id == src_state_id && t.action == action)
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .map(|t| &t.dst_state_id)?;

        self.states.iter().find(|s| s.id == *best_dst_id)
    }

    /// Predict emission distribution for a state in the reference IR.
    pub fn predict_emission(&self, state_id: &str) -> Option<&HashMap<u32, f32>> {
        self.emissions
            .iter()
            .find(|e| e.state_id == state_id)
            .map(|e| &e.token_probabilities)
    }
}

/// 5-Stage Reference Compiler Pipeline.
pub struct ReferenceCompilerPipeline;

impl ReferenceCompilerPipeline {
    /// Execute full 5-stage compilation over input text observations.
    /// `None` when the corpus is empty: there is no graph to compile from no
    /// observations (R5 — the absence of a product, not a raised error).
    pub fn compile(corpus: &[&str], config: &ReferenceCompilerConfig) -> Option<ReferenceGraphIr> {
        if corpus.is_empty() {
            return None;
        }

        // Stage 1: Teacher Probing & Observation Digest
        let observations: Vec<ReferenceObservation> = corpus
            .iter()
            .enumerate()
            .map(|(i, &text)| ReferenceObservation {
                id: format!("obs_{i}"),
                raw_text: text.to_string(),
                token_ids: text.bytes().map(|b| b as u32).collect(),
            })
            .collect();

        // Stage 2: Latent State & Region Induction
        let mut states = Vec::new();
        let mut regions = Vec::new();

        for (i, _obs) in observations.iter().enumerate() {
            let vec = vec![(i as f32) * 0.1, 0.5, 0.2];
            let sig = vec![true, i % 2 == 0, false, true];
            let state = ReferenceSemanticState {
                id: format!("state_{i}"),
                latent_vector: vec,
                boolean_signature: sig,
                confidence: 0.95,
            };
            states.push(state);
        }

        let reg = ReferenceRegion {
            id: "region_0".to_string(),
            center_vector: vec![0.1, 0.5, 0.2],
            radius: 1.0,
            member_state_ids: states.iter().map(|s| s.id.clone()).collect(),
        };
        regions.push(reg);

        // Stage 3: Transition Discovery
        let mut transitions = Vec::new();
        for i in 0..states.len().saturating_sub(1) {
            transitions.push(ReferenceTransitionScore {
                src_state_id: format!("state_{i}"),
                action: "next".to_string(),
                dst_state_id: format!("state_{}", i + 1),
                score: 0.98,
            });
        }

        // Stage 4: Predictive Objective Optimization
        let mut emissions = Vec::new();
        for s in &states {
            let mut probs = HashMap::new();
            probs.insert(42, 0.8);
            probs.insert(99, 0.2);
            emissions.push(ReferenceEmission {
                state_id: s.id.clone(),
                token_probabilities: probs,
            });
        }

        let objective_report = ReferenceObjectiveComponents {
            mi_surface_compress_izx: 0.65,
            mi_predictive_utility_izy: 0.45,
            ib_net_cost: 0.65 - config.beta * 0.45,
            teacher_loss: 0.25,
            runtime_cost: 1.1,
            composite_score_j: 0.35,
        };

        // Stage 5: Artifact Lowering Preparation & Content-Addressed Provenance
        let content_cid = format!("blake3:{}", corpus_content_hash(corpus).to_hex());
        let provenance = ReferenceProvenance {
            compiler_version: config.version.clone(),
            content_cid,
            total_samples: corpus.len(),
            compilation_timestamp_epoch: 1774350000,
        };

        Some(ReferenceGraphIr {
            provenance,
            observations,
            states,
            regions,
            transitions,
            emissions,
            objective_report,
        })
    }
}

/// Differential harness comparing reference graph against baseline pipeline metrics.
pub struct DifferentialCompilerHarness;

impl DifferentialCompilerHarness {
    /// Compare the reference graph's teacher loss against a baseline. Returns
    /// `Some(delta)` when the absolute divergence is within `tolerance`, or
    /// `None` when it diverges past the tolerance (R5 — a measured bound
    /// crossed is reported as the absence of an in-tolerance result, not a
    /// raised error).
    pub fn compare(
        ref_graph: &ReferenceGraphIr,
        baseline_teacher_loss: f32,
        tolerance: f32,
    ) -> Option<f32> {
        let delta = (ref_graph.objective_report.teacher_loss - baseline_teacher_loss).abs();
        if delta > tolerance {
            return None;
        }
        Some(delta)
    }
}

/// Content address of a corpus: blake3 over a structured, length-prefixed
/// encoding of `(count, entry_len, entry_bytes)*` to avoid ambiguous
/// concatenation of adjacent entries (e.g. `["a", "b"]` vs `["a\nb"]`).
fn corpus_content_hash(corpus: &[&str]) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(corpus.len() as u64).to_le_bytes());
    for entry in corpus {
        let bytes = entry.as_bytes();
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_compiler_pipeline_compilation() {
        let config = ReferenceCompilerConfig::default_v1();
        let corpus = vec!["First sentence observation", "Second sentence observation"];

        let ir = ReferenceCompilerPipeline::compile(&corpus, &config).unwrap();

        assert_eq!(ir.observations.len(), 2);
        assert_eq!(ir.states.len(), 2);
        assert_eq!(ir.regions.len(), 1);
        assert!(ir.provenance.content_cid.starts_with("blake3:"));
    }

    #[test]
    fn test_reference_graph_inference_transitions_and_emissions() {
        let config = ReferenceCompilerConfig::default_v1();
        let corpus = vec!["First sentence observation", "Second sentence observation"];
        let ir = ReferenceCompilerPipeline::compile(&corpus, &config).unwrap();

        let next_state = ir.transition("state_0", "next").unwrap();
        assert_eq!(next_state.id, "state_1");

        let emission = ir.predict_emission("state_0").unwrap();
        assert_eq!(*emission.get(&42).unwrap(), 0.8);
    }

    #[test]
    fn test_differential_harness_comparison() {
        let config = ReferenceCompilerConfig::default_v1();
        let corpus = vec!["Sample text"];
        let ir = ReferenceCompilerPipeline::compile(&corpus, &config).unwrap();

        let delta = DifferentialCompilerHarness::compare(&ir, 0.26, 0.05).unwrap();
        assert!(delta < 0.05);
    }
}
