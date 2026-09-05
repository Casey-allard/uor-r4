//! Artifact and fixed-capacity state for the optional typed-value operator.
use super::numeral::{Numeral, Scanner};
use super::value_lexemes::{LexemeState, WordAtom};
use super::*;

pub(super) const VALUE_SCHEMA: &str = "uor-r4.native-typed-value/1";
pub(super) const LEXEME_VALUE_SCHEMA: &str = "uor-r4.native-typed-value/2";
pub(super) const VALUES: usize = 16;
pub(super) const QUERY: usize = 8;
pub(super) const VALUE_FEATURES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueAction {
    Copy,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ValueWork {
    pub input_bytes: u64,
    pub literal_writes: u64,
    pub record_evictions: u64,
    pub proposals: u64,
    pub additions: u64,
    pub overflow_rejections: u64,
    pub feature_lookups: u64,
    pub feature_comparisons: u64,
    pub cue_comparisons: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub lexical_comparisons: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub lexical_byte_comparisons: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub lexical_writes: u64,
    pub h4_reads: u64,
    pub phase_updates: u64,
    pub numeral_steps: u64,
    pub derived_writes: u64,
    pub emission_commits: u64,
    pub emission_mismatches: u64,
}
fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}
impl ValueWork {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub(super) struct ValueFeature {
    pub kind: u8,
    pub a: u64,
    pub b: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueRow {
    pub feature: ValueFeature,
    pub weight: i32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueModel {
    pub schema: String,
    pub codec: String,
    pub capacity: usize,
    pub rows: Vec<ValueRow>,
    pub continuation_score: i32,
    /// Epoch budget, exact f64 learning-rate bits, feature cap, selected epoch.
    pub fit_config: [u64; 4],
    pub training: Vec<DocumentReceipt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueEntry {
    pub sequence: u64,
    pub token: u32,
    pub cue: u32,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueDerivation {
    pub action: ValueAction,
    pub operand_ids: [u64; 2],
    pub operand_values: [i64; 2],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ValueRecord {
    pub id: u64,
    pub value: i64,
    pub start: u64,
    pub end: u64,
    pub derived: bool,
    pub derivation: Option<ValueDerivation>,
    pub(super) cue: [u32; 4],
    pub(super) pose: u16,
    pub(super) phases: [u16; PHASE_CHANNELS],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) lexical: Option<[WordAtom; 4]>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueDecision {
    pub action: ValueAction,
    pub operands: [ValueRecord; 2],
    pub value: i64,
    pub write_id: u64,
    pub token: u32,
    pub cursor: u8,
    pub score: i64,
    pub at_seen: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueEmission {
    pub decision: ValueDecision,
    pub numeral: Numeral,
    pub cursor: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueState {
    pub scanner: Scanner,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexemes: Option<LexemeState>,
    pub recent: [ValueEntry; 32],
    pub recent_len: usize,
    pub recent_cursor: usize,
    pub records: Vec<ValueRecord>,
    pub sources: Vec<ValueRecord>,
    pub next_id: u64,
    pub seen: u64,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
    pub active: bool,
    pub consumed: bool,
    pub started_at: u64,
    pub queries: [ValueEntry; QUERY],
    pub query_len: usize,
    pub emission: Option<ValueEmission>,
    #[serde(skip)]
    pub pending: Option<ValueDecision>,
}
impl ValueState {
    pub fn new(model: &Model) -> Self {
        Self {
            scanner: Scanner::default(),
            lexemes: model
                .values
                .as_ref()
                .filter(|head| head.schema == LEXEME_VALUE_SCHEMA)
                .map(|_| LexemeState::default()),
            recent: [ValueEntry::default(); 32],
            recent_len: 0,
            recent_cursor: 0,
            records: Vec::with_capacity(VALUES),
            sources: Vec::with_capacity(VALUES),
            next_id: 0,
            seen: 0,
            pose: model.geometry.identity,
            phases: [0; PHASE_CHANNELS],
            active: false,
            consumed: false,
            started_at: 0,
            queries: [ValueEntry::default(); QUERY],
            query_len: 0,
            emission: None,
            pending: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueStateView {
    pub active: bool,
    pub retained_values: usize,
    pub captured_values: usize,
    pub committed_write: Option<u64>,
    pub emission_cursor: Option<u8>,
    pub storage_bytes: usize,
}
