//! Sparse scalar gates and offline learning for the integrated attention arm.
//!
//! Only [`SparseGate`] is used at serving: it reads a bias and the selected
//! exact feature-bank rows, adds signed four-bit integers, and enables a read
//! or write at score >= 0. `SparseGateTrainer` and `EnergyTrainer` are offline
//! Rust fitting helpers; their FP32 sigmoid/softmax calculations never enter
//! the served numerical kernel. No hash or dense feature projection is hidden
//! here: callers supply exact `u16` bank IDs and count their own feature work.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

use super::super::group_table::GROUP_ORDER;
use super::geometry::{EnergyTables, GeometryError};

const GATE_VERSION: u16 = 1;
const MAX_FEATURES: usize = (u16::MAX as usize) + 1;
const MAX_ACTIVE: usize = 64;
// The hard reader admits at most 64. Offline source-credit may insert one
// labeled positive into those negatives; the served candidate cap stays 64.
const MAX_CANDIDATES: usize = 65;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrainingSupportError {
    Geometry(GeometryError),
    UnsupportedVersion(u16),
    InvalidFeatureCount(usize),
    TooManyActiveFeatures(usize),
    FeatureOutOfRange(u16),
    InvalidGateWeight(i8),
    InvalidLearningRate,
    InvalidCandidateCount(usize),
    InvalidTargetIndex(usize),
    InvalidCandidateWidth { expected: usize, actual: usize },
    InvalidRelativeId(u8),
    NonFiniteTrainingValue,
    GateScoreOverflow,
    CounterOverflow,
}

impl fmt::Display for TrainingSupportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry(e) => write!(f, "geometry error: {e}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported sparse-gate version {v}"),
            Self::InvalidFeatureCount(n) => {
                write!(f, "feature count {n} outside 1..={MAX_FEATURES}")
            }
            Self::TooManyActiveFeatures(n) => write!(f, "{n} active features exceed {MAX_ACTIVE}"),
            Self::FeatureOutOfRange(id) => write!(f, "feature bank ID {id} outside gate rows"),
            Self::InvalidGateWeight(w) => {
                write!(f, "gate weight {w} outside signed four-bit range")
            }
            Self::InvalidLearningRate => write!(f, "learning rate must be finite and in (0,1]"),
            Self::InvalidCandidateCount(n) => {
                write!(f, "candidate count {n} outside 1..={MAX_CANDIDATES}")
            }
            Self::InvalidTargetIndex(i) => write!(f, "target candidate index {i} is unavailable"),
            Self::InvalidCandidateWidth { expected, actual } => {
                write!(f, "candidate width {actual}, expected {expected}")
            }
            Self::InvalidRelativeId(id) => write!(f, "relative code {id} outside 0..120"),
            Self::NonFiniteTrainingValue => write!(f, "non-finite offline training value"),
            Self::GateScoreOverflow => write!(f, "sparse gate score overflow"),
            Self::CounterOverflow => write!(f, "sparse gate read counter overflow"),
        }
    }
}

impl std::error::Error for TrainingSupportError {}

impl From<GeometryError> for TrainingSupportError {
    fn from(value: GeometryError) -> Self {
        Self::Geometry(value)
    }
}

/// Actual learned coefficient/byte reads for a sparse gate evaluation.
/// Repeated feature IDs are counted repeatedly and contribute repeatedly.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReadCounts {
    pub coefficients: u64,
    pub parameter_bytes: u64,
}

impl GateReadCounts {
    fn record_many(&mut self, n: usize) -> Result<(), TrainingSupportError> {
        let n = u64::try_from(n).map_err(|_| TrainingSupportError::CounterOverflow)?;
        let coefficients = self
            .coefficients
            .checked_add(n)
            .ok_or(TrainingSupportError::CounterOverflow)?;
        let parameter_bytes = self
            .parameter_bytes
            .checked_add(n)
            .ok_or(TrainingSupportError::CounterOverflow)?;
        self.coefficients = coefficients;
        self.parameter_bytes = parameter_bytes;
        Ok(())
    }
}

/// Artifact-serde, integer-served scalar gate. The bias is a learned signed
/// four-bit coefficient and therefore counts as one selected parameter read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseGate {
    format_version: u16,
    bias: i8,
    rows: Vec<i8>,
}

impl SparseGate {
    /// Zero rows and positive bias make read/write possible before training.
    pub fn new(feature_count: usize) -> Result<Self, TrainingSupportError> {
        if feature_count == 0 || feature_count > MAX_FEATURES {
            return Err(TrainingSupportError::InvalidFeatureCount(feature_count));
        }
        Ok(Self {
            format_version: GATE_VERSION,
            bias: 1,
            rows: vec![0; feature_count],
        })
    }

    pub fn validate(&self) -> Result<(), TrainingSupportError> {
        if self.format_version != GATE_VERSION {
            return Err(TrainingSupportError::UnsupportedVersion(
                self.format_version,
            ));
        }
        if self.rows.is_empty() || self.rows.len() > MAX_FEATURES {
            return Err(TrainingSupportError::InvalidFeatureCount(self.rows.len()));
        }
        check_gate_weight(self.bias)?;
        for &weight in &self.rows {
            check_gate_weight(weight)?;
        }
        Ok(())
    }

    pub fn feature_count(&self) -> usize {
        self.rows.len()
    }

    /// Meaningful low-bit learned coefficients, including the bias.
    pub fn coefficient_slots(&self) -> usize {
        self.rows.len() + 1
    }

    /// One signed coefficient per byte. This is not a packed-nibble backing.
    pub fn parameter_bytes(&self) -> usize {
        self.rows.len() + 1
    }

    pub const fn bias(&self) -> i8 {
        self.bias
    }

    pub fn get_weight(&self, feature: u16) -> Result<i8, TrainingSupportError> {
        let weight = self
            .rows
            .get(usize::from(feature))
            .copied()
            .ok_or(TrainingSupportError::FeatureOutOfRange(feature))?;
        check_gate_weight(weight)?;
        Ok(weight)
    }

    /// Selected integer reads/additions only; no serving float, division,
    /// multiplication, allocation or all-row scan. Feature multiplicity counts.
    pub fn score(
        &self,
        features: &[u16],
        reads: &mut GateReadCounts,
    ) -> Result<i32, TrainingSupportError> {
        if self.format_version != GATE_VERSION {
            return Err(TrainingSupportError::UnsupportedVersion(
                self.format_version,
            ));
        }
        if features.len() > MAX_ACTIVE {
            return Err(TrainingSupportError::TooManyActiveFeatures(features.len()));
        }
        check_gate_weight(self.bias)?;
        for &id in features {
            if usize::from(id) >= self.rows.len() {
                return Err(TrainingSupportError::FeatureOutOfRange(id));
            }
        }
        let mut sum = i32::from(self.bias);
        for &id in features {
            let weight = self.get_weight(id)?;
            sum = sum
                .checked_add(i32::from(weight))
                .ok_or(TrainingSupportError::GateScoreOverflow)?;
        }
        reads.record_many(features.len() + 1)?;
        Ok(sum)
    }

    pub fn enabled(
        &self,
        features: &[u16],
        reads: &mut GateReadCounts,
    ) -> Result<bool, TrainingSupportError> {
        Ok(self.score(features, reads)? >= 0)
    }
}

fn check_gate_weight(weight: i8) -> Result<(), TrainingSupportError> {
    if (-8..=7).contains(&weight) {
        Ok(())
    } else {
        Err(TrainingSupportError::InvalidGateWeight(weight))
    }
}

fn check_lr(lr: f64) -> Result<f32, TrainingSupportError> {
    if !lr.is_finite() || lr <= 0.0 || lr > 1.0 {
        return Err(TrainingSupportError::InvalidLearningRate);
    }
    let narrowed = lr as f32;
    if narrowed == 0.0 {
        return Err(TrainingSupportError::InvalidLearningRate);
    }
    Ok(narrowed)
}

fn quantize_signed4(master: f32) -> Result<i8, TrainingSupportError> {
    if !master.is_finite() {
        return Err(TrainingSupportError::NonFiniteTrainingValue);
    }
    Ok(master.clamp(-8.0, 7.0).round() as i8)
}

fn stable_sigmoid(logit: f32) -> f32 {
    if logit >= 0.0 {
        1.0 / (1.0 + (-logit).exp())
    } else {
        let e = logit.exp();
        e / (1.0 + e)
    }
}

fn binary_cross_entropy_from_logit(logit: f32, target: f32) -> f32 {
    if logit >= 0.0 {
        (1.0 - target) * logit + (-logit).exp().ln_1p()
    } else {
        logit.exp().ln_1p() - target * logit
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GateTrainStats {
    pub loss: f32,
    pub probability_before: f32,
    pub master_logit_before: f32,
    pub hard_score_after: i32,
}

/// Sparse selected-row offline logistic trainer. FP32 master values accumulate
/// sub-quantum updates; the exported gate mirrors rounded signed-four-bit rows.
#[derive(Debug, Clone)]
pub struct SparseGateTrainer {
    model: SparseGate,
    master_bias: f32,
    master_rows: Vec<f32>,
}

impl SparseGateTrainer {
    pub fn new(feature_count: usize) -> Result<Self, TrainingSupportError> {
        Self::from_model(SparseGate::new(feature_count)?)
    }

    pub fn from_model(model: SparseGate) -> Result<Self, TrainingSupportError> {
        model.validate()?;
        let master_bias = f32::from(model.bias);
        let master_rows = model.rows.iter().map(|&w| f32::from(w)).collect();
        Ok(Self {
            model,
            master_bias,
            master_rows,
        })
    }

    pub fn model(&self) -> &SparseGate {
        &self.model
    }

    pub fn export(&self) -> SparseGate {
        self.model.clone()
    }

    pub fn fp32_master_bytes(&self) -> usize {
        (self.master_rows.len() + 1) << 2
    }

    pub fn train(
        &mut self,
        features: &[u16],
        target_enabled: bool,
        lr: f64,
    ) -> Result<GateTrainStats, TrainingSupportError> {
        let lr = check_lr(lr)?;
        if features.len() > MAX_ACTIVE {
            return Err(TrainingSupportError::TooManyActiveFeatures(features.len()));
        }
        for &id in features {
            if usize::from(id) >= self.master_rows.len() {
                return Err(TrainingSupportError::FeatureOutOfRange(id));
            }
        }
        let mut logit = self.master_bias;
        for &id in features {
            logit += self.master_rows[usize::from(id)];
        }
        if !logit.is_finite() {
            return Err(TrainingSupportError::NonFiniteTrainingValue);
        }
        let target = if target_enabled { 1.0 } else { 0.0 };
        let probability = stable_sigmoid(logit);
        let loss = binary_cross_entropy_from_logit(logit, target);
        if !loss.is_finite() || !probability.is_finite() {
            return Err(TrainingSupportError::NonFiniteTrainingValue);
        }
        let delta = lr * (probability - target);
        self.master_bias = (self.master_bias - delta).clamp(-8.0, 7.0);
        self.model.bias = quantize_signed4(self.master_bias)?;
        for &id in features {
            let index = usize::from(id);
            self.master_rows[index] = (self.master_rows[index] - delta).clamp(-8.0, 7.0);
            self.model.rows[index] = quantize_signed4(self.master_rows[index])?;
        }
        let hard_score_after = self.model.score(features, &mut GateReadCounts::default())?;
        Ok(GateTrainStats {
            loss,
            probability_before: probability,
            master_logit_before: logit,
            hard_score_after,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyTrainStats {
    pub loss: f32,
    pub target_probability_before: f32,
    pub target_energy_before: f32,
    /// Number of selected factor occurrences updated, including repeats.
    pub selected_coefficient_updates: usize,
}

/// Offline categorical trainer for the hard relative-code energy tables.
/// Softmax is taken over `-energy` for the provided bounded candidate set.
/// `NoRead` and legal scope/version masks are handled by the caller's gate.
#[derive(Debug, Clone)]
pub struct EnergyTrainer {
    model: EnergyTables,
    master_unary: Vec<f32>,
    master_pair: Vec<f32>,
}

impl EnergyTrainer {
    pub fn new(model: EnergyTables) -> Result<Self, TrainingSupportError> {
        model.validate()?;
        let lanes = usize::from(model.lanes());
        let mut master_unary = Vec::with_capacity(lanes * GROUP_ORDER);
        for lane in 0..lanes {
            for id in 0..GROUP_ORDER {
                master_unary.push(f32::from(model.get_unary(lane as u8, id as u8)?));
            }
        }
        let mut master_pair = Vec::with_capacity(model.edges().len() * GROUP_ORDER * GROUP_ORDER);
        for edge in 0..model.edges().len() {
            for left in 0..GROUP_ORDER {
                for right in 0..GROUP_ORDER {
                    master_pair.push(f32::from(model.get_pair(edge, left as u8, right as u8)?));
                }
            }
        }
        Ok(Self {
            model,
            master_unary,
            master_pair,
        })
    }

    pub fn model(&self) -> &EnergyTables {
        &self.model
    }

    pub fn export(&self) -> EnergyTables {
        self.model.clone()
    }

    pub fn coefficient_slots(&self) -> usize {
        self.model.coefficient_slots()
    }

    pub fn packed_parameter_bytes(&self) -> usize {
        self.model.packed_bytes()
    }

    pub fn fp32_master_bytes(&self) -> usize {
        (self.master_unary.len() + self.master_pair.len()) << 2
    }

    fn unary_index(lane: usize, id: u8) -> usize {
        lane * GROUP_ORDER + usize::from(id)
    }

    fn pair_index(edge: usize, left: u8, right: u8) -> usize {
        edge * GROUP_ORDER * GROUP_ORDER + usize::from(left) * GROUP_ORDER + usize::from(right)
    }

    fn master_energy(&self, relatives: &[u8]) -> f32 {
        let mut sum = 0.0f32;
        for (lane, &id) in relatives.iter().enumerate() {
            sum += self.master_unary[Self::unary_index(lane, id)];
        }
        for (edge_index, edge) in self.model.edges().iter().enumerate() {
            let left = relatives[usize::from(edge.left)];
            let right = relatives[usize::from(edge.right)];
            sum += self.master_pair[Self::pair_index(edge_index, left, right)];
        }
        sum
    }

    /// One offline sparse categorical update over already-admitted relative
    /// candidate codes. Positive target energy is lowered; negative energies
    /// rise in proportion to their softmax probability. The serving artifact
    /// remains quantized after each step for hard-forward validation.
    pub fn train_choice(
        &mut self,
        candidates: &[Vec<u8>],
        target_index: usize,
        lr: f64,
    ) -> Result<EnergyTrainStats, TrainingSupportError> {
        let lr = check_lr(lr)?;
        if candidates.is_empty() || candidates.len() > MAX_CANDIDATES {
            return Err(TrainingSupportError::InvalidCandidateCount(
                candidates.len(),
            ));
        }
        if target_index >= candidates.len() {
            return Err(TrainingSupportError::InvalidTargetIndex(target_index));
        }
        let lanes = usize::from(self.model.lanes());
        for candidate in candidates {
            if candidate.len() != lanes {
                return Err(TrainingSupportError::InvalidCandidateWidth {
                    expected: lanes,
                    actual: candidate.len(),
                });
            }
            for &id in candidate {
                if usize::from(id) >= GROUP_ORDER {
                    return Err(TrainingSupportError::InvalidRelativeId(id));
                }
            }
        }
        let mut logits = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let energy = self.master_energy(candidate);
            if !energy.is_finite() {
                return Err(TrainingSupportError::NonFiniteTrainingValue);
            }
            logits.push(-energy);
        }
        let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut total = 0.0f32;
        let mut probabilities = Vec::with_capacity(logits.len());
        for &logit in &logits {
            let unnormalized = (logit - max_logit).exp();
            probabilities.push(unnormalized);
            total += unnormalized;
        }
        if !total.is_finite() || total <= 0.0 {
            return Err(TrainingSupportError::NonFiniteTrainingValue);
        }
        for probability in &mut probabilities {
            *probability /= total;
        }
        let loss = max_logit + total.ln() - logits[target_index];
        if !loss.is_finite() {
            return Err(TrainingSupportError::NonFiniteTrainingValue);
        }
        let target_probability_before = probabilities[target_index];
        let target_energy_before = -logits[target_index];
        let mut selected_coefficient_updates = 0usize;
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let expected = if candidate_index == target_index {
                1.0
            } else {
                0.0
            };
            // d CE / d energy_i = expected - probability_i.
            // Gradient descent therefore adds lr * (probability_i - expected).
            let delta = lr * (probabilities[candidate_index] - expected);
            for (lane, &id) in candidate.iter().enumerate() {
                let index = Self::unary_index(lane, id);
                self.master_unary[index] = (self.master_unary[index] + delta).clamp(-8.0, 7.0);
                let quantized = quantize_signed4(self.master_unary[index])?;
                self.model.set_unary(lane as u8, id, quantized)?;
                selected_coefficient_updates += 1;
            }
            for edge_index in 0..self.model.edges().len() {
                let edge = self.model.edges()[edge_index];
                let left = candidate[usize::from(edge.left)];
                let right = candidate[usize::from(edge.right)];
                let index = Self::pair_index(edge_index, left, right);
                self.master_pair[index] = (self.master_pair[index] + delta).clamp(-8.0, 7.0);
                let quantized = quantize_signed4(self.master_pair[index])?;
                self.model.set_pair(edge_index, left, right, quantized)?;
                selected_coefficient_updates += 1;
            }
        }
        Ok(EnergyTrainStats {
            loss,
            target_probability_before,
            target_energy_before,
            selected_coefficient_updates,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::geometry::{EnergyReadCounts, LanePair};
    use super::*;

    #[test]
    fn gate_initially_enabled_and_trains_selected_exact_feature_rows() {
        let mut trainer = SparseGateTrainer::new(8).unwrap();
        assert_eq!(trainer.model().coefficient_slots(), 9);
        assert_eq!(trainer.fp32_master_bytes(), 36);
        let mut reads = GateReadCounts::default();
        assert!(trainer.model().enabled(&[2, 5], &mut reads).unwrap());
        assert_eq!(reads.coefficients, 3);
        assert_eq!(reads.parameter_bytes, 3);
        for _ in 0..12 {
            trainer.train(&[], false, 1.0).unwrap();
        }
        assert!(!trainer
            .model()
            .enabled(&[], &mut GateReadCounts::default())
            .unwrap());
        for _ in 0..12 {
            trainer.train(&[2], true, 1.0).unwrap();
        }
        assert!(trainer
            .model()
            .enabled(&[2], &mut GateReadCounts::default())
            .unwrap());
        let model = trainer.export();
        let loaded: SparseGate =
            serde_json::from_slice(&serde_json::to_vec(&model).unwrap()).unwrap();
        loaded.validate().unwrap();
        assert_eq!(loaded, model);
    }

    #[test]
    fn gate_rejects_unbounded_or_invalid_features_and_weights() {
        assert!(matches!(
            SparseGate::new(0),
            Err(TrainingSupportError::InvalidFeatureCount(0))
        ));
        let mut gate = SparseGate::new(4).unwrap();
        assert_eq!(
            gate.score(&[4], &mut GateReadCounts::default()),
            Err(TrainingSupportError::FeatureOutOfRange(4))
        );
        assert!(matches!(
            gate.score(&[0; 65], &mut GateReadCounts::default()),
            Err(TrainingSupportError::TooManyActiveFeatures(65))
        ));
        gate.rows[0] = 8;
        assert_eq!(
            gate.validate(),
            Err(TrainingSupportError::InvalidGateWeight(8))
        );
        assert_eq!(
            gate.score(&[0], &mut GateReadCounts::default()),
            Err(TrainingSupportError::InvalidGateWeight(8))
        );
        assert_eq!(
            SparseGateTrainer::new(2)
                .unwrap()
                .train(&[], true, f64::NAN),
            Err(TrainingSupportError::InvalidLearningRate)
        );
    }

    #[test]
    fn energy_trainer_lowers_target_energy_and_raises_negative() {
        let model = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }]).unwrap();
        let mut trainer = EnergyTrainer::new(model).unwrap();
        assert_eq!(trainer.coefficient_slots(), 2 * 120 + 120 * 120);
        assert_eq!(trainer.packed_parameter_bytes(), 2 * 64 + 8192);
        for _ in 0..4 {
            let stats = trainer
                .train_choice(&[vec![1, 3], vec![2, 4]], 0, 1.0)
                .unwrap();
            assert_eq!(stats.selected_coefficient_updates, 6);
            assert!(stats.loss.is_finite());
        }
        let mut counts = EnergyReadCounts::default();
        let target = trainer.model().score(&[1, 3], &mut counts).unwrap();
        let negative = trainer.model().score(&[2, 4], &mut counts).unwrap();
        assert!(target < negative);
        assert_eq!(counts.coefficients, 6);
        trainer.export().validate().unwrap();
    }

    #[test]
    fn energy_training_rejects_invalid_candidates_without_mutation() {
        let model = EnergyTables::zeroed(1, vec![]).unwrap();
        let mut trainer = EnergyTrainer::new(model).unwrap();
        let before = trainer.export();
        assert_eq!(
            trainer.train_choice(&[vec![120]], 0, 0.1),
            Err(TrainingSupportError::InvalidRelativeId(120))
        );
        assert_eq!(
            trainer.train_choice(&[vec![1]], 1, 0.1),
            Err(TrainingSupportError::InvalidTargetIndex(1))
        );
        assert_eq!(
            trainer.train_choice(&[vec![1]], 0, 0.0),
            Err(TrainingSupportError::InvalidLearningRate)
        );
        assert_eq!(trainer.export(), before);
    }

    #[test]
    fn offline_positive_insertion_allows_sixty_four_negatives_plus_one() {
        let model = EnergyTables::zeroed(1, vec![]).unwrap();
        let mut trainer = EnergyTrainer::new(model).unwrap();
        let candidates = vec![vec![0u8]; 65];
        assert!(trainer.train_choice(&candidates, 64, 0.1).is_ok());
    }
}
