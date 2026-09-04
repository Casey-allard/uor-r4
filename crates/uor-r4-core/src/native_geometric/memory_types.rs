//! Serializable learned memory-read operator and its configurable bounds.
use super::*;

pub(super) const MEMORY_FEATURE_COUNT: usize = 18;
pub(super) const MEMORY_SCHEMA: &str = "uor-r4.native-prime-relative-memory-read/2";
pub(super) const LEGACY_MEMORY_SCHEMA: &str = "uor-r4.native-prime-relative-memory-read/1";
pub(super) const CUE_SCHEMA: &str = "leading-unicode-whitespace-word-equivalence/1";
pub(super) const EXACT_CUE_SCHEMA: &str = "exact-token-prime/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadFitConfig {
    pub query_tokens: usize,
    pub source_offsets: usize,
    pub postings_per_address: usize,
    pub candidate_limit: usize,
    pub max_positions: usize,
    /// Epochs per fitting stage: this many pointer pretraining epochs, then
    /// this many max-route refinement epochs. Both counts appear in the report.
    pub epochs: usize,
    pub max_features: usize,
}
impl Default for MemoryReadFitConfig {
    fn default() -> Self {
        Self {
            query_tokens: 8,
            source_offsets: 4,
            postings_per_address: 4,
            candidate_limit: 128,
            max_positions: 4096,
            epochs: 8,
            max_features: 65_536,
        }
    }
}
impl MemoryReadFitConfig {
    pub(super) fn validate(&self, vocabulary: usize) -> Result<()> {
        if !(1..=32).contains(&self.query_tokens)
            || !(1..=16).contains(&self.source_offsets)
            || !(1..=8).contains(&self.postings_per_address)
            || !(1..=256).contains(&self.candidate_limit)
            || self.candidate_limit < self.query_tokens
            || !(1..=16384).contains(&self.max_positions)
            || !(1..=64).contains(&self.epochs)
            || !(1..=262144).contains(&self.max_features)
            || self
                .max_positions
                .saturating_mul(self.candidate_limit + 256)
                > 2_097_152
            || vocabulary
                .saturating_mul(self.source_offsets.next_power_of_two())
                .saturating_mul(self.postings_per_address.next_power_of_two())
                > 8_388_608
        {
            return Err(Error(
                "memory-read configuration exceeds bounded index/candidate/fit capacity".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) struct MemoryFeature {
    pub kind: u8,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MemoryWeight {
    pub feature: MemoryFeature,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CueAliases {
    pub schema: String,
    /// Dense references to existing complete lexical-prime identities. These
    /// are cue classes only; output tokens and geometry retain original IDs.
    pub representatives: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MemoryModel {
    pub schema: String,
    pub baseline_artifact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cue_aliases: Option<CueAliases>,
    pub config: MemoryReadFitConfig,
    pub source_shift: u8,
    pub posting_shift: u8,
    pub training: Vec<DocumentReceipt>,
    pub rows: Vec<MemoryWeight>,
    pub fit_positions: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryReadFitReport {
    pub schema: String,
    pub cue_identity: String,
    pub aliased_lexical_tokens: usize,
    pub objective: String,
    pub pointer_pretrain_epochs: usize,
    pub max_route_refinement_epochs: usize,
    pub calibrated_bias_score: i32,
    pub pointer_fit_correct_before: usize,
    pub pointer_fit_correct_after: usize,
    pub pointer_cross_entropy_before: f64,
    pub pointer_cross_entropy_after: f64,
    pub sampling: String,
    pub documents: usize,
    pub positions: usize,
    pub observed_context_positions: usize,
    pub tail_positions_per_document_limit: usize,
    pub tail_positions: usize,
    pub body_positions: usize,
    pub target_in_candidates: usize,
    pub target_in_memory: usize,
    pub candidate_positions: usize,
    pub fit_correct_before: usize,
    pub fit_correct_after: usize,
    pub candidate_cross_entropy_before: f64,
    pub candidate_cross_entropy_after: f64,
    pub learned_features: usize,
    pub dropped_feature_events: usize,
    pub epochs: usize,
    pub session_memory_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStateView {
    pub retained_tokens: usize,
    pub query_tokens: usize,
    pub source_offsets: usize,
    pub postings_per_address: usize,
    pub candidate_limit: usize,
    pub ring_storage_bytes: usize,
    pub index_storage_bytes: usize,
    pub candidate_storage_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryEntry {
    pub sequence: u64,
    pub token: u32,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
}
#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryReference {
    pub sequence: u64,
    pub slot: usize,
}
#[derive(Debug, Clone, Copy)]
pub(super) struct MemoryCandidate {
    pub token: u32,
    pub score: i64,
    pub features: [MemoryFeature; MEMORY_FEATURE_COUNT],
}
#[derive(Debug, Clone)]
pub(super) struct MemoryState {
    pub ring: Vec<MemoryEntry>,
    pub index: Vec<MemoryReference>,
    pub candidates: Vec<MemoryCandidate>,
    pub cursor: usize,
    pub length: usize,
    pub seen: u64,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
    pub view: MemoryStateView,
}
impl MemoryState {
    /// Host allocation. Prediction and updates live in memory_runtime.rs.
    pub(super) fn new(model: &Model, memory: &MemoryModel) -> Self {
        let ring = vec![
            MemoryEntry {
                sequence: u64::MAX,
                token: BOS,
                pose: model.geometry.identity,
                phases: [0; PHASE_CHANNELS]
            };
            model.config.context_tokens
        ];
        let index = vec![
            MemoryReference {
                sequence: u64::MAX,
                slot: 0
            };
            model.vocabulary_size() << memory.source_shift << memory.posting_shift
        ];
        let candidates = Vec::<MemoryCandidate>::with_capacity(memory.config.candidate_limit);
        let view = MemoryStateView {
            retained_tokens: 0,
            query_tokens: memory.config.query_tokens,
            source_offsets: memory.config.source_offsets,
            postings_per_address: memory.config.postings_per_address,
            candidate_limit: memory.config.candidate_limit,
            ring_storage_bytes: std::mem::size_of_val(ring.as_slice()),
            index_storage_bytes: std::mem::size_of_val(index.as_slice()),
            candidate_storage_bytes: candidates.capacity() * std::mem::size_of::<MemoryCandidate>(),
        };
        Self {
            ring,
            index,
            candidates,
            cursor: 0,
            length: 0,
            seen: 0,
            pose: model.geometry.identity,
            phases: [0; PHASE_CHANNELS],
            view,
        }
    }
}
