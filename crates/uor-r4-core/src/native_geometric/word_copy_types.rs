//! Artifact extension and causal occurrence cursor; no retained text is copied.
use super::value_types::{ValueFeature, ValueRow};
use super::*;

pub(super) const RESPONSE_COPY_SCHEMA: &str = "uor-r4.native-response-entry/2";
pub(super) const WORD_COPY_FEATURES: usize = 24;
pub(super) const WORD_COPY_ROWS: usize = 4096;
pub(super) const WORD_COPY_DICTIONARY: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WordCopyAddress {
    pub bytes: [u8; 32],
    pub len: u8,
    pub prime: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WordCopyModel {
    pub baseline_artifact: String,
    pub dictionary: Vec<WordCopyAddress>,
    pub rows: Vec<ValueRow>,
    pub continuation_rows: Vec<ScoreRow>,
    pub continuation_postings: Vec<u32>,
    pub fit_config: [u64; 5],
    pub fit_positions: usize,
    pub training: Vec<DocumentReceipt>,
    /// Versioned suffix frame derived from the actual observed final copied
    /// byte. False preserves the first /2 artifact's entry-boundary frame.
    #[serde(default, skip_serializing_if = "copy_suffix_disabled")]
    pub completed_word_suffix: bool,
    /// General entry composition and committed-copy dispatch. Omission keeps /2 behavior.
    #[serde(default, skip_serializing_if = "copy_suffix_disabled")]
    pub composed_entry: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefix_rows: Vec<ScoreRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefix_postings: Vec<u32>,
}

fn copy_suffix_disabled(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WordCopyWork {
    pub selector: CompletionWork,
    pub dictionary_lookups: u64,
    pub dictionary_comparisons: u64,
    pub dictionary_byte_comparisons: u64,
    pub word_candidates: u64,
    /// Logical borrowed WordAtom records, not byte-level machine loads.
    pub word_record_reads: u64,
    pub bound_rejections: u64,
    pub byte_reads: u64,
    #[serde(default)]
    pub equality_byte_comparisons: u64,
    #[serde(default)]
    pub dispatch_checks: u64,
    #[serde(default)]
    pub forced_dispatches: u64,
}
impl WordCopyWork {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WordCopyAction {
    Start,
    Byte,
    Emit,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WordCopyDecision {
    pub token: u32,
    pub score: i64,
    pub word_index: u8,
    pub cursor: u8,
    pub source_end: u64,
    pub source_byte_end: u64,
    pub at_seen: u64,
    pub step: u8,
    pub action: WordCopyAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WordCopyProgress {
    #[default]
    Idle,
    Emitting {
        cursor: u8,
    },
    Complete,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WordCopyState {
    /// Immutable selected first-entry occurrence until the entry ends.
    pub origin: Option<u8>,
    pub progress: WordCopyProgress,
    #[serde(default, skip_serializing_if = "copy_start_zero")]
    pub start_step: u8,
    #[serde(skip)]
    pub pending: Option<WordCopyDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordCopyStateView {
    pub origin: Option<u8>,
    pub progress: WordCopyProgress,
    pub storage_bytes: usize,
}

pub(super) struct WordCopyContext {
    pub addresses: [u32; 16],
    pub query_path: Option<u16>,
    pub query_phases: Option<[u16; PHASE_CHANNELS]>,
}

pub(super) type CopyFeatures = ([ValueFeature; WORD_COPY_FEATURES], usize);

fn copy_start_zero(value: &u8) -> bool {
    *value == 0
}
