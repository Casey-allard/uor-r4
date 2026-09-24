//! Selected-row contextual finite-code encoder for the integrated model.
//!
//! A lane scores 120 actions by adding five already-selected rows: the exact
//! observed token, its previous finite state, its next neighbor's state, an
//! optional selected evidence token (or a dedicated None row), and a typed
//! State/Key/Query row. Every row is padded to 128 coefficients, but the
//! serving path reads only its first 120 signed-four-bit values. Row and
//! coefficient addresses use shifts/table indexing. No token ID is reduced
//! modulo 120: each token has an independent learned row. The caller must
//! supply only causally observed/emitted tokens, never the target being scored.
//! State outputs are actions to compose with prior state through the artifact-
//! bound algebra; Key and Query outputs are direct product-code lanes.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const ENCODER_VERSION: u8 = 1;
pub const GROUP_ORDER: usize = 120;
pub const ROW_STRIDE: usize = 128;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_LANES: usize = 16;
const MIN_CODE: i8 = -8;
const MAX_CODE: i8 = 7;
const TRAIN_TEMPERATURE: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncoderKind {
    State,
    Key,
    Query,
}

impl EncoderKind {
    fn index(self) -> usize {
        match self {
            Self::State => 0,
            Self::Key => 1,
            Self::Query => 2,
        }
    }
}

/// `token` is the last already-observed/emitted token in this causal step.
/// Only `previous[..lanes]` is active. Evidence must be an already selected
/// owned token; `None` selects a dedicated, initially zero row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncoderInput {
    pub token: u16,
    pub previous: [u8; MAX_LANES],
    pub evidence: Option<u16>,
    pub kind: EncoderKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderError {
    InvalidShape,
    InvalidInput,
    InvalidCode,
    InvalidCoefficient,
    InvalidLearningRate,
    InvalidLoss,
    CorruptArtifact,
}

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EncoderError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneScores {
    pub scores: [i16; GROUP_ORDER],
    pub best: u8,
    pub runner_up: u8,
    pub best_score: i16,
    pub margin: i16,
    /// Exactly five selected rows × 120 coefficient slots.
    pub coefficient_reads: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncoderOutput {
    /// Active output code/action in `codes[..lanes]`.
    pub codes: [u8; MAX_LANES],
    pub runner_up: [u8; MAX_LANES],
    pub top_scores: [i16; MAX_LANES],
    pub margins: [i16; MAX_LANES],
    pub lanes: u8,
    /// Counts logical i8 learned coefficient reads; bytes equal slots here.
    pub coefficient_reads: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Layout {
    lane_shift: u8,
    previous_base: usize,
    neighbor_base: usize,
    evidence_base: usize,
    kind_base: usize,
    row_count: usize,
}

impl Layout {
    fn new(vocab: usize, lanes: usize) -> Result<Self, EncoderError> {
        let lane_shift =
            u8::try_from(lanes.trailing_zeros()).map_err(|_| EncoderError::InvalidShape)?;
        let previous_base = vocab
            .checked_shl(u32::from(lane_shift))
            .ok_or(EncoderError::InvalidShape)?;
        let neighbor_base = previous_base
            .checked_add(GROUP_ORDER << lane_shift)
            .ok_or(EncoderError::InvalidShape)?;
        let evidence_base = neighbor_base
            .checked_add(GROUP_ORDER << lane_shift)
            .ok_or(EncoderError::InvalidShape)?;
        let kind_base = evidence_base
            .checked_add((vocab + 1) << lane_shift)
            .ok_or(EncoderError::InvalidShape)?;
        let row_count = kind_base
            .checked_add(3usize << lane_shift)
            .ok_or(EncoderError::InvalidShape)?;
        Ok(Self {
            lane_shift,
            previous_base,
            neighbor_base,
            evidence_base,
            kind_base,
            row_count,
        })
    }

    fn row(self, base: usize, value: usize, lane: usize) -> usize {
        base + (value << self.lane_shift) + lane
    }
}

/// Four-bit-valued coefficients stored as unpacked i8 bytes. Only five 120-way
/// rows per lane are read per encode. Call `validate()` after deserialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeEncoder {
    pub version: u8,
    pub vocab: u16,
    pub lanes: u8,
    pub seed: u64,
    layout: Layout,
    coefficients: Vec<i8>,
}

impl CodeEncoder {
    pub fn new(vocab: usize, lanes: usize, seed: u64) -> Result<Self, EncoderError> {
        if !(2..=MAX_VOCAB).contains(&vocab)
            || !vocab.is_power_of_two()
            || !matches!(lanes, 1 | 2 | 4)
        {
            return Err(EncoderError::InvalidShape);
        }
        let layout = Layout::new(vocab, lanes)?;
        let coefficient_count = layout
            .row_count
            .checked_shl(7)
            .ok_or(EncoderError::InvalidShape)?;
        let mut model = Self {
            version: ENCODER_VERSION,
            vocab: u16::try_from(vocab).map_err(|_| EncoderError::InvalidShape)?,
            lanes: u8::try_from(lanes).map_err(|_| EncoderError::InvalidShape)?,
            seed,
            layout,
            coefficients: vec![0; coefficient_count],
        };
        model.initialize(seed)?;
        model.validate()?;
        Ok(model)
    }

    pub fn stored_coefficient_bytes(&self) -> usize {
        self.coefficients.len()
    }

    pub fn stored_coefficient_slots(&self) -> usize {
        self.coefficients.len()
    }

    pub fn coefficients(&self) -> &[i8] {
        &self.coefficients
    }

    pub fn validate(&self) -> Result<(), EncoderError> {
        let vocab = usize::from(self.vocab);
        let lanes = usize::from(self.lanes);
        if self.version != ENCODER_VERSION
            || !(2..=MAX_VOCAB).contains(&vocab)
            || !vocab.is_power_of_two()
            || !matches!(lanes, 1 | 2 | 4)
            || self.layout != Layout::new(vocab, lanes)?
            || self.coefficients.len() != self.layout.row_count << 7
        {
            return Err(EncoderError::InvalidShape);
        }
        for row in 0..self.layout.row_count {
            let base = row << 7;
            for coefficient in &self.coefficients[base..base + GROUP_ORDER] {
                if !(MIN_CODE..=MAX_CODE).contains(coefficient) {
                    return Err(EncoderError::InvalidCoefficient);
                }
            }
            if self.coefficients[base + GROUP_ORDER..base + ROW_STRIDE]
                .iter()
                .any(|coefficient| *coefficient != 0)
            {
                return Err(EncoderError::InvalidCoefficient);
            }
        }
        Ok(())
    }

    fn initialize(&mut self, seed: u64) -> Result<(), EncoderError> {
        let lanes = usize::from(self.lanes);
        let vocab = usize::from(self.vocab);
        for row in 0..self.layout.row_count {
            let base = row << 7;
            for action in 0..GROUP_ORDER {
                let salt = seed ^ ((row as u64) << 7) ^ (action as u64);
                let mixed = splitmix64(salt);
                // Small seeded context variation. Token rows below receive
                // independent larger patterns; this is artifact construction.
                let code = i8::try_from(mixed % 3).map_err(|_| EncoderError::InvalidShape)? - 1;
                self.coefficients[base + action] = code;
            }
        }
        // `None` evidence is a declared row rather than skipping a row. Start
        // it at zero so initial no-read output is not arbitrary evidence.
        for lane in 0..lanes {
            let row = self.layout.row(self.layout.evidence_base, vocab, lane);
            let base = row << 7;
            self.coefficients[base..base + GROUP_ORDER].fill(0);
        }
        // The same frame/content initially maps to compatible Key and Query
        // codes. Training can later differentiate their typed rows.
        for lane in 0..lanes {
            let key_row = self
                .layout
                .row(self.layout.kind_base, EncoderKind::Key.index(), lane);
            let query_row =
                self.layout
                    .row(self.layout.kind_base, EncoderKind::Query.index(), lane);
            let key_base = key_row << 7;
            let query_base = query_row << 7;
            let key = self.coefficients[key_base..key_base + ROW_STRIDE].to_vec();
            self.coefficients[query_base..query_base + ROW_STRIDE].copy_from_slice(&key);
        }
        // Token rows are per exact ID, never token % 120. Two anchor IDs are
        // forced apart so a zero-initialized context cannot collapse all token
        // codes; all other token rows retain seeded independent patterns.
        for lane in 0..lanes {
            let first = usize::try_from(splitmix64(seed ^ (lane as u64)) % GROUP_ORDER as u64)
                .map_err(|_| EncoderError::InvalidShape)?;
            let second = (first + 1) % GROUP_ORDER;
            for (token, favored) in [(0usize, first), (1usize, second)] {
                let row = self.layout.row(0, token, lane);
                let base = row << 7;
                self.coefficients[base..base + GROUP_ORDER].fill(-8);
                self.coefficients[base + favored] = 7;
            }
        }
        Ok(())
    }

    fn check_input(&self, input: EncoderInput) -> Result<(), EncoderError> {
        if input.token >= self.vocab || input.evidence.is_some_and(|token| token >= self.vocab) {
            return Err(EncoderError::InvalidInput);
        }
        for code in input.previous.iter().take(usize::from(self.lanes)) {
            if usize::from(*code) >= GROUP_ORDER {
                return Err(EncoderError::InvalidCode);
            }
        }
        Ok(())
    }

    fn selected_rows(&self, input: EncoderInput, lane: usize) -> Result<[usize; 5], EncoderError> {
        let lanes = usize::from(self.lanes);
        if lane >= lanes {
            return Err(EncoderError::InvalidInput);
        }
        let neighbor = (lane + 1) & (lanes - 1);
        let evidence = input.evidence.map_or(usize::from(self.vocab), usize::from);
        Ok([
            self.layout.row(0, usize::from(input.token), lane),
            self.layout.row(
                self.layout.previous_base,
                usize::from(input.previous[lane]),
                lane,
            ),
            self.layout.row(
                self.layout.neighbor_base,
                usize::from(input.previous[neighbor]),
                lane,
            ),
            self.layout.row(self.layout.evidence_base, evidence, lane),
            self.layout
                .row(self.layout.kind_base, input.kind.index(), lane),
        ])
    }

    pub fn score_lane(&self, input: EncoderInput, lane: usize) -> Result<LaneScores, EncoderError> {
        self.check_input(input)?;
        let rows = self.selected_rows(input, lane)?;
        let mut scores = [0i16; GROUP_ORDER];
        for row in rows {
            let base = row << 7;
            for (action, score) in scores.iter_mut().enumerate() {
                let code = *self
                    .coefficients
                    .get(base + action)
                    .ok_or(EncoderError::CorruptArtifact)?;
                if !(MIN_CODE..=MAX_CODE).contains(&code) {
                    return Err(EncoderError::InvalidCoefficient);
                }
                *score += i16::from(code);
            }
        }
        Ok(rank_scores(scores))
    }

    pub fn encode(&self, input: EncoderInput) -> Result<EncoderOutput, EncoderError> {
        self.check_input(input)?;
        let mut output = EncoderOutput {
            codes: [0; MAX_LANES],
            runner_up: [0; MAX_LANES],
            top_scores: [0; MAX_LANES],
            margins: [0; MAX_LANES],
            lanes: self.lanes,
            coefficient_reads: 0,
        };
        for lane in 0..usize::from(self.lanes) {
            let scored = self.score_lane(input, lane)?;
            output.codes[lane] = scored.best;
            output.runner_up[lane] = scored.runner_up;
            output.top_scores[lane] = scored.best_score;
            output.margins[lane] = scored.margin;
            output.coefficient_reads = output
                .coefficient_reads
                .checked_add(scored.coefficient_reads)
                .ok_or(EncoderError::InvalidShape)?;
        }
        Ok(output)
    }
}

fn rank_scores(scores: [i16; GROUP_ORDER]) -> LaneScores {
    let mut best = 0usize;
    let mut runner_up = 1usize;
    if scores[runner_up] > scores[best] {
        std::mem::swap(&mut best, &mut runner_up);
    }
    for action in 2..GROUP_ORDER {
        if scores[action] > scores[best] {
            runner_up = best;
            best = action;
        } else if scores[action] > scores[runner_up] {
            runner_up = action;
        }
    }
    LaneScores {
        best: best as u8,
        runner_up: runner_up as u8,
        best_score: scores[best],
        margin: scores[best] - scores[runner_up],
        scores,
        coefficient_reads: 5 * GROUP_ORDER as u32,
    }
}

/// SplitMix64 is used only for offline deterministic artifact construction.
fn splitmix64(value: u64) -> u64 {
    let mut z = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Offline selected-row SGD. It supplies local code supervision and a one-step
/// expected-loss surrogate; it does not perform sequence BPTT through hard
/// decisions, memory indexing or the language head. The integrated trainer
/// must evaluate the hard loaded code path and supply source/language losses.
pub struct EncoderTrainer {
    model: CodeEncoder,
    master: Vec<f32>,
}

impl EncoderTrainer {
    pub fn new(vocab: usize, lanes: usize, seed: u64) -> Result<Self, EncoderError> {
        Self::from_model(CodeEncoder::new(vocab, lanes, seed)?)
    }

    pub fn from_model(model: CodeEncoder) -> Result<Self, EncoderError> {
        model.validate()?;
        let master = model
            .coefficients
            .iter()
            .map(|code| f32::from(*code))
            .collect();
        Ok(Self { model, master })
    }

    pub fn model(&self) -> &CodeEncoder {
        &self.model
    }

    pub fn export(self) -> CodeEncoder {
        self.model
    }

    fn distribution(
        &self,
        input: EncoderInput,
        lane: usize,
    ) -> Result<([f64; GROUP_ORDER], [usize; 5]), EncoderError> {
        self.model.check_input(input)?;
        let rows = self.model.selected_rows(input, lane)?;
        let mut scores = [0.0_f64; GROUP_ORDER];
        for row in rows {
            let base = row << 7;
            for (action, score) in scores.iter_mut().enumerate() {
                let master = *self
                    .master
                    .get(base + action)
                    .ok_or(EncoderError::CorruptArtifact)?;
                *score += f64::from(master);
            }
        }
        let maximum = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut total = 0.0_f64;
        for score in &mut scores {
            *score = ((*score - maximum) / TRAIN_TEMPERATURE).exp();
            total += *score;
        }
        if !total.is_finite() || total <= 0.0 {
            return Err(EncoderError::InvalidLoss);
        }
        for score in &mut scores {
            *score /= total;
        }
        Ok((scores, rows))
    }

    fn check_learning_rate(rate: f64) -> Result<f32, EncoderError> {
        if !rate.is_finite() || rate <= 0.0 || rate > 10.0 {
            return Err(EncoderError::InvalidLearningRate);
        }
        Ok(rate as f32)
    }

    fn update_rows(
        &mut self,
        rows: [usize; 5],
        gradients: &[f64; GROUP_ORDER],
        rate: f32,
    ) -> Result<(), EncoderError> {
        for row in rows {
            let base = row << 7;
            for (action, gradient) in gradients.iter().enumerate() {
                let index = base + action;
                let master = self
                    .master
                    .get_mut(index)
                    .ok_or(EncoderError::CorruptArtifact)?;
                let step = rate * (*gradient as f32);
                if !step.is_finite() {
                    return Err(EncoderError::InvalidLoss);
                }
                *master = (*master - step).clamp(f32::from(MIN_CODE), f32::from(MAX_CODE));
                let quantized = master.round() as i8;
                let code = self
                    .model
                    .coefficients
                    .get_mut(index)
                    .ok_or(EncoderError::CorruptArtifact)?;
                *code = quantized;
            }
        }
        Ok(())
    }

    /// Teacher-code CE summed across active lanes. Desired codes are training
    /// annotations only and must never be supplied as serving inputs.
    pub fn supervise(
        &mut self,
        input: EncoderInput,
        desired_codes: &[u8],
        learning_rate: f64,
    ) -> Result<f64, EncoderError> {
        let rate = Self::check_learning_rate(learning_rate)?;
        if desired_codes.len() != usize::from(self.model.lanes)
            || desired_codes
                .iter()
                .any(|code| usize::from(*code) >= GROUP_ORDER)
        {
            return Err(EncoderError::InvalidCode);
        }
        let mut total_loss = 0.0_f64;
        for (lane, desired) in desired_codes.iter().enumerate() {
            let (probabilities, rows) = self.distribution(input, lane)?;
            let target = usize::from(*desired);
            total_loss -= probabilities[target].ln();
            let mut gradients = [0.0_f64; GROUP_ORDER];
            for (action, gradient) in gradients.iter_mut().enumerate() {
                *gradient = (probabilities[action] - f64::from(u8::from(action == target)))
                    / TRAIN_TEMPERATURE;
            }
            self.update_rows(rows, &gradients, rate)?;
        }
        Ok(total_loss)
    }

    /// One-step expected-loss gradient for a single lane. `losses[action]`
    /// must be a training-only downstream counterfactual loss evaluated from
    /// the same causal prefix. This updates selected rows but is not BPTT.
    pub fn train_lane_losses(
        &mut self,
        input: EncoderInput,
        lane: usize,
        losses: &[f64; GROUP_ORDER],
        learning_rate: f64,
    ) -> Result<f64, EncoderError> {
        let rate = Self::check_learning_rate(learning_rate)?;
        if losses
            .iter()
            .any(|loss| !loss.is_finite() || loss.abs() > 1.0e6)
        {
            return Err(EncoderError::InvalidLoss);
        }
        let (probabilities, rows) = self.distribution(input, lane)?;
        let expected = probabilities
            .iter()
            .zip(losses.iter())
            .map(|(probability, loss)| probability * loss)
            .sum::<f64>();
        if !expected.is_finite() {
            return Err(EncoderError::InvalidLoss);
        }
        let mut gradients = [0.0_f64; GROUP_ORDER];
        for (action, gradient) in gradients.iter_mut().enumerate() {
            *gradient = probabilities[action] * (losses[action] - expected) / TRAIN_TEMPERATURE;
        }
        self.update_rows(rows, &gradients, rate)?;
        Ok(expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(token: u16, kind: EncoderKind) -> EncoderInput {
        EncoderInput {
            token,
            previous: [0; MAX_LANES],
            evidence: None,
            kind,
        }
    }

    #[test]
    fn deterministic_noncollapsed_and_bounded_reads() {
        let model = CodeEncoder::new(16, 4, 19).unwrap();
        let first = model.encode(input(0, EncoderKind::Key)).unwrap();
        let second = model.encode(input(1, EncoderKind::Key)).unwrap();
        assert_ne!(first.codes[..4], second.codes[..4]);
        assert_eq!(first.coefficient_reads, 4 * 5 * 120);
        assert_eq!(first, model.encode(input(0, EncoderKind::Key)).unwrap());
        assert_eq!(
            first.codes[..4],
            model.encode(input(0, EncoderKind::Query)).unwrap().codes[..4]
        );
    }

    #[test]
    fn selected_supervision_updates_hard_artifact() {
        let mut trainer = EncoderTrainer::new(16, 2, 7).unwrap();
        let context = input(5, EncoderKind::Query);
        for _ in 0..24 {
            trainer.supervise(context, &[11, 23], 3.0).unwrap();
        }
        let model = trainer.export();
        model.validate().unwrap();
        let result = model.encode(context).unwrap();
        assert_eq!(result.codes[..2], [11, 23]);
    }

    #[test]
    fn one_step_loss_surrogate_is_finite() {
        let mut trainer = EncoderTrainer::new(16, 1, 3).unwrap();
        let mut losses = [2.0; GROUP_ORDER];
        losses[17] = 0.0;
        let expected = trainer
            .train_lane_losses(input(9, EncoderKind::State), 0, &losses, 1.0)
            .unwrap();
        assert!(expected.is_finite());
        trainer.model().validate().unwrap();
    }
}
