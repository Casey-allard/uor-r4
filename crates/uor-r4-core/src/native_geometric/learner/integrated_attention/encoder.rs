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
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

pub const ENCODER_VERSION: u8 = 1;
pub const GROUP_ORDER: usize = 120;
pub const ROW_STRIDE: usize = 128;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_LANES: usize = 16;
pub const CONTEXT_TOKENS: usize = 32;
const MAX_SELECTED_ROWS: usize = CONTEXT_TOKENS + 5;
const MIN_CODE: i8 = -8;
const MAX_CODE: i8 = 7;
const TRAIN_TEMPERATURE: f64 = 4.0;
const CONTRASTIVE_TEMPERATURE: f64 = 8.0;
const OVERLAP_FLOOR: f64 = 1.0e-6;
const UTILIZATION_WEIGHT: f64 = 0.02;

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
    /// Causally observed context. Only `context[..context_len]` is read;
    /// `u16::MAX` masks an anchor whose value the query cannot know.
    #[serde(default)]
    pub context: [u16; CONTEXT_TOKENS],
    #[serde(default)]
    pub context_len: u8,
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
    InvalidHardBatch,
    InvalidHardConfig,
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

/// One causal prefix in a fit-only, paired source edit. `committed_keys` are in
/// actual commit order, not token or event order. The annotated source must
/// already have committed before `query`; the caller establishes causality by
/// replaying the prefix. Unlabeled keys affect page capacity only. Only the
/// explicitly annotated wrong-entity keys are semantic negatives.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardAddressWorld {
    pub query: EncoderInput,
    pub positive_index: usize,
    pub negative_indices: Vec<usize>,
    pub committed_keys: Vec<EncoderInput>,
}

/// Both source-value variants of one authored world. Every pair with the same
/// `entity_group` shares a single latent coarse bucket across styles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardAddressPair {
    pub entity_group: u16,
    pub worlds: [HardAddressWorld; 2],
}

/// Explicit offline search ceilings. Page capacity must match the served
/// memory limit. This greedy integer search has no global optimum guarantee.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HardAddressConfig {
    pub margin: i16,
    pub page_capacity: usize,
    pub max_pairs: usize,
    pub max_keys_per_world: usize,
    pub max_proposals: u64,
    /// Counts latent-bucket evaluations, including the initial objective.
    pub max_candidate_evaluations: u64,
    pub max_accepted_edits: u64,
    pub max_sweeps: u8,
    pub max_duration_ms: u64,
}

impl Default for HardAddressConfig {
    fn default() -> Self {
        Self {
            margin: 2,
            page_capacity: 64,
            max_pairs: 96,
            max_keys_per_world: 256,
            max_proposals: 20_000,
            max_candidate_evaluations: 5_000_000,
            max_accepted_edits: 4_096,
            max_sweeps: 3,
            max_duration_ms: 240_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct HardAddressSnapshot {
    pub objective: i64,
    pub admitted_worlds: u64,
    pub overflow_worlds: u64,
    pub wrong_entity_collisions: u64,
    /// Largest requested coarse-page load across committed keys, including
    /// keys refused after capacity. It is not physical stored posting length.
    pub max_requested_page_load: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardAddressStop {
    CandidatesExhausted,
    ProposalLimit,
    CandidateEvaluationLimit,
    AcceptedEditLimit,
    DurationLimit,
    SweepLimit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardAddressReport {
    pub before: HardAddressSnapshot,
    pub after: HardAddressSnapshot,
    pub pairs: usize,
    pub entity_groups: usize,
    pub skipped_identical_negatives: u64,
    pub proposals_evaluated: u64,
    pub candidate_evaluations: u64,
    pub accepted_edits: u64,
    pub changed_address_coefficients: usize,
    pub changed_observed_coarse_codes: usize,
    pub elapsed_ms: u64,
    pub stop: HardAddressStop,
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

#[derive(Clone, Copy)]
struct SelectedRows {
    ids: [usize; MAX_SELECTED_ROWS],
    len: usize,
}

impl SelectedRows {
    fn new() -> Self {
        Self {
            ids: [0; MAX_SELECTED_ROWS],
            len: 0,
        }
    }

    fn push(&mut self, row: usize) -> Result<(), EncoderError> {
        let slot = self
            .ids
            .get_mut(self.len)
            .ok_or(EncoderError::InvalidShape)?;
        *slot = row;
        self.len += 1;
        Ok(())
    }

    fn as_slice(&self) -> &[usize] {
        &self.ids[..self.len]
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
    /// A2 address mode; absent in A1 artifacts and defaults to false.
    #[serde(default)]
    pub context_addressing: bool,
    layout: Layout,
    coefficients: Vec<i8>,
    /// Allocated only for A2. Context token rows are separate from State's
    /// exact-token rows so address training cannot rewrite State transitions.
    #[serde(default)]
    address_coefficients: Vec<i8>,
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
            context_addressing: false,
            layout,
            coefficients: vec![0; coefficient_count],
            address_coefficients: Vec::new(),
        };
        model.initialize(seed)?;
        model.validate()?;
        Ok(model)
    }

    pub fn stored_coefficient_bytes(&self) -> usize {
        self.coefficients.len() + self.address_coefficients.len()
    }

    pub fn stored_coefficient_slots(&self) -> usize {
        self.coefficients.len() + self.address_coefficients.len()
    }

    pub fn coefficients(&self) -> &[i8] {
        &self.coefficients
    }

    pub fn address_coefficients(&self) -> &[i8] {
        &self.address_coefficients
    }

    /// Enables A2 with a deterministic, separately owned address-token bank.
    /// Setting `context_addressing` without this constructor fails validation.
    pub fn enable_context_addressing(&mut self) -> Result<(), EncoderError> {
        if self.context_addressing {
            return self.validate();
        }
        let token_rows = usize::from(self.vocab)
            .checked_mul(usize::from(self.lanes))
            .ok_or(EncoderError::InvalidShape)?;
        let count = token_rows
            .checked_shl(7)
            .ok_or(EncoderError::InvalidShape)?;
        let mut bank = vec![0i8; count];
        for row in 0..token_rows {
            let base = row << 7;
            for action in 0..GROUP_ORDER {
                let salt = self.seed ^ 0xa2d4_21e3_b692_1c57 ^ ((row as u64) << 7) ^ action as u64;
                bank[base + action] =
                    i8::try_from(splitmix64(salt) % 3).map_err(|_| EncoderError::InvalidShape)? - 1;
            }
        }
        self.address_coefficients = bank;
        self.context_addressing = true;
        self.validate()
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
            || (self.context_addressing
                && self.address_coefficients.len() != (vocab << self.layout.lane_shift) << 7)
            || (!self.context_addressing && !self.address_coefficients.is_empty())
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
        for row in 0..(self.address_coefficients.len() >> 7) {
            let base = row << 7;
            if self.address_coefficients[base..base + GROUP_ORDER]
                .iter()
                .any(|coefficient| !(MIN_CODE..=MAX_CODE).contains(coefficient))
                || self.address_coefficients[base + GROUP_ORDER..base + ROW_STRIDE]
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
        if usize::from(input.context_len) > CONTEXT_TOKENS
            || input.context[..usize::from(input.context_len)]
                .iter()
                .any(|token| *token != u16::MAX && *token >= self.vocab)
        {
            return Err(EncoderError::InvalidInput);
        }
        for code in input.previous.iter().take(usize::from(self.lanes)) {
            if usize::from(*code) >= GROUP_ORDER {
                return Err(EncoderError::InvalidCode);
            }
        }
        Ok(())
    }

    fn selected_rows(
        &self,
        input: EncoderInput,
        lane: usize,
    ) -> Result<SelectedRows, EncoderError> {
        let lanes = usize::from(self.lanes);
        if lane >= lanes {
            return Err(EncoderError::InvalidInput);
        }
        let neighbor = (lane + 1) & (lanes - 1);
        let evidence = input.evidence.map_or(usize::from(self.vocab), usize::from);
        let mut rows = SelectedRows::new();
        if self.context_addressing && input.kind != EncoderKind::State {
            // A2: both sides address from the same type of causal context.
            // The masked source value cannot enter the coarse prefix lane.
            for &token in input.context.iter().take(usize::from(input.context_len)) {
                if token != u16::MAX {
                    rows.push(
                        self.layout.row_count + self.layout.row(0, usize::from(token), lane),
                    )?;
                }
            }
            if lane == 0 {
                // One shared Key/Query role row prevents a global typed offset
                // from swamping the value-blind candidate-admission prefix.
                rows.push(
                    self.layout
                        .row(self.layout.kind_base, EncoderKind::Key.index(), lane),
                )?;
            } else {
                // Later lanes retain content and frame information for the
                // relative geometric ranker; a query has no future evidence.
                rows.push(self.layout.row(
                    self.layout.previous_base,
                    usize::from(input.previous[lane]),
                    lane,
                ))?;
                rows.push(self.layout.row(
                    self.layout.neighbor_base,
                    usize::from(input.previous[neighbor]),
                    lane,
                ))?;
                rows.push(self.layout.row(self.layout.evidence_base, evidence, lane))?;
                rows.push(
                    self.layout
                        .row(self.layout.kind_base, input.kind.index(), lane),
                )?;
            }
        } else {
            // Original A1 path, including State actions in an A2 artifact.
            rows.push(self.layout.row(0, usize::from(input.token), lane))?;
            rows.push(self.layout.row(
                self.layout.previous_base,
                usize::from(input.previous[lane]),
                lane,
            ))?;
            rows.push(self.layout.row(
                self.layout.neighbor_base,
                usize::from(input.previous[neighbor]),
                lane,
            ))?;
            rows.push(self.layout.row(self.layout.evidence_base, evidence, lane))?;
            rows.push(
                self.layout
                    .row(self.layout.kind_base, input.kind.index(), lane),
            )?;
        }
        Ok(rows)
    }

    fn coefficient_at(&self, row: usize, action: usize) -> Result<i8, EncoderError> {
        let index = row
            .checked_shl(7)
            .and_then(|base| base.checked_add(action))
            .ok_or(EncoderError::CorruptArtifact)?;
        let coefficient = if row < self.layout.row_count {
            self.coefficients.get(index)
        } else {
            self.address_coefficients
                .get(index - self.coefficients.len())
        };
        coefficient.copied().ok_or(EncoderError::CorruptArtifact)
    }

    pub fn score_lane(&self, input: EncoderInput, lane: usize) -> Result<LaneScores, EncoderError> {
        self.check_input(input)?;
        let rows = self.selected_rows(input, lane)?;
        let mut scores = [0i16; GROUP_ORDER];
        for &row in rows.as_slice() {
            for (action, score) in scores.iter_mut().enumerate() {
                let code = self.coefficient_at(row, action)?;
                if !(MIN_CODE..=MAX_CODE).contains(&code) {
                    return Err(EncoderError::InvalidCoefficient);
                }
                *score += i16::from(code);
            }
        }
        Ok(rank_scores(scores, rows.len))
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

fn rank_scores(scores: [i16; GROUP_ORDER], selected_rows: usize) -> LaneScores {
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
        coefficient_reads: (selected_rows * GROUP_ORDER) as u32,
    }
}

/// SplitMix64 is used only for offline deterministic artifact construction.
fn splitmix64(value: u64) -> u64 {
    let mut z = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[derive(Clone)]
struct HardExample {
    /// Only dedicated lane-zero address rows can be edited.
    content_rows: Vec<usize>,
    scores: [i16; GROUP_ORDER],
    code: u8,
    runner_up: u8,
    group: usize,
}

struct HardWorldRef {
    query: usize,
    positive: usize,
    committed: Vec<usize>,
    negatives: Vec<usize>,
    positive_index: usize,
}

#[derive(Clone, Copy, Default)]
struct HardGroupOutcome {
    objective: i64,
    bucket: u8,
    admitted_worlds: u64,
    overflow_worlds: u64,
    wrong_entity_collisions: u64,
    max_requested_page_load: usize,
}

struct HardGroup {
    worlds: Vec<usize>,
    outcome: HardGroupOutcome,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct HardEdit {
    row: usize,
    action: usize,
    delta: i8,
}

fn hard_best_other(example: &HardExample, excluded: usize) -> (i16, usize) {
    let other = if usize::from(example.code) == excluded {
        usize::from(example.runner_up)
    } else {
        usize::from(example.code)
    };
    (example.scores[other], other)
}

fn hard_group_outcome(
    group: &HardGroup,
    worlds: &[HardWorldRef],
    examples: &[HardExample],
    config: HardAddressConfig,
) -> HardGroupOutcome {
    // All terms below are integer functions of exported four-bit coefficients.
    // The 128/16 constants prioritize actual page admission over a one-point
    // margin improvement; they are declared loss weights, not a theorem.
    const MISSED_ADMISSION_COST: i64 = 128;
    const NEGATIVE_COLLISION_COST: i64 = 16;
    const PAGE_OVERFLOW_COST: i64 = 128;
    let mut common = HardGroupOutcome::default();
    let mut counts_before = Vec::<[usize; GROUP_ORDER]>::with_capacity(group.worlds.len());
    for &world_id in &group.worlds {
        let world = &worlds[world_id];
        let query_code = usize::from(examples[world.query].code);
        let positive_code = usize::from(examples[world.positive].code);
        let mut counts = [0usize; GROUP_ORDER];
        let mut before = [0usize; GROUP_ORDER];
        for (position, &key) in world.committed.iter().enumerate() {
            if position == world.positive_index {
                before = counts;
            }
            let code = usize::from(examples[key].code);
            counts[code] += 1;
            common.max_requested_page_load = common.max_requested_page_load.max(counts[code]);
        }
        common.overflow_worlds += u64::from(before[positive_code] >= config.page_capacity);
        common.admitted_worlds +=
            u64::from(query_code == positive_code && before[positive_code] < config.page_capacity);
        common.wrong_entity_collisions += world
            .negatives
            .iter()
            .filter(|&&negative| usize::from(examples[negative].code) == query_code)
            .count() as u64;
        counts_before.push(before);
    }
    let hard_penalty = MISSED_ADMISSION_COST
        * (group.worlds.len() as i64 - common.admitted_worlds as i64)
        + NEGATIVE_COLLISION_COST * common.wrong_entity_collisions as i64;
    let mut best_cost = i64::MAX;
    let mut best_bucket = 0u8;
    for bucket in 0..GROUP_ORDER {
        let mut cost = hard_penalty;
        for (world_offset, &world_id) in group.worlds.iter().enumerate() {
            let world = &worlds[world_id];
            for anchor in [world.query, world.positive] {
                let example = &examples[anchor];
                let (other_score, _) = hard_best_other(example, bucket);
                cost += i64::from((config.margin + other_score - example.scores[bucket]).max(0));
            }
            for &negative in &world.negatives {
                let example = &examples[negative];
                let (other_score, _) = hard_best_other(example, bucket);
                cost += i64::from((config.margin + example.scores[bucket] - other_score).max(0));
                cost +=
                    NEGATIVE_COLLISION_COST * i64::from(examples[negative].code == bucket as u8);
            }
            let earlier = counts_before[world_offset][bucket];
            if earlier >= config.page_capacity {
                cost += PAGE_OVERFLOW_COST + (earlier - config.page_capacity + 1) as i64;
            }
        }
        if cost < best_cost {
            best_cost = cost;
            best_bucket = bucket as u8;
        }
    }
    HardGroupOutcome {
        objective: best_cost,
        bucket: best_bucket,
        ..common
    }
}

fn hard_snapshot(groups: &[HardGroup]) -> HardAddressSnapshot {
    let mut snapshot = HardAddressSnapshot::default();
    for group in groups {
        let outcome = group.outcome;
        snapshot.objective += outcome.objective;
        snapshot.admitted_worlds += outcome.admitted_worlds;
        snapshot.overflow_worlds += outcome.overflow_worlds;
        snapshot.wrong_entity_collisions += outcome.wrong_entity_collisions;
        snapshot.max_requested_page_load = snapshot
            .max_requested_page_load
            .max(outcome.max_requested_page_load);
    }
    snapshot
}

fn hard_candidates(
    group: &HardGroup,
    worlds: &[HardWorldRef],
    examples: &[HardExample],
) -> Vec<HardEdit> {
    let bucket = usize::from(group.outcome.bucket);
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    for &world_id in &group.worlds {
        let world = &worlds[world_id];
        for example_id in [world.query, world.positive] {
            let example = &examples[example_id];
            let (_, other) = hard_best_other(example, bucket);
            for &row in &example.content_rows {
                for edit in [
                    HardEdit {
                        row,
                        action: bucket,
                        delta: 1,
                    },
                    HardEdit {
                        row,
                        action: other,
                        delta: -1,
                    },
                ] {
                    if seen.insert(edit) {
                        candidates.push(edit);
                    }
                }
            }
        }
        for &example_id in &world.negatives {
            let example = &examples[example_id];
            let (_, other) = hard_best_other(example, bucket);
            for &row in &example.content_rows {
                for edit in [
                    HardEdit {
                        row,
                        action: bucket,
                        delta: -1,
                    },
                    HardEdit {
                        row,
                        action: other,
                        delta: 1,
                    },
                ] {
                    if seen.insert(edit) {
                        candidates.push(edit);
                    }
                }
            }
        }
    }
    candidates
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
            .chain(model.address_coefficients.iter())
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

    fn push_hard_example(
        &self,
        input: EncoderInput,
        group: usize,
        examples: &mut Vec<HardExample>,
        incidence: &mut BTreeMap<usize, BTreeMap<usize, u8>>,
    ) -> Result<usize, EncoderError> {
        self.model.check_input(input)?;
        let selected = self.model.selected_rows(input, 0)?;
        let scored = self.model.score_lane(input, 0)?;
        let index = examples.len();
        let mut content_rows = Vec::new();
        for &row in selected.as_slice() {
            if row >= self.model.layout.row_count {
                content_rows.push(row);
                let count = incidence.entry(row).or_default().entry(index).or_default();
                *count = count.checked_add(1).ok_or(EncoderError::InvalidHardBatch)?;
            }
        }
        examples.push(HardExample {
            content_rows,
            scores: scored.scores,
            code: scored.best,
            runner_up: scored.runner_up,
            group,
        });
        Ok(index)
    }

    /// Offline A3 greedy coordinate search over the *exported integer* coarse
    /// address bank. Both worlds of every pair, and all styles with one
    /// `entity_group`, choose one latent bucket. Only dedicated lane-zero
    /// context-token coefficients receive bounded ±1 edits; the shared role
    /// row, State actions and fine lanes are untouched. A proposal is accepted
    /// only when the balanced hard batch's integer objective strictly falls.
    /// The objective includes exact argmax admission and chronological 64-like
    /// posting capacity, but it is a fit-only surrogate for real Session
    /// replay; page count/eviction and downstream language still need checks.
    /// Search is local, deterministic for a given input ordering, and bounded
    /// by proposals, latent-bucket evaluations, accepted edits and elapsed time.
    pub fn train_hard_address_batch(
        &mut self,
        pairs: &[HardAddressPair],
        config: HardAddressConfig,
    ) -> Result<HardAddressReport, EncoderError> {
        if !self.model.context_addressing {
            return Err(EncoderError::InvalidHardBatch);
        }
        if pairs.is_empty()
            || pairs.len() > config.max_pairs
            || !(1..=32).contains(&config.margin)
            || !(1..=4096).contains(&config.page_capacity)
            || !(1..=4096).contains(&config.max_keys_per_world)
            || config.max_proposals == 0
            || config.max_accepted_edits == 0
            || !(1..=8).contains(&config.max_sweeps)
            || config.max_duration_ms == 0
            || config.max_candidate_evaluations == 0
        {
            return Err(EncoderError::InvalidHardConfig);
        }
        let started = Instant::now();
        let duration = Duration::from_millis(config.max_duration_ms);
        let mut group_ids = BTreeMap::<u16, usize>::new();
        let mut groups = Vec::<HardGroup>::new();
        let mut worlds = Vec::<HardWorldRef>::new();
        let mut examples = Vec::<HardExample>::new();
        let mut incidence = BTreeMap::<usize, BTreeMap<usize, u8>>::new();
        let mut skipped_identical_negatives = 0u64;
        for pair in pairs {
            let group_id = if let Some(&id) = group_ids.get(&pair.entity_group) {
                id
            } else {
                let id = groups.len();
                group_ids.insert(pair.entity_group, id);
                groups.push(HardGroup {
                    worlds: Vec::new(),
                    outcome: HardGroupOutcome::default(),
                });
                id
            };
            for world in &pair.worlds {
                if world.query.kind != EncoderKind::Query
                    || world.committed_keys.is_empty()
                    || world.committed_keys.len() > config.max_keys_per_world
                    || world.positive_index >= world.committed_keys.len()
                    || world
                        .committed_keys
                        .iter()
                        .any(|key| key.kind != EncoderKind::Key)
                    || world.negative_indices.iter().any(|&index| {
                        index >= world.committed_keys.len() || index == world.positive_index
                    })
                {
                    return Err(EncoderError::InvalidHardBatch);
                }
                let query =
                    self.push_hard_example(world.query, group_id, &mut examples, &mut incidence)?;
                let mut committed = Vec::with_capacity(world.committed_keys.len());
                for &key in &world.committed_keys {
                    committed.push(self.push_hard_example(
                        key,
                        group_id,
                        &mut examples,
                        &mut incidence,
                    )?);
                }
                let positive = committed[world.positive_index];
                let mut positive_signature = examples[positive].content_rows.clone();
                positive_signature.sort_unstable();
                let mut negatives = Vec::new();
                let mut seen_negatives = BTreeSet::new();
                for &index in &world.negative_indices {
                    if !seen_negatives.insert(index) {
                        return Err(EncoderError::InvalidHardBatch);
                    }
                    let negative = committed[index];
                    let mut signature = examples[negative].content_rows.clone();
                    signature.sort_unstable();
                    if signature == positive_signature {
                        skipped_identical_negatives += 1;
                    } else {
                        negatives.push(negative);
                    }
                }
                let world_id = worlds.len();
                worlds.push(HardWorldRef {
                    query,
                    positive,
                    committed,
                    negatives,
                    positive_index: world.positive_index,
                });
                groups[group_id].worlds.push(world_id);
            }
        }
        let initial_evaluations = (groups.len() as u64)
            .checked_mul(GROUP_ORDER as u64)
            .ok_or(EncoderError::InvalidHardConfig)?;
        if initial_evaluations > config.max_candidate_evaluations {
            return Err(EncoderError::InvalidHardConfig);
        }
        let mut candidate_evaluations = initial_evaluations;
        for group in &mut groups {
            group.outcome = hard_group_outcome(group, &worlds, &examples, config);
        }
        let before = hard_snapshot(&groups);
        let initial_codes: Vec<u8> = examples.iter().map(|example| example.code).collect();
        let initial_bank = self.model.address_coefficients.clone();
        let incidence: BTreeMap<usize, Vec<(usize, u8)>> = incidence
            .into_iter()
            .map(|(row, uses)| (row, uses.into_iter().collect()))
            .collect();
        let mut proposals_evaluated = 0u64;
        let mut accepted_edits = 0u64;
        let mut stop = HardAddressStop::CandidatesExhausted;
        // Rebuild candidates after each sweep so a newly chosen shared bucket
        // can generate a new local search direction. Every proposal and
        // latent-code evaluation is separately capped.
        'search: for sweep in 0..config.max_sweeps {
            let candidates: Vec<Vec<HardEdit>> = groups
                .iter()
                .map(|group| hard_candidates(group, &worlds, &examples))
                .collect();
            let maximum = candidates.iter().map(Vec::len).max().unwrap_or(0);
            let mut seen = BTreeSet::new();
            let accepted_before_sweep = accepted_edits;
            for position in 0..maximum {
                // Round-robin prevents a proposal cap from starving later
                // entities because of arbitrary lexical row order.
                for group_candidates in &candidates {
                    let Some(&edit) = group_candidates.get(position) else {
                        continue;
                    };
                    if !seen.insert(edit) {
                        continue;
                    }
                    if proposals_evaluated >= config.max_proposals {
                        stop = HardAddressStop::ProposalLimit;
                        break 'search;
                    }
                    if accepted_edits >= config.max_accepted_edits {
                        stop = HardAddressStop::AcceptedEditLimit;
                        break 'search;
                    }
                    if started.elapsed() >= duration {
                        stop = HardAddressStop::DurationLimit;
                        break 'search;
                    }
                    let Some(uses) = incidence.get(&edit.row) else {
                        continue;
                    };
                    let mut affected = BTreeSet::new();
                    for &(example, _) in uses {
                        affected.insert(examples[example].group);
                    }
                    let required = (affected.len() as u64)
                        .checked_mul(GROUP_ORDER as u64)
                        .ok_or(EncoderError::InvalidHardConfig)?;
                    if candidate_evaluations
                        .checked_add(required)
                        .is_none_or(|total| total > config.max_candidate_evaluations)
                    {
                        stop = HardAddressStop::CandidateEvaluationLimit;
                        break 'search;
                    }
                    let index = (edit.row << 7) + edit.action;
                    let old = *self
                        .model
                        .address_coefficients
                        .get(index - self.model.coefficients.len())
                        .ok_or(EncoderError::CorruptArtifact)?;
                    let Some(new) = old.checked_add(edit.delta) else {
                        continue;
                    };
                    if !(MIN_CODE..=MAX_CODE).contains(&new) {
                        continue;
                    }
                    proposals_evaluated += 1;
                    candidate_evaluations += required;
                    let old_cost: i64 = affected
                        .iter()
                        .map(|&group| groups[group].outcome.objective)
                        .sum();
                    self.set_quantized(index, new)?;
                    for &(example, count) in uses {
                        let sample = &mut examples[example];
                        sample.scores[edit.action] += i16::from(edit.delta) * i16::from(count);
                        let ranked = rank_scores(sample.scores, 0);
                        sample.code = ranked.best;
                        sample.runner_up = ranked.runner_up;
                    }
                    let mut updated = Vec::with_capacity(affected.len());
                    let mut new_cost = 0i64;
                    for &group in &affected {
                        let outcome =
                            hard_group_outcome(&groups[group], &worlds, &examples, config);
                        new_cost += outcome.objective;
                        updated.push((group, outcome));
                    }
                    if new_cost < old_cost {
                        for (group, outcome) in updated {
                            groups[group].outcome = outcome;
                        }
                        *self
                            .master
                            .get_mut(index)
                            .ok_or(EncoderError::CorruptArtifact)? = f32::from(new);
                        accepted_edits += 1;
                    } else {
                        self.set_quantized(index, old)?;
                        for &(example, count) in uses {
                            let sample = &mut examples[example];
                            sample.scores[edit.action] -= i16::from(edit.delta) * i16::from(count);
                            let ranked = rank_scores(sample.scores, 0);
                            sample.code = ranked.best;
                            sample.runner_up = ranked.runner_up;
                        }
                    }
                }
            }
            if accepted_edits == accepted_before_sweep {
                break;
            }
            if sweep + 1 == config.max_sweeps {
                stop = HardAddressStop::SweepLimit;
            }
        }
        let changed_address_coefficients = initial_bank
            .iter()
            .zip(&self.model.address_coefficients)
            .filter(|(before, after)| before != after)
            .count();
        let changed_observed_coarse_codes = initial_codes
            .iter()
            .zip(&examples)
            .filter(|(before, after)| **before != after.code)
            .count();
        let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        Ok(HardAddressReport {
            before,
            after: hard_snapshot(&groups),
            pairs: pairs.len(),
            entity_groups: groups.len(),
            skipped_identical_negatives,
            proposals_evaluated,
            candidate_evaluations,
            accepted_edits,
            changed_address_coefficients,
            changed_observed_coarse_codes,
            elapsed_ms,
            stop,
        })
    }

    fn distribution(
        &self,
        input: EncoderInput,
        lane: usize,
    ) -> Result<([f64; GROUP_ORDER], SelectedRows), EncoderError> {
        self.distribution_at(input, lane, TRAIN_TEMPERATURE)
    }

    fn distribution_at(
        &self,
        input: EncoderInput,
        lane: usize,
        temperature: f64,
    ) -> Result<([f64; GROUP_ORDER], SelectedRows), EncoderError> {
        self.model.check_input(input)?;
        let rows = self.model.selected_rows(input, lane)?;
        let mut scores = [0.0_f64; GROUP_ORDER];
        for &row in rows.as_slice() {
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
            *score = ((*score - maximum) / temperature).exp();
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

    fn set_quantized(&mut self, index: usize, quantized: i8) -> Result<(), EncoderError> {
        let primary_len = self.model.coefficients.len();
        if index < primary_len {
            *self
                .model
                .coefficients
                .get_mut(index)
                .ok_or(EncoderError::CorruptArtifact)? = quantized;
        } else {
            *self
                .model
                .address_coefficients
                .get_mut(index - primary_len)
                .ok_or(EncoderError::CorruptArtifact)? = quantized;
        }
        Ok(())
    }

    fn update_rows(
        &mut self,
        rows: SelectedRows,
        gradients: &[f64; GROUP_ORDER],
        rate: f32,
    ) -> Result<(), EncoderError> {
        for &row in rows.as_slice() {
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
                self.set_quantized(index, quantized)?;
            }
        }
        Ok(())
    }

    fn accumulate_distribution_gradient(
        gradients: &mut BTreeMap<usize, [f64; GROUP_ORDER]>,
        rows: SelectedRows,
        probabilities: &[f64; GROUP_ORDER],
        direct: &[f64; GROUP_ORDER],
        temperature: f64,
    ) -> Result<(), EncoderError> {
        let center = probabilities
            .iter()
            .zip(direct)
            .map(|(p, derivative)| p * derivative)
            .sum::<f64>();
        if !center.is_finite() {
            return Err(EncoderError::InvalidLoss);
        }
        for &row in rows.as_slice() {
            let slot = gradients.entry(row).or_insert([0.0; GROUP_ORDER]);
            for action in 0..GROUP_ORDER {
                slot[action] += probabilities[action] * (direct[action] - center) / temperature;
                if !slot[action].is_finite() {
                    return Err(EncoderError::InvalidLoss);
                }
            }
        }
        Ok(())
    }

    fn apply_accumulated(
        &mut self,
        gradients: BTreeMap<usize, [f64; GROUP_ORDER]>,
        rate: f32,
    ) -> Result<(), EncoderError> {
        // Each coefficient is updated once from the same pre-update batch,
        // including rows shared by Query, positive Key and negative Keys.
        for (row, gradient) in gradients {
            let base = row.checked_shl(7).ok_or(EncoderError::CorruptArtifact)?;
            for (action, derivative) in gradient.into_iter().enumerate() {
                let index = base + action;
                let master = self
                    .master
                    .get_mut(index)
                    .ok_or(EncoderError::CorruptArtifact)?;
                let step = rate * derivative as f32;
                if !step.is_finite() {
                    return Err(EncoderError::InvalidLoss);
                }
                *master = (*master - step).clamp(f32::from(MIN_CODE), f32::from(MAX_CODE));
                let quantized = master.round() as i8;
                self.set_quantized(index, quantized)?;
            }
        }
        Ok(())
    }

    /// Offline A2 address learning. It compares a causally identifiable
    /// positive Key with competing committed Key inputs, using overlap between
    /// their soft 120-way *coarse lane* distributions. A2 hard admission probes
    /// that lane alone; fine lanes need not match an unknown source value.
    /// A small batch mutual-information term favors distinct, confident Key
    /// assignments. This is a surrogate: serving still uses hard argmax codes
    /// and bounded prefix pages, whose occupancy and recall must be measured.
    /// Returns the pre-update contrastive cross entropy, excluding the small
    /// utilization term. Identical negative inputs carry no separating signal
    /// and are ignored; an entirely ambiguous batch is a no-op.
    pub fn train_contrastive(
        &mut self,
        query: EncoderInput,
        positive: EncoderInput,
        negatives: &[EncoderInput],
        learning_rate: f64,
    ) -> Result<f64, EncoderError> {
        let rate = Self::check_learning_rate(learning_rate)?;
        if query.kind != EncoderKind::Query
            || positive.kind != EncoderKind::Key
            || negatives.iter().any(|input| input.kind != EncoderKind::Key)
            || negatives.is_empty()
        {
            return Err(EncoderError::InvalidInput);
        }
        self.model.check_input(query)?;
        self.model.check_input(positive)?;
        let mut positive_signature = self.model.selected_rows(positive, 0)?.as_slice().to_vec();
        positive_signature.sort_unstable();
        let mut keys = Vec::with_capacity(negatives.len() + 1);
        keys.push(positive);
        let mut signatures = vec![positive_signature];
        for &negative in negatives {
            self.model.check_input(negative)?;
            let mut signature = self.model.selected_rows(negative, 0)?.as_slice().to_vec();
            signature.sort_unstable();
            // A2 coarse encoding is an unordered context bag. Different
            // evidence, prior state or token order cannot distinguish a
            // negative if its coarse selected rows are identical.
            if !signatures.contains(&signature) {
                keys.push(negative);
                signatures.push(signature);
            }
        }
        if keys.len() == 1 {
            return Ok(0.0);
        }

        // Only lane zero controls A2 admission. Fine-lane value differences
        // belong to the separately trained relative-energy ranker.
        let lanes = 1;
        let candidates = keys.len();
        let mut queries = Vec::with_capacity(lanes);
        let mut key_batches = Vec::with_capacity(lanes);
        let mut overlaps = Vec::with_capacity(lanes);
        let mut similarities = vec![0.0_f64; candidates];
        for lane in 0..lanes {
            let query_distribution = self.distribution_at(query, lane, CONTRASTIVE_TEMPERATURE)?;
            let mut key_distributions = Vec::with_capacity(candidates);
            let mut lane_overlaps = Vec::with_capacity(candidates);
            for (candidate, &key) in keys.iter().enumerate() {
                let distribution = self.distribution_at(key, lane, CONTRASTIVE_TEMPERATURE)?;
                let overlap = query_distribution
                    .0
                    .iter()
                    .zip(distribution.0.iter())
                    .map(|(q, k)| q * k)
                    .sum::<f64>();
                if !overlap.is_finite() {
                    return Err(EncoderError::InvalidLoss);
                }
                if lane == 0 {
                    similarities[candidate] += (overlap + OVERLAP_FLOOR).ln();
                }
                lane_overlaps.push(overlap);
                key_distributions.push(distribution);
            }
            queries.push(query_distribution);
            key_batches.push(key_distributions);
            overlaps.push(lane_overlaps);
        }
        let maximum = similarities
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let unnormalized: Vec<f64> = similarities
            .iter()
            .map(|score| (score - maximum).exp())
            .collect();
        let total = unnormalized.iter().sum::<f64>();
        if !total.is_finite() || total <= 0.0 {
            return Err(EncoderError::InvalidLoss);
        }
        let responsibilities: Vec<f64> = unnormalized.iter().map(|value| value / total).collect();
        let loss = maximum + total.ln() - similarities[0];
        let mut accumulated = BTreeMap::<usize, [f64; GROUP_ORDER]>::new();
        for lane in 0..lanes {
            let (query_probabilities, query_rows) = &queries[lane];
            let mut query_direct = [0.0_f64; GROUP_ORDER];
            let mut mean_key = [0.0_f64; GROUP_ORDER];
            for (key_probabilities, _) in &key_batches[lane] {
                for action in 0..GROUP_ORDER {
                    mean_key[action] += key_probabilities[action] / candidates as f64;
                }
            }
            for candidate in 0..candidates {
                let (key_probabilities, key_rows) = &key_batches[lane][candidate];
                let pair_weight = if lane == 0 {
                    (responsibilities[candidate] - f64::from(u8::from(candidate == 0)))
                        / (overlaps[lane][candidate] + OVERLAP_FLOOR)
                } else {
                    0.0
                };
                let mut key_direct = [0.0_f64; GROUP_ORDER];
                for action in 0..GROUP_ORDER {
                    query_direct[action] += pair_weight * key_probabilities[action];
                    // Negative batch mutual information: maximize marginal
                    // code use while keeping individual assignments sharp.
                    // Identical Key inputs give no artificial splitting signal.
                    let utilization = UTILIZATION_WEIGHT / candidates as f64
                        * (mean_key[action].max(OVERLAP_FLOOR).ln()
                            - key_probabilities[action].max(OVERLAP_FLOOR).ln());
                    key_direct[action] = pair_weight * query_probabilities[action] + utilization;
                }
                Self::accumulate_distribution_gradient(
                    &mut accumulated,
                    *key_rows,
                    key_probabilities,
                    &key_direct,
                    CONTRASTIVE_TEMPERATURE,
                )?;
            }
            Self::accumulate_distribution_gradient(
                &mut accumulated,
                *query_rows,
                query_probabilities,
                &query_direct,
                CONTRASTIVE_TEMPERATURE,
            )?;
        }
        self.apply_accumulated(accumulated, rate)?;
        Ok(loss)
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
            context: [0; CONTEXT_TOKENS],
            context_len: 0,
        }
    }

    #[test]
    fn a2_coarse_context_masks_anchor_and_counts_selected_rows() {
        let mut model = CodeEncoder::new(16, 4, 19).unwrap();
        model.context_addressing = true;
        assert!(model.validate().is_err());
        model.context_addressing = false;
        model.enable_context_addressing().unwrap();
        let mut key = input(3, EncoderKind::Key);
        key.context[..4].copy_from_slice(&[5, u16::MAX, 7, 9]);
        key.context_len = 4;
        key.evidence = Some(4);
        let mut query = key;
        query.kind = EncoderKind::Query;
        query.evidence = None;
        query.token = 12;
        query.previous = [8; MAX_LANES];
        let key_coarse = model.score_lane(key, 0).unwrap();
        let query_coarse = model.score_lane(query, 0).unwrap();
        assert_eq!(key_coarse.scores, query_coarse.scores);
        assert_eq!(key_coarse.coefficient_reads, 4 * 120);
        assert_eq!(model.score_lane(key, 1).unwrap().coefficient_reads, 7 * 120);
        assert!(model
            .score_lane(
                EncoderInput {
                    context: [u16::MAX; CONTEXT_TOKENS],
                    context_len: 33,
                    ..key
                },
                0,
            )
            .is_err());
    }

    #[test]
    fn a2_contrastive_source_training_reduces_pair_loss() {
        let mut model = CodeEncoder::new(16, 2, 23).unwrap();
        model.enable_context_addressing().unwrap();
        let mut trainer = EncoderTrainer::from_model(model).unwrap();
        let state_input = input(3, EncoderKind::State);
        let state_before = trainer.model().encode(state_input).unwrap();
        let mut query = input(0, EncoderKind::Query);
        query.context[..2].copy_from_slice(&[3, 4]);
        query.context_len = 2;
        let mut positive = input(0, EncoderKind::Key);
        positive.context[..2].copy_from_slice(&[3, 5]);
        positive.context_len = 2;
        let mut negative = input(0, EncoderKind::Key);
        negative.context[..2].copy_from_slice(&[8, 9]);
        negative.context_len = 2;
        let initial = trainer
            .train_contrastive(query, positive, &[negative], 1.0)
            .unwrap();
        let mut last = initial;
        for _ in 0..24 {
            last = trainer
                .train_contrastive(query, positive, &[negative], 1.0)
                .unwrap();
        }
        assert!(last < initial, "contrastive loss did not decrease");
        assert_eq!(trainer.model().encode(state_input).unwrap(), state_before);
        trainer.model().validate().unwrap();
    }

    fn hard_world(query_token: u16, positive_token: u16, negative_token: u16) -> HardAddressWorld {
        let mut query = input(0, EncoderKind::Query);
        query.context[0] = query_token;
        query.context_len = 1;
        let mut positive = input(0, EncoderKind::Key);
        positive.context[0] = positive_token;
        positive.context_len = 1;
        let mut negative = input(0, EncoderKind::Key);
        negative.context[0] = negative_token;
        negative.context_len = 1;
        HardAddressWorld {
            query,
            positive_index: 1,
            negative_indices: vec![0],
            committed_keys: vec![negative, positive],
        }
    }

    #[test]
    fn hard_address_search_edits_only_dedicated_integer_rows() {
        let mut model = CodeEncoder::new(16, 2, 29).unwrap();
        model.enable_context_addressing().unwrap();
        // Deliberately collapsed address bank: the wrong entity fills the
        // capacity-one page before the right source can be posted.
        model.address_coefficients.fill(0);
        let role_row = model
            .layout
            .row(model.layout.kind_base, EncoderKind::Key.index(), 0);
        let role_base = role_row << 7;
        model.coefficients[role_base..role_base + GROUP_ORDER].fill(0);
        model.validate().unwrap();
        let primary_before = model.coefficients.clone();
        let state = input(4, EncoderKind::State);
        let state_before = model.encode(state).unwrap();
        let pair = HardAddressPair {
            entity_group: 7,
            worlds: [hard_world(2, 2, 3), hard_world(2, 2, 3)],
        };
        let mut trainer = EncoderTrainer::from_model(model).unwrap();
        let config = HardAddressConfig {
            page_capacity: 1,
            max_pairs: 1,
            max_proposals: 2_000,
            max_candidate_evaluations: 250_000,
            max_accepted_edits: 100,
            max_duration_ms: 10_000,
            ..HardAddressConfig::default()
        };
        let report = trainer.train_hard_address_batch(&[pair], config).unwrap();
        assert!(report.before.objective > report.after.objective);
        assert!(report.accepted_edits > 0);
        assert!(report.changed_address_coefficients > 0);
        assert!(report.after.admitted_worlds >= report.before.admitted_worlds);
        assert!(report.candidate_evaluations <= config.max_candidate_evaluations);
        assert_eq!(trainer.model().coefficients, primary_before);
        assert_eq!(trainer.model().encode(state).unwrap(), state_before);
        trainer.model().validate().unwrap();
        // An accepted hard edit also resets its FP32 master to the exported
        // integer, so subsequent offline updates cannot revert a hidden bin.
        for (index, &coefficient) in trainer.model().address_coefficients.iter().enumerate() {
            let global = primary_before.len() + index;
            if coefficient != 0 {
                assert_eq!(trainer.master[global], f32::from(coefficient));
            }
        }
    }

    #[test]
    fn hard_address_skip_identical_negative_and_reject_invalid_batch() {
        let mut model = CodeEncoder::new(16, 2, 31).unwrap();
        model.enable_context_addressing().unwrap();
        let mut trainer = EncoderTrainer::from_model(model).unwrap();
        let pair = HardAddressPair {
            entity_group: 0,
            worlds: [hard_world(2, 2, 2), hard_world(2, 2, 2)],
        };
        let report = trainer
            .train_hard_address_batch(
                &[pair.clone()],
                HardAddressConfig {
                    max_pairs: 1,
                    max_proposals: 12,
                    max_candidate_evaluations: 10_000,
                    max_accepted_edits: 12,
                    max_duration_ms: 1_000,
                    ..HardAddressConfig::default()
                },
            )
            .unwrap();
        assert_eq!(report.skipped_identical_negatives, 2);
        assert_eq!(report.before.wrong_entity_collisions, 0);
        let mut bad = pair;
        bad.worlds[0].positive_index = 2;
        assert!(matches!(
            trainer.train_hard_address_batch(&[bad], HardAddressConfig::default()),
            Err(EncoderError::InvalidHardBatch)
        ));
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
