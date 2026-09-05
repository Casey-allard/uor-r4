//! Optional learned lexical-token entry and continuation at a response boundary.
use super::*;

pub(super) const RESPONSE_ENTRY_SCHEMA: &str = "uor-r4.native-response-entry/1";
pub(super) const RESPONSE_ENTRY_FEATURES: usize = 16;
pub(super) const RESPONSE_ENTRY_CANDIDATES: usize = 16;
pub(super) const RESPONSE_ENTRY_POSTINGS: usize = 4;
pub(super) const RESPONSE_ENTRY_ROWS: usize = 4096;
pub(super) const RESPONSE_ENTRY_ASSOCIATIONS: usize = 32768;
pub(super) const RESPONSE_ENTRY_POSITIONS: usize = 4096;
pub(super) const RESPONSE_ENTRY_STEPS: u8 = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResponseEntryModel {
    pub schema: String,
    pub baseline_artifact: String,
    pub rows: Vec<ScoreRow>,
    pub global_postings: Vec<u32>,
    /// Epoch budget, exact learning-rate bits, position cap, selected entry
    /// epoch and selected continuation epoch. The schema fixes canonical
    /// model.encode(response) tokens followed by EOS as the target token law.
    pub fit_config: [u64; 5],
    pub fit_positions: usize,
    pub training: Vec<DocumentReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy: Option<super::word_copy_types::WordCopyModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseEntryAction {
    #[default]
    Base,
    Enter,
    Emit,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEntryDecision {
    pub token: u32,
    pub score: i64,
    /// Absolute observation count at the actual response boundary. This is
    /// distinct from a numeric write identity.
    pub boundary_seen: u64,
    pub step: u8,
    pub at_seen: u64,
    pub action: ResponseEntryAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResponseEntryAnchor {
    pub at_seen: u64,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
    pub query_prime: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct ResponseEntryState {
    pub boundary: Option<ResponseEntryAnchor>,
    pub last: u32,
    pub previous: u32,
    pub seen: u64,
    pub steps: u8,
    pub active: bool,
    pub last_action: ResponseEntryAction,
    #[serde(skip)]
    pub pending: Option<ResponseEntryDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseEntryStateView {
    pub active: bool,
    pub boundary_seen: Option<u64>,
    pub steps: u8,
    pub last_action: ResponseEntryAction,
    pub storage_bytes: usize,
}
