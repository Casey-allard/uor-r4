//! Native count-fitted geometric language prototype.
//!
//! Learned, quantized conditional-score tables read an ordered H4 window,
//! prime-derived fixed-zeta phases and exact signed-root orientation. This is
//! an experimental finite-state language model, not an established general
//! reasoner. It has no neural matrix products or external-model fallback.
//! `runtime::{Session::observe, Session::predict}` is the integer/table kernel;
//! tokenization, serialization, fitting and diagnostic rendering are host work.

mod anchors;
mod memory_runtime;
mod memory_training;
mod memory_types;
mod mixture;
mod runtime;
mod snapshot;
mod training;

use serde::{Deserialize, Serialize};

pub use memory_training::{
    MemoryReadDiagnostic, MemoryReadDocumentExposure, MemoryReadDocumentSupervision,
    MemoryReadSchedule, MemoryReadStreamProgress, MemoryReadStreamReport, MemoryReadSupervision,
    MemoryReadTokenSpan, MemoryReadTrainer,
};
pub use memory_types::{MemoryReadFitConfig, MemoryReadFitReport, MemoryStateView};
pub use mixture::{ReadoutFitConfig, ReadoutFitReport};
pub use runtime::{Session, StateView};
pub use training::Trainer;

pub const SCHEMA: &str = "uor-r4.native-geometric-language/1";
pub const BOS: u32 = 0;
pub const EOS: u32 = 1;
pub const PHASE_CHANNELS: usize = 8;
const LEXICAL_BASE: u32 = 258;
const SCORE_SCALE: f64 = 256.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub context_tokens: usize,
    pub candidate_limit: usize,
    pub max_lexical_pieces: usize,
    pub max_rows: usize,
    pub max_associations: usize,
    pub postings_per_row: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            context_tokens: 128,
            candidate_limit: 32,
            max_lexical_pieces: 4096,
            max_rows: 65_536,
            max_associations: 500_000,
            postings_per_row: 16,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(1..=4096).contains(&self.context_tokens)
            || !(1..=256).contains(&self.candidate_limit)
            || !(1..=65_536).contains(&self.max_lexical_pieces)
            || !(1..=1_000_000).contains(&self.max_rows)
            || !(1..=8_000_000).contains(&self.max_associations)
            || !(1..=256).contains(&self.postings_per_row)
        {
            return Err(Error(
                "native geometric config exceeds supported bounds".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentReceipt {
    pub id: String,
    pub text_cid: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Control {
    #[default]
    Full,
    GeometryDisabled,
    ZetaDisabled,
    H4Disabled,
    OrientationDisabled,
    PairedDisabled,
    RadialDisabled,
    HeatmapDisabled,
    MemoryDisabled,
}

/// Explicit feature addresses, never content digests. Kinds 0/1 are full
/// prime lexical addresses; 2/3 are exact H4 state/trajectory; 4 is signed
/// orientation; 8..16 are the eight fixed-zeta phase channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) struct Feature {
    pub kind: u8,
    pub value: u64,
}
// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_BEGIN
// Runtime feature routing and attenuation; included in the kernel source guard.
impl Feature {
    fn group(self) -> usize {
        match self.kind {
            0 | 1 => 0,
            2 | 3 | 6 => 1,
            4 | 24 | 25 => 2,
            5 => 3,
            7 => 4,
            8..=15 => 5,
            _ => 6,
        }
    }
    fn shift(self) -> u8 {
        if (8..=23).contains(&self.kind) {
            3
        } else if self.kind >= 2 {
            1
        } else {
            0
        }
    }
    fn admitted(self, control: Control) -> bool {
        match control {
            Control::Full | Control::MemoryDisabled => true,
            Control::GeometryDisabled => self.kind < 2,
            Control::ZetaDisabled => !(8..=15).contains(&self.kind) && self.kind != 5,
            Control::H4Disabled => self.kind < 2 || (8..=15).contains(&self.kind),
            Control::OrientationDisabled => self.kind != 4,
            Control::PairedDisabled => self.kind != 6 && !(16..=23).contains(&self.kind),
            Control::RadialDisabled => self.kind != 7,
            Control::HeatmapDisabled => self.kind != 4 && self.kind != 24 && self.kind != 25,
        }
    }
}
// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_END

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenGeometry {
    prime: u32,
    leaf: u16,
    phases: [u16; PHASE_CHANNELS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Geometry {
    root_cid: String,
    product_cid: String,
    zeta_grid: String,
    identity: u16,
    row_bases: Vec<usize>,
    products: Vec<u16>,
    inverses: Vec<u16>,
    /// Encoded signs of exact root coordinates (q0,q1): each -1/0/+1 -> 0/1/2.
    orientation: Vec<u8>,
    anchors: anchors::AnchorTable,
    square_offset: i64,
    squares: Vec<i64>,
    tokens: Vec<TokenGeometry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenScore {
    token: u32,
    score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScoreRow {
    feature: Feature,
    default_score: i32,
    scores: Vec<TokenScore>,
    postings: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrainingProgress {
    pub documents_completed: usize,
    pub target_positions: u64,
    pub feature_events: u64,
    pub dropped_feature_events: u64,
    pub learned_rows: usize,
    pub learned_associations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ModelWire")]
pub struct Model {
    schema: String,
    artifact_cid: String,
    uor_model_address: String,
    config: Config,
    training: TrainingProgress,
    construction: Vec<DocumentReceipt>,
    lexical_pieces: Vec<Vec<u8>>,
    geometry: Geometry,
    prior_scores: Vec<i32>,
    prior_postings: Vec<u32>,
    rows: Vec<ScoreRow>,
    readout: mixture::Readout,
    readout_training: Vec<DocumentReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory_read: Option<memory_types::MemoryModel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelWire {
    schema: String,
    artifact_cid: String,
    uor_model_address: String,
    config: Config,
    training: TrainingProgress,
    construction: Vec<DocumentReceipt>,
    lexical_pieces: Vec<Vec<u8>>,
    geometry: Geometry,
    prior_scores: Vec<i32>,
    prior_postings: Vec<u32>,
    rows: Vec<ScoreRow>,
    readout: mixture::Readout,
    readout_training: Vec<DocumentReceipt>,
    #[serde(default)]
    memory_read: Option<memory_types::MemoryModel>,
}
impl TryFrom<ModelWire> for Model {
    type Error = Error;
    fn try_from(wire: ModelWire) -> Result<Self> {
        let model = Self {
            schema: wire.schema,
            artifact_cid: wire.artifact_cid,
            uor_model_address: wire.uor_model_address,
            config: wire.config,
            training: wire.training,
            construction: wire.construction,
            lexical_pieces: wire.lexical_pieces,
            geometry: wire.geometry,
            prior_scores: wire.prior_scores,
            prior_postings: wire.prior_postings,
            rows: wire.rows,
            readout: wire.readout,
            readout_training: wire.readout_training,
            memory_read: wire.memory_read,
        };
        model.validate()?;
        Ok(model)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Work {
    pub observed_tokens: u64,
    pub evictions: u64,
    pub h4_table_reads: u64,
    pub orientation_table_reads: u64,
    pub anchor_table_reads: u64,
    pub radial_square_reads: u64,
    pub phase_additions: u64,
    pub feature_queries: u64,
    pub matched_rows: u64,
    pub candidate_offers: u64,
    pub candidate_evaluations: u64,
    pub score_lookups: u64,
    pub mixture_gate_reads: u64,
    #[serde(default)]
    pub memory_index_reads: u64,
    #[serde(default)]
    pub memory_index_writes: u64,
    #[serde(default)]
    pub memory_stale_rejections: u64,
    #[serde(default)]
    pub memory_candidates: u64,
    #[serde(default)]
    pub memory_score_lookups: u64,
    #[serde(default)]
    pub memory_h4_reads: u64,
    #[serde(default)]
    pub memory_phase_updates: u64,
    #[serde(default)]
    pub memory_cue_reads: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub token: u32,
    pub score: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prediction {
    pub token: u32,
    pub score: i64,
    pub candidate_count: usize,
    pub geometric_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generation {
    pub text: String,
    pub utf8_valid: bool,
    /// Exact output bytes are retained if byte fallback generates invalid UTF-8.
    pub bytes: Vec<u8>,
    pub token_ids: Vec<u32>,
    pub stop: String,
    pub work: Work,
    pub state: StateView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    pub documents: usize,
    pub positions: u64,
    pub correct: u64,
    pub candidate_hits: u64,
    pub geometric_row_positions: u64,
    pub top1: f64,
    pub candidate_coverage: f64,
    pub work: Work,
}

#[cfg(test)]
mod tests;
