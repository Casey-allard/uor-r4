//! Integer/table prediction kernel. Geometry is compiled before sessions exist.
//! No floating-point values, matrix products, transcendental operations, or
//! external model calls occur in observe/predict. Buffers are allocated once
//! when a session is created; candidate work is bounded by artifact postings.

use super::hopf_metric::HopfFiberPointQ30;
use super::vsa::{Codebook, Hypervector4096, Shortlist};
use super::{
    Candidate, Control, Error, Feature, Model, Prediction, Result, WordCopyProgress, Work, BOS,
    PHASE_CHANNELS,
};
use serde::{Deserialize, Serialize};

pub(super) const FEATURE_COUNT: usize = 26;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_copy: Option<super::WordCopyStateView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_entry: Option<super::ResponseEntryStateView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion: Option<super::CompletionStateView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<super::ValueStateView>,
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
    #[serde(default)]
    pub syntactic_state: u8,
}

/// Active runtime session for native geometric autoregressive inference.
pub type ActiveSession = Session;

#[derive(Debug, Clone)]
pub struct Session {
    pub(super) field_composition: Option<super::field_composition::FieldState>,
    pub(super) routing_decision: Option<super::RoutingDecision>,
    pub(super) word_copy: Option<super::word_copy_types::WordCopyState>,
    pub(super) response_entry: Option<super::response_entry_types::ResponseEntryState>,
    pub(super) completion: Option<super::completion_types::CompletionState>,
    pub(super) values: Option<super::value_types::ValueState>,
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
    pub(super) syntactic_state: u8,
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
            field_composition: model.field_composition.as_ref().map(|_| Default::default()),
            routing_decision: None,
            word_copy: model
                .response_entry
                .as_ref()
                .and_then(|entry| entry.copy.as_ref())
                .map(|_| Default::default()),
            response_entry: model.response_entry.as_ref().map(|_| Default::default()),
            completion: model.completion.as_ref().map(|_| Default::default()),
            values: model
                .values
                .as_ref()
                .map(|_| super::value_types::ValueState::new(model)),
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
            syntactic_state: 0,
        }
    }

    pub fn state(&self) -> StateView {
        StateView {
            word_copy: self
                .word_copy
                .as_ref()
                .map(|state| super::WordCopyStateView {
                    origin: state.origin,
                    progress: state.progress,
                    storage_bytes: std::mem::size_of::<super::word_copy_types::WordCopyState>(),
                }),
            response_entry: self
                .response_entry
                .as_ref()
                .map(|s| super::ResponseEntryStateView {
                    active: s.active,
                    boundary_seen: s.boundary.map(|anchor| anchor.at_seen),
                    steps: s.steps,
                    last_action: s.last_action,
                    storage_bytes: std::mem::size_of::<
                        super::response_entry_types::ResponseEntryState,
                    >(),
                }),
            completion: self
                .completion
                .as_ref()
                .map(|s| super::CompletionStateView {
                    active: s.active,
                    write_id: s.anchor.map(|a| a.write_id),
                    steps: s.steps,
                    last_action: s.last_action,
                    storage_bytes: std::mem::size_of::<super::completion_types::CompletionState>(),
                }),
            values: self.values.as_ref().map(|s| super::ValueStateView {
                active: s.active,
                retained_values: s.records.len(),
                captured_values: s.sources.len(),
                committed_write: s.emission.map(|e| e.decision.write_id),
                emission_cursor: s.emission.map(|e| e.cursor),
                storage_bytes: std::mem::size_of::<super::value_types::ValueState>()
                    + (s.records.capacity() + s.sources.capacity())
                        * std::mem::size_of::<super::ValueRecord>(),
            }),
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
            syntactic_state: self.syntactic_state,
        }
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    const INDUCTION_BONUS_TABLE: [i64; 64] = [
        0, 4096, 2048, 1365, 1024, 819, 682, 585, 512, 455, 409, 372, 341, 315, 292, 273, 256, 240,
        227, 215, 204, 195, 186, 178, 170, 163, 157, 151, 146, 141, 136, 132, 128, 124, 120, 117,
        113, 110, 107, 105, 102, 99, 97, 95, 93, 91, 89, 87, 85, 83, 81, 80, 78, 77, 75, 74, 73,
        71, 70, 69, 68, 67, 66, 65,
    ];

    // NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
    // The source guard covers this region through gate_eighths, plus the
    // Feature methods called here. Keep new kernel helpers in a scanned region.
    #[inline]
    pub fn syntactic_state(&self) -> u8 {
        self.syntactic_state
    }

    #[inline]
    pub fn quotation_parity(&self) -> u8 {
        self.syntactic_state & 1
    }

    #[inline]
    pub fn clause_depth(&self) -> u8 {
        (self.syntactic_state >> 1) & 3
    }

    #[inline]
    fn update_syntactic_state(&mut self, model: &Model, token: u32) {
        let mut q = self.syntactic_state & 1;
        let mut d = (self.syntactic_state >> 1) & 3;

        let byte;
        let bytes: &[u8] = if token < super::LEXICAL_BASE {
            if (2..=257).contains(&token) {
                byte = [(token - 2) as u8];
                &byte[..]
            } else {
                &[]
            }
        } else {
            let idx = (token - super::LEXICAL_BASE) as usize;
            model.lexical_pieces.get(idx).map(|p| &p[..]).unwrap_or(&[])
        };

        for &b in bytes {
            match b {
                b'"' => {
                    q ^= 1;
                }
                b'(' | b'[' | b'{' => {
                    d = (d + 1).min(3);
                }
                b')' | b']' | b'}' => {
                    d = d.saturating_sub(1);
                }
                b',' | b';' | b':' => {
                    if d == 0 {
                        d = 1;
                    }
                }
                b'.' | b'?' | b'!' => {
                    d = 0;
                }
                _ => {}
            }
        }

        self.syntactic_state = (q & 1) | ((d & 3) << 1);
    }
    pub fn routing_decision(&self) -> Option<super::RoutingDecision> {
        self.routing_decision
    }

    /// Most recently predicted response action. This is transient; only an
    /// observation can commit the selected occurrence to response state.
    pub fn response_decision(&self) -> Option<super::ResponseDecision> {
        self.memory.as_ref().and_then(|state| state.pending)
    }

    pub fn value_decision(&self) -> Option<super::ValueDecision> {
        self.values.as_ref().and_then(|s| s.pending)
    }

    pub fn completion_decision(&self) -> Option<super::CompletionDecision> {
        self.completion.as_ref().and_then(|s| s.pending)
    }

    pub fn response_entry_decision(&self) -> Option<super::ResponseEntryDecision> {
        self.response_entry.as_ref().and_then(|state| state.pending)
    }

    pub fn field_composition_decision(&self) -> Option<super::FieldDecision> {
        self.field_composition.as_ref().and_then(|s| s.pending)
    }

    pub fn word_copy_decision(&self) -> Option<super::WordCopyDecision> {
        self.word_copy.as_ref().and_then(|state| state.pending)
    }

    /// Whether a response-specific core state is active, including after restore.
    /// Hosts use this to distinguish a new input turn from another prefill chunk.
    pub fn is_response_active(&self) -> bool {
        self.values.as_ref().is_some_and(|s| s.active)
            || self.completion.as_ref().is_some_and(|s| s.active)
            || self.response_entry.as_ref().is_some_and(|s| s.active)
            || self
                .memory
                .as_ref()
                .and_then(|s| s.response.as_ref())
                .is_some_and(|s| s.active)
    }

    /// A completed/restored response may be inactive while its captured query
    /// still needs closing before new input. Prefill chunks have no capture.
    pub fn needs_input_boundary(&self) -> bool {
        self.is_response_active()
            || self.values.as_ref().is_some_and(|s| s.query_len != 0)
            || self
                .memory
                .as_ref()
                .and_then(|s| s.response.as_ref())
                .is_some_and(|s| !s.queries.is_empty())
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
        if let Some(field) = &mut self.field_composition {
            *field = Default::default();
        }
        if let Some(copy) = &mut self.word_copy {
            copy.reset();
        }
        if let Some(state) = &mut self.completion {
            state.reset();
        }
        if let Some(state) = &mut self.values {
            state.begin(&mut self.work.values);
            if let Some(block) = &model.operation_transition {
                state.max_operations = block.max_operations;
            }
            state.observe_relation_with_control(model, self.control, &mut self.work.values);
            if let Some(relations) = &mut state.relations {
                relations.finish_span(&mut self.work.values);
            }
        }
        if let (Some(entry), Some(values)) = (&mut self.response_entry, &self.values) {
            entry.begin(model, values, self.control, &mut self.work.response_entry);
        }
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
        if let Some(field) = &mut self.field_composition {
            *field = Default::default();
        }
        if let Some(copy) = &mut self.word_copy {
            copy.reset();
        }
        if let Some(entry) = &mut self.response_entry {
            entry.reset();
        }
        if let Some(state) = &mut self.completion {
            state.reset();
        }
        if let Some(state) = &mut self.values {
            state.end();
        }
        if let Some(state) = &mut self.memory {
            state.end_response();
        }
        Ok(())
    }

    pub fn refresh_value_sources(&mut self, model: &Model) -> Result<()> {
        self.check_model(model)?;
        if let Some(state) = &mut self.values {
            state.refresh_sources(&mut self.work.values);
        }
        if let Some(state) = &mut self.completion {
            state.reset();
        }
        Ok(())
    }

    pub fn can_transition(&self) -> bool {
        self.values.as_ref().is_some_and(|v| v.can_transition())
    }

    pub fn maybe_transition(&mut self, model: &Model) -> Result<bool> {
        self.check_model(model)?;
        // Learned operation transitions are offered by predict and committed
        // by observe. This legacy entry never advances state independently.
        Ok(false)
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
        if let Some(state) = &mut self.values {
            let starting_operator = model.operation_transition.is_some()
                && state
                    .pending
                    .is_some_and(|d| d.cursor == 0 && d.token == token && d.at_seen == state.seen);
            if starting_operator {
                if let Some(completion) = &mut self.completion {
                    completion.reset();
                }
            }
            let seed = state
                .pending
                .as_ref()
                .map(super::completion_types::CompletionSeed::from);
            state.observe_with_control(model, token, self.control, &mut self.work.values);
            if let Some(entry) = &mut self.response_entry {
                entry.observe(
                    model,
                    state,
                    token,
                    self.control,
                    &mut self.work.response_entry,
                );
                if let Some(fields) = &mut self.field_composition {
                    fields.observe(model, entry, state, token, &mut self.work.word_copy);
                }
                if let Some(copy) = &mut self.word_copy {
                    copy.observe(entry, state, token, &mut self.work.word_copy);
                }
            }
            if let Some(completion) = &mut self.completion {
                completion.observe(
                    model,
                    state,
                    token,
                    seed,
                    self.control,
                    &mut self.work.completion,
                );
            }
        }
        self.update_syntactic_state(model, token);
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
        prose_fiber: Option<HopfFiberPointQ30>,
        vsa_context: Option<(&Codebook<64>, &Hypervector4096)>,
    ) -> Candidate {
        let prior = model.prior_scores.get(token as usize).copied().unwrap_or(0);
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
            score = score.saturating_add(gate_eighths(value, gate));
        }
        if let (Some(block), Some(decision)) = (&model.learned_routing, self.routing_decision) {
            score = score.saturating_add(block.score(
                model,
                decision,
                token,
                &mut self.work.learned_routing,
            ));
        }
        if let Some(tables) = &model.geometric_prose_tables {
            let cand_idx = token as usize;
            let root_len = tables.token_to_root.len();
            let cand_root = if root_len > 0 {
                tables.token_to_root[cand_idx.min(root_len - 1)] as usize
            } else {
                0
            };
            score = score.saturating_add(i64::from(
                tables.discrete_bias.get(cand_idx).copied().unwrap_or(0),
            ));

            for (l, table) in tables.discrete_tables.iter().enumerate() {
                let lag = l + 1;
                if self.length >= lag {
                    let ctx_token = self.recent(lag) as usize;
                    let ctx_root = if root_len > 0 {
                        tables.token_to_root[ctx_token.min(root_len - 1)] as usize
                    } else {
                        0
                    };
                    score = score.saturating_add(i64::from(table.score(ctx_root, cand_root)));
                }
            }

            if let Some(fiber) = prose_fiber {
                score = score.saturating_add(i64::from(tables.score_readout(cand_idx, fiber)));
            }

            if let Some((codebook, vsa_vec)) = vsa_context {
                score = score.saturating_add(i64::from(
                    tables.score_vsa_candidate(codebook, vsa_vec, token),
                ));
            }

            if let Some(engram) = &tables.engram_table {
                if self.length >= 3 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    if let Some(cands) = engram.lookup_trigram(w_prev2, w_prev, w_curr) {
                        for &(c, q15) in cands {
                            if c == token {
                                score = score.saturating_add(i64::from(q15));
                            }
                        }
                    }
                }
                if self.length >= 2 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    if let Some(cands) = engram.lookup_bigram(w_prev, w_curr) {
                        for &(c, q15) in cands {
                            if c == token {
                                score = score.saturating_add(i64::from(q15));
                            }
                        }
                    }
                }
                for &k in &[2, 4, 8] {
                    if self.length >= k + 1 {
                        let w_curr = self.recent(1);
                        let w_skip = self.recent(k + 1);
                        if let Some(cands) =
                            engram.lookup_skip(w_skip, w_curr, self.syntactic_state)
                        {
                            for &(c, q15) in cands {
                                if c == token {
                                    score = score.saturating_add(i64::from(q15));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(lattice) = &tables.hierarchical_lattice {
                if root_len > 0 {
                    let curr_tok = if self.length >= 1 {
                        self.recent(1) as usize
                    } else {
                        usize::MAX
                    };
                    let is_self = cand_idx == curr_tok;
                    if self.length >= 2 {
                        let prev_tok = self.recent(2) as usize;
                        let r_prev = tables.token_to_root[prev_tok.min(root_len - 1)] as usize;
                        let r_curr = tables.token_to_root[curr_tok.min(root_len - 1)] as usize;
                        let c_curr = lattice.cluster_of(curr_tok);
                        let c_cand = lattice.cluster_of(cand_idx);
                        score = score.saturating_add(i64::from(
                            lattice.score_token(r_prev, r_curr, cand_root, c_curr, c_cand, is_self),
                        ));
                    } else if self.length == 1 {
                        let r_curr = tables.token_to_root[curr_tok.min(root_len - 1)] as usize;
                        let c_curr = lattice.cluster_of(curr_tok);
                        let c_cand = lattice.cluster_of(cand_idx);
                        score = score.saturating_add(i64::from(
                            lattice.score_token(r_curr, r_curr, cand_root, c_curr, c_cand, is_self),
                        ));
                    }
                }
            }

            // Exact Addressed Induction Attention (2-layer Previous-Token + Induction Circuit):
            // When current bigram [w_{t-1}, w_t] matches earlier bigram [w_{t-k-1}, w_{t-k}],
            // continuation token w_{t-k+1} receives addressed transition bonus from table.
            if self.length >= 8 {
                let b_curr = self.recent(1);
                let b_prev = self.recent(2);
                let max_k = self.length.min(64);
                for k in 6..max_k - 1 {
                    if self.recent(k + 2) == b_prev
                        && self.recent(k + 1) == b_curr
                        && self.recent(k) == token
                        && token != b_curr
                    {
                        score = score.saturating_add(Self::INDUCTION_BONUS_TABLE[k.min(63)]);
                        break;
                    }
                }
            }

            // DeltaScore_memory(v) = Score_memory(v) - Score_prior(v)
            if self.control != Control::MemoryDisabled {
                if let Some(state) = &self.memory {
                    let occurrence_composition = model.memory_read.as_ref().is_some_and(|memory| {
                        memory.schema == super::memory_types::OCCURRENCE_MEMORY_SCHEMA
                            || memory.schema == super::memory_types::RESPONSE_MEMORY_SCHEMA
                    });
                    let mem_score = if occurrence_composition {
                        state
                            .composed
                            .iter()
                            .find(|c| c.token == token)
                            .map(|c| c.score)
                    } else {
                        state
                            .candidates
                            .iter()
                            .find(|c| c.token == token)
                            .map(|c| c.score)
                    };
                    if let Some(score_val) = mem_score {
                        let mem_evidence = score_val.saturating_sub(i64::from(prior));
                        score = score.saturating_add(mem_evidence);
                    }
                }
            }
        }
        self.work.candidate_evaluations = self.work.candidate_evaluations.saturating_add(1);
        Candidate { token, score }
    }

    fn offer(
        &mut self,
        model: &Model,
        token: u32,
        rows: &[usize],
        gates: &[u8; 7],
        prose_fiber: Option<HopfFiberPointQ30>,
        vsa_context: Option<(&Codebook<64>, &Hypervector4096)>,
    ) {
        self.work.candidate_offers = self.work.candidate_offers.saturating_add(1);
        if token == super::BOS || self.candidates.iter().any(|item| item.token == token) {
            return;
        }
        let candidate = self.score_candidate(model, token, rows, gates, prose_fiber, vsa_context);
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
        self.routing_decision = None;
        self.work.word_copy.dispatch_checks = self
            .work
            .word_copy
            .dispatch_checks
            .saturating_add(u64::from(super::word_copy_runtime::composed(model)));
        if super::word_copy_runtime::composed(model)
            && self.control != Control::WordCopyDispatchDisabled
            && self
                .word_copy
                .as_ref()
                .is_some_and(|c| matches!(c.progress, WordCopyProgress::Emitting { .. }))
        {
            if let (Some(copy), Some(entry), Some(values)) =
                (&mut self.word_copy, &mut self.response_entry, &self.values)
            {
                // /4 has no causal response writer. No ordinary collection or
                // scoring is required for an immutable committed byte. Score1
                // is a dispatch marker, not a comparable model likelihood.
                entry.pending = None;
                if let Some(best) = copy.offer(
                    model,
                    entry,
                    values,
                    Candidate {
                        token: BOS,
                        score: 0,
                    },
                    None,
                    self.control,
                    &mut self.work.word_copy,
                ) {
                    self.candidates.clear();
                    if let Some(memory) = &mut self.memory {
                        memory.select_response(model, best, &mut self.work);
                    }
                    if let Some(values) = &mut self.values {
                        values.selected(best);
                    }
                    if let Some(completion) = &mut self.completion {
                        completion.selected(best);
                    }
                    entry.selected(best);
                    copy.selected(best);
                    self.work.word_copy.forced_dispatches =
                        self.work.word_copy.forced_dispatches.saturating_add(1);
                    return Ok(Prediction {
                        token: best.token,
                        score: best.score,
                        candidate_count: 1,
                        geometric_rows: 0,
                    });
                }
            }
        }
        if let Some(block) = &model.learned_routing {
            if !matches!(
                self.control,
                Control::LearnedRoutingDisabled | Control::GeometryDisabled
            ) {
                let mut context = [BOS; super::learned_routing::WINDOW];
                let count = self.length.min(context.len());
                let mut cursor = self.cursor;
                for token in context.iter_mut().take(count) {
                    if cursor == 0 {
                        cursor = self.ring.len();
                    }
                    cursor -= 1;
                    *token = self.ring[cursor];
                }
                self.routing_decision = Some(block.route(
                    model,
                    &context[..count],
                    self.control,
                    &mut self.work.learned_routing,
                ));
            }
        }
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
        let prose_fiber = model.geometric_prose_tables.as_ref().map(|tables| {
            let hist_fiber = tables.context_fiber_from_ring(&self.ring, self.cursor, self.length);
            if self.length > 0 {
                let w_t = self.recent(1);
                tables.predict_next_fiber(hist_fiber, w_t)
            } else {
                hist_fiber
            }
        });

        let (vsa_codebook, prose_vsa) = if let Some(tables) = &model.geometric_prose_tables {
            if tables.vsa_scale_q15 != 0 && self.length > 0 {
                let codebook = Codebook::<64>::on_demand(tables.vocab_size, tables.vsa_seed);
                let vsa_vec =
                    tables.context_vsa_from_ring(&codebook, &self.ring, self.cursor, self.length);
                (Some(codebook), Some(vsa_vec))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let vsa_context = match (&vsa_codebook, &prose_vsa) {
            (Some(c), Some(v)) => Some((c, v)),
            _ => None,
        };

        // If exact addressed memory is enabled, collect active memory candidates
        let occurrence_composition = model.memory_read.as_ref().is_some_and(|memory| {
            memory.schema == super::memory_types::OCCURRENCE_MEMORY_SCHEMA
                || memory.schema == super::memory_types::RESPONSE_MEMORY_SCHEMA
        });
        if self.control != Control::MemoryDisabled {
            if let (Some(state), Some(memory)) = (&mut self.memory, &model.memory_read) {
                state.collect(model, memory, self.control, &mut self.work);
            }
        }

        // Collect active memory candidate tokens from exact memory, relation records,
        // value completion, response entry, and copy buffers into a stack-allocated buffer (zero heap allocations).
        let mut memory_tokens = Shortlist::<64>::empty();
        if self.control != Control::MemoryDisabled {
            // 1. Highest priority: active word copy in progress or starting
            if let Some(copy) = &self.word_copy {
                if let Some(values) = &self.values {
                    if let Some(origin) = copy.origin {
                        if let Some(word) = super::relation::source(values, origin) {
                            let cur = match copy.progress {
                                WordCopyProgress::Emitting { cursor } => usize::from(cursor),
                                _ => 0,
                            };
                            let word_len =
                                usize::from(word.len).min(super::value_lexemes::WORD_BYTES);
                            if cur < word_len {
                                let tok = u32::from(word.bytes[cur]) + 2;
                                if (tok as usize) < model.geometry.tokens.len() {
                                    memory_tokens.push(tok);
                                }
                            }
                        }
                    }
                }
            }

            // 2. Active typed decisions (pending completion, response entry, values)
            if let Some(entry) = &self.response_entry {
                if let Some(pending) = entry.pending {
                    let tok = pending.token;
                    if tok != super::BOS && tok != 0 {
                        memory_tokens.push(tok);
                    }
                }
            }
            if let Some(completion) = &self.completion {
                if let Some(pending) = completion.pending {
                    let tok = pending.token;
                    if tok != super::BOS && tok != 0 {
                        memory_tokens.push(tok);
                    }
                }
            }
            if let Some(values) = &self.values {
                if let Some(pending) = &values.pending {
                    let tok = pending.token;
                    if tok != super::BOS && tok != 0 {
                        memory_tokens.push(tok);
                    }
                }
                if let Some(emission) = &values.emission {
                    let cur = usize::from(emission.cursor);
                    if cur < usize::from(emission.numeral.len) {
                        let tok = emission.numeral.tokens[cur];
                        memory_tokens.push(tok);
                    }
                }
            }

            // 3. Addressed memory recall candidates (up to 8 to preserve room)
            if let Some(state) = &self.memory {
                let count = if occurrence_composition {
                    state.composed.len()
                } else {
                    state.candidates.len()
                };
                for i in 0..count.min(8) {
                    let tok = if occurrence_composition {
                        state.composed[i].token
                    } else {
                        state.candidates[i].token
                    };
                    if tok != super::BOS && tok != 0 {
                        memory_tokens.push(tok);
                    }
                }
            }

            // 4. Exact relation records from values.relations (most recent records first)
            if let Some(relations) = self.values.as_ref().and_then(|v| v.relations.as_ref()) {
                let mut ids = [0u64; super::relation::RELATIONS];
                let mut id_count = 0;
                for &id in &relations.directory {
                    if id != 0 {
                        ids[id_count] = id;
                        id_count += 1;
                    }
                }
                ids[..id_count].sort_unstable_by(|a, b| b.cmp(a));

                for &id in &ids[..id_count] {
                    if memory_tokens.len >= 24 {
                        break;
                    }
                    if let Some(record) = relations.record(id) {
                        let len =
                            usize::from(record.value.len).min(super::value_lexemes::WORD_BYTES);
                        for &b in &record.value.bytes[..len] {
                            let tok = u32::from(b) + 2;
                            if (tok as usize) < model.geometry.tokens.len() {
                                memory_tokens.push(tok);
                            }
                        }
                        if let Some(span) = &record.span {
                            let span_len = usize::from(span.len).min(span.bytes.len());
                            for &b in &span.bytes[..span_len] {
                                let tok = u32::from(b) + 2;
                                if (tok as usize) < model.geometry.tokens.len() {
                                    memory_tokens.push(tok);
                                }
                            }
                        }
                    }
                }
            }
        }

        for &token in &model.prior_postings {
            self.offer(model, token, rows, &gates, prose_fiber, vsa_context);
        }
        for &index in rows {
            for &token in &model.rows[index].postings {
                self.offer(model, token, rows, &gates, prose_fiber, vsa_context);
            }
        }
        if let (Some(block), Some(decision)) = (&model.learned_routing, self.routing_decision) {
            if block.joint.is_none() {
                for (head, selected) in block.heads.iter().zip(decision.heads) {
                    for token in &head.emissions[usize::from(selected.output)].postings {
                        self.offer(model, *token, rows, &gates, prose_fiber, vsa_context);
                    }
                }
            }
        }
        if let Some(tables) = &model.geometric_prose_tables {
            // Unified multi-modal shortlist candidate formation:
            // 1. Exact Addressed Memory (M_exact): relation records, value completion, response entry, word copy
            let mut seed_shortlist = Shortlist::<64>::empty();
            for &tok in memory_tokens.as_slice() {
                seed_shortlist.push(tok);
            }

            // 2. Engram Collocation Table (E_colloc): high-frequency 5-gram, 4-gram, trigram, and bigram continuations
            if let Some(engram) = &tables.engram_table {
                if self.length >= 5 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    let w_prev3 = self.recent(4);
                    let w_prev4 = self.recent(5);
                    if let Some(cands) =
                        engram.lookup_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr)
                    {
                        for &(cand_tok, _) in cands {
                            if seed_shortlist.len >= 40 {
                                break;
                            }
                            seed_shortlist.push(cand_tok);
                        }
                    }
                }
                if self.length >= 4 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    let w_prev3 = self.recent(4);
                    if let Some(cands) = engram.lookup_4gram(w_prev3, w_prev2, w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            if seed_shortlist.len >= 40 {
                                break;
                            }
                            seed_shortlist.push(cand_tok);
                        }
                    }
                }
                if self.length >= 3 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    if let Some(cands) = engram.lookup_trigram(w_prev2, w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            if seed_shortlist.len >= 40 {
                                break;
                            }
                            seed_shortlist.push(cand_tok);
                        }
                    }
                }
                if self.length >= 2 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    if let Some(cands) = engram.lookup_bigram(w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            if seed_shortlist.len >= 40 {
                                break;
                            }
                            seed_shortlist.push(cand_tok);
                        }
                    }
                }
                for &k in &[2, 4, 8] {
                    if self.length > k {
                        let w_curr = self.recent(1);
                        let w_skip = self.recent(k + 1);
                        if let Some(cands) =
                            engram.lookup_skip(w_skip, w_curr, self.syntactic_state)
                        {
                            for &(cand_tok, _) in cands {
                                if seed_shortlist.len >= 40 {
                                    break;
                                }
                                seed_shortlist.push(cand_tok);
                            }
                        }
                    }
                }
                if self.quotation_parity() == 1 {
                    if (36 as usize) < model.geometry.tokens.len() && seed_shortlist.len < 40 {
                        seed_shortlist.push(36);
                    }
                    if let Ok(idx) = model
                        .lexical_pieces
                        .binary_search_by(|p| p.as_slice().cmp(b"\""))
                    {
                        let quote_token = super::LEXICAL_BASE + idx as u32;
                        if (quote_token as usize) < model.geometry.tokens.len()
                            && seed_shortlist.len < 40
                        {
                            seed_shortlist.push(quote_token);
                        }
                    }
                }
                if self.length >= 8 {
                    let b_curr = self.recent(1);
                    let b_prev = self.recent(2);
                    let max_k = self.length.min(64);
                    for k in 6..max_k - 1 {
                        if self.recent(k + 2) == b_prev && self.recent(k + 1) == b_curr {
                            let cont_tok = self.recent(k);
                            if cont_tok != b_curr
                                && (cont_tok as usize) < model.geometry.tokens.len()
                                && seed_shortlist.len < 40
                            {
                                seed_shortlist.push(cont_tok);
                                break;
                            }
                        }
                    }
                }
            }

            // 3. Hierarchical Voronoi Lattice (L_hier): coarse root trigram sectors and fine Hamming leaf clusters
            //    evaluated via forward-predicted geometric state s_hat_{t+1}.
            //    Seed shortlist has at most 40 items, ensuring at least 24 slots remain for Voronoi lattice clusters!
            let shortlist = if let Some(hierarchical) = &tables.hierarchical_codebook {
                let query_s3 = prose_fiber.map(|pf| pf.to_unit_s3_q30());
                hierarchical.route_shortlist_q30_with_memory::<64>(
                    query_s3,
                    prose_vsa.as_ref(),
                    seed_shortlist.as_slice(),
                )
            } else {
                let mut sl = Shortlist::<64>::empty();
                for &tok in seed_shortlist.as_slice() {
                    sl.push(tok);
                }
                let max_token = tables.vocab_size.min(model.geometry.tokens.len());
                for token in 0..max_token {
                    if sl.is_full() {
                        break;
                    }
                    sl.push(token as u32);
                }
                sl
            };

            for &token in shortlist.as_slice() {
                self.offer(model, token, rows, &gates, prose_fiber, vsa_context);
            }

            if let Some(engram) = &tables.engram_table {
                if self.length >= 5 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    let w_prev3 = self.recent(4);
                    let w_prev4 = self.recent(5);
                    if let Some(cands) =
                        engram.lookup_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr)
                    {
                        for &(cand_tok, _) in cands {
                            self.offer(model, cand_tok, rows, &gates, prose_fiber, vsa_context);
                        }
                    }
                }
                if self.length >= 4 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    let w_prev3 = self.recent(4);
                    if let Some(cands) = engram.lookup_4gram(w_prev3, w_prev2, w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            self.offer(model, cand_tok, rows, &gates, prose_fiber, vsa_context);
                        }
                    }
                }
                if self.length >= 3 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    let w_prev2 = self.recent(3);
                    if let Some(cands) = engram.lookup_trigram(w_prev2, w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            self.offer(model, cand_tok, rows, &gates, prose_fiber, vsa_context);
                        }
                    }
                }
                if self.length >= 2 {
                    let w_curr = self.recent(1);
                    let w_prev = self.recent(2);
                    if let Some(cands) = engram.lookup_bigram(w_prev, w_curr) {
                        for &(cand_tok, _) in cands {
                            self.offer(model, cand_tok, rows, &gates, prose_fiber, vsa_context);
                        }
                    }
                }
                for &k in &[2, 4, 8] {
                    if self.length > k {
                        let w_curr = self.recent(1);
                        let w_skip = self.recent(k + 1);
                        if let Some(cands) =
                            engram.lookup_skip(w_skip, w_curr, self.syntactic_state)
                        {
                            for &(cand_tok, _) in cands {
                                self.offer(model, cand_tok, rows, &gates, prose_fiber, vsa_context);
                            }
                        }
                    }
                }
                if self.quotation_parity() == 1 {
                    if (36 as usize) < model.geometry.tokens.len() {
                        self.offer(model, 36, rows, &gates, prose_fiber, vsa_context);
                    }
                    if let Ok(idx) = model
                        .lexical_pieces
                        .binary_search_by(|p| p.as_slice().cmp(b"\""))
                    {
                        let quote_token = super::LEXICAL_BASE + idx as u32;
                        if (quote_token as usize) < model.geometry.tokens.len() {
                            self.offer(model, quote_token, rows, &gates, prose_fiber, vsa_context);
                        }
                    }
                }
                if self.length >= 8 {
                    let b_curr = self.recent(1);
                    let b_prev = self.recent(2);
                    let max_k = self.length.min(64);
                    for k in 6..max_k - 1 {
                        if self.recent(k + 2) == b_prev && self.recent(k + 1) == b_curr {
                            let cont_tok = self.recent(k);
                            if cont_tok != b_curr
                                && (cont_tok as usize) < model.geometry.tokens.len()
                            {
                                self.offer(model, cont_tok, rows, &gates, prose_fiber, vsa_context);
                                break;
                            }
                        }
                    }
                }
            }
        }
        if self.control != Control::MemoryDisabled {
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
                    if model.geometric_prose_tables.is_some() {
                        let geom_cand = self.score_candidate(
                            model,
                            candidate.token,
                            rows,
                            &gates,
                            prose_fiber,
                            vsa_context,
                        );
                        self.offer_memory(model, geom_cand);
                    } else {
                        self.offer_memory(model, candidate);
                    }
                }
            }
        }
        // An active entry proves that the enabled typed gate offered no value
        // before the selected Enter token was observed. Its sources, query
        // tokens/words and model/control stay frozen throughout that response;
        // the changing pose/history and baseline score do not decide NoWrite.
        // Enter observation clears typed pending state, and restored active
        // entries independently recheck this gate. Boundaries/EOS/cap clear
        // entry activity, so their next prediction uses the ordinary gate.
        let suppressing_entry = self
            .response_entry
            .as_ref()
            .is_some_and(|entry| entry.active)
            && self.word_copy.as_ref().is_none_or(|copy| {
                copy.progress != WordCopyProgress::Idle || copy.origin.is_some()
            });
        if !suppressing_entry {
            if let Some(baseline) = self.candidates.first().copied() {
                let offer = self
                    .values
                    .as_mut()
                    .and_then(|s| s.offer(model, baseline, self.control, &mut self.work.values));
                if let Some(candidate) = offer {
                    self.offer_memory(model, candidate);
                }
            }
        }
        if let (Some(baseline), Some(state), Some(values)) = (
            self.candidates.first().copied(),
            &mut self.completion,
            &self.values,
        ) {
            let offer = state.offer(
                model,
                values,
                baseline,
                self.control,
                &mut self.work.completion,
            );
            if let Some(candidate) = offer {
                self.offer_memory(model, candidate);
            }
        }
        if let (Some(baseline), Some(entry), Some(values)) = (
            self.candidates.first().copied(),
            &mut self.response_entry,
            &self.values,
        ) {
            let copying = self.word_copy.as_ref().is_some_and(|copy| {
                matches!(
                    copy.progress,
                    super::WordCopyProgress::Emitting { .. } | super::WordCopyProgress::Complete
                )
            });
            let lexical = if copying {
                entry.pending = None;
                None
            } else {
                entry.offer(
                    model,
                    values,
                    baseline,
                    self.control,
                    &mut self.work.response_entry,
                )
            };
            let offered = if let Some(copy) = &mut self.word_copy {
                copy.offer(
                    model,
                    entry,
                    values,
                    baseline,
                    lexical,
                    self.control,
                    &mut self.work.word_copy,
                )
            } else {
                lexical
            };
            if let Some(candidate) = offered {
                self.offer_memory(model, candidate);
            }
        }
        let mut best = *self
            .candidates
            .first()
            .ok_or_else(|| Error("artifact offers no output candidates".into()))?;
        if let (Some(joint), Some(decision)) = (
            model
                .learned_routing
                .as_ref()
                .and_then(|b| b.joint.as_ref()),
            self.routing_decision,
        ) {
            // The joint decision sees the actual assembled parent winner,
            // including typed/copy/entry/EOS competition. It does not alter the
            // frozen parent parameters or unconditionally force an entry token.
            let flags = self.recurrent_flags();
            let keys = super::recurrent_routing::features(
                model,
                decision,
                best.token,
                flags,
                &mut self.work.learned_routing,
            );
            let (action, score, base_score) =
                joint.choose(keys, best.token, &mut self.work.learned_routing);
            if action != BOS {
                best = Candidate {
                    token: action,
                    score: best.score + score - base_score,
                };
                self.offer_memory(model, best);
            }
        }
        if let (Some(values), Some(completion)) = (&mut self.values, &self.completion) {
            if let Some(candidate) = super::operation_transition::offer(
                model,
                values,
                completion,
                best,
                self.control,
                &mut self.work.values,
            ) {
                best = candidate;
                self.offer_memory(model, candidate);
            }
        }
        if let (Some(fields), Some(copy), Some(entry), Some(values)) = (
            &mut self.field_composition,
            &mut self.word_copy,
            &mut self.response_entry,
            &self.values,
        ) {
            if let Some(candidate) = super::field_composition::offer(
                model,
                fields,
                copy,
                entry,
                values,
                best,
                self.control,
                &mut self.work.word_copy,
            ) {
                best = candidate;
                self.offer_memory(model, candidate);
            }
        }
        if let Some(state) = &mut self.memory {
            state.select_response(model, best, &mut self.work);
        }
        if let Some(state) = &mut self.values {
            state.selected(best);
        }
        if let Some(state) = &mut self.completion {
            state.selected(best);
        }
        if let Some(state) = &mut self.response_entry {
            state.selected(best);
        }
        if let Some(state) = &mut self.word_copy {
            state.selected(best);
        }
        if let Some(state) = &mut self.field_composition {
            state.selected(best);
        }
        Ok(Prediction {
            token: best.token,
            score: best.score,
            candidate_count: self.candidates.len(),
            geometric_rows,
        })
    }

    pub(super) fn recurrent_flags(&self) -> u32 {
        u32::from(
            self.response_entry
                .as_ref()
                .is_some_and(|s| s.boundary.is_some()),
        ) | (u32::from(self.response_entry.as_ref().is_some_and(|s| s.active)) << 1)
            | (u32::from(
                self.word_copy
                    .as_ref()
                    .is_some_and(|s| matches!(s.progress, WordCopyProgress::Complete)),
            ) << 2)
    }

    pub(super) fn recurrent_context(&self) -> ([u32; super::learned_routing::WINDOW], usize) {
        let mut context = [BOS; super::learned_routing::WINDOW];
        let count = self.length.min(context.len());
        let mut cursor = self.cursor;
        for token in context.iter_mut().take(count) {
            if cursor == 0 {
                cursor = self.ring.len();
            }
            cursor -= 1;
            *token = self.ring[cursor];
        }
        (context, count)
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
