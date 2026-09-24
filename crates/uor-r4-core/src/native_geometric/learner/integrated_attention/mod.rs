//! One experimental causal artifact for learned sparse attention and language.
//!
//! Raw event identity is exact. Learned finite codes propose memory access; they
//! do not establish semantic identity or complete search. All training is offline
//! Rust. Runtime numerical decisions use selected low-bit rows and integer tables.
#![forbid(unsafe_code)]

pub mod encoder;
pub mod geometry;
pub mod memory;
pub mod output;
pub mod read_action;
pub mod training;
pub mod training_support;

use encoder::{CodeEncoder, EncoderInput, EncoderKind, CONTEXT_TOKENS};
use geometry::{
    AlgebraKind, AlgebraReadCounts, EnergyReadCounts, EnergyTables, FiniteAlgebra, LanePair,
};
use memory::{ExactKey, ExactMemory, MemoryLimits, MemorySnapshot, ProductCode, RecordId};
use output::{Action, OutputTrace, SparseOutput};
use read_action::{ReadActionModel, ReadActionReadCounts};
use serde::{Deserialize, Serialize};
use training_support::{GateReadCounts, SparseGate};

const MAGIC: &[u8; 8] = b"UORIA01\0";
const MAGIC_A2: &[u8; 8] = b"UORIA02\0";
const MAGIC_A4: &[u8; 8] = b"UORIA03\0";
pub const MAX_ARTIFACT_BYTES: usize = 256 << 20;
pub const MAX_LANES: usize = 4;
/// A2 indexes an exact occurrence only after this much right context arrives.
pub const ADDRESS_COMMIT_DELAY: u64 = 16;

#[derive(Debug)]
pub enum ModelError {
    Configuration(&'static str),
    Artifact(&'static str),
    InvalidToken(u32),
    Geometry(geometry::GeometryError),
    Encoder(encoder::EncoderError),
    Memory(memory::MemoryError),
    Output(output::OutputError),
    Training(training_support::TrainingSupportError),
    ReadAction(read_action::ReadActionError),
    Serialization(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ModelError {}
macro_rules! convert_error {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for ModelError {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
}
convert_error!(geometry::GeometryError, Geometry);
convert_error!(encoder::EncoderError, Encoder);
convert_error!(memory::MemoryError, Memory);
convert_error!(output::OutputError, Output);
convert_error!(training_support::TrainingSupportError, Training);
convert_error!(read_action::ReadActionError, ReadAction);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub vocabulary: usize,
    pub lanes: usize,
    pub memory: MemoryLimits,
}
impl ModelConfig {
    pub fn pilot(vocabulary: usize, lanes: usize) -> Result<Self, ModelError> {
        let mut memory = MemoryLimits::pilot();
        memory.event_capacity = 4096;
        memory.record_capacity = 4096;
        memory.max_token_bytes = 1024;
        memory.max_record_bytes = 1024;
        memory.max_record_tokens = 1;
        memory.max_examined_entries = 512;
        let config = Self {
            vocabulary,
            lanes,
            memory,
        };
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.vocabulary.is_power_of_two()
            || !(8..=4096).contains(&self.vocabulary)
            || !self.lanes.is_power_of_two()
            || self.lanes > MAX_LANES
        {
            return Err(ModelError::Configuration(
                "power-of-two vocab 8..4096 and lanes 1/2/4 required",
            ));
        }
        self.memory.validate()?;
        if self.memory.event_capacity > 65_536
            || self.memory.record_capacity > 65_536
            || self.memory.max_token_bytes > 4096
            || self.memory.max_record_bytes > 4096
            || self.memory.max_index_pages > 262_144
            || self.memory.max_postings_per_page > 4096
            || self.memory.max_pages_per_query > 4
            || self.memory.max_examined_entries > 512
        {
            return Err(ModelError::Configuration("memory/search exceeds A1 bounds"));
        }
        if self.memory.max_record_tokens != 1
            || self.memory.max_key_bytes < 8
            || self.memory.max_record_bytes < self.memory.max_token_bytes
        {
            return Err(ModelError::Configuration(
                "A1 requires one-token records, event-ID keys and complete token payloads",
            ));
        }
        Ok(())
    }
    pub fn feature_bank(&self) -> usize {
        (self.vocabulary << 1) + (self.lanes << 7) + 1
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactBinding {
    pub source_commit: String,
    pub tokenizer_sha256: String,
    pub training_data_blake3: String,
    pub training_seed: u64,
    pub training_updates: u64,
    pub credit_assignment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegratedModel {
    pub version: u16,
    pub config: ModelConfig,
    pub binding: ArtifactBinding,
    pub algebra: FiniteAlgebra,
    pub encoder: CodeEncoder,
    pub energy: EnergyTables,
    pub output: SparseOutput,
    pub read_gate: SparseGate,
    pub write_gate: SparseGate,
    /// A4 replaces sequential rank/gate with one candidate-or-NoRead decision.
    /// Omission preserves the prior A1/A2 serialized payload layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_action: Option<ReadActionModel>,
    #[serde(skip)]
    digest: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct Runtime<'a> {
    pub config: &'a ModelConfig,
    pub algebra: &'a FiniteAlgebra,
    pub encoder: &'a CodeEncoder,
    pub energy: &'a EnergyTables,
    pub output: &'a SparseOutput,
    pub read_gate: &'a SparseGate,
    pub write_gate: &'a SparseGate,
    pub read_action: Option<&'a ReadActionModel>,
    pub digest: [u8; 32],
}

impl IntegratedModel {
    /// Offline construction; the 2I constructor may build a floating reference table.
    pub fn new(config: ModelConfig, kind: AlgebraKind, seed: u64) -> Result<Self, ModelError> {
        config.validate()?;
        let algebra = match kind {
            AlgebraKind::BinaryIcosahedral => FiniteAlgebra::new_2i()?,
            AlgebraKind::Cyclic120 => FiniteAlgebra::new_c120()?,
        };
        let mut edges: Vec<LanePair> = (0..config.lanes.saturating_sub(1))
            .map(|i| LanePair {
                left: i as u8,
                right: (i + 1) as u8,
            })
            .collect();
        if config.lanes > 2 {
            edges.push(LanePair {
                left: 0,
                right: (config.lanes - 1) as u8,
            });
        }
        let encoder = CodeEncoder::new(config.vocabulary, config.lanes, seed)?;
        let energy = EnergyTables::zeroed(config.lanes as u8, edges)?;
        let output = SparseOutput::new(config.vocabulary, config.feature_bank(), config.lanes + 2)?;
        let read_gate = SparseGate::new(config.feature_bank())?;
        let write_gate = SparseGate::new(config.feature_bank())?;
        Ok(Self {
            version: 1,
            config,
            binding: ArtifactBinding {
                training_seed: seed,
                ..Default::default()
            },
            algebra,
            encoder,
            energy,
            output,
            read_gate,
            write_gate,
            read_action: None,
            digest: [0; 32],
        })
    }
    pub fn enable_context_addressing(&mut self) -> Result<(), ModelError> {
        if self.config.memory.event_capacity < CONTEXT_TOKENS {
            return Err(ModelError::Configuration(
                "context window exceeds raw history",
            ));
        }
        self.encoder.enable_context_addressing()?;
        self.version = if self.read_action.is_some() { 3 } else { 2 };
        self.digest = [0; 32];
        self.validate()
    }
    pub fn enable_read_actions(&mut self) -> Result<(), ModelError> {
        if !self.encoder.context_addressing {
            return Err(ModelError::Configuration(
                "A4 requires contextual addressing",
            ));
        }
        if self.read_action.is_none() {
            self.read_action = Some(ReadActionModel::new(
                self.config.vocabulary,
                self.config.lanes as u8,
                self.energy.edges().to_vec(),
            )?);
        }
        self.version = 3;
        self.digest = [0; 32];
        self.validate()
    }
    pub fn runtime(&self) -> Runtime<'_> {
        Runtime {
            config: &self.config,
            algebra: &self.algebra,
            encoder: &self.encoder,
            energy: &self.energy,
            output: &self.output,
            read_gate: &self.read_gate,
            write_gate: &self.write_gate,
            read_action: self.read_action.as_ref(),
            digest: self.digest,
        }
    }
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.version
            != if self.read_action.is_some() {
                3
            } else if self.encoder.context_addressing {
                2
            } else {
                1
            }
        {
            return Err(ModelError::Artifact("unsupported model version"));
        }
        self.config.validate()?;
        self.algebra.validate()?;
        self.encoder.validate()?;
        if self.encoder.context_addressing && self.config.memory.event_capacity < CONTEXT_TOKENS {
            return Err(ModelError::Artifact(
                "context addressing needs a complete context window",
            ));
        }
        self.energy.validate()?;
        self.output.validate()?;
        self.read_gate.validate()?;
        self.write_gate.validate()?;
        if let Some(action) = &self.read_action {
            action.validate()?;
            if !self.encoder.context_addressing
                || action.vocabulary() != self.config.vocabulary
                || usize::from(action.lanes()) != self.config.lanes
                || action.feature_count() != self.config.feature_bank()
            {
                return Err(ModelError::Artifact("read action shape mismatch"));
            }
        }
        if usize::from(self.output.vocab) != self.config.vocabulary
            || usize::from(self.output.feature_bank) != self.config.feature_bank()
            || usize::from(self.energy.lanes()) != self.config.lanes
            || usize::from(self.encoder.vocab) != self.config.vocabulary
            || usize::from(self.encoder.lanes) != self.config.lanes
            || self.read_gate.feature_count() != self.config.feature_bank()
            || self.write_gate.feature_count() != self.config.feature_bank()
        {
            return Err(ModelError::Artifact("component shape mismatch"));
        }
        Ok(())
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, ModelError> {
        self.validate()?;
        let valid_digest = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
        if self.binding.source_commit.is_empty()
            || self.binding.credit_assignment.is_empty()
            || !valid_digest(&self.binding.tokenizer_sha256)
            || !valid_digest(&self.binding.training_data_blake3)
        {
            return Err(ModelError::Artifact(
                "source, tokenizer, data and training contract must be bound before export",
            ));
        }
        let mut payload = Vec::new();
        ciborium::into_writer(self, &mut payload)
            .map_err(|e| ModelError::Serialization(e.to_string()))?;
        if payload.len() > MAX_ARTIFACT_BYTES {
            return Err(ModelError::Artifact("artifact exceeds byte bound"));
        }
        let mut bytes = Vec::with_capacity(40 + payload.len());
        bytes.extend_from_slice(match self.version {
            3 => MAGIC_A4,
            2 => MAGIC_A2,
            _ => MAGIC,
        });
        bytes.extend_from_slice(blake3::hash(&payload).as_bytes());
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ModelError> {
        if bytes.len() < 40
            || bytes.len() > MAX_ARTIFACT_BYTES + 40
            || (&bytes[..8] != MAGIC && &bytes[..8] != MAGIC_A2 && &bytes[..8] != MAGIC_A4)
        {
            return Err(ModelError::Artifact("invalid artifact envelope"));
        }
        let digest = *blake3::hash(&bytes[40..]).as_bytes();
        if bytes[8..40] != digest {
            return Err(ModelError::Artifact("artifact digest mismatch"));
        }
        let mut cursor = std::io::Cursor::new(&bytes[40..]);
        let mut model: Self = ciborium::from_reader(&mut cursor)
            .map_err(|e| ModelError::Serialization(e.to_string()))?;
        if cursor.position() != (bytes.len() - 40) as u64 {
            return Err(ModelError::Artifact("trailing artifact bytes"));
        }
        model.validate()?;
        let expected_magic = match model.version {
            3 => MAGIC_A4,
            2 => MAGIC_A2,
            _ => MAGIC,
        };
        if &bytes[..8] != expected_magic {
            return Err(ModelError::Artifact(
                "artifact envelope and schema disagree",
            ));
        }
        let valid_digest = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
        if model.binding.source_commit.is_empty()
            || model.binding.credit_assignment.is_empty()
            || !valid_digest(&model.binding.tokenizer_sha256)
            || !valid_digest(&model.binding.training_data_blake3)
        {
            return Err(ModelError::Artifact("unbound loaded artifact"));
        }
        model.digest = digest;
        Ok(model)
    }
    pub fn parameter_bytes(&self) -> usize {
        self.encoder.stored_coefficient_bytes()
            + self.output.stored_coefficient_bytes()
            + self.energy.packed_bytes()
            + self.read_gate.parameter_bytes()
            + self.write_gate.parameter_bytes()
            + self
                .read_action
                .as_ref()
                .map_or(0, ReadActionModel::parameter_bytes)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct AccessCounts {
    pub encoder_coefficients: u64,
    pub gate_coefficients: u64,
    pub energy_coefficients: u64,
    pub energy_byte_reads: u64,
    pub output_coefficients: u64,
    pub group_table_reads: u64,
    pub pages_visited: u64,
    pub posting_entries_examined: u64,
    #[serde(default)]
    pub raw_context_events: u64,
    #[serde(default)]
    pub read_action_coefficients: u64,
}
impl AccessCounts {
    pub fn learned_coefficient_reads(&self) -> u64 {
        self.encoder_coefficients
            + self.gate_coefficients
            + self.energy_coefficients
            + self.output_coefficients
            + self.read_action_coefficients
    }
    pub fn add(&mut self, other: Self) {
        self.encoder_coefficients += other.encoder_coefficients;
        self.gate_coefficients += other.gate_coefficients;
        self.energy_coefficients += other.energy_coefficients;
        self.energy_byte_reads += other.energy_byte_reads;
        self.output_coefficients += other.output_coefficients;
        self.group_table_reads += other.group_table_reads;
        self.pages_visited += other.pages_visited;
        self.posting_entries_examined += other.posting_entries_examined;
        self.raw_context_events += other.raw_context_events;
        self.read_action_coefficients += other.read_action_coefficients;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CandidateView {
    pub record_id: RecordId,
    pub source_event_id: u64,
    pub token: u16,
    pub key: [u8; 16],
    pub relative: [u8; 16],
    pub energy: i32,
    /// Higher is better for A4; legacy empirical energy remains diagnostic.
    pub action_score: Option<i32>,
}

pub struct ReadTrace {
    pub input: EncoderInput,
    pub query: [u8; 16],
    pub candidates: [CandidateView; 64],
    pub candidate_count: usize,
    pub ranked: Option<usize>,
    pub selected: Option<usize>,
    pub gate_enabled: bool,
    pub no_read_score: Option<i32>,
    pub search_incomplete: bool,
    pub access: AccessCounts,
}
impl ReadTrace {
    pub fn selected_candidate(&self) -> Option<CandidateView> {
        self.selected.map(|i| self.candidates[i])
    }
    pub fn selected_token(&self) -> Option<u16> {
        self.selected_candidate().map(|c| c.token)
    }
}

#[derive(Clone, Copy)]
pub struct FeatureList {
    ids: [u16; 8],
    len: usize,
}
impl FeatureList {
    pub fn as_slice(&self) -> &[u16] {
        &self.ids[..self.len]
    }
}
pub fn features(
    config: &ModelConfig,
    last: u16,
    state: &[u8; 16],
    read: Option<u16>,
) -> FeatureList {
    let mut ids = [0; 8];
    ids[0] = last;
    for lane in 0..config.lanes {
        ids[lane + 1] = (config.vocabulary + (lane << 7) + usize::from(state[lane])) as u16;
    }
    ids[config.lanes + 1] = (config.vocabulary
        + (config.lanes << 7)
        + read.map_or(config.vocabulary, usize::from)) as u16;
    FeatureList {
        ids,
        len: config.lanes + 2,
    }
}

pub struct ObservationTrace {
    pub event_id: u64,
    /// Exact earlier source indexed by this commit; absent during right-context warmup.
    pub indexed_source_event_id: Option<u64>,
    pub record_id: Option<RecordId>,
    pub key_input: EncoderInput,
    pub key: [u8; 16],
    pub state_input: EncoderInput,
    pub actions: [u8; 16],
    pub gate_features: FeatureList,
    pub access: AccessCounts,
}

pub struct Session {
    pub memory: ExactMemory,
    pub state: [u8; 16],
    pub last_token: u16,
    pub source_id: u64,
    pub byte_offset: u64,
    pub last_transition: Option<(EncoderInput, [u8; 16])>,
    model_digest: [u8; 32],
    poisoned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub memory: MemorySnapshot,
    pub state: [u8; 16],
    pub last_token: u16,
    pub source_id: u64,
    pub byte_offset: u64,
    pub model_digest: [u8; 32],
}

impl Session {
    pub fn new(runtime: Runtime<'_>, source_id: u64) -> Result<Self, ModelError> {
        let mut state = [0; 16];
        state[..runtime.config.lanes].fill(runtime.algebra.identity());
        Ok(Self {
            memory: ExactMemory::new(runtime.config.memory)?,
            state,
            last_token: 0,
            source_id,
            byte_offset: 0,
            last_transition: None,
            model_digest: runtime.digest,
            poisoned: false,
        })
    }
    fn check(&self, runtime: Runtime<'_>) -> Result<(), ModelError> {
        if self.poisoned {
            return Err(ModelError::Artifact(
                "discard session after a failed memory commit",
            ));
        }
        if self.model_digest != runtime.digest {
            return Err(ModelError::Artifact("session belongs to another model"));
        }
        Ok(())
    }
    fn observed_context(
        &self,
        masked_event: Option<u64>,
    ) -> Result<([u16; CONTEXT_TOKENS], u8), ModelError> {
        let mut context = [0; CONTEXT_TOKENS];
        let Some(latest) = self.memory.latest_event_id() else {
            return Ok((context, 0));
        };
        let start = latest.saturating_sub(CONTEXT_TOKENS as u64 - 1).max(1);
        let mut length = 0usize;
        for id in start..=latest {
            let event = self.memory.raw_event(id)?;
            if event.source_id != self.source_id {
                return Err(ModelError::Artifact(
                    "context window crosses source ownership",
                ));
            }
            context[length] = if masked_event == Some(id) {
                u16::MAX
            } else {
                u16::try_from(event.token_id)
                    .map_err(|_| ModelError::InvalidToken(event.token_id))?
            };
            length += 1;
        }
        Ok((context, length as u8))
    }
    pub fn read(&self, runtime: Runtime<'_>, enabled: bool) -> Result<ReadTrace, ModelError> {
        self.check(runtime)?;
        let (context, context_len) = if runtime.encoder.context_addressing {
            self.observed_context(None)?
        } else {
            ([0; CONTEXT_TOKENS], 0)
        };
        let input = EncoderInput {
            context,
            context_len,
            token: self.last_token,
            previous: self.state,
            evidence: None,
            kind: EncoderKind::Query,
        };
        let encoded = runtime.encoder.encode(input)?;
        let mut trace = ReadTrace {
            input,
            query: encoded.codes,
            candidates: [CandidateView::default(); 64],
            candidate_count: 0,
            ranked: None,
            selected: None,
            gate_enabled: false,
            no_read_score: None,
            search_incomplete: false,
            access: AccessCounts {
                encoder_coefficients: encoded.coefficient_reads as u64,
                raw_context_events: u64::from(context_len),
                ..Default::default()
            },
        };
        let Some(cutoff) = self.memory.latest_event_id() else {
            return Ok(trace);
        };
        if !enabled {
            return Ok(trace);
        }
        let result = self.memory.query(
            // Coarse admission must not condition on value-bearing fine lanes.
            ProductCode::new(
                &trace.query[..if runtime.encoder.context_addressing {
                    1
                } else {
                    runtime.config.lanes
                }],
            )?,
            None,
            Some(self.source_id),
            cutoff,
        )?;
        trace.access.pages_visited = result.pages_visited as u64;
        trace.access.posting_entries_examined = result.entries_examined as u64;
        trace.search_incomplete = result.status.incomplete_index
            || result.status.page_limit
            || result.status.examined_limit
            || result.status.candidate_limit;
        let mut energy_reads = EnergyReadCounts::default();
        let mut group_reads = AlgebraReadCounts::default();
        let mut action_reads = ReadActionReadCounts::default();
        if let Some(action) = runtime.read_action {
            let none_features = features(runtime.config, self.last_token, &self.state, None);
            trace.no_read_score =
                Some(action.score_none(none_features.as_slice(), &mut action_reads)?);
        }
        let actions = [runtime.algebra.identity(); 16];
        for &id in result.candidates() {
            let record = self.memory.record(id)?;
            let Some(&token) = record.token_ids().first() else {
                continue;
            };
            if token as usize >= runtime.config.vocabulary {
                return Err(ModelError::InvalidToken(token));
            }
            let mut candidate = CandidateView {
                record_id: id,
                source_event_id: record.source_event_id,
                token: token as u16,
                ..Default::default()
            };
            candidate.key[..runtime.config.lanes].copy_from_slice(record.code().as_slice());
            runtime.algebra.relative_lanes_counted(
                &trace.query[..runtime.config.lanes],
                &actions[..runtime.config.lanes],
                &candidate.key[..runtime.config.lanes],
                &mut candidate.relative[..runtime.config.lanes],
                &mut group_reads,
            )?;
            candidate.energy = runtime.energy.score(
                &candidate.relative[..runtime.config.lanes],
                &mut energy_reads,
            )?;
            if let Some(action) = runtime.read_action {
                let candidate_features = features(
                    runtime.config,
                    self.last_token,
                    &self.state,
                    Some(candidate.token),
                );
                candidate.action_score = Some(action.score_read(
                    candidate_features.as_slice(),
                    &candidate.relative[..runtime.config.lanes],
                    &mut action_reads,
                )?);
            }
            let index = trace.candidate_count;
            trace.candidates[index] = candidate;
            trace.candidate_count += 1;
            if runtime.read_action.is_some() {
                if trace
                    .ranked
                    .is_none_or(|best| candidate.action_score > trace.candidates[best].action_score)
                {
                    trace.ranked = Some(index);
                }
            } else if trace.selected.is_none_or(|best| {
                candidate.energy < trace.candidates[best].energy
                    || (candidate.energy == trace.candidates[best].energy
                        && id > trace.candidates[best].record_id)
            }) {
                trace.selected = Some(index);
            }
        }
        trace.access.energy_coefficients = energy_reads.coefficients;
        trace.access.energy_byte_reads = energy_reads.packed_bytes;
        trace.access.group_table_reads = group_reads.product_reads + group_reads.inverse_reads;
        trace.access.read_action_coefficients = action_reads.coefficients;
        if runtime.read_action.is_some() {
            // A single action argmax: NoRead wins ties, then actual candidate
            // iteration order resolves read ties. No second gate is applied.
            if let Some(best) = trace.ranked {
                if trace.candidates[best].action_score > trace.no_read_score {
                    trace.selected = Some(best);
                }
            }
            trace.gate_enabled = trace.selected.is_some();
            return Ok(trace);
        }
        trace.ranked = trace.selected;
        if let Some(index) = trace.ranked {
            let gate_features = features(
                runtime.config,
                self.last_token,
                &self.state,
                Some(trace.candidates[index].token),
            );
            let mut gate_reads = GateReadCounts::default();
            trace.gate_enabled = runtime
                .read_gate
                .enabled(gate_features.as_slice(), &mut gate_reads)?;
            trace.access.gate_coefficients = gate_reads.coefficients;
            if !trace.gate_enabled {
                trace.selected = None;
            }
        }
        Ok(trace)
    }
    pub fn output(
        &self,
        runtime: Runtime<'_>,
        read: &ReadTrace,
        draws: Option<&[u8]>,
    ) -> Result<OutputTrace, ModelError> {
        self.check(runtime)?;
        let feat = features(
            runtime.config,
            self.last_token,
            &self.state,
            read.selected_token(),
        );
        Ok(match draws {
            Some(random) => runtime.output.sample_from_bytes(
                feat.as_slice(),
                read.selected.is_some(),
                random,
            )?,
            None => runtime
                .output
                .greedy_branch(feat.as_slice(), read.selected.is_some())?,
        })
    }
    pub fn observe(
        &mut self,
        runtime: Runtime<'_>,
        token: u32,
        raw_bytes: &[u8],
        evidence: Option<u16>,
    ) -> Result<ObservationTrace, ModelError> {
        self.check(runtime)?;
        if token as usize >= runtime.config.vocabulary {
            return Err(ModelError::InvalidToken(token));
        }
        let state_input = EncoderInput {
            token: token as u16,
            previous: self.state,
            evidence,
            kind: EncoderKind::State,
            context: [0; CONTEXT_TOKENS],
            context_len: 0,
        };
        let action = runtime.encoder.encode(state_input)?;
        let next_offset = self
            .byte_offset
            .checked_add(raw_bytes.len() as u64)
            .ok_or(ModelError::Artifact("byte offset exhausted"))?;
        let mut groups = AlgebraReadCounts::default();
        let mut next_state = self.state;
        for lane in 0..runtime.config.lanes {
            next_state[lane] = runtime.algebra.compose_counted(
                self.state[lane],
                action.codes[lane],
                &mut groups,
            )?;
        }
        // A failed partial memory commit poisons the session, preserving the existing contract.
        self.poisoned = true;
        let event_id =
            self.memory
                .observe(self.source_id, 0, token, self.byte_offset, raw_bytes)?;
        self.byte_offset = next_offset;
        let indexed_source = if runtime.encoder.context_addressing {
            event_id
                .checked_sub(ADDRESS_COMMIT_DELAY)
                .filter(|&id| id > 0)
        } else {
            Some(event_id)
        };
        let (context, context_len) = if runtime.encoder.context_addressing {
            self.observed_context(indexed_source)?
        } else {
            ([0; CONTEXT_TOKENS], 0)
        };
        let source_token = match indexed_source {
            Some(id) => self.memory.raw_event(id)?.token_id,
            None => token,
        };
        let source_token_u16 =
            u16::try_from(source_token).map_err(|_| ModelError::InvalidToken(source_token))?;
        if usize::from(source_token_u16) >= runtime.config.vocabulary {
            return Err(ModelError::InvalidToken(source_token));
        }
        let key_input = EncoderInput {
            token: self.last_token,
            previous: self.state,
            evidence: Some(source_token_u16),
            kind: EncoderKind::Key,
            context,
            context_len,
        };
        let key = runtime.encoder.encode(key_input)?;
        let gate_features = features(runtime.config, source_token_u16, &self.state, None);
        let mut gate_reads = GateReadCounts::default();
        let write = indexed_source.is_some()
            && runtime
                .write_gate
                .enabled(gate_features.as_slice(), &mut gate_reads)?;
        let record_id = if write {
            let source =
                indexed_source.ok_or(ModelError::Artifact("missing indexed occurrence"))?;
            let exact = ExactKey::new(&source.to_le_bytes(), runtime.config.memory.max_key_bytes)?;
            let payload = if runtime.encoder.context_addressing {
                std::borrow::Cow::Owned(self.memory.raw_event(source)?.bytes().to_vec())
            } else {
                std::borrow::Cow::Borrowed(raw_bytes)
            };
            Some(
                self.memory
                    .write(
                        source,
                        &exact,
                        &payload,
                        &[source_token],
                        ProductCode::new(&key.codes[..runtime.config.lanes])?,
                    )?
                    .record_id,
            )
        } else {
            None
        };
        self.state = next_state;
        self.last_token = token as u16;
        self.last_transition = Some((state_input, action.codes));
        self.poisoned = false;
        Ok(ObservationTrace {
            event_id,
            indexed_source_event_id: record_id.and(indexed_source),
            record_id,
            key_input,
            key: key.codes,
            state_input,
            actions: action.codes,
            gate_features,
            access: AccessCounts {
                encoder_coefficients: (key.coefficient_reads + action.coefficient_reads) as u64,
                raw_context_events: u64::from(context_len),
                gate_coefficients: gate_reads.coefficients,
                group_table_reads: groups.product_reads + groups.inverse_reads,
                ..Default::default()
            },
        })
    }
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            memory: self.memory.snapshot(),
            state: self.state,
            last_token: self.last_token,
            source_id: self.source_id,
            byte_offset: self.byte_offset,
            model_digest: self.model_digest,
        }
    }
    pub fn restore(runtime: Runtime<'_>, snapshot: SessionSnapshot) -> Result<Self, ModelError> {
        if runtime.digest == [0; 32]
            || snapshot.model_digest != runtime.digest
            || snapshot.last_token as usize >= runtime.config.vocabulary
            || snapshot.state[..runtime.config.lanes]
                .iter()
                .any(|&x| x >= 120)
            || snapshot.state[runtime.config.lanes..]
                .iter()
                .any(|&x| x != 0)
        {
            return Err(ModelError::Artifact("invalid or foreign session snapshot"));
        }
        let memory = ExactMemory::restore_with_limits(snapshot.memory, runtime.config.memory)?;
        if memory.limits() != runtime.config.memory {
            return Err(ModelError::Artifact("snapshot limits mismatch"));
        }
        if let Some(id) = memory.latest_event_id() {
            let last = memory.raw_event(id)?;
            if last.token_id != u32::from(snapshot.last_token)
                || last.source_id != snapshot.source_id
                || last.byte_end() != Some(snapshot.byte_offset)
            {
                return Err(ModelError::Artifact(
                    "snapshot cursor disagrees with raw history",
                ));
            }
        } else if snapshot.last_token != 0 || snapshot.byte_offset != 0 {
            return Err(ModelError::Artifact("nonempty cursor without history"));
        }
        Ok(Self {
            memory,
            state: snapshot.state,
            last_token: snapshot.last_token,
            source_id: snapshot.source_id,
            byte_offset: snapshot.byte_offset,
            model_digest: snapshot.model_digest,
            last_transition: None,
            poisoned: false,
        })
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    #[test]
    fn delayed_context_commit_preserves_occurrence_and_causal_cutoff() -> Result<(), ModelError> {
        let mut model =
            IntegratedModel::new(ModelConfig::pilot(64, 4)?, AlgebraKind::Cyclic120, 17)?;
        model.enable_context_addressing()?;
        let mut session = Session::new(model.runtime(), 91)?;
        for token in 1..=16u32 {
            let observed = session.observe(model.runtime(), token, b"x", None)?;
            assert!(observed.record_id.is_none());
            assert_eq!(session.memory.raw_event(observed.event_id)?.token_id, token);
        }
        let observed = session.observe(model.runtime(), 17, b"y", None)?;
        assert_eq!(observed.indexed_source_event_id, Some(1));
        assert_eq!(observed.key_input.context_len, 17);
        assert_eq!(observed.key_input.context[0], u16::MAX);
        assert_eq!(observed.key_input.context[16], 17);
        let record = session.memory.record(
            observed
                .record_id
                .ok_or(ModelError::Artifact("missing delayed record"))?,
        )?;
        assert_eq!(record.source_event_id, 1);
        assert_eq!(record.commit_event_id, 17);
        assert_eq!(record.token_ids(), &[1]);
        assert_eq!(record.payload(), b"x");
        let code = record.code();
        assert!(session
            .memory
            .query(code, None, Some(91), 16)?
            .candidates()
            .is_empty());
        assert_eq!(
            session
                .memory
                .query(code, None, Some(91), 17)?
                .candidates()
                .len(),
            1
        );
        Ok(())
    }
}
