//! Selected-row, full-support vocabulary and legal-action output.
//!
//! Rows 0 and 1 form a mode tree: Stop/Continue, then (when copying is legal)
//! Copy/Generate. Rows 2 onward form a complete binary vocabulary tree. At
//! each visited node, the serving kernel adds one bias and at most eight
//! caller-selected, exact feature-ID coefficients. A fixed dyadic lookup turns
//! the bounded sum into a nonzero branch probability. The serving methods use
//! integer add/subtract/compare/shift and table reads; training-only methods
//! below use floating point. Coefficients occupy `i8` bytes in this artifact
//! even though their declared value range is signed four bit.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const OUTPUT_VERSION: u8 = 1;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_FEATURE_BANK: usize = 16_384;
pub const MAX_ACTIVE_FEATURES: usize = 8;
pub const MAX_PATH_STEPS: usize = 14;
const MODE_STOP_ROW: usize = 0;
const MODE_COPY_ROW: usize = 1;
const VOCAB_ROW_BASE: usize = 2;
const MIN_CODE: i8 = -8;
const MAX_CODE: i8 = 7;

// round(256 * sigmoid(logit / 4)), clamped to [1,255], for logits -16..16.
// This is a fixed declared inference table; its generation is not in serving.
const RIGHT_NUMERATOR: [u16; 33] = [
    5, 6, 8, 10, 12, 15, 19, 24, 31, 38, 47, 57, 69, 82, 97, 112, 128, 144, 159, 174, 187, 199,
    209, 218, 225, 232, 237, 241, 244, 246, 248, 250, 251,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Generate(u16),
    Copy,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputError {
    InvalidShape,
    InvalidCoefficient,
    InvalidFeature,
    InvalidAction,
    InsufficientDraws,
    InvalidLearningRate,
    CorruptArtifact,
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for OutputError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchTrace {
    /// Selected row: 0 Stop/Continue, 1 Copy/Generate, 2+ vocabulary node.
    pub row: u16,
    /// Whether the right branch was taken.
    pub right: bool,
    /// Exact right-branch probability numerator over 256.
    pub right_numerator: u16,
    /// Integer pre-lookup logit, before saturation to [-16,16].
    pub logit: i16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputTrace {
    pub action: Action,
    pub steps: [BranchTrace; MAX_PATH_STEPS],
    pub len: u8,
    /// Number of selected learned coefficient slots read, including biases.
    pub coefficient_reads: u32,
    /// Uniform random bytes consumed; zero for scoring or greedy decoding.
    pub draws_used: u8,
}

impl OutputTrace {
    fn new(action: Action) -> Self {
        Self {
            action,
            steps: [BranchTrace::default(); MAX_PATH_STEPS],
            len: 0,
            coefficient_reads: 0,
            draws_used: 0,
        }
    }

    fn push(
        &mut self,
        row: usize,
        right: bool,
        right_numerator: u16,
        logit: i16,
        feature_count: usize,
    ) -> Result<(), OutputError> {
        let index = usize::from(self.len);
        let Some(slot) = self.steps.get_mut(index) else {
            return Err(OutputError::CorruptArtifact);
        };
        *slot = BranchTrace {
            row: u16::try_from(row).map_err(|_| OutputError::CorruptArtifact)?,
            right,
            right_numerator,
            logit,
        };
        self.len += 1;
        self.coefficient_reads = self
            .coefficient_reads
            .checked_add(u32::try_from(feature_count + 1).map_err(|_| OutputError::InvalidShape)?)
            .ok_or(OutputError::InvalidShape)?;
        Ok(())
    }

    /// Offline exact-dyadic action-path negative log likelihood in natural units.
    /// This is action likelihood, not surface-token likelihood when Generate and
    /// Copy can emit the same token and induce different future states.
    pub fn action_ce_offline(&self) -> f64 {
        let mut loss = 0.0_f64;
        for step in self.steps.iter().take(usize::from(self.len)) {
            let numerator = if step.right {
                step.right_numerator
            } else {
                256 - step.right_numerator
            };
            loss -= (f64::from(numerator) / 256.0).ln();
        }
        loss
    }
}

/// A complete binary vocabulary tree with two normalized mode decisions.
/// `row_offsets` are built at construction so serving does not multiply a
/// variable row ID by a learned-row stride in its numerical kernel. An artifact
/// loader must call `validate()` once after deserialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparseOutput {
    pub version: u8,
    pub vocab: u16,
    pub feature_bank: u16,
    pub max_active: u8,
    coefficients: Vec<i8>,
    row_offsets: Vec<usize>,
}

impl SparseOutput {
    pub fn new(vocab: usize, feature_bank: usize, max_active: usize) -> Result<Self, OutputError> {
        if !(2..=MAX_VOCAB).contains(&vocab)
            || !vocab.is_power_of_two()
            || !(1..=MAX_FEATURE_BANK).contains(&feature_bank)
            || !(1..=MAX_ACTIVE_FEATURES).contains(&max_active)
        {
            return Err(OutputError::InvalidShape);
        }
        let stride = feature_bank
            .checked_add(1)
            .ok_or(OutputError::InvalidShape)?;
        let row_count = vocab.checked_add(1).ok_or(OutputError::InvalidShape)?;
        let mut row_offsets = Vec::with_capacity(row_count);
        let mut offset = 0usize;
        for _ in 0..row_count {
            row_offsets.push(offset);
            offset = offset
                .checked_add(stride)
                .ok_or(OutputError::InvalidShape)?;
        }
        Ok(Self {
            version: OUTPUT_VERSION,
            vocab: u16::try_from(vocab).map_err(|_| OutputError::InvalidShape)?,
            feature_bank: u16::try_from(feature_bank).map_err(|_| OutputError::InvalidShape)?,
            max_active: u8::try_from(max_active).map_err(|_| OutputError::InvalidShape)?,
            coefficients: vec![0; offset],
            row_offsets,
        })
    }

    pub fn from_coefficients(
        vocab: usize,
        feature_bank: usize,
        max_active: usize,
        coefficients: Vec<i8>,
    ) -> Result<Self, OutputError> {
        let mut model = Self::new(vocab, feature_bank, max_active)?;
        if coefficients.len() != model.coefficients.len() {
            return Err(OutputError::InvalidShape);
        }
        model.coefficients = coefficients;
        model.validate()?;
        Ok(model)
    }

    pub fn coefficients(&self) -> &[i8] {
        &self.coefficients
    }

    pub fn stored_coefficient_bytes(&self) -> usize {
        self.coefficients.len()
    }

    pub fn validate(&self) -> Result<(), OutputError> {
        let vocab = usize::from(self.vocab);
        let bank = usize::from(self.feature_bank);
        if self.version != OUTPUT_VERSION
            || !(2..=MAX_VOCAB).contains(&vocab)
            || !vocab.is_power_of_two()
            || !(1..=MAX_FEATURE_BANK).contains(&bank)
            || !(1..=MAX_ACTIVE_FEATURES).contains(&usize::from(self.max_active))
            || self.row_offsets.len() != vocab + 1
        {
            return Err(OutputError::InvalidShape);
        }
        let stride = bank + 1;
        let mut expected = 0usize;
        for offset in &self.row_offsets {
            if *offset != expected {
                return Err(OutputError::CorruptArtifact);
            }
            expected = expected
                .checked_add(stride)
                .ok_or(OutputError::InvalidShape)?;
        }
        if self.coefficients.len() != expected {
            return Err(OutputError::InvalidShape);
        }
        if self
            .coefficients
            .iter()
            .any(|code| !(MIN_CODE..=MAX_CODE).contains(code))
        {
            return Err(OutputError::InvalidCoefficient);
        }
        Ok(())
    }

    fn check_features(&self, features: &[u16]) -> Result<(), OutputError> {
        if features.len() > usize::from(self.max_active) {
            return Err(OutputError::InvalidFeature);
        }
        for (index, feature) in features.iter().enumerate() {
            if usize::from(*feature) >= usize::from(self.feature_bank)
                || features[..index].contains(feature)
            {
                return Err(OutputError::InvalidFeature);
            }
        }
        Ok(())
    }

    fn check_action(&self, copy_legal: bool, action: Action) -> Result<(), OutputError> {
        match action {
            Action::Generate(token) if token < self.vocab => Ok(()),
            Action::Copy if copy_legal => Ok(()),
            Action::Stop => Ok(()),
            _ => Err(OutputError::InvalidAction),
        }
    }

    fn coefficient_index(&self, row: usize, feature: Option<u16>) -> Result<usize, OutputError> {
        let base = *self
            .row_offsets
            .get(row)
            .ok_or(OutputError::CorruptArtifact)?;
        let index = match feature {
            Some(id) => base
                .checked_add(1 + usize::from(id))
                .ok_or(OutputError::CorruptArtifact)?,
            None => base,
        };
        if index >= self.coefficients.len() {
            return Err(OutputError::CorruptArtifact);
        }
        Ok(index)
    }

    fn score_row(&self, row: usize, features: &[u16]) -> Result<(i16, u16), OutputError> {
        let bias_index = self.coefficient_index(row, None)?;
        let mut logit = i16::from(self.coefficients[bias_index]);
        if !(MIN_CODE..=MAX_CODE).contains(&self.coefficients[bias_index]) {
            return Err(OutputError::InvalidCoefficient);
        }
        for id in features {
            let index = self.coefficient_index(row, Some(*id))?;
            let code = self.coefficients[index];
            if !(MIN_CODE..=MAX_CODE).contains(&code) {
                return Err(OutputError::InvalidCoefficient);
            }
            logit += i16::from(code);
        }
        let table_index = usize::try_from(i32::from(logit).clamp(-16, 16) + 16)
            .map_err(|_| OutputError::CorruptArtifact)?;
        let numerator = *RIGHT_NUMERATOR
            .get(table_index)
            .ok_or(OutputError::CorruptArtifact)?;
        Ok((logit, numerator))
    }

    fn push_scored(
        &self,
        trace: &mut OutputTrace,
        row: usize,
        right: bool,
        features: &[u16],
    ) -> Result<(), OutputError> {
        let (logit, numerator) = self.score_row(row, features)?;
        trace.push(row, right, numerator, logit, features.len())
    }

    /// Scores a declared legal *action* by visiting only its mode and token
    /// path. It never scans all vocabulary nodes. `action_ce_offline()` on the
    /// result gives the exact dyadic action-path CE, not marginal token CE.
    pub fn score_action(
        &self,
        features: &[u16],
        copy_legal: bool,
        action: Action,
    ) -> Result<OutputTrace, OutputError> {
        self.check_features(features)?;
        self.check_action(copy_legal, action)?;
        let mut trace = OutputTrace::new(action);
        self.push_scored(&mut trace, MODE_STOP_ROW, action == Action::Stop, features)?;
        if action == Action::Stop {
            return Ok(trace);
        }
        if copy_legal {
            self.push_scored(&mut trace, MODE_COPY_ROW, action == Action::Copy, features)?;
        }
        if let Action::Generate(token) = action {
            let mut node = 0usize;
            let depth = self.vocab.trailing_zeros();
            for level in 0..depth {
                let shift = depth - level - 1;
                let right = (token >> shift) & 1 == 1;
                self.push_scored(&mut trace, VOCAB_ROW_BASE + node, right, features)?;
                node = (node << 1) + 1 + usize::from(right);
            }
        }
        Ok(trace)
    }

    /// A declared local greedy-branch decoder. It is not global token MAP.
    pub fn greedy_branch(
        &self,
        features: &[u16],
        copy_legal: bool,
    ) -> Result<OutputTrace, OutputError> {
        self.check_features(features)?;
        let mut trace = OutputTrace::new(Action::Stop);
        let (logit, numerator) = self.score_row(MODE_STOP_ROW, features)?;
        let stop = numerator > 128;
        trace.push(MODE_STOP_ROW, stop, numerator, logit, features.len())?;
        if stop {
            return Ok(trace);
        }
        if copy_legal {
            let (logit, numerator) = self.score_row(MODE_COPY_ROW, features)?;
            let copy = numerator > 128;
            trace.push(MODE_COPY_ROW, copy, numerator, logit, features.len())?;
            if copy {
                trace.action = Action::Copy;
                return Ok(trace);
            }
        }
        let mut node = 0usize;
        let mut token = 0u16;
        for _ in 0..self.vocab.trailing_zeros() {
            let row = VOCAB_ROW_BASE + node;
            let (logit, numerator) = self.score_row(row, features)?;
            let right = numerator > 128;
            trace.push(row, right, numerator, logit, features.len())?;
            token = (token << 1) | u16::from(right);
            node = (node << 1) + 1 + usize::from(right);
        }
        trace.action = Action::Generate(token);
        Ok(trace)
    }

    /// Exact ancestral sampling for the declared quantized dyadic action
    /// distribution when each supplied byte is an independent uniform draw.
    /// The caller owns and records its random source/counter for replay.
    pub fn sample_from_bytes(
        &self,
        features: &[u16],
        copy_legal: bool,
        draws: &[u8],
    ) -> Result<OutputTrace, OutputError> {
        self.check_features(features)?;
        let mut trace = OutputTrace::new(Action::Stop);
        let mut used = 0usize;
        let (logit, numerator) = self.score_row(MODE_STOP_ROW, features)?;
        let draw = *draws.get(used).ok_or(OutputError::InsufficientDraws)?;
        used += 1;
        let stop = u16::from(draw) < numerator;
        trace.push(MODE_STOP_ROW, stop, numerator, logit, features.len())?;
        if stop {
            trace.draws_used = u8::try_from(used).map_err(|_| OutputError::InvalidShape)?;
            return Ok(trace);
        }
        if copy_legal {
            let (logit, numerator) = self.score_row(MODE_COPY_ROW, features)?;
            let draw = *draws.get(used).ok_or(OutputError::InsufficientDraws)?;
            used += 1;
            let copy = u16::from(draw) < numerator;
            trace.push(MODE_COPY_ROW, copy, numerator, logit, features.len())?;
            if copy {
                trace.action = Action::Copy;
                trace.draws_used = u8::try_from(used).map_err(|_| OutputError::InvalidShape)?;
                return Ok(trace);
            }
        }
        let mut node = 0usize;
        let mut token = 0u16;
        for _ in 0..self.vocab.trailing_zeros() {
            let row = VOCAB_ROW_BASE + node;
            let (logit, numerator) = self.score_row(row, features)?;
            let draw = *draws.get(used).ok_or(OutputError::InsufficientDraws)?;
            used += 1;
            let right = u16::from(draw) < numerator;
            trace.push(row, right, numerator, logit, features.len())?;
            token = (token << 1) | u16::from(right);
            node = (node << 1) + 1 + usize::from(right);
        }
        trace.action = Action::Generate(token);
        trace.draws_used = u8::try_from(used).map_err(|_| OutputError::InvalidShape)?;
        Ok(trace)
    }
}

/// Offline selected-row Bernoulli-CE SGD with floating master coefficients.
/// This owns a quantized hard-forward model. It only updates rows on the
/// teacher action path; gradients from downstream state, writes and missed
/// memory candidates remain the integrated trainer's responsibility.
pub struct OutputTrainer {
    model: SparseOutput,
    master: Vec<f32>,
}

impl OutputTrainer {
    pub fn new(vocab: usize, feature_bank: usize, max_active: usize) -> Result<Self, OutputError> {
        Self::from_model(SparseOutput::new(vocab, feature_bank, max_active)?)
    }

    pub fn from_model(model: SparseOutput) -> Result<Self, OutputError> {
        model.validate()?;
        let master = model
            .coefficients
            .iter()
            .map(|code| f32::from(*code))
            .collect();
        Ok(Self { model, master })
    }

    pub fn model(&self) -> &SparseOutput {
        &self.model
    }

    pub fn export(self) -> SparseOutput {
        self.model
    }

    fn master_logit(&self, row: usize, features: &[u16]) -> Result<f32, OutputError> {
        let bias = self.model.coefficient_index(row, None)?;
        let mut logit = *self.master.get(bias).ok_or(OutputError::CorruptArtifact)?;
        for id in features {
            let index = self.model.coefficient_index(row, Some(*id))?;
            logit += *self.master.get(index).ok_or(OutputError::CorruptArtifact)?;
        }
        Ok(logit)
    }

    fn master_branch_ce(logit: f32, right: bool) -> (f64, f32) {
        // Match the hard table's saturated range; use a straight-through
        // derivative at the clamp so high-margin mistakes can recover.
        let bounded = logit.clamp(-16.0_f32, 16.0_f32);
        let probability = 1.0_f32 / (1.0_f32 + (-bounded / 4.0_f32).exp());
        let target = if right { 1.0_f32 } else { 0.0_f32 };
        let clipped = probability.clamp(1.0e-7_f32, 1.0_f32 - 1.0e-7_f32);
        let loss = if right {
            -f64::from(clipped).ln()
        } else {
            -f64::from(1.0_f32 - clipped).ln()
        };
        (loss, (probability - target) / 4.0_f32)
    }

    /// Soft-master action CE without an update. This is not marginal surface
    /// token CE when Copy and Generate have overlapping emissions.
    pub fn action_ce(
        &self,
        features: &[u16],
        copy_legal: bool,
        target: Action,
    ) -> Result<f64, OutputError> {
        let trace = self.model.score_action(features, copy_legal, target)?;
        let mut loss = 0.0;
        for step in trace.steps.iter().take(usize::from(trace.len)) {
            let logit = self.master_logit(usize::from(step.row), features)?;
            loss += Self::master_branch_ce(logit, step.right).0;
        }
        Ok(loss)
    }

    /// Actual hard quantized model CE, evaluated offline from selected dyadic
    /// branch probabilities. Use this to audit train-soft/hard-forward gaps.
    pub fn hard_action_ce(
        &self,
        features: &[u16],
        copy_legal: bool,
        target: Action,
    ) -> Result<f64, OutputError> {
        Ok(self
            .model
            .score_action(features, copy_legal, target)?
            .action_ce_offline())
    }

    pub fn train_action(
        &mut self,
        features: &[u16],
        copy_legal: bool,
        target: Action,
        learning_rate: f64,
    ) -> Result<f64, OutputError> {
        if !learning_rate.is_finite()
            || !(0.0..=10.0).contains(&learning_rate)
            || learning_rate == 0.0
        {
            return Err(OutputError::InvalidLearningRate);
        }
        let trace = self.model.score_action(features, copy_legal, target)?;
        let rate = learning_rate as f32;
        let mut loss = 0.0;
        for step in trace.steps.iter().take(usize::from(trace.len)) {
            let row = usize::from(step.row);
            let logit = self.master_logit(row, features)?;
            let (part_loss, gradient) = Self::master_branch_ce(logit, step.right);
            loss += part_loss;
            let bias_index = self.model.coefficient_index(row, None)?;
            self.update_one(bias_index, rate, gradient)?;
            for feature in features {
                let index = self.model.coefficient_index(row, Some(*feature))?;
                self.update_one(index, rate, gradient)?;
            }
        }
        Ok(loss)
    }

    /// Offline CE for the *surface token* in A1's one-token Copy model. When
    /// the selected source token equals `target`, Generate(target) and Copy
    /// are two action paths to the same observed token and the same future
    /// state. This optimizes their summed probability using responsibilities
    /// computed from the pre-update floating master. The common Continue row
    /// and Copy/Generate mode row each receive one combined gradient; selected
    /// vocabulary rows receive only the Generate responsibility. It is not a
    /// general span-copy marginalization and does not backpropagate through the
    /// source selector, write decision or earlier state transitions.
    pub fn train_token(
        &mut self,
        features: &[u16],
        selected: Option<u16>,
        target: u16,
        learning_rate: f64,
    ) -> Result<f64, OutputError> {
        if !learning_rate.is_finite()
            || !(0.0..=10.0).contains(&learning_rate)
            || learning_rate == 0.0
        {
            return Err(OutputError::InvalidLearningRate);
        }
        let copy_legal = selected.is_some();
        if selected != Some(target) {
            return self.train_action(
                features,
                copy_legal,
                Action::Generate(target),
                learning_rate,
            );
        }

        let generated = self
            .model
            .score_action(features, true, Action::Generate(target))?;
        let copied = self.model.score_action(features, true, Action::Copy)?;
        let generate_ce = self.master_trace_ce(&generated, features)?;
        let copy_ce = self.master_trace_ce(&copied, features)?;
        let minimum = generate_ce.min(copy_ce);
        let generate_weight = (minimum - generate_ce).exp();
        let copy_weight = (minimum - copy_ce).exp();
        let total_weight = generate_weight + copy_weight;
        let responsibility_generate = (generate_weight / total_weight) as f32;
        let responsibility_copy = (copy_weight / total_weight) as f32;
        let marginal_ce = minimum - total_weight.ln();

        // All row gradients are formed before the first master update. The
        // two actions share row 0 (Continue) and row 1 (Copy/Generate), so
        // adding two full action gradients would be incorrect.
        let mut row_gradients = [(0usize, 0.0_f32); MAX_PATH_STEPS];
        let mut gradient_count = 0usize;
        let common = generated.steps[0];
        let common_logit = self.master_logit(usize::from(common.row), features)?;
        row_gradients[gradient_count] = (
            usize::from(common.row),
            Self::master_branch_ce(common_logit, false).1,
        );
        gradient_count += 1;

        let gen_mode = generated.steps[1];
        let copy_mode = copied.steps[1];
        if gen_mode.row != copy_mode.row || gen_mode.right || !copy_mode.right || common.right {
            return Err(OutputError::CorruptArtifact);
        }
        let mode_logit = self.master_logit(usize::from(gen_mode.row), features)?;
        let mode_gradient = responsibility_generate * Self::master_branch_ce(mode_logit, false).1
            + responsibility_copy * Self::master_branch_ce(mode_logit, true).1;
        row_gradients[gradient_count] = (usize::from(gen_mode.row), mode_gradient);
        gradient_count += 1;

        for step in generated
            .steps
            .iter()
            .take(usize::from(generated.len))
            .skip(2)
        {
            let row = usize::from(step.row);
            let logit = self.master_logit(row, features)?;
            let gradient = responsibility_generate * Self::master_branch_ce(logit, step.right).1;
            let Some(slot) = row_gradients.get_mut(gradient_count) else {
                return Err(OutputError::CorruptArtifact);
            };
            *slot = (row, gradient);
            gradient_count += 1;
        }
        let rate = learning_rate as f32;
        for (row, gradient) in row_gradients.into_iter().take(gradient_count) {
            let bias_index = self.model.coefficient_index(row, None)?;
            self.update_one(bias_index, rate, gradient)?;
            for feature in features {
                let index = self.model.coefficient_index(row, Some(*feature))?;
                self.update_one(index, rate, gradient)?;
            }
        }
        Ok(marginal_ce)
    }

    fn master_trace_ce(&self, trace: &OutputTrace, features: &[u16]) -> Result<f64, OutputError> {
        let mut loss = 0.0;
        for step in trace.steps.iter().take(usize::from(trace.len)) {
            let logit = self.master_logit(usize::from(step.row), features)?;
            loss += Self::master_branch_ce(logit, step.right).0;
        }
        Ok(loss)
    }

    fn update_one(&mut self, index: usize, rate: f32, gradient: f32) -> Result<(), OutputError> {
        let master = self
            .master
            .get_mut(index)
            .ok_or(OutputError::CorruptArtifact)?;
        *master = (*master - rate * gradient).clamp(f32::from(MIN_CODE), f32::from(MAX_CODE));
        let quantized = master.round() as i8;
        let code = self
            .model
            .coefficients
            .get_mut(index)
            .ok_or(OutputError::CorruptArtifact)?;
        *code = quantized;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_mode_support_and_selected_row_accounting() {
        let output = SparseOutput::new(8, 16, 2).unwrap();
        let features = [3, 12];
        let generated = output
            .score_action(&features, true, Action::Generate(5))
            .unwrap();
        let copied = output.score_action(&features, true, Action::Copy).unwrap();
        let stopped = output.score_action(&features, true, Action::Stop).unwrap();
        assert_eq!(generated.len, 5);
        assert_eq!(generated.coefficient_reads, 15);
        assert_eq!(copied.len, 2);
        assert_eq!(stopped.len, 1);
        assert!(output.score_action(&features, false, Action::Copy).is_err());
        assert_eq!(
            output
                .score_action(&features, false, Action::Generate(5))
                .unwrap()
                .len,
            4
        );
        assert_eq!(generated.steps[2].row, 2);
    }

    #[test]
    fn every_token_path_has_positive_dyadic_probability() {
        let output = SparseOutput::new(16, 4, 1).unwrap();
        for token in 0..16 {
            let trace = output
                .score_action(&[2], true, Action::Generate(token))
                .unwrap();
            assert_eq!(trace.len, 6);
            assert!(trace
                .steps
                .iter()
                .take(6)
                .all(|step| (1..=255).contains(&step.right_numerator)));
            assert!(trace.action_ce_offline().is_finite());
        }
    }

    #[test]
    fn legal_actions_normalize_with_and_without_copy() {
        let mut output = SparseOutput::new(8, 4, 1).unwrap();
        output.coefficients[0] = 7;
        output.coefficients[output.row_offsets[MODE_COPY_ROW]] = -8;
        output.coefficients[output.row_offsets[VOCAB_ROW_BASE]] = 5;
        for copy_legal in [false, true] {
            let mut sum = (-output
                .score_action(&[2], copy_legal, Action::Stop)
                .unwrap()
                .action_ce_offline())
            .exp();
            if copy_legal {
                sum += (-output
                    .score_action(&[2], true, Action::Copy)
                    .unwrap()
                    .action_ce_offline())
                .exp();
            }
            for token in 0..8 {
                sum += (-output
                    .score_action(&[2], copy_legal, Action::Generate(token))
                    .unwrap()
                    .action_ce_offline())
                .exp();
            }
            assert!((sum - 1.0).abs() < 1.0e-12, "sum={sum}");
        }
    }

    #[test]
    fn trainer_keeps_hard_artifact_valid() {
        let mut trainer = OutputTrainer::new(8, 16, 2).unwrap();
        for _ in 0..20 {
            trainer
                .train_action(&[1, 5], false, Action::Generate(7), 2.0)
                .unwrap();
        }
        let model = trainer.export();
        model.validate().unwrap();
        let trace = model
            .score_action(&[1, 5], false, Action::Generate(7))
            .unwrap();
        assert!(trace.action_ce_offline() < 4.0);
    }

    #[test]
    fn marginal_copy_gradient_counts_shared_rows_once() {
        let mut trainer = OutputTrainer::new(8, 16, 1).unwrap();
        let loss = trainer.train_token(&[3], Some(5), 5, 1.0).unwrap();
        assert!((loss + (9.0_f64 / 32.0_f64).ln()).abs() < 1.0e-6);
        let continue_bias = trainer.model.row_offsets[MODE_STOP_ROW];
        assert!((trainer.master[continue_bias] + 0.125).abs() < 1.0e-6);
        let copy_mode_bias = trainer.model.row_offsets[MODE_COPY_ROW];
        let expected_copy_gradient = (0.5_f32 - 8.0_f32 / 9.0_f32) / 4.0_f32;
        assert!((trainer.master[copy_mode_bias] + expected_copy_gradient).abs() < 1.0e-6);
    }
}
