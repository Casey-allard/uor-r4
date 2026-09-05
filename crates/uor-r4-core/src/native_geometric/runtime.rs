//! Integer/table prediction kernel. Geometry is compiled before sessions exist.
//! No floating-point values, matrix products, transcendental operations, or
//! external model calls occur in observe/predict. Buffers are allocated once
//! when a session is created; candidate work is bounded by artifact postings.

use super::{Candidate, Control, Error, Feature, Model, Prediction, Result, Work, PHASE_CHANNELS};
use serde::{Deserialize, Serialize};

pub(super) const FEATURE_COUNT: usize = 26;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<super::ResponseStateView>,
    pub memory_read: Option<super::MemoryStateView>,
    pub tokens_seen: u64,
    pub retained_tokens: usize,
    pub context_capacity: usize,
    pub h4_index: u16,
    pub previous_h4_index: u16,
    pub phase_turns_u16: [u16; PHASE_CHANNELS],
    /// Exact window sum [a0,a1,a2,a3,b0,b1,b2,b3] in the canonical paired basis.
    pub paired_h4_coefficients: [i64; 8],
    /// Exact norm squared of the additive window carrier: (A+B*phi)/4.
    /// This is not a varying radius of the unit H4 group element.
    pub radial_squared_zphi_numerator: [i64; 2],
    pub control: Control,
    pub ring_storage_bytes: usize,
    pub candidate_storage_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub(super) memory: Option<super::memory_types::MemoryState>,
    pub(super) artifact_cid: String,
    pub(super) ring: Vec<u32>,
    pub(super) cursor: usize,
    pub(super) length: usize,
    pub(super) h4: u16,
    pub(super) previous_h4: u16,
    pub(super) previous_evicted: Option<u32>,
    pub(super) phases: [u16; PHASE_CHANNELS],
    pub(super) paired_coefficients: [i64; 8],
    pub(super) radial: [i64; 2],
    pub(super) candidates: Vec<Candidate>,
    candidate_storage_bytes: usize,
    pub(super) control: Control,
    pub work: Work,
}

// Host boundary: allocation/session construction and diagnostic accessors.
// Serialization/checkpoint restoration lives in snapshot.rs; fitting and
// geometry-table construction live in training.rs, mixture.rs and anchors.rs.
impl Session {
    pub(super) fn new(model: &Model, control: Control) -> Self {
        let candidates = Vec::with_capacity(model.config.candidate_limit);
        let candidate_storage_bytes = candidates
            .capacity()
            .saturating_mul(std::mem::size_of::<Candidate>());
        Self {
            memory: model
                .memory_read
                .as_ref()
                .map(|memory| super::memory_types::MemoryState::new(model, memory)),
            artifact_cid: model.artifact_cid.clone(),
            ring: vec![0; model.config.context_tokens],
            cursor: 0,
            length: 0,
            h4: model.geometry.identity,
            previous_h4: model.geometry.identity,
            previous_evicted: None,
            phases: [0; PHASE_CHANNELS],
            paired_coefficients: [0; 8],
            radial: [0; 2],
            candidates,
            candidate_storage_bytes,
            control,
            work: Work::default(),
        }
    }

    pub fn state(&self) -> StateView {
        StateView {
            response: self.memory.as_ref().and_then(|state| state.response_view()),
            memory_read: self.memory.as_ref().map(|state| state.state()),
            tokens_seen: self.work.observed_tokens,
            retained_tokens: self.length,
            context_capacity: self.ring.len(),
            h4_index: self.h4,
            previous_h4_index: self.previous_h4,
            phase_turns_u16: self.phases,
            paired_h4_coefficients: self.paired_coefficients,
            radial_squared_zphi_numerator: self.radial,
            control: self.control,
            ring_storage_bytes: std::mem::size_of_val(self.ring.as_slice()),
            candidate_storage_bytes: self.candidate_storage_bytes,
        }
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    // NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
    // The source guard covers this region through gate_eighths, plus the
    // Feature methods called here. Keep new kernel helpers in a scanned region.
    /// Most recently predicted response action. This is transient; only an
    /// observation can commit the selected occurrence to response state.
    pub fn response_decision(&self) -> Option<super::ResponseDecision> {
        self.memory.as_ref().and_then(|state| state.pending)
    }

    fn check_model(&self, model: &Model) -> Result<()> {
        if self.artifact_cid != model.artifact_cid {
            return Err(Error(
                "session belongs to a different native geometric artifact".into(),
            ));
        }
        Ok(())
    }

    pub fn begin_response(&mut self, model: &Model) -> Result<()> {
        self.check_model(model)?;
        if self.control != Control::MemoryDisabled && self.control != Control::ResponseStateDisabled
        {
            if let (Some(state), Some(memory)) = (&mut self.memory, &model.memory_read) {
                state.begin_response(model, memory, &mut self.work);
            }
        }
        Ok(())
    }

    pub fn end_response(&mut self, model: &Model) -> Result<()> {
        self.check_model(model)?;
        if let Some(state) = &mut self.memory {
            state.end_response();
        }
        Ok(())
    }

    fn product(&mut self, model: &Model, left: u16, right: u16) -> u16 {
        self.work.h4_table_reads = self.work.h4_table_reads.saturating_add(1);
        model.geometry.products[model.geometry.row_bases[usize::from(left)] + usize::from(right)]
    }

    /// Causal append. Sliding eviction removes the oldest left factor from
    /// the ordered group fold; phase removal/addition is modular integer work.
    pub fn observe(&mut self, model: &Model, token: u32) -> Result<()> {
        self.check_model(model)?;
        let Some(next) = model.geometry.tokens.get(token as usize) else {
            return Err(Error(
                "observed token is outside the artifact vocabulary".into(),
            ));
        };
        self.previous_h4 = self.h4;
        self.previous_evicted = None;
        if self.length == self.ring.len() {
            self.previous_evicted = Some(self.ring[self.cursor]);
            let old = &model.geometry.tokens[self.ring[self.cursor] as usize];
            let inverse = model.geometry.inverses[usize::from(old.leaf)];
            self.work.h4_table_reads = self.work.h4_table_reads.saturating_add(1);
            self.h4 = self.product(model, inverse, self.h4);
            for (sum, coefficient) in self
                .paired_coefficients
                .iter_mut()
                .zip(model.geometry.anchors.rows[usize::from(old.leaf)].paired_coefficients)
            {
                *sum -= coefficient;
            }
            self.work.anchor_table_reads = self.work.anchor_table_reads.saturating_add(1);
            for (phase, delta) in self.phases.iter_mut().zip(old.phases) {
                *phase = phase.wrapping_sub(delta);
                self.work.phase_additions = self.work.phase_additions.saturating_add(1);
            }
            self.work.evictions = self.work.evictions.saturating_add(1);
        } else {
            self.length += 1;
        }
        self.h4 = self.product(model, self.h4, next.leaf);
        for (sum, coefficient) in self
            .paired_coefficients
            .iter_mut()
            .zip(model.geometry.anchors.rows[usize::from(next.leaf)].paired_coefficients)
        {
            *sum += coefficient;
        }
        self.work.anchor_table_reads = self.work.anchor_table_reads.saturating_add(1);
        self.radial = [0; 2];
        for axis in 0..4 {
            let a = self.paired_coefficients[axis];
            let b = self.paired_coefficients[axis + 4];
            let aa = model.geometry.squares[(model.geometry.square_offset + a) as usize];
            let bb = model.geometry.squares[(model.geometry.square_offset + b) as usize];
            let combined = model.geometry.squares[(model.geometry.square_offset + a + b) as usize];
            self.radial[0] += aa + bb;
            self.radial[1] += combined - aa;
            self.work.radial_square_reads = self.work.radial_square_reads.saturating_add(3);
        }
        for (phase, delta) in self.phases.iter_mut().zip(next.phases) {
            *phase = phase.wrapping_add(delta);
            self.work.phase_additions = self.work.phase_additions.saturating_add(1);
        }
        self.ring[self.cursor] = token;
        self.cursor += 1;
        if self.cursor == self.ring.len() {
            self.cursor = 0;
        }
        self.work.observed_tokens = self.work.observed_tokens.saturating_add(1);
        if let (Some(state), Some(memory)) = (&mut self.memory, &model.memory_read) {
            state.observe(model, memory, token, &mut self.work);
        }
        Ok(())
    }

    fn recent(&self, distance: usize) -> u32 {
        if self.length < distance {
            return super::BOS;
        }
        let index = if self.cursor >= distance {
            self.cursor - distance
        } else {
            self.ring.len() - (distance - self.cursor)
        };
        self.ring[index]
    }

    pub(super) fn features(&self, model: &Model) -> [Feature; FEATURE_COUNT] {
        let last = model.geometry.tokens[self.recent(1) as usize].prime;
        let previous = model.geometry.tokens[self.recent(2) as usize].prime;
        let mut features = [Feature { kind: 0, value: 0 }; FEATURE_COUNT];
        let anchor = &model.geometry.anchors.rows[usize::from(self.h4)];
        features[0] = Feature {
            kind: 0,
            value: u64::from(last),
        };
        features[1] = Feature {
            kind: 1,
            value: (u64::from(previous) << 32) | u64::from(last),
        };
        features[2] = Feature {
            kind: 2,
            value: u64::from(self.h4),
        };
        features[3] = Feature {
            kind: 3,
            value: (u64::from(self.previous_h4) << 16) | u64::from(self.h4),
        };
        features[4] = Feature {
            kind: 4,
            value: u64::from(model.geometry.orientation[usize::from(self.h4)]),
        };
        features[5] = Feature {
            kind: 5,
            value: (u64::from(self.h4) << 4) | u64::from(self.phases[0] >> 12),
        };
        features[6] = Feature {
            kind: 6,
            value: u64::from(anchor.paired_class),
        };
        features[7] = Feature {
            kind: 7,
            value: ((self.radial[0] as u64) << 32) | u64::from(self.radial[1] as i32 as u32),
        };
        for (index, phase) in self.phases.iter().enumerate() {
            features[index + 8] = Feature {
                kind: 8 + index as u8,
                value: u64::from(*phase >> 12),
            };
        }
        for (index, &coefficient) in self.paired_coefficients.iter().enumerate() {
            features[index + 16] = Feature {
                kind: 16 + index as u8,
                value: coefficient as u64,
            };
        }
        features[24] = Feature {
            kind: 24,
            value: u64::from(anchor.heatmap_class),
        };
        features[25] = Feature {
            kind: 25,
            value: u64::from(anchor.projection_radius_class),
        };
        features
    }

    fn score_candidate(
        &mut self,
        model: &Model,
        token: u32,
        rows: &[usize],
        gates: &[u8; 7],
    ) -> Candidate {
        let prior = model.prior_scores[token as usize];
        let mut score = i64::from(prior);
        let mut groups = [0_i64; 7];
        for &row_index in rows {
            let row = &model.rows[row_index];
            self.work.score_lookups = self.work.score_lookups.saturating_add(1);
            let conditional = match row.scores.binary_search_by_key(&token, |item| item.token) {
                Ok(index) => row.scores[index].score,
                Err(_) => row.default_score,
            };
            groups[row.feature.group()] +=
                (i64::from(conditional) - i64::from(prior)) >> row.feature.shift();
        }
        for (value, &gate) in groups.into_iter().zip(gates) {
            score += gate_eighths(value, gate);
        }
        self.work.candidate_evaluations = self.work.candidate_evaluations.saturating_add(1);
        Candidate { token, score }
    }

    fn offer(&mut self, model: &Model, token: u32, rows: &[usize], gates: &[u8; 7]) {
        self.work.candidate_offers = self.work.candidate_offers.saturating_add(1);
        if token == super::BOS || self.candidates.iter().any(|item| item.token == token) {
            return;
        }
        let candidate = self.score_candidate(model, token, rows, gates);
        let position = self.candidates.partition_point(|item| {
            item.score > candidate.score
                || (item.score == candidate.score && item.token < candidate.token)
        });
        if position >= model.config.candidate_limit {
            return;
        }
        if self.candidates.len() == model.config.candidate_limit {
            self.candidates.pop();
        }
        self.candidates.insert(position, candidate);
    }

    /// Scores only candidates offered by the finite feature-posting lists.
    /// This is a bounded shortlist approximation, with coverage reported by
    /// evaluation. The model never scans the vocabulary or retained prefix.
    pub fn predict(&mut self, model: &Model) -> Result<Prediction> {
        self.check_model(model)?;
        let features = self.features(model);
        let gates = match model
            .readout
            .queries
            .binary_search_by_key(&(features[0].value as u32), |query| query.prime)
        {
            Ok(index) => model.readout.queries[index].weights,
            Err(_) => model.readout.global,
        };
        self.work.mixture_gate_reads = self.work.mixture_gate_reads.saturating_add(1);
        self.work.orientation_table_reads = self.work.orientation_table_reads.saturating_add(1);
        self.work.anchor_table_reads = self.work.anchor_table_reads.saturating_add(1);
        let mut row_indices = [0_usize; FEATURE_COUNT];
        let mut row_count = 0;
        let mut geometric_rows = 0;
        for feature in features {
            if !feature.admitted(self.control) {
                continue;
            }
            self.work.feature_queries = self.work.feature_queries.saturating_add(1);
            if let Ok(index) = model.rows.binary_search_by_key(&feature, |row| row.feature) {
                row_indices[row_count] = index;
                row_count += 1;
                geometric_rows += usize::from(feature.kind >= 2);
                self.work.matched_rows = self.work.matched_rows.saturating_add(1);
            }
        }
        let rows = &row_indices[..row_count];
        self.candidates.clear();
        for &token in &model.prior_postings {
            self.offer(model, token, rows, &gates);
        }
        for &index in rows {
            for &token in &model.rows[index].postings {
                self.offer(model, token, rows, &gates);
            }
        }
        if self.control != Control::MemoryDisabled {
            let occurrence_composition = model.memory_read.as_ref().is_some_and(|memory| {
                memory.schema == super::memory_types::OCCURRENCE_MEMORY_SCHEMA
                    || memory.schema == super::memory_types::RESPONSE_MEMORY_SCHEMA
            });
            if let (Some(state), Some(memory)) = (&mut self.memory, &model.memory_read) {
                state.collect(model, memory, self.control, &mut self.work);
            }
            let memory_count = self
                .memory
                .as_ref()
                .map(|state| {
                    if occurrence_composition {
                        state.composed.len()
                    } else {
                        state.candidates.len()
                    }
                })
                .unwrap_or(0);
            for index in 0..memory_count {
                if let Some(state) = &self.memory {
                    let candidate = if occurrence_composition {
                        let candidate = state.composed[index];
                        Candidate {
                            token: candidate.token,
                            score: candidate.score,
                        }
                    } else {
                        let candidate = state.candidates[index];
                        Candidate {
                            token: candidate.token,
                            score: candidate.score,
                        }
                    };
                    self.offer_memory(model, candidate);
                }
            }
        }
        let best = *self
            .candidates
            .first()
            .ok_or_else(|| Error("artifact offers no output candidates".into()))?;
        if let Some(state) = &mut self.memory {
            state.select_response(model, best, &mut self.work);
        }
        Ok(Prediction {
            token: best.token,
            score: best.score,
            candidate_count: self.candidates.len(),
            geometric_rows,
        })
    }

    fn offer_memory(&mut self, model: &Model, candidate: Candidate) {
        if let Some(index) = self
            .candidates
            .iter()
            .position(|known| known.token == candidate.token)
        {
            if self.candidates[index].score >= candidate.score {
                return;
            }
            self.candidates.remove(index);
        }
        let position = self.candidates.partition_point(|known| {
            known.score > candidate.score
                || (known.score == candidate.score && known.token < candidate.token)
        });
        if position >= model.config.candidate_limit {
            return;
        }
        if self.candidates.len() == model.config.candidate_limit {
            self.candidates.pop();
        }
        self.candidates.insert(position, candidate);
    }
}

/// A quantized learned gate in eighths, 0..=16. The bounded five-bit
/// coefficient is expanded using shifts and integer additions only.
fn gate_eighths(value: i64, gate: u8) -> i64 {
    let mut total = 0_i64;
    if gate & 1 != 0 {
        total += value;
    }
    if gate & 2 != 0 {
        total += value << 1;
    }
    if gate & 4 != 0 {
        total += value << 2;
    }
    if gate & 8 != 0 {
        total += value << 3;
    }
    if gate & 16 != 0 {
        total += value << 4;
    }
    total >> 3
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
