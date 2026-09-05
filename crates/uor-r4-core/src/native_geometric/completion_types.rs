//! Optional bounded next-byte completion after an observed typed numeral.
use super::value_types::{ValueAction, ValueDecision};
use super::*;

pub(super) const COMPLETION_SCHEMA: &str = "uor-r4.native-value-completion/1";
pub(super) const COMPLETION_FEATURES: usize = 16;
pub(super) const COMPLETION_CANDIDATES: usize = 16;
pub(super) const COMPLETION_POSTINGS: usize = 4;
pub(super) const COMPLETION_ROWS: usize = 4096;
pub(super) const COMPLETION_ASSOCIATIONS: usize = 32768;
pub(super) const COMPLETION_POSITIONS: usize = 4096;
pub(super) const COMPLETION_STEPS: u8 = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompletionModel {
    pub schema: String,
    pub baseline_artifact: String,
    pub rows: Vec<ScoreRow>,
    pub global_postings: Vec<u32>,
    /// Epoch budget, exact learning-rate bits, position cap, selected epoch.
    pub fit_config: [u64; 4],
    pub fit_positions: usize,
    pub training: Vec<DocumentReceipt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompletionWork {
    pub observations: u64,
    pub anchors: u64,
    pub metadata_reads: u64,
    pub state_copies: u64,
    pub feature_queries: u64,
    pub row_comparisons: u64,
    pub matched_rows: u64,
    pub posting_offers: u64,
    pub candidate_comparisons: u64,
    pub candidate_writes: u64,
    pub candidate_drops: u64,
    pub candidate_evaluations: u64,
    pub score_lookups: u64,
    pub score_comparisons: u64,
    pub h4_reads: u64,
    pub orientation_reads: u64,
    pub phase_subtractions: u64,
    pub commits: u64,
    pub base_steps: u64,
    pub mismatches: u64,
    pub stops: u64,
    pub step_limits: u64,
}
impl CompletionWork {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompletionAction {
    #[default]
    Base,
    Emit,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionDecision {
    pub token: u32,
    pub score: i64,
    pub write_id: u64,
    pub step: u8,
    pub at_seen: u64,
    pub action: CompletionAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompletionAnchor {
    pub write_id: u64,
    pub action: ValueAction,
    /// Observation count immediately after the final numeric byte.
    pub at_seen: u64,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
    pub query_prime: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CompletionSeed {
    pub token: u32,
    pub write_id: u64,
    pub action: ValueAction,
    pub at_seen: u64,
}
impl From<ValueDecision> for CompletionSeed {
    fn from(decision: ValueDecision) -> Self {
        Self::from(&decision)
    }
}
impl From<&ValueDecision> for CompletionSeed {
    fn from(decision: &ValueDecision) -> Self {
        Self {
            token: decision.token,
            write_id: decision.write_id,
            action: decision.action,
            at_seen: decision.at_seen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct CompletionState {
    pub anchor: Option<CompletionAnchor>,
    pub last: u32,
    pub previous: u32,
    pub seen: u64,
    pub steps: u8,
    pub active: bool,
    pub last_action: CompletionAction,
    #[serde(skip)]
    pub pending: Option<CompletionDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionStateView {
    pub active: bool,
    pub write_id: Option<u64>,
    pub steps: u8,
    pub last_action: CompletionAction,
    pub storage_bytes: usize,
}
