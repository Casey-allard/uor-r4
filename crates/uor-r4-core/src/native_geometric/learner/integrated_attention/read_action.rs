//! One bounded integer decision over admitted records and `NoRead`.
//!
//! A read score adds selected causal feature rows, signed relative-code factors,
//! and exact candidate-value/relative-code interaction rows. `NoRead` has its own
//! selected feature rows. All coefficients are signed four bit; higher scores
//! win, with `NoRead` winning ties. The value interaction is a learned table over
//! an observed token and a relative code, not an ordinal distance or a hash of
//! record identity. Both finite-algebra arms use the same shape and access bound.
//!
//! The offline learner changes these integer coefficients directly. Its bounded
//! perceptron corrections are local to an example; they make no claim of global
//! objective descent or preservation of previously fitted examples. Serving uses
//! selected table reads, additions, comparisons and shifts, with no allocation,
//! floating point, multiplication or division in either scoring method.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use super::geometry::{EnergyReadCounts, EnergyTables, GeometryError, LanePair};

const FORMAT_VERSION: u16 = 1;
const MAX_VOCABULARY: usize = 4096;
const MAX_LANES: usize = 32;
const MAX_ACTIVE_FEATURES: usize = 64;
const MAX_CANDIDATES: usize = 64;
const MAX_ACTIONS: usize = MAX_CANDIDATES + 1;
const MAX_TRAIN_STEPS: u8 = 32;
const MAX_MARGIN: i32 = 256;
const GROUP_ORDER: u8 = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadActionError {
    Geometry(GeometryError),
    UnsupportedVersion(u16),
    InvalidVocabulary(usize),
    InvalidLaneCount(usize),
    InvalidShape(&'static str),
    InvalidCoefficient(i8),
    InvalidFeature(u16),
    TooManyActiveFeatures(usize),
    MissingCandidateValue,
    MultipleCandidateValues,
    CandidateValueInNoRead,
    NoReadValueInRead,
    InvalidRelativeWidth { expected: usize, actual: usize },
    InvalidRelative(u8),
    TooManyCandidates(usize),
    InvalidAcceptableCount(usize),
    InvalidTarget(usize),
    InvalidMargin(i32),
    InvalidTrainSteps(u8),
    ScoreOverflow,
    CounterOverflow,
}

impl fmt::Display for ReadActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry(error) => write!(f, "read-action geometry: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported read-action version {version}")
            }
            Self::InvalidVocabulary(n) => {
                write!(
                    f,
                    "read-action vocabulary {n} must be a power of two in 8..=4096"
                )
            }
            Self::InvalidLaneCount(n) => write!(
                f,
                "read-action lane count {n} must be a power of two in 1..={MAX_LANES}"
            ),
            Self::InvalidShape(field) => write!(f, "invalid read-action shape: {field}"),
            Self::InvalidCoefficient(weight) => {
                write!(f, "read-action coefficient {weight} outside -8..=7")
            }
            Self::InvalidFeature(feature) => write!(f, "read-action feature {feature} unavailable"),
            Self::TooManyActiveFeatures(n) => {
                write!(
                    f,
                    "{n} active read-action features exceed {MAX_ACTIVE_FEATURES}"
                )
            }
            Self::MissingCandidateValue => write!(f, "read action has no candidate-value feature"),
            Self::MultipleCandidateValues => {
                write!(f, "read action has multiple candidate-value features")
            }
            Self::CandidateValueInNoRead => write!(f, "NoRead contains a candidate-value feature"),
            Self::NoReadValueInRead => write!(f, "read action contains the NoRead sentinel"),
            Self::InvalidRelativeWidth { expected, actual } => {
                write!(
                    f,
                    "read-action relative width {actual}, expected {expected}"
                )
            }
            Self::InvalidRelative(relative) => {
                write!(f, "read-action relative ID {relative} outside 0..120")
            }
            Self::TooManyCandidates(n) => {
                write!(f, "{n} read candidates exceed {MAX_CANDIDATES}")
            }
            Self::InvalidAcceptableCount(n) => {
                write!(f, "{n} acceptable read actions outside 1..={MAX_ACTIONS}")
            }
            Self::InvalidTarget(index) => write!(f, "read-action target {index} unavailable"),
            Self::InvalidMargin(margin) => {
                write!(f, "read-action margin {margin} outside 1..={MAX_MARGIN}")
            }
            Self::InvalidTrainSteps(steps) => {
                write!(f, "read-action steps {steps} outside 1..={MAX_TRAIN_STEPS}")
            }
            Self::ScoreOverflow => write!(f, "read-action score overflow"),
            Self::CounterOverflow => write!(f, "read-action counter overflow"),
        }
    }
}

impl std::error::Error for ReadActionError {}

impl From<GeometryError> for ReadActionError {
    fn from(value: GeometryError) -> Self {
        Self::Geometry(value)
    }
}

/// Actual selected coefficient/byte reads, including repeated feature IDs.
/// Packed relative factors load one backing byte for each selected coefficient.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionReadCounts {
    pub coefficients: u64,
    pub parameter_bytes: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionScoreComponents {
    /// Includes the learned read bias.
    pub read_feature_score: i32,
    pub relative_score: i32,
    pub value_relative_score: i32,
    pub total: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionBankStats {
    pub nonzero: u64,
    pub positive: u64,
    pub negative: u64,
    pub l1: u64,
}

impl ReadActionBankStats {
    fn record(&mut self, weight: i8) {
        if weight > 0 {
            self.positive += 1;
        } else if weight < 0 {
            self.negative += 1;
        }
        if weight != 0 {
            self.nonzero += 1;
        }
        self.l1 += u64::from(weight.unsigned_abs());
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionParameterStats {
    pub read_bias: i8,
    pub no_read_bias: i8,
    pub read_features: ReadActionBankStats,
    pub no_read_features: ReadActionBankStats,
    pub relative_unary: ReadActionBankStats,
    pub relative_pair: ReadActionBankStats,
    pub value_relative: ReadActionBankStats,
}

impl ReadActionReadCounts {
    fn record(&mut self, coefficients: u64, bytes: u64) -> Result<(), ReadActionError> {
        let next_coefficients = self
            .coefficients
            .checked_add(coefficients)
            .ok_or(ReadActionError::CounterOverflow)?;
        let next_bytes = self
            .parameter_bytes
            .checked_add(bytes)
            .ok_or(ReadActionError::CounterOverflow)?;
        self.coefficients = next_coefficients;
        self.parameter_bytes = next_bytes;
        Ok(())
    }
}

/// One actually admitted record. Features contain its observed candidate-value
/// token using the containing model's existing exact feature-bank layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionCandidate {
    pub features: Vec<u16>,
    pub relative: Vec<u8>,
}

/// Offline labels never enter the scoring methods. `None` means that NoRead is
/// the training target; `Some(i)` addresses only the supplied admitted list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionExample {
    pub no_read_features: Vec<u16>,
    pub candidates: Vec<ReadActionCandidate>,
    pub target: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionTrainConfig {
    pub margin: i32,
    pub max_steps: u8,
}

impl Default for ReadActionTrainConfig {
    fn default() -> Self {
        Self {
            margin: 2,
            max_steps: 4,
        }
    }
}

impl ReadActionTrainConfig {
    fn validate(self) -> Result<(), ReadActionError> {
        if !(1..=MAX_MARGIN).contains(&self.margin) {
            return Err(ReadActionError::InvalidMargin(self.margin));
        }
        if self.max_steps == 0 || self.max_steps > MAX_TRAIN_STEPS {
            return Err(ReadActionError::InvalidTrainSteps(self.max_steps));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadActionTrainStop {
    MarginSatisfied,
    NoRival,
    NoAvailableEdit,
    StepLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionTrainReport {
    pub selected_before: Option<usize>,
    pub selected_after: Option<usize>,
    /// Best acceptable score minus the strongest excluded action; absent only
    /// when every available action is acceptable.
    pub margin_before: Option<i32>,
    pub margin_after: Option<i32>,
    pub steps: u8,
    /// Number of actual signed unit coefficient changes, including repeats.
    pub coefficient_edits: u64,
    /// Sum of absolute net changes from the example's starting coefficients.
    pub integer_l1_delta: u64,
    /// Number of coordinates whose final coefficient differs from its start.
    pub changed_coefficients: u64,
    pub saturated_coordinates: u64,
    pub stop: ReadActionTrainStop,
}

/// All rows are signed four bit. Feature and value-relative coefficients use
/// one `i8` byte each; only `relative_factors` use packed nibbles. The padded
/// value table has `vocabulary * lanes * 128` bytes (2 MiB for V4096/L4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadActionModel {
    format_version: u16,
    vocabulary: usize,
    lanes: u8,
    lane_shift: u8,
    read_bias: i8,
    no_read_bias: i8,
    read_rows: Vec<i8>,
    no_read_rows: Vec<i8>,
    relative_factors: EnergyTables,
    value_relative_rows: Vec<i8>,
}

impl ReadActionModel {
    /// Zero initialization abstains until a read acquires a strictly larger
    /// learned score. No record identity or future target is stored here.
    pub fn new(
        vocabulary: usize,
        lanes: u8,
        edges: Vec<LanePair>,
    ) -> Result<Self, ReadActionError> {
        check_dimensions(vocabulary, lanes)?;
        let lane_shift = lanes.trailing_zeros() as u8;
        let feature_count = (vocabulary << 1) + (usize::from(lanes) << 7) + 1;
        let result = Self {
            format_version: FORMAT_VERSION,
            vocabulary,
            lanes,
            lane_shift,
            read_bias: 0,
            no_read_bias: 0,
            read_rows: vec![0; feature_count],
            no_read_rows: vec![0; feature_count],
            relative_factors: EnergyTables::zeroed(lanes, edges)?,
            value_relative_rows: vec![0; (vocabulary << lane_shift) << 7],
        };
        result.validate()?;
        Ok(result)
    }

    pub const fn vocabulary(&self) -> usize {
        self.vocabulary
    }

    pub const fn lanes(&self) -> u8 {
        self.lanes
    }

    pub fn feature_count(&self) -> usize {
        (self.vocabulary << 1) + (usize::from(self.lanes) << 7) + 1
    }

    pub fn parameter_bytes(&self) -> usize {
        2 + self.read_rows.len()
            + self.no_read_rows.len()
            + self.relative_factors.packed_bytes()
            + self.value_relative_rows.len()
    }

    /// Meaningful coefficients, excluding padded relative IDs 120..127.
    pub fn coefficient_slots(&self) -> usize {
        let value_lanes = self.vocabulary << self.lane_shift;
        2 + self.read_rows.len()
            + self.no_read_rows.len()
            + self.relative_factors.coefficient_slots()
            + (value_lanes << 7)
            - (value_lanes << 3)
    }

    /// Offline inspection of actual exported integer coefficients, excluding
    /// padded IDs in relative factors. This is an all-row diagnostic, not a
    /// serving method or a geometric-advantage measure.
    pub fn parameter_stats(&self) -> Result<ReadActionParameterStats, ReadActionError> {
        self.validate()?;
        let mut result = ReadActionParameterStats {
            read_bias: self.read_bias,
            no_read_bias: self.no_read_bias,
            ..ReadActionParameterStats::default()
        };
        for &weight in &self.read_rows {
            result.read_features.record(weight);
        }
        for &weight in &self.no_read_rows {
            result.no_read_features.record(weight);
        }
        for lane in 0..self.lanes {
            for relative in 0..GROUP_ORDER {
                result
                    .relative_unary
                    .record(self.relative_factors.get_unary(lane, relative)?);
            }
        }
        for edge in 0..self.relative_factors.edges().len() {
            for left in 0..GROUP_ORDER {
                for right in 0..GROUP_ORDER {
                    result
                        .relative_pair
                        .record(self.relative_factors.get_pair(edge, left, right)?);
                }
            }
        }
        for &weight in &self.value_relative_rows {
            result.value_relative.record(weight);
        }
        Ok(result)
    }

    /// Full artifact validation belongs at load/export, not the hot score path.
    pub fn validate(&self) -> Result<(), ReadActionError> {
        self.validate_shape()?;
        check_weight(self.read_bias)?;
        check_weight(self.no_read_bias)?;
        for &weight in self.read_rows.iter().chain(&self.no_read_rows) {
            check_weight(weight)?;
        }
        for (index, &weight) in self.value_relative_rows.iter().enumerate() {
            check_weight(weight)?;
            if index & 127 >= usize::from(GROUP_ORDER) && weight != 0 {
                return Err(ReadActionError::InvalidShape(
                    "nonzero value-relative padding",
                ));
            }
        }
        self.relative_factors.validate()?;
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), ReadActionError> {
        if self.format_version != FORMAT_VERSION {
            return Err(ReadActionError::UnsupportedVersion(self.format_version));
        }
        check_dimensions(self.vocabulary, self.lanes)?;
        if u32::from(self.lane_shift) != self.lanes.trailing_zeros() {
            return Err(ReadActionError::InvalidShape("lane shift"));
        }
        if self.read_rows.len() != self.feature_count()
            || self.no_read_rows.len() != self.feature_count()
        {
            return Err(ReadActionError::InvalidShape("feature rows"));
        }
        if self.value_relative_rows.len() != (self.vocabulary << self.lane_shift) << 7 {
            return Err(ReadActionError::InvalidShape("value-relative rows"));
        }
        if self.relative_factors.lanes() != self.lanes {
            return Err(ReadActionError::InvalidShape("relative factor lanes"));
        }
        Ok(())
    }

    /// Selected integer reads only. The existing action feature identifies the
    /// candidate token; exactly one such feature is required for a read.
    pub fn score_read(
        &self,
        features: &[u16],
        relatives: &[u8],
        reads: &mut ReadActionReadCounts,
    ) -> Result<i32, ReadActionError> {
        Ok(self
            .score_read_components(features, relatives, reads)?
            .total)
    }

    /// The same served computation, with its additive components exposed for
    /// diagnostics. The supplied counters count exactly one scoring call.
    pub fn score_read_components(
        &self,
        features: &[u16],
        relatives: &[u8],
        reads: &mut ReadActionReadCounts,
    ) -> Result<ReadActionScoreComponents, ReadActionError> {
        self.validate_shape()?;
        self.check_relatives(relatives)?;
        let token = self
            .check_features(features, true)?
            .ok_or(ReadActionError::MissingCandidateValue)?;
        check_weight(self.read_bias)?;
        let mut feature_score = i32::from(self.read_bias);
        for &feature in features {
            let weight = self.read_rows[usize::from(feature)];
            check_weight(weight)?;
            feature_score = add_score(feature_score, weight)?;
        }
        let mut relative_reads = EnergyReadCounts::default();
        let relative_score = self
            .relative_factors
            .score(relatives, &mut relative_reads)?;
        let mut value_relative_score = 0;
        for (lane, &relative) in relatives.iter().enumerate() {
            let index = self.value_index(token, lane as u8, relative);
            let weight = self.value_relative_rows[index];
            check_weight(weight)?;
            value_relative_score = add_score(value_relative_score, weight)?;
        }
        let selected = 1 + features.len() as u64 + u64::from(self.lanes);
        reads.record(
            selected + relative_reads.coefficients,
            selected + relative_reads.packed_bytes,
        )?;
        let total = feature_score
            .checked_add(relative_score)
            .and_then(|value| value.checked_add(value_relative_score))
            .ok_or(ReadActionError::ScoreOverflow)?;
        Ok(ReadActionScoreComponents {
            read_feature_score: feature_score,
            relative_score,
            value_relative_score,
            total,
        })
    }

    /// Query-only rows may include the existing NoRead sentinel. A candidate
    /// value is rejected so it cannot leak into the NoRead alternative.
    pub fn score_none(
        &self,
        query_features: &[u16],
        reads: &mut ReadActionReadCounts,
    ) -> Result<i32, ReadActionError> {
        self.validate_shape()?;
        self.check_features(query_features, false)?;
        check_weight(self.no_read_bias)?;
        let mut score = i32::from(self.no_read_bias);
        for &feature in query_features {
            let weight = self.no_read_rows[usize::from(feature)];
            check_weight(weight)?;
            score = add_score(score, weight)?;
        }
        let selected = 1 + query_features.len() as u64;
        reads.record(selected, selected)?;
        Ok(score)
    }

    /// Convenience selection for offline examples. Scoring ignores `target`.
    /// NoRead wins every tie; tied reads retain their supplied candidate order.
    pub fn select(&self, example: &ReadActionExample) -> Result<Option<usize>, ReadActionError> {
        let scores = self.score_actions(example)?;
        Ok(selected(&scores, example.candidates.len() + 1))
    }

    /// Bounded online integer updates against the strongest non-target action.
    /// Shared coordinates cancel before a unit edit is applied. Scores and the
    /// strongest rival are recomputed after every round. A correction may harm
    /// another example and is not asserted to descend a global objective.
    pub fn train_example(
        &mut self,
        example: &ReadActionExample,
        config: &ReadActionTrainConfig,
    ) -> Result<ReadActionTrainReport, ReadActionError> {
        self.train_acceptable(example, &[example.target], config)
    }

    /// Multiple useful candidates need not be assigned contradictory negative
    /// labels. Correct the highest-scoring acceptable action against the best
    /// excluded action. Duplicate acceptable indices are harmless and do not
    /// receive extra weight. `example.target` is ignored by this method.
    pub fn train_acceptable(
        &mut self,
        example: &ReadActionExample,
        acceptable: &[Option<usize>],
        config: &ReadActionTrainConfig,
    ) -> Result<ReadActionTrainReport, ReadActionError> {
        config.validate()?;
        if acceptable.is_empty() || acceptable.len() > MAX_ACTIONS {
            return Err(ReadActionError::InvalidAcceptableCount(acceptable.len()));
        }
        let mut scores = self.score_actions(example)?;
        let action_count = example.candidates.len() + 1;
        let mut mask = [false; MAX_ACTIONS];
        for &action in acceptable {
            let index = match action {
                Some(index) if index < example.candidates.len() => index + 1,
                Some(index) => return Err(ReadActionError::InvalidTarget(index)),
                None => 0,
            };
            mask[index] = true;
        }
        let margin_before = acceptable_margin(&scores, action_count, &mask)?;
        let mut report = ReadActionTrainReport {
            selected_before: selected(&scores, action_count),
            selected_after: selected(&scores, action_count),
            margin_before,
            margin_after: margin_before,
            steps: 0,
            coefficient_edits: 0,
            integer_l1_delta: 0,
            changed_coefficients: 0,
            saturated_coordinates: 0,
            stop: ReadActionTrainStop::StepLimit,
        };
        let mut starting_weights = BTreeMap::new();
        for _ in 0..config.max_steps {
            let Some(rival) = strongest_masked(&scores, action_count, &mask, false) else {
                report.stop = ReadActionTrainStop::NoRival;
                break;
            };
            let target = strongest_masked(&scores, action_count, &mask, true)
                .ok_or(ReadActionError::InvalidAcceptableCount(0))?;
            if scores[target] - scores[rival] >= config.margin {
                report.stop = ReadActionTrainStop::MarginSatisfied;
                break;
            }
            let mut changes = BTreeMap::new();
            self.add_action_coordinates(example, target, 1, &mut changes)?;
            self.add_action_coordinates(example, rival, -1, &mut changes)?;
            let mut round_edits = 0u64;
            for (coordinate, difference) in changes {
                if difference == 0 {
                    continue;
                }
                let weight = self.get_coordinate(coordinate)?;
                let change = if difference > 0 { 1 } else { -1 };
                let next = (weight + change).clamp(-8, 7);
                if next == weight {
                    report.saturated_coordinates += 1;
                    continue;
                }
                starting_weights.entry(coordinate).or_insert(weight);
                self.set_coordinate(coordinate, next)?;
                round_edits += 1;
            }
            report.steps += 1;
            report.coefficient_edits += round_edits;
            scores = self.score_actions(example)?;
            if round_edits == 0 {
                report.stop = ReadActionTrainStop::NoAvailableEdit;
                break;
            }
        }
        report.selected_after = selected(&scores, action_count);
        report.margin_after = acceptable_margin(&scores, action_count, &mask)?;
        if report
            .margin_after
            .is_some_and(|margin| margin >= config.margin)
        {
            report.stop = ReadActionTrainStop::MarginSatisfied;
        }
        for (coordinate, before) in starting_weights {
            let after = self.get_coordinate(coordinate)?;
            let delta = (i16::from(after) - i16::from(before)).unsigned_abs();
            if delta != 0 {
                report.changed_coefficients += 1;
                report.integer_l1_delta += u64::from(delta);
            }
        }
        Ok(report)
    }

    fn check_features(&self, features: &[u16], read: bool) -> Result<Option<u16>, ReadActionError> {
        if features.len() > MAX_ACTIVE_FEATURES {
            return Err(ReadActionError::TooManyActiveFeatures(features.len()));
        }
        let value_base = self.vocabulary + (usize::from(self.lanes) << 7);
        let no_read_id = value_base + self.vocabulary;
        let mut token = None;
        for &feature in features {
            let index = usize::from(feature);
            if index >= self.feature_count() {
                return Err(ReadActionError::InvalidFeature(feature));
            }
            if index >= value_base && index < no_read_id {
                if !read {
                    return Err(ReadActionError::CandidateValueInNoRead);
                }
                if token.is_some() {
                    return Err(ReadActionError::MultipleCandidateValues);
                }
                token = Some((index - value_base) as u16);
            } else if index == no_read_id && read {
                return Err(ReadActionError::NoReadValueInRead);
            }
        }
        if read && token.is_none() {
            return Err(ReadActionError::MissingCandidateValue);
        }
        Ok(token)
    }

    fn check_relatives(&self, relatives: &[u8]) -> Result<(), ReadActionError> {
        if relatives.len() != usize::from(self.lanes) {
            return Err(ReadActionError::InvalidRelativeWidth {
                expected: usize::from(self.lanes),
                actual: relatives.len(),
            });
        }
        for &relative in relatives {
            if relative >= GROUP_ORDER {
                return Err(ReadActionError::InvalidRelative(relative));
            }
        }
        Ok(())
    }

    fn value_index(&self, token: u16, lane: u8, relative: u8) -> usize {
        (((usize::from(token) << self.lane_shift) | usize::from(lane)) << 7) | usize::from(relative)
    }

    fn score_actions(
        &self,
        example: &ReadActionExample,
    ) -> Result<[i32; MAX_ACTIONS], ReadActionError> {
        if example.candidates.len() > MAX_CANDIDATES {
            return Err(ReadActionError::TooManyCandidates(example.candidates.len()));
        }
        let mut reads = ReadActionReadCounts::default();
        let mut scores = [0; MAX_ACTIONS];
        scores[0] = self.score_none(&example.no_read_features, &mut reads)?;
        for (index, candidate) in example.candidates.iter().enumerate() {
            scores[index + 1] =
                self.score_read(&candidate.features, &candidate.relative, &mut reads)?;
        }
        Ok(scores)
    }

    fn add_action_coordinates(
        &self,
        example: &ReadActionExample,
        action: usize,
        sign: i32,
        coordinates: &mut BTreeMap<Coordinate, i32>,
    ) -> Result<(), ReadActionError> {
        let mut add = |coordinate| *coordinates.entry(coordinate).or_insert(0) += sign;
        if action == 0 {
            add(Coordinate::NoReadBias);
            for &feature in &example.no_read_features {
                add(Coordinate::NoReadFeature(feature));
            }
            return Ok(());
        }
        let candidate = example
            .candidates
            .get(action - 1)
            .ok_or(ReadActionError::InvalidTarget(action - 1))?;
        let token = self
            .check_features(&candidate.features, true)?
            .ok_or(ReadActionError::MissingCandidateValue)?;
        add(Coordinate::ReadBias);
        for &feature in &candidate.features {
            add(Coordinate::ReadFeature(feature));
        }
        for (lane, &relative) in candidate.relative.iter().enumerate() {
            add(Coordinate::Unary(lane as u8, relative));
            add(Coordinate::ValueRelative(token, lane as u8, relative));
        }
        for (index, edge) in self.relative_factors.edges().iter().enumerate() {
            add(Coordinate::Pair(
                index,
                candidate.relative[usize::from(edge.left)],
                candidate.relative[usize::from(edge.right)],
            ));
        }
        Ok(())
    }

    fn get_coordinate(&self, coordinate: Coordinate) -> Result<i8, ReadActionError> {
        let result = match coordinate {
            Coordinate::ReadBias => self.read_bias,
            Coordinate::NoReadBias => self.no_read_bias,
            Coordinate::ReadFeature(feature) => self.read_rows[usize::from(feature)],
            Coordinate::NoReadFeature(feature) => self.no_read_rows[usize::from(feature)],
            Coordinate::Unary(lane, relative) => self.relative_factors.get_unary(lane, relative)?,
            Coordinate::Pair(edge, left, right) => {
                self.relative_factors.get_pair(edge, left, right)?
            }
            Coordinate::ValueRelative(token, lane, relative) => {
                self.value_relative_rows[self.value_index(token, lane, relative)]
            }
        };
        check_weight(result)?;
        Ok(result)
    }

    fn set_coordinate(
        &mut self,
        coordinate: Coordinate,
        weight: i8,
    ) -> Result<(), ReadActionError> {
        check_weight(weight)?;
        match coordinate {
            Coordinate::ReadBias => self.read_bias = weight,
            Coordinate::NoReadBias => self.no_read_bias = weight,
            Coordinate::ReadFeature(feature) => self.read_rows[usize::from(feature)] = weight,
            Coordinate::NoReadFeature(feature) => self.no_read_rows[usize::from(feature)] = weight,
            Coordinate::Unary(lane, relative) => {
                self.relative_factors.set_unary(lane, relative, weight)?
            }
            Coordinate::Pair(edge, left, right) => {
                self.relative_factors.set_pair(edge, left, right, weight)?
            }
            Coordinate::ValueRelative(token, lane, relative) => {
                let index = self.value_index(token, lane, relative);
                self.value_relative_rows[index] = weight;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Coordinate {
    ReadBias,
    NoReadBias,
    ReadFeature(u16),
    NoReadFeature(u16),
    Unary(u8, u8),
    Pair(usize, u8, u8),
    ValueRelative(u16, u8, u8),
}

fn check_dimensions(vocabulary: usize, lanes: u8) -> Result<(), ReadActionError> {
    if !(8..=MAX_VOCABULARY).contains(&vocabulary) || !vocabulary.is_power_of_two() {
        return Err(ReadActionError::InvalidVocabulary(vocabulary));
    }
    if lanes == 0 || usize::from(lanes) > MAX_LANES || !lanes.is_power_of_two() {
        return Err(ReadActionError::InvalidLaneCount(usize::from(lanes)));
    }
    Ok(())
}

fn check_weight(weight: i8) -> Result<(), ReadActionError> {
    if !(-8..=7).contains(&weight) {
        return Err(ReadActionError::InvalidCoefficient(weight));
    }
    Ok(())
}

fn add_score(score: i32, weight: i8) -> Result<i32, ReadActionError> {
    score
        .checked_add(i32::from(weight))
        .ok_or(ReadActionError::ScoreOverflow)
}

fn selected(scores: &[i32; MAX_ACTIONS], action_count: usize) -> Option<usize> {
    let mut best = 0;
    for index in 1..action_count {
        if scores[index] > scores[best] {
            best = index;
        }
    }
    best.checked_sub(1)
}

fn strongest_masked(
    scores: &[i32; MAX_ACTIONS],
    action_count: usize,
    mask: &[bool; MAX_ACTIONS],
    accepted: bool,
) -> Option<usize> {
    let mut best = None;
    for index in 0..action_count {
        if mask[index] == accepted && best.is_none_or(|old| scores[index] > scores[old]) {
            best = Some(index);
        }
    }
    best
}

fn acceptable_margin(
    scores: &[i32; MAX_ACTIONS],
    action_count: usize,
    mask: &[bool; MAX_ACTIONS],
) -> Result<Option<i32>, ReadActionError> {
    let target = strongest_masked(scores, action_count, mask, true)
        .ok_or(ReadActionError::InvalidAcceptableCount(0))?;
    strongest_masked(scores, action_count, mask, false)
        .map(|rival| {
            scores[target]
                .checked_sub(scores[rival])
                .ok_or(ReadActionError::ScoreOverflow)
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> ReadActionExample {
        // V8/L2: query-token rows 0..8, state rows 8..264, value rows
        // 264..272, NoRead sentinel 272. Both candidates deliberately have the
        // same value, making their relative geometry necessary to distinguish.
        ReadActionExample {
            no_read_features: vec![2, 8, 136, 272],
            candidates: vec![
                ReadActionCandidate {
                    features: vec![2, 8, 136, 267],
                    relative: vec![3, 4],
                },
                ReadActionCandidate {
                    features: vec![2, 8, 136, 267],
                    relative: vec![3, 5],
                },
            ],
            target: Some(1),
        }
    }

    #[test]
    fn integer_learning_changes_hard_action_and_retains_shared_coordinates(
    ) -> Result<(), ReadActionError> {
        let mut model = ReadActionModel::new(8, 2, vec![LanePair { left: 0, right: 1 }])?;
        let data = example();
        assert_eq!(model.select(&data)?, None);
        let report = model.train_example(&data, &ReadActionTrainConfig::default())?;
        assert_eq!(report.selected_before, None);
        assert_eq!(report.selected_after, Some(1));
        assert_eq!(report.margin_before, Some(0));
        assert!(report.margin_after.is_some_and(|margin| margin >= 2));
        assert!(report.coefficient_edits > 0);
        assert!(report.integer_l1_delta > 0);
        assert!(report.changed_coefficients > 0);
        assert_eq!(report.stop, ReadActionTrainStop::MarginSatisfied);
        model.validate()?;

        let mut reads = ReadActionReadCounts::default();
        let _ = model.score_read(&data.candidates[1].features, &[3, 5], &mut reads)?;
        // Bias + four selected rows + two unary factors + one pair + two
        // value-relative factors; counters include the packed factor accesses.
        assert_eq!(reads.coefficients, 10);
        assert_eq!(reads.parameter_bytes, 10);

        // Identical score paths cannot be separated by metadata labels. Their
        // coordinates cancel exactly and training reports its inability.
        let mut identical = data.clone();
        identical.candidates[0] = identical.candidates[1].clone();
        identical.target = Some(1);
        let before = model.clone();
        let unresolved = model.train_example(&identical, &ReadActionTrainConfig::default())?;
        assert_eq!(unresolved.stop, ReadActionTrainStop::NoAvailableEdit);
        assert_eq!(unresolved.coefficient_edits, 0);
        assert_eq!(unresolved.integer_l1_delta, 0);
        assert_eq!(model, before);
        assert_eq!(model.select(&identical)?, Some(0));

        // Equivalent useful records may both be accepted, so their shared
        // value and relative representation receives no contradictory label.
        let mut multi = ReadActionModel::new(8, 2, vec![LanePair { left: 0, right: 1 }])?;
        let multi_report = multi.train_acceptable(
            &identical,
            &[Some(0), Some(1)],
            &ReadActionTrainConfig::default(),
        )?;
        assert_eq!(multi_report.selected_after, Some(0));
        assert!(multi_report.margin_after.is_some_and(|margin| margin >= 2));
        let components = multi.score_read_components(
            &identical.candidates[0].features,
            &identical.candidates[0].relative,
            &mut ReadActionReadCounts::default(),
        )?;
        assert_eq!(
            components.total,
            components.read_feature_score
                + components.relative_score
                + components.value_relative_score
        );
        let stats = multi.parameter_stats()?;
        assert!(stats.relative_pair.nonzero > 0);
        assert!(stats.value_relative.nonzero > 0);
        let all = multi.train_acceptable(
            &identical,
            &[None, Some(0), Some(1)],
            &ReadActionTrainConfig::default(),
        )?;
        assert_eq!(all.stop, ReadActionTrainStop::NoRival);
        assert_eq!(all.coefficient_edits, 0);
        Ok(())
    }

    #[test]
    fn rejects_invalid_inputs_and_preserves_serde_and_no_read_ties(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut model = ReadActionModel::new(8, 2, vec![LanePair { left: 0, right: 1 }])?;
        let mut data = example();
        data.target = None;
        let report = model.train_example(&data, &ReadActionTrainConfig::default())?;
        assert_eq!(report.selected_after, None);
        assert!(report.margin_after.is_some_and(|margin| margin >= 2));
        let bytes = serde_json::to_vec(&model)?;
        let loaded: ReadActionModel = serde_json::from_slice(&bytes)?;
        loaded.validate()?;
        assert_eq!(loaded, model);
        assert_eq!(loaded.select(&data)?, None);

        let before = model.clone();
        let mut invalid = example();
        invalid.candidates[0].relative[1] = 120;
        assert!(matches!(
            model.train_example(&invalid, &ReadActionTrainConfig::default()),
            Err(ReadActionError::InvalidRelative(120))
        ));
        assert_eq!(model, before);
        invalid = example();
        invalid.target = Some(2);
        assert_eq!(
            model.train_example(&invalid, &ReadActionTrainConfig::default()),
            Err(ReadActionError::InvalidTarget(2))
        );
        let mut counts = ReadActionReadCounts::default();
        assert_eq!(
            model.score_read(&[2, 8, 136], &[3, 4], &mut counts),
            Err(ReadActionError::MissingCandidateValue)
        );
        assert_eq!(
            model.score_none(&[267], &mut counts),
            Err(ReadActionError::CandidateValueInNoRead)
        );
        assert_eq!(
            model.score_read(&[267, 267], &[3, 4], &mut counts),
            Err(ReadActionError::MultipleCandidateValues)
        );
        let candidate = example().candidates.remove(0);
        invalid.candidates = vec![candidate; 65];
        assert_eq!(
            model.select(&invalid),
            Err(ReadActionError::TooManyCandidates(65))
        );

        let mut malformed: serde_json::Value = serde_json::from_slice(&bytes)?;
        malformed["read_bias"] = serde_json::json!(8);
        let rejected: ReadActionModel = serde_json::from_value(malformed)?;
        assert_eq!(
            rejected.validate(),
            Err(ReadActionError::InvalidCoefficient(8))
        );

        let empty = ReadActionExample {
            no_read_features: vec![2, 272],
            candidates: Vec::new(),
            target: None,
        };
        let unchanged = model.train_example(&empty, &ReadActionTrainConfig::default())?;
        assert_eq!(unchanged.stop, ReadActionTrainStop::NoRival);
        assert_eq!(unchanged.margin_after, None);
        assert_eq!(unchanged.coefficient_edits, 0);
        assert_eq!(model.select(&empty)?, None);
        Ok(())
    }
}
