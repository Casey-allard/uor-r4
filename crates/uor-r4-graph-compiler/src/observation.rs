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
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::TeacherOracle;
use uor_r4_model_source::geometry::GeometryProjection;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::progress::Progress;

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
            completed: BTreeMap::new(),
            total_records: 0,
        }
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

#[cfg(not(target_arch = "wasm32"))]
struct ShardHandle {
    file: BufWriter<fs::File>,
    probability: Option<BufWriter<fs::File>>,
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
        let entry = ShardEntry {
            records: length / RECORD_SIZE as u64,
            kappa: file_kappa(&path)?,
            partitions: self
                .partitions_active
                .then_some(self.partition_counts[shard as usize]),
            probability_kappa,
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

/// Run the teacher generation of `compile_hugging_face`'s corpus step,
/// spilling v4 records plus aligned probability metadata into content-addressed
/// shards instead of one
/// append-only corpus file. The teacher stream is the same deterministic
/// stream (seed 0x5EED, whole-story checkpointing); each record is routed
/// to `shard_of(sample_id(context_window), shard_bits)`, where the context
/// window is the existing 8-token window of fed tokens. Resume: a rerun
/// continues the stream from `state.bin`, skips shards the manifest marks
/// complete, and appends to the incomplete ones.
#[cfg(not(target_arch = "wasm32"))]
pub fn observe_sharded(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    shard_bits: u8,
    out: &Path,
    token_byte_lengths: Option<&[u32]>,
) -> Result<ObserveSummary, SourceUnavailable> {
    let mut writer = ObservationShardWriter::open(out, shard_bits)?;
    for shard in 0..writer.manifest().shard_count() {
        reconcile_probability_pair(out, shard_bits, shard)?;
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
            oracle.step(token, pos, &mut logits);
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
            if writer.write_record_with_probability(
                &record,
                ProbabilityMetadata {
                    target_logprob_nats: stats.target_logprob_nats,
                    entropy_bits: stats.entropy_bits,
                    top8_mass: stats.top8_mass,
                    target_rank: stats.target_rank,
                },
                shard,
            )? {
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
