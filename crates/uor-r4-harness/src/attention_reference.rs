//! Host-only receipt validator and same-cell reference scorer for M1 Unit 2b
//! (`preflight-dev`) and Dose 1 paired DEV (`paired-dev`).
//!
//! All model forward, routing, distance, and candidate admission observations
//! must arrive from GPU-produced receipts (`uor-r4.attention-preflight-dev-capture/1`
//! and `uor-r4.discrete-corpus-driver/1`). No model forward or training executes here.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const PREFLIGHT_PANEL_SCHEMA: &str = "uor-r4.attention-preflight-panel/1";
pub const PREFLIGHT_CAPTURE_SCHEMA: &str = "uor-r4.attention-preflight-dev-capture/1";
pub const DRIVER_EVAL_SCHEMA: &str = "uor-r4.discrete-corpus-driver/1";
pub const ENVELOPED_EVAL_SCHEMA: &str = "uor-r4.attention-learning-evaluation/1";
pub const DRIVER_CHECKPOINT_SCHEMA: &str = "uor-r4.discrete-driver-checkpoint/1";
pub const ATTENTION_CHECKPOINT_SCHEMA: &str = "uor-r4.attention-learning-checkpoint/2";
pub const D34_RECORD_SCHEMA: &str = "uor-r4.successor-table-control/1";
pub const CANDIDATE_CELL_TABLE_SCHEMA: &str = "uor-r4.attention-candidate-cell-table/1";

pub const PREFLIGHT_SCORE_SCHEMA: &str = "uor-r4.attention-preflight-reference-score/1";
pub const PAIRED_DEV_SCORE_SCHEMA: &str = "uor-r4.attention-paired-dev-reference-score/1";

pub const CANONICAL_DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
pub const CANONICAL_DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
pub const CANONICAL_TRAIN_TOKENS_SHA256: &str =
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8";
pub const CANONICAL_TRAIN_MASK_SHA256: &str =
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3";
pub const CANONICAL_ORDERED_ROW_BINDING_SHA256: &str =
    "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446";
pub const CANONICAL_D34_RECORD_SHA256: &str =
    "2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec";

pub const CANONICAL_DEV_ELIGIBLE_ROWS: usize = 14_287;
pub const CANONICAL_DEV_PSUCC_ROWS: usize = 3_847;
pub const CANONICAL_DEV_COMP_ROWS: usize = 10_440;
pub const CANONICAL_TRAIN_ELIGIBLE_ROWS: usize = 2_425_925;
pub const CANONICAL_TRAIN_MATCHED_TARGETS: usize = 1_049_856;
pub const CANONICAL_VOCAB: usize = 4_096;
pub const CANONICAL_CONTEXT: usize = 192;
pub const CANONICAL_BOS: u16 = 0;
pub const UNUSED_CELL_SLOT: u32 = u32::MAX;

pub const TOKEN_CELLS_RECEIPT_SCHEMA: &str = "uor-r4.attention-token-cells/1";
pub const TOKEN_CELLS_RECEIPT_STATUS: &str = "DEVICE_TOKEN_CELL_MAP_EXPORTED_ZERO_APPLY";

pub const CANONICAL_PSUCC_TOP1_FLOOR: u64 = 24;
pub const CANONICAL_PSUCC_TOP8_FLOOR: u64 = 39;
pub const CANONICAL_COMPLEMENT_TOP1_FLOOR: u64 = 22;
pub const CANONICAL_COMPLEMENT_TOP8_FLOOR: u64 = 63;

/// Pure-integer SHA-256 implementation (FIPS 180-4) for receipt pinning.
pub fn sha256_hex(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(bytes.len() + 72);
    padded.extend_from_slice(bytes);
    padded.push(0x80);
    while (padded.len() + 8) % 64 != 0 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
    let mut out = String::with_capacity(64);
    for word in state {
        use std::fmt::Write;
        let _ = write!(&mut out, "{word:08x}");
    }
    out
}

pub const CANONICAL_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
pub const CANONICAL_TRAIN_PAYLOAD_TOKENS: u64 = 7_512_797;
pub const CANONICAL_DEV_PAYLOAD_TOKENS: u64 = 42_856;
pub const DISCRETE_SNAPSHOT_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";
pub const OBJECTIVE_EVALUATION_SCHEMA: &str = "uor-r4.discrete-objective-evaluation/1";

fn validate_sha256_str(s: &str, label: &str) -> Result<(), String> {
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{label}: invalid 64-hex sha256 '{s}'"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilePinRecord {
    pub path: std::path::PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitRecord {
    Train,
    Dev,
    Fresh,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BosIdentityRecord {
    pub token: String,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSourceRecord {
    pub split: SplitRecord,
    pub tokens: FilePinRecord,
    pub mask: FilePinRecord,
    pub expected_payload_tokens: u64,
    pub expected_eligible_targets: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusBindingRecord {
    pub source: CorpusSourceRecord,
    pub vocabulary: u32,
    pub bos: BosIdentityRecord,
    pub payload_tokens: u64,
    pub mask_bytes: u64,
    pub mask_selected_positions: u64,
    pub eligible_targets: u64,
    pub documents: u64,
    pub window_policy: String,
    pub identity_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecRecord {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReadSpecRecord {
    Route,
    Flock { kept: u32 },
    Relation { kept: u32, sweeps: u32 },
}

impl<'de> Deserialize<'de> for ReadSpecRecord {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Route {},
            Flock { kept: u32 },
            Relation { kept: u32, sweeps: u32 },
        }
        Ok(match <Wire as Deserialize>::deserialize(deserializer)? {
            Wire::Route {} => Self::Route,
            Wire::Flock { kept } => Self::Flock { kept },
            Wire::Relation { kept, sweeps } => Self::Relation { kept, sweeps },
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectivePolicyRecord {
    #[default]
    NearestAddressMarginV1,
    OccupiedCellRankHingeV1,
}

impl ObjectivePolicyRecord {
    pub fn is_nearest_address_margin(&self) -> bool {
        *self == Self::NearestAddressMarginV1
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadInitializationRecord {
    #[default]
    ZeroGatesV1,
    ActiveLeafV1,
}

impl ReadInitializationRecord {
    pub fn is_zero_gates(&self) -> bool {
        *self == Self::ZeroGatesV1
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationLearningPolicyRecord {
    #[default]
    LearnV1,
    FrozenZeroV1,
}

impl RelationLearningPolicyRecord {
    pub fn is_learn(&self) -> bool {
        *self == Self::LearnV1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulePolicyRecord {
    CanonicalVectorThenReadV1,
    InterleavedReadEvery8V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverConfigRecord {
    pub schema: String,
    pub spec: SpecRecord,
    pub read_spec: ReadSpecRecord,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicyRecord::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicyRecord,
    #[serde(
        default,
        skip_serializing_if = "ReadInitializationRecord::is_zero_gates"
    )]
    pub read_initialization: ReadInitializationRecord,
    #[serde(
        default,
        skip_serializing_if = "RelationLearningPolicyRecord::is_learn"
    )]
    pub relation_learning_policy: RelationLearningPolicyRecord,
    pub seed: u32,
    pub batch_rows: u32,
    pub train: CorpusSourceRecord,
    pub dev: CorpusSourceRecord,
    pub tokenizer: FilePinRecord,
    pub bos: BosIdentityRecord,
    pub schedule: SchedulePolicyRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverLineageRecord {
    pub schema: String,
    pub driver_source_sha256: String,
    pub config: DriverConfigRecord,
    pub train: CorpusBindingRecord,
    pub dev: CorpusBindingRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalModelBindingsRecord {
    pub schema: String,
    pub spec: SpecRecord,
    pub read_spec: ReadSpecRecord,
    pub objective_policy: ObjectivePolicyRecord,
    pub seed: u32,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub operator_kappa: String,
    pub composition_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenCellsModelBindingsRecord {
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub spec: SpecRecord,
    pub read_spec: ReadSpecRecord,
    pub objective_policy: ObjectivePolicyRecord,
    pub read_initialization: ReadInitializationRecord,
    pub seed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverCursorRecord {
    pub completed_batches: u64,
    pub accepted_updates: u64,
    pub corpus_pass: u64,
    pub next_target: u64,
    pub next_coordinate: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverCheckpointRecord {
    pub schema: String,
    pub lineage: DriverLineageRecord,
    pub cursor: DriverCursorRecord,
    pub model_sha256: String,
    pub model: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceSnapshotRecord {
    pub schema: String,
    pub spec: SpecRecord,
    pub read_spec: ReadSpecRecord,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicyRecord::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicyRecord,
    #[serde(
        default,
        skip_serializing_if = "ReadInitializationRecord::is_zero_gates"
    )]
    pub read_initialization: ReadInitializationRecord,
    pub seed: u32,
    pub epoch: u64,
    pub table: Vec<i32>,
    pub routes: Vec<u32>,
    pub gates: Vec<i32>,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

/// Validated GPU-exported `token_cells[4096]` loaded from an `attention-export-token-cells`
/// receipt (`uor-r4.attention-token-cells/1`), its bound `token_cells.u16`, and its pinned
/// `DriverCheckpoint` (`uor-r4.discrete-driver-checkpoint/1`).
/// All producer fields emitted by `#2601` / `#2611` and all four checkpoint/model hash domains are mandatory:
/// - `checkpoint_file_sha256`: raw `DriverCheckpoint` JSON file SHA256 (Domain D)
/// - `driver_checkpoint_sha256`: canonical typed `DriverCheckpointRecord` SHA256 (Domain C)
/// - `model_storage_sha256` (`model_snapshot_sha256` alias): canonical `serde_json::Value` model SHA256 (`checkpoint.model_sha256`, Domain A)
/// - `typed_model_snapshot_sha256`: typed `DeviceSnapshotRecord` SHA256 (`eval.model_snapshot_sha256`, Domain B)
#[derive(Debug, Clone)]
pub struct LoadedTokenCells {
    pub token_cells: Vec<u32>,
    pub occupied_cells: usize,
    pub token_cells_sha256: String,
    pub report_sha256: String,
    pub config_sha256: String,
    pub config_file_sha256: String,
    pub lineage_sha256: String,
    pub driver_checkpoint_sha256: String,
    pub checkpoint_file_sha256: String,
    pub model_storage_sha256: String,
    pub model_snapshot_sha256: String,
    pub typed_model_snapshot_sha256: String,
    pub model_epoch: u64,
    pub completed_batches: u64,
    pub model_bindings: TokenCellsModelBindingsRecord,
}

/// External receipt supplying GPU-exported token cells and registered non-model ranked candidate lists.
/// Scoring requires cross-checking every ranked table against `CorpusReferenceTables` rebuilt from
/// the SHA-verified `TRAIN` and `DEV` corpus files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateCellTable {
    pub schema: String,
    pub driver_checkpoint_sha256: String,
    #[serde(default)]
    pub model_snapshot_sha256: Option<String>,
    #[serde(default)]
    pub model_storage_sha256: Option<String>,
    #[serde(default)]
    pub model_value_sha256: Option<String>,
    #[serde(default)]
    pub token_cells_sha256: Option<String>,
    #[serde(default)]
    pub ordered_row_binding_sha256: Option<String>,
    #[serde(default)]
    pub t2_ranked_table_sha256: Option<String>,
    #[serde(default)]
    pub u_succ_ranked_table_sha256: Option<String>,
    #[serde(default)]
    pub u_train_ranked_table_sha256: Option<String>,
    #[serde(default)]
    pub dev_window_successors_sha256: Option<String>,
    pub token_cells: Vec<u32>,
    pub t2_ranked_candidates_by_query: Vec<Vec<u32>>,
    pub u_succ_ranked_candidates: Vec<u32>,
    #[serde(default)]
    pub u_train_ranked_candidates: Option<Vec<u32>>,
    #[serde(default)]
    pub dev_window_query_successors_most_recent_first: Option<Vec<u32_vec_alias::VecU32>>,
}

mod u32_vec_alias {
    pub type VecU32 = Vec<u32>;
}

/// Corpus files bundle for rebuilding registered non-model `T1/T1b/T2/U_succ`
/// count tables and cross-checking against the sealed D34 record.
#[derive(Debug, Clone)]
pub struct CorpusPaths<'a> {
    pub train_tokens: &'a Path,
    pub train_mask: &'a Path,
    pub dev_tokens: &'a Path,
    pub dev_mask: &'a Path,
}

#[derive(Debug, Clone)]
pub struct CorpusReferenceTables {
    pub train_tokens_sha256: String,
    pub train_mask_sha256: String,
    pub dev_tokens_sha256: String,
    pub dev_mask_sha256: String,
    pub ordered_row_binding_sha256: String,
    pub train_matched_targets: u64,
    pub train_bigram_total_pairs: u64,
    pub u_train_total_counts: u64,
    pub dev_psucc_rows: u64,
    pub u_succ_total_counts: u64,
    pub conservation_ok: bool,
    pub t2_ranked_table_sha256: String,
    pub u_train_ranked_table_sha256: String,
    pub u_succ_ranked_table_sha256: String,
    pub dev_window_successors_sha256: String,
    pub t2_ranked_by_query: Vec<Vec<u32>>,
    pub u_train_ranked: Vec<u32>,
    pub u_succ_ranked: Vec<u32>,
    pub dev_rows: Vec<DevCorpusRow>,
}

#[derive(Debug, Clone)]
pub struct DevCorpusRow {
    pub ordinal: usize,
    pub position: u64,
    pub query_token: u32,
    pub target_token: u32,
    pub is_psucc: bool,
    pub distance_bucket: Option<u8>,
    pub conflict: bool,
    pub t1_candidate: Option<u32>,
    pub t1b_candidate: Option<u32>,
    pub window_query_successors_most_recent_first: Vec<u32>,
}

/// Encode `4,096` cell IDs (`0..256`) into the canonical `8,192`-byte raw little-endian
/// `token_cells.u16` byte representation (no header).
pub fn encode_token_cells_u16_le(token_cells: &[u32]) -> Result<Vec<u8>, String> {
    if token_cells.len() != CANONICAL_VOCAB {
        return Err(format!(
            "token_cells length {} != {CANONICAL_VOCAB}",
            token_cells.len()
        ));
    }
    let mut out = Vec::with_capacity(CANONICAL_VOCAB * 2);
    for (idx, &c) in token_cells.iter().enumerate() {
        if c >= 256 {
            return Err(format!("token_cells[{idx}] = {c} >= 256"));
        }
        out.extend_from_slice(&(c as u16).to_le_bytes());
    }
    Ok(out)
}

/// Parse a raw `8,192`-byte (`4,096 * u16` little-endian, no header) `token_cells.u16`
/// byte slice into `4,096` cell IDs in `0..256`.
pub fn parse_token_cells_u16_bytes(bytes: &[u8], label: &str) -> Result<(Vec<u32>, usize), String> {
    if bytes.len() != CANONICAL_VOCAB * 2 {
        return Err(format!(
            "{label}: expected {} bytes (raw u16x4096 little-endian, no header), got {} bytes",
            CANONICAL_VOCAB * 2,
            bytes.len()
        ));
    }

    let mut token_cells = Vec::with_capacity(CANONICAL_VOCAB);
    let mut occupied = BTreeSet::new();
    for (idx, pair) in bytes.chunks_exact(2).enumerate() {
        let cell = u32::from(u16::from_le_bytes([pair[0], pair[1]]));
        if cell >= 256 {
            return Err(format!("{label}: token_cells[{idx}] = {cell} >= 256"));
        }
        token_cells.push(cell);
        occupied.insert(cell);
    }
    Ok((token_cells, occupied.len()))
}

fn parse_file_pin(val: Option<&Value>, label: &str) -> Result<FilePinRecord, String> {
    let raw = val.ok_or_else(|| format!("{label}: missing FilePin object"))?;
    let pin: FilePinRecord = serde_json::from_value(raw.clone())
        .map_err(|e| format!("{label}: invalid FilePin: {e}"))?;
    if pin.path.as_os_str().is_empty() || pin.sha256.is_empty() {
        return Err(format!(
            "{label}: FilePin path and sha256 must be non-empty"
        ));
    }
    Ok(pin)
}

/// Authenticate a raw `DriverCheckpoint` JSON byte slice across all four producer hash domains:
/// 1. Raw `DriverCheckpoint` JSON file (`checkpoint_file.sha256`, Domain D)
/// 2. Canonical typed `DriverCheckpointRecord` (`driver_checkpoint_sha256`, Domain C)
/// 3. Canonical `serde_json::Value` model (`checkpoint.model_sha256 == expected_model_value_sha256`, Domain A)
/// 4. Typed `DeviceSnapshotRecord` re-encoding (`eval.model_snapshot_sha256`, Domain B)
///
/// Returns the typed `DeviceSnapshotRecord` SHA256 (`eval.model_snapshot_sha256` Domain B).
#[allow(clippy::too_many_arguments)]
pub fn verify_driver_checkpoint_domains(
    ckpt_bytes: &[u8],
    expected_checkpoint_file_sha256: &str,
    expected_driver_checkpoint_sha256: &str,
    expected_config_sha256: &str,
    expected_lineage_sha256: &str,
    expected_model_value_sha256: &str,
    expected_model_epoch: u64,
    expected_completed_batches: u64,
    expected_model_bindings: &TokenCellsModelBindingsRecord,
) -> Result<String, String> {
    let actual_raw_sha = sha256_hex(ckpt_bytes);
    if actual_raw_sha != expected_checkpoint_file_sha256 {
        return Err(format!(
            "checkpoint: raw file sha256 ({actual_raw_sha}) != checkpoint_file.sha256 ({expected_checkpoint_file_sha256})"
        ));
    }
    let ckpt: DriverCheckpointRecord = serde_json::from_slice(ckpt_bytes)
        .map_err(|e| format!("checkpoint: invalid DriverCheckpoint JSON: {e}"))?;
    if ckpt.schema != DRIVER_CHECKPOINT_SCHEMA {
        return Err(format!(
            "checkpoint: expected schema '{DRIVER_CHECKPOINT_SCHEMA}', got '{}'",
            ckpt.schema
        ));
    }
    let actual_ckpt_sha = serde_json::to_vec(&ckpt)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("checkpoint: serialize DriverCheckpointRecord: {e}"))?;
    if actual_ckpt_sha != expected_driver_checkpoint_sha256 {
        return Err(format!(
            "checkpoint: canonical driver_checkpoint_sha256 ({actual_ckpt_sha}) != expected ({expected_driver_checkpoint_sha256})"
        ));
    }
    let actual_lin_sha = serde_json::to_vec(&ckpt.lineage)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("checkpoint.lineage: serialize: {e}"))?;
    if actual_lin_sha != expected_lineage_sha256 {
        return Err(format!(
            "checkpoint: lineage_sha256 ({actual_lin_sha}) != expected ({expected_lineage_sha256})"
        ));
    }
    let actual_cfg_sha = serde_json::to_vec(&ckpt.lineage.config)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("checkpoint.lineage.config: serialize: {e}"))?;
    if actual_cfg_sha != expected_config_sha256 {
        return Err(format!(
            "checkpoint: config_sha256 ({actual_cfg_sha}) != expected ({expected_config_sha256})"
        ));
    }
    if ckpt.cursor.accepted_updates != expected_model_epoch
        || ckpt.cursor.completed_batches != expected_completed_batches
    {
        return Err(format!(
            "checkpoint: cursor (completed_batches={}, accepted_updates={}) != expected ({expected_completed_batches}, {expected_model_epoch})",
            ckpt.cursor.completed_batches, ckpt.cursor.accepted_updates
        ));
    }
    let batch_rows = u64::from(ckpt.lineage.config.batch_rows);
    if batch_rows == 0 || batch_rows > 64 {
        return Err(format!("checkpoint: invalid batch_rows {batch_rows}"));
    }
    let batches_per_pass = ckpt.lineage.train.eligible_targets.div_ceil(batch_rows);
    if batches_per_pass == 0
        || ckpt.cursor.accepted_updates > ckpt.cursor.completed_batches
        || ckpt.cursor.corpus_pass != ckpt.cursor.completed_batches / batches_per_pass
        || ckpt.cursor.next_target
            != (ckpt.cursor.completed_batches % batches_per_pass) * batch_rows
    {
        return Err(
            "checkpoint: cursor differs from declared corpus/schedule cycle invariants".to_string(),
        );
    }

    validate_sha256_str(&ckpt.model_sha256, "checkpoint.model_sha256")?;
    let actual_model_val_sha = serde_json::to_vec(&ckpt.model)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("checkpoint.model: serialize Value: {e}"))?;
    if actual_model_val_sha != ckpt.model_sha256 {
        return Err(format!(
            "checkpoint: model Value sha256 ({actual_model_val_sha}) != checkpoint.model_sha256 ({})",
            ckpt.model_sha256
        ));
    }
    if actual_model_val_sha != expected_model_value_sha256 {
        return Err(format!(
            "checkpoint: model Value sha256 ({actual_model_val_sha}) != token_cells model storage/value sha256 ({expected_model_value_sha256})"
        ));
    }

    let snapshot: DeviceSnapshotRecord = serde_json::from_value(ckpt.model.clone())
        .map_err(|e| format!("checkpoint.model: invalid DeviceSnapshotRecord: {e}"))?;
    if snapshot.schema != DISCRETE_SNAPSHOT_SCHEMA {
        return Err(format!(
            "checkpoint.model: expected schema '{DISCRETE_SNAPSHOT_SCHEMA}', got '{}'",
            snapshot.schema
        ));
    }
    if snapshot.spec != ckpt.lineage.config.spec
        || snapshot.spec != expected_model_bindings.spec
        || snapshot.read_spec != ckpt.lineage.config.read_spec
        || snapshot.read_spec != expected_model_bindings.read_spec
        || snapshot.objective_policy != ckpt.lineage.config.objective_policy
        || snapshot.objective_policy != expected_model_bindings.objective_policy
        || snapshot.read_initialization != ckpt.lineage.config.read_initialization
        || snapshot.read_initialization != expected_model_bindings.read_initialization
        || snapshot.seed != ckpt.lineage.config.seed
        || snapshot.seed != expected_model_bindings.seed
        || snapshot.epoch != ckpt.cursor.accepted_updates
        || snapshot.epoch != expected_model_epoch
    {
        return Err(
            "checkpoint.model: spec/read_spec/objective_policy/read_initialization/seed/epoch disagrees with lineage or token_cells.model_bindings"
                .to_string(),
        );
    }
    validate_sha256_str(&snapshot.source_sha256, "checkpoint.model.source_sha256")?;
    validate_sha256_str(&snapshot.kernel_sha256, "checkpoint.model.kernel_sha256")?;
    validate_sha256_str(
        &snapshot.geometry_sha256,
        "checkpoint.model.geometry_sha256",
    )?;
    if snapshot.source_sha256 != expected_model_bindings.source_sha256
        || snapshot.kernel_sha256 != expected_model_bindings.kernel_sha256
        || snapshot.geometry_sha256 != expected_model_bindings.geometry_sha256
    {
        return Err(
            "checkpoint.model: source/kernel/geometry sha256 disagrees with token_cells.model_bindings"
                .to_string(),
        );
    }
    let expected_table_len =
        (snapshot.spec.vocabulary as usize) * (snapshot.spec.blocks as usize) * 8;
    if snapshot.table.len() != expected_table_len {
        return Err(format!(
            "checkpoint.model: table len {} != expected {}",
            snapshot.table.len(),
            expected_table_len
        ));
    }
    if snapshot.gates.iter().any(|t| !(-1..=1).contains(t)) {
        return Err("checkpoint.model: gate trit outside -1..=1".to_string());
    }
    if !ckpt.lineage.config.relation_learning_policy.is_learn() {
        let mut cap = 1usize;
        let mut stages = 0u32;
        while cap < snapshot.spec.context as usize {
            cap *= 3;
            stages += 1;
        }
        let combine_gates = (snapshot.spec.blocks * stages) as usize;
        if snapshot.gates.len() <= combine_gates
            || snapshot.gates[combine_gates..].iter().any(|&t| t != 0)
        {
            return Err(
                "checkpoint.model: FrozenZeroV1 requires every relation gate trit to be 0"
                    .to_string(),
            );
        }
    }

    serde_json::to_vec(&snapshot)
        .map(|b| sha256_hex(&b))
        .map_err(|e| format!("checkpoint.model: serialize DeviceSnapshotRecord: {e}"))
}

fn resolve_checkpoint_bytes_for_report(
    checkpoint_pin: &FilePinRecord,
    report_dir: &Path,
    explicit_checkpoint_path: Option<&Path>,
) -> Result<Vec<u8>, String> {
    if let Some(cp) = explicit_checkpoint_path {
        return fs::read(cp).map_err(|e| format!("explicit checkpoint ({}): {e}", cp.display()));
    }
    let mut candidates = Vec::new();
    candidates.push(checkpoint_pin.path.clone());
    if checkpoint_pin.path.is_relative() {
        candidates.push(report_dir.join(&checkpoint_pin.path));
    }
    if let (Some(parent), Some(fname)) = (
        checkpoint_pin.path.parent().and_then(|p| p.file_name()),
        checkpoint_pin.path.file_name(),
    ) {
        if let Some(rep_parent) = report_dir.parent() {
            candidates.push(rep_parent.join(parent).join(fname));
        }
        candidates.push(report_dir.join(parent).join(fname));
    }
    if let Some(fname) = checkpoint_pin.path.file_name() {
        if fname != std::ffi::OsStr::new("report.json") {
            candidates.push(report_dir.join(fname));
        }
    }
    candidates.push(report_dir.join("checkpoint.json"));
    candidates.push(report_dir.join("driver-checkpoint.json"));

    for cand in &candidates {
        if cand.is_file() {
            if let Ok(bytes) = fs::read(cand) {
                if sha256_hex(&bytes) == checkpoint_pin.sha256 {
                    return Ok(bytes);
                }
            }
        }
    }
    for cand in &candidates {
        if cand.is_file() {
            return fs::read(cand).map_err(|e| format!("checkpoint ({}): {e}", cand.display()));
        }
    }
    Err(format!(
        "token_cells report: cannot locate pinned DriverCheckpoint file '{}' (sha256={}) locally; pass --checkpoint <path> (or --relation-checkpoint / --control-checkpoint) to bind the local DriverCheckpoint file",
        checkpoint_pin.path.display(),
        checkpoint_pin.sha256
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_loaded_from_report_and_u16(
    rep_val: &Value,
    rep_sha: String,
    u16_bytes: &[u8],
    u16_label: &str,
    report_dir: &Path,
    explicit_checkpoint_path: Option<&Path>,
    expected_driver_checkpoint_sha256: &str,
    expected_model_snapshot_sha256: &str,
) -> Result<LoadedTokenCells, String> {
    if rep_val.get("schema").and_then(Value::as_str) != Some(TOKEN_CELLS_RECEIPT_SCHEMA) {
        return Err(format!(
            "token_cells report: expected schema '{TOKEN_CELLS_RECEIPT_SCHEMA}', got {:?}",
            rep_val.get("schema")
        ));
    }
    if rep_val.get("status").and_then(Value::as_str) != Some(TOKEN_CELLS_RECEIPT_STATUS) {
        return Err(format!(
            "token_cells report: expected status '{TOKEN_CELLS_RECEIPT_STATUS}', got {:?}",
            rep_val.get("status")
        ));
    }
    if rep_val.get("token_cells").is_some() {
        return Err(
            "token_cells report: inline token_cells JSON array is refused; require raw_u16_le_no_header token_cells.u16 paired with report.json"
                .to_string(),
        );
    }
    if rep_val.get("token_count").and_then(Value::as_u64) != Some(CANONICAL_VOCAB as u64)
        || rep_val.get("byte_count").and_then(Value::as_u64) != Some((CANONICAL_VOCAB * 2) as u64)
        || rep_val.get("encoding").and_then(Value::as_str) != Some("raw_u16_le_no_header")
        || rep_val.get("ordering").and_then(Value::as_str)
            != Some("ascending_token_id_0_through4095")
        || rep_val.get("cell_range") != Some(&json!({"min": 0, "max": 255}))
        || rep_val.get("absent_cells_allowed").and_then(Value::as_bool) != Some(false)
        || rep_val.get("model_restores").and_then(Value::as_u64) != Some(1)
        || rep_val.get("export_kernel_calls").and_then(Value::as_u64) != Some(1)
        || rep_val
            .get("token_cell_evaluations")
            .and_then(Value::as_u64)
            != Some(CANONICAL_VOCAB as u64)
        || rep_val.get("gpu_forward_calls").and_then(Value::as_u64) != Some(0)
        || rep_val.get("proposals").and_then(Value::as_u64) != Some(0)
        || rep_val.get("updates_applied").and_then(Value::as_u64) != Some(0)
        || rep_val.get("snapshot_unchanged").and_then(Value::as_bool) != Some(true)
        || rep_val
            .get("training_capability_issued")
            .and_then(Value::as_bool)
            != Some(false)
        || rep_val
            .get("host_cell_reconstruction")
            .and_then(Value::as_bool)
            != Some(false)
        || rep_val.get("training").and_then(Value::as_str) != Some("HELD")
        || rep_val.get("m2").and_then(Value::as_str) != Some("HELD")
        || rep_val.get("stage_b").and_then(Value::as_str) != Some("HELD")
        || rep_val.get("q4").and_then(Value::as_str) != Some("NOT_EXECUTABLE")
    {
        return Err(
            "token_cells report: mandatory #2601 zero-apply / raw_u16_le_no_header / hold invariants missing or violated"
                .to_string(),
        );
    }

    let config_file = parse_file_pin(rep_val.get("config_file"), "token_cells.config_file")?;
    let config_sha256 = rep_val
        .get("config_sha256")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "token_cells report: missing config_sha256".to_string())?
        .to_string();
    let lineage_sha256 = rep_val
        .get("lineage_sha256")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "token_cells report: missing lineage_sha256".to_string())?
        .to_string();
    let checkpoint_file = parse_file_pin(
        rep_val.get("checkpoint_file"),
        "token_cells.checkpoint_file",
    )?;
    let ckpt_sha = rep_val
        .get("driver_checkpoint_sha256")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "token_cells report: missing driver_checkpoint_sha256".to_string())?
        .to_string();
    let snap_sha = rep_val
        .get("model_snapshot_sha256")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "token_cells report: missing model_snapshot_sha256".to_string())?
        .to_string();
    validate_sha256_str(&snap_sha, "token_cells.model_snapshot_sha256")?;
    let dual_record_storage_sha = match rep_val.get("model_storage_sha256") {
        Some(v) => {
            let s = v
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    "token_cells report: model_storage_sha256 must be non-empty string".to_string()
                })?
                .to_string();
            validate_sha256_str(&s, "token_cells.model_storage_sha256")?;
            Some(s)
        }
        None => None,
    };
    let expected_storage_sha = dual_record_storage_sha
        .as_deref()
        .unwrap_or(snap_sha.as_str());
    let model_epoch = rep_val
        .get("model_epoch")
        .and_then(Value::as_u64)
        .ok_or_else(|| "token_cells report: missing model_epoch".to_string())?;
    let completed_batches = rep_val
        .get("completed_batches")
        .and_then(Value::as_u64)
        .ok_or_else(|| "token_cells report: missing completed_batches".to_string())?;
    let mb_raw = rep_val
        .get("model_bindings")
        .ok_or_else(|| "token_cells report: missing model_bindings".to_string())?;
    let model_bindings: TokenCellsModelBindingsRecord = serde_json::from_value(mb_raw.clone())
        .map_err(|e| format!("token_cells report: invalid model_bindings: {e}"))?;
    let token_cells_file = parse_file_pin(
        rep_val.get("token_cells_file"),
        "token_cells.token_cells_file",
    )?;

    if !expected_driver_checkpoint_sha256.is_empty()
        && ckpt_sha != expected_driver_checkpoint_sha256
    {
        return Err(format!(
            "token_cells report driver_checkpoint_sha256 ({ckpt_sha}) != expected ({expected_driver_checkpoint_sha256})"
        ));
    }

    let ckpt_bytes = resolve_checkpoint_bytes_for_report(
        &checkpoint_file,
        report_dir,
        explicit_checkpoint_path,
    )?;
    let typed_model_snapshot_sha256 = verify_driver_checkpoint_domains(
        &ckpt_bytes,
        &checkpoint_file.sha256,
        &ckpt_sha,
        &config_sha256,
        &lineage_sha256,
        expected_storage_sha,
        model_epoch,
        completed_batches,
        &model_bindings,
    )?;

    if dual_record_storage_sha.is_some() && snap_sha != typed_model_snapshot_sha256 {
        return Err(format!(
            "token_cells report: dual-record model_snapshot_sha256 ({snap_sha}) != checkpoint typed model snapshot sha256 ({typed_model_snapshot_sha256})"
        ));
    }

    if !expected_model_snapshot_sha256.is_empty()
        && typed_model_snapshot_sha256 != expected_model_snapshot_sha256
    {
        return Err(format!(
            "token_cells report typed model snapshot sha256 ({typed_model_snapshot_sha256}, storage={expected_storage_sha}) != expected ({expected_model_snapshot_sha256})"
        ));
    }

    let u16_sha = sha256_hex(u16_bytes);
    if token_cells_file.sha256 != u16_sha {
        return Err(format!(
            "token_cells.u16 sha256 ({u16_sha}) != report token_cells_file.sha256 ({})",
            token_cells_file.sha256
        ));
    }
    let (token_cells, occupied_cells) = parse_token_cells_u16_bytes(u16_bytes, u16_label)?;
    let storage_sha = expected_storage_sha.to_string();
    Ok(LoadedTokenCells {
        token_cells,
        occupied_cells,
        token_cells_sha256: u16_sha,
        report_sha256: rep_sha,
        config_sha256,
        config_file_sha256: config_file.sha256,
        lineage_sha256,
        driver_checkpoint_sha256: ckpt_sha,
        checkpoint_file_sha256: checkpoint_file.sha256,
        model_storage_sha256: storage_sha.clone(),
        model_snapshot_sha256: storage_sha,
        typed_model_snapshot_sha256,
        model_epoch,
        completed_batches,
        model_bindings,
    })
}

/// Load GPU-exported `token_cells[4096]` from either:
/// - a binary `token_cells.u16` file (`8,192` raw little-endian bytes) accompanied by a
///   required sibling `report.json` (`uor-r4.attention-token-cells/1`), or
/// - a JSON `report.json` (`uor-r4.attention-token-cells/1`) referencing its sibling
///   `token_cells.u16` binary file, and authenticate its pinned `DriverCheckpoint` across
///   all four producer hash domains.
///
/// Inline `token_cells` JSON arrays and bare `token_cells.u16` files without a sibling
/// `report.json` are refused.
pub fn load_gpu_token_cells_file(
    path: &Path,
    expected_driver_checkpoint_sha256: &str,
    expected_model_snapshot_sha256: &str,
) -> Result<LoadedTokenCells, String> {
    load_gpu_token_cells_file_with_checkpoint(
        path,
        None,
        expected_driver_checkpoint_sha256,
        expected_model_snapshot_sha256,
    )
}

pub fn load_gpu_token_cells_file_with_checkpoint(
    path: &Path,
    explicit_checkpoint_path: Option<&Path>,
    expected_driver_checkpoint_sha256: &str,
    expected_model_snapshot_sha256: &str,
) -> Result<LoadedTokenCells, String> {
    let bytes = fs::read(path).map_err(|e| format!("token_cells ({}): {e}", path.display()))?;
    let file_sha = sha256_hex(&bytes);
    let trimmed = bytes.iter().copied().find(|b| !b.is_ascii_whitespace());
    let report_dir = path.parent().unwrap_or_else(|| Path::new("."));

    if trimmed == Some(b'{') {
        let val: Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("token_cells JSON ({}): {e}", path.display()))?;
        if val.get("token_cells").is_some() {
            return Err(format!(
                "token_cells report ({}) contains inline token_cells JSON array; inline token_cells are refused — provide report.json paired with raw_u16_le_no_header token_cells.u16",
                path.display()
            ));
        }
        let u16_path = report_dir.join("token_cells.u16");
        let u16_bytes = fs::read(&u16_path).map_err(|e| {
            format!(
                "token_cells report ({}) sibling {} could not be read: {e}",
                path.display(),
                u16_path.display()
            )
        })?;
        return build_loaded_from_report_and_u16(
            &val,
            file_sha,
            &u16_bytes,
            &u16_path.display().to_string(),
            report_dir,
            explicit_checkpoint_path,
            expected_driver_checkpoint_sha256,
            expected_model_snapshot_sha256,
        );
    }

    // Binary token_cells.u16 file: require sibling report.json with uor-r4.attention-token-cells/1
    let sibling_report = report_dir.join("report.json");
    if !sibling_report.is_file() {
        return Err(format!(
            "bare token_cells.u16 ({}) without sibling report.json ({TOKEN_CELLS_RECEIPT_SCHEMA}) is refused; device-export receipt is required",
            path.display()
        ));
    }
    let rep_bytes = fs::read(&sibling_report)
        .map_err(|e| format!("sibling report ({}): {e}", sibling_report.display()))?;
    let rep_sha = sha256_hex(&rep_bytes);
    let rep_val: Value = serde_json::from_slice(&rep_bytes).map_err(|e| {
        format!(
            "sibling report ({}): invalid JSON: {e}",
            sibling_report.display()
        )
    })?;
    build_loaded_from_report_and_u16(
        &rep_val,
        rep_sha,
        &bytes,
        &path.display().to_string(),
        report_dir,
        explicit_checkpoint_path,
        expected_driver_checkpoint_sha256,
        expected_model_snapshot_sha256,
    )
}

fn parse_uort_v1(bytes: &[u8], label: &str) -> Result<Vec<u16>, String> {
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err(format!("{label}: missing UORT header magic"));
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != 1 {
        return Err(format!("{label}: unsupported UORT version {version}"));
    }
    let count = u64::from_le_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]) as usize;
    let vocab = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
    if vocab != CANONICAL_VOCAB {
        return Err(format!(
            "{label}: expected vocab {CANONICAL_VOCAB}, got {vocab}"
        ));
    }
    if bytes.len() != 64 + count * 2 {
        return Err(format!(
            "{label}: byte length {} != 64 + 2*{count}",
            bytes.len()
        ));
    }
    let mut out = Vec::with_capacity(count);
    for pair in bytes[64..].chunks_exact(2) {
        let tok = u16::from_le_bytes([pair[0], pair[1]]);
        if usize::from(tok) >= vocab {
            return Err(format!("{label}: token {tok} >= vocab {vocab}"));
        }
        out.push(tok);
    }
    Ok(out)
}

/// Build the registered non-model `T1`, `T1b`, `T2`, `U_succ`, and `U_train` reference
/// tables from the pinned `TRAIN` and `DEV` corpus files and cross-check every DEV row
/// against the sealed `d34-successor-table-record.json`.
pub fn build_corpus_reference_tables(
    paths: &CorpusPaths<'_>,
    d34_rows: &[ValidatedD34Row],
    enforce_canonical_pins: bool,
) -> Result<CorpusReferenceTables, String> {
    let train_tok_bytes = fs::read(paths.train_tokens).map_err(|e| format!("train_tokens: {e}"))?;
    let train_mask_bytes = fs::read(paths.train_mask).map_err(|e| format!("train_mask: {e}"))?;
    let dev_tok_bytes = fs::read(paths.dev_tokens).map_err(|e| format!("dev_tokens: {e}"))?;
    let dev_mask_bytes = fs::read(paths.dev_mask).map_err(|e| format!("dev_mask: {e}"))?;

    let train_tokens_sha256 = sha256_hex(&train_tok_bytes);
    let train_mask_sha256 = sha256_hex(&train_mask_bytes);
    let dev_tokens_sha256 = sha256_hex(&dev_tok_bytes);
    let dev_mask_sha256 = sha256_hex(&dev_mask_bytes);

    if enforce_canonical_pins {
        if train_tokens_sha256 != CANONICAL_TRAIN_TOKENS_SHA256 {
            return Err(format!(
                "train_tokens sha256 ({train_tokens_sha256}) != canonical ({CANONICAL_TRAIN_TOKENS_SHA256})"
            ));
        }
        if train_mask_sha256 != CANONICAL_TRAIN_MASK_SHA256 {
            return Err(format!(
                "train_mask sha256 ({train_mask_sha256}) != canonical ({CANONICAL_TRAIN_MASK_SHA256})"
            ));
        }
        if dev_tokens_sha256 != CANONICAL_DEV_TOKENS_SHA256 {
            return Err(format!(
                "dev_tokens sha256 ({dev_tokens_sha256}) != canonical ({CANONICAL_DEV_TOKENS_SHA256})"
            ));
        }
        if dev_mask_sha256 != CANONICAL_DEV_MASK_SHA256 {
            return Err(format!(
                "dev_mask sha256 ({dev_mask_sha256}) != canonical ({CANONICAL_DEV_MASK_SHA256})"
            ));
        }
    }

    let train_tokens = parse_uort_v1(&train_tok_bytes, "train_tokens")?;
    if train_mask_bytes.len() != train_tokens.len() || train_mask_bytes.iter().any(|&m| m > 1) {
        return Err("train_mask length or values invalid".to_string());
    }
    let dev_tokens = parse_uort_v1(&dev_tok_bytes, "dev_tokens")?;
    if dev_mask_bytes.len() != dev_tokens.len() || dev_mask_bytes.iter().any(|&m| m > 1) {
        return Err("dev_mask length or values invalid".to_string());
    }
    if train_tokens.first().copied() != Some(CANONICAL_BOS)
        || dev_tokens.first().copied() != Some(CANONICAL_BOS)
    {
        return Err("corpus tokens must start with BOS=0".to_string());
    }

    let train_eligible: Vec<usize> = (1..train_tokens.len())
        .filter(|&p| train_tokens[p] != CANONICAL_BOS && train_mask_bytes[p] == 1)
        .collect();
    if enforce_canonical_pins && train_eligible.len() != CANONICAL_TRAIN_ELIGIBLE_ROWS {
        return Err(format!(
            "train eligible targets {} != {CANONICAL_TRAIN_ELIGIBLE_ROWS}",
            train_eligible.len()
        ));
    }
    let matched_len = if enforce_canonical_pins {
        CANONICAL_TRAIN_MATCHED_TARGETS
    } else {
        train_eligible.len().min(CANONICAL_TRAIN_MATCHED_TARGETS)
    };
    let matched = &train_eligible[..matched_len];

    // Build TRAIN bigram counts[x][y] and unigram counts hist_train[y].
    let mut bigram_counts = vec![vec![0u32; CANONICAL_VOCAB]; CANONICAL_VOCAB];
    let mut u_train_counts = vec![0u32; CANONICAL_VOCAB];
    for &p in matched {
        let x = usize::from(train_tokens[p - 1]);
        let y = usize::from(train_tokens[p]);
        bigram_counts[x][y] += 1;
        u_train_counts[y] += 1;
    }

    let mut t2_ranked_by_query: Vec<Vec<u32>> = Vec::with_capacity(CANONICAL_VOCAB);
    for row in &bigram_counts {
        let mut pairs: Vec<(u32, u32)> = row
            .iter()
            .enumerate()
            .filter_map(|(y, &cnt)| (cnt > 0).then_some((cnt, y as u32)))
            .collect();
        pairs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        t2_ranked_by_query.push(pairs.into_iter().map(|(_, y)| y).collect());
    }

    let mut u_train_pairs: Vec<(u32, u32)> = u_train_counts
        .iter()
        .enumerate()
        .filter_map(|(y, &cnt)| (cnt > 0).then_some((cnt, y as u32)))
        .collect();
    u_train_pairs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let u_train_ranked: Vec<u32> = u_train_pairs.into_iter().map(|(_, y)| y).collect();
    let u_train_top1 = u_train_ranked.first().copied().unwrap_or(0);

    // Partition DEV and build per-row window query-occurrence successor lists.
    let documents: Vec<usize> = dev_tokens
        .iter()
        .enumerate()
        .filter_map(|(i, &t)| (t == CANONICAL_BOS).then_some(i))
        .collect();
    let dev_eligible: Vec<usize> = (1..dev_tokens.len())
        .filter(|&p| dev_tokens[p] != CANONICAL_BOS && dev_mask_bytes[p] == 1)
        .collect();
    if dev_eligible.len() != d34_rows.len() {
        return Err(format!(
            "dev_eligible len {} != d34_rows len {}",
            dev_eligible.len(),
            d34_rows.len()
        ));
    }

    let mut dev_rows = Vec::with_capacity(dev_eligible.len());
    let mut binding_lines = String::new();

    for (ordinal, &p) in dev_eligible.iter().enumerate() {
        let doc_idx = documents.partition_point(|&d| d <= p) - 1;
        let start = p.saturating_sub(CANONICAL_CONTEXT).max(documents[doc_idx]);
        let query = u32::from(dev_tokens[p - 1]);
        let target = u32::from(dev_tokens[p]);
        let mut most_recent: Option<usize> = None;
        let mut second_most_recent: Option<usize> = None;
        let mut any_succ = false;
        let mut present = dev_tokens[p - 1] == dev_tokens[p];
        let mut occ_successors = Vec::new();
        for j in start..(p - 1) {
            if u32::from(dev_tokens[j]) == target {
                present = true;
            }
            if u32::from(dev_tokens[j]) == query {
                second_most_recent = most_recent;
                most_recent = Some(j);
                let succ = u32::from(dev_tokens[j + 1]);
                occ_successors.push(succ);
                if succ == target {
                    any_succ = true;
                }
            }
        }
        occ_successors.reverse(); // Most recent first.
        let mut copy_successor = false;
        let mut bucket: Option<u8> = None;
        if let Some(mr) = most_recent {
            let distance = (p - 1) - mr;
            if u32::from(dev_tokens[mr + 1]) == target {
                copy_successor = true;
                bucket = Some(if distance <= 8 {
                    0
                } else if distance <= 32 {
                    1
                } else if distance <= 96 {
                    2
                } else {
                    3
                });
            }
        }
        let conflict = any_succ && !copy_successor;
        let t1_candidate = most_recent.map(|mr| u32::from(dev_tokens[mr + 1]));
        let t1b_candidate = second_most_recent.map(|smr| u32::from(dev_tokens[smr + 1]));

        // Cross-check every row against the sealed D34 per-row record.
        let d34_r = &d34_rows[ordinal];
        if d34_r.position != p as u64
            || d34_r.query_token != query
            || d34_r.target_token != target
            || d34_r.is_psucc != copy_successor
            || d34_r.distance_bucket != bucket
            || d34_r.conflict != conflict
        {
            return Err(format!(
                "DEV corpus partition row {ordinal} disagrees with sealed D34 row metadata"
            ));
        }
        let expected_t1b_pred = t1b_candidate.unwrap_or(u_train_top1);
        let expected_t2_pred = t2_ranked_by_query[query as usize]
            .first()
            .copied()
            .unwrap_or(u_train_top1);
        let expected_t2_top8: &[u32] = if t2_ranked_by_query[query as usize].is_empty() {
            &u_train_ranked[..u_train_ranked.len().min(8)]
        } else {
            let r = &t2_ranked_by_query[query as usize];
            &r[..r.len().min(8)]
        };
        if d34_r.t1b_prediction != expected_t1b_pred
            || d34_r.t1b_hit1 != (expected_t1b_pred == target)
            || d34_r.t2_prediction != expected_t2_pred
            || d34_r.t2_hit1 != (expected_t2_pred == target)
            || d34_r.t2_hit8 != expected_t2_top8.contains(&target)
            || d34_r.u_hit1 != (u_train_top1 == target)
            || d34_r.u_hit8 != u_train_ranked[..u_train_ranked.len().min(8)].contains(&target)
        {
            return Err(format!(
                "DEV row {ordinal} unconditional T1b/T2/U prediction or hit disagrees with sealed D34 record"
            ));
        }

        let bucket_label = match bucket {
            Some(0) => "1_8",
            Some(1) => "9_32",
            Some(2) => "33_96",
            Some(3) => "97_192",
            _ => "none",
        };
        use std::fmt::Write;
        let _ = writeln!(
            &mut binding_lines,
            "{ordinal}:{p}:{query}:{target}:{}:{}:{}:{}:{bucket_label}",
            u8::from(most_recent.is_some()),
            u8::from(copy_successor),
            u8::from(any_succ),
            u8::from(present),
        );

        dev_rows.push(DevCorpusRow {
            ordinal,
            position: p as u64,
            query_token: query,
            target_token: target,
            is_psucc: copy_successor,
            distance_bucket: bucket,
            conflict,
            t1_candidate,
            t1b_candidate,
            window_query_successors_most_recent_first: occ_successors,
        });
    }

    let ordered_row_binding_sha256 = sha256_hex(binding_lines.as_bytes());
    if enforce_canonical_pins && ordered_row_binding_sha256 != CANONICAL_ORDERED_ROW_BINDING_SHA256
    {
        return Err(format!(
            "rebuilt ordered_row_binding_sha256 ({ordered_row_binding_sha256}) != canonical ({CANONICAL_ORDERED_ROW_BINDING_SHA256})"
        ));
    }

    // Per D34 §6.4.6 and Kimi PM 6240513498, U_succ is the same model-matched
    // TRAIN unigram ranking (u_train_ranked, total count 1,049,856) scored on P_succ;
    // it must never be fitted on DEV labels.
    let u_succ_ranked: Vec<u32> = u_train_ranked.clone();

    let train_matched_targets = matched.len() as u64;
    let train_bigram_total_pairs: u64 = bigram_counts
        .iter()
        .flat_map(|row| row.iter())
        .map(|&c| u64::from(c))
        .sum();
    let u_train_total_counts: u64 = u_train_counts.iter().map(|&c| u64::from(c)).sum();
    let dev_psucc_rows = dev_rows.iter().filter(|r| r.is_psucc).count() as u64;
    let u_succ_total_counts: u64 = u_train_total_counts;
    let t1_all_match_target = dev_rows
        .iter()
        .filter(|r| r.is_psucc)
        .all(|r| r.t1_candidate == Some(r.target_token));

    if train_bigram_total_pairs != train_matched_targets
        || u_train_total_counts != train_matched_targets
        || u_succ_total_counts != train_matched_targets
        || !t1_all_match_target
    {
        return Err(format!(
            "corpus table conservation check failed: bigram_total={train_bigram_total_pairs} (expected {train_matched_targets}), u_train_total={u_train_total_counts}, u_succ_total={u_succ_total_counts} (expected {train_matched_targets}), t1_all_match={t1_all_match_target}"
        ));
    }
    if enforce_canonical_pins
        && (train_matched_targets != CANONICAL_TRAIN_MATCHED_TARGETS as u64
            || dev_psucc_rows != CANONICAL_DEV_PSUCC_ROWS as u64)
    {
        return Err(format!(
            "canonical conservation target mismatch: train_matched={train_matched_targets}, dev_psucc={dev_psucc_rows}"
        ));
    }

    let t2_ranked_table_sha256 = sha256_hex(
        &serde_json::to_vec(&t2_ranked_by_query).map_err(|e| format!("t2 json sha: {e}"))?,
    );
    let u_train_ranked_table_sha256 = sha256_hex(
        &serde_json::to_vec(&u_train_ranked).map_err(|e| format!("u_train json sha: {e}"))?,
    );
    let u_succ_ranked_table_sha256 = u_train_ranked_table_sha256.clone();
    let window_succs: Vec<&Vec<u32>> = dev_rows
        .iter()
        .map(|r| &r.window_query_successors_most_recent_first)
        .collect();
    let dev_window_successors_sha256 = sha256_hex(
        &serde_json::to_vec(&window_succs).map_err(|e| format!("window_succs json sha: {e}"))?,
    );

    Ok(CorpusReferenceTables {
        train_tokens_sha256,
        train_mask_sha256,
        dev_tokens_sha256,
        dev_mask_sha256,
        ordered_row_binding_sha256,
        train_matched_targets,
        train_bigram_total_pairs,
        u_train_total_counts,
        dev_psucc_rows,
        u_succ_total_counts,
        conservation_ok: true,
        t2_ranked_table_sha256,
        u_train_ranked_table_sha256,
        u_succ_ranked_table_sha256,
        dev_window_successors_sha256,
        t2_ranked_by_query,
        u_train_ranked,
        u_succ_ranked,
        dev_rows,
    })
}

#[derive(Debug, Clone)]
pub struct ValidatedRow {
    pub row_ordinal: usize,
    pub target_position: u64,
    pub target_token: u32,
    pub query_token: u32,
    pub is_psucc: bool,
    pub distance_bucket: Option<u8>,
    pub conflict: bool,
    pub eval_target_admitted: bool,
    pub active_cells: Vec<u32>,
    pub raw_cells: [u32; 16],
    pub eval_admitted_cells_count: u64,
    pub eval_address_rivals_count: u64,
    pub model_top1: bool,
    pub model_top8: bool,
    pub d34_t1b_prediction: u32,
    pub d34_t1b_hit1: bool,
    pub d34_t2_prediction: u32,
    pub d34_t2_hit1: bool,
    pub d34_t2_hit8: bool,
    pub d34_t3_prediction: u32,
    pub d34_t3_hit1: bool,
    pub d34_u_hit1: bool,
    pub d34_u_hit8: bool,
}

#[derive(Debug, Clone)]
pub struct ValidatedArmInputs {
    pub panel_sha256: String,
    pub capture_sha256: String,
    pub eval_sha256: String,
    pub d34_sha256: String,
    pub config_sha256: String,
    pub config_file_sha256: String,
    pub panel_config_sha256: String,
    pub panel_config_file_sha256: String,
    pub lineage_sha256: String,
    pub panel_lineage_sha256: String,
    pub driver_checkpoint_sha256: String,
    pub checkpoint_file_sha256: String,
    pub evaluation_file_sha256: String,
    pub model_storage_sha256: Option<String>,
    pub model_snapshot_sha256: String,
    pub ordered_row_binding_sha256: String,
    pub model_epoch: u64,
    pub completed_batches: u64,
    pub lineage: DriverLineageRecord,
    pub model_bindings: EvalModelBindingsRecord,
    pub rows: Vec<ValidatedRow>,
}

fn read_json_and_sha(path: &Path, label: &str) -> Result<(Value, String), String> {
    let bytes = fs::read(path).map_err(|e| format!("{label} ({}): {e}", path.display()))?;
    let sha = sha256_hex(&bytes);
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{label} ({}): invalid JSON: {e}", path.display()))?;
    Ok((value, sha))
}

type PanelRowTuple = (usize, u64, u32, bool);

fn parse_panel_rows(
    panel: &Value,
    enforce_canonical_pins: bool,
) -> Result<
    (
        String,
        String,
        String,
        CorpusBindingRecord,
        Vec<PanelRowTuple>,
    ),
    String,
> {
    if panel.get("schema").and_then(Value::as_str) != Some(PREFLIGHT_PANEL_SCHEMA) {
        return Err(format!(
            "panel: expected schema '{PREFLIGHT_PANEL_SCHEMA}', got {:?}",
            panel.get("schema")
        ));
    }
    if panel.get("status").and_then(Value::as_str) != Some("PINNED_PANEL_AND_PROBE_METADATA_ONLY")
        || panel.get("scoring_performed").and_then(Value::as_bool) != Some(false)
        || panel.get("cuda_context_created").and_then(Value::as_bool) != Some(false)
        || panel.get("model_constructed").and_then(Value::as_bool) != Some(false)
        || panel.get("updates_applied").and_then(Value::as_u64) != Some(0)
        || panel.get("release").and_then(Value::as_str) != Some("STILL_REQUIRED")
        || panel.get("training").and_then(Value::as_str) != Some("HELD")
        || panel.get("q4").and_then(Value::as_str) != Some("NOT_EXECUTABLE")
    {
        return Err("panel: zero-compute / zero-update / hold invariants violated".to_string());
    }
    let config_sha = panel
        .get("config_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "panel: missing config_sha256".to_string())?
        .to_string();
    let config_file: FilePinRecord = serde_json::from_value(
        panel
            .get("config_file")
            .cloned()
            .ok_or_else(|| "panel: missing config_file".to_string())?,
    )
    .map_err(|e| format!("panel.config_file: {e}"))?;
    if config_file.path.as_os_str().is_empty() || config_file.sha256.is_empty() {
        return Err("panel.config_file: path and sha256 must be non-empty".to_string());
    }
    let lineage_sha = panel
        .get("lineage_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "panel: missing lineage_sha256".to_string())?
        .to_string();
    let dev_binding: CorpusBindingRecord = serde_json::from_value(
        panel
            .get("dev_binding")
            .cloned()
            .ok_or_else(|| "panel: missing dev_binding".to_string())?,
    )
    .map_err(|e| format!("panel.dev_binding: {e}"))?;
    if dev_binding.source.split != SplitRecord::Dev
        || dev_binding.bos.token != "<|bos|>"
        || dev_binding.bos.id != u32::from(CANONICAL_BOS)
        || dev_binding.vocabulary != CANONICAL_VOCAB as u32
        || dev_binding.source.tokens.path.as_os_str().is_empty()
        || dev_binding.source.mask.path.as_os_str().is_empty()
    {
        return Err("panel.dev_binding: invalid DEV source or BOS metadata".to_string());
    }
    let tok_sha = dev_binding.source.tokens.sha256.as_str();
    let mask_sha = dev_binding.source.mask.sha256.as_str();
    if enforce_canonical_pins {
        validate_sha256_str(&config_sha, "panel.config_sha256")?;
        validate_sha256_str(&config_file.sha256, "panel.config_file.sha256")?;
        validate_sha256_str(&lineage_sha, "panel.lineage_sha256")?;
        if tok_sha != CANONICAL_DEV_TOKENS_SHA256 {
            return Err(format!(
                "panel: dev tokens sha256 '{tok_sha}' != canonical '{CANONICAL_DEV_TOKENS_SHA256}'"
            ));
        }
        if mask_sha != CANONICAL_DEV_MASK_SHA256 {
            return Err(format!(
                "panel: dev mask sha256 '{mask_sha}' != canonical '{CANONICAL_DEV_MASK_SHA256}'"
            ));
        }
        if dev_binding.eligible_targets != CANONICAL_DEV_ELIGIBLE_ROWS as u64 {
            return Err(format!(
                "panel.dev_binding: eligible_targets {} != {CANONICAL_DEV_ELIGIBLE_ROWS}",
                dev_binding.eligible_targets
            ));
        }
    }
    let rows_arr = panel
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "panel: missing rows array".to_string())?;
    if enforce_canonical_pins && rows_arr.len() != CANONICAL_DEV_ELIGIBLE_ROWS {
        return Err(format!(
            "panel: expected {CANONICAL_DEV_ELIGIBLE_ROWS} rows, got {}",
            rows_arr.len()
        ));
    }
    if dev_binding.eligible_targets != rows_arr.len() as u64 {
        return Err(format!(
            "panel.dev_binding.eligible_targets ({}) != panel rows ({})",
            dev_binding.eligible_targets,
            rows_arr.len()
        ));
    }
    let mut rows = Vec::with_capacity(rows_arr.len());
    let mut prev_pos: Option<u64> = None;
    let mut psucc_count = 0usize;
    for (idx, r) in rows_arr.iter().enumerate() {
        let ord = r
            .get("row_ordinal")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("panel row {idx}: missing row_ordinal"))?
            as usize;
        let pos = r
            .get("target_position")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("panel row {idx}: missing target_position"))?;
        let tok = r
            .get("target_token")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| format!("panel row {idx}: missing/invalid target_token"))?;
        let is_psucc = r
            .get("is_psucc")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("panel row {idx}: missing is_psucc"))?;
        if ord != idx {
            return Err(format!("panel row {idx}: row_ordinal {ord} != {idx}"));
        }
        if tok >= CANONICAL_VOCAB as u32 {
            return Err(format!(
                "panel row {idx}: target_token {tok} >= {CANONICAL_VOCAB}"
            ));
        }
        if let Some(p) = prev_pos {
            if pos <= p {
                return Err(format!(
                    "panel row {idx}: target_position {pos} not strictly ascending after {p}"
                ));
            }
        }
        prev_pos = Some(pos);
        if is_psucc {
            psucc_count += 1;
        }
        rows.push((ord, pos, tok, is_psucc));
    }
    if panel.get("row_count").and_then(Value::as_u64) != Some(rows.len() as u64)
        || panel.get("psucc_count").and_then(Value::as_u64) != Some(psucc_count as u64)
    {
        return Err("panel: row_count or psucc_count disagrees with rows array".to_string());
    }
    if enforce_canonical_pins && psucc_count != CANONICAL_DEV_PSUCC_ROWS {
        return Err(format!(
            "panel: expected {CANONICAL_DEV_PSUCC_ROWS} P_succ rows, got {psucc_count}"
        ));
    }
    Ok((
        config_sha,
        config_file.sha256,
        lineage_sha,
        dev_binding,
        rows,
    ))
}

#[derive(Debug, Clone)]
struct EvalRowObs {
    target_position: u64,
    target_token: u32,
    admitted_cells: u64,
    address_rivals: u64,
    top1_correct: bool,
    top8_correct: bool,
}

#[derive(Debug, Clone)]
struct ParsedEval {
    raw_eval_sha256: String,
    enveloped_eval_sha256: Option<String>,
    enveloped_driver_checkpoint_sha256: Option<String>,
    lineage_sha256: String,
    lineage: DriverLineageRecord,
    model_bindings: EvalModelBindingsRecord,
    model_storage_sha256: Option<String>,
    model_snapshot_sha256: String,
    model_epoch: u64,
    rows: Vec<EvalRowObs>,
}

/// Parse either a raw `uor-r4.discrete-corpus-driver/1` DEV evaluation record
/// (`training::DRIVER_SCHEMA`) or an `uor-r4.attention-learning-evaluation/1`
/// envelope wrapping a DEV evaluation (`transfer == null`).
fn parse_eval_rows(
    eval_input: &Value,
    eval_file_sha256: &str,
    expected_len: usize,
    enforce_canonical_pins: bool,
) -> Result<ParsedEval, String> {
    let top_schema = eval_input
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "eval: missing schema".to_string())?;
    let (eval, raw_sha, env_sha, env_ckpt_sha, env_storage_sha, env_snap_sha) = if top_schema
        == DRIVER_EVAL_SCHEMA
    {
        if eval_input.get("transfer_corpus").is_some() || eval_input.get("evaluation").is_some() {
            return Err(
                "eval: requires raw DEV eval-corpus report, not transfer or enveloped evidence"
                    .to_string(),
            );
        }
        (
            eval_input,
            eval_file_sha256.to_string(),
            None,
            None,
            None,
            None,
        )
    } else if top_schema == ENVELOPED_EVAL_SCHEMA {
        if eval_input.get("transfer") != Some(&Value::Null) {
            return Err("eval envelope: transfer must be null for DEV evaluation".to_string());
        }
        let ckpt_sha = eval_input
            .get("driver_checkpoint_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "eval envelope: missing driver_checkpoint_sha256".to_string())?
            .to_string();
        let env_storage = match eval_input.get("model_storage_sha256") {
            Some(v) => {
                let s = v
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        "eval envelope: model_storage_sha256 must be non-empty string".to_string()
                    })?
                    .to_string();
                validate_sha256_str(&s, "eval_envelope.model_storage_sha256")?;
                Some(s)
            }
            None => None,
        };
        let env_snap = match eval_input.get("model_snapshot_sha256") {
            Some(v) => {
                let s = v
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        "eval envelope: model_snapshot_sha256 must be non-empty string".to_string()
                    })?
                    .to_string();
                validate_sha256_str(&s, "eval_envelope.model_snapshot_sha256")?;
                Some(s)
            }
            None => None,
        };
        let inner = eval_input
            .get("evaluation")
            .ok_or_else(|| "eval envelope: missing inner evaluation".to_string())?;
        if inner.get("schema").and_then(Value::as_str) != Some(DRIVER_EVAL_SCHEMA)
            || inner.get("transfer_corpus").is_some()
        {
            return Err(
                "eval envelope: inner evaluation must have schema uor-r4.discrete-corpus-driver/1 without transfer_corpus"
                    .to_string(),
            );
        }
        let inner_bytes = serde_json::to_vec(inner).map_err(|e| format!("eval encode: {e}"))?;
        (
            inner,
            sha256_hex(&inner_bytes),
            Some(eval_file_sha256.to_string()),
            Some(ckpt_sha),
            env_storage,
            env_snap,
        )
    } else {
        return Err(format!(
            "eval: expected schema '{DRIVER_EVAL_SCHEMA}' or '{ENVELOPED_EVAL_SCHEMA}', got '{top_schema}'"
        ));
    };

    let lineage_sha256 = eval
        .get("evaluation_lineage_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "eval: missing evaluation_lineage_sha256".to_string())?
        .to_string();
    let lineage: DriverLineageRecord = serde_json::from_value(
        eval.get("evaluation_lineage")
            .cloned()
            .ok_or_else(|| "eval: missing evaluation_lineage".to_string())?,
    )
    .map_err(|e| format!("eval.evaluation_lineage: {e}"))?;
    let recomputed_lineage_sha = sha256_hex(
        &serde_json::to_vec(&lineage)
            .map_err(|e| format!("eval.evaluation_lineage encode: {e}"))?,
    );
    if recomputed_lineage_sha != lineage_sha256 {
        return Err(format!(
            "eval: recomputed evaluation_lineage sha256 ({recomputed_lineage_sha}) != evaluation_lineage_sha256 ({lineage_sha256})"
        ));
    }
    if lineage.schema != DRIVER_EVAL_SCHEMA {
        return Err(format!(
            "eval.evaluation_lineage: expected schema '{DRIVER_EVAL_SCHEMA}', got '{}'",
            lineage.schema
        ));
    }
    let cfg = &lineage.config;
    if cfg.schema != DRIVER_EVAL_SCHEMA {
        return Err(format!(
            "eval.evaluation_lineage.config: expected schema '{DRIVER_EVAL_SCHEMA}', got '{}'",
            cfg.schema
        ));
    }
    let expected_spec = SpecRecord {
        vocabulary: CANONICAL_VOCAB as u32,
        blocks: 4,
        context: CANONICAL_CONTEXT as u32,
        kept_cells: 8,
    };
    if cfg.spec != expected_spec {
        return Err(format!(
            "eval.evaluation_lineage.config.spec {:?} != canonical {:?}",
            cfg.spec, expected_spec
        ));
    }
    if cfg.objective_policy != ObjectivePolicyRecord::NearestAddressMarginV1
        || cfg.read_initialization != ReadInitializationRecord::ActiveLeafV1
        || cfg.schedule != SchedulePolicyRecord::InterleavedReadEvery8V1
        || cfg.batch_rows != 64
        || !matches!(cfg.seed, 1 | 2)
    {
        return Err(
            "eval.evaluation_lineage.config: objective_policy, read_initialization, schedule, batch_rows, or seed outside admitted contract"
                .to_string(),
        );
    }
    match cfg.read_spec {
        ReadSpecRecord::Flock { kept: 8 } | ReadSpecRecord::Relation { kept: 8, sweeps: 1 } => {}
        ref other => {
            return Err(format!(
                "eval.evaluation_lineage.config.read_spec {other:?} outside admitted K=8 contract"
            ))
        }
    }
    if cfg.train.split != SplitRecord::Train
        || cfg.dev.split != SplitRecord::Dev
        || cfg.bos.token != "<|bos|>"
        || cfg.bos.id != u32::from(CANONICAL_BOS)
        || lineage.train.bos != cfg.bos
        || lineage.dev.bos != cfg.bos
        || lineage.train.source != cfg.train
        || lineage.dev.source != cfg.dev
        || lineage.train.eligible_targets != cfg.train.expected_eligible_targets
        || lineage.dev.eligible_targets != cfg.dev.expected_eligible_targets
        || lineage.dev.eligible_targets != expected_len as u64
    {
        return Err(
            "eval.evaluation_lineage: train/dev source or eligible_targets binding mismatch"
                .to_string(),
        );
    }
    if enforce_canonical_pins
        && (cfg.tokenizer.sha256 != CANONICAL_TOKENIZER_SHA256
            || cfg.train.tokens.sha256 != CANONICAL_TRAIN_TOKENS_SHA256
            || cfg.train.mask.sha256 != CANONICAL_TRAIN_MASK_SHA256
            || cfg.dev.tokens.sha256 != CANONICAL_DEV_TOKENS_SHA256
            || cfg.dev.mask.sha256 != CANONICAL_DEV_MASK_SHA256
            || cfg.train.expected_eligible_targets != CANONICAL_TRAIN_ELIGIBLE_ROWS as u64
            || lineage.train.eligible_targets != CANONICAL_TRAIN_ELIGIBLE_ROWS as u64)
    {
        return Err(
            "eval.evaluation_lineage: canonical TRAIN/DEV corpus pin or eligible_targets mismatch"
                .to_string(),
        );
    }

    let model_bindings: EvalModelBindingsRecord = serde_json::from_value(
        eval.get("model_bindings")
            .cloned()
            .ok_or_else(|| "eval: missing model_bindings".to_string())?,
    )
    .map_err(|e| format!("eval.model_bindings: {e}"))?;
    if model_bindings.schema != DISCRETE_SNAPSHOT_SCHEMA
        || model_bindings.spec != cfg.spec
        || model_bindings.read_spec != cfg.read_spec
        || model_bindings.objective_policy != cfg.objective_policy
        || model_bindings.seed != cfg.seed
    {
        return Err(
            "eval.model_bindings disagrees with evaluation_lineage.config (schema, spec, read_spec, objective_policy, or seed)"
                .to_string(),
        );
    }
    validate_sha256_str(
        &model_bindings.source_sha256,
        "eval.model_bindings.source_sha256",
    )?;
    validate_sha256_str(
        &model_bindings.kernel_sha256,
        "eval.model_bindings.kernel_sha256",
    )?;
    validate_sha256_str(
        &model_bindings.geometry_sha256,
        "eval.model_bindings.geometry_sha256",
    )?;

    let model_snapshot_sha256 = eval
        .get("model_snapshot_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "eval: missing model_snapshot_sha256".to_string())?
        .to_string();
    let inner_storage_sha = match eval.get("model_storage_sha256") {
        Some(v) => {
            let s = v
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "eval: model_storage_sha256 must be non-empty string".to_string())?
                .to_string();
            validate_sha256_str(&s, "eval.model_storage_sha256")?;
            Some(s)
        }
        None => None,
    };
    if let Some(ref es) = env_snap_sha {
        if es != &model_snapshot_sha256 {
            return Err(format!(
                "eval envelope: model_snapshot_sha256 ({es}) != inner evaluation.model_snapshot_sha256 ({model_snapshot_sha256})"
            ));
        }
    }
    let model_storage_sha256 = match (env_storage_sha, inner_storage_sha) {
        (Some(es), Some(is)) => {
            if es != is {
                return Err(format!(
                    "eval envelope: model_storage_sha256 ({es}) != inner evaluation.model_storage_sha256 ({is})"
                ));
            }
            Some(is)
        }
        (Some(es), None) => Some(es),
        (None, Some(is)) => Some(is),
        (None, None) => None,
    };
    if enforce_canonical_pins {
        validate_sha256_str(&lineage_sha256, "eval.evaluation_lineage_sha256")?;
        validate_sha256_str(&model_snapshot_sha256, "eval.model_snapshot_sha256")?;
    }
    let model_epoch = eval
        .get("model_epoch")
        .and_then(Value::as_u64)
        .ok_or_else(|| "eval: missing model_epoch".to_string())?;
    let accepted_updates = eval
        .get("accepted_updates_in_model")
        .and_then(Value::as_u64)
        .ok_or_else(|| "eval: missing accepted_updates_in_model".to_string())?;
    if accepted_updates != model_epoch {
        return Err(format!(
            "eval: accepted_updates_in_model ({accepted_updates}) != model_epoch ({model_epoch})"
        ));
    }
    if eval
        .get("evaluation_mutates_model")
        .and_then(Value::as_bool)
        != Some(false)
        || eval
            .get("updates_applied_by_evaluation")
            .and_then(Value::as_u64)
            != Some(0)
    {
        return Err(
            "eval: evaluation_mutates_model must be false and updates_applied_by_evaluation must be 0"
                .to_string(),
        );
    }

    let batches = eval
        .get("batches")
        .and_then(Value::as_array)
        .ok_or_else(|| "eval: missing batches array".to_string())?;
    let mut rows = Vec::with_capacity(expected_len);
    for (b_idx, b) in batches.iter().enumerate() {
        let r_arr = b
            .get("rows")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("eval batch {b_idx}: missing rows array"))?;
        for r in r_arr {
            let ord = rows.len();
            let pos = r
                .get("target_position")
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("eval row {ord}: missing target_position"))?;
            let tok = r
                .get("target_token")
                .and_then(Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| format!("eval row {ord}: missing/invalid target_token"))?;
            let admitted = r
                .get("admitted_cells")
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("eval row {ord}: missing admitted_cells"))?;
            let rivals = r
                .get("address_rivals")
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("eval row {ord}: missing address_rivals"))?;
            let top1 = match r.get("top1_correct").and_then(Value::as_u64) {
                Some(0) => false,
                Some(1) => true,
                other => {
                    return Err(format!(
                        "eval row {ord}: top1_correct must be 0 or 1, got {other:?}"
                    ))
                }
            };
            let top8 = match r.get("top8_correct").and_then(Value::as_u64) {
                Some(0) => false,
                Some(1) => true,
                other => {
                    return Err(format!(
                        "eval row {ord}: top8_correct must be 0 or 1, got {other:?}"
                    ))
                }
            };
            if top1 && !top8 {
                return Err(format!(
                    "eval row {ord}: top1_correct=1 with top8_correct=0"
                ));
            }
            if admitted > 8 || rivals > 8 {
                return Err(format!(
                    "eval row {ord}: admitted_cells ({admitted}) or address_rivals ({rivals}) exceeds K8"
                ));
            }
            rows.push(EvalRowObs {
                target_position: pos,
                target_token: tok,
                admitted_cells: admitted,
                address_rivals: rivals,
                top1_correct: top1,
                top8_correct: top8,
            });
        }
    }
    if rows.len() != expected_len {
        return Err(format!(
            "eval: expected {expected_len} rows across batches, got {}",
            rows.len()
        ));
    }
    Ok(ParsedEval {
        raw_eval_sha256: raw_sha,
        enveloped_eval_sha256: env_sha,
        enveloped_driver_checkpoint_sha256: env_ckpt_sha,
        lineage_sha256,
        lineage,
        model_bindings,
        model_storage_sha256,
        model_snapshot_sha256,
        model_epoch,
        rows,
    })
}

#[derive(Debug, Clone)]
pub struct ValidatedD34Row {
    pub ordinal: usize,
    pub position: u64,
    pub query_token: u32,
    pub target_token: u32,
    pub is_psucc: bool,
    pub distance_bucket: Option<u8>,
    pub conflict: bool,
    pub t1b_prediction: u32,
    pub t1b_hit1: bool,
    pub t2_prediction: u32,
    pub t2_hit1: bool,
    pub t2_hit8: bool,
    pub t3_prediction: u32,
    pub t3_hit1: bool,
    pub u_hit1: bool,
    pub u_hit8: bool,
}

pub fn parse_d34_record(
    d34: &Value,
    d34_sha: &str,
    expected_len: usize,
    enforce_canonical_pins: bool,
) -> Result<(String, Vec<ValidatedD34Row>), String> {
    if d34.get("schema").and_then(Value::as_str) != Some(D34_RECORD_SCHEMA) {
        return Err(format!(
            "d34: expected schema '{D34_RECORD_SCHEMA}', got {:?}",
            d34.get("schema")
        ));
    }
    if enforce_canonical_pins && d34_sha != CANONICAL_D34_RECORD_SHA256 {
        return Err(format!(
            "d34: sha256 '{d34_sha}' != canonical '{CANONICAL_D34_RECORD_SHA256}'"
        ));
    }
    if d34.get("host_only").and_then(Value::as_bool) != Some(true)
        || d34.get("model_compute_executed").and_then(Value::as_bool) != Some(false)
        || d34.get("applied_updates").and_then(Value::as_u64) != Some(0)
    {
        return Err("d34: host_only / zero model compute invariants violated".to_string());
    }
    let binding_sha = d34
        .get("inputs")
        .and_then(|i| i.get("partition"))
        .and_then(|p| p.get("ordered_row_binding_sha256"))
        .and_then(Value::as_str)
        .ok_or_else(|| "d34: missing inputs.partition.ordered_row_binding_sha256".to_string())?
        .to_string();
    if enforce_canonical_pins && binding_sha != CANONICAL_ORDERED_ROW_BINDING_SHA256 {
        return Err(format!(
            "d34: ordered_row_binding_sha256 '{binding_sha}' != canonical '{CANONICAL_ORDERED_ROW_BINDING_SHA256}'"
        ));
    }
    let t2_pairs = d34
        .get("tables")
        .and_then(|t| t.get("T2"))
        .and_then(|t2| t2.get("pairs"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "d34: missing tables.T2.pairs".to_string())?;
    let t2_cons = d34
        .get("tables")
        .and_then(|t| t.get("T2"))
        .and_then(|t2| t2.get("conservation_ok"))
        .and_then(Value::as_bool)
        .ok_or_else(|| "d34: missing tables.T2.conservation_ok".to_string())?;
    if !t2_cons || (enforce_canonical_pins && t2_pairs != CANONICAL_TRAIN_MATCHED_TARGETS as u64) {
        return Err(format!(
            "d34: T2 conservation_ok={t2_cons}, pairs={t2_pairs}"
        ));
    }
    let per_row = d34
        .get("per_row")
        .and_then(Value::as_array)
        .ok_or_else(|| "d34: missing per_row array".to_string())?;
    if per_row.len() != expected_len {
        return Err(format!(
            "d34: expected {expected_len} rows in per_row, got {}",
            per_row.len()
        ));
    }
    let mut out = Vec::with_capacity(expected_len);
    for (idx, r) in per_row.iter().enumerate() {
        let ord = r
            .get("ordinal")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("d34 row {idx}: missing ordinal"))? as usize;
        if ord != idx {
            return Err(format!("d34 row {idx}: ordinal {ord} != {idx}"));
        }
        let pos = r
            .get("position")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("d34 row {idx}: missing position"))?;
        let q_tok = r
            .get("query_token")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| format!("d34 row {idx}: missing/invalid query_token"))?;
        let t_tok = r
            .get("target_token")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| format!("d34 row {idx}: missing/invalid target_token"))?;
        let is_psucc = r
            .get("copy_successor")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("d34 row {idx}: missing copy_successor"))?;
        let bucket = match r.get("distance_bucket") {
            Some(Value::Null) | None => None,
            Some(v) => {
                let b = v
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| format!("d34 row {idx}: invalid distance_bucket"))?;
                if b > 3 || !is_psucc {
                    return Err(format!(
                        "d34 row {idx}: invalid distance_bucket {b} for is_psucc={is_psucc}"
                    ));
                }
                Some(b)
            }
        };
        if is_psucc && bucket.is_none() {
            return Err(format!(
                "d34 row {idx}: copy_successor=true missing distance_bucket"
            ));
        }
        let conflict = r
            .get("conflict")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("d34 row {idx}: missing conflict"))?;
        if conflict && is_psucc {
            return Err(format!(
                "d34 row {idx}: conflict=true cannot be copy_successor=true"
            ));
        }
        let u32_pred = |field: &str| -> Result<u32, String> {
            let v = r
                .get(field)
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| format!("d34 row {idx}: missing/invalid {field}"))?;
            if v >= CANONICAL_VOCAB as u32 {
                return Err(format!(
                    "d34 row {idx}: {field}={v} >= vocab {CANONICAL_VOCAB}"
                ));
            }
            Ok(v)
        };
        let bit = |field: &str| -> Result<bool, String> {
            match r.get(field).and_then(Value::as_u64) {
                Some(0) => Ok(false),
                Some(1) => Ok(true),
                other => Err(format!(
                    "d34 row {idx}: {field} must be 0 or 1, got {other:?}"
                )),
            }
        };
        let t1b_pred = u32_pred("t1b_prediction")?;
        let t1b_hit1 = bit("t1b_hit1")?;
        let t2_pred = u32_pred("t2_prediction")?;
        let t2_hit1 = bit("t2_hit1")?;
        let t2_hit8 = bit("t2_hit8")?;
        let t3_pred = u32_pred("t3_prediction")?;
        let t3_hit1 = bit("t3_hit1")?;
        let u_hit1 = bit("u_hit1")?;
        let u_hit8 = bit("u_hit8")?;
        if t1b_hit1 != (t1b_pred == t_tok)
            || t2_hit1 != (t2_pred == t_tok)
            || t3_hit1 != (t3_pred == t_tok)
            || (t2_hit1 && !t2_hit8)
            || (u_hit1 && !u_hit8)
        {
            return Err(format!(
                "d34 row {idx}: internal prediction/hit1/hit8 inconsistency"
            ));
        }
        out.push(ValidatedD34Row {
            ordinal: ord,
            position: pos,
            query_token: q_tok,
            target_token: t_tok,
            is_psucc,
            distance_bucket: bucket,
            conflict,
            t1b_prediction: t1b_pred,
            t1b_hit1,
            t2_prediction: t2_pred,
            t2_hit1,
            t2_hit8,
            t3_prediction: t3_pred,
            t3_hit1,
            u_hit1,
            u_hit8,
        });
    }
    let psucc_t1_hits = out.iter().filter(|r| r.is_psucc).count() as u64;
    let recorded_t1 = d34
        .get("slices")
        .and_then(|s| s.get("copy_successor_subpanel"))
        .and_then(|c| c.get("t1_hit1"))
        .and_then(Value::as_u64);
    if recorded_t1 != Some(psucc_t1_hits) {
        return Err(format!(
            "d34: slices.copy_successor_subpanel.t1_hit1 ({recorded_t1:?}) != P_succ count ({psucc_t1_hits})"
        ));
    }
    Ok((binding_sha, out))
}

#[allow(clippy::too_many_arguments)]
pub fn validate_arm_inputs(
    panel: &Value,
    panel_sha: String,
    capture: &Value,
    capture_sha: String,
    eval: &Value,
    eval_sha: String,
    d34: &Value,
    d34_sha: String,
    enforce_canonical_pins: bool,
) -> Result<ValidatedArmInputs, String> {
    validate_arm_inputs_with_mode(
        panel,
        panel_sha,
        capture,
        capture_sha,
        eval,
        eval_sha,
        d34,
        d34_sha,
        enforce_canonical_pins,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validate_arm_inputs_with_mode(
    panel: &Value,
    panel_sha: String,
    capture: &Value,
    capture_sha: String,
    eval: &Value,
    eval_sha: String,
    d34: &Value,
    d34_sha: String,
    enforce_canonical_pins: bool,
    enforce_panel_config_lineage: bool,
) -> Result<ValidatedArmInputs, String> {
    let (panel_config_sha, panel_config_file_sha, panel_lineage_sha, panel_dev_binding, panel_rows) =
        parse_panel_rows(panel, enforce_canonical_pins)?;
    let expected_len = panel_rows.len();

    if capture.get("schema").and_then(Value::as_str) != Some(PREFLIGHT_CAPTURE_SCHEMA) {
        return Err(format!(
            "capture: expected schema '{PREFLIGHT_CAPTURE_SCHEMA}', got {:?}",
            capture.get("schema")
        ));
    }
    if capture.get("status").and_then(Value::as_str)
        != Some("ACTUAL_GPU_FORWARD_CELL_IDS_CAPTURED_ZERO_APPLY")
    {
        return Err(format!(
            "capture: unexpected status {:?}",
            capture.get("status")
        ));
    }
    let config_sha = capture
        .get("config_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "capture: missing config_sha256".to_string())?
        .to_string();
    let config_file: FilePinRecord = serde_json::from_value(
        capture
            .get("config_file")
            .cloned()
            .ok_or_else(|| "capture: missing config_file".to_string())?,
    )
    .map_err(|e| format!("capture.config_file: {e}"))?;
    if config_file.path.as_os_str().is_empty() || config_file.sha256.is_empty() {
        return Err("capture.config_file: path and sha256 must be non-empty".to_string());
    }
    let lineage_sha = capture
        .get("lineage_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "capture: missing lineage_sha256".to_string())?
        .to_string();
    if enforce_canonical_pins {
        validate_sha256_str(&config_sha, "capture.config_sha256")?;
        validate_sha256_str(&config_file.sha256, "capture.config_file.sha256")?;
        validate_sha256_str(&lineage_sha, "capture.lineage_sha256")?;
    }
    let capture_dev_binding: CorpusBindingRecord = serde_json::from_value(
        capture
            .get("dev_binding")
            .cloned()
            .ok_or_else(|| "capture: missing dev_binding".to_string())?,
    )
    .map_err(|e| format!("capture.dev_binding: {e}"))?;
    if capture_dev_binding != panel_dev_binding {
        return Err("capture: dev_binding differs from panel".to_string());
    }
    if enforce_panel_config_lineage {
        if config_sha != panel_config_sha || lineage_sha != panel_lineage_sha {
            return Err(format!(
                "capture: config_sha256 ({config_sha}) or lineage_sha256 ({lineage_sha}) differs from panel ({panel_config_sha}, {panel_lineage_sha})"
            ));
        }
        if panel_config_file_sha != config_file.sha256 {
            return Err(format!(
                "capture: config_file.sha256 ({}) != panel.config_file.sha256 ({panel_config_file_sha})",
                config_file.sha256
            ));
        }
    }
    let expected_gpu_forward_calls = (expected_len as u64).div_ceil(64);
    if capture
        .get("physical_cell_words_per_row")
        .and_then(Value::as_u64)
        != Some(16)
        || capture.get("unused_slot_marker").and_then(Value::as_u64)
            != Some(u64::from(UNUSED_CELL_SLOT))
        || capture.get("row_count").and_then(Value::as_u64) != Some(expected_len as u64)
        || capture
            .get("additional_zero_apply_gpu_forward_passes")
            .and_then(Value::as_u64)
            != Some(1)
        || capture.get("gpu_forward_calls").and_then(Value::as_u64)
            != Some(expected_gpu_forward_calls)
        || capture.get("model_restores").and_then(Value::as_u64) != Some(1)
        || capture.get("updates_applied").and_then(Value::as_u64) != Some(0)
        || capture.get("proposals").and_then(Value::as_u64) != Some(0)
        || capture.get("snapshot_unchanged").and_then(Value::as_bool) != Some(true)
        || capture
            .get("training_capability_issued")
            .and_then(Value::as_bool)
            != Some(false)
        || capture.get("training").and_then(Value::as_str) != Some("HELD")
        || capture.get("m2").and_then(Value::as_str) != Some("HELD")
        || capture.get("stage_b").and_then(Value::as_str) != Some("HELD")
        || capture.get("q4").and_then(Value::as_str) != Some("NOT_EXECUTABLE")
    {
        return Err(
            "capture: physical16-cell / zero-apply / unchanged-snapshot / forward-call / hold invariants violated"
                .to_string(),
        );
    }
    let driver_ckpt_sha = capture
        .get("driver_checkpoint_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "capture: missing driver_checkpoint_sha256".to_string())?
        .to_string();
    let checkpoint_file: FilePinRecord = serde_json::from_value(
        capture
            .get("checkpoint_file")
            .cloned()
            .ok_or_else(|| "capture: missing checkpoint_file".to_string())?,
    )
    .map_err(|e| format!("capture.checkpoint_file: {e}"))?;
    if checkpoint_file.path.as_os_str().is_empty() || checkpoint_file.sha256.is_empty() {
        return Err("capture.checkpoint_file: path and sha256 must be non-empty".to_string());
    }
    let evaluation_file: FilePinRecord = serde_json::from_value(
        capture
            .get("evaluation_file")
            .cloned()
            .ok_or_else(|| "capture: missing evaluation_file".to_string())?,
    )
    .map_err(|e| format!("capture.evaluation_file: {e}"))?;
    if evaluation_file.path.as_os_str().is_empty() || evaluation_file.sha256.is_empty() {
        return Err("capture.evaluation_file: path and sha256 must be non-empty".to_string());
    }
    if enforce_canonical_pins {
        validate_sha256_str(&driver_ckpt_sha, "capture.driver_checkpoint_sha256")?;
        validate_sha256_str(&checkpoint_file.sha256, "capture.checkpoint_file.sha256")?;
        validate_sha256_str(&evaluation_file.sha256, "capture.evaluation_file.sha256")?;
    }
    let model_epoch = capture
        .get("model_epoch")
        .and_then(Value::as_u64)
        .ok_or_else(|| "capture: missing model_epoch".to_string())?;
    let completed_batches = capture
        .get("completed_batches")
        .and_then(Value::as_u64)
        .ok_or_else(|| "capture: missing completed_batches".to_string())?;

    let parsed_eval = parse_eval_rows(eval, &eval_sha, expected_len, enforce_canonical_pins)?;
    if parsed_eval.lineage_sha256 != lineage_sha {
        return Err(format!(
            "eval: evaluation_lineage_sha256 ({}) != capture.lineage_sha256 ({lineage_sha})",
            parsed_eval.lineage_sha256
        ));
    }
    let recomputed_config_sha = sha256_hex(
        &serde_json::to_vec(&parsed_eval.lineage.config)
            .map_err(|e| format!("eval.evaluation_lineage.config encode: {e}"))?,
    );
    if recomputed_config_sha != config_sha {
        return Err(format!(
            "eval: recomputed evaluation_lineage.config sha256 ({recomputed_config_sha}) != capture.config_sha256 ({config_sha})"
        ));
    }
    if capture_dev_binding != parsed_eval.lineage.dev
        || panel_dev_binding != parsed_eval.lineage.dev
    {
        return Err(
            "capture/panel dev_binding disagrees with eval.evaluation_lineage.dev".to_string(),
        );
    }
    if parsed_eval.model_epoch != model_epoch {
        return Err(format!(
            "eval: model_epoch ({}) != capture.model_epoch ({model_epoch})",
            parsed_eval.model_epoch
        ));
    }
    if let Some(cap_snap_val) = capture.get("model_snapshot_sha256") {
        let cap_snap_sha = cap_snap_val
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "capture: model_snapshot_sha256 must be non-empty string".to_string())?;
        validate_sha256_str(cap_snap_sha, "capture.model_snapshot_sha256")?;
        if cap_snap_sha != parsed_eval.model_snapshot_sha256 {
            return Err(format!(
                "capture: model_snapshot_sha256 ({cap_snap_sha}) != eval.model_snapshot_sha256 ({})",
                parsed_eval.model_snapshot_sha256
            ));
        }
    }
    let cap_storage_sha = match capture.get("model_storage_sha256") {
        Some(v) => {
            let s = v
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    "capture: model_storage_sha256 must be non-empty string".to_string()
                })?
                .to_string();
            validate_sha256_str(&s, "capture.model_storage_sha256")?;
            if let Some(ref eval_storage) = parsed_eval.model_storage_sha256 {
                if &s != eval_storage {
                    return Err(format!(
                        "capture: model_storage_sha256 ({s}) != eval.model_storage_sha256 ({eval_storage})"
                    ));
                }
            }
            Some(s)
        }
        None => None,
    };
    let model_storage_sha256 = parsed_eval.model_storage_sha256.or(cap_storage_sha);
    if let Some(ref env_ckpt) = parsed_eval.enveloped_driver_checkpoint_sha256 {
        if env_ckpt != &driver_ckpt_sha {
            return Err(format!(
                "eval envelope: driver_checkpoint_sha256 ({env_ckpt}) != capture.driver_checkpoint_sha256 ({driver_ckpt_sha})"
            ));
        }
    }
    if evaluation_file.sha256 != parsed_eval.raw_eval_sha256 && evaluation_file.sha256 != eval_sha {
        return Err(format!(
            "capture: evaluation_file.sha256 ({}) matches neither raw eval sha256 ({}) nor file sha256 ({eval_sha})",
            evaluation_file.sha256, parsed_eval.raw_eval_sha256
        ));
    }

    let (ordered_row_binding_sha256, d34_rows) =
        parse_d34_record(d34, &d34_sha, expected_len, enforce_canonical_pins)?;

    let cap_rows = capture
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "capture: missing rows array".to_string())?;
    if cap_rows.len() != expected_len {
        return Err(format!(
            "capture: expected {expected_len} rows, got {}",
            cap_rows.len()
        ));
    }

    let mut validated = Vec::with_capacity(expected_len);
    let mut b3_full = 0u64;
    let mut b3_psucc = 0u64;
    let mut b3_comp = 0u64;

    for idx in 0..expected_len {
        let (p_ord, p_pos, p_tok, p_psucc) = panel_rows[idx];
        let c_row = &cap_rows[idx];
        let e_row = &parsed_eval.rows[idx];
        let d_row = &d34_rows[idx];

        let c_ord = c_row
            .get("row_ordinal")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("capture row {idx}: missing row_ordinal"))?
            as usize;
        let c_pos = c_row
            .get("target_position")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("capture row {idx}: missing target_position"))?;
        let c_tok = c_row
            .get("target_token")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| format!("capture row {idx}: missing/invalid target_token"))?;
        let c_psucc = c_row
            .get("is_psucc")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("capture row {idx}: missing is_psucc"))?;
        let c_adm = c_row
            .get("eval_target_admitted")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("capture row {idx}: missing eval_target_admitted"))?;
        let c_epoch = c_row
            .get("model_epoch")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("capture row {idx}: missing model_epoch"))?;

        if c_ord != p_ord
            || c_pos != p_pos
            || c_tok != p_tok
            || c_psucc != p_psucc
            || c_epoch != model_epoch
        {
            return Err(format!(
                "capture row {idx}: identity or model_epoch mismatch against panel"
            ));
        }
        if e_row.target_position != p_pos || e_row.target_token != p_tok {
            return Err(format!("eval row {idx}: identity mismatch against panel"));
        }
        if d_row.ordinal != p_ord
            || d_row.position != p_pos
            || d_row.target_token != p_tok
            || d_row.is_psucc != p_psucc
        {
            return Err(format!("d34 row {idx}: identity mismatch against panel"));
        }

        let eval_target_adm = match e_row.admitted_cells.checked_sub(e_row.address_rivals) {
            Some(0) => false,
            Some(1) => true,
            _ => {
                return Err(format!(
                    "eval row {idx}: admitted_cells ({}) - address_rivals ({}) not in {{0,1}}",
                    e_row.admitted_cells, e_row.address_rivals
                ))
            }
        };
        if c_adm != eval_target_adm {
            return Err(format!(
                "row {idx}: capture.eval_target_admitted ({c_adm}) != eval (admitted_cells - address_rivals == 1) ({eval_target_adm})"
            ));
        }

        let cells_arr = c_row
            .get("admitted_cells")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("capture row {idx}: admitted_cells must be a 16-word array"))?;
        if cells_arr.len() != 16 {
            return Err(format!(
                "capture row {idx}: admitted_cells length {} != 16",
                cells_arr.len()
            ));
        }
        let mut raw_cells = [UNUSED_CELL_SLOT; 16];
        let mut active_cells = Vec::with_capacity(8);
        let mut seen_active = BTreeSet::new();
        let mut seen_unused = false;
        for (slot_idx, slot_val) in cells_arr.iter().enumerate() {
            let cell = slot_val
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| format!("capture row {idx} slot {slot_idx}: not a u32"))?;
            raw_cells[slot_idx] = cell;
            if cell == UNUSED_CELL_SLOT {
                seen_unused = true;
            } else {
                if seen_unused {
                    return Err(format!(
                        "capture row {idx}: active cell {cell} at slot {slot_idx} after unused marker"
                    ));
                }
                if cell >= 256 {
                    return Err(format!(
                        "capture row {idx} slot {slot_idx}: cell ID {cell} >= 256"
                    ));
                }
                if !seen_active.insert(cell) {
                    return Err(format!(
                        "capture row {idx}: duplicate active cell ID {cell}"
                    ));
                }
                active_cells.push(cell);
            }
        }
        if active_cells.len() > 8 {
            return Err(format!(
                "capture row {idx}: {} active cells exceeds K=8",
                active_cells.len()
            ));
        }
        if active_cells.len() as u64 != e_row.admitted_cells {
            return Err(format!(
                "row {idx}: capture active cell count ({}) != eval admitted_cells ({})",
                active_cells.len(),
                e_row.admitted_cells
            ));
        }

        if c_adm {
            b3_full += 1;
            if p_psucc {
                b3_psucc += 1;
            } else {
                b3_comp += 1;
            }
        }

        validated.push(ValidatedRow {
            row_ordinal: p_ord,
            target_position: p_pos,
            target_token: p_tok,
            query_token: d_row.query_token,
            is_psucc: p_psucc,
            distance_bucket: d_row.distance_bucket,
            conflict: d_row.conflict,
            eval_target_admitted: c_adm,
            active_cells,
            raw_cells,
            eval_admitted_cells_count: e_row.admitted_cells,
            eval_address_rivals_count: e_row.address_rivals,
            model_top1: e_row.top1_correct,
            model_top8: e_row.top8_correct,
            d34_t1b_prediction: d_row.t1b_prediction,
            d34_t1b_hit1: d_row.t1b_hit1,
            d34_t2_prediction: d_row.t2_prediction,
            d34_t2_hit1: d_row.t2_hit1,
            d34_t2_hit8: d_row.t2_hit8,
            d34_t3_prediction: d_row.t3_prediction,
            d34_t3_hit1: d_row.t3_hit1,
            d34_u_hit1: d_row.u_hit1,
            d34_u_hit8: d_row.u_hit8,
        });
    }

    // Require capture.b3 summary object and cross-check its counts.
    let cap_b3 = capture
        .get("b3")
        .ok_or_else(|| "capture: missing b3 summary object".to_string())?;
    let check_b3 = |key: &str, exp_rows: u64, exp_adm: u64| -> Result<(), String> {
        let s = cap_b3
            .get(key)
            .ok_or_else(|| format!("capture.b3: missing {key}"))?;
        if s.get("rows").and_then(Value::as_u64) != Some(exp_rows)
            || s.get("target_admitted").and_then(Value::as_u64) != Some(exp_adm)
        {
            return Err(format!("capture.b3.{key} disagrees with row tally"));
        }
        Ok(())
    };
    let psucc_rows = validated.iter().filter(|r| r.is_psucc).count() as u64;
    let comp_rows = (validated.len() as u64) - psucc_rows;
    check_b3("full", validated.len() as u64, b3_full)?;
    check_b3("psucc", psucc_rows, b3_psucc)?;
    check_b3("complement", comp_rows, b3_comp)?;

    let _ = parsed_eval.enveloped_eval_sha256;
    Ok(ValidatedArmInputs {
        panel_sha256: panel_sha,
        capture_sha256: capture_sha,
        eval_sha256: eval_sha,
        d34_sha256: d34_sha,
        config_sha256: config_sha,
        config_file_sha256: config_file.sha256,
        panel_config_sha256: panel_config_sha,
        panel_config_file_sha256: panel_config_file_sha,
        lineage_sha256: lineage_sha,
        panel_lineage_sha256: panel_lineage_sha,
        driver_checkpoint_sha256: driver_ckpt_sha,
        checkpoint_file_sha256: checkpoint_file.sha256,
        evaluation_file_sha256: evaluation_file.sha256,
        model_storage_sha256,
        model_snapshot_sha256: parsed_eval.model_snapshot_sha256,
        ordered_row_binding_sha256,
        model_epoch,
        completed_batches,
        lineage: parsed_eval.lineage,
        model_bindings: parsed_eval.model_bindings,
        rows: validated,
    })
}

fn rate_json(hits: u64, total: u64) -> Value {
    json!({
        "hits": hits,
        "total": total,
    })
}

fn score_slice(rows: &[&ValidatedRow]) -> Value {
    let n = rows.len() as u64;
    let b3_rows: Vec<&&ValidatedRow> = rows.iter().filter(|r| r.eval_target_admitted).collect();
    let b3_n = b3_rows.len() as u64;

    let count_if =
        |pred: fn(&ValidatedRow) -> bool| -> u64 { rows.iter().filter(|r| pred(r)).count() as u64 };
    let count_b3 = |pred: fn(&ValidatedRow) -> bool| -> u64 {
        b3_rows.iter().filter(|r| pred(r)).count() as u64
    };

    json!({
        "panel_rows": n,
        "b3_target_admitted": b3_n,
        "model": {
            "top1": rate_json(count_if(|r| r.model_top1), n),
            "top8": rate_json(count_if(|r| r.model_top8), n),
            "top1_on_b3": rate_json(count_b3(|r| r.model_top1), b3_n),
            "top8_on_b3": rate_json(count_b3(|r| r.model_top8), b3_n),
        },
        "unconditional_sealed_d34": {
            "t1_hit1": rate_json(count_if(|r| r.is_psucc), n),
            "t1b_hit1": rate_json(count_if(|r| r.d34_t1b_hit1), n),
            "t2_hit1": rate_json(count_if(|r| r.d34_t2_hit1), n),
            "t2_hit8": rate_json(count_if(|r| r.d34_t2_hit8), n),
            "t3_hit1": rate_json(count_if(|r| r.d34_t3_hit1), n),
            "u_hit1": rate_json(count_if(|r| r.d34_u_hit1), n),
            "u_hit8": rate_json(count_if(|r| r.d34_u_hit8), n),
        },
        "on_gpu_target_admitted_b3_rows": {
            "note": "Scores the sealed D34 unconditional prediction on rows where the GPU admitted the target cell (eval_target_admitted == true). Lower bound on same-cell re-ranked T2/U_succ.",
            "t1_hit1": rate_json(count_b3(|r| r.is_psucc), b3_n),
            "t1b_hit1": rate_json(count_b3(|r| r.d34_t1b_hit1), b3_n),
            "t2_hit1": rate_json(count_b3(|r| r.d34_t2_hit1), b3_n),
            "t2_hit8": rate_json(count_b3(|r| r.d34_t2_hit8), b3_n),
            "t3_hit1": rate_json(count_b3(|r| r.d34_t3_hit1), b3_n),
            "u_hit1": rate_json(count_b3(|r| r.d34_u_hit1), b3_n),
            "u_hit8": rate_json(count_b3(|r| r.d34_u_hit8), b3_n),
        }
    })
}

/// Verify that `token_cells[target_token]` is in `row.active_cells` if and only if
/// `row.eval_target_admitted` is true across all 14,287 DEV rows.
pub fn verify_token_cells_against_gpu_capture(
    arm: &ValidatedArmInputs,
    token_cells: &[u32],
) -> Result<(), String> {
    if token_cells.len() != CANONICAL_VOCAB || token_cells.iter().any(|&c| c >= 256) {
        return Err(format!(
            "token_cells must have length {CANONICAL_VOCAB} with values in 0..256"
        ));
    }
    for row in &arm.rows {
        let target_cell = token_cells[row.target_token as usize];
        let in_captured_slots = row.active_cells.contains(&target_cell);
        if in_captured_slots != row.eval_target_admitted {
            return Err(format!(
                "row {}: token_cells[target={}]={target_cell} in captured slots is {in_captured_slots}, but eval_target_admitted is {}",
                row.row_ordinal, row.target_token, row.eval_target_admitted
            ));
        }
    }
    Ok(())
}

/// Per-row same-cell re-ranked predictions and hit flags for a given checkpoint's
/// `token_cells: [u32; 4096]` and GPU-captured `admitted_cells[16]`.
#[derive(Debug, Clone)]
pub struct SameCellRowScore {
    pub t1_admitted_hit1: bool,
    pub t1b_unconditional_in_admitted_hit1: bool,
    pub t1b_nearest_admitted_occ_hit1: bool,
    pub t2_same_cell_hit1: bool,
    pub t2_same_cell_strict_no_fallback_hit1: bool,
    pub t2_same_cell_hit8: bool,
    pub u_succ_same_cell_hit1: bool,
    pub u_succ_same_cell_hit8: bool,
    pub u_train_same_cell_hit1: bool,
    pub u_train_same_cell_hit8: bool,
}

pub fn score_same_cell_rows_from_corpus(
    arm: &ValidatedArmInputs,
    token_cells: &[u32],
    corpus_tables: &CorpusReferenceTables,
) -> Result<Vec<SameCellRowScore>, String> {
    verify_token_cells_against_gpu_capture(arm, token_cells)?;
    if corpus_tables.dev_rows.len() != arm.rows.len() {
        return Err("corpus_tables.dev_rows length mismatch".to_string());
    }
    let mut out = Vec::with_capacity(arm.rows.len());
    for (idx, row) in arm.rows.iter().enumerate() {
        let c_row = &corpus_tables.dev_rows[idx];
        let is_admitted =
            |tok: u32| -> bool { row.active_cells.contains(&token_cells[tok as usize]) };

        let u_train_adm_top1 = corpus_tables
            .u_train_ranked
            .iter()
            .copied()
            .find(|&y| is_admitted(y));
        let u_succ_adm_top1 = corpus_tables
            .u_succ_ranked
            .iter()
            .copied()
            .find(|&y| is_admitted(y))
            .or(u_train_adm_top1);

        // T1 same-cell: on P_succ, c_row.t1_candidate == Some(target_token).
        let t1_admitted_hit1 = c_row
            .t1_candidate
            .is_some_and(|y1| is_admitted(y1) && y1 == row.target_token);

        // T1b same-cell:
        // (a) whether the second-most-recent query occurrence's successor is admitted and equals target
        let t1b_unconditional_in_admitted_hit1 = match c_row.t1b_candidate {
            Some(y1b) if is_admitted(y1b) => y1b == row.target_token,
            _ => u_train_adm_top1 == Some(row.target_token),
        };
        // (b) second-most-recent occurrence whose successor is in S_r
        let admitted_occ_succs: Vec<u32> = c_row
            .window_query_successors_most_recent_first
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .collect();
        let t1b_nearest_admitted_occ_hit1 =
            admitted_occ_succs.get(1).copied().or(u_train_adm_top1) == Some(row.target_token);

        // T2 same-cell (m1-learned-attention-link-2026-10-17.md §5.2 item 2):
        // Rank y in S_r by descending count_TRAIN(x_r -> y), tie-break lowest token id,
        // with unigram fallback within S_r when max_{y in S_r} count_TRAIN(x_r -> y) == 0.
        let t2_q = &corpus_tables.t2_ranked_by_query[row.query_token as usize];
        let mut t2_adm: Vec<u32> = t2_q.iter().copied().filter(|&y| is_admitted(y)).collect();
        let t2_strict_top1 = t2_adm.first().copied();
        let t2_top1_with_fallback = t2_strict_top1.or(u_train_adm_top1);
        if t2_adm.len() < 8 {
            for &u_tok in &corpus_tables.u_train_ranked {
                if t2_adm.len() >= 8 {
                    break;
                }
                if is_admitted(u_tok) && !t2_adm.contains(&u_tok) {
                    t2_adm.push(u_tok);
                }
            }
        }
        let t2_same_cell_hit8 = t2_adm[..t2_adm.len().min(8)].contains(&row.target_token);

        // U_succ same-cell and U_train same-cell
        let mut u_succ_adm: Vec<u32> = corpus_tables
            .u_succ_ranked
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .collect();
        if u_succ_adm.len() < 8 {
            for &u_tok in &corpus_tables.u_train_ranked {
                if u_succ_adm.len() >= 8 {
                    break;
                }
                if is_admitted(u_tok) && !u_succ_adm.contains(&u_tok) {
                    u_succ_adm.push(u_tok);
                }
            }
        }
        let u_train_adm: Vec<u32> = corpus_tables
            .u_train_ranked
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .take(8)
            .collect();

        out.push(SameCellRowScore {
            t1_admitted_hit1,
            t1b_unconditional_in_admitted_hit1,
            t1b_nearest_admitted_occ_hit1,
            t2_same_cell_hit1: t2_top1_with_fallback == Some(row.target_token),
            t2_same_cell_strict_no_fallback_hit1: t2_strict_top1 == Some(row.target_token),
            t2_same_cell_hit8,
            u_succ_same_cell_hit1: u_succ_adm_top1 == Some(row.target_token),
            u_succ_same_cell_hit8: u_succ_adm[..u_succ_adm.len().min(8)]
                .contains(&row.target_token),
            u_train_same_cell_hit1: u_train_adm_top1 == Some(row.target_token),
            u_train_same_cell_hit8: u_train_adm.contains(&row.target_token),
        });
    }
    Ok(out)
}

fn summarize_same_cell_slice(rows: &[&ValidatedRow], scores: &[SameCellRowScore]) -> Value {
    let n = rows.len() as u64;
    let b3_rows: Vec<&ValidatedRow> = rows
        .iter()
        .copied()
        .filter(|r| r.eval_target_admitted)
        .collect();
    let b3_n = b3_rows.len() as u64;

    let count_all = |pred: fn(&SameCellRowScore) -> bool| -> u64 {
        rows.iter().filter(|r| pred(&scores[r.row_ordinal])).count() as u64
    };
    let count_b3 = |pred: fn(&SameCellRowScore) -> bool| -> u64 {
        b3_rows
            .iter()
            .filter(|r| pred(&scores[r.row_ordinal]))
            .count() as u64
    };

    json!({
        "panel_rows": n,
        "b3_target_admitted": b3_n,
        "on_all_slice_rows": {
            "t1_same_cell_hit1": rate_json(count_all(|s| s.t1_admitted_hit1), n),
            "t1b_same_cell_hit1": rate_json(count_all(|s| s.t1b_unconditional_in_admitted_hit1), n),
            "t1b_second_admitted_occurrence_hit1": rate_json(count_all(|s| s.t1b_nearest_admitted_occ_hit1), n),
            "t2_same_cell_hit1": rate_json(count_all(|s| s.t2_same_cell_hit1), n),
            "t2_same_cell_strict_no_fallback_hit1": rate_json(count_all(|s| s.t2_same_cell_strict_no_fallback_hit1), n),
            "t2_same_cell_hit8": rate_json(count_all(|s| s.t2_same_cell_hit8), n),
            "u_succ_same_cell_hit1": rate_json(count_all(|s| s.u_succ_same_cell_hit1), n),
            "u_succ_same_cell_hit8": rate_json(count_all(|s| s.u_succ_same_cell_hit8), n),
            "u_train_same_cell_hit1": rate_json(count_all(|s| s.u_train_same_cell_hit1), n),
            "u_train_same_cell_hit8": rate_json(count_all(|s| s.u_train_same_cell_hit8), n),
        },
        "on_b3_rows": {
            "t1_same_cell_hit1": rate_json(count_b3(|s| s.t1_admitted_hit1), b3_n),
            "t1b_same_cell_hit1": rate_json(count_b3(|s| s.t1b_unconditional_in_admitted_hit1), b3_n),
            "t1b_second_admitted_occurrence_hit1": rate_json(count_b3(|s| s.t1b_nearest_admitted_occ_hit1), b3_n),
            "t2_same_cell_hit1": rate_json(count_b3(|s| s.t2_same_cell_hit1), b3_n),
            "t2_same_cell_strict_no_fallback_hit1": rate_json(count_b3(|s| s.t2_same_cell_strict_no_fallback_hit1), b3_n),
            "t2_same_cell_hit8": rate_json(count_b3(|s| s.t2_same_cell_hit8), b3_n),
            "u_succ_same_cell_hit1": rate_json(count_b3(|s| s.u_succ_same_cell_hit1), b3_n),
            "u_succ_same_cell_hit8": rate_json(count_b3(|s| s.u_succ_same_cell_hit8), b3_n),
            "u_train_same_cell_hit1": rate_json(count_b3(|s| s.u_train_same_cell_hit1), b3_n),
            "u_train_same_cell_hit8": rate_json(count_b3(|s| s.u_train_same_cell_hit8), b3_n),
        }
    })
}

fn validate_unique_vocab_tokens(tokens: &[u32], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for (idx, &t) in tokens.iter().enumerate() {
        if t >= CANONICAL_VOCAB as u32 {
            return Err(format!(
                "{label}[{idx}]: token {t} >= vocab {CANONICAL_VOCAB}"
            ));
        }
        if !seen.insert(t) {
            return Err(format!("{label}: duplicate candidate token {t}"));
        }
    }
    Ok(())
}

pub fn verify_loaded_token_cells_against_arm(
    arm: &ValidatedArmInputs,
    loaded: &LoadedTokenCells,
    label: &str,
) -> Result<(), String> {
    if loaded.report_sha256.is_empty() {
        return Err(format!(
            "{label}: missing bound '{TOKEN_CELLS_RECEIPT_SCHEMA}' report receipt"
        ));
    }
    if loaded.driver_checkpoint_sha256 != arm.driver_checkpoint_sha256 {
        return Err(format!(
            "{label}: driver_checkpoint_sha256 ({}) != capture ({})",
            loaded.driver_checkpoint_sha256, arm.driver_checkpoint_sha256
        ));
    }
    if loaded.checkpoint_file_sha256 != arm.checkpoint_file_sha256 {
        return Err(format!(
            "{label}: checkpoint_file.sha256 ({}) != capture.checkpoint_file.sha256 ({})",
            loaded.checkpoint_file_sha256, arm.checkpoint_file_sha256
        ));
    }
    validate_sha256_str(
        &loaded.model_storage_sha256,
        "token_cells.model_storage_sha256",
    )?;
    validate_sha256_str(
        &loaded.model_snapshot_sha256,
        "token_cells.model_snapshot_sha256",
    )?;
    if loaded.model_storage_sha256 != loaded.model_snapshot_sha256 {
        return Err(format!(
            "{label}: model_storage_sha256 ({}) != model_snapshot_sha256 ({})",
            loaded.model_storage_sha256, loaded.model_snapshot_sha256
        ));
    }
    validate_sha256_str(
        &loaded.typed_model_snapshot_sha256,
        "token_cells.typed_model_snapshot_sha256",
    )?;
    if loaded.typed_model_snapshot_sha256 != arm.model_snapshot_sha256 {
        return Err(format!(
            "{label}: typed_model_snapshot_sha256 ({}) != eval.model_snapshot_sha256 ({})",
            loaded.typed_model_snapshot_sha256, arm.model_snapshot_sha256
        ));
    }
    if let Some(ref arm_storage_sha) = arm.model_storage_sha256 {
        if loaded.model_storage_sha256 != *arm_storage_sha {
            return Err(format!(
                "{label}: model_storage_sha256 ({}) != arm.model_storage_sha256 ({arm_storage_sha})",
                loaded.model_storage_sha256
            ));
        }
    }
    if loaded.config_sha256 != arm.config_sha256 {
        return Err(format!(
            "{label}: config_sha256 ({}) != capture ({})",
            loaded.config_sha256, arm.config_sha256
        ));
    }
    if loaded.config_file_sha256 != arm.config_file_sha256 {
        return Err(format!(
            "{label}: config_file.sha256 ({}) != capture.config_file.sha256 ({})",
            loaded.config_file_sha256, arm.config_file_sha256
        ));
    }
    if loaded.lineage_sha256 != arm.lineage_sha256 {
        return Err(format!(
            "{label}: lineage_sha256 ({}) != capture ({})",
            loaded.lineage_sha256, arm.lineage_sha256
        ));
    }
    if loaded.model_epoch != arm.model_epoch {
        return Err(format!(
            "{label}: model_epoch ({}) != capture ({})",
            loaded.model_epoch, arm.model_epoch
        ));
    }
    if loaded.completed_batches != arm.completed_batches {
        return Err(format!(
            "{label}: completed_batches ({}) != capture ({})",
            loaded.completed_batches, arm.completed_batches
        ));
    }
    let mb = &loaded.model_bindings;
    if mb.source_sha256 != arm.model_bindings.source_sha256
        || mb.kernel_sha256 != arm.model_bindings.kernel_sha256
        || mb.geometry_sha256 != arm.model_bindings.geometry_sha256
        || mb.spec != arm.model_bindings.spec
        || mb.read_spec != arm.model_bindings.read_spec
        || mb.objective_policy != arm.model_bindings.objective_policy
        || mb.read_initialization != arm.lineage.config.read_initialization
        || mb.seed != arm.model_bindings.seed
    {
        return Err(format!(
            "{label}: model_bindings disagrees with arm eval/lineage model_bindings"
        ));
    }
    Ok(())
}

pub fn score_same_cell_rows_from_table(
    arm: &ValidatedArmInputs,
    table: &CandidateCellTable,
    corpus_tables: &CorpusReferenceTables,
) -> Result<(Vec<SameCellRowScore>, usize), String> {
    if table.schema != CANDIDATE_CELL_TABLE_SCHEMA {
        return Err(format!(
            "candidate_cells: expected schema '{CANDIDATE_CELL_TABLE_SCHEMA}', got '{}'",
            table.schema
        ));
    }
    if table.driver_checkpoint_sha256 != arm.driver_checkpoint_sha256 {
        return Err(format!(
            "candidate_cells: driver_checkpoint_sha256 ({}) != capture ({})",
            table.driver_checkpoint_sha256, arm.driver_checkpoint_sha256
        ));
    }
    let snap_sha = table
        .model_snapshot_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing model_snapshot_sha256".to_string())?;
    if snap_sha != arm.model_snapshot_sha256 {
        return Err(format!(
            "candidate_cells: model_snapshot_sha256 ({snap_sha}) != eval ({})",
            arm.model_snapshot_sha256
        ));
    }
    if let (Some(tbl_storage), Some(arm_storage)) = (
        table.model_storage_sha256.as_deref(),
        arm.model_storage_sha256.as_deref(),
    ) {
        if tbl_storage != arm_storage {
            return Err(format!(
                "candidate_cells: model_storage_sha256 ({tbl_storage}) != arm ({arm_storage})"
            ));
        }
    }
    let bind_sha = table
        .ordered_row_binding_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing ordered_row_binding_sha256".to_string())?;
    if bind_sha != arm.ordered_row_binding_sha256
        || bind_sha != corpus_tables.ordered_row_binding_sha256
    {
        return Err(format!(
            "candidate_cells: ordered_row_binding_sha256 ({bind_sha}) != d34 ({})",
            arm.ordered_row_binding_sha256
        ));
    }
    verify_token_cells_against_gpu_capture(arm, &table.token_cells)?;

    let expected_tc_sha = table
        .token_cells_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing token_cells_sha256".to_string())?;
    let mut u16_bytes = Vec::with_capacity(CANONICAL_VOCAB * 2);
    for &c in &table.token_cells {
        u16_bytes.extend_from_slice(&(c as u16).to_le_bytes());
    }
    let actual_tc_sha = sha256_hex(&u16_bytes);
    if actual_tc_sha != expected_tc_sha {
        return Err(format!(
            "candidate_cells: token_cells_sha256 ({expected_tc_sha}) != recomputed u16LE sha256 ({actual_tc_sha})"
        ));
    }

    let occupied_cells = table
        .token_cells
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    if table.t2_ranked_candidates_by_query.len() != CANONICAL_VOCAB {
        return Err(format!(
            "candidate_cells: t2_ranked_candidates_by_query len {} != {CANONICAL_VOCAB}",
            table.t2_ranked_candidates_by_query.len()
        ));
    }
    for (q, q_cands) in table.t2_ranked_candidates_by_query.iter().enumerate() {
        validate_unique_vocab_tokens(
            q_cands,
            &format!("candidate_cells.t2_ranked_candidates_by_query[{q}]"),
        )?;
    }
    if table.t2_ranked_candidates_by_query != corpus_tables.t2_ranked_by_query {
        return Err(
            "candidate_cells: t2_ranked_candidates_by_query disagrees with corpus-recomputed T2 table"
                .to_string(),
        );
    }
    if table.u_succ_ranked_candidates.is_empty() {
        return Err("candidate_cells: empty u_succ_ranked_candidates".to_string());
    }
    validate_unique_vocab_tokens(
        &table.u_succ_ranked_candidates,
        "candidate_cells.u_succ_ranked_candidates",
    )?;
    if table.u_succ_ranked_candidates != corpus_tables.u_succ_ranked {
        return Err(
            "candidate_cells: u_succ_ranked_candidates disagrees with corpus-recomputed U_succ table"
                .to_string(),
        );
    }
    let u_train_ranked = table
        .u_train_ranked_candidates
        .as_ref()
        .ok_or_else(|| "candidate_cells: missing u_train_ranked_candidates".to_string())?;
    if u_train_ranked.is_empty() {
        return Err("candidate_cells: empty u_train_ranked_candidates".to_string());
    }
    validate_unique_vocab_tokens(u_train_ranked, "candidate_cells.u_train_ranked_candidates")?;
    if u_train_ranked != &table.u_succ_ranked_candidates
        || u_train_ranked != &corpus_tables.u_train_ranked
    {
        return Err(
            "candidate_cells: u_train_ranked_candidates disagrees with corpus-recomputed U_train table (or u_succ_ranked_candidates)"
                .to_string(),
        );
    }

    let occs = table
        .dev_window_query_successors_most_recent_first
        .as_ref()
        .ok_or_else(|| {
            "candidate_cells: missing dev_window_query_successors_most_recent_first".to_string()
        })?;
    if occs.len() != arm.rows.len() || occs.len() != corpus_tables.dev_rows.len() {
        return Err(format!(
            "candidate_cells: dev_window_query_successors_most_recent_first len {} != {}",
            occs.len(),
            arm.rows.len()
        ));
    }
    for (r_idx, (r_occs, c_row)) in occs.iter().zip(corpus_tables.dev_rows.iter()).enumerate() {
        for (c_idx, &tok) in r_occs.iter().enumerate() {
            if tok >= CANONICAL_VOCAB as u32 {
                return Err(format!(
                    "candidate_cells.dev_window_query_successors_most_recent_first[{r_idx}][{c_idx}]: token {tok} >= vocab {CANONICAL_VOCAB}"
                ));
            }
        }
        if r_occs != &c_row.window_query_successors_most_recent_first {
            return Err(format!(
                "candidate_cells.dev_window_query_successors_most_recent_first[{r_idx}] disagrees with corpus-recomputed window successors"
            ));
        }
    }

    let expected_t2_sha = table
        .t2_ranked_table_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing t2_ranked_table_sha256".to_string())?;
    let actual_t2_sha = sha256_hex(
        &serde_json::to_vec(&table.t2_ranked_candidates_by_query)
            .map_err(|e| format!("t2 table json sha: {e}"))?,
    );
    if actual_t2_sha != expected_t2_sha || actual_t2_sha != corpus_tables.t2_ranked_table_sha256 {
        return Err(format!(
            "candidate_cells: t2_ranked_table_sha256 ({expected_t2_sha}) != recomputed ({actual_t2_sha})"
        ));
    }

    let expected_usucc_sha = table
        .u_succ_ranked_table_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing u_succ_ranked_table_sha256".to_string())?;
    let actual_usucc_sha = sha256_hex(
        &serde_json::to_vec(&table.u_succ_ranked_candidates)
            .map_err(|e| format!("u_succ table json sha: {e}"))?,
    );
    if actual_usucc_sha != expected_usucc_sha
        || actual_usucc_sha != corpus_tables.u_succ_ranked_table_sha256
    {
        return Err(format!(
            "candidate_cells: u_succ_ranked_table_sha256 ({expected_usucc_sha}) != recomputed ({actual_usucc_sha})"
        ));
    }

    let expected_utrain_sha = table
        .u_train_ranked_table_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing u_train_ranked_table_sha256".to_string())?;
    let actual_utrain_sha = sha256_hex(
        &serde_json::to_vec(u_train_ranked).map_err(|e| format!("u_train table json sha: {e}"))?,
    );
    if actual_utrain_sha != expected_utrain_sha
        || expected_usucc_sha != expected_utrain_sha
        || actual_utrain_sha != corpus_tables.u_train_ranked_table_sha256
    {
        return Err(format!(
            "candidate_cells: u_train_ranked_table_sha256 ({expected_utrain_sha}) mismatch against recomputed ({actual_utrain_sha}) or u_succ ({expected_usucc_sha})"
        ));
    }

    let expected_win_sha = table
        .dev_window_successors_sha256
        .as_deref()
        .ok_or_else(|| "candidate_cells: missing dev_window_successors_sha256".to_string())?;
    let actual_win_sha = sha256_hex(
        &serde_json::to_vec(occs).map_err(|e| format!("dev_window_successors json sha: {e}"))?,
    );
    if actual_win_sha != expected_win_sha
        || actual_win_sha != corpus_tables.dev_window_successors_sha256
    {
        return Err(format!(
            "candidate_cells: dev_window_successors_sha256 ({expected_win_sha}) != recomputed ({actual_win_sha})"
        ));
    }

    let u_train_top1 = u_train_ranked[0];

    let mut out = Vec::with_capacity(arm.rows.len());
    for (idx, row) in arm.rows.iter().enumerate() {
        let row_occs = &occs[idx];
        let t2_q = &table.t2_ranked_candidates_by_query[row.query_token as usize];

        // Cross-check unconditional predictions/hits against sealed d34 per-row record.
        if row.is_psucc && row_occs.first().copied() != Some(row.target_token) {
            return Err(format!(
                "candidate_cells row {idx}: P_succ row most-recent successor {:?} != target_token {}",
                row_occs.first(),
                row.target_token
            ));
        }
        let expected_t1b_pred = row_occs.get(1).copied().unwrap_or(u_train_top1);
        let expected_t2_pred = t2_q.first().copied().unwrap_or(u_train_top1);
        let expected_t2_top8: &[u32] = if t2_q.is_empty() {
            &u_train_ranked[..u_train_ranked.len().min(8)]
        } else {
            &t2_q[..t2_q.len().min(8)]
        };
        if row.d34_t1b_prediction != expected_t1b_pred
            || row.d34_t1b_hit1 != (expected_t1b_pred == row.target_token)
            || row.d34_t2_prediction != expected_t2_pred
            || row.d34_t2_hit1 != (expected_t2_pred == row.target_token)
            || row.d34_t2_hit8 != expected_t2_top8.contains(&row.target_token)
            || row.d34_u_hit1 != (u_train_top1 == row.target_token)
            || row.d34_u_hit8
                != u_train_ranked[..u_train_ranked.len().min(8)].contains(&row.target_token)
        {
            return Err(format!(
                "candidate_cells row {idx}: unconditional T1b/T2/U prediction or hit disagrees with sealed D34 record"
            ));
        }

        let is_admitted =
            |tok: u32| -> bool { row.active_cells.contains(&table.token_cells[tok as usize]) };

        let u_train_adm_top1 = u_train_ranked.iter().copied().find(|&y| is_admitted(y));
        let u_succ_adm_top1 = table
            .u_succ_ranked_candidates
            .iter()
            .copied()
            .find(|&y| is_admitted(y))
            .or(u_train_adm_top1);

        let t1_admitted_hit1 = row_occs
            .first()
            .copied()
            .is_some_and(|y1| is_admitted(y1) && y1 == row.target_token);
        let t1b_unconditional_in_admitted_hit1 = match row_occs.get(1).copied() {
            Some(y1b) if is_admitted(y1b) => y1b == row.target_token,
            _ => u_train_adm_top1 == Some(row.target_token),
        };
        let t1b_nearest_admitted_occ_hit1 = row_occs
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .nth(1)
            .or(u_train_adm_top1)
            == Some(row.target_token);

        let mut t2_adm: Vec<u32> = t2_q.iter().copied().filter(|&y| is_admitted(y)).collect();
        let t2_strict_top1 = t2_adm.first().copied();
        let t2_top1_with_fallback = t2_strict_top1.or(u_train_adm_top1);
        if t2_adm.len() < 8 {
            for &u_tok in u_train_ranked {
                if t2_adm.len() >= 8 {
                    break;
                }
                if is_admitted(u_tok) && !t2_adm.contains(&u_tok) {
                    t2_adm.push(u_tok);
                }
            }
        }
        let t2_same_cell_hit8 = t2_adm[..t2_adm.len().min(8)].contains(&row.target_token);

        let mut u_succ_adm: Vec<u32> = table
            .u_succ_ranked_candidates
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .collect();
        if u_succ_adm.len() < 8 {
            for &u_tok in u_train_ranked {
                if u_succ_adm.len() >= 8 {
                    break;
                }
                if is_admitted(u_tok) && !u_succ_adm.contains(&u_tok) {
                    u_succ_adm.push(u_tok);
                }
            }
        }
        let u_train_adm: Vec<u32> = u_train_ranked
            .iter()
            .copied()
            .filter(|&y| is_admitted(y))
            .take(8)
            .collect();

        out.push(SameCellRowScore {
            t1_admitted_hit1,
            t1b_unconditional_in_admitted_hit1,
            t1b_nearest_admitted_occ_hit1,
            t2_same_cell_hit1: t2_top1_with_fallback == Some(row.target_token),
            t2_same_cell_strict_no_fallback_hit1: t2_strict_top1 == Some(row.target_token),
            t2_same_cell_hit8,
            u_succ_same_cell_hit1: u_succ_adm_top1 == Some(row.target_token),
            u_succ_same_cell_hit8: u_succ_adm[..u_succ_adm.len().min(8)]
                .contains(&row.target_token),
            u_train_same_cell_hit1: u_train_adm_top1 == Some(row.target_token),
            u_train_same_cell_hit8: u_train_adm.contains(&row.target_token),
        });
    }
    Ok((out, occupied_cells))
}

/// Bundle a GPU-exported `LoadedTokenCells` (`token_cells.u16` + sibling `report.json` or JSON receipt)
/// with the non-model `T1/T1b/T2/U_succ/U_train` count tables rebuilt from the pinned corpus files
/// and cross-checked against `d34.json`.
pub fn build_candidate_cell_table_from_loaded(
    loaded: &LoadedTokenCells,
    explicit_driver_checkpoint_sha256: Option<&str>,
    d34_value: &Value,
    d34_sha256: &str,
    corpus_paths: &CorpusPaths<'_>,
    enforce_canonical_pins: bool,
) -> Result<CandidateCellTable, String> {
    if loaded.report_sha256.is_empty() {
        return Err(format!(
            "token_cells: missing bound '{TOKEN_CELLS_RECEIPT_SCHEMA}' report receipt"
        ));
    }
    let receipt_ckpt_sha = loaded.driver_checkpoint_sha256.as_str();
    if let Some(exp) = explicit_driver_checkpoint_sha256 {
        if !exp.is_empty() && exp != receipt_ckpt_sha {
            return Err(format!(
                "token_cells: explicit driver_checkpoint_sha256 ({exp}) != receipt ({receipt_ckpt_sha})"
            ));
        }
    }
    let ckpt_sha = receipt_ckpt_sha.to_string();
    validate_sha256_str(&ckpt_sha, "token_cells.driver_checkpoint_sha256")?;
    validate_sha256_str(
        &loaded.model_snapshot_sha256,
        "token_cells.model_snapshot_sha256",
    )?;
    validate_sha256_str(
        &loaded.typed_model_snapshot_sha256,
        "token_cells.typed_model_snapshot_sha256",
    )?;

    let expected_rows = if enforce_canonical_pins {
        CANONICAL_DEV_ELIGIBLE_ROWS
    } else {
        d34_value
            .get("per_row")
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    };
    let (ordered_row_binding_sha256, d34_rows) =
        parse_d34_record(d34_value, d34_sha256, expected_rows, enforce_canonical_pins)?;
    let corpus_tables =
        build_corpus_reference_tables(corpus_paths, &d34_rows, enforce_canonical_pins)?;

    let dev_window_query_successors_most_recent_first = corpus_tables
        .dev_rows
        .into_iter()
        .map(|r| r.window_query_successors_most_recent_first)
        .collect();

    Ok(CandidateCellTable {
        schema: CANDIDATE_CELL_TABLE_SCHEMA.to_string(),
        driver_checkpoint_sha256: ckpt_sha,
        model_snapshot_sha256: Some(loaded.typed_model_snapshot_sha256.clone()),
        model_storage_sha256: Some(loaded.model_storage_sha256.clone()),
        model_value_sha256: Some(loaded.model_snapshot_sha256.clone()),
        token_cells_sha256: Some(loaded.token_cells_sha256.clone()),
        ordered_row_binding_sha256: Some(ordered_row_binding_sha256),
        t2_ranked_table_sha256: Some(corpus_tables.t2_ranked_table_sha256),
        u_succ_ranked_table_sha256: Some(corpus_tables.u_succ_ranked_table_sha256),
        u_train_ranked_table_sha256: Some(corpus_tables.u_train_ranked_table_sha256),
        dev_window_successors_sha256: Some(corpus_tables.dev_window_successors_sha256),
        token_cells: loaded.token_cells.clone(),
        t2_ranked_candidates_by_query: corpus_tables.t2_ranked_by_query,
        u_succ_ranked_candidates: corpus_tables.u_succ_ranked,
        u_train_ranked_candidates: Some(corpus_tables.u_train_ranked),
        dev_window_query_successors_most_recent_first: Some(
            dev_window_query_successors_most_recent_first,
        ),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn score_preflight_dev_values(
    panel: &Value,
    panel_sha: String,
    capture: &Value,
    capture_sha: String,
    eval: &Value,
    eval_sha: String,
    d34: &Value,
    d34_sha: String,
    candidate_cells: Option<(&CandidateCellTable, String)>,
    token_cells_and_corpus: Option<(&LoadedTokenCells, &CorpusPaths<'_>)>,
    enforce_canonical_pins: bool,
) -> Result<Value, String> {
    let arm = validate_arm_inputs(
        panel,
        panel_sha,
        capture,
        capture_sha,
        eval,
        eval_sha,
        d34,
        d34_sha,
        enforce_canonical_pins,
    )?;

    let all_refs: Vec<&ValidatedRow> = arm.rows.iter().collect();
    let psucc_refs: Vec<&ValidatedRow> = arm.rows.iter().filter(|r| r.is_psucc).collect();
    let comp_refs: Vec<&ValidatedRow> = arm.rows.iter().filter(|r| !r.is_psucc).collect();
    let conflict_refs: Vec<&ValidatedRow> = arm.rows.iter().filter(|r| r.conflict).collect();
    let b0_refs: Vec<&ValidatedRow> = arm
        .rows
        .iter()
        .filter(|r| r.distance_bucket == Some(0))
        .collect();
    let b1_refs: Vec<&ValidatedRow> = arm
        .rows
        .iter()
        .filter(|r| r.distance_bucket == Some(1))
        .collect();
    let b2_refs: Vec<&ValidatedRow> = arm
        .rows
        .iter()
        .filter(|r| r.distance_bucket == Some(2))
        .collect();
    let b3_refs: Vec<&ValidatedRow> = arm
        .rows
        .iter()
        .filter(|r| r.distance_bucket == Some(3))
        .collect();

    let (_, d34_rows) =
        parse_d34_record(d34, &arm.d34_sha256, arm.rows.len(), enforce_canonical_pins)?;

    let same_cell_block = if let Some((loaded_tc, corpus_paths)) = token_cells_and_corpus {
        verify_loaded_token_cells_against_arm(&arm, loaded_tc, "token_cells")?;
        let corpus_tables =
            build_corpus_reference_tables(corpus_paths, &d34_rows, enforce_canonical_pins)?;
        let row_scores =
            score_same_cell_rows_from_corpus(&arm, &loaded_tc.token_cells, &corpus_tables)?;
        let psucc_b3_slice = summarize_same_cell_slice(&psucc_refs, &row_scores);
        json!({
            "status": "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
            "token_cells_sha256": loaded_tc.token_cells_sha256,
            "token_cells_report_sha256": loaded_tc.report_sha256,
            "model_storage_sha256": loaded_tc.model_storage_sha256,
            "export_model_value_sha256": loaded_tc.model_snapshot_sha256,
            "eval_typed_model_snapshot_sha256": loaded_tc.typed_model_snapshot_sha256,
            "model_snapshot_sha256": arm.model_snapshot_sha256,
            "occupied_writer_cells": loaded_tc.occupied_cells,
            "target_cell_cross_check_rows": arm.rows.len(),
            "d34_unconditional_cross_check_rows": arm.rows.len(),
            "psucc_b3_rows": psucc_b3_slice["b3_target_admitted"],
            "t2_same_cell_top1_on_psucc_b3": psucc_b3_slice["on_b3_rows"]["t2_same_cell_hit1"],
            "t2_same_cell_top8_on_psucc_b3": psucc_b3_slice["on_b3_rows"]["t2_same_cell_hit8"],
            "u_succ_same_cell_top1_on_psucc_b3": psucc_b3_slice["on_b3_rows"]["u_succ_same_cell_hit1"],
            "u_succ_same_cell_top8_on_psucc_b3": psucc_b3_slice["on_b3_rows"]["u_succ_same_cell_hit8"],
            "corpus_pins": {
                "train_tokens_sha256": corpus_tables.train_tokens_sha256,
                "train_mask_sha256": corpus_tables.train_mask_sha256,
                "dev_tokens_sha256": corpus_tables.dev_tokens_sha256,
                "dev_mask_sha256": corpus_tables.dev_mask_sha256,
                "ordered_row_binding_sha256": corpus_tables.ordered_row_binding_sha256,
                "train_matched_targets": corpus_tables.train_matched_targets,
                "train_bigram_total_pairs": corpus_tables.train_bigram_total_pairs,
                "u_train_total_counts": corpus_tables.u_train_total_counts,
                "dev_psucc_rows": corpus_tables.dev_psucc_rows,
                "u_succ_total_counts": corpus_tables.u_succ_total_counts,
                "conservation_ok": corpus_tables.conservation_ok,
                "t2_ranked_table_sha256": corpus_tables.t2_ranked_table_sha256,
                "u_train_ranked_table_sha256": corpus_tables.u_train_ranked_table_sha256,
                "u_succ_ranked_table_sha256": corpus_tables.u_succ_ranked_table_sha256,
                "dev_window_successors_sha256": corpus_tables.dev_window_successors_sha256,
            },
            "slices": {
                "full_panel": summarize_same_cell_slice(&all_refs, &row_scores),
                "copy_successor_subpanel": psucc_b3_slice,
                "complement_subpanel": summarize_same_cell_slice(&comp_refs, &row_scores),
                "conflict_rows": summarize_same_cell_slice(&conflict_refs, &row_scores),
                "d_1_8": summarize_same_cell_slice(&b0_refs, &row_scores),
                "d_9_32": summarize_same_cell_slice(&b1_refs, &row_scores),
                "d_33_96": summarize_same_cell_slice(&b2_refs, &row_scores),
                "d_97_192": summarize_same_cell_slice(&b3_refs, &row_scores),
            }
        })
    } else if let Some((_table, _table_sha)) = candidate_cells {
        return Err(
            "preflight-dev: external --candidate-cells without --token-cells and pinned corpus files (--train-tokens/--train-mask/--dev-tokens/--dev-mask) is refused; provide --token-cells alongside the 4 corpus files so T2/U_succ/U_train/window tables are authenticated against the corpus and token_cells report receipt"
                .to_string(),
        );
    } else {
        json!({
            "status": "UNAVAILABLE_MISSING_GPU_TOKEN_CELL_TABLE_AND_RANKED_CANDIDATE_RECEIPT",
            "missing_inputs": [
                "gpu_token_cells_4096_and_registered_corpus_files_or_candidate_cell_table"
            ],
            "explanation": "attention-capture-dev exports per-row admitted_cells[16] and eval_target_admitted. Provide --candidate-cells (or --token-cells with --train-tokens/--train-mask/--dev-tokens/--dev-mask) to score full same-cell candidate re-ranking."
        })
    };

    Ok(json!({
        "schema": PREFLIGHT_SCORE_SCHEMA,
        "status": "VALIDATED_GPU_CELL_CAPTURE_AND_SEALED_D34_SCORE",
        "pins": {
            "panel_sha256": arm.panel_sha256,
            "capture_sha256": arm.capture_sha256,
            "eval_sha256": arm.eval_sha256,
            "d34_sha256": arm.d34_sha256,
            "config_sha256": arm.config_sha256,
            "config_file_sha256": arm.config_file_sha256,
            "lineage_sha256": arm.lineage_sha256,
            "driver_checkpoint_sha256": arm.driver_checkpoint_sha256,
            "checkpoint_file_sha256": arm.checkpoint_file_sha256,
            "model_storage_sha256": arm.model_storage_sha256,
            "model_snapshot_sha256": arm.model_snapshot_sha256,
            "ordered_row_binding_sha256": arm.ordered_row_binding_sha256,
            "model_epoch": arm.model_epoch,
            "completed_batches": arm.completed_batches,
        },
        "slot_inventory_verification": {
            "row_count": arm.rows.len(),
            "physical_words_per_row": 16,
            "max_active_cells_per_row": 8,
            "unused_slot_marker": UNUSED_CELL_SLOT,
            "all_rows_match_eval_admitted_cells_count": true,
            "all_rows_match_eval_target_admitted_bit": true,
        },
        "b3_and_sealed_d34_slices": {
            "full_panel": score_slice(&all_refs),
            "copy_successor_subpanel": score_slice(&psucc_refs),
            "complement_subpanel": score_slice(&comp_refs),
            "conflict_rows": score_slice(&conflict_refs),
            "d_1_8": score_slice(&b0_refs),
            "d_9_32": score_slice(&b1_refs),
            "d_33_96": score_slice(&b2_refs),
            "d_97_192": score_slice(&b3_refs),
        },
        "same_cell_candidate_reranking": same_cell_block,
        "model_compute_executed_by_scorer": false,
        "updates_applied": 0,
        "training": "HELD",
        "m2": "HELD",
        "stage_b": "HELD",
        "q4": "NOT_EXECUTABLE"
    }))
}

/// Exact integer ceiling of `max(mde_floor, 2 * sqrt(b + c))`:
/// returns the smallest integer `k >= mde_floor` such that `k * k >= 4 * (b + c)`.
pub fn paired_integer_floor(mde_floor: u64, discordant_sum: u64) -> u64 {
    let target_sq = 4u128 * u128::from(discordant_sum);
    let mut k = mde_floor;
    while u128::from(k) * u128::from(k) < target_sq {
        k += 1;
    }
    k
}

fn compute_paired_2x2(
    a_hits: &[bool],
    b_hits: &[bool],
    mde_floor: u64,
    label: &str,
) -> Result<Value, String> {
    if a_hits.len() != b_hits.len() {
        return Err(format!("{label}: paired length mismatch"));
    }
    let mut n11 = 0u64;
    let mut n10 = 0u64;
    let mut n01 = 0u64;
    let mut n00 = 0u64;
    for (&a, &b) in a_hits.iter().zip(b_hits.iter()) {
        match (a, b) {
            (true, true) => n11 += 1,
            (true, false) => n10 += 1,
            (false, true) => n01 += 1,
            (false, false) => n00 += 1,
        }
    }
    let n = a_hits.len() as u64;
    let a_total = n11 + n10;
    let b_total = n11 + n01;
    let diff = (a_total as i64) - (b_total as i64);
    let disc = n10 + n01;
    let req_floor = paired_integer_floor(mde_floor, disc);
    Ok(json!({
        "n": n,
        "a_hits": a_total,
        "b_hits": b_total,
        "n11_both_correct": n11,
        "b_a_only": n10,
        "c_b_only": n01,
        "n00_neither": n00,
        "discordant_b_plus_c": disc,
        "difference_a_minus_b": diff,
        "mde_floor": mde_floor,
        "required_integer_bar_max_mde_2sqrt_b_plus_c": req_floor,
        "meets_or_exceeds_bar": diff >= (req_floor as i64),
    }))
}

#[allow(clippy::too_many_arguments)]
pub fn score_paired_dev_values(
    panel: &Value,
    panel_sha: String,
    rel_capture: &Value,
    rel_capture_sha: String,
    rel_eval: &Value,
    rel_eval_sha: String,
    ctrl_capture: &Value,
    ctrl_capture_sha: String,
    ctrl_eval: &Value,
    ctrl_eval_sha: String,
    ablation_eval: Option<(&Value, String)>,
    d34: &Value,
    d34_sha: String,
    candidate_tables: Option<(&CandidateCellTable, String, &CandidateCellTable, String)>,
    token_cells_and_corpus: Option<(&LoadedTokenCells, &LoadedTokenCells, &CorpusPaths<'_>)>,
    enforce_canonical_pins: bool,
) -> Result<Value, String> {
    // Validate each arm against its own config_sha256 / lineage_sha256 while requiring
    // both arms to share the exact panel dev_binding and 14,287 DEV rows.
    let rel = validate_arm_inputs_with_mode(
        panel,
        panel_sha.clone(),
        rel_capture,
        rel_capture_sha,
        rel_eval,
        rel_eval_sha,
        d34,
        d34_sha.clone(),
        enforce_canonical_pins,
        false,
    )?;
    let ctrl = validate_arm_inputs_with_mode(
        panel,
        panel_sha,
        ctrl_capture,
        ctrl_capture_sha,
        ctrl_eval,
        ctrl_eval_sha,
        d34,
        d34_sha,
        enforce_canonical_pins,
        false,
    )?;

    // The provided panel must bind to either the Relation or Control arm's config/lineage.
    let panel_matches_rel = rel.panel_config_sha256 == rel.config_sha256
        && rel.panel_lineage_sha256 == rel.lineage_sha256;
    let panel_matches_ctrl = ctrl.panel_config_sha256 == ctrl.config_sha256
        && ctrl.panel_lineage_sha256 == ctrl.lineage_sha256;
    if !panel_matches_rel && !panel_matches_ctrl {
        return Err(format!(
            "paired-dev: panel (config_sha256={}, lineage_sha256={}) matches neither relation ({}, {}) nor control ({}, {})",
            rel.panel_config_sha256,
            rel.panel_lineage_sha256,
            rel.config_sha256,
            rel.lineage_sha256,
            ctrl.config_sha256,
            ctrl.lineage_sha256
        ));
    }
    if panel_matches_rel && rel.panel_config_file_sha256 != rel.config_file_sha256 {
        return Err(format!(
            "paired-dev: panel.config_file.sha256 ({}) != relation capture.config_file.sha256 ({})",
            rel.panel_config_file_sha256, rel.config_file_sha256
        ));
    }
    if panel_matches_ctrl && ctrl.panel_config_file_sha256 != ctrl.config_file_sha256 {
        return Err(format!(
            "paired-dev: panel.config_file.sha256 ({}) != control capture.config_file.sha256 ({})",
            ctrl.panel_config_file_sha256, ctrl.config_file_sha256
        ));
    }

    // Treatment (LearnV1) and Control (FrozenZeroV1) have distinct configs and lineages,
    // and must be identical in every DriverConfig/DriverLineage field EXCEPT relation_learning_policy.
    if rel.config_sha256 == ctrl.config_sha256
        || rel.lineage_sha256 == ctrl.lineage_sha256
        || rel.config_file_sha256 == ctrl.config_file_sha256
    {
        return Err(format!(
            "paired-dev: relation and control must have distinct config_sha256, config_file.sha256, and lineage_sha256 (got rel=({}, {}), ctrl=({}, {}))",
            rel.config_sha256, rel.lineage_sha256, ctrl.config_sha256, ctrl.lineage_sha256
        ));
    }
    let expected_read_spec = ReadSpecRecord::Relation { kept: 8, sweeps: 1 };
    if rel.lineage.config.read_spec != expected_read_spec {
        return Err(format!(
            "paired-dev: relation arm read_spec must be Relation {{ kept: 8, sweeps: 1 }}, got {:?}",
            rel.lineage.config.read_spec
        ));
    }
    if rel.lineage.config.relation_learning_policy != RelationLearningPolicyRecord::LearnV1 {
        return Err(format!(
            "paired-dev: relation arm relation_learning_policy must be LearnV1, got {:?}",
            rel.lineage.config.relation_learning_policy
        ));
    }
    if ctrl.lineage.config.relation_learning_policy != RelationLearningPolicyRecord::FrozenZeroV1 {
        return Err(format!(
            "paired-dev: control arm relation_learning_policy must be FrozenZeroV1, got {:?}",
            ctrl.lineage.config.relation_learning_policy
        ));
    }
    let mut expected_ctrl_config = rel.lineage.config.clone();
    expected_ctrl_config.relation_learning_policy = RelationLearningPolicyRecord::FrozenZeroV1;
    if ctrl.lineage.config != expected_ctrl_config {
        return Err(
            "paired-dev: control arm DriverConfig differs from relation arm in fields other than relation_learning_policy (LearnV1 vs FrozenZeroV1)"
                .to_string(),
        );
    }
    let mut expected_ctrl_lineage = rel.lineage.clone();
    expected_ctrl_lineage.config = expected_ctrl_config;
    if ctrl.lineage != expected_ctrl_lineage {
        return Err(
            "paired-dev: control arm DriverLineage differs from relation arm in corpus_binding or spec"
                .to_string(),
        );
    }
    if ctrl.model_bindings != rel.model_bindings {
        return Err(
            "paired-dev: control arm model_bindings differs from relation arm model_bindings"
                .to_string(),
        );
    }

    if rel.completed_batches != ctrl.completed_batches {
        return Err(format!(
            "paired-dev: relation completed_batches ({}) != control completed_batches ({})",
            rel.completed_batches, ctrl.completed_batches
        ));
    }
    if rel.rows.len() != ctrl.rows.len() {
        return Err("paired-dev: row count mismatch between relation and control".to_string());
    }

    let abl_parsed = if let Some((abl_val, abl_sha)) = ablation_eval {
        let parsed = parse_eval_rows(abl_val, &abl_sha, rel.rows.len(), enforce_canonical_pins)?;
        if parsed.lineage_sha256 != rel.lineage_sha256 || parsed.lineage != rel.lineage {
            return Err(format!(
                "ablation_eval: evaluation_lineage / lineage_sha256 ({}) != relation ({})",
                parsed.lineage_sha256, rel.lineage_sha256
            ));
        }
        if parsed.model_epoch != rel.model_epoch {
            return Err(format!(
                "ablation_eval: model_epoch ({}) != relation ({})",
                parsed.model_epoch, rel.model_epoch
            ));
        }
        if parsed.model_bindings != rel.model_bindings {
            return Err(
                "ablation_eval: model_bindings disagrees with relation arm model_bindings"
                    .to_string(),
            );
        }
        for (idx, (a_r, r_r)) in parsed.rows.iter().zip(rel.rows.iter()).enumerate() {
            if a_r.target_position != r_r.target_position || a_r.target_token != r_r.target_token {
                return Err(format!(
                    "ablation_eval row {idx}: target identity mismatch against relation"
                ));
            }
        }
        Some((parsed, abl_sha))
    } else {
        None
    };

    let mut b3_r = 0u64;
    let mut b3_c = 0u64;
    let mut b3_i = 0u64;
    let mut adm_r_only = 0u64;
    let mut adm_c_only = 0u64;
    let mut adm_neither = 0u64;

    let mut rel_b3_i_top1 = Vec::new();
    let mut ctrl_b3_i_top1 = Vec::new();
    let mut t2_uncond_b3_i_top1 = Vec::new();
    let mut u_uncond_b3_i_top1 = Vec::new();
    let mut abl_b3_i_top1 = Vec::new();

    let mut rel_b3_r_top1 = Vec::new();
    let mut t2_uncond_b3_r_top1 = Vec::new();
    let mut u_uncond_b3_r_top1 = Vec::new();
    let mut abl_b3_r_top1 = Vec::new();

    let mut rel_itt_top1 = Vec::new();
    let mut ctrl_itt_top1 = Vec::new();
    let mut rel_itt_top8 = Vec::new();
    let mut ctrl_itt_top8 = Vec::new();

    let mut rel_comp_top1 = Vec::new();
    let mut ctrl_comp_top1 = Vec::new();
    let mut rel_comp_top8 = Vec::new();
    let mut ctrl_comp_top8 = Vec::new();

    for (idx, (r_row, c_row)) in rel.rows.iter().zip(ctrl.rows.iter()).enumerate() {
        let abl_hit = abl_parsed
            .as_ref()
            .map(|(p, _)| p.rows[idx].top1_correct)
            .unwrap_or(false);
        if r_row.is_psucc {
            rel_itt_top1.push(r_row.model_top1);
            ctrl_itt_top1.push(c_row.model_top1);
            rel_itt_top8.push(r_row.model_top8);
            ctrl_itt_top8.push(c_row.model_top8);

            if r_row.eval_target_admitted {
                b3_r += 1;
                rel_b3_r_top1.push(r_row.model_top1);
                t2_uncond_b3_r_top1.push(r_row.d34_t2_hit1);
                u_uncond_b3_r_top1.push(r_row.d34_u_hit1);
                abl_b3_r_top1.push(abl_hit);
            }
            if c_row.eval_target_admitted {
                b3_c += 1;
            }
            match (r_row.eval_target_admitted, c_row.eval_target_admitted) {
                (true, true) => {
                    b3_i += 1;
                    rel_b3_i_top1.push(r_row.model_top1);
                    ctrl_b3_i_top1.push(c_row.model_top1);
                    t2_uncond_b3_i_top1.push(r_row.d34_t2_hit1);
                    u_uncond_b3_i_top1.push(r_row.d34_u_hit1);
                    abl_b3_i_top1.push(abl_hit);
                }
                (true, false) => adm_r_only += 1,
                (false, true) => adm_c_only += 1,
                (false, false) => adm_neither += 1,
            }
        } else {
            rel_comp_top1.push(r_row.model_top1);
            ctrl_comp_top1.push(c_row.model_top1);
            rel_comp_top8.push(r_row.model_top8);
            ctrl_comp_top8.push(c_row.model_top8);
        }
    }

    let psucc_total = rel_itt_top1.len() as u64;
    if b3_i + adm_r_only + adm_c_only + adm_neither != psucc_total {
        return Err("admission 2x2 conservation failure on P_succ".to_string());
    }

    let ablation_block = if let Some((ref parsed_abl, ref abl_sha)) = abl_parsed {
        json!({
            "status": "SCORED",
            "ablation_eval_sha256": abl_sha,
            "ablation_model_snapshot_sha256": parsed_abl.model_snapshot_sha256,
            "relation_vs_ablation_on_b3_r_top1": compute_paired_2x2(&rel_b3_r_top1, &abl_b3_r_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_abl_b3_r")?,
            "relation_vs_ablation_on_b3_i_top1": compute_paired_2x2(&rel_b3_i_top1, &abl_b3_i_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_abl_b3_i")?,
        })
    } else {
        json!({
            "status": "NOT_PROVIDED"
        })
    };

    let build_paired_same_cell_json = |status: &str,
                                       rel_pin_key: &str,
                                       rel_pin_sha: String,
                                       ctrl_pin_key: &str,
                                       ctrl_pin_sha: String,
                                       rel_occupied: usize,
                                       ctrl_occupied: usize,
                                       rel_sc: &[SameCellRowScore],
                                       ctrl_sc: &[SameCellRowScore]|
     -> Result<Value, String> {
        let mut t2_r_on_b3_r = Vec::new();
        let mut u_succ_r_on_b3_r = Vec::new();
        let mut t2_r_on_b3_i = Vec::new();
        let mut t2_c_on_b3_i = Vec::new();
        let mut u_succ_r_on_b3_i = Vec::new();
        let mut ctrl_b3_c_top1 = Vec::new();
        let mut t2_c_on_b3_c = Vec::new();

        for (idx, (r_row, c_row)) in rel.rows.iter().zip(ctrl.rows.iter()).enumerate() {
            if !r_row.is_psucc {
                continue;
            }
            if r_row.eval_target_admitted {
                t2_r_on_b3_r.push(rel_sc[idx].t2_same_cell_hit1);
                u_succ_r_on_b3_r.push(rel_sc[idx].u_succ_same_cell_hit1);
            }
            if c_row.eval_target_admitted {
                ctrl_b3_c_top1.push(c_row.model_top1);
                t2_c_on_b3_c.push(ctrl_sc[idx].t2_same_cell_hit1);
            }
            if r_row.eval_target_admitted && c_row.eval_target_admitted {
                t2_r_on_b3_i.push(rel_sc[idx].t2_same_cell_hit1);
                t2_c_on_b3_i.push(ctrl_sc[idx].t2_same_cell_hit1);
                u_succ_r_on_b3_i.push(rel_sc[idx].u_succ_same_cell_hit1);
            }
        }

        let mut out = json!({
            "status": status,
            "relation_occupied_writer_cells": rel_occupied,
            "control_occupied_writer_cells": ctrl_occupied,
            "relation_vs_t2_r_same_cell_on_b3_r_top1": compute_paired_2x2(&rel_b3_r_top1, &t2_r_on_b3_r, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_t2_r_b3_r")?,
            "relation_vs_u_succ_r_same_cell_on_b3_r_top1": compute_paired_2x2(&rel_b3_r_top1, &u_succ_r_on_b3_r, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_u_succ_r_b3_r")?,
            "relation_vs_t2_r_same_cell_on_b3_i_top1": compute_paired_2x2(&rel_b3_i_top1, &t2_r_on_b3_i, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_t2_r_b3_i")?,
            "control_vs_t2_c_same_cell_on_b3_i_top1": compute_paired_2x2(&ctrl_b3_i_top1, &t2_c_on_b3_i, CANONICAL_PSUCC_TOP1_FLOOR, "ctrl_vs_t2_c_b3_i")?,
            "control_vs_t2_c_same_cell_on_b3_c_top1": compute_paired_2x2(&ctrl_b3_c_top1, &t2_c_on_b3_c, CANONICAL_PSUCC_TOP1_FLOOR, "ctrl_vs_t2_c_b3_c")?,
            "relation_vs_u_succ_r_same_cell_on_b3_i_top1": compute_paired_2x2(&rel_b3_i_top1, &u_succ_r_on_b3_i, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_u_succ_r_b3_i")?,
        });
        out[rel_pin_key] = Value::String(rel_pin_sha);
        out[ctrl_pin_key] = Value::String(ctrl_pin_sha);
        Ok(out)
    };

    let same_cell_paired_block = if let Some((rel_tc, ctrl_tc, corpus_paths)) =
        token_cells_and_corpus
    {
        verify_loaded_token_cells_against_arm(&rel, rel_tc, "relation_token_cells")?;
        verify_loaded_token_cells_against_arm(&ctrl, ctrl_tc, "control_token_cells")?;
        let (_, d34_rows) =
            parse_d34_record(d34, &rel.d34_sha256, rel.rows.len(), enforce_canonical_pins)?;
        let corpus_tables =
            build_corpus_reference_tables(corpus_paths, &d34_rows, enforce_canonical_pins)?;
        let rel_sc = score_same_cell_rows_from_corpus(&rel, &rel_tc.token_cells, &corpus_tables)?;
        let ctrl_sc =
            score_same_cell_rows_from_corpus(&ctrl, &ctrl_tc.token_cells, &corpus_tables)?;
        let mut block = build_paired_same_cell_json(
            "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
            "relation_token_cells_sha256",
            rel_tc.token_cells_sha256.clone(),
            "control_token_cells_sha256",
            ctrl_tc.token_cells_sha256.clone(),
            rel_tc.occupied_cells,
            ctrl_tc.occupied_cells,
            &rel_sc,
            &ctrl_sc,
        )?;
        block["relation_token_cells_report_sha256"] = json!(rel_tc.report_sha256);
        block["control_token_cells_report_sha256"] = json!(ctrl_tc.report_sha256);
        block["relation_model_storage_sha256"] = json!(rel_tc.model_storage_sha256);
        block["relation_export_model_value_sha256"] = json!(rel_tc.model_snapshot_sha256);
        block["relation_eval_typed_model_snapshot_sha256"] =
            json!(rel_tc.typed_model_snapshot_sha256);
        block["control_model_storage_sha256"] = json!(ctrl_tc.model_storage_sha256);
        block["control_export_model_value_sha256"] = json!(ctrl_tc.model_snapshot_sha256);
        block["control_eval_typed_model_snapshot_sha256"] =
            json!(ctrl_tc.typed_model_snapshot_sha256);
        block["corpus_pins"] = json!({
            "train_tokens_sha256": corpus_tables.train_tokens_sha256,
            "train_mask_sha256": corpus_tables.train_mask_sha256,
            "dev_tokens_sha256": corpus_tables.dev_tokens_sha256,
            "dev_mask_sha256": corpus_tables.dev_mask_sha256,
            "ordered_row_binding_sha256": corpus_tables.ordered_row_binding_sha256,
            "train_matched_targets": corpus_tables.train_matched_targets,
            "train_bigram_total_pairs": corpus_tables.train_bigram_total_pairs,
            "u_train_total_counts": corpus_tables.u_train_total_counts,
            "dev_psucc_rows": corpus_tables.dev_psucc_rows,
            "u_succ_total_counts": corpus_tables.u_succ_total_counts,
            "conservation_ok": corpus_tables.conservation_ok,
            "t2_ranked_table_sha256": corpus_tables.t2_ranked_table_sha256,
            "u_train_ranked_table_sha256": corpus_tables.u_train_ranked_table_sha256,
            "u_succ_ranked_table_sha256": corpus_tables.u_succ_ranked_table_sha256,
            "dev_window_successors_sha256": corpus_tables.dev_window_successors_sha256,
        });
        block
    } else if let Some((_rel_tbl, _rel_tbl_sha, _ctrl_tbl, _ctrl_tbl_sha)) = candidate_tables {
        return Err(
            "paired-dev: external --relation-candidate-cells / --control-candidate-cells without --relation-token-cells / --control-token-cells and pinned corpus files (--train-tokens/--train-mask/--dev-tokens/--dev-mask) is refused; provide --relation-token-cells and --control-token-cells alongside the 4 corpus files so T2/U_succ/U_train/window tables are authenticated against the corpus and token_cells report receipts"
                .to_string(),
        );
    } else {
        json!({
            "status": "UNAVAILABLE_MISSING_GPU_TOKEN_CELL_TABLES_AND_CORPUS_OR_CANDIDATE_TABLES",
            "missing_inputs": [
                "relation_and_control_candidate_cell_tables_or_token_cells_receipts_plus_corpus_files"
            ]
        })
    };

    Ok(json!({
        "schema": PAIRED_DEV_SCORE_SCHEMA,
        "status": "VALIDATED_PAIRED_DEV_CAPTURE_AND_SEALED_D34_SCORE",
        "pins": {
            "panel_sha256": rel.panel_sha256,
            "d34_sha256": rel.d34_sha256,
            "ordered_row_binding_sha256": rel.ordered_row_binding_sha256,
            "completed_batches": rel.completed_batches,
            "relation": {
                "capture_sha256": rel.capture_sha256,
                "eval_sha256": rel.eval_sha256,
                "config_sha256": rel.config_sha256,
                "config_file_sha256": rel.config_file_sha256,
                "lineage_sha256": rel.lineage_sha256,
                "driver_checkpoint_sha256": rel.driver_checkpoint_sha256,
                "checkpoint_file_sha256": rel.checkpoint_file_sha256,
                "model_storage_sha256": rel.model_storage_sha256,
                "model_snapshot_sha256": rel.model_snapshot_sha256,
                "model_epoch": rel.model_epoch,
            },
            "control": {
                "capture_sha256": ctrl.capture_sha256,
                "eval_sha256": ctrl.eval_sha256,
                "config_sha256": ctrl.config_sha256,
                "config_file_sha256": ctrl.config_file_sha256,
                "lineage_sha256": ctrl.lineage_sha256,
                "driver_checkpoint_sha256": ctrl.driver_checkpoint_sha256,
                "checkpoint_file_sha256": ctrl.checkpoint_file_sha256,
                "model_storage_sha256": ctrl.model_storage_sha256,
                "model_snapshot_sha256": ctrl.model_snapshot_sha256,
                "model_epoch": ctrl.model_epoch,
            },
        },
        "admission_summary_psucc": {
            "psucc_rows": psucc_total,
            "b3_r": b3_r,
            "b3_c": b3_c,
            "b3_i_intersection": b3_i,
            "r_only": adm_r_only,
            "c_only": adm_c_only,
            "neither": adm_neither,
            "agreement_numerator": b3_i + adm_neither,
            "agreement_denominator": psucc_total,
            "b3_i_meets_minimum_200": b3_i >= 200,
            "mde_rel_24_over_b3_i": {
                "numerator": CANONICAL_PSUCC_TOP1_FLOOR,
                "denominator": b3_i,
            },
        },
        "paired_comparisons": {
            "relation_vs_control_on_b3_i_top1": compute_paired_2x2(&rel_b3_i_top1, &ctrl_b3_i_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_ctrl_b3_i")?,
            "relation_vs_unconditional_t2_on_b3_i_top1_lower_bound": compute_paired_2x2(&rel_b3_i_top1, &t2_uncond_b3_i_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_t2_uncond_b3_i")?,
            "control_vs_unconditional_t2_on_b3_i_top1_lower_bound": compute_paired_2x2(&ctrl_b3_i_top1, &t2_uncond_b3_i_top1, CANONICAL_PSUCC_TOP1_FLOOR, "ctrl_vs_t2_uncond_b3_i")?,
            "relation_vs_unconditional_u_on_b3_i_top1_lower_bound": compute_paired_2x2(&rel_b3_i_top1, &u_uncond_b3_i_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_u_uncond_b3_i")?,
            "relation_vs_unconditional_t2_on_b3_r_top1_lower_bound": compute_paired_2x2(&rel_b3_r_top1, &t2_uncond_b3_r_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_t2_uncond_b3_r")?,
            "relation_vs_unconditional_u_on_b3_r_top1_lower_bound": compute_paired_2x2(&rel_b3_r_top1, &u_uncond_b3_r_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_u_uncond_b3_r")?,
            "relation_vs_control_psucc_itt_top1": compute_paired_2x2(&rel_itt_top1, &ctrl_itt_top1, CANONICAL_PSUCC_TOP1_FLOOR, "rel_vs_ctrl_itt_top1")?,
            "relation_vs_control_psucc_itt_top8": compute_paired_2x2(&rel_itt_top8, &ctrl_itt_top8, CANONICAL_PSUCC_TOP8_FLOOR, "rel_vs_ctrl_itt_top8")?,
            "relation_vs_control_complement_top1": compute_paired_2x2(&rel_comp_top1, &ctrl_comp_top1, CANONICAL_COMPLEMENT_TOP1_FLOOR, "rel_vs_ctrl_comp_top1")?,
            "relation_vs_control_complement_top8": compute_paired_2x2(&rel_comp_top8, &ctrl_comp_top8, CANONICAL_COMPLEMENT_TOP8_FLOOR, "rel_vs_ctrl_comp_top8")?,
            "ablation": ablation_block,
            "same_cell_reranked": same_cell_paired_block,
        },
        "model_compute_executed_by_scorer": false,
        "updates_applied": 0,
        "scientific_verdict": "DEFERRED_TO_PM_DISPOSITION",
        "training": "HELD",
        "m2": "HELD",
        "stage_b": "HELD",
        "q4": "NOT_EXECUTABLE"
    }))
}

pub fn run_preflight_dev_files(
    panel_path: &Path,
    capture_path: &Path,
    eval_path: &Path,
    d34_path: &Path,
    candidate_cells_path: Option<&Path>,
    token_cells_and_corpus_paths: Option<(&Path, &CorpusPaths<'_>)>,
) -> Result<Value, String> {
    run_preflight_dev_files_with_checkpoint(
        panel_path,
        capture_path,
        eval_path,
        d34_path,
        candidate_cells_path,
        token_cells_and_corpus_paths,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_preflight_dev_files_with_checkpoint(
    panel_path: &Path,
    capture_path: &Path,
    eval_path: &Path,
    d34_path: &Path,
    candidate_cells_path: Option<&Path>,
    token_cells_and_corpus_paths: Option<(&Path, &CorpusPaths<'_>)>,
    explicit_checkpoint_path: Option<&Path>,
) -> Result<Value, String> {
    let (panel, panel_sha) = read_json_and_sha(panel_path, "panel")?;
    let (capture, capture_sha) = read_json_and_sha(capture_path, "capture")?;
    let (eval, eval_sha) = read_json_and_sha(eval_path, "eval")?;
    let (d34, d34_sha) = read_json_and_sha(d34_path, "d34")?;
    let cand_opt = if let Some(p) = candidate_cells_path {
        let bytes = fs::read(p).map_err(|e| format!("candidate_cells ({}): {e}", p.display()))?;
        let sha = sha256_hex(&bytes);
        let parsed: CandidateCellTable = serde_json::from_slice(&bytes)
            .map_err(|e| format!("candidate_cells ({}): invalid JSON: {e}", p.display()))?;
        Some((parsed, sha))
    } else {
        None
    };
    let tc_opt = if let Some((tc_path, corpus_paths)) = token_cells_and_corpus_paths {
        let loaded =
            load_gpu_token_cells_file_with_checkpoint(tc_path, explicit_checkpoint_path, "", "")?;
        Some((loaded, corpus_paths))
    } else {
        None
    };
    score_preflight_dev_values(
        &panel,
        panel_sha,
        &capture,
        capture_sha,
        &eval,
        eval_sha,
        &d34,
        d34_sha,
        cand_opt.as_ref().map(|(t, s)| (t, s.clone())),
        tc_opt.as_ref().map(|(l, cp)| (l, *cp)),
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_paired_dev_files(
    panel_path: &Path,
    rel_capture_path: &Path,
    rel_eval_path: &Path,
    ctrl_capture_path: &Path,
    ctrl_eval_path: &Path,
    ablation_eval_path: Option<&Path>,
    d34_path: &Path,
    candidate_tables_paths: Option<(&Path, &Path)>,
    token_cells_and_corpus_paths: Option<(&Path, &Path, &CorpusPaths<'_>)>,
) -> Result<Value, String> {
    run_paired_dev_files_with_checkpoints(
        panel_path,
        rel_capture_path,
        rel_eval_path,
        ctrl_capture_path,
        ctrl_eval_path,
        ablation_eval_path,
        d34_path,
        candidate_tables_paths,
        token_cells_and_corpus_paths,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_paired_dev_files_with_checkpoints(
    panel_path: &Path,
    rel_capture_path: &Path,
    rel_eval_path: &Path,
    ctrl_capture_path: &Path,
    ctrl_eval_path: &Path,
    ablation_eval_path: Option<&Path>,
    d34_path: &Path,
    candidate_tables_paths: Option<(&Path, &Path)>,
    token_cells_and_corpus_paths: Option<(&Path, &Path, &CorpusPaths<'_>)>,
    rel_checkpoint_path: Option<&Path>,
    ctrl_checkpoint_path: Option<&Path>,
) -> Result<Value, String> {
    let (panel, panel_sha) = read_json_and_sha(panel_path, "panel")?;
    let (rel_cap, rel_cap_sha) = read_json_and_sha(rel_capture_path, "relation_capture")?;
    let (rel_eval, rel_eval_sha) = read_json_and_sha(rel_eval_path, "relation_eval")?;
    let (ctrl_cap, ctrl_cap_sha) = read_json_and_sha(ctrl_capture_path, "control_capture")?;
    let (ctrl_eval, ctrl_eval_sha) = read_json_and_sha(ctrl_eval_path, "control_eval")?;
    let abl_opt = if let Some(p) = ablation_eval_path {
        Some(read_json_and_sha(p, "ablation_eval")?)
    } else {
        None
    };
    let (d34, d34_sha) = read_json_and_sha(d34_path, "d34")?;
    let cand_opt = if let Some((rel_c_path, ctrl_c_path)) = candidate_tables_paths {
        let r_bytes = fs::read(rel_c_path)
            .map_err(|e| format!("relation_candidate_cells ({}): {e}", rel_c_path.display()))?;
        let r_sha = sha256_hex(&r_bytes);
        let r_parsed: CandidateCellTable = serde_json::from_slice(&r_bytes).map_err(|e| {
            format!(
                "relation_candidate_cells ({}): invalid JSON: {e}",
                rel_c_path.display()
            )
        })?;
        let c_bytes = fs::read(ctrl_c_path)
            .map_err(|e| format!("control_candidate_cells ({}): {e}", ctrl_c_path.display()))?;
        let c_sha = sha256_hex(&c_bytes);
        let c_parsed: CandidateCellTable = serde_json::from_slice(&c_bytes).map_err(|e| {
            format!(
                "control_candidate_cells ({}): invalid JSON: {e}",
                ctrl_c_path.display()
            )
        })?;
        Some((r_parsed, r_sha, c_parsed, c_sha))
    } else {
        None
    };
    let tcs_opt = if let Some((rel_tc_path, ctrl_tc_path, corpus_paths)) =
        token_cells_and_corpus_paths
    {
        let r_loaded =
            load_gpu_token_cells_file_with_checkpoint(rel_tc_path, rel_checkpoint_path, "", "")?;
        let c_loaded =
            load_gpu_token_cells_file_with_checkpoint(ctrl_tc_path, ctrl_checkpoint_path, "", "")?;
        Some((r_loaded, c_loaded, corpus_paths))
    } else {
        None
    };
    score_paired_dev_values(
        &panel,
        panel_sha,
        &rel_cap,
        rel_cap_sha,
        &rel_eval,
        rel_eval_sha,
        &ctrl_cap,
        ctrl_cap_sha,
        &ctrl_eval,
        ctrl_eval_sha,
        abl_opt.as_ref().map(|(v, s)| (v, s.clone())),
        &d34,
        d34_sha,
        cand_opt
            .as_ref()
            .map(|(rt, rts, ct, cts)| (rt, rts.clone(), ct, cts.clone())),
        tcs_opt.as_ref().map(|(rl, cl, cp)| (rl, cl, *cp)),
        true,
    )
}

pub fn run_bundle_candidate_cells_files(
    token_cells_path: &Path,
    explicit_driver_checkpoint_sha256: Option<&str>,
    d34_path: &Path,
    corpus_paths: &CorpusPaths<'_>,
) -> Result<CandidateCellTable, String> {
    run_bundle_candidate_cells_files_with_checkpoint(
        token_cells_path,
        None,
        explicit_driver_checkpoint_sha256,
        d34_path,
        corpus_paths,
    )
}

pub fn run_bundle_candidate_cells_files_with_checkpoint(
    token_cells_path: &Path,
    explicit_checkpoint_path: Option<&Path>,
    explicit_driver_checkpoint_sha256: Option<&str>,
    d34_path: &Path,
    corpus_paths: &CorpusPaths<'_>,
) -> Result<CandidateCellTable, String> {
    let loaded = load_gpu_token_cells_file_with_checkpoint(
        token_cells_path,
        explicit_checkpoint_path,
        explicit_driver_checkpoint_sha256.unwrap_or(""),
        "",
    )?;
    let (d34_val, d34_sha) = read_json_and_sha(d34_path, "d34")?;
    build_candidate_cell_table_from_loaded(
        &loaded,
        explicit_driver_checkpoint_sha256,
        &d34_val,
        &d34_sha,
        corpus_paths,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_fips_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn paired_integer_floor_uses_exact_ceiling() {
        assert_eq!(paired_integer_floor(24, 0), 24);
        assert_eq!(paired_integer_floor(24, 144), 24);
        assert_eq!(paired_integer_floor(24, 145), 25);
        assert_eq!(paired_integer_floor(22, 121), 22);
        assert_eq!(paired_integer_floor(22, 122), 23);
    }

    fn make_fixture_lineage(policy: RelationLearningPolicyRecord) -> (Value, String, String) {
        let spec = SpecRecord {
            vocabulary: CANONICAL_VOCAB as u32,
            blocks: 4,
            context: CANONICAL_CONTEXT as u32,
            kept_cells: 8,
        };
        let bos = BosIdentityRecord {
            token: "<|bos|>".to_string(),
            id: u32::from(CANONICAL_BOS),
        };
        let train_source = CorpusSourceRecord {
            split: SplitRecord::Train,
            tokens: FilePinRecord {
                path: "/tmp/train.u16".into(),
                sha256: CANONICAL_TRAIN_TOKENS_SHA256.to_string(),
            },
            mask: FilePinRecord {
                path: "/tmp/train.u8".into(),
                sha256: CANONICAL_TRAIN_MASK_SHA256.to_string(),
            },
            expected_payload_tokens: CANONICAL_TRAIN_PAYLOAD_TOKENS,
            expected_eligible_targets: CANONICAL_TRAIN_ELIGIBLE_ROWS as u64,
        };
        let dev_source = CorpusSourceRecord {
            split: SplitRecord::Dev,
            tokens: FilePinRecord {
                path: "/tmp/dev.u16".into(),
                sha256: CANONICAL_DEV_TOKENS_SHA256.to_string(),
            },
            mask: FilePinRecord {
                path: "/tmp/dev.u8".into(),
                sha256: CANONICAL_DEV_MASK_SHA256.to_string(),
            },
            expected_payload_tokens: CANONICAL_DEV_PAYLOAD_TOKENS,
            expected_eligible_targets: 2,
        };
        let cfg = DriverConfigRecord {
            schema: DRIVER_EVAL_SCHEMA.to_string(),
            spec,
            read_spec: ReadSpecRecord::Relation { kept: 8, sweeps: 1 },
            objective_policy: ObjectivePolicyRecord::NearestAddressMarginV1,
            read_initialization: ReadInitializationRecord::ActiveLeafV1,
            relation_learning_policy: policy,
            seed: 1,
            batch_rows: 64,
            train: train_source.clone(),
            dev: dev_source.clone(),
            tokenizer: FilePinRecord {
                path: "/tmp/tok.json".into(),
                sha256: CANONICAL_TOKENIZER_SHA256.to_string(),
            },
            bos: bos.clone(),
            schedule: SchedulePolicyRecord::InterleavedReadEvery8V1,
        };
        let cfg_sha = sha256_hex(&serde_json::to_vec(&cfg).unwrap());
        let lineage = DriverLineageRecord {
            schema: DRIVER_EVAL_SCHEMA.to_string(),
            driver_source_sha256: sha256_hex(b"driver_source"),
            config: cfg,
            train: CorpusBindingRecord {
                source: train_source,
                vocabulary: CANONICAL_VOCAB as u32,
                bos: bos.clone(),
                payload_tokens: CANONICAL_TRAIN_PAYLOAD_TOKENS,
                mask_bytes: CANONICAL_TRAIN_PAYLOAD_TOKENS,
                mask_selected_positions: CANONICAL_TRAIN_ELIGIBLE_ROWS as u64,
                eligible_targets: CANONICAL_TRAIN_ELIGIBLE_ROWS as u64,
                documents: 100,
                window_policy: "causal_document_bounded_v1".to_string(),
                identity_policy: "exact_uor_v1".to_string(),
            },
            dev: CorpusBindingRecord {
                source: dev_source,
                vocabulary: CANONICAL_VOCAB as u32,
                bos,
                payload_tokens: CANONICAL_DEV_PAYLOAD_TOKENS,
                mask_bytes: CANONICAL_DEV_PAYLOAD_TOKENS,
                mask_selected_positions: 2,
                eligible_targets: 2,
                documents: 1,
                window_policy: "causal_document_bounded_v1".to_string(),
                identity_policy: "exact_uor_v1".to_string(),
            },
        };
        let lin_val = serde_json::to_value(&lineage).unwrap();
        let lin_sha = sha256_hex(&serde_json::to_vec(&lineage).unwrap());
        (lin_val, cfg_sha, lin_sha)
    }

    fn fixture_receipts(
        rel_top1_row0: u64,
        ctrl_top1_row0: u64,
    ) -> (Value, Value, Value, Value, Value, Value) {
        let (rel_lineage_val, rel_cfg_sha, rel_lin_sha) =
            make_fixture_lineage(RelationLearningPolicyRecord::LearnV1);
        let (ctrl_lineage_val, ctrl_cfg_sha, ctrl_lin_sha) =
            make_fixture_lineage(RelationLearningPolicyRecord::FrozenZeroV1);

        let raw_rel_cfg_file_sha = sha256_hex(b"raw_rel_cfg_file");
        let raw_ctrl_cfg_file_sha = sha256_hex(b"raw_ctrl_cfg_file");
        let raw_ckpt_rel_file_sha = sha256_hex(b"raw_ckpt_rel_file");
        let raw_ckpt_ctrl_file_sha = sha256_hex(b"raw_ckpt_ctrl_file");
        let ckpt_rel_sha = sha256_hex(b"ckpt_rel_canonical");
        let ckpt_ctrl_sha = sha256_hex(b"ckpt_ctrl_canonical");
        let snap_rel_sha = sha256_hex(b"snap_rel_canonical");
        let snap_ctrl_sha = sha256_hex(b"snap_ctrl_canonical");
        let source_sha = sha256_hex(b"constructed_source");
        let kernel_sha = sha256_hex(b"kernel_source");
        let geom_sha = sha256_hex(b"geometry_source");
        let dev_binding_val = rel_lineage_val["dev"].clone();

        let panel = json!({
            "schema": PREFLIGHT_PANEL_SCHEMA,
            "status": "PINNED_PANEL_AND_PROBE_METADATA_ONLY",
            "config_file": {"path": "/tmp/rel_cfg.json", "sha256": raw_rel_cfg_file_sha},
            "config_sha256": rel_cfg_sha,
            "lineage_sha256": rel_lin_sha,
            "dev_binding": dev_binding_val,
            "row_count": 2,
            "psucc_count": 1,
            "rows": [
                {"row_ordinal": 0, "target_position": 10, "target_token": 7, "is_psucc": true},
                {"row_ordinal": 1, "target_position": 20, "target_token": 9, "is_psucc": false}
            ],
            "scoring_performed": false,
            "cuda_context_created": false,
            "model_constructed": false,
            "updates_applied": 0,
            "release": "STILL_REQUIRED",
            "training": "HELD",
            "m2": "HELD",
            "stage_b": "HELD",
            "q4": "NOT_EXECUTABLE"
        });

        let make_eval = |lin_val: &Value, lin_sha: &str, snap_sha: &str, top1_row0: u64| {
            json!({
                "schema": DRIVER_EVAL_SCHEMA,
                "evaluation_lineage": lin_val,
                "evaluation_lineage_sha256": lin_sha,
                "model_snapshot_sha256": snap_sha,
                "model_epoch": 0,
                "accepted_updates_in_model": 0,
                "evaluation_mutates_model": false,
                "updates_applied_by_evaluation": 0,
                "model_bindings": {
                    "schema": DISCRETE_SNAPSHOT_SCHEMA,
                    "spec": lin_val["config"]["spec"],
                    "read_spec": lin_val["config"]["read_spec"],
                    "objective_policy": "nearest_address_margin_v1",
                    "seed": lin_val["config"]["seed"],
                    "source_sha256": source_sha,
                    "kernel_sha256": kernel_sha,
                    "geometry_sha256": geom_sha,
                    "operator_kappa": "spiralcore_fbs_v1",
                    "composition_kappa": "g2_tree_v1"
                },
                "batches": [
                    {
                        "rows": [
                            {
                                "target_position": 10,
                                "target_token": 7,
                                "admitted_cells": 2,
                                "address_rivals": 1,
                                "top1_correct": top1_row0,
                                "top8_correct": 1
                            },
                            {
                                "target_position": 20,
                                "target_token": 9,
                                "admitted_cells": 1,
                                "address_rivals": 1,
                                "top1_correct": 0,
                                "top8_correct": 0
                            }
                        ]
                    }
                ]
            })
        };

        let rel_eval = make_eval(&rel_lineage_val, &rel_lin_sha, &snap_rel_sha, rel_top1_row0);
        let ctrl_eval = make_eval(
            &ctrl_lineage_val,
            &ctrl_lin_sha,
            &snap_ctrl_sha,
            ctrl_top1_row0,
        );
        let rel_eval_sha = sha256_hex(&serde_json::to_vec(&rel_eval).unwrap());
        let ctrl_eval_sha = sha256_hex(&serde_json::to_vec(&ctrl_eval).unwrap());

        // Matches protected attention-capture-dev (attention_preflight.rs:421-435): omits model_snapshot_sha256.
        let make_cap = |cfg_file_sha: &str,
                        cfg_sha: &str,
                        lin_sha: &str,
                        ckpt_file_sha: &str,
                        ckpt_sha: &str,
                        eval_sha: &str| {
            json!({
                "schema": PREFLIGHT_CAPTURE_SCHEMA,
                "status": "ACTUAL_GPU_FORWARD_CELL_IDS_CAPTURED_ZERO_APPLY",
                "config_file": {"path": "/tmp/cfg.json", "sha256": cfg_file_sha},
                "config_sha256": cfg_sha,
                "lineage_sha256": lin_sha,
                "checkpoint_file": {"path": "/tmp/ckpt.json", "sha256": ckpt_file_sha},
                "driver_checkpoint_sha256": ckpt_sha,
                "evaluation_file": {"path": "/tmp/eval.json", "sha256": eval_sha},
                "dev_binding": dev_binding_val,
                "model_epoch": 0,
                "completed_batches": 0,
                "physical_cell_words_per_row": 16,
                "unused_slot_marker": UNUSED_CELL_SLOT,
                "row_count": 2,
                "b3": {
                    "full": {"rows": 2, "target_admitted": 1},
                    "psucc": {"rows": 1, "target_admitted": 1},
                    "complement": {"rows": 1, "target_admitted": 0}
                },
                "rows": [
                    {
                        "row_ordinal": 0,
                        "target_position": 10,
                        "target_token": 7,
                        "is_psucc": true,
                        "eval_target_admitted": true,
                        "model_epoch": 0,
                        "admitted_cells": [0, 34, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT]
                    },
                    {
                        "row_ordinal": 1,
                        "target_position": 20,
                        "target_token": 9,
                        "is_psucc": false,
                        "eval_target_admitted": false,
                        "model_epoch": 0,
                        "admitted_cells": [5, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT, UNUSED_CELL_SLOT]
                    }
                ],
                "additional_zero_apply_gpu_forward_passes": 1,
                "gpu_forward_calls": 1,
                "model_restores": 1,
                "updates_applied": 0,
                "proposals": 0,
                "snapshot_unchanged": true,
                "training_capability_issued": false,
                "release": "STILL_REQUIRED",
                "training": "HELD",
                "m2": "HELD",
                "stage_b": "HELD",
                "q4": "NOT_EXECUTABLE"
            })
        };

        let d34 = json!({
            "schema": D34_RECORD_SCHEMA,
            "host_only": true,
            "model_compute_executed": false,
            "applied_updates": 0,
            "inputs": {
                "partition": {
                    "ordered_row_binding_sha256": CANONICAL_ORDERED_ROW_BINDING_SHA256
                }
            },
            "tables": {
                "T2": {
                    "pairs": CANONICAL_TRAIN_MATCHED_TARGETS,
                    "conservation_ok": true
                }
            },
            "slices": {
                "copy_successor_subpanel": {
                    "t1_hit1": 1
                }
            },
            "per_row": [
                {
                    "ordinal": 0,
                    "position": 10,
                    "query_token": 3,
                    "target_token": 7,
                    "copy_successor": true,
                    "distance_bucket": 0,
                    "conflict": false,
                    "t1b_prediction": 4,
                    "t1b_hit1": 0,
                    "t2_prediction": 7,
                    "t2_hit1": 1,
                    "t2_hit8": 1,
                    "t3_prediction": 7,
                    "t3_hit1": 1,
                    "u_hit1": 0,
                    "u_hit8": 1
                },
                {
                    "ordinal": 1,
                    "position": 20,
                    "query_token": 5,
                    "target_token": 9,
                    "copy_successor": false,
                    "distance_bucket": null,
                    "conflict": true,
                    "t1b_prediction": 9,
                    "t1b_hit1": 1,
                    "t2_prediction": 8,
                    "t2_hit1": 0,
                    "t2_hit8": 1,
                    "t3_prediction": 8,
                    "t3_hit1": 0,
                    "u_hit1": 0,
                    "u_hit8": 0
                }
            ]
        });

        let rel_cap = make_cap(
            &raw_rel_cfg_file_sha,
            &rel_cfg_sha,
            &rel_lin_sha,
            &raw_ckpt_rel_file_sha,
            &ckpt_rel_sha,
            &rel_eval_sha,
        );
        let ctrl_cap = make_cap(
            &raw_ctrl_cfg_file_sha,
            &ctrl_cfg_sha,
            &ctrl_lin_sha,
            &raw_ckpt_ctrl_file_sha,
            &ckpt_ctrl_sha,
            &ctrl_eval_sha,
        );
        (panel, rel_cap, rel_eval, ctrl_cap, ctrl_eval, d34)
    }

    #[test]
    fn preflight_and_paired_scoring_validate_distinct_file_and_canonical_hashes_and_complement_floor_22(
    ) {
        let (panel, rel_cap, rel_eval, ctrl_cap, ctrl_eval, d34) = fixture_receipts(1, 0);
        assert!(rel_cap.get("model_snapshot_sha256").is_none());
        assert_eq!(
            rel_eval["evaluation_lineage"]["train"]["eligible_targets"],
            CANONICAL_TRAIN_ELIGIBLE_ROWS as u64
        );
        assert_eq!(
            d34["tables"]["T2"]["pairs"],
            CANONICAL_TRAIN_MATCHED_TARGETS
        );
        let rel_eval_sha = rel_cap["evaluation_file"]["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        let ctrl_eval_sha = ctrl_cap["evaluation_file"]["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        // Verify canonical pin enforcement accepts full TRAIN inventory (2,425,925) and rejects consumed budget (1,049,856) in lineage.train.eligible_targets.
        parse_eval_rows(&rel_eval, &rel_eval_sha, 2, true).unwrap();
        let mut wrong_train_inv_eval = rel_eval.clone();
        wrong_train_inv_eval["evaluation_lineage"]["config"]["train"]
            ["expected_eligible_targets"] = json!(CANONICAL_TRAIN_MATCHED_TARGETS as u64);
        wrong_train_inv_eval["evaluation_lineage"]["train"]["source"]
            ["expected_eligible_targets"] = json!(CANONICAL_TRAIN_MATCHED_TARGETS as u64);
        wrong_train_inv_eval["evaluation_lineage"]["train"]["eligible_targets"] =
            json!(CANONICAL_TRAIN_MATCHED_TARGETS as u64);
        let wrong_lin: DriverLineageRecord =
            serde_json::from_value(wrong_train_inv_eval["evaluation_lineage"].clone()).unwrap();
        wrong_train_inv_eval["evaluation_lineage_sha256"] =
            json!(sha256_hex(&serde_json::to_vec(&wrong_lin).unwrap()));
        let wrong_eval_sha = sha256_hex(&serde_json::to_vec(&wrong_train_inv_eval).unwrap());
        let inv_err = parse_eval_rows(&wrong_train_inv_eval, &wrong_eval_sha, 2, true).unwrap_err();
        assert!(
            inv_err.contains("canonical TRAIN/DEV corpus pin or eligible_targets mismatch"),
            "{inv_err}"
        );

        let pre = score_preflight_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .expect("preflight score succeeds with distinct raw FilePin vs canonical hashes");

        assert_eq!(pre["schema"], PREFLIGHT_SCORE_SCHEMA);
        assert_eq!(pre["pins"]["config_sha256"], rel_cap["config_sha256"]);
        assert_eq!(
            pre["pins"]["config_file_sha256"],
            rel_cap["config_file"]["sha256"]
        );
        assert_eq!(
            pre["pins"]["driver_checkpoint_sha256"],
            rel_cap["driver_checkpoint_sha256"]
        );
        assert_eq!(
            pre["pins"]["checkpoint_file_sha256"],
            rel_cap["checkpoint_file"]["sha256"]
        );
        assert_eq!(
            pre["pins"]["model_snapshot_sha256"],
            rel_eval["model_snapshot_sha256"]
        );
        assert_eq!(
            pre["same_cell_candidate_reranking"]["status"],
            "UNAVAILABLE_MISSING_GPU_TOKEN_CELL_TABLE_AND_RANKED_CANDIDATE_RECEIPT"
        );

        // Optional capture.model_snapshot_sha256 is consistency-checked when present.
        let mut bad_snap_cap = rel_cap.clone();
        bad_snap_cap["model_snapshot_sha256"] = json!(sha256_hex(b"wrong_snap"));
        let snap_err = score_preflight_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &bad_snap_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(snap_err.contains("model_snapshot_sha256"), "{snap_err}");

        let paired = score_paired_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &ctrl_cap,
            sha256_hex(b"cc_sha"),
            &ctrl_eval,
            ctrl_eval_sha.clone(),
            None,
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .expect("paired score succeeds with distinct LearnV1 vs FrozenZeroV1 configs/lineages");

        assert_eq!(paired["schema"], PAIRED_DEV_SCORE_SCHEMA);
        assert_eq!(paired["admission_summary_psucc"]["b3_i_intersection"], 1);
        assert_eq!(
            paired["paired_comparisons"]["relation_vs_control_on_b3_i_top1"]
                ["difference_a_minus_b"],
            1
        );
        assert_eq!(
            paired["paired_comparisons"]["relation_vs_control_complement_top1"]["mde_floor"],
            22
        );
        assert_eq!(paired["scientific_verdict"], "DEFERRED_TO_PM_DISPOSITION");

        // Rejects paired arms if Relation and Control are swapped (Relation has FrozenZeroV1, Control has LearnV1).
        let swapped_err = score_paired_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &ctrl_cap,
            sha256_hex(b"cc_sha"),
            &ctrl_eval,
            ctrl_eval_sha.clone(),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            None,
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(
            swapped_err.contains("relation arm relation_learning_policy must be LearnV1"),
            "{swapped_err}"
        );

        // Rejects paired arms if Control differs in a non-policy DriverConfig field (e.g. seed 2 vs 1).
        let mut forged_ctrl_eval = ctrl_eval.clone();
        forged_ctrl_eval["evaluation_lineage"]["config"]["seed"] = json!(2);
        forged_ctrl_eval["model_bindings"]["seed"] = json!(2);
        let forged_ctrl_cfg: DriverConfigRecord =
            serde_json::from_value(forged_ctrl_eval["evaluation_lineage"]["config"].clone())
                .unwrap();
        let forged_ctrl_cfg_sha = sha256_hex(&serde_json::to_vec(&forged_ctrl_cfg).unwrap());
        let forged_ctrl_lin: DriverLineageRecord =
            serde_json::from_value(forged_ctrl_eval["evaluation_lineage"].clone()).unwrap();
        let forged_ctrl_lin_sha = sha256_hex(&serde_json::to_vec(&forged_ctrl_lin).unwrap());
        forged_ctrl_eval["evaluation_lineage_sha256"] = json!(forged_ctrl_lin_sha);
        let forged_ctrl_eval_sha = sha256_hex(&serde_json::to_vec(&forged_ctrl_eval).unwrap());
        let mut forged_ctrl_cap = ctrl_cap.clone();
        forged_ctrl_cap["config_sha256"] = json!(forged_ctrl_cfg_sha);
        forged_ctrl_cap["lineage_sha256"] = json!(forged_ctrl_lin_sha);
        forged_ctrl_cap["evaluation_file"]["sha256"] = json!(forged_ctrl_eval_sha);
        let seed_err = score_paired_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha,
            &forged_ctrl_cap,
            sha256_hex(b"cc_sha"),
            &forged_ctrl_eval,
            forged_ctrl_eval_sha,
            None,
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(
            seed_err.contains("other than relation_learning_policy"),
            "{seed_err}"
        );
    }

    #[test]
    fn rejects_scalar_or_mismatched_slot_captures() {
        let (panel, mut rel_cap, rel_eval, _, _, d34) = fixture_receipts(1, 0);
        let rel_eval_sha = rel_cap["evaluation_file"]["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        rel_cap["rows"][0]["admitted_cells"] = json!(2);
        let err = score_preflight_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha,
            &d34,
            sha256_hex(b"d34_sha"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(err.contains("16-word array"));
    }

    #[test]
    fn token_cells_receipt_and_candidate_table_validate_hashes_ranges_and_d34_consistency() {
        let (panel, rel_cap, rel_eval, ctrl_cap, ctrl_eval, d34) = fixture_receipts(1, 0);
        let rel_eval_sha = rel_cap["evaluation_file"]["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        let ctrl_eval_sha = ctrl_cap["evaluation_file"]["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        let arm = validate_arm_inputs(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            false,
        )
        .unwrap();

        let token_cells = vec![0u32; CANONICAL_VOCAB];
        let mut u16_bytes = Vec::with_capacity(CANONICAL_VOCAB * 2);
        for &c in &token_cells {
            u16_bytes.extend_from_slice(&(c as u16).to_le_bytes());
        }
        let tc_sha = sha256_hex(&u16_bytes);
        verify_token_cells_against_gpu_capture(&arm, &token_cells).unwrap();

        let mut t2_by_q = vec![vec![7u32]; CANONICAL_VOCAB];
        t2_by_q[3] = vec![7, 4];
        t2_by_q[5] = vec![8, 9];
        let u_ranked = vec![2u32, 7, 4, 8];
        let occs = vec![vec![7u32, 4], vec![8u32, 9]];
        let t2_sha = sha256_hex(&serde_json::to_vec(&t2_by_q).unwrap());
        let u_sha = sha256_hex(&serde_json::to_vec(&u_ranked).unwrap());
        let win_sha = sha256_hex(&serde_json::to_vec(&occs).unwrap());

        let corpus_tables = CorpusReferenceTables {
            train_tokens_sha256: CANONICAL_TRAIN_TOKENS_SHA256.to_string(),
            train_mask_sha256: CANONICAL_TRAIN_MASK_SHA256.to_string(),
            dev_tokens_sha256: CANONICAL_DEV_TOKENS_SHA256.to_string(),
            dev_mask_sha256: CANONICAL_DEV_MASK_SHA256.to_string(),
            ordered_row_binding_sha256: CANONICAL_ORDERED_ROW_BINDING_SHA256.to_string(),
            train_matched_targets: CANONICAL_TRAIN_MATCHED_TARGETS as u64,
            train_bigram_total_pairs: CANONICAL_TRAIN_MATCHED_TARGETS as u64,
            u_train_total_counts: CANONICAL_TRAIN_MATCHED_TARGETS as u64,
            dev_psucc_rows: 1,
            u_succ_total_counts: CANONICAL_TRAIN_MATCHED_TARGETS as u64,
            conservation_ok: true,
            t2_ranked_by_query: t2_by_q.clone(),
            t2_ranked_table_sha256: t2_sha.clone(),
            u_train_ranked: u_ranked.clone(),
            u_train_ranked_table_sha256: u_sha.clone(),
            u_succ_ranked: u_ranked.clone(),
            u_succ_ranked_table_sha256: u_sha.clone(),
            dev_rows: vec![
                DevCorpusRow {
                    ordinal: 0,
                    position: 10,
                    query_token: 3,
                    target_token: 7,
                    is_psucc: true,
                    distance_bucket: Some(0),
                    conflict: false,
                    t1_candidate: Some(7),
                    t1b_candidate: Some(4),
                    window_query_successors_most_recent_first: vec![7, 4],
                },
                DevCorpusRow {
                    ordinal: 1,
                    position: 20,
                    query_token: 5,
                    target_token: 9,
                    is_psucc: false,
                    distance_bucket: None,
                    conflict: true,
                    t1_candidate: Some(8),
                    t1b_candidate: Some(9),
                    window_query_successors_most_recent_first: vec![8, 9],
                },
            ],
            dev_window_successors_sha256: win_sha.clone(),
        };

        let mut cand_table = CandidateCellTable {
            schema: CANDIDATE_CELL_TABLE_SCHEMA.to_string(),
            driver_checkpoint_sha256: arm.driver_checkpoint_sha256.clone(),
            model_snapshot_sha256: Some(arm.model_snapshot_sha256.clone()),
            model_storage_sha256: None,
            model_value_sha256: None,
            ordered_row_binding_sha256: Some(CANONICAL_ORDERED_ROW_BINDING_SHA256.to_string()),
            token_cells_sha256: Some(tc_sha),
            t2_ranked_table_sha256: Some(t2_sha),
            u_succ_ranked_table_sha256: Some(u_sha.clone()),
            u_train_ranked_table_sha256: Some(u_sha),
            dev_window_successors_sha256: Some(win_sha),
            token_cells: token_cells.clone(),
            t2_ranked_candidates_by_query: t2_by_q,
            u_succ_ranked_candidates: u_ranked.clone(),
            u_train_ranked_candidates: Some(u_ranked),
            dev_window_query_successors_most_recent_first: Some(occs),
        };

        let (row_scores, occupied_cells) =
            score_same_cell_rows_from_table(&arm, &cand_table, &corpus_tables).unwrap();
        assert_eq!(occupied_cells, 1);
        assert_eq!(row_scores.len(), 2);
        assert!(row_scores[0].t1_admitted_hit1);
        assert!(row_scores[0].t2_same_cell_hit1);

        // Standalone external CandidateCellTable without token_cells + corpus files must be refused by both scorers.
        let ext_pre_err = score_preflight_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            Some((&cand_table, "cand_sha".to_string())),
            None,
            false,
        )
        .unwrap_err();
        assert!(
            ext_pre_err.contains("external --candidate-cells without --token-cells"),
            "{ext_pre_err}"
        );
        let ext_paired_err = score_paired_dev_values(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha,
            &ctrl_cap,
            sha256_hex(b"cc_sha"),
            &ctrl_eval,
            ctrl_eval_sha,
            None,
            &d34,
            sha256_hex(b"d34_sha"),
            Some((
                &cand_table,
                "cand_rel".to_string(),
                &cand_table,
                "cand_ctrl".to_string(),
            )),
            None,
            false,
        )
        .unwrap_err();
        assert!(
            ext_paired_err.contains(
                "external --relation-candidate-cells / --control-candidate-cells without"
            ),
            "{ext_paired_err}"
        );

        // Tampering with non-top-1 tail of t2_ranked_candidates_by_query (even with recomputed JSON sha) must fail corpus cross-check.
        let mut tampered_tail_table = cand_table.clone();
        tampered_tail_table.t2_ranked_candidates_by_query[3] = vec![7, 4, 15];
        tampered_tail_table.t2_ranked_table_sha256 = Some(sha256_hex(
            &serde_json::to_vec(&tampered_tail_table.t2_ranked_candidates_by_query).unwrap(),
        ));
        let tail_err = score_same_cell_rows_from_table(&arm, &tampered_tail_table, &corpus_tables)
            .unwrap_err();
        assert!(
            tail_err.contains("disagrees with corpus-recomputed T2 table"),
            "{tail_err}"
        );

        // Out-of-range token ID in t2_ranked_candidates_by_query must fail cleanly without panic.
        let mut bad_table = cand_table.clone();
        bad_table.t2_ranked_candidates_by_query[3].push(4096);
        assert!(score_same_cell_rows_from_table(&arm, &bad_table, &corpus_tables).is_err());

        // Mismatched u_succ_ranked_candidates vs u_train_ranked_candidates must fail cleanly.
        cand_table.u_succ_ranked_candidates = vec![7, 2, 4, 8];
        assert!(score_same_cell_rows_from_table(&arm, &cand_table, &corpus_tables).is_err());
    }

    #[test]
    fn raw_token_cells_u16_requires_sibling_report_receipt_and_rejects_bare_file() {
        let (panel, mut rel_cap, mut rel_eval, _, _, d34) = fixture_receipts(1, 0);
        let mut raw = Vec::with_capacity(CANONICAL_VOCAB * 2);
        for tok in 0..CANONICAL_VOCAB {
            let cell = ((tok * 17) % 256) as u16;
            raw.extend_from_slice(&cell.to_le_bytes());
        }
        let (cells, occupied) = parse_token_cells_u16_bytes(&raw, "token_cells.u16").unwrap();
        assert_eq!(cells.len(), CANONICAL_VOCAB);
        assert_eq!(occupied, 256);
        assert_eq!(cells[7], 119);

        let tmp_dir = std::env::temp_dir().join(format!("uor_r4_tc_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();
        let u16_path = tmp_dir.join("token_cells.u16");
        fs::write(&u16_path, &raw).unwrap();

        // Construct a real DriverCheckpointRecord whose 4 hash domains are all distinct:
        // 1. checkpoint_file.sha256 (pretty-printed JSON bytes)
        // 2. driver_checkpoint_sha256 (canonical DriverCheckpointRecord bytes)
        // 3. checkpoint.model_sha256 == export.model_snapshot_sha256 (serde_json::Value alphabetically sorted keys)
        // 4. typed_model_snapshot_sha256 == eval.model_snapshot_sha256 (DeviceSnapshotRecord declaration order)
        let typed_snapshot = DeviceSnapshotRecord {
            schema: DISCRETE_SNAPSHOT_SCHEMA.to_string(),
            spec: serde_json::from_value(rel_eval["model_bindings"]["spec"].clone()).unwrap(),
            read_spec: serde_json::from_value(rel_eval["model_bindings"]["read_spec"].clone())
                .unwrap(),
            objective_policy: ObjectivePolicyRecord::NearestAddressMarginV1,
            read_initialization: ReadInitializationRecord::ActiveLeafV1,
            seed: 1,
            epoch: 0,
            table: vec![0; 131072],
            routes: vec![0; 16384],
            gates: vec![0; 45],
            operator_kappa: "spiralcore_fbs_v1".to_string(),
            composition_kappa: "g2_tree_v1".to_string(),
            kernel_sha256: rel_eval["model_bindings"]["kernel_sha256"]
                .as_str()
                .unwrap()
                .to_string(),
            source_sha256: rel_eval["model_bindings"]["source_sha256"]
                .as_str()
                .unwrap()
                .to_string(),
            geometry_sha256: rel_eval["model_bindings"]["geometry_sha256"]
                .as_str()
                .unwrap()
                .to_string(),
        };
        let typed_snap_sha = sha256_hex(&serde_json::to_vec(&typed_snapshot).unwrap());
        let model_val = serde_json::to_value(&typed_snapshot).unwrap();
        let value_model_sha = sha256_hex(&serde_json::to_vec(&model_val).unwrap());
        assert_ne!(typed_snap_sha, value_model_sha);

        let ckpt_rec = DriverCheckpointRecord {
            schema: DRIVER_CHECKPOINT_SCHEMA.to_string(),
            lineage: serde_json::from_value(rel_eval["evaluation_lineage"].clone()).unwrap(),
            cursor: DriverCursorRecord {
                completed_batches: 0,
                accepted_updates: 0,
                corpus_pass: 0,
                next_target: 0,
                next_coordinate: 0,
            },
            model_sha256: value_model_sha.clone(),
            model: model_val,
        };
        let ckpt_sha = sha256_hex(&serde_json::to_vec(&ckpt_rec).unwrap());
        let ckpt_pretty_bytes = serde_json::to_vec_pretty(&ckpt_rec).unwrap();
        let ckpt_file_sha = sha256_hex(&ckpt_pretty_bytes);
        assert_ne!(ckpt_file_sha, ckpt_sha);
        assert_ne!(ckpt_sha, value_model_sha);
        assert_ne!(ckpt_sha, typed_snap_sha);

        let ckpt_path = tmp_dir.join("checkpoint.json");
        fs::write(&ckpt_path, &ckpt_pretty_bytes).unwrap();

        rel_eval["model_snapshot_sha256"] = json!(typed_snap_sha);
        let rel_eval_sha = sha256_hex(&serde_json::to_vec(&rel_eval).unwrap());
        rel_cap["checkpoint_file"] =
            json!({"path": ckpt_path.to_string_lossy(), "sha256": ckpt_file_sha});
        rel_cap["driver_checkpoint_sha256"] = json!(ckpt_sha);
        rel_cap["evaluation_file"]["sha256"] = json!(rel_eval_sha);

        // Bare token_cells.u16 without sibling report.json must be refused.
        let bare_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(
            bare_err.contains("without sibling report.json"),
            "{bare_err}"
        );

        // Inline token_cells JSON array without raw_u16_le_no_header binary file must be refused.
        let inline_json_path = tmp_dir.join("inline_receipt.json");
        let inline_val = json!({
            "schema": TOKEN_CELLS_RECEIPT_SCHEMA,
            "status": TOKEN_CELLS_RECEIPT_STATUS,
            "driver_checkpoint_sha256": ckpt_sha,
            "model_snapshot_sha256": value_model_sha,
            "updates_applied": 0,
            "snapshot_unchanged": true,
            "token_cells": cells
        });
        fs::write(&inline_json_path, serde_json::to_vec(&inline_val).unwrap()).unwrap();
        let inline_err =
            load_gpu_token_cells_file(&inline_json_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(
            inline_err.contains("inline token_cells are refused"),
            "{inline_err}"
        );

        // Writing a valid #2601 sibling report.json (with Value model_snapshot_sha256) succeeds and binds all 4 checkpoint/model domains.
        let u16_sha = sha256_hex(&raw);
        let report_val = json!({
            "schema": TOKEN_CELLS_RECEIPT_SCHEMA,
            "status": TOKEN_CELLS_RECEIPT_STATUS,
            "config_file": rel_cap["config_file"],
            "config_sha256": rel_cap["config_sha256"],
            "lineage_sha256": rel_cap["lineage_sha256"],
            "checkpoint_file": {"path": ckpt_path.to_string_lossy(), "sha256": ckpt_file_sha},
            "driver_checkpoint_sha256": ckpt_sha,
            "model_snapshot_sha256": value_model_sha,
            "model_epoch": 0,
            "completed_batches": 0,
            "token_cells_file": {"path": u16_path.to_string_lossy(), "sha256": u16_sha},
            "token_count": 4096,
            "byte_count": 8192,
            "encoding": "raw_u16_le_no_header",
            "ordering": "ascending_token_id_0_through4095",
            "cell_range": {"min": 0, "max": 255},
            "absent_cells_allowed": false,
            "model_bindings": {
                "source_sha256": rel_eval["model_bindings"]["source_sha256"],
                "kernel_sha256": rel_eval["model_bindings"]["kernel_sha256"],
                "geometry_sha256": rel_eval["model_bindings"]["geometry_sha256"],
                "spec": rel_eval["model_bindings"]["spec"],
                "read_spec": rel_eval["model_bindings"]["read_spec"],
                "objective_policy": rel_eval["model_bindings"]["objective_policy"],
                "read_initialization": "active_leaf_v1",
                "seed": rel_eval["model_bindings"]["seed"]
            },
            "model_restores": 1,
            "export_kernel_calls": 1,
            "token_cell_evaluations": 4096,
            "gpu_forward_calls": 0,
            "proposals": 0,
            "updates_applied": 0,
            "snapshot_unchanged": true,
            "training_capability_issued": false,
            "host_cell_reconstruction": false,
            "training": "HELD",
            "m2": "HELD",
            "stage_b": "HELD",
            "q4": "NOT_EXECUTABLE"
        });
        let report_path = tmp_dir.join("report.json");
        fs::write(&report_path, serde_json::to_vec(&report_val).unwrap()).unwrap();
        let loaded = load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap();
        assert_eq!(loaded.occupied_cells, 256);
        assert_eq!(loaded.checkpoint_file_sha256, ckpt_file_sha);
        assert_eq!(loaded.driver_checkpoint_sha256, ckpt_sha);
        assert_eq!(loaded.model_storage_sha256, value_model_sha);
        assert_eq!(loaded.model_snapshot_sha256, value_model_sha);
        assert_eq!(loaded.typed_model_snapshot_sha256, typed_snap_sha);

        let arm = validate_arm_inputs(
            &panel,
            sha256_hex(b"p_sha"),
            &rel_cap,
            sha256_hex(b"rc_sha"),
            &rel_eval,
            rel_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            false,
        )
        .unwrap();
        let mut matches_gpu_arm = loaded.clone();
        matches_gpu_arm.token_cells[7] = 0;
        verify_loaded_token_cells_against_arm(&arm, &matches_gpu_arm, "test").unwrap();

        // Post-#2611 (dfa3f4830) dual-record format:
        // report.json, eval, and capture emit model_storage_sha256 = Domain A (value_model_sha)
        // and model_snapshot_sha256 = Domain B (typed_snap_sha), with A != B.
        let mut dual_report = report_val.clone();
        dual_report["model_storage_sha256"] = json!(value_model_sha);
        dual_report["model_snapshot_sha256"] = json!(typed_snap_sha);
        fs::write(&report_path, serde_json::to_vec(&dual_report).unwrap()).unwrap();
        let dual_loaded = load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap();
        assert_eq!(dual_loaded.model_storage_sha256, value_model_sha);
        assert_eq!(dual_loaded.model_snapshot_sha256, value_model_sha);
        assert_eq!(dual_loaded.typed_model_snapshot_sha256, typed_snap_sha);

        let mut dual_eval = rel_eval.clone();
        dual_eval["model_storage_sha256"] = json!(value_model_sha);
        dual_eval["model_snapshot_sha256"] = json!(typed_snap_sha);
        let dual_eval_sha = sha256_hex(&serde_json::to_vec(&dual_eval).unwrap());
        let mut dual_cap = rel_cap.clone();
        dual_cap["model_storage_sha256"] = json!(value_model_sha);
        dual_cap["model_snapshot_sha256"] = json!(typed_snap_sha);
        dual_cap["evaluation_file"]["sha256"] = json!(dual_eval_sha);
        let dual_arm = validate_arm_inputs(
            &panel,
            sha256_hex(b"p_sha"),
            &dual_cap,
            sha256_hex(b"rc_sha"),
            &dual_eval,
            dual_eval_sha.clone(),
            &d34,
            sha256_hex(b"d34_sha"),
            false,
        )
        .unwrap();
        assert_eq!(
            dual_arm.model_storage_sha256.as_deref(),
            Some(value_model_sha.as_str())
        );
        assert_eq!(dual_arm.model_snapshot_sha256, typed_snap_sha);
        let mut dual_matches_gpu_arm = dual_loaded.clone();
        dual_matches_gpu_arm.token_cells[7] = 0;
        verify_loaded_token_cells_against_arm(&dual_arm, &dual_matches_gpu_arm, "test_dual")
            .unwrap();

        // Dual-record report.json that leaves Domain A in model_snapshot_sha256 when model_storage_sha256 is present must be refused.
        let mut bad_dual_report_a = dual_report.clone();
        bad_dual_report_a["model_snapshot_sha256"] = json!(value_model_sha);
        fs::write(
            &report_path,
            serde_json::to_vec(&bad_dual_report_a).unwrap(),
        )
        .unwrap();
        let bad_dual_a_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(
            bad_dual_a_err.contains("typed model snapshot sha256"),
            "{bad_dual_a_err}"
        );

        // Dual-record report.json that puts Domain B in model_storage_sha256 must be refused.
        let mut bad_dual_report_b = dual_report.clone();
        bad_dual_report_b["model_storage_sha256"] = json!(typed_snap_sha);
        fs::write(
            &report_path,
            serde_json::to_vec(&bad_dual_report_b).unwrap(),
        )
        .unwrap();
        let bad_dual_b_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(
            bad_dual_b_err.contains("model storage/value sha256"),
            "{bad_dual_b_err}"
        );

        // Mismatched capture.model_storage_sha256 vs eval.model_storage_sha256 must be refused.
        let mut bad_dual_cap = dual_cap.clone();
        bad_dual_cap["model_storage_sha256"] = json!(typed_snap_sha);
        let bad_cap_storage_err = validate_arm_inputs(
            &panel,
            sha256_hex(b"p_sha"),
            &bad_dual_cap,
            sha256_hex(b"rc_sha"),
            &dual_eval,
            dual_eval_sha,
            &d34,
            sha256_hex(b"d34_sha"),
            false,
        )
        .unwrap_err();
        assert!(
            bad_cap_storage_err.contains("eval.model_storage_sha256"),
            "{bad_cap_storage_err}"
        );

        // Restore valid report_val for remaining checks.
        fs::write(&report_path, serde_json::to_vec(&report_val).unwrap()).unwrap();

        // Tampering with eval.model_snapshot_sha256 (wrong typed snapshot hash) must be refused by load_gpu_token_cells_file.
        let wrong_typed_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &value_model_sha).unwrap_err();
        assert!(
            wrong_typed_err.contains("typed model snapshot sha256"),
            "{wrong_typed_err}"
        );

        // Removing the local checkpoint file without --checkpoint override must be refused.
        fs::remove_file(&ckpt_path).unwrap();
        let missing_local_ckpt_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(
            missing_local_ckpt_err.contains("cannot locate pinned DriverCheckpoint file"),
            "{missing_local_ckpt_err}"
        );

        // Missing checkpoint_file in report.json must be refused.
        let mut missing_ckpt_report = report_val.clone();
        missing_ckpt_report
            .as_object_mut()
            .unwrap()
            .remove("checkpoint_file");
        fs::write(
            &report_path,
            serde_json::to_vec(&missing_ckpt_report).unwrap(),
        )
        .unwrap();
        let missing_err =
            load_gpu_token_cells_file(&u16_path, &ckpt_sha, &typed_snap_sha).unwrap_err();
        assert!(missing_err.contains("checkpoint_file"), "{missing_err}");
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn u_succ_ranked_uses_train_unigram_ranking_not_dev_labels() {
        let tmp_dir =
            std::env::temp_dir().join(format!("uor_r4_usucc_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        // Synthetic TRAIN: BOS(0), 3, 2, 3, 2, 3, 7 -> matched targets at p=2,4,6 are 2, 2, 7.
        // Modal TRAIN target is token 2 (count 2), followed by token 7 (count 1).
        let train_toks: [u16; 7] = [0, 3, 2, 3, 2, 3, 7];
        let train_mask: [u8; 7] = [0, 0, 1, 0, 1, 0, 1];

        // Synthetic DEV: BOS(0), 3, 7, 3, 7 -> eligible row at p=4 has query=3, target=7 (P_succ).
        // If U_succ were fitted on DEV P_succ labels, its top-1 would be 7 instead of TRAIN modal 2.
        let dev_toks: [u16; 5] = [0, 3, 7, 3, 7];
        let dev_mask: [u8; 5] = [0, 0, 0, 0, 1];

        let write_u16 = |path: &Path, slice: &[u16]| {
            let mut b = vec![0u8; 64];
            b[..4].copy_from_slice(b"UORT");
            b[4..8].copy_from_slice(&1u32.to_le_bytes());
            b[8..16].copy_from_slice(&(slice.len() as u64).to_le_bytes());
            b[16..20].copy_from_slice(&(CANONICAL_VOCAB as u32).to_le_bytes());
            for &x in slice {
                b.extend_from_slice(&x.to_le_bytes());
            }
            fs::write(path, b).unwrap();
        };
        let train_tok_p = tmp_dir.join("train.tokens.u16");
        let train_mask_p = tmp_dir.join("train.mask.u8");
        let dev_tok_p = tmp_dir.join("dev.tokens.u16");
        let dev_mask_p = tmp_dir.join("dev.mask.u8");
        write_u16(&train_tok_p, &train_toks);
        fs::write(&train_mask_p, train_mask).unwrap();
        write_u16(&dev_tok_p, &dev_toks);
        fs::write(&dev_mask_p, dev_mask).unwrap();

        let d34_rows = vec![ValidatedD34Row {
            ordinal: 0,
            position: 4,
            query_token: 3,
            target_token: 7,
            is_psucc: true,
            distance_bucket: Some(0),
            conflict: false,
            t1b_prediction: 2,
            t1b_hit1: false,
            t2_prediction: 2,
            t2_hit1: false,
            t2_hit8: true,
            t3_prediction: 2,
            t3_hit1: false,
            u_hit1: false,
            u_hit8: true,
        }];
        let paths = CorpusPaths {
            train_tokens: &train_tok_p,
            train_mask: &train_mask_p,
            dev_tokens: &dev_tok_p,
            dev_mask: &dev_mask_p,
        };
        let tables = build_corpus_reference_tables(&paths, &d34_rows, false).unwrap();
        assert_eq!(tables.train_matched_targets, 3);
        assert_eq!(tables.u_train_total_counts, 3);
        assert_eq!(tables.u_succ_total_counts, 3);
        assert_eq!(tables.u_train_ranked, vec![2, 7]);
        assert_eq!(tables.u_succ_ranked, vec![2, 7]);
        assert_eq!(
            tables.u_succ_ranked_table_sha256,
            tables.u_train_ranked_table_sha256
        );
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
