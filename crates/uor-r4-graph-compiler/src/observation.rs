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
//! Resume extends the corpus' append-only resumability (`compiler.rs`):
//! `state.bin` checkpoints the deterministic teacher stream at whole-story
//! boundaries (same 25-byte layout as the corpus meta), and
//! `manifest.json` records which shards are complete with their content κ.
//! A rerun skips completed shards and regenerates only missing/incomplete
//! ones, continuing the stream from the checkpoint.

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
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::TeacherOracle;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::geometry::GeometryProjection;
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

/// Generator checkpoint file name within an observation directory.
pub const STATE_FILE: &str = "state.bin";

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
    /// oracle computed while producing these observations
    /// (`standard-source-attention/1`, or
    /// `experimental-r4-source-attention/1` when the `r4_attention`
    /// switch was on), when the producing pipeline's oracle declares
    /// one. `None` marks the legacy interpretation documented in
    /// `docs/MODEL_LIFECYCLE.md`. Optional with a serde default so
    /// every legacy manifest stays readable and legacy manifest bytes
    /// are unchanged when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention_operator: Option<AttentionOperatorSpec>,
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
            trace_profile: None,
            trace_row_bytes: None,
            completed: BTreeMap::new(),
            total_records: 0,
        }
    }

    /// Canonical serialization of the manifest's identity BUNDLE (#603):
    /// the five identity fields (`input_cid`, `source_manifest_kappa`,
    /// `geometry` #600, `tokenizer_adapter` #601, `attention_operator`
    /// #602) plus the #603 `trace_profile`, in that fixed order, one
    /// line per component. Absence is part of the digest input as an
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
        let mut text = String::from("uor-r4-observation-identity-bundle/1\n");
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

    /// Load the manifest of an observation directory, if present.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(dir: &Path) -> Result<Option<Self>, SourceUnavailable> {
        match fs::read(dir.join(MANIFEST_FILE)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| invalid_data(format!("invalid observation manifest: {error}"))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Persist the manifest atomically (write-then-rename). Shard files
    /// are always flushed before this runs, so a crash loses at most the
    /// manifest update; the affected shard is then regenerated on the
    /// next run.
    #[cfg(not(target_arch = "wasm32"))]
    fn store(&self, dir: &Path) -> Result<(), SourceUnavailable> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| invalid_data(format!("manifest serialization: {error}")))?;
        let tmp = dir.join(".manifest.json.tmp");
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, dir.join(MANIFEST_FILE))?;
        Ok(())
    }
}

/// Name of one shard's #603 trace sidecar: `<shard file>.trace`,
/// mirroring the `.prob` probability sidecar naming.
pub fn trace_sidecar_name(shard_bits: u8, shard: u32) -> String {
    format!("{}.trace", shard_file_name(shard_bits, shard))
}

#[cfg(not(target_arch = "wasm32"))]
struct ShardHandle {
    file: BufWriter<fs::File>,
    probability: Option<BufWriter<fs::File>>,
    trace: Option<BufWriter<fs::File>>,
}

/// Spills observation records into per-shard files with a κ-pinned
/// manifest. Records may arrive interleaved across shards (routed by
/// [`shard_of`]); each incomplete shard is appended to, so an interrupted
/// pass resumes from the bytes already on disk. Completed shards are
/// never rewritten: writes routed to them are skipped.
#[cfg(not(target_arch = "wasm32"))]
pub struct ObservationShardWriter {
    dir: PathBuf,
    manifest: ObservationManifest,
    handles: Vec<Option<ShardHandle>>,
    partition_counts: Vec<PartitionCounts>,
    partitions_active: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl ObservationShardWriter {
    /// Open (or create) an observation shard directory. An existing
    /// manifest pins the fan-out; requesting a different `shard_bits` for
    /// the same directory is an error.
    pub fn open(dir: impl AsRef<Path>, shard_bits: u8) -> Result<Self, SourceUnavailable> {
        if shard_bits > MAX_SHARD_BITS {
            return Err(invalid_input(format!(
                "shard_bits {shard_bits} exceeds the writer maximum {MAX_SHARD_BITS}"
            )));
        }
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        let manifest = match ObservationManifest::load(&dir)? {
            Some(manifest) => {
                if manifest.shard_bits != shard_bits {
                    return Err(invalid_input(format!(
                        "manifest shard_bits {} does not match requested {shard_bits}",
                        manifest.shard_bits
                    )));
                }
                manifest
            }
            None => ObservationManifest::new(shard_bits),
        };
        let handles = (0..manifest.shard_count()).map(|_| None).collect();
        let partition_counts = (0..manifest.shard_count())
            .map(|_| PartitionCounts::default())
            .collect();
        Ok(Self {
            dir,
            manifest,
            handles,
            partition_counts,
            partitions_active: false,
        })
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
        if self.manifest.partition_rule.as_deref() != Some(rule) {
            self.manifest.partition_rule = Some(rule.to_owned());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the input CID in the manifest (idempotent, atomic store).
    pub fn set_input_cid(&mut self, cid: &str) -> Result<(), SourceUnavailable> {
        if self.manifest.input_cid.as_deref() != Some(cid) {
            self.manifest.input_cid = Some(cid.to_owned());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the #597 source-snapshot manifest root κ of the teacher
    /// source in the observation manifest (idempotent, atomic store).
    pub fn set_source_manifest_kappa(&mut self, kappa: &str) -> Result<(), SourceUnavailable> {
        if self.manifest.source_manifest_kappa.as_deref() != Some(kappa) {
            self.manifest.source_manifest_kappa = Some(kappa.to_owned());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the #600 typed geometry-projection record of the teacher
    /// oracle in the observation manifest (idempotent, atomic store).
    pub fn set_geometry(&mut self, geometry: &GeometryProjection) -> Result<(), SourceUnavailable> {
        if self.manifest.geometry.as_ref() != Some(geometry) {
            self.manifest.geometry = Some(geometry.clone());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the #601 typed tokenizer-adapter identity record of the
    /// producing pipeline's tokenizer in the observation manifest
    /// (idempotent, atomic store).
    pub fn set_tokenizer_adapter(
        &mut self,
        adapter: &TokenizerAdapter,
    ) -> Result<(), SourceUnavailable> {
        if self.manifest.tokenizer_adapter.as_ref() != Some(adapter) {
            self.manifest.tokenizer_adapter = Some(adapter.clone());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the #602 typed attention-operator identity record of the
    /// teacher oracle in the observation manifest (idempotent, atomic
    /// store).
    pub fn set_attention_operator(
        &mut self,
        operator: &AttentionOperatorSpec,
    ) -> Result<(), SourceUnavailable> {
        if self.manifest.attention_operator.as_ref() != Some(operator) {
            self.manifest.attention_operator = Some(operator.clone());
            self.manifest.store(&self.dir)?;
        }
        Ok(())
    }

    /// Record the #603 typed teacher-trace-profile record of the
    /// producing pass in the observation manifest (idempotent, atomic
    /// store), mirroring
    /// [`ObservationShardWriter::set_geometry`]/[`ObservationShardWriter::set_tokenizer_adapter`].
    /// The minimal profile is never recorded — absence marks it — so
    /// legacy manifest bytes are unchanged for every minimal pass.
    pub fn set_trace_profile(&mut self, profile: &TraceProfile) -> Result<(), SourceUnavailable> {
        if self.manifest.trace_profile.as_ref() != Some(profile) {
            self.manifest.trace_profile = Some(profile.clone());
            self.manifest.store(&self.dir)?;
        }
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
                self.manifest.trace_row_bytes = Some(trace_row.len() as u64);
                self.manifest.store(&self.dir)?;
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
    /// Resolve the row layout from a trace profile and capture geometry —
    /// the same widths [`TraceCapture`] pins and every reader expects.
    pub fn new(
        profile: &TraceProfile,
        geometry: &uor_r4_model_source::TraceCaptureGeometry,
    ) -> Self {
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
        let kv_width = geometry.residual_width * geometry.kv_heads / geometry.heads;
        let row_bytes = residual_layers * residual_width * 4
            + usize::from(final_hidden) * residual_width * 4
            + qkv_layers * (residual_width + 2 * kv_width) * 4
            + attention_layers * geometry.heads * support * 8;
        Self {
            residual_layers,
            final_hidden,
            qkv_layers,
            attention_layers,
            heads: geometry.heads,
            support,
            residual_width,
            kv_width,
            row_bytes,
        }
    }

    /// Decode one `row_bytes`-long row into its structured lanes. Errors if
    /// the slice length does not match the pinned width.
    pub fn read_row(&self, row: &[u8]) -> Result<TraceRow, SourceUnavailable> {
        if row.len() != self.row_bytes {
            return Err(invalid_data(format!(
                "trace row is {} bytes, layout pins {}",
                row.len(),
                self.row_bytes
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
                    if position == SUPPORT_ABSENT_MARKER && weight_bits == SUPPORT_ABSENT_MARKER {
                        // Explicit absence marker: the slot is absent, never
                        // a zero-valued entry.
                        continue;
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
struct TraceCapture {
    profile: TraceProfile,
    residual_layers: Vec<usize>,
    qkv_layers: Vec<usize>,
    attention_layers: Vec<usize>,
    support: usize,
    final_hidden: bool,
    residual_width: usize,
    row_bytes: usize,
    row: Vec<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TraceCapture {
    /// Resolve `profile` against `oracle`'s declared capture geometry.
    /// Refused — never zero-filled — when the oracle exposes no capture
    /// surface, no hidden state for a declared final-hidden lane, or a
    /// declared layer index outside its layer range.
    fn new(profile: &TraceProfile, oracle: &dyn TeacherOracle) -> Result<Self, SourceUnavailable> {
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
        let row_bytes = TraceRowLayout::new(profile, &geometry).row_bytes;
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
    fn step(
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

/// Run the teacher generation of `compile_hugging_face`'s corpus step,
/// spilling v4 records plus aligned probability metadata into content-addressed
/// shards instead of one
/// append-only corpus file. The teacher stream is the same deterministic
/// stream (seed 0x5EED, whole-story checkpointing); each record is routed
/// to `shard_of(sample_id(context_window), shard_bits)`, where the context
/// window is the existing 8-token window of fed tokens. Resume: a rerun
/// continues the stream from `state.bin`, skips shards the manifest marks
/// complete, and appends to the incomplete ones.
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
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn observe_sharded_inner(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    shard_bits: u8,
    out: &Path,
    token_byte_lengths: Option<&[u32]>,
    mut trace: Option<TraceCapture>,
) -> Result<ObserveSummary, SourceUnavailable> {
    let mut writer = ObservationShardWriter::open(out, shard_bits)?;
    // #603 profile pinning: a corpus is captured under ONE profile. A
    // recorded profile must match the requested one (minimal requests
    // refuse traced corpora and vice versa once bytes exist); a fresh
    // traced pass records its profile before any record is written.
    let recorded = writer.manifest().trace_profile.clone();
    let has_prior_state = out.join(STATE_FILE).exists() || !writer.manifest().completed.is_empty();
    match (&recorded, trace.as_ref()) {
        (None, None) => {}
        (Some(recorded), Some(trace)) if *recorded == trace.profile => {}
        (None, Some(trace)) => {
            if has_prior_state {
                return Err(invalid_input(format!(
                    "{} was captured under the minimal trace profile; profile \
                     {}/{} cannot be introduced mid-corpus",
                    out.display(),
                    trace.profile.id,
                    trace.profile.version
                )));
            }
            writer.set_trace_profile(&trace.profile)?;
        }
        (Some(recorded), Some(trace)) => {
            return Err(invalid_input(format!(
                "{} is pinned to trace profile {}/{}; requested {}/{}",
                out.display(),
                recorded.id,
                recorded.version,
                trace.profile.id,
                trace.profile.version
            )));
        }
        (Some(recorded), None) => {
            return Err(invalid_input(format!(
                "{} is pinned to trace profile {}/{}; pass the same profile to resume",
                out.display(),
                recorded.id,
                recorded.version
            )));
        }
    }
    let trace_row_bytes = writer.manifest().trace_row_bytes;
    for shard in 0..writer.manifest().shard_count() {
        match (trace.as_ref(), trace_row_bytes) {
            (Some(_), Some(row_bytes)) => {
                reconcile_trace_triple(out, shard_bits, shard, row_bytes)?;
            }
            _ => reconcile_probability_pair(out, shard_bits, shard)?,
        }
    }
    let state_path = out.join(STATE_FILE);
    let (mut n, mut stories, mut rng, mut done) = match fs::read(&state_path) {
        Ok(bytes) if bytes.len() == 25 => (
            u64::from_le_bytes(bytes[0..8].try_into().expect("8-byte slice")),
            u64::from_le_bytes(bytes[8..16].try_into().expect("8-byte slice")),
            u64::from_le_bytes(bytes[16..24].try_into().expect("8-byte slice")),
            bytes[24],
        ),
        _ => (0, 0, 0x5EED, 0),
    };
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
        // Whole-story checkpoint: flush shard bytes first so they cover
        // exactly the completed stories, then pin the stream position
        // (identical 25-byte layout to the corpus meta).
        writer.flush()?;
        let mut state = [0u8; 25];
        state[0..8].copy_from_slice(&n.to_le_bytes());
        state[8..16].copy_from_slice(&stories.to_le_bytes());
        state[16..24].copy_from_slice(&rng.to_le_bytes());
        state[24] = done;
        fs::write(&state_path, state)?;
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
}
