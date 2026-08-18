//! Observation pipeline v2 (graph-compiler plan §5 Phase 2): content-
//! addressed sample identity, deterministic shard spill/resume, and ordered
//! merge for the cover-induction stages that follow.
//!
//! Determinism contract (plan §4.1, D2):
//!
//! - **Sample ids are content addresses**: blake3 over the little-endian
//!   bytes of the 8-token context window (the same window `runtime`'s
//!   `history_token` bundles: the current input token plus up to seven
//!   predecessors within one story). The same context yields the same id
//!   regardless of when or where it is produced.
//! - **Shard assignment is a pure function of the sample id** — the first
//!   `shard_bits` bits of the id — independent of iteration order, worker
//!   count, and thread count (T-invariance: T=1 and T=N agree).
//! - **Per-shard files are κ-pinned when finalized and merged in ascending
//!   shard-id order**, so shard completion order never changes the merged
//!   observation bytes.
//!
//! Resume is transactional at whole-story boundaries. `raw-committed.bin` is
//! the authoritative, versioned snapshot: it binds the observation identity
//! and row layout to the deterministic teacher state plus one committed record
//! count per shard. Every affected base/probability/trace stream is flushed and
//! synchronized before that snapshot is atomically replaced. `state.bin`
//! remains the historical 25-byte compatibility mirror and is published only
//! after the authoritative snapshot. On resume, every stream is inspected
//! before any is truncated, so complete aligned bytes beyond the committed
//! vector are tentative and are regenerated exactly once. Fully finalized,
//! kappa-validated legacy raw bundles remain readable; an unfinished legacy
//! raw directory has no provable transaction boundary and fails closed.

#[cfg(not(target_arch = "wasm32"))]
use crate::trace_profile::SUPPORT_ABSENT_MARKER;
use crate::trace_profile::TraceProfile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, BufWriter, Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::hf_bpe::adapter_constructor;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::TeacherOracle;
use uor_r4_model_source::attention::AttentionOperatorSpec;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::attention::operator_spec;
use uor_r4_model_source::dense::DenseOperatorSpec;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::dense::operator_spec as dense_operator_spec;
use uor_r4_model_source::geometry::GeometryProjection;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::geometry::projection_implementation;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::progress::Progress;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::{TraceCaptureRequest, TraceCaptureSinks};

/// Observation record width: the v4 corpus record layout (story, next,
/// top-8 tokens, top-8 weights, span, byte anchors) — see
/// [`compiler::encode_v4_record`].
pub const RECORD_SIZE: usize = 88;

/// Width of one probability sidecar row. The row is aligned with the
/// corresponding record in each shard, but is kept separate so the existing
/// 88-byte observation ABI and old fixtures remain readable.
pub const PROBABILITY_METADATA_SIZE: usize = 16;

/// Compiler-side probability metadata for one observation row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbabilityMetadata {
    pub target_logprob_nats: f32,
    pub entropy_bits: f32,
    pub top8_mass: f32,
    pub target_rank: u16,
}

impl ProbabilityMetadata {
    pub fn encode(self) -> [u8; PROBABILITY_METADATA_SIZE] {
        let mut bytes = [0u8; PROBABILITY_METADATA_SIZE];
        bytes[0..4].copy_from_slice(&self.target_logprob_nats.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.entropy_bits.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.top8_mass.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.target_rank.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PROBABILITY_METADATA_SIZE {
            return None;
        }
        Some(Self {
            target_logprob_nats: f32::from_le_bytes(bytes[0..4].try_into().ok()?),
            entropy_bits: f32::from_le_bytes(bytes[4..8].try_into().ok()?),
            top8_mass: f32::from_le_bytes(bytes[8..12].try_into().ok()?),
            target_rank: u16::from_le_bytes(bytes[12..14].try_into().ok()?),
        })
    }
}

/// Return the information content of a message in bits without multiplying
/// tiny probabilities. A message probability is the product of its
/// conditional token probabilities, so its log-domain information is their
/// additive sum.
pub fn message_information_bits(metadata: &[ProbabilityMetadata]) -> f64 {
    metadata
        .iter()
        .map(|row| -f64::from(row.target_logprob_nats) / f64::from(std::f32::consts::LN_2))
        .sum()
}

/// Return average message information in bits per recorded token.
pub fn message_bits_per_token(metadata: &[ProbabilityMetadata]) -> Option<f64> {
    if metadata.is_empty()
        || metadata
            .iter()
            .any(|row| !row.target_logprob_nats.is_finite())
    {
        return None;
    }
    Some(message_information_bits(metadata) / metadata.len() as f64)
}

/// Maximum shard fan-out accepted by [`ObservationShardWriter`]: shard
/// files are held open during a pass, so the writer caps the fan-out at
/// 2^8. [`shard_of`] itself is defined for up to 32 bits.
pub const MAX_SHARD_BITS: u8 = 8;

/// Manifest file name within an observation directory.
pub const MANIFEST_FILE: &str = "manifest.json";

/// Permanent, empty coordination inode used only for an OS-backed exclusive
/// writer lock. It is not observation payload and is never removed as a lease,
/// so process death releases the lock without leaving a stale-lock state.
#[cfg(not(target_arch = "wasm32"))]
const OBSERVATION_SESSION_COORDINATION_DIR: &str = ".uor-r4-observation-sessions";

#[cfg(not(target_arch = "wasm32"))]
const OBSERVATION_PAYLOAD_FILES: [&str; 8] = [
    STATE_FILE,
    RAW_COMMITTED_FILE,
    "merged.bin",
    "committed.bin",
    ".committed.bin.tmp",
    "stories.jsonl",
    "stories.tmp",
    "tokenizer.bin",
];

#[cfg(not(target_arch = "wasm32"))]
static MANIFEST_TEMP_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(not(target_arch = "wasm32"))]
static RAW_CHECKPOINT_TEMP_SEQUENCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Generator checkpoint file name within an observation directory.
pub const STATE_FILE: &str = "state.bin";

/// Authoritative raw-observation transaction snapshot. This is deliberately
/// distinct from the text driver's `committed.bin`; neither format is accepted
/// by the other driver.
pub const RAW_COMMITTED_FILE: &str = "raw-committed.bin";

#[cfg(not(target_arch = "wasm32"))]
const RAW_COMMITTED_MAGIC: [u8; 8] = *b"R4RAWCOM";
#[cfg(not(target_arch = "wasm32"))]
const RAW_COMMITTED_SCHEMA: u16 = 1;
#[cfg(not(target_arch = "wasm32"))]
const RAW_COMMITTED_HEADER_BYTES: usize = 88;

#[cfg(not(target_arch = "wasm32"))]
type ObservationState = (u64, u64, u64, u8);

/// Canonical raw-observation transaction snapshot.
///
/// The encoded representation is exactly `88 + 8 * shard_count` bytes. It is
/// timestamp-free and stores record counts rather than redundant companion
/// lengths; probability and trace lengths are derived with checked arithmetic
/// from the pinned row widths.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCommittedCheckpoint {
    shard_bits: u8,
    trace_row_bytes: Option<u64>,
    n: u64,
    stories: u64,
    rng: u64,
    done: bool,
    identity_digest: [u8; 32],
    committed_records: Vec<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
impl RawCommittedCheckpoint {
    /// Construct and validate a checkpoint against the manifest identity and
    /// layout that it commits.
    pub fn new(
        manifest: &ObservationManifest,
        n: u64,
        stories: u64,
        rng: u64,
        done: bool,
        committed_records: Vec<u64>,
    ) -> Result<Self, SourceUnavailable> {
        manifest.validate_loaded_semantics()?;
        let checkpoint = Self {
            shard_bits: manifest.shard_bits,
            trace_row_bytes: manifest.trace_row_bytes,
            n,
            stories,
            rng,
            done,
            identity_digest: *blake3::hash(&manifest.identity_bundle_bytes()).as_bytes(),
            committed_records,
        };
        checkpoint.validate_intrinsic()?;
        checkpoint.validate_manifest(manifest)?;
        Ok(checkpoint)
    }

    /// Decode the exact canonical binary representation. Compatibility with a
    /// particular manifest is checked separately by the locked resume path.
    pub fn decode(bytes: &[u8]) -> Result<Self, SourceUnavailable> {
        if bytes.len() < RAW_COMMITTED_HEADER_BYTES {
            return Err(invalid_data(format!(
                "raw observation checkpoint has {} bytes, shorter than the {RAW_COMMITTED_HEADER_BYTES}-byte header",
                bytes.len()
            )));
        }
        if bytes[0..8] != RAW_COMMITTED_MAGIC {
            return Err(invalid_data(
                "raw observation checkpoint has invalid magic; refusing mixed or corrupt checkpoint format"
                    .to_owned(),
            ));
        }
        let schema = u16::from_le_bytes(bytes[8..10].try_into().expect("2-byte slice"));
        if schema != RAW_COMMITTED_SCHEMA {
            return Err(invalid_data(format!(
                "unsupported raw observation checkpoint schema {schema}; expected {RAW_COMMITTED_SCHEMA}"
            )));
        }
        let shard_bits = bytes[10];
        let done = match bytes[11] {
            0 => false,
            1 => true,
            value => {
                return Err(invalid_data(format!(
                    "raw observation checkpoint has invalid done byte {value}; expected 0 or 1"
                )));
            }
        };
        let record_size = u32::from_le_bytes(bytes[12..16].try_into().expect("4-byte slice"));
        if record_size != RECORD_SIZE as u32 {
            return Err(invalid_data(format!(
                "raw observation checkpoint records {record_size}-byte base rows, expected {RECORD_SIZE}"
            )));
        }
        let probability_size = u32::from_le_bytes(bytes[16..20].try_into().expect("4-byte slice"));
        if probability_size != PROBABILITY_METADATA_SIZE as u32 {
            return Err(invalid_data(format!(
                "raw observation checkpoint records {probability_size}-byte probability rows, expected {PROBABILITY_METADATA_SIZE}"
            )));
        }
        let trace_width = u64::from_le_bytes(bytes[20..28].try_into().expect("8-byte slice"));
        let trace_row_bytes = (trace_width != 0).then_some(trace_width);
        let n = u64::from_le_bytes(bytes[28..36].try_into().expect("8-byte slice"));
        let stories = u64::from_le_bytes(bytes[36..44].try_into().expect("8-byte slice"));
        let rng = u64::from_le_bytes(bytes[44..52].try_into().expect("8-byte slice"));
        let identity_digest = bytes[52..84].try_into().expect("32-byte slice");
        let encoded_shards = u32::from_le_bytes(bytes[84..88].try_into().expect("4-byte slice"));
        let expected_shards = 1u32.checked_shl(u32::from(shard_bits)).ok_or_else(|| {
            invalid_data(format!(
                "raw observation checkpoint shard_bits {shard_bits} cannot define a bounded fan-out"
            ))
        })?;
        if shard_bits > MAX_SHARD_BITS || encoded_shards != expected_shards {
            return Err(invalid_data(format!(
                "raw observation checkpoint fan-out {encoded_shards} does not match shard_bits {shard_bits} (maximum {MAX_SHARD_BITS})"
            )));
        }
        let vector_bytes = usize::try_from(encoded_shards)
            .ok()
            .and_then(|count| count.checked_mul(8))
            .and_then(|length| RAW_COMMITTED_HEADER_BYTES.checked_add(length))
            .ok_or_else(|| invalid_data("raw checkpoint length overflows usize".to_owned()))?;
        if bytes.len() != vector_bytes {
            return Err(invalid_data(format!(
                "raw observation checkpoint has {} bytes, expected exactly {vector_bytes} for {encoded_shards} shards",
                bytes.len()
            )));
        }
        let mut committed_records = Vec::with_capacity(encoded_shards as usize);
        for row in bytes[RAW_COMMITTED_HEADER_BYTES..].chunks_exact(8) {
            committed_records.push(u64::from_le_bytes(row.try_into().expect("8-byte slice")));
        }
        let checkpoint = Self {
            shard_bits,
            trace_row_bytes,
            n,
            stories,
            rng,
            done,
            identity_digest,
            committed_records,
        };
        checkpoint.validate_intrinsic()?;
        Ok(checkpoint)
    }

    /// Encode the canonical binary representation.
    pub fn encode(&self) -> Result<Vec<u8>, SourceUnavailable> {
        self.validate_intrinsic()?;
        let capacity = RAW_COMMITTED_HEADER_BYTES
            .checked_add(self.committed_records.len().checked_mul(8).ok_or_else(|| {
                invalid_data("raw checkpoint vector length overflows usize".to_owned())
            })?)
            .ok_or_else(|| invalid_data("raw checkpoint length overflows usize".to_owned()))?;
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(&RAW_COMMITTED_MAGIC);
        bytes.extend_from_slice(&RAW_COMMITTED_SCHEMA.to_le_bytes());
        bytes.push(self.shard_bits);
        bytes.push(u8::from(self.done));
        bytes.extend_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
        bytes.extend_from_slice(&(PROBABILITY_METADATA_SIZE as u32).to_le_bytes());
        bytes.extend_from_slice(&self.trace_row_bytes.unwrap_or(0).to_le_bytes());
        bytes.extend_from_slice(&self.n.to_le_bytes());
        bytes.extend_from_slice(&self.stories.to_le_bytes());
        bytes.extend_from_slice(&self.rng.to_le_bytes());
        bytes.extend_from_slice(&self.identity_digest);
        bytes.extend_from_slice(&(self.committed_records.len() as u32).to_le_bytes());
        for records in &self.committed_records {
            bytes.extend_from_slice(&records.to_le_bytes());
        }
        debug_assert_eq!(bytes.len(), capacity);
        Ok(bytes)
    }

    /// Global teacher state selected by this transaction.
    pub fn state(&self) -> (u64, u64, u64, bool) {
        (self.n, self.stories, self.rng, self.done)
    }

    /// Per-shard committed record counts in canonical shard order.
    pub fn committed_records(&self) -> &[u64] {
        &self.committed_records
    }

    /// Pinned shard fan-out exponent.
    pub fn shard_bits(&self) -> u8 {
        self.shard_bits
    }

    /// Pinned trace row width, absent for a minimal raw pass.
    pub fn trace_row_bytes(&self) -> Option<u64> {
        self.trace_row_bytes
    }

    fn validate_intrinsic(&self) -> Result<(), SourceUnavailable> {
        if self.shard_bits > MAX_SHARD_BITS {
            return Err(invalid_data(format!(
                "raw observation checkpoint shard_bits {} exceeds maximum {MAX_SHARD_BITS}",
                self.shard_bits
            )));
        }
        let shard_count = 1usize << self.shard_bits;
        if self.committed_records.len() != shard_count {
            return Err(invalid_data(format!(
                "raw observation checkpoint has {} shard counters, expected {shard_count}",
                self.committed_records.len()
            )));
        }
        if self.trace_row_bytes == Some(0) {
            return Err(invalid_data(
                "raw observation checkpoint records a zero-byte trace row".to_owned(),
            ));
        }
        if self.stories > u32::MAX as u64 {
            return Err(invalid_data(format!(
                "raw observation checkpoint records {} stories, outside the u32 story wire domain",
                self.stories
            )));
        }
        let total = self.committed_records.iter().try_fold(0u64, |sum, count| {
            count.checked_mul(RECORD_SIZE as u64).ok_or_else(|| {
                invalid_data("raw checkpoint base byte length overflows u64".to_owned())
            })?;
            count
                .checked_mul(PROBABILITY_METADATA_SIZE as u64)
                .ok_or_else(|| {
                    invalid_data("raw checkpoint probability byte length overflows u64".to_owned())
                })?;
            if let Some(row_bytes) = self.trace_row_bytes {
                count.checked_mul(row_bytes).ok_or_else(|| {
                    invalid_data("raw checkpoint trace byte length overflows u64".to_owned())
                })?;
            }
            sum.checked_add(*count).ok_or_else(|| {
                invalid_data("raw observation checkpoint record total overflows u64".to_owned())
            })
        })?;
        if total != self.n {
            return Err(invalid_data(format!(
                "raw observation checkpoint n={} does not match per-shard record total {total}",
                self.n
            )));
        }
        Ok(())
    }

    fn validate_manifest(&self, manifest: &ObservationManifest) -> Result<(), SourceUnavailable> {
        if self.shard_bits != manifest.shard_bits
            || self.committed_records.len() != manifest.shard_count() as usize
        {
            return Err(invalid_data(format!(
                "raw observation checkpoint fan-out does not match manifest shard_bits {}",
                manifest.shard_bits
            )));
        }
        if self.trace_row_bytes != manifest.trace_row_bytes {
            return Err(invalid_data(format!(
                "raw observation checkpoint trace row width {:?} does not match manifest {:?}",
                self.trace_row_bytes, manifest.trace_row_bytes
            )));
        }
        let expected_identity = *blake3::hash(&manifest.identity_bundle_bytes()).as_bytes();
        if self.identity_digest != expected_identity {
            return Err(invalid_data(format!(
                "raw observation checkpoint identity does not match manifest bundle {}",
                manifest.identity_bundle_digest()
            )));
        }
        for (&shard, entry) in &manifest.completed {
            if self.committed_records[shard as usize] != entry.records {
                return Err(invalid_data(format!(
                    "raw observation checkpoint commits {} records for finalized shard {shard}, but the manifest commits {}",
                    self.committed_records[shard as usize], entry.records
                )));
            }
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_observation_state(dir: &Path) -> Result<Option<ObservationState>, SourceUnavailable> {
    let path = dir.join(STATE_FILE);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(invalid_input(format!(
                "observation checkpoint {} is not a regular file",
                path.display()
            )));
        }
        Ok(_) => {}
    }
    let bytes = fs::read(&path)?;
    if bytes.len() != 25 {
        return Err(invalid_data(format!(
            "observation checkpoint {} has {} bytes, expected exactly 25",
            path.display(),
            bytes.len()
        )));
    }
    let done = bytes[24];
    if done > 1 {
        return Err(invalid_data(format!(
            "observation checkpoint {} has invalid done byte {done}; expected 0 or 1",
            path.display()
        )));
    }
    let stories = u64::from_le_bytes(bytes[8..16].try_into().expect("8-byte slice"));
    if stories > u32::MAX as u64 {
        return Err(invalid_data(format!(
            "observation checkpoint {} records {stories} stories, outside the u32 story wire domain",
            path.display()
        )));
    }
    Ok(Some((
        u64::from_le_bytes(bytes[0..8].try_into().expect("8-byte slice")),
        stories,
        u64::from_le_bytes(bytes[16..24].try_into().expect("8-byte slice")),
        done,
    )))
}

#[cfg(not(target_arch = "wasm32"))]
fn encode_observation_state(state: ObservationState) -> [u8; 25] {
    let (n, stories, rng, done) = state;
    let mut bytes = [0u8; 25];
    bytes[0..8].copy_from_slice(&n.to_le_bytes());
    bytes[8..16].copy_from_slice(&stories.to_le_bytes());
    bytes[16..24].copy_from_slice(&rng.to_le_bytes());
    bytes[24] = done;
    bytes
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_directory(dir: &Path) -> Result<(), SourceUnavailable> {
    fs::File::open(dir)?.sync_all()?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn atomic_publish_observation_file(
    dir: &Path,
    destination_name: &str,
    temporary_stem: &str,
    bytes: &[u8],
) -> Result<(), SourceUnavailable> {
    let destination = dir.join(destination_name);
    match fs::symlink_metadata(&destination) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(invalid_input(format!(
                "observation transaction file {} is not a regular file; refusing atomic replacement",
                destination.display()
            )));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    for _ in 0..64 {
        let sequence =
            RAW_CHECKPOINT_TEMP_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temporary = dir.join(format!(
            ".{temporary_stem}.tmp-{}-{sequence}",
            std::process::id()
        ));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        };
        let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        if let Err(error) = fs::rename(&temporary, &destination) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        sync_directory(dir)?;
        return Ok(());
    }
    Err(invalid_input(format!(
        "could not reserve a unique {destination_name} temporary in {}",
        dir.display()
    )))
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_raw_checkpoint(
    dir: &Path,
    checkpoint: &RawCommittedCheckpoint,
) -> Result<(), SourceUnavailable> {
    atomic_publish_observation_file(
        dir,
        RAW_COMMITTED_FILE,
        RAW_COMMITTED_FILE,
        &checkpoint.encode()?,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_state_mirror(
    dir: &Path,
    checkpoint: &RawCommittedCheckpoint,
) -> Result<(), SourceUnavailable> {
    let (n, stories, rng, done) = checkpoint.state();
    atomic_publish_observation_file(
        dir,
        STATE_FILE,
        STATE_FILE,
        &encode_observation_state((n, stories, rng, u8::from(done))),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn read_raw_checkpoint(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<Option<RawCommittedCheckpoint>, SourceUnavailable> {
    let path = dir.join(RAW_COMMITTED_FILE);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(invalid_input(format!(
                "raw observation checkpoint {} is not a regular file",
                path.display()
            )));
        }
        Ok(_) => {}
    }
    let checkpoint = RawCommittedCheckpoint::decode(&fs::read(path)?)?;
    checkpoint.validate_manifest(manifest)?;
    Ok(Some(checkpoint))
}

#[cfg(not(target_arch = "wasm32"))]
fn is_canonical_blake3_address(value: &str) -> bool {
    value.len() == "blake3:".len() + 64
        && value.starts_with("blake3:")
        && value["blake3:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Validate that an attention provenance record is exactly one immutable
/// entry from the source-operator registry. Naming a registered `(id,
/// version)` is insufficient: every declared field and the implementation
/// digest must agree with the registry's canonical record.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate_registered_source_attention_operator(
    operator: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    let registered = operator_spec(&operator.id, operator.version)?;
    if registered != *operator {
        return Err(invalid_input(format!(
            "attention operator {}/{} diverges from its immutable registry record; refusing provenance before mutation",
            operator.id, operator.version
        )));
    }
    let is_source_operator = operator.id == AttentionOperatorSpec::STANDARD_ID
        || operator.id == AttentionOperatorSpec::EXPERIMENTAL_R4_ID
        || operator.id == AttentionOperatorSpec::LEARNED_ABSOLUTE_ID;
    if !is_source_operator {
        return Err(invalid_input(format!(
            "registered attention operator {}/{} is not an allowed source teacher operator; refusing source provenance before mutation",
            operator.id, operator.version
        )));
    }
    Ok(())
}

/// Validate that a dense-execution provenance record is exactly one immutable
/// entry from the model-source registry. Dense records describe host-side
/// teacher execution; they are never deployed runtime operators.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate_registered_source_dense_operator(
    operator: &DenseOperatorSpec,
) -> Result<(), SourceUnavailable> {
    let registered = dense_operator_spec(&operator.id, operator.version)?;
    if registered != *operator {
        return Err(invalid_input(format!(
            "dense operator {}/{} diverges from its immutable registry record; refusing provenance before mutation",
            operator.id, operator.version
        )));
    }
    Ok(())
}

/// Validate the composite host-execution identity, after each present record
/// has independently matched its immutable registry entry. Dense provenance
/// is GPT-2-specific: it is meaningless without learned-absolute attention,
/// and an explicit dense era must agree with the corresponding attention era.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate_source_execution_identity(
    attention: Option<&AttentionOperatorSpec>,
    dense: Option<&DenseOperatorSpec>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    uor_r4_model_source::dense::validate_source_execution_pair(attention, dense).map_err(
        |mut error| {
            error.reason = format!("{context}: {}", error.reason);
            error
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_registered_tokenizer_adapter(
    adapter: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    adapter_constructor(&adapter.family, adapter.version)?;
    for (field, value) in [
        ("tokenizer_cid", adapter.tokenizer_cid.as_str()),
        ("adapter_digest", adapter.adapter_digest.as_str()),
    ] {
        if !is_canonical_blake3_address(value) {
            return Err(invalid_input(format!(
                "tokenizer adapter {}/{} has non-canonical {field} {value}; expected lowercase blake3:<64 hex>; refusing provenance before mutation",
                adapter.family, adapter.version,
            )));
        }
    }
    let declared_digest = adapter.declared_digest();
    if adapter.adapter_digest != declared_digest {
        return Err(invalid_input(format!(
            "tokenizer adapter {}/{} claims digest {}, but its canonical fields declare {}; refusing inconsistent provenance before mutation",
            adapter.family, adapter.version, adapter.adapter_digest, declared_digest,
        )));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate_registered_geometry_projection(
    geometry: &GeometryProjection,
) -> Result<(), SourceUnavailable> {
    projection_implementation(&geometry.id, geometry.version)?;
    let registered =
        GeometryProjection::bucket_average(geometry.source_width, geometry.compiled_width);
    if registered != *geometry
        || geometry.compiled_width == 0
        || geometry.source_width < geometry.compiled_width
    {
        return Err(invalid_input(format!(
            "geometry projection {}/{} diverges from its immutable registry record for {} -> {}; refusing provenance before mutation",
            geometry.id, geometry.version, geometry.source_width, geometry.compiled_width
        )));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_registered_trace_profile(profile: &TraceProfile) -> Result<(), SourceUnavailable> {
    let layer_indices = profile
        .layer_lane
        .as_ref()
        .map(|lane| lane.layer_indices.clone())
        .or_else(|| {
            profile
                .qkv_lane
                .as_ref()
                .map(|lane| lane.layer_indices.clone())
        })
        .or_else(|| {
            profile
                .attention_support_lane
                .as_ref()
                .map(|lane| lane.layer_indices.clone())
        })
        .unwrap_or_default();
    let support_size = profile
        .attention_support_lane
        .as_ref()
        .map_or(crate::trace_profile::PRIMARY_TOP_K, |lane| {
            lane.support_size
        });
    let registered = crate::trace_profile::profile_spec(
        &profile.id,
        profile.version,
        &crate::trace_profile::TraceCaptureBounds {
            layer_indices,
            support_size,
        },
    )?;
    if registered != *profile || profile.is_minimal() {
        return Err(invalid_input(format!(
            "trace profile {}/{} diverges from its immutable non-minimal registry record; refusing provenance before mutation",
            profile.id, profile.version
        )));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_registered_trace_request(profile: &TraceProfile) -> Result<(), SourceUnavailable> {
    if profile.is_minimal() {
        if *profile == TraceProfile::minimal() {
            Ok(())
        } else {
            Err(invalid_input(format!(
                "trace profile {}/{} diverges from the immutable minimal registry record; refusing provenance before mutation",
                profile.id, profile.version
            )))
        }
    } else {
        validate_registered_trace_profile(profile)
    }
}

/// Content address of one observation sample: blake3 over the
/// little-endian token bytes of the context window.
pub fn sample_id(tokens: &[u32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for token in tokens {
        hasher.update(&token.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Deterministic shard partition: the first `shard_bits` bits of the
/// sample id, big-endian bit order starting at byte 0 (bit 0 of the shard
/// index is the most significant bit of `id[0]`). Independent of iteration
/// order and worker count; the same id always maps to the same shard.
pub fn shard_of(id: &[u8; 32], shard_bits: u8) -> u32 {
    assert!(
        shard_bits <= 32,
        "shard_bits exceeds the 32-bit shard index"
    );
    let mut shard = 0u32;
    for bit in 0..shard_bits as usize {
        let set = u32::from(id[bit / 8] >> (7 - (bit % 8)) & 1);
        shard = (shard << 1) | set;
    }
    shard
}

/// Name of one shard file: `shard-NN.bin`, zero-padded so lexicographic
/// order matches shard-id order for the configured fan-out.
pub fn shard_file_name(shard_bits: u8, shard: u32) -> String {
    let max_shard = if shard_bits >= 32 {
        u32::MAX
    } else {
        (1u32 << shard_bits) - 1
    };
    let width = max_shard.to_string().len().max(2);
    format!("shard-{shard:0width$}.bin")
}

/// Filename-only discriminator for payload/checkpoint evidence owned
/// exclusively by an observation session. Compile-style corpus producers use
/// this at their shared lower preflight so a torn or stripped observation
/// root cannot be mistaken for a fresh compile destination.
pub(crate) fn is_observation_exclusive_entry_name(name: &str) -> bool {
    matches!(
        name,
        MANIFEST_FILE
            | STATE_FILE
            | RAW_COMMITTED_FILE
            | "merged.bin"
            | "committed.bin"
            | ".committed.bin.tmp"
            | "stories.jsonl"
            | "stories.tmp"
    ) || name.starts_with("shard-")
        || name.starts_with("stories.tmp-")
        || name.starts_with(".manifest.json.tmp")
        || name.starts_with(".raw-committed.bin.tmp")
        || name.starts_with(".state.bin.tmp")
        || name.starts_with(".committed.bin.tmp")
}

/// R5: the host-ingestion boundary reports exactly one condition — a declared
/// external source or sink could not be ingested or persisted. These helpers
/// keep the observed malformation as the reason.
#[cfg(not(target_arch = "wasm32"))]
fn invalid_input(message: String) -> SourceUnavailable {
    SourceUnavailable::new(message)
}

#[cfg(not(target_arch = "wasm32"))]
fn invalid_data(message: String) -> SourceUnavailable {
    SourceUnavailable::new(message)
}

#[cfg(not(target_arch = "wasm32"))]
fn file_kappa(path: &Path) -> Result<String, SourceUnavailable> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

/// Document-level partition of one observation record, recorded per shard
/// so downstream consumers can split a merged observation corpus exactly
/// (the from-text driver of `super::observe_text` tags every record with
/// its article's partition at write time).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordPartition {
    Construction,
    HeldOut,
}

/// Per-partition record counts of one shard.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionCounts {
    pub construction: u64,
    pub held_out: u64,
}

impl PartitionCounts {
    #[cfg(not(target_arch = "wasm32"))]
    fn add(&mut self, partition: RecordPartition) {
        match partition {
            RecordPartition::Construction => self.construction += 1,
            RecordPartition::HeldOut => self.held_out += 1,
        }
    }

    /// Total records across both partitions.
    pub fn total(&self) -> u64 {
        self.construction + self.held_out
    }
}

/// One completed shard's entry in the observation manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardEntry {
    /// Number of observation records in the shard file.
    pub records: u64,
    /// Content κ of the shard file bytes.
    pub kappa: String,
    /// Per-partition record counts, when the producing pipeline tags
    /// records with a document-level partition (absent for the generation
    /// path, so its manifest bytes are unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partitions: Option<PartitionCounts>,
    /// Content κ of the aligned probability sidecar, when probability
    /// metadata was captured for this shard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probability_kappa: Option<String>,
    /// Content κ of the aligned #603 teacher-trace sidecar
    /// (`<shard>.trace`), when a non-minimal trace profile captured
    /// richer lanes for this shard. Absent for the minimal profile, so
    /// legacy manifest bytes are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_kappa: Option<String>,
}

/// Manifest of an observation shard directory: the fan-out, the completed
/// shards with their content κ, and the total record count across
/// completed shards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationManifest {
    pub schema: u32,
    pub shard_bits: u8,
    /// The document-level partition rule records are tagged with, when the
    /// producing pipeline has one (from-text driver; absent otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partition_rule: Option<String>,
    /// CID of the exact input the observations were derived from (the
    /// from-text driver records the articles-file κ, i.e. the corpus CID
    /// of the D3 manifest; absent for teacher-generated streams).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_cid: Option<String>,
    /// Root κ of the #597 source-snapshot manifest
    /// (`source_manifest.json`) of the teacher source the observations
    /// were generated from, when the producing pipeline knows it.
    /// Optional with a serde default so every legacy manifest stays
    /// readable and legacy manifest bytes are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_manifest_kappa: Option<String>,
    /// #600 typed record of the source→compiled geometry projection the
    /// teacher oracle applied while producing these observations (e.g.
    /// `bucket-average/1`, 576→288 for the pinned SmolLM2-135M), when the
    /// producing pipeline's oracle declares one. Optional with a serde
    /// default so every legacy manifest stays readable and legacy
    /// manifest bytes are unchanged when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryProjection>,
    /// #601 typed record of the versioned tokenizer adapter the
    /// producing pipeline segmented these observations with (family,
    /// version, tokenizer CID, encode/decode policy, adapter digest),
    /// when the pipeline's tokenizer declares one (the HF byte-level
    /// BPE path; the legacy llama2.c tokenizer declares none).
    /// Optional with a serde default so every legacy manifest stays
    /// readable and legacy manifest bytes are unchanged when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer_adapter: Option<TokenizerAdapter>,
    /// #602 typed record of the source attention operator the teacher
    /// oracle computed while producing these observations. Current Llama
    /// sources declare `standard-source-attention/2` or
    /// `experimental-r4-source-attention/2`; GPT-2 declares
    /// `learned-absolute-source-attention/2`. Immutable v1 records remain
    /// accepted. `None` marks the implicit historical standard/1
    /// interpretation documented in `docs/MODEL_LIFECYCLE.md`. Optional with
    /// a serde default so every legacy manifest stays readable and legacy
    /// manifest bytes are unchanged when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention_operator: Option<AttentionOperatorSpec>,
    /// Typed record of the host-side dense arithmetic the teacher executed.
    /// GPT-2 current sources declare `gpt2-source-dense/2`; Llama and legacy
    /// corpora declare none. Absence stays typed absence rather than being
    /// guessed as a GPT-2 historical version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dense_operator: Option<DenseOperatorSpec>,
    /// #603 typed record of the teacher-trace profile the producing pass
    /// captured under (which lanes, which declared layer indices, which
    /// caps), when a non-minimal profile was active. `None` marks the
    /// minimal profile — exactly today's surface, the implicit legacy
    /// era — so every legacy manifest stays readable and legacy manifest
    /// bytes are unchanged when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_profile: Option<TraceProfile>,
    /// Width in bytes of one #603 trace-sidecar row for this directory
    /// (a pure function of the trace profile's declared bounds and the
    /// oracle's capture geometry, pinned at the first traced write so
    /// resume and merge validate alignment). Absent for the minimal
    /// profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_row_bytes: Option<u64>,
    #[serde(default)]
    pub completed: BTreeMap<u32, ShardEntry>,
    #[serde(default)]
    pub total_records: u64,
}

impl ObservationManifest {
    pub fn new(shard_bits: u8) -> Self {
        Self {
            schema: 1,
            shard_bits,
            partition_rule: None,
            input_cid: None,
            source_manifest_kappa: None,
            geometry: None,
            tokenizer_adapter: None,
            attention_operator: None,
            dense_operator: None,
            trace_profile: None,
            trace_row_bytes: None,
            completed: BTreeMap::new(),
            total_records: 0,
        }
    }

    /// Canonical serialization of the manifest's identity BUNDLE (#603):
    /// the seven identity fields (`input_cid`, `source_manifest_kappa`,
    /// `geometry` #600, `tokenizer_adapter` #601, `attention_operator`
    /// #602, optional `dense_operator` #704, and the #603 `trace_profile`),
    /// in that fixed order, one line per component. The legacy `/1` stream
    /// omits the later dense component entirely; `/2` is selected only when
    /// dense provenance is present. Absence is part of the digest input as an
    /// EXPLICIT marker: an unset component serializes as
    /// `<name>=absent`, a set component as `<name>=present:<value>`
    /// (the raw string for the κ/CID fields, the declared-identity
    /// digest recomputed from the typed records) — so an absent field is
    /// never confusable with an empty or zero one, and the digest is a
    /// pure function of the field values regardless of the order they
    /// were set in.
    pub fn identity_bundle_bytes(&self) -> Vec<u8> {
        fn line(text: &mut String, name: &str, value: Option<String>) {
            match value {
                None => text.push_str(&format!("{name}=absent\n")),
                Some(value) => text.push_str(&format!("{name}=present:{value}\n")),
            }
        }
        // The legacy `/1` byte stream is immutable. Merely teaching this
        // manifest about an optional dense identity must not change any old
        // checkpoint digest. A present dense record alone selects `/2`.
        let mut text = if self.dense_operator.is_some() {
            String::from("uor-r4-observation-identity-bundle/2\n")
        } else {
            String::from("uor-r4-observation-identity-bundle/1\n")
        };
        line(&mut text, "input_cid", self.input_cid.clone());
        line(
            &mut text,
            "source_manifest_kappa",
            self.source_manifest_kappa.clone(),
        );
        line(
            &mut text,
            "geometry",
            self.geometry
                .as_ref()
                .map(|record| record.declared_digest()),
        );
        line(
            &mut text,
            "tokenizer_adapter",
            self.tokenizer_adapter
                .as_ref()
                .map(|record| record.declared_digest()),
        );
        line(
            &mut text,
            "attention_operator",
            self.attention_operator
                .as_ref()
                .map(|record| record.declared_digest()),
        );
        if self.dense_operator.is_some() {
            line(
                &mut text,
                "dense_operator",
                self.dense_operator
                    .as_ref()
                    .map(|record| record.declared_digest()),
            );
        }
        line(
            &mut text,
            "trace_profile",
            self.trace_profile
                .as_ref()
                .map(|record| record.declared_digest()),
        );
        text.into_bytes()
    }

    /// The identity-bundle digest: `blake3:<hex>` over
    /// [`ObservationManifest::identity_bundle_bytes`]. This is the ONE
    /// stable bundle identity of an observation corpus's provenance
    /// seam: it moves when any component changes, is independent of the
    /// order components were recorded in, and distinguishes an absent
    /// component from an empty one.
    pub fn identity_bundle_digest(&self) -> String {
        format!(
            "blake3:{}",
            blake3::hash(&self.identity_bundle_bytes()).to_hex()
        )
    }

    /// Number of shards in the configured fan-out (2^shard_bits).
    pub fn shard_count(&self) -> u32 {
        1u32 << self.shard_bits.min(31)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn validate_loaded_semantics(&self) -> Result<(), SourceUnavailable> {
        if self.schema != 1 {
            return Err(invalid_data(format!(
                "unsupported observation manifest schema {}; expected 1",
                self.schema
            )));
        }
        if self.shard_bits > MAX_SHARD_BITS {
            return Err(invalid_data(format!(
                "observation manifest shard_bits {} exceeds the maximum {MAX_SHARD_BITS}",
                self.shard_bits
            )));
        }
        if self.trace_profile.is_none() && self.trace_row_bytes.is_some() {
            return Err(invalid_data(
                "observation manifest records a trace row width without a trace profile".to_owned(),
            ));
        }
        if self.trace_row_bytes == Some(0) {
            return Err(invalid_data(
                "observation manifest records a zero-byte trace row width".to_owned(),
            ));
        }
        if let Some(kappa) = self.source_manifest_kappa.as_deref()
            && !is_canonical_blake3_address(kappa)
        {
            return Err(invalid_data(format!(
                "observation manifest has non-canonical source manifest κ {kappa}; expected lowercase blake3:<64 hex>"
            )));
        }
        if let Some(cid) = self.input_cid.as_deref()
            && !is_canonical_blake3_address(cid)
        {
            return Err(invalid_data(format!(
                "observation manifest has non-canonical input CID {cid}; expected lowercase blake3:<64 hex>"
            )));
        }

        let shard_count = self.shard_count();
        let mut total_records = 0u64;
        for (&shard, entry) in &self.completed {
            if shard >= shard_count {
                return Err(invalid_data(format!(
                    "observation manifest completes shard {shard}, outside its {shard_count}-shard fan-out"
                )));
            }
            for (field, value) in [
                ("kappa", Some(entry.kappa.as_str())),
                ("probability_kappa", entry.probability_kappa.as_deref()),
                ("trace_kappa", entry.trace_kappa.as_deref()),
            ] {
                if let Some(value) = value
                    && !is_canonical_blake3_address(value)
                {
                    return Err(invalid_data(format!(
                        "observation manifest shard {shard} has non-canonical {field} {value}; expected lowercase blake3:<64 hex>"
                    )));
                }
            }
            if entry.trace_kappa.is_some() && self.trace_profile.is_none() {
                return Err(invalid_data(format!(
                    "observation manifest shard {shard} records a trace sidecar without a trace profile"
                )));
            }
            if entry.trace_kappa.is_some() && self.trace_row_bytes.is_none() {
                return Err(invalid_data(format!(
                    "observation manifest shard {shard} records a trace sidecar without a trace row width"
                )));
            }
            if let Some(partitions) = entry.partitions {
                let partition_records = partitions
                    .construction
                    .checked_add(partitions.held_out)
                    .ok_or_else(|| {
                        invalid_data(format!(
                            "observation manifest shard {shard} partition counts overflow"
                        ))
                    })?;
                if partition_records != entry.records {
                    return Err(invalid_data(format!(
                        "observation manifest shard {shard} partition counts total {partition_records}, but the shard records {}",
                        entry.records
                    )));
                }
            }
            total_records = total_records.checked_add(entry.records).ok_or_else(|| {
                invalid_data("observation manifest completed record count overflows u64".to_owned())
            })?;
        }
        if total_records != self.total_records {
            return Err(invalid_data(format!(
                "observation manifest total_records {} does not match completed shard total {total_records}",
                self.total_records
            )));
        }
        Ok(())
    }

    /// Parse and registry-validate one manifest generation already captured
    /// by a caller-owned filesystem snapshot. Unlike [`Self::load`], this
    /// never reopens `manifest.json`; recorded-corpus consumers can therefore
    /// bind the returned execution identity to the exact captured manifest
    /// bytes. It deliberately preserves [`Self::load`]'s validation of every
    /// manifest-referenced shard payload and trace layout.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn parse_captured_execution_identity(
        dir: &Path,
        path: &Path,
        bytes: &[u8],
    ) -> Result<Self, SourceUnavailable> {
        let manifest: Self = serde_json::from_slice(bytes).map_err(|error| {
            invalid_data(format!(
                "invalid observation manifest {}: {error}",
                path.display()
            ))
        })?;
        manifest.validate_loaded_semantics()?;
        validate_canonical_shard_payload_names(dir, &manifest)?;
        validate_completed_payloads(dir, &manifest)?;
        if manifest.trace_profile.is_some()
            && manifest.trace_row_bytes.is_none()
            && (regular_file_present(&dir.join(RAW_COMMITTED_FILE), "trace-layout")?
                || regular_file_present(&dir.join(STATE_FILE), "trace-layout")?
                || observation_shard_payload_present(dir)?)
        {
            return Err(invalid_data(format!(
                "observation manifest records a trace profile but no trace row width despite existing raw stream payload in {}",
                dir.display()
            )));
        }
        if let Some(geometry) = manifest.geometry.as_ref() {
            validate_registered_geometry_projection(geometry)?;
        }
        if let Some(adapter) = manifest.tokenizer_adapter.as_ref() {
            validate_registered_tokenizer_adapter(adapter)?;
        }
        if let Some(operator) = manifest.attention_operator.as_ref() {
            validate_registered_source_attention_operator(operator)?;
        }
        if let Some(operator) = manifest.dense_operator.as_ref() {
            validate_registered_source_dense_operator(operator)?;
        }
        validate_source_execution_identity(
            manifest.attention_operator.as_ref(),
            manifest.dense_operator.as_ref(),
            &format!("observation manifest {}", path.display()),
        )?;
        if let Some(profile) = manifest.trace_profile.as_ref() {
            validate_registered_trace_profile(profile)?;
        }
        Ok(manifest)
    }

    /// Load the manifest of an observation directory, if present.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(dir: &Path) -> Result<Option<Self>, SourceUnavailable> {
        let path = dir.join(MANIFEST_FILE);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.file_type().is_file() => Err(invalid_data(format!(
                "observation manifest {} is not a regular file; symlinks and non-file entries are refused",
                path.display()
            ))),
            Ok(_) => fs::read(&path)
                .map_err(SourceUnavailable::from)
                .and_then(|bytes| {
                    let manifest: Self = serde_json::from_slice(&bytes).map_err(|error| {
                        invalid_data(format!("invalid observation manifest: {error}"))
                    })?;
                    manifest.validate_loaded_semantics()?;
                    validate_canonical_shard_payload_names(dir, &manifest)?;
                    validate_completed_payloads(dir, &manifest)?;
                    if manifest.trace_profile.is_some()
                        && manifest.trace_row_bytes.is_none()
                        && (regular_file_present(
                            &dir.join(RAW_COMMITTED_FILE),
                            "trace-layout",
                        )? || regular_file_present(&dir.join(STATE_FILE), "trace-layout")?
                            || observation_shard_payload_present(dir)?)
                    {
                        return Err(invalid_data(format!(
                            "observation manifest records a trace profile but no trace row width despite existing raw stream payload in {}",
                            dir.display()
                        )));
                    }
                    if let Some(geometry) = manifest.geometry.as_ref() {
                        validate_registered_geometry_projection(geometry)?;
                    }
                    if let Some(adapter) = manifest.tokenizer_adapter.as_ref() {
                        validate_registered_tokenizer_adapter(adapter)?;
                    }
                    if let Some(operator) = manifest.attention_operator.as_ref() {
                        validate_registered_source_attention_operator(operator)?;
                    }
                    if let Some(operator) = manifest.dense_operator.as_ref() {
                        validate_registered_source_dense_operator(operator)?;
                    }
                    validate_source_execution_identity(
                        manifest.attention_operator.as_ref(),
                        manifest.dense_operator.as_ref(),
                        &format!("observation manifest {}", path.display()),
                    )?;
                    if let Some(profile) = manifest.trace_profile.as_ref() {
                        validate_registered_trace_profile(profile)?;
                    }
                    Ok(Some(manifest))
                }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Persist the manifest atomically (write-then-rename). Shard files are
    /// flushed before completed entries are published. A lost finalization
    /// update leaves that shard incomplete; raw resume restores the
    /// authoritative committed prefix before retrying finalization.
    #[cfg(not(target_arch = "wasm32"))]
    fn store(&self, dir: &Path) -> Result<(), SourceUnavailable> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| invalid_data(format!("manifest serialization: {error}")))?;
        let destination = dir.join(MANIFEST_FILE);
        match fs::symlink_metadata(&destination) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(invalid_input(format!(
                    "observation manifest {} is not a regular file; refusing atomic replacement",
                    destination.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        // Each publisher owns a create_new temporary. It never opens or
        // removes a caller-planted path, and concurrent publishers cannot
        // clobber one another's bytes before the final atomic rename.
        for _ in 0..64 {
            let sequence =
                MANIFEST_TEMP_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let tmp = dir.join(format!(
                ".manifest.json.tmp-{}-{sequence}",
                std::process::id()
            ));
            let mut file = match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)
            {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            };
            let write_result = file.write_all(&bytes).and_then(|()| file.sync_all());
            drop(file);
            if let Err(error) = write_result {
                let _ = fs::remove_file(&tmp);
                return Err(error.into());
            }
            if let Err(error) = fs::rename(&tmp, &destination) {
                let _ = fs::remove_file(&tmp);
                return Err(error.into());
            }
            sync_directory(dir)?;
            return Ok(());
        }
        Err(invalid_input(format!(
            "could not reserve a unique observation-manifest temporary in {}",
            dir.display()
        )))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_base_shard_payload_name(name: &str) -> bool {
    name.strip_prefix("shard-")
        .and_then(|rest| rest.strip_suffix(".bin"))
        .is_some_and(|index| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(not(target_arch = "wasm32"))]
fn is_shard_payload_name(name: &str) -> bool {
    let base = name
        .strip_suffix(".prob")
        .or_else(|| name.strip_suffix(".trace"))
        .unwrap_or(name);
    is_base_shard_payload_name(base)
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_canonical_shard_payload_names(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<(), SourceUnavailable> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_shard_payload_name(name) {
            continue;
        }
        let (base, suffix) = if let Some(base) = name.strip_suffix(".prob") {
            (base, ".prob")
        } else if let Some(base) = name.strip_suffix(".trace") {
            (base, ".trace")
        } else {
            (name, "")
        };
        let shard = base
            .strip_prefix("shard-")
            .and_then(|rest| rest.strip_suffix(".bin"))
            .and_then(|index| index.parse::<u32>().ok())
            .ok_or_else(|| invalid_data(format!("invalid observation shard name {name}")))?;
        if shard >= manifest.shard_count() {
            return Err(invalid_data(format!(
                "observation payload {name} is outside the manifest's {}-shard fan-out",
                manifest.shard_count()
            )));
        }
        let canonical = format!("{}{suffix}", shard_file_name(manifest.shard_bits, shard));
        if name != canonical {
            return Err(invalid_data(format!(
                "observation payload {name} is a non-canonical numeric alias of {canonical}; refusing ambiguous shard bytes"
            )));
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_completed_payloads(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<(), SourceUnavailable> {
    let validate_file = |path: &Path,
                         expected_bytes: u64,
                         expected_kappa: &str|
     -> Result<(), SourceUnavailable> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_file() => metadata,
            Ok(_) => {
                return Err(invalid_data(format!(
                    "completed observation payload {} is not a regular file",
                    path.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(invalid_data(format!(
                    "completed observation payload {} is missing",
                    path.display()
                )));
            }
            Err(error) => return Err(error.into()),
        };
        if metadata.len() != expected_bytes {
            return Err(invalid_data(format!(
                "completed observation payload {} has {} bytes, but its manifest commits {expected_bytes}",
                path.display(),
                metadata.len()
            )));
        }
        let actual_kappa = file_kappa(path)?;
        if actual_kappa != expected_kappa {
            return Err(invalid_data(format!(
                "completed observation payload {} has κ {actual_kappa}, but its manifest commits {expected_kappa}",
                path.display()
            )));
        }
        Ok(())
    };

    for (&shard, entry) in &manifest.completed {
        let base_name = shard_file_name(manifest.shard_bits, shard);
        let base_path = dir.join(&base_name);
        let base_bytes = entry
            .records
            .checked_mul(RECORD_SIZE as u64)
            .ok_or_else(|| {
                invalid_data(format!("completed shard {shard} byte length overflows"))
            })?;
        validate_file(&base_path, base_bytes, &entry.kappa)?;

        let probability_path = dir.join(format!("{base_name}.prob"));
        match entry.probability_kappa.as_deref() {
            Some(kappa) => {
                let bytes = entry
                    .records
                    .checked_mul(PROBABILITY_METADATA_SIZE as u64)
                    .ok_or_else(|| {
                        invalid_data(format!(
                            "completed shard {shard} probability length overflows"
                        ))
                    })?;
                validate_file(&probability_path, bytes, kappa)?;
            }
            None if regular_file_present(&probability_path, "completed-shard")? => {
                return Err(invalid_data(format!(
                    "completed shard {shard} has an uncommitted probability sidecar"
                )));
            }
            None => {}
        }

        let trace_path = dir.join(format!("{base_name}.trace"));
        match entry.trace_kappa.as_deref() {
            Some(kappa) => {
                let row_bytes = manifest.trace_row_bytes.ok_or_else(|| {
                    invalid_data(format!(
                        "completed traced shard {shard} has no trace row width"
                    ))
                })?;
                let bytes = entry.records.checked_mul(row_bytes).ok_or_else(|| {
                    invalid_data(format!("completed shard {shard} trace length overflows"))
                })?;
                validate_file(&trace_path, bytes, kappa)?;
            }
            None if regular_file_present(&trace_path, "completed-shard")? => {
                return Err(invalid_data(format!(
                    "completed shard {shard} has an uncommitted trace sidecar"
                )));
            }
            None => {}
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn is_observation_payload_name(name: &str) -> bool {
    OBSERVATION_PAYLOAD_FILES.contains(&name)
        || is_manifest_temp_name(name)
        || is_raw_transaction_temp_name(name)
        || name.starts_with("stories.tmp-")
        || is_shard_payload_name(name)
}

#[cfg(not(target_arch = "wasm32"))]
fn is_manifest_temp_name(name: &str) -> bool {
    name == ".manifest.json.tmp" || name.starts_with(".manifest.json.tmp-")
}

#[cfg(not(target_arch = "wasm32"))]
fn is_raw_transaction_temp_name(name: &str) -> bool {
    name.starts_with(".raw-committed.bin.tmp-") || name.starts_with(".state.bin.tmp-")
}

#[cfg(not(target_arch = "wasm32"))]
fn regular_file_present(path: &Path, provenance: &str) -> Result<bool, SourceUnavailable> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => Err(invalid_input(format!(
            "observation payload path {} is not a regular file; refusing {provenance} provenance mutation",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

/// Validate every entry that a writer may later open, truncate, append, or
/// atomically replace. This metadata-only pass runs before the coordination
/// inode can be created so a static symlink/nonregular refusal leaves the
/// observation directory byte-identical; it is repeated under the lock before
/// any payload bytes are trusted.
#[cfg(not(target_arch = "wasm32"))]
fn validate_observation_entry_types(dir: &Path) -> Result<(), SourceUnavailable> {
    let manifest_path = dir.join(MANIFEST_FILE);
    match fs::symlink_metadata(&manifest_path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => {
            return Err(invalid_input(format!(
                "observation manifest {} is not a regular file; symlinks and non-file entries are refused",
                manifest_path.display()
            )));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                if name.to_str().is_some_and(is_observation_payload_name) {
                    let _ = regular_file_present(&entry.path(), "observation")?;
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

/// Discard unpublished manifest-store residue while the caller holds the
/// observation session lock. A temp file alone cannot establish an arithmetic
/// era: `store` publishes only at rename, before any row write can proceed.
/// Zero/truncated bytes are therefore safe crash residue; only a nonregular
/// entry fails closed so recovery never follows or removes a symlink target.
#[cfg(not(target_arch = "wasm32"))]
fn recover_manifest_temp_residue(dir: &Path) -> Result<(), SourceUnavailable> {
    let mut temporaries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if !name.to_str().is_some_and(is_manifest_temp_name) {
            continue;
        }
        let path = entry.path();
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(invalid_input(format!(
                "observation manifest temporary {} is not a regular file; refusing recovery",
                path.display()
            )));
        }
        temporaries.push(path);
    }
    temporaries.sort();
    for path in temporaries {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn prepare_observation_parent(dir: &Path) -> Result<(), SourceUnavailable> {
    let parent = dir
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    match fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(()),
        Ok(_) => Err(invalid_input(format!(
            "observation output parent {} is not a real directory",
            parent.display()
        ))),
        Err(error) => Err(error.into()),
    }
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
fn observation_metadata_identity_matches(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(all(not(target_arch = "wasm32"), not(unix)))]
fn observation_metadata_identity_matches(_left: &fs::Metadata, _right: &fs::Metadata) -> bool {
    true
}

#[cfg(not(target_arch = "wasm32"))]
fn open_observation_directory_nofollow(
    path: &Path,
    context: &str,
) -> Result<fs::File, SourceUnavailable> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        );
    }
    let file = options.open(path).map_err(|error| {
        invalid_input(format!(
            "{context} {} cannot be opened without following links: {error}",
            path.display()
        ))
    })?;
    let path_metadata = fs::symlink_metadata(path)?;
    let file_metadata = file.metadata()?;
    if !path_metadata.file_type().is_dir()
        || !file_metadata.file_type().is_dir()
        || !observation_metadata_identity_matches(&path_metadata, &file_metadata)
    {
        return Err(invalid_input(format!(
            "{context} {} is not one stable real directory",
            path.display()
        )));
    }
    Ok(file)
}

/// Acquire the process-crash-safe, full-session writer lock. The permanent
/// empty file is coordination metadata rather than a create/delete lease: the
/// OS releases its exclusive lock when the `File` guard is dropped, including
/// after process death, so no stale-lock recovery protocol is needed.
#[cfg(not(target_arch = "wasm32"))]
fn acquire_observation_session_lock(dir: &Path) -> Result<fs::File, SourceUnavailable> {
    // Keep coordination outside the observation directory so an absent root
    // remains absent when another producer owns the common logical-path
    // guard. The exact basename lives inside a protected sibling directory;
    // filesystem collation therefore makes case/Unicode aliases contend on
    // the same inode without needing a lossy path hash.
    let root_name = dir.file_name().ok_or_else(|| {
        invalid_input(format!(
            "observation output {} has no final component for its coordination lock",
            dir.display()
        ))
    })?;
    if root_name == std::ffi::OsStr::new(".") || root_name == std::ffi::OsStr::new("..") {
        return Err(invalid_input(
            "observation output cannot be '.' or '..'".to_owned(),
        ));
    }
    let requested_parent = dir
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical_parent = fs::canonicalize(requested_parent)?;
    let parent_file = open_observation_directory_nofollow(
        &canonical_parent,
        "observation session canonical parent",
    )?;
    let coordination = canonical_parent.join(OBSERVATION_SESSION_COORDINATION_DIR);
    match fs::create_dir(&coordination) {
        Ok(()) => parent_file.sync_all()?,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    let coordination_file = open_observation_directory_nofollow(
        &coordination,
        "observation session coordination directory",
    )?;
    let path = coordination.join(root_name);
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(&path).map_err(|error| {
        invalid_input(format!(
            "observation session lock {} cannot be opened without following links: {error}",
            path.display()
        ))
    })?;
    let initial_path = fs::symlink_metadata(&path)?;
    let initial_file = file.metadata()?;
    if !initial_path.file_type().is_file()
        || !initial_file.file_type().is_file()
        || !observation_metadata_identity_matches(&initial_path, &initial_file)
    {
        return Err(invalid_input(format!(
            "observation session lock {} is not one stable regular non-symlink inode",
            path.display()
        )));
    }
    file.lock()?;
    let final_path = fs::symlink_metadata(&path)?;
    let final_file = file.metadata()?;
    if !final_path.file_type().is_file()
        || !final_file.file_type().is_file()
        || !observation_metadata_identity_matches(&initial_file, &final_file)
        || !observation_metadata_identity_matches(&final_path, &final_file)
    {
        return Err(invalid_input(format!(
            "observation session lock {} changed identity or type while acquired",
            path.display()
        )));
    }
    let final_parent = parent_file.metadata()?;
    let path_parent = fs::symlink_metadata(&canonical_parent)?;
    let final_coordination = coordination_file.metadata()?;
    let path_coordination = fs::symlink_metadata(&coordination)?;
    if !observation_metadata_identity_matches(&final_parent, &path_parent)
        || !observation_metadata_identity_matches(&final_coordination, &path_coordination)
    {
        return Err(invalid_input(
            "observation session coordination directories changed identity while acquired"
                .to_owned(),
        ));
    }
    Ok(file)
}

#[cfg(not(target_arch = "wasm32"))]
fn acquire_observation_session_guards_with_hook<F>(
    dir: &Path,
    after_outer_lock: F,
) -> Result<
    (
        crate::recorded_corpus::RecordedCorpusProducerGuard,
        fs::File,
    ),
    SourceUnavailable,
>
where
    F: FnOnce(),
{
    let observation_lock = acquire_observation_session_lock(dir)?;
    after_outer_lock();
    let recorded_corpus_guard =
        crate::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(dir)?;
    Ok((recorded_corpus_guard, observation_lock))
}

/// Name of one shard's #603 trace sidecar: `<shard file>.trace`,
/// mirroring the `.prob` probability sidecar naming.
pub fn trace_sidecar_name(shard_bits: u8, shard: u32) -> String {
    format!("{}.trace", shard_file_name(shard_bits, shard))
}

/// Whether an observation directory already contains payload or resumable
/// state whose tokenizer and attention eras are fixed. This inventory is the
/// single fail-closed source used by both provenance setters and by their
/// read-only entry-point preflights.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn observation_payload_present(
    dir: &Path,
    manifest: &ObservationManifest,
    provenance: &str,
) -> Result<bool, SourceUnavailable> {
    let manifest_has_payload = manifest.total_records != 0 || !manifest.completed.is_empty();
    let mut files_have_payload = false;
    // Shards are not the only tokenizer/attention-era evidence. The raw
    // driver checkpoints its token stream in state.bin; the text driver
    // additionally checkpoints article progress and story identity; the CLI
    // can persist a merged record stream and the exact runtime token table in
    // the same directory. A stripped/torn historical manifest must not make
    // any of those sessions look fresh. The committed temp is included
    // because a crash between write and rename is still evidence of an
    // in-progress tokenizer- and operator-dependent session.
    for name in OBSERVATION_PAYLOAD_FILES {
        files_have_payload |= regular_file_present(&dir.join(name), provenance)?;
    }
    // Discover shard payload from the directory itself, not from the
    // manifest/requested fan-out. A stripped manifest combined with a smaller
    // requested shard_bits must not hide (for example) shard-02.bin or either
    // of its sidecars from the era check.
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                if let Some(name) = name.to_str() {
                    if is_shard_payload_name(name) || name.starts_with("stories.tmp-") {
                        files_have_payload |= regular_file_present(&entry.path(), provenance)?;
                    } else if is_manifest_temp_name(name) {
                        // Unpublished store residue does not establish an era,
                        // but it must still be regular before locked recovery.
                        let _ = regular_file_present(&entry.path(), provenance)?;
                    }
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(manifest_has_payload || files_have_payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn observation_shard_payload_present(dir: &Path) -> Result<bool, SourceUnavailable> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(is_shard_payload_name)
        {
            let _ = regular_file_present(&entry.path(), "checkpoint")?;
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawResumeStatus {
    /// No raw or text transaction evidence exists yet.
    Fresh,
    /// A versioned raw checkpoint authoritatively selects these bytes/state.
    Authoritative(RawCommittedCheckpoint),
    /// A pre-checkpoint raw bundle whose complete manifest and kappas prove its
    /// final boundaries. It is accepted read-only and is never migrated.
    FinalizedLegacy {
        records: u64,
        stories: u64,
        rng: u64,
    },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct RawPayloadRecovery {
    path: PathBuf,
    current_bytes: u64,
    committed_bytes: u64,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct RawRecoveryPlan {
    checkpoint: RawCommittedCheckpoint,
    payloads: Vec<RawPayloadRecovery>,
    repair_state_mirror: bool,
}

#[cfg(not(target_arch = "wasm32"))]
enum RawResumePlan {
    Fresh,
    Authoritative(RawRecoveryPlan),
    FinalizedLegacy(ObservationState),
}

#[cfg(not(target_arch = "wasm32"))]
fn preflight_recovery_payload(
    path: PathBuf,
    row_bytes: u64,
    committed_rows: u64,
    label: &str,
) -> Result<Option<RawPayloadRecovery>, SourceUnavailable> {
    let committed_bytes = committed_rows.checked_mul(row_bytes).ok_or_else(|| {
        invalid_data(format!(
            "raw checkpoint {label} committed byte length overflows u64"
        ))
    })?;
    let current_bytes = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => metadata.len(),
        Ok(_) => {
            return Err(invalid_input(format!(
                "raw observation {label} {} is not a regular file",
                path.display()
            )));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound && committed_bytes == 0 => {
            return Ok(None);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(invalid_data(format!(
                "raw observation {label} {} is missing but the checkpoint commits {committed_bytes} bytes",
                path.display()
            )));
        }
        Err(error) => return Err(error.into()),
    };
    if current_bytes % row_bytes != 0 {
        return Err(invalid_data(format!(
            "raw observation {label} {} has a torn row ({current_bytes} bytes at width {row_bytes})",
            path.display()
        )));
    }
    if current_bytes < committed_bytes {
        return Err(invalid_data(format!(
            "raw observation {label} {} has {current_bytes} bytes, shorter than the committed {committed_bytes}-byte prefix",
            path.display()
        )));
    }
    Ok(Some(RawPayloadRecovery {
        path,
        current_bytes,
        committed_bytes,
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn current_raw_record_counts(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<Vec<u64>, SourceUnavailable> {
    let row_count = |path: &Path, row_bytes: u64, label: &str| match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let length = metadata.len();
            if length % row_bytes != 0 {
                return Err(invalid_data(format!(
                    "raw observation {label} {} has a torn row ({length} bytes at width {row_bytes})",
                    path.display()
                )));
            }
            Ok(Some(length / row_bytes))
        }
        Ok(_) => Err(invalid_input(format!(
            "raw observation {label} {} is not a regular file",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    };
    let mut counts = Vec::with_capacity(manifest.shard_count() as usize);
    for shard in 0..manifest.shard_count() {
        let base_name = shard_file_name(manifest.shard_bits, shard);
        let base = row_count(&dir.join(&base_name), RECORD_SIZE as u64, "base shard")?.unwrap_or(0);
        let probability = row_count(
            &dir.join(format!("{base_name}.prob")),
            PROBABILITY_METADATA_SIZE as u64,
            "probability sidecar",
        )?
        .unwrap_or(0);
        if probability != base {
            return Err(invalid_data(format!(
                "raw observation shard {shard} has {base} base rows but {probability} probability rows"
            )));
        }
        let trace_path = dir.join(trace_sidecar_name(manifest.shard_bits, shard));
        match manifest.trace_row_bytes {
            Some(row_bytes) => {
                let trace = row_count(&trace_path, row_bytes, "trace sidecar")?.unwrap_or(0);
                if trace != base {
                    return Err(invalid_data(format!(
                        "raw observation shard {shard} has {base} base rows but {trace} trace rows"
                    )));
                }
            }
            None if regular_file_present(&trace_path, "raw-checkpoint")? => {
                return Err(invalid_data(format!(
                    "raw observation shard {shard} has a trace sidecar under the minimal layout"
                )));
            }
            None => {}
        }
        counts.push(base);
    }
    Ok(counts)
}

#[cfg(not(target_arch = "wasm32"))]
fn state_mirror_needs_repair(
    dir: &Path,
    checkpoint: &RawCommittedCheckpoint,
) -> Result<bool, SourceUnavailable> {
    let path = dir.join(STATE_FILE);
    let expected = {
        let (n, stories, rng, done) = checkpoint.state();
        encode_observation_state((n, stories, rng, u8::from(done)))
    };
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(fs::read(path)? != expected),
        Ok(_) => Err(invalid_input(format!(
            "observation state mirror {} is not a regular file",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error.into()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn preflight_raw_resume(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<RawResumePlan, SourceUnavailable> {
    let text_story_evidence = fs::read_dir(dir)?.try_fold(false, |found, entry| {
        let entry = entry?;
        let is_story = entry.file_name().to_str().is_some_and(|name| {
            name == "stories.jsonl" || name == "stories.tmp" || name.starts_with("stories.tmp-")
        });
        Ok::<_, io::Error>(found || is_story)
    })?;
    if manifest.partition_rule.is_some()
        || manifest.input_cid.is_some()
        || text_story_evidence
        || regular_file_present(&dir.join("committed.bin"), "raw-checkpoint")?
        || regular_file_present(&dir.join(".committed.bin.tmp"), "raw-checkpoint")?
    {
        return Err(invalid_data(format!(
            "{} contains the text observation committed.bin format; refusing a mixed raw observation resume",
            dir.display()
        )));
    }
    let Some(checkpoint) = read_raw_checkpoint(dir, manifest)? else {
        let state = read_observation_state(dir)?;
        let shard_payload = observation_shard_payload_present(dir)?;
        let merged = regular_file_present(&dir.join("merged.bin"), "raw-checkpoint")?;
        let stream_evidence = state.is_some()
            || shard_payload
            || merged
            || manifest.total_records != 0
            || !manifest.completed.is_empty();
        if !stream_evidence {
            return Ok(RawResumePlan::Fresh);
        }
        let fully_finalized = manifest.completed.len() == manifest.shard_count() as usize;
        if fully_finalized
            && let Some((n, stories, rng, 1)) = state
            && n == manifest.total_records
        {
            return Ok(RawResumePlan::FinalizedLegacy((n, stories, rng, 1)));
        }
        return Err(invalid_data(format!(
            "{} is an unfinished legacy raw observation directory with no authoritative {RAW_COMMITTED_FILE}; its aligned bytes cannot prove a whole-story boundary, so restart in a fresh output directory",
            dir.display()
        )));
    };
    if !checkpoint.done && !manifest.completed.is_empty() {
        return Err(invalid_data(format!(
            "{} has finalized raw shards but its authoritative checkpoint is unfinished; refusing an inconsistent resume",
            dir.display()
        )));
    }

    let mut payloads = Vec::new();
    for (shard, &records) in checkpoint.committed_records().iter().enumerate() {
        let shard = shard as u32;
        let base_name = shard_file_name(manifest.shard_bits, shard);
        if let Some(plan) = preflight_recovery_payload(
            dir.join(&base_name),
            RECORD_SIZE as u64,
            records,
            "base shard",
        )? {
            payloads.push(plan);
        }
        if let Some(plan) = preflight_recovery_payload(
            dir.join(format!("{base_name}.prob")),
            PROBABILITY_METADATA_SIZE as u64,
            records,
            "probability sidecar",
        )? {
            payloads.push(plan);
        }
        let trace_path = dir.join(trace_sidecar_name(manifest.shard_bits, shard));
        match checkpoint.trace_row_bytes() {
            Some(row_bytes) => {
                if let Some(plan) =
                    preflight_recovery_payload(trace_path, row_bytes, records, "trace sidecar")?
                {
                    payloads.push(plan);
                }
            }
            None if regular_file_present(&trace_path, "raw-checkpoint")? => {
                return Err(invalid_data(format!(
                    "raw observation shard {shard} has a trace sidecar, but the checkpoint pins the minimal trace layout"
                )));
            }
            None => {}
        }
    }
    Ok(RawResumePlan::Authoritative(RawRecoveryPlan {
        repair_state_mirror: state_mirror_needs_repair(dir, &checkpoint)?,
        checkpoint,
        payloads,
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn status_from_plan(plan: &RawResumePlan) -> RawResumeStatus {
    match plan {
        RawResumePlan::Fresh => RawResumeStatus::Fresh,
        RawResumePlan::Authoritative(plan) => {
            RawResumeStatus::Authoritative(plan.checkpoint.clone())
        }
        RawResumePlan::FinalizedLegacy((records, stories, rng, _)) => {
            RawResumeStatus::FinalizedLegacy {
                records: *records,
                stories: *stories,
                rng: *rng,
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_raw_recovery(dir: &Path, plan: &RawRecoveryPlan) -> Result<(), SourceUnavailable> {
    // Phase two still opens and revalidates every file before changing the
    // first one. This keeps a late permission/race failure from partially
    // applying the plan assembled by the read-only phase.
    let mut files = Vec::with_capacity(plan.payloads.len());
    for payload in &plan.payloads {
        let truncate = payload.current_bytes != payload.committed_bytes;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(truncate)
            .open(&payload.path)?;
        if file.metadata()?.len() != payload.current_bytes {
            return Err(invalid_data(format!(
                "raw observation payload {} changed after preflight; refusing recovery",
                payload.path.display()
            )));
        }
        files.push((file, payload.committed_bytes, truncate));
    }
    for (file, committed_bytes, truncate) in files {
        if truncate {
            file.set_len(committed_bytes)?;
            file.sync_all()?;
        }
    }
    if plan
        .payloads
        .iter()
        .any(|payload| payload.current_bytes != payload.committed_bytes)
    {
        sync_directory(dir)?;
    }
    if plan.repair_state_mirror {
        publish_state_mirror(dir, &plan.checkpoint)?;
    }
    Ok(())
}

/// Inspect raw recovery under an already-held observation session without
/// mutating payload, provenance, checkpoint, or mirror bytes.
#[cfg(not(target_arch = "wasm32"))]
pub fn preflight_raw_observation_in_session(
    session: &ObservationSession,
) -> Result<RawResumeStatus, SourceUnavailable> {
    let writer = session.writer_for_preflight()?;
    let plan = preflight_raw_resume(session.dir(), writer.manifest())?;
    Ok(status_from_plan(&plan))
}

/// Apply an authoritative raw recovery under an already-held session. Fresh
/// outputs are left untouched (the observe driver publishes their identity-
/// bound zero checkpoint), and finalized legacy bundles remain read-only.
#[cfg(not(target_arch = "wasm32"))]
pub fn recover_raw_observation_in_session(
    session: &ObservationSession,
) -> Result<RawResumeStatus, SourceUnavailable> {
    let writer = session.writer_for_preflight()?;
    let plan = preflight_raw_resume(session.dir(), writer.manifest())?;
    drop(writer);
    session.recover_recorded_corpus_binding_after_preflight()?;
    if let RawResumePlan::Authoritative(recovery) = &plan {
        apply_raw_recovery(session.dir(), recovery)?;
    }
    Ok(status_from_plan(&plan))
}

#[cfg(not(target_arch = "wasm32"))]
fn preflight_observation_state(
    dir: &Path,
    manifest: &ObservationManifest,
) -> Result<Option<ObservationState>, SourceUnavailable> {
    match preflight_raw_resume(dir, manifest)? {
        RawResumePlan::Fresh => Ok(None),
        RawResumePlan::Authoritative(plan) => {
            let (n, stories, rng, done) = plan.checkpoint.state();
            Ok(Some((n, stories, rng, u8::from(done))))
        }
        RawResumePlan::FinalizedLegacy(state) => Ok(Some(state)),
    }
}

/// Exclusive coordination guard for one observation output directory.
///
/// Hold this value across the complete provenance-pin, tokenizer-export,
/// reconciliation, generation, checkpoint, and finalization session. Writers
/// opened through [`ObservationSession::writer`] share the already-acquired OS
/// lock; the lock is released only after the session and every such writer are
/// dropped.
#[cfg(not(target_arch = "wasm32"))]
pub struct ObservationSession {
    dir: PathBuf,
    shard_bits: u8,
    state: Arc<ObservationSessionState>,
}

#[cfg(not(target_arch = "wasm32"))]
struct ObservationSessionState {
    _recorded_corpus_guard: crate::recorded_corpus::RecordedCorpusProducerGuard,
    _lock: fs::File,
    binding_recovery_needed: std::sync::atomic::AtomicBool,
    writer_active: std::sync::atomic::AtomicBool,
}

#[cfg(not(target_arch = "wasm32"))]
struct ObservationWriterLease {
    state: Arc<ObservationSessionState>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for ObservationWriterLease {
    fn drop(&mut self) {
        self.state
            .writer_active
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ObservationSession {
    /// Acquire the exclusive full-session lock and validate the authoritative
    /// manifest/payload inventory under it.
    pub fn acquire(dir: impl AsRef<Path>, shard_bits: u8) -> Result<Self, SourceUnavailable> {
        if shard_bits > MAX_SHARD_BITS {
            return Err(invalid_input(format!(
                "shard_bits {shard_bits} exceeds the writer maximum {MAX_SHARD_BITS}"
            )));
        }
        let requested_dir = dir.as_ref().to_path_buf();
        prepare_observation_parent(&requested_dir)?;
        // Fail static semantic refusals before creating the permanent
        // coordination inode. This snapshot is only an optimization for
        // failure atomicity; `load_manifest` repeats it authoritatively after
        // the lock is acquired.
        match fs::symlink_metadata(&requested_dir) {
            Ok(metadata) if metadata.file_type().is_dir() => {
                validate_observation_entry_types(&requested_dir)?;
                if let Some(manifest) = ObservationManifest::load(&requested_dir)?
                    && manifest.shard_bits != shard_bits
                {
                    return Err(invalid_input(format!(
                        "manifest shard_bits {} does not match requested {shard_bits}",
                        manifest.shard_bits
                    )));
                }
            }
            Ok(_) => {
                return Err(invalid_input(format!(
                    "observation output root {} is not a real directory",
                    requested_dir.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        // Global acquisition order is writer-specific outer session before
        // the common recorded-corpus guard. Every public raw/text/batched
        // session therefore owns both for its complete mutable lifetime. A
        // common-guard BUSY refusal drops the just-acquired outer file before
        // returning, so no path can retain a prefix of the lock order.
        let (mut recorded_corpus_guard, observation_lock) =
            acquire_observation_session_guards_with_hook(&requested_dir, || {})?;
        if recorded_corpus_guard.root_exists() {
            recorded_corpus_guard.preflight_planned_output_scope(&[])?;
            let _ = recorded_corpus_guard.preflight_publication_namespace_for(
                crate::recorded_corpus::RecordedCorpusRole::Observation,
            )?;
        }
        recorded_corpus_guard.ensure_root()?;
        let dir = recorded_corpus_guard.root().to_path_buf();
        validate_observation_entry_types(&dir)?;
        // Namespace validation is read-only at generic acquisition. A
        // command may not promote an exact `.tmp` until its requested
        // tokenizer/geometry/attention/dense/text identities have all passed
        // under this session; otherwise a mismatched invocation could mutate
        // a directory it must reject byte-for-byte.
        recorded_corpus_guard.preflight_planned_output_scope(&[])?;
        let binding_recovery_needed = recorded_corpus_guard.preflight_publication_namespace_for(
            crate::recorded_corpus::RecordedCorpusRole::Observation,
        )?;
        let state = Arc::new(ObservationSessionState {
            _recorded_corpus_guard: recorded_corpus_guard,
            _lock: observation_lock,
            binding_recovery_needed: std::sync::atomic::AtomicBool::new(binding_recovery_needed),
            writer_active: std::sync::atomic::AtomicBool::new(false),
        });
        let session = Self {
            dir,
            shard_bits,
            state,
        };
        // Every manifest and payload read that can authorize mutation happens
        // after lock acquisition. The metadata-only scan above exists solely
        // to make static nonregular-entry refusal failure-atomic.
        let _ = session.load_manifest()?;
        Ok(session)
    }

    /// Observation output directory protected by this session.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Fan-out pinned by this session.
    pub fn shard_bits(&self) -> u8 {
        self.shard_bits
    }

    /// Common recorded-corpus producer guard held for this session's full
    /// provenance, shard, checkpoint, merge, and binding-publication lifetime.
    pub fn recorded_corpus_guard(&self) -> &crate::recorded_corpus::RecordedCorpusProducerGuard {
        &self.state._recorded_corpus_guard
    }

    /// Promote an exact crash temporary only after the caller's complete
    /// command-specific identity preflight has succeeded under this session.
    pub(crate) fn recover_recorded_corpus_binding_after_preflight(
        &self,
    ) -> Result<(), SourceUnavailable> {
        if !self
            .state
            .binding_recovery_needed
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Ok(());
        }
        crate::recorded_corpus::publish_binding(
            self.recorded_corpus_guard(),
            &self.dir.join(STATE_FILE),
            &self.dir.join("merged.bin"),
        )?;
        self.state
            .binding_recovery_needed
            .store(false, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    /// Open a writer that shares this session's already-held exclusive lock.
    /// The manifest and payload entry types are reloaded under the lock.
    pub fn writer(&self) -> Result<ObservationShardWriter, SourceUnavailable> {
        if self
            .state
            .binding_recovery_needed
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(invalid_input(format!(
                "observation session for {} has an unpublished canonical corpus binding temporary; complete command-specific identity preflight and recovery before mutation",
                self.dir.display()
            )));
        }
        self.writer_for_preflight()
    }

    pub(crate) fn writer_for_preflight(&self) -> Result<ObservationShardWriter, SourceUnavailable> {
        if self
            .state
            .writer_active
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(invalid_input(format!(
                "observation session for {} already has a live writer; drop it before opening another",
                self.dir.display()
            )));
        }
        let lease = ObservationWriterLease {
            state: Arc::clone(&self.state),
        };
        let manifest = self.load_manifest()?;
        let handles = (0..manifest.shard_count()).map(|_| None).collect();
        let partition_counts = (0..manifest.shard_count())
            .map(|_| PartitionCounts::default())
            .collect();
        Ok(ObservationShardWriter {
            dir: self.dir.clone(),
            manifest,
            handles,
            partition_counts,
            partitions_active: false,
            _session_lease: lease,
        })
    }

    fn load_manifest(&self) -> Result<ObservationManifest, SourceUnavailable> {
        validate_observation_entry_types(&self.dir)?;
        let manifest = match ObservationManifest::load(&self.dir)? {
            Some(manifest) => {
                if manifest.shard_bits != self.shard_bits {
                    return Err(invalid_input(format!(
                        "manifest shard_bits {} does not match requested {}",
                        manifest.shard_bits, self.shard_bits
                    )));
                }
                manifest
            }
            None => ObservationManifest::new(self.shard_bits),
        };
        validate_canonical_shard_payload_names(&self.dir, &manifest)?;
        let _ = observation_payload_present(&self.dir, &manifest, "observation")?;
        Ok(manifest)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn preflight_writer_identity_bundle(
    writer: &ObservationShardWriter,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&GeometryProjection>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
    trace_profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    validate_source_execution_identity(
        writer.manifest().attention_operator.as_ref(),
        writer.manifest().dense_operator.as_ref(),
        &format!("observation manifest {}", writer.dir.display()),
    )?;
    validate_source_execution_identity(
        attention_operator,
        dense_operator,
        "requested observation source execution",
    )?;
    writer.preflight_source_manifest_kappa(source_manifest_kappa)?;
    writer.preflight_geometry(geometry)?;
    writer.preflight_tokenizer_adapter(tokenizer_adapter)?;
    writer.preflight_attention_operator(attention_operator)?;
    writer.preflight_dense_operator(dense_operator)?;
    writer.preflight_trace_profile(trace_profile)
}

/// Joint, read-only provenance preflight under a caller-held observation
/// session. Commands call this before tokenizer export or any identity setter;
/// the lower driver repeats the checks before reconciliation and row writes.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn preflight_observation_identities_in_session_with_dense(
    session: &ObservationSession,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&GeometryProjection>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
    trace_profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    let writer = session.writer_for_preflight()?;
    let _ = preflight_observation_state(session.dir(), writer.manifest())?;
    preflight_writer_identity_bundle(
        &writer,
        source_manifest_kappa,
        geometry,
        tokenizer_adapter,
        attention_operator,
        dense_operator,
        trace_profile,
    )
}

/// Compatibility entry point for producers from the dense-operator-absent
/// era. New source-aware callers should use
/// [`preflight_observation_identities_in_session_with_dense`].
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn preflight_observation_identities_in_session(
    session: &ObservationSession,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&GeometryProjection>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    trace_profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    preflight_observation_identities_in_session_with_dense(
        session,
        source_manifest_kappa,
        geometry,
        tokenizer_adapter,
        attention_operator,
        None,
        trace_profile,
    )
}

/// Resolve the requested trace capture layout against the actual oracle and
/// compare its row width with the manifest before tokenizer export or any
/// reconciliation. The richer trace profile alone does not identify capture
/// geometry, so this is a separate read-only half of the joint preflight.
#[cfg(not(target_arch = "wasm32"))]
pub fn preflight_observation_trace_layout_in_session(
    session: &ObservationSession,
    oracle: &dyn TeacherOracle,
    profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    if let Some(profile) = profile {
        validate_registered_trace_request(profile)?;
    }
    let trace = match profile {
        Some(profile) if !profile.is_minimal() => Some(TraceCapture::new(profile, oracle)?),
        Some(_) | None => None,
    };
    let writer = session.writer_for_preflight()?;
    writer.preflight_trace_row_bytes(trace.as_ref().map(|trace| trace.row_bytes as u64))
}

/// Jointly preflight, then publish all present identity records under one
/// exclusive session. No setter runs until every requested/recorded identity
/// has passed the read-only bundle check.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn pin_observation_identities_in_session_with_dense(
    session: &ObservationSession,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&GeometryProjection>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
    trace_profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    let mut writer = session.writer_for_preflight()?;
    let _ = preflight_observation_state(session.dir(), writer.manifest())?;
    preflight_writer_identity_bundle(
        &writer,
        source_manifest_kappa,
        geometry,
        tokenizer_adapter,
        attention_operator,
        dense_operator,
        trace_profile,
    )?;
    session.recover_recorded_corpus_binding_after_preflight()?;
    if let Some(adapter) = tokenizer_adapter {
        writer.set_tokenizer_adapter(adapter)?;
    }
    if let Some(kappa) = source_manifest_kappa {
        writer.set_source_manifest_kappa(kappa)?;
    }
    if let Some(geometry) = geometry {
        writer.set_geometry(geometry)?;
    }
    writer.set_source_execution_pair(attention_operator, dense_operator)?;
    if let Some(profile) = trace_profile {
        writer.set_trace_profile(profile)?;
    }
    Ok(())
}

/// Compatibility entry point preserving the pre-#704 positional API and
/// exact dense-absent semantics.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn pin_observation_identities_in_session(
    session: &ObservationSession,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&GeometryProjection>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    trace_profile: Option<&TraceProfile>,
) -> Result<(), SourceUnavailable> {
    pin_observation_identities_in_session_with_dense(
        session,
        source_manifest_kappa,
        geometry,
        tokenizer_adapter,
        attention_operator,
        None,
        trace_profile,
    )
}

#[cfg(not(target_arch = "wasm32"))]
struct ShardHandle {
    file: BufWriter<fs::File>,
    probability: Option<BufWriter<fs::File>>,
    trace: Option<BufWriter<fs::File>>,
}

/// Spills observation records into per-shard files with a κ-pinned
/// manifest. Records may arrive interleaved across shards (routed by
/// [`shard_of`]); each incomplete shard is appended to after its aligned
/// record/sidecar prefix is validated. Completed shards are never rewritten:
/// writes routed to them are skipped. The raw driver commits this writer's
/// synchronized per-shard lengths at whole-story boundaries.
#[cfg(not(target_arch = "wasm32"))]
pub struct ObservationShardWriter {
    dir: PathBuf,
    manifest: ObservationManifest,
    handles: Vec<Option<ShardHandle>>,
    partition_counts: Vec<PartitionCounts>,
    partitions_active: bool,
    // Keep the guard last so buffered shard handles are dropped before this
    // writer releases its share of the full-session exclusive lock.
    _session_lease: ObservationWriterLease,
}

#[cfg(not(target_arch = "wasm32"))]
impl ObservationShardWriter {
    /// Open (or create) an observation shard directory. An existing
    /// manifest pins the fan-out; requesting a different `shard_bits` for
    /// the same directory is an error.
    pub fn open(dir: impl AsRef<Path>, shard_bits: u8) -> Result<Self, SourceUnavailable> {
        let session = ObservationSession::acquire(dir, shard_bits)?;
        session.recover_recorded_corpus_binding_after_preflight()?;
        session.writer()
    }

    pub fn manifest(&self) -> &ObservationManifest {
        &self.manifest
    }

    pub fn is_complete(&self, shard: u32) -> bool {
        self.manifest.completed.contains_key(&shard)
    }

    /// Record the document-level partition rule in the manifest (persisted
    /// atomically). Idempotent: storing the already-recorded rule is a
    /// no-op and does not rewrite the manifest.
    pub fn set_partition_rule(&mut self, rule: &str) -> Result<(), SourceUnavailable> {
        if self.manifest.partition_rule.as_deref() == Some(rule) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.partition_rule = Some(rule.to_owned());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Record the input CID in the manifest (idempotent, atomic store).
    pub fn set_input_cid(&mut self, cid: &str) -> Result<(), SourceUnavailable> {
        if !is_canonical_blake3_address(cid) {
            return Err(invalid_input(format!(
                "input CID {cid} is not canonical lowercase blake3:<64 hex>; refusing provenance before mutation"
            )));
        }
        if self.manifest.input_cid.as_deref() == Some(cid) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.input_cid = Some(cid.to_owned());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only symmetric compatibility check for the source-snapshot κ.
    /// Once payload exists, an absent κ cannot be backfilled and a bound κ
    /// must be requested exactly.
    pub fn preflight_source_manifest_kappa(
        &self,
        requested: Option<&str>,
    ) -> Result<(), SourceUnavailable> {
        for (role, kappa) in [
            ("recorded", self.manifest.source_manifest_kappa.as_deref()),
            ("requested", requested),
        ] {
            if let Some(kappa) = kappa
                && !is_canonical_blake3_address(kappa)
            {
                return Err(invalid_input(format!(
                    "{role} source manifest κ {kappa} is not canonical lowercase blake3:<64 hex>; refusing provenance before mutation"
                )));
            }
        }
        let payload_present =
            observation_payload_present(&self.dir, &self.manifest, "source-manifest")?;
        match (self.manifest.source_manifest_kappa.as_deref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to source manifest κ {recorded}; requested {requested}; incompatible observation resume refused before mutation",
                self.dir.display()
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to source manifest κ {recorded}; the requested producer declares none; incompatible observation resume refused before mutation",
                self.dir.display()
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded source manifest κ but already contains observation payload; refusing to relabel legacy bytes as {requested} before mutation",
                self.dir.display()
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    /// Record the #597 source-snapshot manifest root κ of the teacher source.
    pub fn set_source_manifest_kappa(&mut self, kappa: &str) -> Result<(), SourceUnavailable> {
        self.preflight_source_manifest_kappa(Some(kappa))?;
        if self.manifest.source_manifest_kappa.as_deref() == Some(kappa) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.source_manifest_kappa = Some(kappa.to_owned());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only symmetric compatibility check for source geometry. `None`
    /// means the pass-through geometry era and is therefore authoritative.
    pub fn preflight_geometry(
        &self,
        requested: Option<&GeometryProjection>,
    ) -> Result<(), SourceUnavailable> {
        if let Some(recorded) = self.manifest.geometry.as_ref() {
            validate_registered_geometry_projection(recorded)?;
        }
        if let Some(requested) = requested {
            validate_registered_geometry_projection(requested)?;
        }
        let payload_present = observation_payload_present(&self.dir, &self.manifest, "geometry")?;
        match (self.manifest.geometry.as_ref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to geometry {}/{} digest {}; requested {}/{} digest {}; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to geometry {}/{} digest {}; the requested producer declares pass-through/none; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded geometry but already contains observation payload from the pass-through/legacy era; refusing to relabel those bytes as {}/{} before mutation",
                self.dir.display(),
                requested.id,
                requested.version,
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    /// Record the #600 typed geometry-projection record of the teacher oracle.
    pub fn set_geometry(&mut self, geometry: &GeometryProjection) -> Result<(), SourceUnavailable> {
        self.preflight_geometry(Some(geometry))?;
        if self.manifest.geometry.as_ref() == Some(geometry) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.geometry = Some(geometry.clone());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only compatibility check for the #601 tokenizer era. This is
    /// symmetric with the attention check so a checkpoint/adapterless producer
    /// cannot resume bytes pinned to a registered tokenizer.
    pub fn preflight_tokenizer_adapter(
        &self,
        requested: Option<&TokenizerAdapter>,
    ) -> Result<(), SourceUnavailable> {
        if let Some(recorded) = self.manifest.tokenizer_adapter.as_ref() {
            validate_registered_tokenizer_adapter(recorded)?;
        }
        if let Some(requested) = requested {
            validate_registered_tokenizer_adapter(requested)?;
        }
        let payload_present = observation_payload_present(&self.dir, &self.manifest, "tokenizer")?;
        match (self.manifest.tokenizer_adapter.as_ref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested {}/{} (CID {}, digest {}); incompatible resume refused before mutation",
                self.dir.display(),
                recorded.family,
                recorded.version,
                recorded.tokenizer_cid,
                recorded.adapter_digest,
                requested.family,
                requested.version,
                requested.tokenizer_cid,
                requested.adapter_digest,
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested the adapterless legacy tokenizer; incompatible resume refused before mutation",
                self.dir.display(),
                recorded.family,
                recorded.version,
                recorded.tokenizer_cid,
                recorded.adapter_digest,
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded tokenizer adapter but already contains observation payload; refusing to relabel legacy/unpinned bytes as {}/{} (CID {}, digest {}) before mutation",
                self.dir.display(),
                requested.family,
                requested.version,
                requested.tokenizer_cid,
                requested.adapter_digest,
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    /// Record the #601 typed tokenizer-adapter identity record of the
    /// producing pipeline's tokenizer in the observation manifest. The cloned
    /// candidate replaces in-memory state only after its atomic store succeeds.
    pub fn set_tokenizer_adapter(
        &mut self,
        adapter: &TokenizerAdapter,
    ) -> Result<(), SourceUnavailable> {
        self.preflight_tokenizer_adapter(Some(adapter))?;
        recover_manifest_temp_residue(&self.dir)?;
        if self.manifest.tokenizer_adapter.as_ref() == Some(adapter) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.tokenizer_adapter = Some(adapter.clone());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only compatibility check for the #602 attention-operator era.
    /// Compatibility is symmetric: an operator-declaring producer cannot
    /// relabel operatorless legacy payload, and an operatorless producer
    /// cannot resume a directory pinned to an explicit operator.
    pub fn preflight_attention_operator(
        &self,
        requested: Option<&AttentionOperatorSpec>,
    ) -> Result<(), SourceUnavailable> {
        if let Some(recorded) = self.manifest.attention_operator.as_ref() {
            validate_registered_source_attention_operator(recorded)?;
        }
        if let Some(requested) = requested {
            validate_registered_source_attention_operator(requested)?;
        }
        // Always scan for nonregular payload entries, even when the recorded
        // and requested operators match. Otherwise reconciliation or append
        // could follow a shard/sidecar symlink and mutate its external target.
        let payload_present =
            observation_payload_present(&self.dir, &self.manifest, "attention-operator")?;

        match (self.manifest.attention_operator.as_ref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to attention operator {}/{} digest {}; requested {}/{} digest {}; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to attention operator {}/{} digest {}; the requested producer declares none; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded attention operator but already contains observation payload from the implicit legacy era; refusing to relabel those bytes as {}/{} digest {} before mutation",
                self.dir.display(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    /// Record the #602 typed attention-operator identity record of the
    /// teacher oracle in the observation manifest (idempotent, atomic store).
    /// Only an exact immutable registry record can be stored, and the cloned
    /// candidate replaces in-memory state only after its atomic store succeeds.
    pub fn set_attention_operator(
        &mut self,
        operator: &AttentionOperatorSpec,
    ) -> Result<(), SourceUnavailable> {
        self.preflight_attention_operator(Some(operator))?;
        recover_manifest_temp_residue(&self.dir)?;
        if self.manifest.attention_operator.as_ref() == Some(operator) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.attention_operator = Some(operator.clone());
        validate_source_execution_identity(
            candidate.attention_operator.as_ref(),
            candidate.dense_operator.as_ref(),
            "proposed observation manifest",
        )?;
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only symmetric compatibility check for the host-side dense
    /// execution era. An explicit producer may not relabel payload created
    /// before dense provenance existed, and an operatorless producer may not
    /// resume bytes already pinned to one.
    pub fn preflight_dense_operator(
        &self,
        requested: Option<&DenseOperatorSpec>,
    ) -> Result<(), SourceUnavailable> {
        if let Some(recorded) = self.manifest.dense_operator.as_ref() {
            validate_registered_source_dense_operator(recorded)?;
        }
        if let Some(requested) = requested {
            validate_registered_source_dense_operator(requested)?;
        }
        let payload_present =
            observation_payload_present(&self.dir, &self.manifest, "dense-operator")?;

        match (self.manifest.dense_operator.as_ref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to dense operator {}/{} digest {}; requested {}/{} digest {}; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to dense operator {}/{} digest {}; the requested producer declares none; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded dense operator but already contains observation payload from a legacy execution era; refusing to relabel those bytes as {}/{} digest {} before mutation",
                self.dir.display(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    /// Atomically record the exact dense execution owner declared by the
    /// teacher oracle. Registry equality and payload compatibility are checked
    /// again immediately before publication.
    pub fn set_dense_operator(
        &mut self,
        operator: &DenseOperatorSpec,
    ) -> Result<(), SourceUnavailable> {
        self.preflight_dense_operator(Some(operator))?;
        recover_manifest_temp_residue(&self.dir)?;
        if self.manifest.dense_operator.as_ref() == Some(operator) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.dense_operator = Some(operator.clone());
        validate_source_execution_identity(
            candidate.attention_operator.as_ref(),
            candidate.dense_operator.as_ref(),
            "proposed observation manifest",
        )?;
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Atomically publish the complete source-execution pair in one manifest
    /// replacement. Dense-aware producers use this instead of two setters so
    /// no crash prefix can expose learned-attention/current-dense as a
    /// plausible markerless attention-only generation.
    pub fn set_source_execution_pair(
        &mut self,
        attention_operator: Option<&AttentionOperatorSpec>,
        dense_operator: Option<&DenseOperatorSpec>,
    ) -> Result<(), SourceUnavailable> {
        validate_source_execution_identity(
            attention_operator,
            dense_operator,
            "proposed observation manifest source execution",
        )?;
        self.preflight_attention_operator(attention_operator)?;
        self.preflight_dense_operator(dense_operator)?;
        if self.manifest.attention_operator.as_ref() == attention_operator
            && self.manifest.dense_operator.as_ref() == dense_operator
        {
            return Ok(());
        }
        recover_manifest_temp_residue(&self.dir)?;
        let mut candidate = self.manifest.clone();
        candidate.attention_operator = attention_operator.cloned();
        candidate.dense_operator = dense_operator.cloned();
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Read-only symmetric compatibility check for the #603 trace-profile
    /// era. The full payload inventory gates introduction of a richer profile;
    /// orphan shards and sidecars count just like checkpoints.
    pub fn preflight_trace_profile(
        &self,
        requested: Option<&TraceProfile>,
    ) -> Result<(), SourceUnavailable> {
        if let Some(recorded) = self.manifest.trace_profile.as_ref() {
            validate_registered_trace_profile(recorded)?;
        }
        if let Some(requested) = requested {
            validate_registered_trace_profile(requested)?;
        }
        let payload_present =
            observation_payload_present(&self.dir, &self.manifest, "trace-profile")?;
        match (self.manifest.trace_profile.as_ref(), requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to trace profile {}/{}; requested {}/{}; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
                requested.id,
                requested.version,
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to trace profile {}/{}; requested the minimal/absent profile; incompatible observation resume refused before mutation",
                self.dir.display(),
                recorded.id,
                recorded.version,
            ))),
            (None, Some(requested)) if payload_present => Err(invalid_input(format!(
                "{} has no recorded trace profile but already contains observation payload from the minimal era; refusing to relabel those bytes as {}/{} before mutation",
                self.dir.display(),
                requested.id,
                requested.version,
            ))),
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    fn preflight_trace_row_bytes(&self, requested: Option<u64>) -> Result<(), SourceUnavailable> {
        match (self.manifest.trace_row_bytes, requested) {
            (Some(recorded), Some(requested)) if recorded == requested => Ok(()),
            (Some(recorded), Some(requested)) => Err(invalid_input(format!(
                "{} is pinned to {recorded}-byte trace rows; the requested oracle/profile resolves to {requested}-byte rows; incompatible trace capture geometry refused before mutation",
                self.dir.display()
            ))),
            (Some(recorded), None) => Err(invalid_input(format!(
                "{} is pinned to {recorded}-byte trace rows; the requested pass has no richer trace rows",
                self.dir.display()
            ))),
            (None, Some(_))
                if regular_file_present(&self.dir.join(RAW_COMMITTED_FILE), "trace-layout")?
                    || regular_file_present(&self.dir.join(STATE_FILE), "trace-layout")?
                    || observation_shard_payload_present(&self.dir)? =>
            {
                Err(invalid_input(format!(
                    "{} has traced stream payload but no recorded trace row width; refusing to backfill an impossible capture layout before mutation",
                    self.dir.display()
                )))
            }
            (None, Some(_)) | (None, None) => Ok(()),
        }
    }

    fn set_trace_row_bytes(&mut self, row_bytes: u64) -> Result<(), SourceUnavailable> {
        if row_bytes == 0 {
            return Err(invalid_input(
                "trace row width must be greater than zero".to_owned(),
            ));
        }
        self.preflight_trace_row_bytes(Some(row_bytes))?;
        if self.manifest.trace_row_bytes == Some(row_bytes) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.trace_row_bytes = Some(row_bytes);
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Record a non-minimal #603 trace-profile identity after compatibility
    /// succeeds. The minimal profile remains represented by absence.
    pub fn set_trace_profile(&mut self, profile: &TraceProfile) -> Result<(), SourceUnavailable> {
        self.preflight_trace_profile(Some(profile))?;
        if self.manifest.trace_profile.as_ref() == Some(profile) {
            return Ok(());
        }
        let mut candidate = self.manifest.clone();
        candidate.trace_profile = Some(profile.clone());
        candidate.store(&self.dir)?;
        self.manifest = candidate;
        Ok(())
    }

    /// Pending per-shard partition counts (records written so far via
    /// [`ObservationShardWriter::write_record_in_partition`] plus any
    /// counts restored by [`ObservationShardWriter::restore_partition_counts`]).
    pub fn partition_counts(&self, shard: u32) -> Option<PartitionCounts> {
        self.partition_counts.get(shard as usize).copied()
    }

    /// Restore per-shard partition counts from an earlier pass's
    /// checkpoint, so counts accumulated across a resume cover the whole
    /// shard rather than only this invocation's writes.
    pub fn restore_partition_counts(
        &mut self,
        counts: &[PartitionCounts],
    ) -> Result<(), SourceUnavailable> {
        if counts.len() != self.partition_counts.len() {
            return Err(invalid_input(format!(
                "partition count table has {} shards, expected {}",
                counts.len(),
                self.partition_counts.len()
            )));
        }
        self.partition_counts.copy_from_slice(counts);
        self.partitions_active = counts.iter().any(|count| count.total() != 0);
        Ok(())
    }

    fn shard_path(&self, shard: u32) -> PathBuf {
        self.dir
            .join(shard_file_name(self.manifest.shard_bits, shard))
    }

    /// Append one record to `shard`. Returns `Ok(false)` — skipping the
    /// write — when the shard is already complete; `Ok(true)` when the
    /// record was written.
    pub fn write_record(
        &mut self,
        record: &[u8; RECORD_SIZE],
        shard: u32,
    ) -> Result<bool, SourceUnavailable> {
        if self.is_complete(shard) {
            return Ok(false);
        }
        let index = shard as usize;
        if index >= self.handles.len() {
            return Err(invalid_input(format!(
                "shard {shard} is outside the configured fan-out {}",
                self.handles.len()
            )));
        }
        if self.handles[index].is_none() {
            // Append mode: bytes left by an interrupted earlier pass are
            // the deterministic prefix of this shard's stream.
            let path = self.shard_path(shard);
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            let existing = file.metadata()?.len();
            if existing % RECORD_SIZE as u64 != 0 {
                return Err(invalid_data(format!(
                    "shard file {} has a torn record ({} bytes); delete it and rerun",
                    path.display(),
                    existing
                )));
            }
            self.handles[index] = Some(ShardHandle {
                file: BufWriter::new(file),
                probability: None,
                trace: None,
            });
        }
        let handle = self.handles[index]
            .as_mut()
            .expect("shard handle opened above");
        handle.file.write_all(record)?;
        Ok(true)
    }

    /// Append one partitioned record to `shard`: identical write semantics
    /// to [`ObservationShardWriter::write_record`], additionally counting
    /// the record under its document-level partition so the finalized
    /// shard entry lists construction vs held-out counts.
    pub fn write_record_in_partition(
        &mut self,
        record: &[u8; RECORD_SIZE],
        shard: u32,
        partition: RecordPartition,
    ) -> Result<bool, SourceUnavailable> {
        let written = self.write_record(record, shard)?;
        if written {
            self.partition_counts[shard as usize].add(partition);
            self.partitions_active = true;
        }
        Ok(written)
    }

    /// Append an observation record and its aligned probability metadata.
    /// Finalization validates that every non-empty shard has a complete,
    /// aligned sidecar rather than silently publishing partial metadata.
    pub fn write_record_with_probability(
        &mut self,
        record: &[u8; RECORD_SIZE],
        probability: ProbabilityMetadata,
        shard: u32,
    ) -> Result<bool, SourceUnavailable> {
        if self.is_complete(shard) {
            return Ok(false);
        }
        let written = self.write_record(record, shard)?;
        if !written {
            return Ok(false);
        }
        let index = shard as usize;
        let path = self.dir.join(format!(
            "{}.prob",
            shard_file_name(self.manifest.shard_bits, shard)
        ));
        let handle = self.handles[index]
            .as_mut()
            .ok_or_else(|| invalid_data(format!("shard {shard} handle was not opened")))?;
        if handle.probability.is_none() {
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            let existing = file.metadata()?.len();
            if existing % PROBABILITY_METADATA_SIZE as u64 != 0 {
                return Err(invalid_data(format!(
                    "probability sidecar {} has a torn metadata row",
                    path.display()
                )));
            }
            handle.probability = Some(BufWriter::new(file));
        }
        handle
            .probability
            .as_mut()
            .expect("probability handle opened above")
            .write_all(&probability.encode())?;
        Ok(true)
    }

    /// Append an observation record, its aligned probability metadata,
    /// and its aligned #603 trace-sidecar row (the richer lanes of a
    /// non-minimal trace profile, assembled by the traced observe
    /// driver). The first traced write pins the row width in the
    /// manifest; every later row must match it, and finalization
    /// validates that every non-empty shard has a complete, aligned
    /// trace sidecar rather than silently publishing partial lanes. The
    /// primary shard record bytes are written through the unchanged v4
    /// path — richer lanes never alter them.
    pub fn write_record_with_probability_and_trace(
        &mut self,
        record: &[u8; RECORD_SIZE],
        probability: ProbabilityMetadata,
        trace_row: &[u8],
        shard: u32,
    ) -> Result<bool, SourceUnavailable> {
        if trace_row.is_empty() {
            return Err(invalid_input(
                "a trace-sidecar row must be non-empty; minimal passes use \
                 write_record_with_probability instead"
                    .to_owned(),
            ));
        }
        if self.is_complete(shard) {
            return Ok(false);
        }
        match self.manifest.trace_row_bytes {
            Some(width) if width != trace_row.len() as u64 => {
                return Err(invalid_input(format!(
                    "trace row is {} bytes but the manifest pins {width}-byte rows; \
                     the trace profile or capture geometry cannot change mid-corpus",
                    trace_row.len()
                )));
            }
            None => {
                self.set_trace_row_bytes(trace_row.len() as u64)?;
            }
            Some(_) => {}
        }
        let written = self.write_record_with_probability(record, probability, shard)?;
        if !written {
            return Ok(false);
        }
        let index = shard as usize;
        let path = self
            .dir
            .join(trace_sidecar_name(self.manifest.shard_bits, shard));
        let handle = self.handles[index]
            .as_mut()
            .ok_or_else(|| invalid_data(format!("shard {shard} handle was not opened")))?;
        if handle.trace.is_none() {
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            let existing = file.metadata()?.len();
            if existing % trace_row.len() as u64 != 0 {
                return Err(invalid_data(format!(
                    "trace sidecar {} has a torn row",
                    path.display()
                )));
            }
            handle.trace = Some(BufWriter::new(file));
        }
        handle
            .trace
            .as_mut()
            .expect("trace handle opened above")
            .write_all(trace_row)?;
        Ok(true)
    }

    /// Probability-sidecar variant of
    /// [`ObservationShardWriter::write_record_in_partition`].
    pub fn write_record_with_probability_in_partition(
        &mut self,
        record: &[u8; RECORD_SIZE],
        probability: ProbabilityMetadata,
        shard: u32,
        partition: RecordPartition,
    ) -> Result<bool, SourceUnavailable> {
        let written = self.write_record_with_probability(record, probability, shard)?;
        if written {
            self.partition_counts[shard as usize].add(partition);
            self.partitions_active = true;
        }
        Ok(written)
    }

    /// Partition-counting variant of
    /// [`ObservationShardWriter::write_record_with_probability_and_trace`]
    /// — one append carrying the record, its probability metadata, its
    /// #603 trace-sidecar row, AND the D3 partition count, for the
    /// teacher-forced text pipeline's traced passes (#605 S1).
    pub fn write_record_with_probability_and_trace_in_partition(
        &mut self,
        record: &[u8; RECORD_SIZE],
        probability: ProbabilityMetadata,
        trace_row: &[u8],
        shard: u32,
        partition: RecordPartition,
    ) -> Result<bool, SourceUnavailable> {
        let written =
            self.write_record_with_probability_and_trace(record, probability, trace_row, shard)?;
        if written {
            self.partition_counts[shard as usize].add(partition);
            self.partitions_active = true;
        }
        Ok(written)
    }

    /// Flush every open shard handle. Called at whole-story checkpoints so
    /// the on-disk shard bytes always cover exactly the completed stories
    /// of the deterministic stream.
    pub fn flush(&mut self) -> Result<(), SourceUnavailable> {
        for handle in self.handles.iter_mut().flatten() {
            handle.file.flush()?;
            if let Some(probability) = handle.probability.as_mut() {
                probability.flush()?;
            }
            if let Some(trace) = handle.trace.as_mut() {
                trace.flush()?;
            }
        }
        Ok(())
    }

    /// Durably synchronize every open raw stream, then return its strict
    /// aligned record-count vector. This is the payload half of a whole-story
    /// transaction and must complete before `raw-committed.bin` is replaced.
    fn synchronize_raw_transaction(&mut self) -> Result<Vec<u64>, SourceUnavailable> {
        self.flush()?;
        for handle in self.handles.iter().flatten() {
            handle.file.get_ref().sync_all()?;
            if let Some(probability) = handle.probability.as_ref() {
                probability.get_ref().sync_all()?;
            }
            if let Some(trace) = handle.trace.as_ref() {
                trace.get_ref().sync_all()?;
            }
        }
        // Persist any stream directory entries created since the previous
        // transaction before publishing a checkpoint that refers to them.
        sync_directory(&self.dir)?;
        current_raw_record_counts(&self.dir, &self.manifest)
    }

    /// Finalize one shard: flush, κ-pin its file, and record it in the
    /// manifest (persisted atomically). Idempotent; an untouched shard
    /// finalizes as an empty file.
    pub fn finish_shard(&mut self, shard: u32) -> Result<(), SourceUnavailable> {
        if self.is_complete(shard) {
            return Ok(());
        }
        if shard as usize >= self.handles.len() {
            return Err(invalid_input(format!(
                "shard {shard} is outside the configured fan-out {}",
                self.handles.len()
            )));
        }
        if let Some(handle) = self.handles[shard as usize].as_mut() {
            handle.file.flush()?;
            if let Some(probability) = handle.probability.as_mut() {
                probability.flush()?;
            }
            if let Some(trace) = handle.trace.as_mut() {
                trace.flush()?;
            }
        }
        let path = self.shard_path(shard);
        if !path.exists() {
            fs::write(&path, [])?;
        }
        let length = fs::metadata(&path)?.len();
        if length % RECORD_SIZE as u64 != 0 {
            return Err(invalid_data(format!(
                "shard file {} has a torn record ({} bytes); delete it and rerun",
                path.display(),
                length
            )));
        }
        let probability_path = self.dir.join(format!(
            "{}.prob",
            shard_file_name(self.manifest.shard_bits, shard)
        ));
        let probability_kappa = if probability_path.exists() {
            let probability_length = fs::metadata(&probability_path)?.len();
            let records = length / RECORD_SIZE as u64;
            if probability_length != records * PROBABILITY_METADATA_SIZE as u64 {
                return Err(invalid_data(format!(
                    "probability sidecar {} has {} bytes for {} records",
                    probability_path.display(),
                    probability_length,
                    records
                )));
            }
            Some(file_kappa(&probability_path)?)
        } else {
            None
        };
        let trace_path = self
            .dir
            .join(trace_sidecar_name(self.manifest.shard_bits, shard));
        let trace_kappa = if trace_path.exists() {
            let row_bytes = self.manifest.trace_row_bytes.ok_or_else(|| {
                invalid_data(format!(
                    "trace sidecar {} exists but the manifest pins no row width",
                    trace_path.display()
                ))
            })?;
            let trace_length = fs::metadata(&trace_path)?.len();
            let records = length / RECORD_SIZE as u64;
            if trace_length != records * row_bytes {
                return Err(invalid_data(format!(
                    "trace sidecar {} has {} bytes for {} records of {}-byte rows",
                    trace_path.display(),
                    trace_length,
                    records,
                    row_bytes
                )));
            }
            Some(file_kappa(&trace_path)?)
        } else if self.manifest.trace_row_bytes.is_some() && length != 0 {
            return Err(invalid_data(format!(
                "shard {shard} has records but no trace sidecar; the trace \
                 profile cannot change mid-corpus"
            )));
        } else {
            None
        };
        let entry = ShardEntry {
            records: length / RECORD_SIZE as u64,
            kappa: file_kappa(&path)?,
            partitions: self
                .partitions_active
                .then_some(self.partition_counts[shard as usize]),
            probability_kappa,
            trace_kappa,
        };
        self.manifest.total_records = self.manifest.total_records.saturating_add(entry.records);
        self.manifest.completed.insert(shard, entry);
        self.manifest.store(&self.dir)?;
        Ok(())
    }

    /// Finalize every shard in the fan-out (ascending shard-id order).
    pub fn finalize_all(&mut self) -> Result<(), SourceUnavailable> {
        for shard in 0..self.manifest.shard_count() {
            self.finish_shard(shard)?;
        }
        Ok(())
    }
}

/// Merge an observation directory into one record stream by reading the
/// completed shards in ascending shard-id order. The result depends only
/// on shard contents — never on the order shards were completed in.
#[cfg(not(target_arch = "wasm32"))]
pub fn merge_shards(dir: impl AsRef<Path>) -> Result<Vec<u8>, SourceUnavailable> {
    let dir = dir.as_ref();
    let manifest = ObservationManifest::load(dir)?
        .ok_or_else(|| invalid_data(format!("no observation manifest in {}", dir.display())))?;
    if manifest.shard_bits > MAX_SHARD_BITS {
        return Err(invalid_data(format!(
            "manifest shard_bits {} exceeds the maximum {MAX_SHARD_BITS}",
            manifest.shard_bits
        )));
    }
    let mut merged = Vec::new();
    for shard in 0..manifest.shard_count() {
        if !manifest.completed.contains_key(&shard) {
            continue;
        }
        let path = dir.join(shard_file_name(manifest.shard_bits, shard));
        let bytes = fs::read(&path)?;
        if bytes.len() % RECORD_SIZE != 0 {
            return Err(invalid_data(format!(
                "shard file {} has a torn record ({} bytes)",
                path.display(),
                bytes.len()
            )));
        }
        merged.extend_from_slice(&bytes);
    }
    Ok(merged)
}

/// Merge aligned probability sidecars in the same deterministic shard order
/// as [`merge_shards`].
#[cfg(not(target_arch = "wasm32"))]
pub fn merge_probability_metadata(
    dir: impl AsRef<Path>,
) -> Result<Vec<ProbabilityMetadata>, SourceUnavailable> {
    let dir = dir.as_ref();
    let manifest = ObservationManifest::load(dir)?
        .ok_or_else(|| invalid_data(format!("no observation manifest in {}", dir.display())))?;
    let mut metadata = Vec::new();
    for shard in 0..manifest.shard_count() {
        let Some(entry) = manifest.completed.get(&shard) else {
            continue;
        };
        if entry.probability_kappa.is_none() {
            if entry.records == 0 {
                continue;
            }
            return Err(invalid_data(format!(
                "shard {shard} has no probability metadata sidecar"
            )));
        }
        let path = dir.join(format!(
            "{}.prob",
            shard_file_name(manifest.shard_bits, shard)
        ));
        let bytes = fs::read(&path)?;
        if bytes.len() != entry.records as usize * PROBABILITY_METADATA_SIZE {
            return Err(invalid_data(format!(
                "probability sidecar {} is not aligned",
                path.display()
            )));
        }
        for row in bytes.chunks_exact(PROBABILITY_METADATA_SIZE) {
            metadata.push(ProbabilityMetadata::decode(row).ok_or_else(|| {
                invalid_data(format!(
                    "invalid probability metadata in {}",
                    path.display()
                ))
            })?);
        }
    }
    Ok(metadata)
}

/// Reconcile a probability sidecar to the committed observation prefix after
/// an interrupted producer pass.
#[cfg(not(target_arch = "wasm32"))]
pub fn reconcile_probability_shard(
    dir: impl AsRef<Path>,
    shard_bits: u8,
    shard: u32,
    committed_record_bytes: u64,
) -> Result<(), SourceUnavailable> {
    let path = dir
        .as_ref()
        .join(format!("{}.prob", shard_file_name(shard_bits, shard)));
    let expected = committed_record_bytes / RECORD_SIZE as u64 * PROBABILITY_METADATA_SIZE as u64;
    let length = match fs::metadata(&path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if expected == 0 {
                return Ok(());
            }
            return Err(invalid_data(format!(
                "{} is missing but the checkpoint commits probability metadata",
                path.display()
            )));
        }
        Err(error) => return Err(error.into()),
    };
    if length % PROBABILITY_METADATA_SIZE as u64 != 0 || length < expected {
        return Err(invalid_data(format!(
            "probability sidecar {} is shorter than the committed prefix",
            path.display()
        )));
    }
    if length > expected {
        let file = fs::OpenOptions::new().write(true).open(&path)?;
        file.set_len(expected)?;
    }
    Ok(())
}

/// Reconcile a generation-path record/sidecar pair after a crash between the
/// two buffered writes. This path checkpoints only at whole-story boundaries,
/// so the common prefix of the two files is the recoverable prefix.
#[cfg(not(target_arch = "wasm32"))]
pub fn reconcile_probability_pair(
    dir: impl AsRef<Path>,
    shard_bits: u8,
    shard: u32,
) -> Result<(), SourceUnavailable> {
    let dir = dir.as_ref();
    let record_path = dir.join(shard_file_name(shard_bits, shard));
    let probability_path = dir.join(format!("{}.prob", shard_file_name(shard_bits, shard)));
    let probability_length = match fs::metadata(&probability_path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if probability_length % PROBABILITY_METADATA_SIZE as u64 != 0 {
        return Err(invalid_data(format!(
            "probability sidecar {} has a torn metadata row",
            probability_path.display()
        )));
    }
    let record_length = fs::metadata(&record_path)?.len();
    if record_length % RECORD_SIZE as u64 != 0 {
        return Err(invalid_data(format!(
            "observation shard {} has a torn record",
            record_path.display()
        )));
    }
    let records = (record_length / RECORD_SIZE as u64)
        .min(probability_length / PROBABILITY_METADATA_SIZE as u64);
    let expected_record_length = records * RECORD_SIZE as u64;
    let expected_probability_length = records * PROBABILITY_METADATA_SIZE as u64;
    if record_length > expected_record_length {
        fs::OpenOptions::new()
            .write(true)
            .open(&record_path)?
            .set_len(expected_record_length)?;
    }
    if probability_length > expected_probability_length {
        fs::OpenOptions::new()
            .write(true)
            .open(&probability_path)?
            .set_len(expected_probability_length)?;
    }
    Ok(())
}

/// Merge aligned #603 trace-sidecar rows in the same deterministic shard
/// order as [`merge_shards`]. The result depends only on shard contents,
/// never on completion order; rows are returned as raw bytes (the lane
/// layout is declared by the manifest's `trace_profile` and
/// `trace_row_bytes`).
#[cfg(not(target_arch = "wasm32"))]
pub fn merge_trace_rows(dir: impl AsRef<Path>) -> Result<Vec<u8>, SourceUnavailable> {
    let dir = dir.as_ref();
    let manifest = ObservationManifest::load(dir)?
        .ok_or_else(|| invalid_data(format!("no observation manifest in {}", dir.display())))?;
    let row_bytes = manifest.trace_row_bytes.ok_or_else(|| {
        invalid_data(format!(
            "{} has no trace sidecars (minimal trace profile)",
            dir.display()
        ))
    })?;
    let mut merged = Vec::new();
    for shard in 0..manifest.shard_count() {
        let Some(entry) = manifest.completed.get(&shard) else {
            continue;
        };
        if entry.trace_kappa.is_none() {
            if entry.records == 0 {
                continue;
            }
            return Err(invalid_data(format!("shard {shard} has no trace sidecar")));
        }
        let path = dir.join(trace_sidecar_name(manifest.shard_bits, shard));
        let bytes = fs::read(&path)?;
        if bytes.len() as u64 != entry.records * row_bytes {
            return Err(invalid_data(format!(
                "trace sidecar {} is not aligned",
                path.display()
            )));
        }
        merged.extend_from_slice(&bytes);
    }
    Ok(merged)
}

/// Reconcile a generation-path record/probability/trace triple after a
/// crash between the buffered writes of one position: the common record
/// prefix of the three files is the recoverable prefix (the #603
/// extension of [`reconcile_probability_pair`], applied when the
/// manifest pins a trace row width). Records without trace rows are
/// refused — a corpus cannot silently change trace profile mid-stream.
#[cfg(not(target_arch = "wasm32"))]
pub fn reconcile_trace_triple(
    dir: impl AsRef<Path>,
    shard_bits: u8,
    shard: u32,
    trace_row_bytes: u64,
) -> Result<(), SourceUnavailable> {
    let dir = dir.as_ref();
    reconcile_probability_pair(dir, shard_bits, shard)?;
    let record_path = dir.join(shard_file_name(shard_bits, shard));
    let record_length = match fs::metadata(&record_path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let trace_path = dir.join(trace_sidecar_name(shard_bits, shard));
    let trace_length = match fs::metadata(&trace_path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if record_length == 0 {
                return Ok(());
            }
            return Err(invalid_data(format!(
                "shard {shard} has records but no trace sidecar; the trace \
                 profile cannot change mid-corpus"
            )));
        }
        Err(error) => return Err(error.into()),
    };
    let records = (record_length / RECORD_SIZE as u64).min(trace_length / trace_row_bytes.max(1));
    let targets = [
        (record_path, records * RECORD_SIZE as u64),
        (
            dir.join(format!("{}.prob", shard_file_name(shard_bits, shard))),
            records * PROBABILITY_METADATA_SIZE as u64,
        ),
        (trace_path, records * trace_row_bytes),
    ];
    for (path, expected) in targets {
        let length = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if length > expected {
            fs::OpenOptions::new()
                .write(true)
                .open(&path)?
                .set_len(expected)?;
        }
    }
    Ok(())
}

/// Outcome of one [`observe_sharded`] invocation.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserveSummary {
    /// Tokens generated so far by the underlying teacher stream.
    pub records: u64,
    /// Stories started so far by the underlying teacher stream.
    pub stories: u64,
    /// Records written during this invocation (excludes skipped complete
    /// shards and records already on disk from earlier invocations).
    pub written: u64,
    /// Records routed to already-complete shards and therefore skipped.
    pub skipped: u64,
    /// Whether the target was reached and every shard is κ-pinned.
    pub done: bool,
}

/// The byte layout of one #603 trace-sidecar row — a pure function of the
/// trace profile and an oracle's declared capture geometry. This is the
/// SINGLE source of truth for the lane order and widths that
/// [`TraceCapture`] assembles and every reader decodes, so a layout change
/// cannot drift between the writer and a consumer: both derive it here.
///
/// Lane order (all little-endian; an absent lane contributes zero bytes):
/// per-layer residuals (ascending declared layers) · final-hidden (#95) ·
/// per-layer q/k/v (ascending, q then k then v) · attention support
/// (ascending layers, heads ascending, `support` fixed slots of
/// `(u32 position, f32 weight)`, unfilled slots = [`SUPPORT_ABSENT_MARKER`]).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceRowLayout {
    /// Number of captured per-layer residual streams.
    pub residual_layers: usize,
    /// Whether the final-hidden (#95) lane is present.
    pub final_hidden: bool,
    /// Number of captured q/k/v layers.
    pub qkv_layers: usize,
    /// Number of captured attention-support layers.
    pub attention_layers: usize,
    /// Attention heads per layer.
    pub heads: usize,
    /// Per-head attention-support slot count (the declared cap).
    pub support: usize,
    /// Residual-stream / q width.
    pub residual_width: usize,
    /// k and v width (grouped-query: `residual_width * kv_heads / heads`).
    pub kv_width: usize,
    /// Total row width in bytes (pinned into the manifest at first write).
    pub row_bytes: usize,
}

/// One decoded trace-sidecar row: the structured lanes a consumer reads.
/// Absent lanes are empty (never zero-filled); an absent attention-support
/// slot is dropped (marker-aware), never surfaced as a zero-valued entry.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRow {
    /// Per captured residual layer: the `residual_width` residual stream.
    pub residual: Vec<Vec<f32>>,
    /// The final-hidden (#95) state, when the lane is present.
    pub final_hidden: Option<Vec<f32>>,
    /// Per captured q/k/v layer: `(q[residual_width], k[kv_width], v[kv_width])`.
    pub qkv: Vec<(Vec<f32>, Vec<f32>, Vec<f32>)>,
    /// Per captured attention layer, per head: the present support entries
    /// `(position, weight)` in stored order (absent slots dropped).
    pub support: Vec<Vec<Vec<(u32, f32)>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TraceRowLayout {
    fn checked_row_bytes(&self) -> Result<usize, SourceUnavailable> {
        let residual_bytes = self
            .residual_layers
            .checked_add(usize::from(self.final_hidden))
            .and_then(|lanes| lanes.checked_mul(self.residual_width))
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| invalid_data("trace residual lane width overflows usize".to_owned()))?;
        let qkv_vector_width = self
            .kv_width
            .checked_mul(2)
            .and_then(|width| self.residual_width.checked_add(width))
            .ok_or_else(|| invalid_data("trace q/k/v lane width overflows usize".to_owned()))?;
        let qkv_bytes = self
            .qkv_layers
            .checked_mul(qkv_vector_width)
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| invalid_data("trace q/k/v bytes overflow usize".to_owned()))?;
        let attention_bytes = self
            .attention_layers
            .checked_mul(self.heads)
            .and_then(|rows| rows.checked_mul(self.support))
            .and_then(|slots| slots.checked_mul(8))
            .ok_or_else(|| invalid_data("trace attention bytes overflow usize".to_owned()))?;
        residual_bytes
            .checked_add(qkv_bytes)
            .and_then(|width| width.checked_add(attention_bytes))
            .ok_or_else(|| invalid_data("trace row width overflows usize".to_owned()))
    }

    fn try_new(
        profile: &TraceProfile,
        geometry: &uor_r4_model_source::TraceCaptureGeometry,
    ) -> Result<Self, SourceUnavailable> {
        if geometry.heads == 0
            || geometry.kv_heads == 0
            || geometry.residual_width == 0
            || geometry.kv_heads > geometry.heads
        {
            return Err(invalid_input(format!(
                "invalid trace capture geometry: heads={}, kv_heads={}, residual_width={}",
                geometry.heads, geometry.kv_heads, geometry.residual_width
            )));
        }
        // Resolve the row layout from a trace profile and capture geometry —
        // the same widths `TraceCapture` pins and every reader expects.
        let residual_layers = profile
            .layer_lane
            .as_ref()
            .map_or(0, |lane| lane.layer_indices.len());
        let final_hidden = profile
            .layer_lane
            .as_ref()
            .is_some_and(|lane| lane.final_hidden);
        let qkv_layers = profile
            .qkv_lane
            .as_ref()
            .map_or(0, |lane| lane.layer_indices.len());
        let (attention_layers, support) = profile
            .attention_support_lane
            .as_ref()
            .map_or((0, 0), |lane| {
                (lane.layer_indices.len(), lane.support_size as usize)
            });
        let residual_width = geometry.residual_width;
        let kv_numerator = residual_width
            .checked_mul(geometry.kv_heads)
            .ok_or_else(|| invalid_input("trace k/v width overflows usize".to_owned()))?;
        if kv_numerator % geometry.heads != 0 {
            return Err(invalid_input(
                "trace capture geometry does not define an integral k/v width".to_owned(),
            ));
        }
        let kv_width = kv_numerator / geometry.heads;
        let residual_bytes = residual_layers
            .checked_add(usize::from(final_hidden))
            .and_then(|lanes| lanes.checked_mul(residual_width))
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| invalid_input("trace residual lane width overflows usize".to_owned()))?;
        let qkv_vector_width = kv_width
            .checked_mul(2)
            .and_then(|width| residual_width.checked_add(width))
            .ok_or_else(|| invalid_input("trace q/k/v lane width overflows usize".to_owned()))?;
        let qkv_bytes = qkv_layers
            .checked_mul(qkv_vector_width)
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| invalid_input("trace q/k/v bytes overflow usize".to_owned()))?;
        let attention_bytes = attention_layers
            .checked_mul(geometry.heads)
            .and_then(|rows| rows.checked_mul(support))
            .and_then(|slots| slots.checked_mul(8))
            .ok_or_else(|| invalid_input("trace attention bytes overflow usize".to_owned()))?;
        let row_bytes = residual_bytes
            .checked_add(qkv_bytes)
            .and_then(|width| width.checked_add(attention_bytes))
            .ok_or_else(|| invalid_input("trace row width overflows usize".to_owned()))?;
        Ok(Self {
            residual_layers,
            final_hidden,
            qkv_layers,
            attention_layers,
            heads: geometry.heads,
            support,
            residual_width,
            kv_width,
            row_bytes,
        })
    }

    /// Resolve the row layout from a trace profile and capture geometry —
    /// the same widths [`TraceCapture`] pins and every reader expects. Invalid
    /// external geometry produces a sentinel-width layout; mutation paths use
    /// the checked constructor and return a focused error instead.
    pub fn new(
        profile: &TraceProfile,
        geometry: &uor_r4_model_source::TraceCaptureGeometry,
    ) -> Self {
        Self::try_new(profile, geometry).unwrap_or(Self {
            residual_layers: 0,
            final_hidden: false,
            qkv_layers: 0,
            attention_layers: 0,
            heads: geometry.heads,
            support: 0,
            residual_width: geometry.residual_width,
            kv_width: 0,
            row_bytes: usize::MAX,
        })
    }

    /// Decode one `row_bytes`-long row into its structured lanes. Errors if
    /// the slice length does not match the pinned width.
    pub fn read_row(&self, row: &[u8]) -> Result<TraceRow, SourceUnavailable> {
        let expected_row_bytes = self.checked_row_bytes()?;
        if self.row_bytes != expected_row_bytes {
            return Err(invalid_data(format!(
                "trace layout pins {} row bytes, but its lane fields require {expected_row_bytes}",
                self.row_bytes
            )));
        }
        if row.len() != expected_row_bytes {
            return Err(invalid_data(format!(
                "trace row is {} bytes, layout pins {}",
                row.len(),
                expected_row_bytes
            )));
        }
        let read_f32 = |offset: usize| {
            f32::from_le_bytes([
                row[offset],
                row[offset + 1],
                row[offset + 2],
                row[offset + 3],
            ])
        };
        let read_u32 = |offset: usize| {
            u32::from_le_bytes([
                row[offset],
                row[offset + 1],
                row[offset + 2],
                row[offset + 3],
            ])
        };
        let mut offset = 0usize;
        let take_vec = |offset: &mut usize, width: usize| {
            let mut out = Vec::with_capacity(width);
            for i in 0..width {
                out.push(read_f32(*offset + i * 4));
            }
            *offset += width * 4;
            out
        };
        let mut residual = Vec::with_capacity(self.residual_layers);
        for _ in 0..self.residual_layers {
            residual.push(take_vec(&mut offset, self.residual_width));
        }
        let final_hidden = if self.final_hidden {
            Some(take_vec(&mut offset, self.residual_width))
        } else {
            None
        };
        let mut qkv = Vec::with_capacity(self.qkv_layers);
        for _ in 0..self.qkv_layers {
            let q = take_vec(&mut offset, self.residual_width);
            let k = take_vec(&mut offset, self.kv_width);
            let v = take_vec(&mut offset, self.kv_width);
            qkv.push((q, k, v));
        }
        let mut support = Vec::with_capacity(self.attention_layers);
        for _ in 0..self.attention_layers {
            let mut per_head = Vec::with_capacity(self.heads);
            for _ in 0..self.heads {
                let mut entries = Vec::with_capacity(self.support);
                for slot in 0..self.support {
                    let position = read_u32(offset + slot * 8);
                    let weight_bits = read_u32(offset + slot * 8 + 4);
                    match (
                        position == SUPPORT_ABSENT_MARKER,
                        weight_bits == SUPPORT_ABSENT_MARKER,
                    ) {
                        (true, true) => {
                            // Explicit absence marker: the slot is absent,
                            // never a zero-valued entry.
                            continue;
                        }
                        (true, false) | (false, true) => {
                            return Err(invalid_data(
                                "trace support slot has a partial absence marker".to_owned(),
                            ));
                        }
                        (false, false) => {}
                    }
                    entries.push((position, f32::from_bits(weight_bits)));
                }
                offset += self.support * 8;
                per_head.push(entries);
            }
            support.push(per_head);
        }
        Ok(TraceRow {
            residual,
            final_hidden,
            qkv,
            support,
        })
    }
}

/// The bounded per-step capture state of one traced observation pass
/// (#603): the resolved lane plan of a non-minimal [`TraceProfile`]
/// against one oracle's declared capture geometry, and the assembly
/// buffer for one trace-sidecar row. Lane order within a row is fixed
/// and documented: per-layer residuals (ascending declared layers), the
/// final-hidden state (#95 lane), q/k/v rows (ascending declared
/// layers, q then k then v), then attention support (ascending declared
/// layers, heads ascending, S fixed slots of `(u32 position, f32
/// weight)` — unfilled slots carry the explicit
/// [`SUPPORT_ABSENT_MARKER`], never zeros). All values little-endian
/// f32/u32 bytes; the row width is a pure function of (profile,
/// geometry), pinned into the manifest at the first write.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct TraceCapture {
    pub(crate) profile: TraceProfile,
    residual_layers: Vec<usize>,
    qkv_layers: Vec<usize>,
    attention_layers: Vec<usize>,
    support: usize,
    final_hidden: bool,
    residual_width: usize,
    pub(crate) row_bytes: usize,
    pub(crate) row: Vec<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TraceCapture {
    /// Resolve `profile` against `oracle`'s declared capture geometry.
    /// Refused — never zero-filled — when the oracle exposes no capture
    /// surface, no hidden state for a declared final-hidden lane, or a
    /// declared layer index outside its layer range.
    pub(crate) fn new(
        profile: &TraceProfile,
        oracle: &dyn TeacherOracle,
    ) -> Result<Self, SourceUnavailable> {
        validate_registered_trace_profile(profile)?;
        let geometry = oracle.trace_capture_geometry().ok_or_else(|| {
            invalid_input(format!(
                "trace profile {}/{} needs the oracle's bounded capture surface, \
                 but this oracle declares none; richer lanes are refused, not zero-filled",
                profile.id, profile.version
            ))
        })?;
        let resolve = |indices: &[u32], lane: &str| -> Result<Vec<usize>, SourceUnavailable> {
            let mut resolved = Vec::with_capacity(indices.len());
            for &index in indices {
                if index as usize >= geometry.layers {
                    return Err(invalid_input(format!(
                        "{lane} capture layer {index} is outside the oracle's {} layers",
                        geometry.layers
                    )));
                }
                resolved.push(index as usize);
            }
            Ok(resolved)
        };
        let (residual_layers, final_hidden) = match &profile.layer_lane {
            Some(lane) => (resolve(&lane.layer_indices, "residual")?, lane.final_hidden),
            None => (Vec::new(), false),
        };
        if final_hidden && oracle.hidden_state().is_none() {
            return Err(invalid_input(format!(
                "trace profile {}/{} declares the final-hidden lane, but this \
                 oracle retains no hidden state; the lane is refused, not zero-filled",
                profile.id, profile.version
            )));
        }
        let qkv_layers = match &profile.qkv_lane {
            Some(lane) => resolve(&lane.layer_indices, "q/k/v")?,
            None => Vec::new(),
        };
        let (attention_layers, support) = match &profile.attention_support_lane {
            Some(lane) => (
                resolve(&lane.layer_indices, "attention-support")?,
                lane.support_size as usize,
            ),
            None => (Vec::new(), 0),
        };
        // Single source of truth for the row width — the same layout every
        // reader derives, so the writer and consumers cannot drift.
        let row_bytes = TraceRowLayout::try_new(profile, &geometry)?.row_bytes;
        if row_bytes == 0 {
            return Err(invalid_input(format!(
                "trace profile {}/{} declares no captured bytes; use the minimal profile",
                profile.id, profile.version
            )));
        }
        Ok(Self {
            profile: profile.clone(),
            residual_layers,
            qkv_layers,
            attention_layers,
            support,
            final_hidden,
            residual_width: geometry.residual_width,
            row_bytes,
            row: Vec::with_capacity(row_bytes),
        })
    }

    /// One traced teacher step: capture the declared lanes through the
    /// oracle's exact-executor capture path and assemble this position's
    /// trace-sidecar row into `self.row`. Deterministic: the row is a
    /// pure function of (oracle state, token, pos, profile).
    pub(crate) fn step(
        &mut self,
        oracle: &mut dyn TeacherOracle,
        token: usize,
        pos: usize,
        logits: &mut [f32],
    ) -> Result<(), SourceUnavailable> {
        let mut residual_bytes: Vec<u8> = Vec::new();
        let mut qkv_bytes: Vec<u8> = Vec::new();
        let mut attention_bytes: Vec<u8> = Vec::new();
        let support = self.support;
        let captured = {
            let mut residual_sink = |_layer: usize, x: &[f32]| {
                for value in x {
                    residual_bytes.extend_from_slice(&value.to_le_bytes());
                }
            };
            let mut qkv_sink = |_layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
                for vector in [q, k, v] {
                    for value in vector {
                        qkv_bytes.extend_from_slice(&value.to_le_bytes());
                    }
                }
            };
            let mut attention_sink = |_layer: usize, _head: usize, att: &[f32]| {
                // Bounded top-S support: positions ordered by descending
                // weight, ties to the lower position (the same canonical
                // tie-break as the top-k trace surface).
                let mut order: Vec<u32> = (0..att.len() as u32).collect();
                order.sort_by(|a, b| {
                    att[*b as usize]
                        .total_cmp(&att[*a as usize])
                        .then_with(|| a.cmp(b))
                });
                for slot in 0..support {
                    match order.get(slot) {
                        Some(&position) => {
                            attention_bytes.extend_from_slice(&position.to_le_bytes());
                            attention_bytes
                                .extend_from_slice(&att[position as usize].to_le_bytes());
                        }
                        None => {
                            // Fewer prefix positions than the cap: mark
                            // the slot explicitly absent — never a
                            // zero-filled entry.
                            attention_bytes.extend_from_slice(&SUPPORT_ABSENT_MARKER.to_le_bytes());
                            attention_bytes.extend_from_slice(&SUPPORT_ABSENT_MARKER.to_le_bytes());
                        }
                    }
                }
            };
            oracle.step_with_trace_capture(
                token,
                pos,
                logits,
                &TraceCaptureRequest {
                    residual_layers: &self.residual_layers,
                    qkv_layers: &self.qkv_layers,
                    attention_layers: &self.attention_layers,
                },
                &mut TraceCaptureSinks {
                    residual: &mut residual_sink,
                    qkv: &mut qkv_sink,
                    attention: &mut attention_sink,
                },
            )
        };
        if !captured {
            return Err(invalid_input(format!(
                "trace profile {}/{} needs the oracle's exact-executor capture \
                 step, but this oracle performs only plain steps; richer lanes \
                 are refused, not zero-filled",
                self.profile.id, self.profile.version
            )));
        }
        self.row.clear();
        self.row.extend_from_slice(&residual_bytes);
        if self.final_hidden {
            let hidden = oracle.hidden_state().ok_or_else(|| {
                invalid_data("oracle hidden state disappeared mid-pass".to_owned())
            })?;
            if hidden.len() != self.residual_width {
                return Err(invalid_data(format!(
                    "hidden state is {} wide, expected the capture geometry's {}",
                    hidden.len(),
                    self.residual_width
                )));
            }
            for value in hidden {
                self.row.extend_from_slice(&value.to_le_bytes());
            }
        }
        self.row.extend_from_slice(&qkv_bytes);
        self.row.extend_from_slice(&attention_bytes);
        if self.row.len() != self.row_bytes {
            return Err(invalid_data(format!(
                "assembled trace row is {} bytes, expected {} — the oracle's \
                 captures do not match its declared geometry",
                self.row.len(),
                self.row_bytes
            )));
        }
        Ok(())
    }
}

/// #603 profile pinning, read-only half: a corpus is captured under ONE
/// profile. A recorded profile must match the requested one (minimal
/// requests refuse traced corpora and vice versa once bytes exist); a
/// fresh traced pass returns the profile to record before any record is
/// written. Shared by the generation driver and the teacher-forced text
/// driver so the two cannot drift.
#[cfg(not(target_arch = "wasm32"))]
fn preflight_trace_profile_pin(
    writer: &ObservationShardWriter,
    trace: Option<&TraceCapture>,
    out: &Path,
) -> Result<Option<TraceProfile>, SourceUnavailable> {
    writer.preflight_trace_profile(trace.map(|trace| &trace.profile))?;
    writer.preflight_trace_row_bytes(trace.map(|trace| trace.row_bytes as u64))?;
    let recorded = writer.manifest().trace_profile.clone();
    match (&recorded, trace) {
        (None, None) => Ok(None),
        (Some(recorded), Some(trace)) if *recorded == trace.profile => Ok(None),
        (None, Some(trace)) => Ok(Some(trace.profile.clone())),
        (Some(recorded), Some(trace)) => Err(invalid_input(format!(
            "{} is pinned to trace profile {}/{}; requested {}/{}",
            out.display(),
            recorded.id,
            recorded.version,
            trace.profile.id,
            trace.profile.version
        ))),
        (Some(recorded), None) => Err(invalid_input(format!(
            "{} is pinned to trace profile {}/{}; pass the same profile to resume",
            out.display(),
            recorded.id,
            recorded.version
        ))),
    }
}

/// [`preflight_trace_profile_pin`] plus the write half — record the
/// profile and pin the row width — in one call, for drivers (the
/// teacher-forced text pipeline) whose other identity records are
/// published elsewhere.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn pin_writer_trace_profile(
    writer: &mut ObservationShardWriter,
    trace: Option<&TraceCapture>,
    out: &Path,
) -> Result<(), SourceUnavailable> {
    let to_set = preflight_trace_profile_pin(writer, trace, out)?;
    if let Some(profile) = to_set.as_ref() {
        writer.set_trace_profile(profile)?;
    }
    if let Some(trace) = trace {
        writer.set_trace_row_bytes(trace.row_bytes as u64)?;
    }
    Ok(())
}

/// Run the teacher generation of `compile_hugging_face`'s corpus step,
/// spilling v4 records plus aligned probability metadata into content-addressed
/// shards instead of one
/// append-only corpus file. The teacher stream is the same deterministic
/// stream (seed 0x5EED, whole-story checkpointing); each record is routed
/// to `shard_of(sample_id(context_window), shard_bits)`, where the context
/// window is the existing 8-token window of fed tokens. Resume continues the
/// stream from a strictly decoded `state.bin`, skips completed shards, and
/// appends to checkpoint-trimmed incomplete shards. `raw-committed.bin` is the
/// authoritative whole-story boundary; the legacy `state.bin` is repaired as
/// its compatibility mirror. Aligned bytes past the committed vector are
/// tentative and are regenerated exactly once.
///
/// This is the minimal trace profile: byte-for-byte today's records,
/// sidecars, and manifest (no `trace_profile` field is recorded). Richer
/// #603 profiles are opt-in through [`observe_sharded_traced`].
#[cfg(not(target_arch = "wasm32"))]
pub fn observe_sharded(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    shard_bits: u8,
    out: &Path,
    token_byte_lengths: Option<&[u32]>,
) -> Result<ObserveSummary, SourceUnavailable> {
    observe_sharded_inner(
        oracle,
        budget_s,
        target,
        shard_bits,
        out,
        token_byte_lengths,
        None,
        None,
    )
}

/// [`observe_sharded`] while retaining a caller-owned exclusive observation
/// session across earlier identity pinning and tokenizer export.
#[cfg(not(target_arch = "wasm32"))]
pub fn observe_sharded_in_session(
    session: &ObservationSession,
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    token_byte_lengths: Option<&[u32]>,
) -> Result<ObserveSummary, SourceUnavailable> {
    observe_sharded_inner(
        oracle,
        budget_s,
        target,
        session.shard_bits(),
        session.dir(),
        token_byte_lengths,
        None,
        Some(session),
    )
}

/// [`observe_sharded`] under an explicit #603 teacher-trace profile: the
/// SAME pipeline (same deterministic stream, records, sharding,
/// checkpointing, and resume), extended — when the profile is not
/// minimal — with one aligned trace-sidecar row per record carrying the
/// declared richer lanes, captured through the oracle's exact-executor
/// capture step within the profile's declared bounds. Passing the
/// minimal profile is exactly [`observe_sharded`]. Deterministic: the
/// same inputs and profile produce byte-identical shard and sidecar
/// bytes; a corpus's profile is pinned in its manifest and cannot change
/// mid-stream.
#[cfg(not(target_arch = "wasm32"))]
pub fn observe_sharded_traced(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    shard_bits: u8,
    out: &Path,
    token_byte_lengths: Option<&[u32]>,
    profile: &TraceProfile,
) -> Result<ObserveSummary, SourceUnavailable> {
    validate_registered_trace_request(profile)?;
    let trace = if profile.is_minimal() {
        None
    } else {
        Some(TraceCapture::new(profile, oracle)?)
    };
    observe_sharded_inner(
        oracle,
        budget_s,
        target,
        shard_bits,
        out,
        token_byte_lengths,
        trace,
        None,
    )
}

/// [`observe_sharded_traced`] under a caller-owned exclusive observation
/// session.
#[cfg(not(target_arch = "wasm32"))]
pub fn observe_sharded_traced_in_session(
    session: &ObservationSession,
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    token_byte_lengths: Option<&[u32]>,
    profile: &TraceProfile,
) -> Result<ObserveSummary, SourceUnavailable> {
    validate_registered_trace_request(profile)?;
    let trace = if profile.is_minimal() {
        None
    } else {
        Some(TraceCapture::new(profile, oracle)?)
    };
    observe_sharded_inner(
        oracle,
        budget_s,
        target,
        session.shard_bits(),
        session.dir(),
        token_byte_lengths,
        trace,
        Some(session),
    )
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn observe_sharded_inner(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    shard_bits: u8,
    out: &Path,
    token_byte_lengths: Option<&[u32]>,
    mut trace: Option<TraceCapture>,
    session: Option<&ObservationSession>,
) -> Result<ObserveSummary, SourceUnavailable> {
    // Resolve and registry-validate the producer's actual arithmetic identity
    // before even opening a new output directory. The writer then performs the
    // symmetric resume/payload check before trace reconciliation or any other
    // stateful operation.
    let geometry = oracle.geometry_projection();
    if let Some(geometry) = geometry.as_ref() {
        validate_registered_geometry_projection(geometry)?;
    }
    let attention_operator = oracle.attention_operator_spec();
    if let Some(operator) = attention_operator.as_ref() {
        validate_registered_source_attention_operator(operator)?;
    }
    let dense_operator = oracle.dense_operator_spec();
    if let Some(operator) = dense_operator.as_ref() {
        validate_registered_source_dense_operator(operator)?;
    }
    validate_source_execution_identity(
        attention_operator.as_ref(),
        dense_operator.as_ref(),
        "teacher-declared observation source execution",
    )?;
    let mut writer = match session {
        Some(session) => session.writer()?,
        None => ObservationShardWriter::open(out, shard_bits)?,
    };
    writer.preflight_geometry(geometry.as_ref())?;
    writer.preflight_attention_operator(attention_operator.as_ref())?;
    writer.preflight_dense_operator(dense_operator.as_ref())?;
    let preflight_state = preflight_observation_state(out, writer.manifest())?;
    let requested_target = u64::try_from(target).unwrap_or(u64::MAX);
    if preflight_state.is_some_and(|(n, _, _, _)| n < requested_target)
        && !writer.manifest().completed.is_empty()
    {
        return Err(invalid_input(format!(
            "{} has finalized observation shards at a smaller raw target; extending a finalized corpus would skip rows routed to those shards, so use a fresh output directory",
            out.display()
        )));
    }
    // #603 profile pinning (shared with the teacher-forced text driver via
    // `preflight_trace_profile_pin`, so the two drivers cannot drift): a
    // corpus is captured under ONE profile; mismatches refuse before any
    // stateful operation.
    let trace_profile_to_set = preflight_trace_profile_pin(&writer, trace.as_ref(), out)?;
    // Every identity check above is read-only. Only after all of them agree do
    // we publish either missing record.
    if let Some(geometry) = geometry.as_ref() {
        writer.set_geometry(geometry)?;
    }
    if let Some(operator) = attention_operator.as_ref() {
        writer.set_attention_operator(operator)?;
    }
    if let Some(operator) = dense_operator.as_ref() {
        writer.set_dense_operator(operator)?;
    }
    if let Some(profile) = trace_profile_to_set.as_ref() {
        writer.set_trace_profile(profile)?;
    }
    if let Some(trace) = trace.as_ref() {
        writer.set_trace_row_bytes(trace.row_bytes as u64)?;
    }
    // Re-read the complete checkpoint/payload snapshot after every identity
    // and layout setter has landed. Fresh runs publish the authoritative empty
    // prefix before reconciliation or the first append. Existing runs apply
    // the all-shard plan only after its read-only phase succeeded in full.
    let resume = preflight_raw_resume(out, writer.manifest())?;
    let restored_state = match &resume {
        RawResumePlan::Fresh => {
            let checkpoint = RawCommittedCheckpoint::new(
                writer.manifest(),
                0,
                0,
                0x5EED,
                false,
                vec![0; writer.manifest().shard_count() as usize],
            )?;
            publish_raw_checkpoint(out, &checkpoint)?;
            publish_state_mirror(out, &checkpoint)?;
            Some((0, 0, 0x5EED, 0))
        }
        RawResumePlan::Authoritative(plan) => {
            apply_raw_recovery(out, plan)?;
            let (n, stories, rng, done) = plan.checkpoint.state();
            Some((n, stories, rng, u8::from(done)))
        }
        RawResumePlan::FinalizedLegacy(state) => Some(*state),
    };
    let trace_row_bytes = writer.manifest().trace_row_bytes;
    for shard in 0..writer.manifest().shard_count() {
        match (trace.as_ref(), trace_row_bytes) {
            (Some(_), Some(row_bytes)) => {
                reconcile_trace_triple(out, shard_bits, shard, row_bytes)?;
            }
            _ => reconcile_probability_pair(out, shard_bits, shard)?,
        }
    }
    let (mut n, mut stories, mut rng, mut done) = restored_state.unwrap_or((0, 0, 0x5EED, 0));
    if (n as usize) < target {
        done = 0;
    }
    if done == 1 {
        // The stream already reached its target; make sure a crash between
        // the last checkpoint and finalization did not leave shards
        // unpinned, then stop without touching completed shard files.
        writer.finalize_all()?;
        println!("observation corpus already complete: {n} tokens");
        return Ok(ObserveSummary {
            records: n,
            stories,
            written: 0,
            skipped: 0,
            done: true,
        });
    }
    let vocab = oracle.vocab();
    let seq_len = oracle.seq_len();
    let mut logits = vec![0f32; vocab];
    let mut progress = Progress::new("observations", target);
    progress.set(n as usize);
    let mut window: Vec<u32> = Vec::with_capacity(compiler::WINDOW);
    let mut written = 0u64;
    let mut skipped = 0u64;
    let t0 = std::time::Instant::now();
    while done == 0 && t0.elapsed().as_secs() < budget_s {
        oracle.reset();
        let mut token = oracle.bos_token();
        let mut story_byte_offset = 0u32;
        window.clear();
        for pos in 0..seq_len {
            progress.set(n as usize);
            match trace.as_mut() {
                None => oracle.step(token, pos, &mut logits),
                Some(trace) => trace.step(oracle, token, pos, &mut logits)?,
            }
            let (next, top_tokens, top_weights, stats) =
                compiler::softmax_top8_sample_with_stats(&mut logits, &mut rng);
            window.push(token as u32);
            if window.len() > compiler::WINDOW {
                window.remove(0);
            }
            let id = sample_id(&window);
            let shard = shard_of(&id, shard_bits);
            let span_start = pos as u32;
            let span_end = span_start.saturating_add(1);
            let (byte_start, byte_end) =
                compiler::byte_anchors(token_byte_lengths, story_byte_offset, next);
            let record = compiler::encode_v4_record(
                stories as u32,
                next as u32,
                &top_tokens,
                &top_weights,
                (span_start, span_end),
                (byte_start, byte_end),
            );
            let probability = ProbabilityMetadata {
                target_logprob_nats: stats.target_logprob_nats,
                entropy_bits: stats.entropy_bits,
                top8_mass: stats.top8_mass,
                target_rank: stats.target_rank,
            };
            let wrote = match trace.as_ref() {
                None => writer.write_record_with_probability(&record, probability, shard)?,
                Some(trace) => writer.write_record_with_probability_and_trace(
                    &record,
                    probability,
                    &trace.row,
                    shard,
                )?,
            };
            if wrote {
                written += 1;
            } else {
                skipped += 1;
            }
            if token_byte_lengths.is_some() {
                story_byte_offset = byte_end;
            }
            n += 1;
            progress.set(n as usize);
            if n as usize >= target {
                done = 1;
                break;
            }
            if next == oracle.eos_token() {
                break;
            }
            token = next;
        }
        stories += 1;
        // Whole-story transaction: payload synchronization, authoritative
        // per-shard checkpoint replacement, then the legacy state mirror.
        let committed_records = writer.synchronize_raw_transaction()?;
        let checkpoint = RawCommittedCheckpoint::new(
            writer.manifest(),
            n,
            stories,
            rng,
            done == 1,
            committed_records,
        )?;
        publish_raw_checkpoint(out, &checkpoint)?;
        publish_state_mirror(out, &checkpoint)?;
    }
    if done == 1 {
        writer.finalize_all()?;
        progress.finish();
    }
    println!(
        "observations: {} / {} tokens, {} stories, {}/{} shards complete, done={}",
        n,
        target,
        stories,
        writer.manifest().completed.len(),
        writer.manifest().shard_count(),
        done
    );
    Ok(ObserveSummary {
        records: n,
        stories,
        written,
        skipped,
        done: done == 1,
    })
}

// ---------------------------------------------------------------------------
// #645 round-trip test for the shared TraceRowLayout reader: a row assembled
// exactly as TraceCapture writes it (front-loaded support entries, trailing
// slots as the explicit absence marker) must decode back to the same
// structured lanes through read_row — the guarantee the writer and every
// consumer cannot drift.
// ---------------------------------------------------------------------------
#[cfg(all(test, not(target_arch = "wasm32")))]
mod trace_row_tests {
    use super::*;

    /// Encode a row byte-for-byte the way the observe driver assembles it.
    fn encode(
        layout: &TraceRowLayout,
        residual: &[Vec<f32>],
        hidden: &Option<Vec<f32>>,
        qkv: &[(Vec<f32>, Vec<f32>, Vec<f32>)],
        support: &[Vec<Vec<(u32, f32)>>],
    ) -> Vec<u8> {
        let mut row = Vec::with_capacity(layout.row_bytes);
        for lane in residual {
            for &v in lane {
                row.extend_from_slice(&v.to_le_bytes());
            }
        }
        if let Some(h) = hidden {
            for &v in h {
                row.extend_from_slice(&v.to_le_bytes());
            }
        }
        for (q, k, v) in qkv {
            for vector in [q, k, v] {
                for &x in vector {
                    row.extend_from_slice(&x.to_le_bytes());
                }
            }
        }
        for layer in support {
            for head in layer {
                for slot in 0..layout.support {
                    match head.get(slot) {
                        Some(&(pos, weight)) => {
                            row.extend_from_slice(&pos.to_le_bytes());
                            row.extend_from_slice(&weight.to_le_bytes());
                        }
                        None => {
                            row.extend_from_slice(&SUPPORT_ABSENT_MARKER.to_le_bytes());
                            row.extend_from_slice(&SUPPORT_ABSENT_MARKER.to_le_bytes());
                        }
                    }
                }
            }
        }
        row
    }

    #[test]
    fn trace_row_round_trips_through_the_shared_reader() {
        // All lanes present; GQA (kv_width < residual_width); a support cap
        // of 3 with heads carrying fewer real entries, exercising the
        // absence marker in the trailing slots.
        let (residual_layers, qkv_layers, attention_layers) = (2usize, 2usize, 2usize);
        let (heads, support, residual_width, kv_width) = (2usize, 3usize, 4usize, 2usize);
        let row_bytes = residual_layers * residual_width * 4
            + residual_width * 4
            + qkv_layers * (residual_width + 2 * kv_width) * 4
            + attention_layers * heads * support * 8;
        let layout = TraceRowLayout {
            residual_layers,
            final_hidden: true,
            qkv_layers,
            attention_layers,
            heads,
            support,
            residual_width,
            kv_width,
            row_bytes,
        };

        let residual = vec![vec![1.0f32, 2.0, 3.0, 4.0], vec![5.0, 6.0, 7.0, 8.0]];
        let hidden = Some(vec![9.0f32, 10.0, 11.0, 12.0]);
        let qkv = vec![
            (vec![0.1f32, 0.2, 0.3, 0.4], vec![0.5, 0.6], vec![0.7, 0.8]),
            (vec![1.1, 1.2, 1.3, 1.4], vec![1.5, 1.6], vec![1.7, 1.8]),
        ];
        let support_entries: Vec<Vec<Vec<(u32, f32)>>> = vec![
            vec![vec![(0, 0.9f32), (2, 0.1)], vec![(1, 1.0)]],
            vec![vec![(0, 0.5), (1, 0.3), (2, 0.2)], vec![(3, 0.4), (0, 0.6)]],
        ];

        let row = encode(&layout, &residual, &hidden, &qkv, &support_entries);
        assert_eq!(row.len(), layout.row_bytes);

        let decoded = layout.read_row(&row).expect("decode a well-formed row");
        assert_eq!(decoded.residual, residual);
        assert_eq!(decoded.final_hidden, hidden);
        assert_eq!(decoded.qkv, qkv);
        assert_eq!(decoded.support, support_entries);

        // A wrong-length row is rejected, never silently mis-decoded.
        assert!(layout.read_row(&row[..row.len() - 1]).is_err());
        assert!(layout.read_row(&[]).is_err());
    }

    #[test]
    fn trace_reader_rejects_inconsistent_layouts_and_partial_absence_markers() {
        let inconsistent = TraceRowLayout {
            residual_layers: 1,
            final_hidden: false,
            qkv_layers: 0,
            attention_layers: 0,
            heads: 1,
            support: 0,
            residual_width: 1,
            kv_width: 1,
            row_bytes: 0,
        };
        let error = inconsistent
            .read_row(&[])
            .expect_err("public layout fields must not authorize out-of-bounds decoding");
        assert!(error.reason.contains("lane fields require"), "{error}");

        let support_layout = TraceRowLayout {
            residual_layers: 0,
            final_hidden: false,
            qkv_layers: 0,
            attention_layers: 1,
            heads: 1,
            support: 1,
            residual_width: 1,
            kv_width: 1,
            row_bytes: 8,
        };
        for (label, position, weight_bits) in [
            ("position-only", SUPPORT_ABSENT_MARKER, 1.0f32.to_bits()),
            ("weight-only", 0, SUPPORT_ABSENT_MARKER),
        ] {
            let mut row = Vec::with_capacity(8);
            row.extend_from_slice(&position.to_le_bytes());
            row.extend_from_slice(&weight_bits.to_le_bytes());
            let error = support_layout
                .read_row(&row)
                .expect_err("half of an absence marker must not become a support entry");
            assert!(
                error.reason.contains("partial absence marker"),
                "{label}: {error}"
            );
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod attention_operator_resume_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Barrier, mpsc};
    use std::time::Duration;

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-attention-resume-{label}-{}-{}",
                std::process::id(),
                NEXT_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn bytes(&self) -> Vec<(String, Vec<u8>)> {
            let mut files = fs::read_dir(&self.0)
                .expect("read test directory")
                .map(|entry| entry.expect("read directory entry").path())
                .filter(|path| path.is_file())
                .map(|path| {
                    (
                        path.file_name()
                            .expect("file name")
                            .to_string_lossy()
                            .into_owned(),
                        fs::read(path).expect("read file bytes"),
                    )
                })
                .collect::<Vec<_>>();
            files.sort_by(|left, right| left.0.cmp(&right.0));
            files
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn public_writer_refuses_symlinked_valid_manifest_without_mutation() {
        use std::os::unix::fs::symlink;

        let dir = TestDir::new("manifest-symlink");
        let target_name = "valid-manifest-target.json";
        fs::write(
            dir.path().join(target_name),
            serde_json::to_vec_pretty(&ObservationManifest::new(1))
                .expect("serialize valid manifest"),
        )
        .expect("write valid manifest target");
        let link = dir.path().join(MANIFEST_FILE);
        symlink(target_name, &link).expect("create manifest symlink");
        let before = dir.bytes();
        let link_target_before = fs::read_link(&link).expect("read manifest link");

        let error = match ObservationShardWriter::open(dir.path(), 1) {
            Ok(_) => panic!("public writer followed a manifest symlink"),
            Err(error) => error,
        };
        assert!(error.reason.contains("not a regular file"), "{error}");
        assert_eq!(dir.bytes(), before, "refusal changed directory bytes");
        assert_eq!(
            fs::read_link(&link).expect("reread manifest link"),
            link_target_before,
            "refusal replaced the manifest symlink"
        );
    }

    #[test]
    fn public_writer_rejects_unknown_nested_provenance_claims_without_mutation() {
        for (label, field, mut manifest) in [
            ("attention", "attention_operator", {
                let mut manifest = ObservationManifest::new(1);
                manifest.attention_operator = Some(AttentionOperatorSpec::standard());
                manifest
            }),
            ("geometry", "geometry", {
                let mut manifest = ObservationManifest::new(1);
                manifest.geometry = Some(GeometryProjection::bucket_average(4, 2));
                manifest
            }),
            ("tokenizer", "tokenizer_adapter", {
                let mut manifest = ObservationManifest::new(1);
                manifest.tokenizer_adapter = Some(test_tokenizer_adapter());
                manifest
            }),
            ("trace", "trace_profile", {
                let mut manifest = ObservationManifest::new(1);
                manifest.trace_profile = Some(TraceProfile::layer(&[0]));
                manifest
            }),
        ] {
            let dir = TestDir::new(&format!("manifest-extra-{label}-claim"));
            let mut value = serde_json::to_value(&mut manifest).expect("serialize manifest value");
            value
                .get_mut(field)
                .and_then(serde_json::Value::as_object_mut)
                .expect("nested provenance object")
                .insert(
                    "unregistered_claim".to_owned(),
                    serde_json::Value::Bool(true),
                );
            fs::write(
                dir.path().join(MANIFEST_FILE),
                serde_json::to_vec_pretty(&value).expect("serialize malformed manifest"),
            )
            .expect("write malformed manifest");
            let before = dir.bytes();

            let error = match ObservationShardWriter::open(dir.path(), 1) {
                Ok(_) => panic!("writer accepted an unregistered nested {label} claim"),
                Err(error) => error,
            };
            assert!(
                error.reason.contains("unregistered_claim"),
                "{label}: {error}"
            );
            assert_eq!(
                dir.bytes(),
                before,
                "{label} refusal changed manifest bytes"
            );
        }
    }

    #[test]
    fn public_writer_rejects_unknown_manifest_semantics_without_mutation() {
        for (label, mutate) in [
            (
                "future-schema",
                Box::new(|value: &mut serde_json::Value| value["schema"] = 2.into())
                    as Box<dyn Fn(&mut serde_json::Value)>,
            ),
            (
                "unknown-root-claim",
                Box::new(|value: &mut serde_json::Value| {
                    value
                        .as_object_mut()
                        .expect("manifest object")
                        .insert("future_semantics".to_owned(), true.into());
                }),
            ),
            (
                "out-of-range-fanout",
                Box::new(|value: &mut serde_json::Value| value["shard_bits"] = 9.into()),
            ),
        ] {
            let dir = TestDir::new(label);
            let mut value =
                serde_json::to_value(ObservationManifest::new(1)).expect("serialize manifest");
            mutate(&mut value);
            fs::write(
                dir.path().join(MANIFEST_FILE),
                serde_json::to_vec_pretty(&value).expect("serialize adversarial manifest"),
            )
            .expect("write adversarial manifest");
            let before = dir.bytes();

            let error = match ObservationShardWriter::open(dir.path(), 1) {
                Ok(_) => panic!("writer accepted {label}"),
                Err(error) => error,
            };
            assert!(
                error.reason.contains("schema")
                    || error.reason.contains("unknown field")
                    || error.reason.contains("shard_bits"),
                "{label}: {error}"
            );
            assert_eq!(dir.bytes(), before, "{label} refusal changed bytes");
        }

        let dir = TestDir::new("malformed-recorded-source-kappa");
        let mut manifest = ObservationManifest::new(1);
        manifest.source_manifest_kappa = Some("blake3:not-a-digest".to_owned());
        fs::write(
            dir.path().join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("serialize malformed manifest"),
        )
        .expect("write malformed manifest");
        let before = dir.bytes();
        let error = match ObservationShardWriter::open(dir.path(), 1) {
            Ok(_) => panic!("writer accepted malformed recorded source kappa"),
            Err(error) => error,
        };
        assert!(error.reason.contains("source manifest"), "{error}");
        assert_eq!(dir.bytes(), before);

        let dir = TestDir::new("malformed-recorded-input-cid");
        let mut manifest = ObservationManifest::new(1);
        manifest.input_cid = Some("blake3:not-a-digest".to_owned());
        fs::write(
            dir.path().join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("serialize malformed manifest"),
        )
        .expect("write malformed manifest");
        let before = dir.bytes();
        let error = match ObservationShardWriter::open(dir.path(), 1) {
            Ok(_) => panic!("writer accepted malformed recorded input CID"),
            Err(error) => error,
        };
        assert!(error.reason.contains("input CID"), "{error}");
        assert_eq!(dir.bytes(), before);

        let fresh = TestDir::new("malformed-requested-input-cid");
        let before = fresh.bytes();
        let mut writer = ObservationShardWriter::open(fresh.path(), 1).expect("open fresh writer");
        let error = writer
            .set_input_cid("not-a-cid")
            .expect_err("malformed requested input CID must fail");
        assert!(error.reason.contains("input CID"), "{error}");
        assert_eq!(fresh.bytes(), before);
    }

    #[test]
    fn trace_layout_is_fail_closed_before_reconciliation() {
        let profile = TraceProfile::layer(&[0]);
        let dir = TestDir::new("trace-width-mismatch");
        let mut manifest = ObservationManifest::new(1);
        manifest.trace_profile = Some(profile.clone());
        manifest.trace_row_bytes = Some(16);
        fs::write(
            dir.path().join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("serialize traced manifest"),
        )
        .expect("write traced manifest");
        let before = dir.bytes();
        let writer = ObservationShardWriter::open(dir.path(), 1).expect("open traced writer");
        let error = writer
            .preflight_trace_row_bytes(Some(32))
            .expect_err("different capture geometry must fail before reconciliation");
        assert!(error.reason.contains("trace rows"), "{error}");
        assert_eq!(dir.bytes(), before);

        let missing_width = TestDir::new("trace-width-missing-with-payload");
        let mut manifest = ObservationManifest::new(1);
        manifest.trace_profile = Some(profile);
        fs::write(
            missing_width.path().join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("serialize traced manifest"),
        )
        .expect("write traced manifest");
        fs::write(missing_width.path().join("shard-00.bin"), []).expect("write shard-era evidence");
        let before = missing_width.bytes();
        let error = match ObservationShardWriter::open(missing_width.path(), 1) {
            Ok(_) => panic!("trace profile without row width accepted over shard payload"),
            Err(error) => error,
        };
        assert!(error.reason.contains("no trace row width"), "{error}");
        assert_eq!(missing_width.bytes(), before);

        let unregistered = TestDir::new("unregistered-trace-before-capture");
        let mut adversarial = TraceProfile::attention_support(&[0], 1);
        adversarial
            .attention_support_lane
            .as_mut()
            .expect("support lane")
            .support_size = u32::MAX;
        adversarial.declared_digest = adversarial.declared_digest();
        let mut oracle = DeclaredOperatorOracle {
            operator: Some(AttentionOperatorSpec::standard()),
            geometry: None,
        };
        let before = unregistered.bytes();
        let error = observe_sharded_traced(
            &mut oracle,
            60,
            1,
            1,
            unregistered.path(),
            None,
            &adversarial,
        )
        .expect_err("unregistered trace bounds must fail before capture allocation");
        assert!(
            error.reason.contains("support") || error.reason.contains("trace profile"),
            "{error}"
        );
        assert_eq!(unregistered.bytes(), before);
    }

    #[test]
    fn public_writer_rejects_target_and_noncanonical_manifest_operators() {
        for (label, operator) in [
            (
                "target-route",
                AttentionOperatorSpec::r4_route_attention_v1(),
            ),
            ("tampered", {
                let mut operator = AttentionOperatorSpec::standard();
                operator.value_aggregation = "tampered-value-fold".to_owned();
                operator
            }),
            ("unknown", {
                let mut operator = AttentionOperatorSpec::standard();
                operator.version = 999;
                operator.implementation_digest = operator.declared_digest();
                operator
            }),
        ] {
            let dir = TestDir::new(label);
            let mut manifest = ObservationManifest::new(1);
            manifest.attention_operator = Some(operator);
            fs::write(
                dir.path().join(MANIFEST_FILE),
                serde_json::to_vec_pretty(&manifest).expect("serialize adversarial manifest"),
            )
            .expect("write adversarial manifest");
            let before = dir.bytes();

            let error = match ObservationShardWriter::open(dir.path(), 1) {
                Ok(_) => panic!("writer accepted {label} source provenance"),
                Err(error) => error,
            };
            assert!(
                error.reason.contains("attention operator")
                    || error.reason.contains("unknown attention"),
                "{label}: {error}"
            );
            assert_eq!(dir.bytes(), before, "{label} refusal changed bytes");
            assert!(
                !dir.path().join("shard-00.bin").exists(),
                "{label} refusal created a shard"
            );
        }
    }

    #[test]
    fn regular_manifest_temp_crash_residue_is_recovered_before_pinning() {
        let dir = TestDir::new("manifest-temp-recovery");
        let fixed = dir.path().join(".manifest.json.tmp");
        let unique = dir.path().join(".manifest.json.tmp-123-7");
        fs::write(&fixed, []).expect("write zero-byte fixed crash residue");
        fs::write(&unique, b"{\"schema\":").expect("write truncated unique crash residue");

        let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
        writer
            .set_attention_operator(&AttentionOperatorSpec::standard())
            .expect("unpublished regular temps do not establish an era");

        assert!(!fixed.exists(), "fixed crash residue was not recovered");
        assert!(!unique.exists(), "unique crash residue was not recovered");
        assert_eq!(
            writer.manifest().attention_operator.as_ref(),
            Some(&AttentionOperatorSpec::standard())
        );
    }

    #[cfg(unix)]
    #[test]
    fn manifest_temp_symlink_is_refused_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let dir = TestDir::new("manifest-temp-symlink");
        let external = TestDir::new("manifest-temp-external");
        let target = external.path().join("sentinel");
        fs::write(&target, b"external sentinel").expect("write external target");
        let link = dir.path().join(".manifest.json.tmp");
        symlink(&target, &link).expect("plant manifest temp symlink");
        let before = dir.bytes();
        let target_before = fs::read(&target).expect("read target before refusal");

        let error = match ObservationShardWriter::open(dir.path(), 1) {
            Ok(_) => panic!("writer accepted a manifest-temp symlink"),
            Err(error) => error,
        };
        assert!(error.reason.contains("not a regular file"), "{error}");
        assert_eq!(dir.bytes(), before, "refusal changed directory bytes");
        assert_eq!(
            fs::read(&target).expect("read target after refusal"),
            target_before,
            "refusal followed the temp symlink"
        );
        assert_eq!(fs::read_link(&link).expect("temp symlink remains"), target);
    }

    #[cfg(unix)]
    #[test]
    fn matching_operator_resume_refuses_shard_symlink_without_mutation() {
        use std::os::unix::fs::symlink;

        let dir = TestDir::new("matching-shard-symlink");
        let standard = AttentionOperatorSpec::standard();
        {
            let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
            writer
                .set_attention_operator(&standard)
                .expect("pin standard operator");
        }
        let external = TestDir::new("matching-shard-external");
        let target = external.path().join("sentinel");
        fs::write(&target, vec![0x5a; RECORD_SIZE * 2]).expect("write external shard target");
        let link = dir.path().join("shard-00.bin");
        symlink(&target, &link).expect("plant shard symlink");
        let before = dir.bytes();
        let target_before = fs::read(&target).expect("read target before refusal");
        let mut producer = DeclaredOperatorOracle {
            operator: Some(standard),
            geometry: None,
        };

        let error = observe_sharded(&mut producer, 0, 1, 1, dir.path(), None)
            .expect_err("matching identity must still reject a shard symlink");
        assert!(error.reason.contains("not a regular file"), "{error}");
        assert_eq!(dir.bytes(), before, "refusal changed observation bytes");
        assert_eq!(
            fs::read(&target).expect("read target after refusal"),
            target_before,
            "refusal followed or truncated the shard symlink"
        );
        assert_eq!(fs::read_link(&link).expect("shard symlink remains"), target);
    }

    #[test]
    fn setter_accepts_only_exact_registry_records_and_is_failure_atomic() {
        for (label, operator) in [
            ("tampered", {
                let mut operator = AttentionOperatorSpec::standard();
                operator.value_aggregation = "tampered-value-fold".to_owned();
                operator
            }),
            ("unknown", {
                let mut operator = AttentionOperatorSpec::standard();
                operator.version = 999;
                operator.implementation_digest = operator.declared_digest();
                operator
            }),
            (
                "target-route",
                AttentionOperatorSpec::r4_route_attention_v1(),
            ),
        ] {
            let dir = TestDir::new(label);
            let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
            let error = writer
                .set_attention_operator(&operator)
                .expect_err("only exact source provenance may be stored");
            if label == "target-route" {
                assert!(error.reason.contains("not an allowed source"), "{error}");
            }
            assert!(dir.bytes().is_empty(), "{label} refusal wrote output");
        }

        for operator in [
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::standard_v2(),
            AttentionOperatorSpec::experimental_r4_v1(),
            AttentionOperatorSpec::experimental_r4_v2(),
            AttentionOperatorSpec::learned_absolute_v1(),
            AttentionOperatorSpec::learned_absolute_v2(),
        ] {
            let dir = TestDir::new(&operator.id);
            let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
            writer
                .set_attention_operator(&operator)
                .expect("every registered source operator is admissible");
            assert_eq!(
                writer.manifest().attention_operator.as_ref(),
                Some(&operator)
            );
        }

        let dir = TestDir::new("registered");
        let standard = AttentionOperatorSpec::standard();
        let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
        writer
            .set_attention_operator(&standard)
            .expect("pin registered operator");
        let pinned = dir.bytes();
        writer
            .set_attention_operator(&standard)
            .expect("identical record is idempotent");
        assert_eq!(dir.bytes(), pinned);

        let error = writer
            .set_attention_operator(&AttentionOperatorSpec::experimental_r4())
            .expect_err("different registered era must be refused");
        assert!(error.reason.contains("incompatible observation resume"));
        assert_eq!(
            writer.manifest().attention_operator.as_ref(),
            Some(&standard)
        );
        assert_eq!(dir.bytes(), pinned, "mismatch changed directory bytes");

        let same_family = TestDir::new("standard-v1-to-v2");
        let v1 = AttentionOperatorSpec::standard_v1();
        let v2 = AttentionOperatorSpec::standard_v2();
        let mut writer =
            ObservationShardWriter::open(same_family.path(), 1).expect("open same-family writer");
        writer
            .set_attention_operator(&v1)
            .expect("pin immutable v1 record");
        let pinned = same_family.bytes();
        let error = writer
            .set_attention_operator(&v2)
            .expect_err("v2 cannot resume a v1 observation directory");
        assert!(error.reason.contains("incompatible observation resume"));
        assert_eq!(writer.manifest().attention_operator.as_ref(), Some(&v1));
        assert_eq!(
            same_family.bytes(),
            pinned,
            "v1-to-v2 refusal changed observation bytes"
        );
    }

    #[test]
    fn every_tokenizer_era_payload_marker_blocks_attention_relabelling() {
        let shard = shard_file_name(1, 0);
        let payload_names = [
            STATE_FILE.to_owned(),
            "merged.bin".to_owned(),
            "committed.bin".to_owned(),
            ".committed.bin.tmp".to_owned(),
            "stories.jsonl".to_owned(),
            "stories.tmp".to_owned(),
            "stories.tmp-123-0".to_owned(),
            "tokenizer.bin".to_owned(),
            shard.clone(),
            format!("{shard}.prob"),
            format!("{shard}.trace"),
        ];
        for name in payload_names {
            let dir = TestDir::new(&name.replace('.', "-"));
            fs::write(dir.path().join(&name), b"legacy payload").expect("write payload marker");
            let before = dir.bytes();
            let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
            let error = writer
                .set_attention_operator(&AttentionOperatorSpec::standard())
                .expect_err("legacy payload cannot be relabelled");
            assert!(
                error.reason.contains("implicit legacy era"),
                "{name}: {error}"
            );
            assert_eq!(dir.bytes(), before, "{name} refusal changed bytes");
            assert_eq!(writer.manifest().attention_operator, None);
        }

        let dir = TestDir::new("manifest-count");
        let mut manifest = ObservationManifest::new(1);
        manifest.total_records = 1;
        manifest.store(dir.path()).expect("store legacy manifest");
        let before = dir.bytes();
        let error = match ObservationShardWriter::open(dir.path(), 1) {
            Ok(_) => panic!("inconsistent manifest record count was accepted"),
            Err(error) => error,
        };
        assert!(error.reason.contains("total_records"), "{error}");
        assert_eq!(dir.bytes(), before);
    }

    #[test]
    fn zero_byte_payload_entries_still_pin_the_legacy_operator_era() {
        for name in [STATE_FILE, "shard-00.bin"] {
            let dir = TestDir::new(&format!("zero-{}", name.replace('.', "-")));
            fs::write(dir.path().join(name), []).expect("write zero-byte crash residue");
            let before = dir.bytes();
            let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");

            let error = writer
                .set_attention_operator(&AttentionOperatorSpec::standard())
                .expect_err("payload entry presence must pin the legacy era");
            assert!(error.reason.contains("implicit legacy era"), "{error}");
            assert_eq!(dir.bytes(), before, "{name} refusal changed bytes");
        }
    }

    #[test]
    fn stripped_manifest_cannot_hide_out_of_fanout_shard_payload() {
        // Requested shard_bits=1 implies only shards 00 and 01. These files
        // came from a wider historical fan-out whose manifest was stripped.
        for name in ["shard-02.bin", "shard-02.bin.prob", "shard-02.bin.trace"] {
            let dir = TestDir::new(&format!("hidden-{}", name.replace('.', "-")));
            fs::write(dir.path().join(name), b"legacy hidden row")
                .expect("write hidden shard payload");
            let before = dir.bytes();

            let error = match ObservationShardWriter::open(dir.path(), 1) {
                Ok(_) => panic!("narrow writer accepted out-of-fanout payload"),
                Err(error) => error,
            };
            assert!(error.reason.contains("outside"), "{error}");
            assert_eq!(dir.bytes(), before, "{name} refusal changed legacy bytes");
            assert!(
                !dir.path().join(MANIFEST_FILE).exists(),
                "{name} refusal relabelled the stripped manifest"
            );
        }
    }

    fn test_tokenizer_adapter() -> TokenizerAdapter {
        let mut adapter = TokenizerAdapter {
            family: TokenizerAdapter::HF_BYTE_BPE_FAMILY.to_owned(),
            version: TokenizerAdapter::HF_BYTE_BPE_VERSION,
            tokenizer_cid: format!("blake3:{}", blake3::hash(b"test-tokenizer").to_hex()),
            policy: Default::default(),
            adapter_digest: String::new(),
        };
        adapter.adapter_digest = adapter.declared_digest();
        adapter
    }

    #[test]
    fn observation_session_rejects_any_planned_output_residue_before_root_mutation() {
        let dir = TestDir::new("observation-planned-output-preflight");
        let residue = dir.path().join(format!(
            "{}{}--77.3.writing",
            crate::recorded_corpus::PLANNED_OUTPUT_RESERVED_PREFIX,
            crate::recorded_corpus::PlannedOutputMember::Records.stable_name()
        ));
        fs::write(&residue, b"partial compile output").expect("planned-output residue");
        let before = dir.bytes();

        let error = match ObservationSession::acquire(dir.path(), 1) {
            Ok(_) => panic!("observation sessions own an empty planned-output scope"),
            Err(error) => error,
        };
        assert!(error.reason.contains("foreign member"), "{error}");
        assert_eq!(dir.bytes(), before, "preflight refusal mutated the root");
        assert!(!dir.path().join(MANIFEST_FILE).exists());
    }

    #[test]
    fn one_session_never_issues_stale_or_concurrent_writers() {
        let dir = TestDir::new("single-writer-lease");
        let session = ObservationSession::acquire(dir.path(), 1).expect("acquire session");
        let adapter = test_tokenizer_adapter();
        let mut first = session.writer().expect("open first writer");
        first
            .set_tokenizer_adapter(&adapter)
            .expect("pin tokenizer through first writer");

        let error = match session.writer() {
            Ok(_) => panic!("session issued a stale second writer"),
            Err(error) => error,
        };
        assert!(
            error.reason.contains("already has a live writer"),
            "{error}"
        );
        drop(first);

        let mut second = session.writer().expect("reload after first writer drops");
        assert_eq!(
            second.manifest().tokenizer_adapter.as_ref(),
            Some(&adapter),
            "fresh writer did not reload the first identity update"
        );
        second
            .set_attention_operator(&AttentionOperatorSpec::standard())
            .expect("pin operator without erasing tokenizer");
        second
            .write_record(&[0u8; RECORD_SIZE], 0)
            .expect("write one shard row");
        second.flush().expect("flush shard row");
        assert!(
            session.writer().is_err(),
            "session issued a same-shard writer while one was live"
        );
        drop(second);

        let manifest = ObservationManifest::load(dir.path())
            .expect("reload manifest")
            .expect("manifest exists");
        assert_eq!(manifest.tokenizer_adapter.as_ref(), Some(&adapter));
        assert_eq!(
            manifest.attention_operator.as_ref(),
            Some(&AttentionOperatorSpec::standard())
        );
        assert_eq!(
            fs::metadata(dir.path().join("shard-00.bin"))
                .expect("shard exists")
                .len(),
            RECORD_SIZE as u64
        );
    }

    #[test]
    fn full_session_lock_prevents_cross_field_mixed_era_publication() {
        let dir = TestDir::new("cross-field-session-race");
        let session_a = ObservationSession::acquire(dir.path(), 1).expect("acquire session A");
        let mut writer_a = session_a.writer().expect("open session A writer");
        let standard = AttentionOperatorSpec::standard();
        writer_a
            .preflight_tokenizer_adapter(None)
            .expect("checkpoint tokenizer half is fresh");
        writer_a
            .preflight_attention_operator(Some(&standard))
            .expect("checkpoint operator half is fresh");

        let barrier = std::sync::Arc::new(Barrier::new(2));
        let (sender, receiver) = mpsc::channel();
        let thread_dir = dir.path().to_path_buf();
        let thread_barrier = std::sync::Arc::clone(&barrier);
        let adapter = test_tokenizer_adapter();
        let adapter_for_thread = adapter.clone();
        let worker = std::thread::spawn(move || {
            thread_barrier.wait();
            let result = (|| -> Result<(), SourceUnavailable> {
                let session_b = ObservationSession::acquire(&thread_dir, 1)?;
                let mut writer_b = session_b.writer()?;
                let experimental = AttentionOperatorSpec::experimental_r4();
                // Jointly check both halves before either setter. Once A has
                // committed a row, B must fail here and publish nothing.
                writer_b.preflight_tokenizer_adapter(Some(&adapter_for_thread))?;
                writer_b.preflight_attention_operator(Some(&experimental))?;
                writer_b.set_tokenizer_adapter(&adapter_for_thread)?;
                writer_b.set_attention_operator(&experimental)?;
                writer_b.write_record(&[1u8; RECORD_SIZE], 0)?;
                Ok(())
            })();
            sender
                .send(result.map_err(|error| error.reason))
                .expect("send worker result");
        });
        barrier.wait();
        assert!(
            receiver.recv_timeout(Duration::from_millis(100)).is_err(),
            "session B entered while session A still held the OS lock"
        );

        writer_a
            .set_attention_operator(&standard)
            .expect("session A pins standard operator");
        writer_a
            .write_record(&[0u8; RECORD_SIZE], 0)
            .expect("session A writes its row");
        writer_a.flush().expect("flush session A row");
        drop(writer_a);
        drop(session_a);

        let error = receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("session B resumes after A drops")
            .expect_err("session B must refuse the mixed identity pair");
        assert!(
            error.contains("tokenizer adapter") || error.contains("attention operator"),
            "{error}"
        );
        worker.join().expect("worker thread joins");

        let manifest = ObservationManifest::load(dir.path())
            .expect("load final manifest")
            .expect("manifest exists");
        assert_eq!(
            manifest.tokenizer_adapter, None,
            "loser added its tokenizer"
        );
        assert_eq!(manifest.attention_operator.as_ref(), Some(&standard));
        assert_eq!(
            fs::metadata(dir.path().join("shard-00.bin"))
                .expect("winner shard exists")
                .len(),
            RECORD_SIZE as u64,
            "loser appended a row"
        );
        assert!(
            !dir.path().join("tokenizer.bin").exists(),
            "loser exported tokenizer bytes"
        );
    }

    #[test]
    fn observation_outer_lock_precedes_common_guard_and_busy_unwinds_outer() {
        let dir = TestDir::new("outer-before-common-order");
        let thread_dir = dir.path().to_path_buf();
        let (outer_held_tx, outer_held_rx) = mpsc::channel();
        let release = std::sync::Arc::new(Barrier::new(2));
        let thread_release = std::sync::Arc::clone(&release);
        let worker = std::thread::spawn(move || {
            acquire_observation_session_guards_with_hook(&thread_dir, || {
                outer_held_tx.send(()).expect("announce outer lock");
                thread_release.wait();
            })
        });

        outer_held_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("worker acquired outer observation lock");
        let common = crate::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(dir.path())
            .expect("worker has not inverted order by acquiring common first");
        release.wait();
        let error = worker
            .join()
            .expect("worker joins")
            .expect_err("common contender is BUSY");
        assert!(error.reason.contains("BUSY"), "{error}");
        drop(common);

        let outer_retry = acquire_observation_session_lock(dir.path())
            .expect("BUSY path unwound its outer observation lock");
        drop(outer_retry);
    }

    struct DeclaredOperatorOracle {
        operator: Option<AttentionOperatorSpec>,
        geometry: Option<GeometryProjection>,
    }

    impl uor_r4_model_source::RepresentationSource for DeclaredOperatorOracle {
        fn vocab_size(&self) -> usize {
            2
        }
        fn source_dimension(&self) -> usize {
            1
        }
        fn tokenizer_address(&self) -> &str {
            "test-tokenizer"
        }
        fn read_embedding_rows(
            &self,
            _range: std::ops::Range<usize>,
            output: &mut [f32],
        ) -> Option<()> {
            output.fill(0.0);
            Some(())
        }
    }

    impl uor_r4_model_source::BehaviorSource for DeclaredOperatorOracle {
        fn reset(&mut self) {}
        fn step(&mut self, _token: usize, _pos: usize, logits: &mut [f32]) {
            logits.fill(0.0);
        }
    }

    impl TeacherOracle for DeclaredOperatorOracle {
        fn vocab(&self) -> usize {
            2
        }
        fn dim(&self) -> usize {
            1
        }
        fn seq_len(&self) -> usize {
            1
        }
        fn kappa(&self) -> String {
            "blake3:test-source".to_owned()
        }
        fn source_bytes(&self) -> usize {
            1
        }
        fn embedding(&self, _token: usize, out: &mut [f32]) {
            out.fill(0.0);
        }
        fn attention_operator_spec(&self) -> Option<AttentionOperatorSpec> {
            self.operator.clone()
        }
        fn geometry_projection(&self) -> Option<GeometryProjection> {
            self.geometry.clone()
        }
    }

    #[test]
    fn raw_sharded_path_binds_actual_operator_and_refuses_symmetric_resume() {
        let dir = TestDir::new("raw-path");
        let standard = AttentionOperatorSpec::standard();
        let mut producer = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: None,
        };
        observe_sharded(&mut producer, 0, 1, 1, dir.path(), None)
            .expect("fresh raw path binds the operator before generation");
        let manifest = ObservationManifest::load(dir.path())
            .expect("load manifest")
            .expect("manifest exists");
        assert_eq!(manifest.attention_operator.as_ref(), Some(&standard));
        let before = dir.bytes();

        let mut undeclared = DeclaredOperatorOracle {
            operator: None,
            geometry: None,
        };
        let error = observe_sharded(&mut undeclared, 0, 1, 1, dir.path(), None)
            .expect_err("explicit-to-none resume must fail");
        assert!(error.reason.contains("declares none"), "{error}");
        assert_eq!(dir.bytes(), before);

        let mut different = DeclaredOperatorOracle {
            operator: Some(AttentionOperatorSpec::experimental_r4()),
            geometry: None,
        };
        observe_sharded(&mut different, 0, 1, 1, dir.path(), None)
            .expect_err("different explicit era must fail");
        assert_eq!(dir.bytes(), before);
    }

    #[test]
    fn raw_path_binds_geometry_and_refuses_both_absence_directions() {
        let geometry = GeometryProjection::bucket_average(4, 2);
        let standard = AttentionOperatorSpec::standard();

        let projected_dir = TestDir::new("raw-geometry-projected");
        let mut projected = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: Some(geometry.clone()),
        };
        observe_sharded(&mut projected, 60, 1, 1, projected_dir.path(), None)
            .expect("projected raw pass");
        let projected_manifest = ObservationManifest::load(projected_dir.path())
            .expect("load projected manifest")
            .expect("projected manifest exists");
        assert_eq!(projected_manifest.geometry.as_ref(), Some(&geometry));
        let projected_before = projected_dir.bytes();
        let error = observe_sharded(&mut projected, 60, 2, 1, projected_dir.path(), None)
            .expect_err("a finalized raw corpus cannot be extended in place");
        assert!(
            error.reason.contains("finalized observation shards"),
            "{error}"
        );
        assert_eq!(projected_dir.bytes(), projected_before);
        let mut pass_through = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: None,
        };
        let error = observe_sharded(&mut pass_through, 0, 1, 1, projected_dir.path(), None)
            .expect_err("pass-through geometry cannot resume projected bytes");
        assert!(error.reason.contains("geometry"), "{error}");
        assert_eq!(projected_dir.bytes(), projected_before);

        let legacy_dir = TestDir::new("raw-geometry-pass-through");
        let mut legacy = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: None,
        };
        observe_sharded(&mut legacy, 60, 1, 1, legacy_dir.path(), None)
            .expect("pass-through raw pass");
        let legacy_before = legacy_dir.bytes();
        let mut relabel = DeclaredOperatorOracle {
            operator: Some(standard),
            geometry: Some(geometry),
        };
        let error = observe_sharded(&mut relabel, 0, 1, 1, legacy_dir.path(), None)
            .expect_err("projected geometry cannot relabel pass-through payload");
        assert!(error.reason.contains("no recorded geometry"), "{error}");
        assert_eq!(legacy_dir.bytes(), legacy_before);
    }

    #[test]
    fn raw_path_refuses_malformed_present_checkpoint_before_mutation() {
        for (label, bytes) in [
            ("zero", Vec::new()),
            ("truncated", vec![0u8; 24]),
            ("invalid-done", {
                let mut bytes = vec![0u8; 25];
                bytes[24] = 2;
                bytes
            }),
            ("stories-outside-wire-domain", {
                let mut bytes = vec![0u8; 25];
                bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
                bytes
            }),
        ] {
            let dir = TestDir::new(&format!("raw-state-{label}"));
            {
                let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
                writer
                    .set_attention_operator(&AttentionOperatorSpec::standard())
                    .expect("pin source operator");
            }
            fs::write(dir.path().join(STATE_FILE), bytes).expect("write malformed state");
            let before = dir.bytes();
            let mut oracle = DeclaredOperatorOracle {
                operator: Some(AttentionOperatorSpec::standard()),
                geometry: None,
            };

            let error = observe_sharded(&mut oracle, 60, 1, 1, dir.path(), None)
                .expect_err("present malformed checkpoint must not mean fresh");
            assert!(error.reason.contains("checkpoint"), "{label}: {error}");
            assert_eq!(dir.bytes(), before, "{label} refusal changed bytes");
        }
    }

    #[test]
    fn raw_state_pairing_is_validated_before_identity_or_reconciliation() {
        let standard = AttentionOperatorSpec::standard();
        {
            let marker = "merged.bin";
            let dir = TestDir::new("raw-missing-state-stream-evidence");
            {
                let mut writer = ObservationShardWriter::open(dir.path(), 1).expect("open writer");
                writer
                    .set_attention_operator(&standard)
                    .expect("pin operator");
            }
            fs::write(dir.path().join(marker), []).expect("write stream evidence");
            let before = dir.bytes();
            let mut oracle = DeclaredOperatorOracle {
                operator: Some(standard.clone()),
                geometry: None,
            };
            let error = observe_sharded(&mut oracle, 60, 1, 1, dir.path(), None)
                .expect_err("stream evidence without state must fail");
            assert!(error.reason.contains("unfinished legacy"), "{error}");
            assert!(error.reason.contains(RAW_COMMITTED_FILE), "{error}");
            assert!(error.reason.contains("fresh output directory"), "{error}");
            assert_eq!(dir.bytes(), before);
        }

        let missing_rows = TestDir::new("raw-state-missing-rows");
        {
            let mut writer =
                ObservationShardWriter::open(missing_rows.path(), 1).expect("open writer");
            writer
                .set_attention_operator(&standard)
                .expect("pin operator");
        }
        let mut state = [0u8; 25];
        state[0..8].copy_from_slice(&1u64.to_le_bytes());
        fs::write(missing_rows.path().join(STATE_FILE), state).expect("write valid state");
        let before = missing_rows.bytes();
        let mut oracle = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: None,
        };
        let error = observe_sharded(&mut oracle, 60, 1, 1, missing_rows.path(), None)
            .expect_err("checkpoint rows missing from base shards must fail");
        assert!(error.reason.contains("unfinished legacy"), "{error}");
        assert!(error.reason.contains(RAW_COMMITTED_FILE), "{error}");
        assert!(error.reason.contains("fresh output directory"), "{error}");
        assert_eq!(missing_rows.bytes(), before);

        for alias_name in ["shard-1.bin", "shard-000.bin"] {
            let alias = TestDir::new("raw-noncanonical-shard-alias");
            {
                let mut writer =
                    ObservationShardWriter::open(alias.path(), 1).expect("open writer");
                writer
                    .set_attention_operator(&standard)
                    .expect("pin operator");
            }
            fs::write(alias.path().join(STATE_FILE), state).expect("write valid state");
            fs::write(alias.path().join(alias_name), [0u8; RECORD_SIZE])
                .expect("write numeric alias");
            let before = alias.bytes();
            let error = observe_sharded(&mut oracle, 60, 1, 1, alias.path(), None)
                .expect_err("noncanonical numeric shard aliases must fail");
            assert!(
                error.reason.contains("non-canonical numeric alias"),
                "{error}"
            );
            assert_eq!(alias.bytes(), before);
        }

        let tampered = TestDir::new("raw-completed-same-length-tamper");
        let mut oracle = DeclaredOperatorOracle {
            operator: Some(standard.clone()),
            geometry: None,
        };
        observe_sharded(&mut oracle, 60, 1, 1, tampered.path(), None)
            .expect("complete corpus before tampering");
        let manifest = ObservationManifest::load(tampered.path())
            .expect("load completed manifest")
            .expect("completed manifest exists");
        let (&shard, _) = manifest
            .completed
            .iter()
            .find(|(_, entry)| entry.records > 0)
            .expect("fixture completes a non-empty shard");
        let shard_path = tampered
            .path()
            .join(shard_file_name(manifest.shard_bits, shard));
        let mut shard_bytes = fs::read(&shard_path).expect("completed shard bytes");
        shard_bytes[0] ^= 0x80;
        fs::write(&shard_path, shard_bytes).expect("tamper completed shard in place");
        let before = tampered.bytes();
        let error = observe_sharded(&mut oracle, 60, 1, 1, tampered.path(), None)
            .expect_err("same-length completed-shard tampering must fail");
        assert!(error.reason.contains("manifest commits"), "{error}");
        assert_eq!(tampered.bytes(), before);

        let completed = TestDir::new("raw-completed-without-state");
        let mut oracle = DeclaredOperatorOracle {
            operator: Some(standard),
            geometry: None,
        };
        observe_sharded(&mut oracle, 60, 1, 1, completed.path(), None)
            .expect("complete raw corpus");
        let before = completed.bytes();
        fs::remove_file(completed.path().join(STATE_FILE)).expect("remove checkpoint");
        observe_sharded(&mut oracle, 60, 1, 1, completed.path(), None)
            .expect("authoritative checkpoint repairs a missing state mirror");
        assert_eq!(completed.bytes(), before);
    }

    #[test]
    fn source_snapshot_kappa_is_symmetric_once_bound_or_payload_exists() {
        let source_kappa = format!("blake3:{}", blake3::hash(b"source-a").to_hex());
        let bound_dir = TestDir::new("source-kappa-bound");
        let mut writer = ObservationShardWriter::open(bound_dir.path(), 1).expect("open writer");
        writer
            .set_source_manifest_kappa(&source_kappa)
            .expect("bind fresh source kappa");
        let before = bound_dir.bytes();
        let error = writer
            .preflight_source_manifest_kappa(None)
            .expect_err("bound source kappa requires an exact request");
        assert!(error.reason.contains("declares none"), "{error}");
        assert_eq!(bound_dir.bytes(), before);

        let legacy_dir = TestDir::new("source-kappa-orphan");
        fs::write(legacy_dir.path().join("shard-00.bin"), []).expect("write orphan shard");
        let before = legacy_dir.bytes();
        let writer = ObservationShardWriter::open(legacy_dir.path(), 1).expect("open legacy");
        let error = writer
            .preflight_source_manifest_kappa(Some(&source_kappa))
            .expect_err("orphan payload cannot acquire a source kappa");
        assert!(error.reason.contains("no recorded source"), "{error}");
        assert_eq!(legacy_dir.bytes(), before);

        let malformed_dir = TestDir::new("source-kappa-malformed");
        let before = malformed_dir.bytes();
        let mut writer =
            ObservationShardWriter::open(malformed_dir.path(), 1).expect("open fresh writer");
        let error = writer
            .set_source_manifest_kappa("blake3:not-a-digest")
            .expect_err("malformed requested kappa must fail");
        assert!(error.reason.contains("not canonical"), "{error}");
        assert_eq!(malformed_dir.bytes(), before);
    }
}
