//! A sealed inference checkpoint of the geometric stack with its session
//! store (I1).
//!
//! # Contents
//!
//! A checkpoint is one report root, claimed exclusively
//! ([`report_output::claim`]) and sealed ([`report_output::seal`]):
//!
//! | File | Content |
//! |---|---|
//! | `attempt.json` | the claim sentinel |
//! | `config.json`, `model.safetensors` | the [`StackModel`], written by its own `save` |
//! | `identity.json` | the [`CheckpointRecord`]: schema, restore kind, provenance identities and digests |
//! | `memory.json` | the session [`StackStore`] in `Memory`'s own serialization, when a store is given |
//! | `manifest.json` | the seal: every file with its size and BLAKE3 digest |
//!
//! # Inference reload, not training resume
//!
//! This is an **inference** checkpoint. It restores the model's configuration
//! and weights, the model's provenance identities and the session store. A
//! reloaded session therefore computes the same logits and reads the same
//! memory. The checkpoint holds none of the following:
//! - optimizer moments (`StackAdamW`);
//! - the learning-rate schedule position;
//! - the data-sampler position;
//! - RNG state.
//!
//! Training cannot resume from it. The training driver's own resumable
//! checkpoints (`geometric-stack dialogue-train`, `checkpoint/`) remain the
//! resume path.
//!
//! # What load checks
//!
//! 1. The seal (`report_output::verify`): every listed file is unchanged, and
//!    there is no unlisted file.
//! 2. The exact file set of a checkpoint. A failed save leaves `error.json`
//!    and never loads.
//! 3. The schema and restore kind of `identity.json`.
//! 4. The declared identities: their formats, the tokenizer the protocol
//!    names, and the recomputed protocol identity.
//! 5. The recorded SHA-256 of `config.json`, `model.safetensors` and
//!    `memory.json`.
//! 6. The model: its configuration and tensor shapes (`StackModel::load`),
//!    its parameter count, and the protocol's special ids against its
//!    vocabulary.
//! 7. The store:
//!    - `Memory`'s validation and the token encoding;
//!    - the recorded lineage, so a foreign lineage is rejected;
//!    - capacity, commit, record count and history digest;
//!    - every stored token against the vocabulary.
//!
//! The seal and the digests detect changed, missing and added files. They are
//! an integrity check, not authentication: a writer who rewrites every file
//! consistently and reseals produces a checkpoint that loads.
//!
//! Reload restores the float weights bit for bit. On the same build and
//! backend, the reloaded model's logits are bit-identical; this is tested on
//! the CPU. Equality across backends is not claimed.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueError, DialogueProtocol, SCHEMA as DIALOGUE_SCHEMA};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_stack::{StackConfig, StackModel};
use crate::stack_store::{StackStore, StackStoreError};
use crate::{sha256_file, TrainingError};

/// The schema of `identity.json`.
pub const CHECKPOINT_SCHEMA: &str = "uor-r4.stack-checkpoint/1";
/// The only restore kind: inference reload, never training resume.
pub const RESTORE_INFERENCE: &str = "inference";
/// The files `StackModel::save` writes.
pub const CONFIG_FILE: &str = "config.json";
pub const MODEL_FILE: &str = "model.safetensors";
pub const IDENTITY_FILE: &str = "identity.json";
pub const MEMORY_FILE: &str = "memory.json";
/// Written beside a failed save before its root is sealed. A root holding it
/// never loads.
pub const ERROR_FILE: &str = "error.json";

const RESTORE_SCOPE: &str = "Inference reload of the geometric stack: configuration, weights, \
provenance identities and the optional session store. No optimizer moments, learning-rate \
schedule position, data-sampler position or RNG state: training cannot resume from this \
checkpoint.";

const CHECKPOINT_FILES: [&str; 5] = [
    report_output::ATTEMPT_FILE,
    CONFIG_FILE,
    MODEL_FILE,
    IDENTITY_FILE,
    MEMORY_FILE,
];

/// The content identity of one training input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataIdentity {
    /// A caller label, unique within a checkpoint, such as `train_tokens`.
    pub label: String,
    pub bytes: u64,
    /// Lowercase hex SHA-256 of the content.
    pub sha256: String,
}

impl DataIdentity {
    /// The identity of a file's current contents.
    pub fn of_file(label: impl Into<String>, path: &Path) -> Result<Self, StackCheckpointError> {
        Ok(Self {
            label: label.into(),
            bytes: fs::metadata(path)?.len(),
            sha256: sha256_file(path)?,
        })
    }
}

/// The provenance a caller declares for a checkpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointIdentity {
    /// Lowercase hex SHA-256 of the `tokenizer.json` bytes.
    pub tokenizer_sha256: String,
    /// The tokenizer's content address, `ByteBpeTokenizer::address`:
    /// `blake3:` and 64 lowercase hex digits.
    pub tokenizer_cid: String,
    /// The dialogue protocol the model was trained under. Its identity is
    /// recorded, and it is recomputed on load.
    pub protocol: DialogueProtocol,
    /// The training data.
    pub data: Vec<DataIdentity>,
    /// SHA-256 of the sealed training report's `manifest.json`
    /// ([`sealed_manifest_sha256`]).
    pub training_manifest_sha256: String,
}

impl CheckpointIdentity {
    /// The identity of a model trained with `tokenizer_json` under the
    /// literal-role dialogue protocol. The SHA-256, the CID and the protocol
    /// all come from the same bytes.
    pub fn from_tokenizer(
        tokenizer_json: &[u8],
        data: Vec<DataIdentity>,
        training_manifest_sha256: String,
    ) -> Result<Self, StackCheckpointError> {
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json)
            .ok_or_else(|| identity_error("unreadable tokenizer.json"))?;
        let identity = Self {
            tokenizer_sha256: sha256_hex(tokenizer_json),
            tokenizer_cid: tokenizer.address(),
            protocol: DialogueProtocol::literal_roles_v1(&tokenizer)?,
            data,
            training_manifest_sha256,
        };
        identity.validate()?;
        Ok(identity)
    }

    /// Check that `tokenizer_json` is the declared tokenizer and that the
    /// protocol binds to it.
    pub fn check_tokenizer(&self, tokenizer_json: &[u8]) -> Result<(), StackCheckpointError> {
        let actual = sha256_hex(tokenizer_json);
        if actual != self.tokenizer_sha256 {
            return Err(identity_error(format!(
                "tokenizer.json has SHA-256 {actual}; the checkpoint declares {}",
                self.tokenizer_sha256
            )));
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json)
            .ok_or_else(|| identity_error("unreadable tokenizer.json"))?;
        if tokenizer.address() != self.tokenizer_cid {
            return Err(identity_error(
                "the tokenizer's content address differs from the declared CID",
            ));
        }
        self.protocol.bind(&tokenizer)?;
        Ok(())
    }

    /// Check that each declared identity is well formed and that the
    /// protocol names the declared tokenizer.
    pub fn validate(&self) -> Result<(), StackCheckpointError> {
        if !is_sha256_hex(&self.tokenizer_sha256) {
            return Err(identity_error(
                "tokenizer_sha256 must be 64 lowercase hex digits",
            ));
        }
        if !self
            .tokenizer_cid
            .strip_prefix("blake3:")
            .is_some_and(is_sha256_hex)
        {
            return Err(identity_error(
                "tokenizer_cid must be blake3: and 64 lowercase hex digits",
            ));
        }
        if self.protocol.schema != DIALOGUE_SCHEMA {
            return Err(identity_error(format!(
                "unsupported dialogue protocol {}",
                self.protocol.schema
            )));
        }
        if self.protocol.tokenizer_cid != self.tokenizer_cid {
            return Err(identity_error(
                "the dialogue protocol names another tokenizer",
            ));
        }
        if self.data.is_empty() {
            return Err(identity_error("no training data identity is declared"));
        }
        let mut labels = BTreeSet::new();
        for data in &self.data {
            if data.label.is_empty() || !labels.insert(data.label.as_str()) {
                return Err(identity_error(format!(
                    "data label {:?} is empty or repeated",
                    data.label
                )));
            }
            if !is_sha256_hex(&data.sha256) {
                return Err(identity_error(format!(
                    "data {} needs a 64-digit lowercase hex SHA-256",
                    data.label
                )));
            }
        }
        if !is_sha256_hex(&self.training_manifest_sha256) {
            return Err(identity_error(
                "training_manifest_sha256 must be 64 lowercase hex digits",
            ));
        }
        Ok(())
    }
}

/// The identity of a checkpoint's session store (`memory.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryRecord {
    pub sha256: String,
    pub bytes: u64,
    /// The store's lineage. A file of any other lineage is rejected.
    pub lineage: u64,
    pub capacity: usize,
    pub commit: u64,
    pub records: usize,
    /// `Memory::history_sha256` at the latest commit.
    pub history_sha256: String,
}

/// `identity.json`: what a checkpoint is and what it binds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointRecord {
    /// Always [`CHECKPOINT_SCHEMA`].
    pub schema: String,
    /// Always [`RESTORE_INFERENCE`].
    pub restore: String,
    /// A plain statement of what a load restores.
    pub scope: String,
    pub identity: CheckpointIdentity,
    /// `DialogueProtocol::identity` of `identity.protocol`.
    pub protocol_identity: String,
    /// SHA-256 of `config.json`.
    pub config_sha256: String,
    /// SHA-256 of `model.safetensors`.
    pub model_sha256: String,
    pub parameters: usize,
    pub memory: Option<MemoryRecord>,
}

/// A loaded, verified checkpoint.
pub struct StackCheckpoint {
    pub model: StackModel,
    /// The verified `identity.json`.
    pub record: CheckpointRecord,
    /// The session store, when the checkpoint holds one.
    pub memory: Option<StackStore>,
}

impl StackCheckpoint {
    /// The provenance the checkpoint declares.
    pub fn identity(&self) -> &CheckpointIdentity {
        &self.record.identity
    }
}

impl fmt::Debug for StackCheckpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackCheckpoint")
            .field("config", &self.model.config)
            .field("record", &self.record)
            .field("memory", &self.memory)
            .finish()
    }
}

/// Classified failures at the checkpoint boundary.
#[derive(Debug)]
pub enum StackCheckpointError {
    /// The root could not be claimed exclusively. It may exist; choose a new
    /// path.
    Claim(std::io::Error),
    /// `report_output::verify` refused the root: a changed, missing or
    /// unlisted file.
    Seal(std::io::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    /// Saving or loading the model, or hashing a file, failed.
    Training(TrainingError),
    Store(StackStoreError),
    Protocol(DialogueError),
    /// `identity.json` names a schema other than [`CHECKPOINT_SCHEMA`].
    Schema {
        found: String,
    },
    /// A declared identity is malformed or inconsistent with the model.
    Identity(String),
    /// The sealed files are not a checkpoint's file set.
    FileSet(String),
    /// A file differs from its recorded SHA-256.
    Digest {
        file: &'static str,
        recorded: String,
        actual: String,
    },
    /// The store differs from its recorded identity.
    Memory(String),
    /// The model is in served (QAT) mode. Its forward reads the export's
    /// values, which an inference checkpoint does not store, so a reload
    /// would silently run the float weights. Save with served mode off, or
    /// export the model instead.
    ServedMode,
    /// The model snaps its transport (`StackModel::set_transport_snap`).
    /// An inference checkpoint restores the float weights without the snap,
    /// so a reload would silently run the unsnapped transport.
    TransportSnap,
}

impl fmt::Display for StackCheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claim(error) => write!(f, "stack checkpoint root not claimed: {error}"),
            Self::Seal(error) => write!(f, "stack checkpoint seal: {error}"),
            Self::Io(error) => write!(f, "stack checkpoint I/O: {error}"),
            Self::Json(error) => write!(f, "stack checkpoint JSON: {error}"),
            Self::Training(error) => write!(f, "stack checkpoint model: {error}"),
            Self::Store(error) => write!(f, "stack checkpoint memory: {error}"),
            Self::Protocol(error) => write!(f, "stack checkpoint protocol: {error}"),
            Self::Schema { found } => write!(
                f,
                "stack checkpoint schema {found:?} is not {CHECKPOINT_SCHEMA:?}"
            ),
            Self::Identity(message) => write!(f, "stack checkpoint identity: {message}"),
            Self::FileSet(message) => write!(f, "stack checkpoint file set: {message}"),
            Self::Digest {
                file,
                recorded,
                actual,
            } => write!(
                f,
                "stack checkpoint {file} has SHA-256 {actual}; {recorded} is recorded"
            ),
            Self::Memory(message) => write!(f, "stack checkpoint memory record: {message}"),
            Self::ServedMode => write!(
                f,
                "stack checkpoint: the model is in served (QAT) mode; its forward reads exported \
                 values that an inference checkpoint does not store"
            ),
            Self::TransportSnap => write!(
                f,
                "stack checkpoint: the model snaps its transport; an inference checkpoint \
                 restores the float weights without the snap"
            ),
        }
    }
}

impl std::error::Error for StackCheckpointError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Claim(error) | Self::Seal(error) | Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Training(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}

macro_rules! convert {
    ($source:ty, $variant:ident) => {
        impl From<$source> for StackCheckpointError {
            fn from(error: $source) -> Self {
                Self::$variant(error)
            }
        }
    };
}
convert!(std::io::Error, Io);
convert!(serde_json::Error, Json);
convert!(TrainingError, Training);
convert!(StackStoreError, Store);
convert!(DialogueError, Protocol);

/// The SHA-256 of a sealed report's `manifest.json`, after the report's
/// complete file set verifies. This is the value
/// [`CheckpointIdentity::training_manifest_sha256`] binds.
pub fn sealed_manifest_sha256(report_root: &Path) -> Result<String, StackCheckpointError> {
    report_output::verify(report_root).map_err(StackCheckpointError::Seal)?;
    Ok(sha256_file(
        &report_root.join(report_output::MANIFEST_FILE),
    )?)
}

/// Write a sealed inference checkpoint of `model` and, when given, the
/// session store `memory` to the new root `root`.
///
/// Every argument is checked, and the store serialized, before the root is
/// claimed. An existing root is refused and left unchanged. If a write fails
/// after the claim, the attempt is sealed beside `error.json`. It is kept as
/// evidence and never loads, and a retry needs a new root.
pub fn save_checkpoint(
    root: &Path,
    model: &StackModel,
    identity: &CheckpointIdentity,
    memory: Option<&StackStore>,
) -> Result<CheckpointRecord, StackCheckpointError> {
    if model.served_codec().is_some() {
        return Err(StackCheckpointError::ServedMode);
    }
    if model.transport_snap().is_some() {
        return Err(StackCheckpointError::TransportSnap);
    }
    identity.validate()?;
    check_against_model(identity, &model.config)?;
    let protocol_identity = identity.protocol.identity()?;
    let memory = memory
        .map(|store| prepare_memory(store, model.config.vocab_size))
        .transpose()?;
    report_output::claim(root).map_err(StackCheckpointError::Claim)?;
    match write_checkpoint(root, model, identity, protocol_identity, memory) {
        Ok(record) => {
            report_output::seal(root)?;
            report_output::verify(root).map_err(StackCheckpointError::Seal)?;
            Ok(record)
        }
        Err(error) => {
            // The primary error is returned. A failure to record it cannot
            // make the root loadable, because the file set is incomplete.
            let note = serde_json::json!({ "error": error.to_string() });
            if let Ok(bytes) = serde_json::to_vec_pretty(&note) {
                let _ = write_new(&root.join(ERROR_FILE), &bytes);
            }
            let _ = report_output::seal(root);
            Err(error)
        }
    }
}

/// Verify and load the sealed checkpoint at `root` onto `device`. The
/// module docs list the checks.
pub fn load_checkpoint(
    root: &Path,
    device: &Device,
) -> Result<StackCheckpoint, StackCheckpointError> {
    report_output::verify(root).map_err(StackCheckpointError::Seal)?;
    let listed = sealed_files(root)?;
    for name in [
        report_output::ATTEMPT_FILE,
        CONFIG_FILE,
        MODEL_FILE,
        IDENTITY_FILE,
    ] {
        if !listed.contains(name) {
            return Err(StackCheckpointError::FileSet(format!("{name} is missing")));
        }
    }
    if let Some(name) = listed
        .iter()
        .find(|name| !CHECKPOINT_FILES.contains(&name.as_str()))
    {
        return Err(StackCheckpointError::FileSet(format!(
            "{name} is not a checkpoint file"
        )));
    }
    let record = read_record(&root.join(IDENTITY_FILE))?;
    if record.restore != RESTORE_INFERENCE {
        return Err(identity_error(format!(
            "restore kind {:?} is not {RESTORE_INFERENCE:?}",
            record.restore
        )));
    }
    if listed.contains(MEMORY_FILE) != record.memory.is_some() {
        return Err(StackCheckpointError::FileSet(
            "memory.json and the recorded memory identity disagree".into(),
        ));
    }
    record.identity.validate()?;
    if record.identity.protocol.identity()? != record.protocol_identity {
        return Err(identity_error(
            "the recorded protocol identity differs from the protocol's",
        ));
    }
    check_digest(root, CONFIG_FILE, &record.config_sha256)?;
    check_digest(root, MODEL_FILE, &record.model_sha256)?;
    let model = StackModel::load(root, device)?;
    check_against_model(&record.identity, &model.config)?;
    if model.parameter_count() != record.parameters {
        return Err(identity_error(format!(
            "the model has {} parameters; {} are recorded",
            model.parameter_count(),
            record.parameters
        )));
    }
    let memory = record
        .memory
        .as_ref()
        .map(|entry| load_memory(root, entry, model.config.vocab_size))
        .transpose()?;
    Ok(StackCheckpoint {
        model,
        record,
        memory,
    })
}

fn identity_error(message: impl Into<String>) -> StackCheckpointError {
    StackCheckpointError::Identity(message.into())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The protocol's special ids must be tokens of the model.
fn check_against_model(
    identity: &CheckpointIdentity,
    config: &StackConfig,
) -> Result<(), StackCheckpointError> {
    let protocol = &identity.protocol;
    for (name, id) in [
        ("BOS", protocol.bos_id),
        ("EOS", protocol.eos_id),
        ("UNK", protocol.unk_id),
    ] {
        if id as usize >= config.vocab_size {
            return Err(identity_error(format!(
                "the protocol's {name} id {id} is outside the model's vocabulary of {}",
                config.vocab_size
            )));
        }
    }
    Ok(())
}

fn prepare_memory(
    store: &StackStore,
    vocab_size: usize,
) -> Result<(Vec<u8>, MemoryRecord), StackCheckpointError> {
    store.check_vocabulary(vocab_size)?;
    let bytes = store.to_bytes()?;
    let record = MemoryRecord {
        sha256: sha256_hex(&bytes),
        bytes: bytes.len() as u64,
        lineage: store.lineage(),
        capacity: store.capacity(),
        commit: store.commit(),
        records: store.records(),
        history_sha256: store.history_sha256()?,
    };
    Ok((bytes, record))
}

fn write_checkpoint(
    root: &Path,
    model: &StackModel,
    identity: &CheckpointIdentity,
    protocol_identity: String,
    memory: Option<(Vec<u8>, MemoryRecord)>,
) -> Result<CheckpointRecord, StackCheckpointError> {
    model.save(root)?;
    let memory = match memory {
        Some((bytes, record)) => {
            write_new(&root.join(MEMORY_FILE), &bytes)?;
            Some(record)
        }
        None => None,
    };
    let record = CheckpointRecord {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        restore: RESTORE_INFERENCE.to_owned(),
        scope: RESTORE_SCOPE.to_owned(),
        identity: identity.clone(),
        protocol_identity,
        config_sha256: sha256_file(&root.join(CONFIG_FILE))?,
        model_sha256: sha256_file(&root.join(MODEL_FILE))?,
        parameters: model.parameter_count(),
        memory,
    };
    write_new(
        &root.join(IDENTITY_FILE),
        &serde_json::to_vec_pretty(&record)?,
    )?;
    Ok(record)
}

/// Write a file that must not exist yet.
fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    fs::File::create_new(path)?.write_all(bytes)
}

/// The files the seal lists, from `manifest.json`.
fn sealed_files(root: &Path) -> Result<BTreeSet<String>, StackCheckpointError> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join(report_output::MANIFEST_FILE))?)?;
    let files = manifest["files"]
        .as_array()
        .ok_or_else(|| StackCheckpointError::FileSet("the manifest lists no files".into()))?;
    files
        .iter()
        .map(|entry| {
            entry["path"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| StackCheckpointError::FileSet("a manifest entry has no path".into()))
        })
        .collect()
}

/// Parse `identity.json`, checking its schema before its fields.
fn read_record(path: &Path) -> Result<CheckpointRecord, StackCheckpointError> {
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    match value.get("schema").and_then(Value::as_str) {
        Some(CHECKPOINT_SCHEMA) => Ok(serde_json::from_value(value)?),
        found => Err(StackCheckpointError::Schema {
            found: found.unwrap_or("none").to_owned(),
        }),
    }
}

fn check_digest(
    root: &Path,
    file: &'static str,
    recorded: &str,
) -> Result<(), StackCheckpointError> {
    let actual = sha256_file(&root.join(file))?;
    if actual == recorded {
        Ok(())
    } else {
        Err(StackCheckpointError::Digest {
            file,
            recorded: recorded.to_owned(),
            actual,
        })
    }
}

fn load_memory(
    root: &Path,
    entry: &MemoryRecord,
    vocab_size: usize,
) -> Result<StackStore, StackCheckpointError> {
    let bytes = fs::read(root.join(MEMORY_FILE))?;
    let actual = sha256_hex(&bytes);
    if actual != entry.sha256 {
        return Err(StackCheckpointError::Digest {
            file: MEMORY_FILE,
            recorded: entry.sha256.clone(),
            actual,
        });
    }
    let store = StackStore::from_bytes(&bytes, entry.lineage)?;
    let found = (
        bytes.len() as u64,
        store.capacity(),
        store.commit(),
        store.records(),
        store.history_sha256()?,
    );
    let recorded = (
        entry.bytes,
        entry.capacity,
        entry.commit,
        entry.records,
        entry.history_sha256.clone(),
    );
    if found != recorded {
        return Err(StackCheckpointError::Memory(format!(
            "the store has (bytes, capacity, commit, records, history) {found:?}; {recorded:?} is recorded"
        )));
    }
    store.check_vocabulary(vocab_size)?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::geometric_stack::{ReadScore, StackArch};
    use crate::stack_store::{test_store, HistoryView, Update, VIEWS};

    const VOCAB: usize = 64;

    fn scratch(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "uor-r4-stack-checkpoint-{}-{nonce}-{name}",
            std::process::id()
        ))
    }

    /// A `vocab`-token tokenizer with the literal-role specials at ids 0, 1
    /// and 2.
    fn tokenizer_json(vocab: usize) -> Vec<u8> {
        let mut entries = serde_json::Map::new();
        for id in 0..vocab as u32 {
            let token = match id {
                0 => "<|bos|>".to_owned(),
                1 => "<|eos|>".to_owned(),
                2 => "<|unk|>".to_owned(),
                _ => format!("v{id}"),
            };
            entries.insert(token, json!(id));
        }
        json!({
            "model": {"type": "BPE", "vocab": entries, "merges": []},
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ]
        })
        .to_string()
        .into_bytes()
    }

    /// A fresh base directory and an identity built from real bytes: a
    /// tokenizer, a data file and a sealed stand-in training report.
    fn fixture(name: &str) -> (PathBuf, CheckpointIdentity) {
        let base = scratch(name);
        fs::create_dir_all(&base).expect("base");
        let data = base.join("train.tokens");
        fs::write(&data, [1u8, 2, 3, 4]).expect("data");
        let report = base.join("training-report");
        report_output::claim(&report).expect("claim training report");
        fs::write(report.join("report.json"), b"{\"steps\":1}").expect("report");
        report_output::seal(&report).expect("seal training report");
        let identity = CheckpointIdentity::from_tokenizer(
            &tokenizer_json(VOCAB),
            vec![DataIdentity::of_file("train_tokens", &data).expect("data identity")],
            sealed_manifest_sha256(&report).expect("manifest digest"),
        )
        .expect("identity");
        (base, identity)
    }

    /// The main-line layout, `rrarra` with quaternion transport, at test
    /// width.
    fn tiny_model() -> StackModel {
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: VOCAB,
            width: 16,
            heads: 2,
            mlp_hidden: 24,
            context: 12,
            pattern: "rrarra".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 5,
            memory: None,
            select: None,
            pointer: None,
        };
        StackModel::new(config, &Device::Cpu).expect("tiny stack")
    }

    fn logit_bits(model: &StackModel) -> Vec<u32> {
        let ids: Vec<u32> = (0..24u32).map(|i| (i * 7 + 3) % VOCAB as u32).collect();
        model
            .forward(&ids, 2, 12)
            .expect("forward")
            .flatten_all()
            .expect("flatten")
            .to_vec1::<f32>()
            .expect("logits")
            .into_iter()
            .map(f32::to_bits)
            .collect()
    }

    /// A saved checkpoint of the tiny model and the test store.
    fn saved(
        base: &Path,
        name: &str,
        identity: &CheckpointIdentity,
    ) -> (PathBuf, StackModel, StackStore) {
        let (model, store) = (tiny_model(), test_store(41));
        let root = base.join(name);
        save_checkpoint(&root, &model, identity, Some(&store)).expect("save");
        (root, model, store)
    }

    /// Copy `source`'s checkpoint files into the new sealed root `target`
    /// after `edit`. The copy's seal verifies, so only the checkpoint's own
    /// checks can reject it.
    fn reseal(source: &Path, target: &Path, edit: impl FnOnce(&Path)) {
        report_output::claim(target).expect("claim");
        for name in [CONFIG_FILE, MODEL_FILE, IDENTITY_FILE, MEMORY_FILE] {
            if source.join(name).exists() {
                fs::copy(source.join(name), target.join(name)).expect("copy");
            }
        }
        edit(target);
        report_output::seal(target).expect("seal");
    }

    fn edit_record(root: &Path, edit: impl FnOnce(&mut Value)) {
        let path = root.join(IDENTITY_FILE);
        let mut value: Value =
            serde_json::from_slice(&fs::read(&path).expect("read")).expect("record");
        edit(&mut value);
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("record")).expect("write");
    }

    fn load_error(root: &Path) -> StackCheckpointError {
        match load_checkpoint(root, &Device::Cpu) {
            Ok(_) => panic!("{} loaded", root.display()),
            Err(error) => error,
        }
    }

    #[test]
    fn save_and_load_reproduce_logits_and_memory_reads_bit_for_bit() {
        let (base, identity) = fixture("round-trip");
        let (root, model, store) = saved(&base, "checkpoint", &identity);
        let loaded = load_checkpoint(&root, &Device::Cpu).expect("load");
        assert_eq!(loaded.identity(), &identity);
        assert_eq!(loaded.record.schema, CHECKPOINT_SCHEMA);
        assert_eq!(loaded.record.restore, RESTORE_INFERENCE);
        assert_eq!(
            loaded.record.model_sha256,
            sha256_file(&root.join(MODEL_FILE)).expect("digest")
        );
        assert_eq!(loaded.model.config, model.config);
        assert_eq!(logit_bits(&loaded.model), logit_bits(&model));
        let memory = loaded.memory.as_ref().expect("memory");
        assert_eq!(memory, &store);
        for address in store.addresses().expect("addresses") {
            for view in VIEWS {
                let (scope, entity, relation) = (&address.scope, &address.entity, address.relation);
                assert_eq!(
                    memory.read(scope, entity, relation, view),
                    store.read(scope, entity, relation, view),
                    "{address:?} {view:?}"
                );
            }
        }
        // The session continues identically after reload.
        let (mut before, mut after) = (store.clone(), memory.clone());
        for (value, update) in [(&[30u32][..], Update::Assert), (&[31], Update::Correct)] {
            assert_eq!(
                before.write(b"alice", &[3, 4], 1, value, update),
                after.write(b"alice", &[3, 4], 1, value, update)
            );
        }
        for view in VIEWS {
            assert_eq!(
                after.read(b"alice", &[3, 4], 1, view),
                before.read(b"alice", &[3, 4], 1, view)
            );
        }
        assert_eq!(
            before.to_bytes().expect("bytes"),
            after.to_bytes().expect("bytes")
        );
        // The identity binds the tokenizer it was built from, and no other.
        identity
            .check_tokenizer(&tokenizer_json(VOCAB))
            .expect("the declared tokenizer");
        assert!(matches!(
            identity.check_tokenizer(&tokenizer_json(VOCAB + 1)),
            Err(StackCheckpointError::Identity(_))
        ));
        // Without a store, a checkpoint has no memory file and loads without one.
        let bare = base.join("bare");
        save_checkpoint(&bare, &model, &identity, None).expect("save");
        assert!(!bare.join(MEMORY_FILE).exists());
        let loaded = load_checkpoint(&bare, &Device::Cpu).expect("load");
        assert!(loaded.memory.is_none() && loaded.record.memory.is_none());
        assert_eq!(logit_bits(&loaded.model), logit_bits(&model));
        fs::remove_dir_all(&base).expect("clean");
    }

    #[test]
    fn a_tampered_file_is_rejected() {
        let (base, identity) = fixture("tampered");
        let (clean, _, store) = saved(&base, "clean", &identity);
        let (root, _, _) = saved(&base, "checkpoint", &identity);
        // A change under the seal.
        let mut weights = fs::read(root.join(MODEL_FILE)).expect("read");
        let last = weights.len() - 1;
        weights[last] ^= 1;
        fs::write(root.join(MODEL_FILE), &weights).expect("write");
        assert!(matches!(load_error(&root), StackCheckpointError::Seal(_)));
        // The same change, resealed, fails the recorded digest.
        let resealed = base.join("resealed-model");
        reseal(&root, &resealed, |_| {});
        assert!(matches!(
            load_error(&resealed),
            StackCheckpointError::Digest {
                file: MODEL_FILE,
                ..
            }
        ));
        // Another valid store of the same lineage, resealed, fails the memory
        // digest.
        let mut other = store.clone();
        other
            .write(b"alice", &[5], 1, &[20], Update::Correct)
            .expect("write");
        let other = other.to_bytes().expect("bytes");
        let swapped = base.join("resealed-memory");
        reseal(&clean, &swapped, |root| {
            fs::write(root.join(MEMORY_FILE), &other).expect("write")
        });
        assert!(matches!(
            load_error(&swapped),
            StackCheckpointError::Digest {
                file: MEMORY_FILE,
                ..
            }
        ));
        // An edited recorded identity, resealed, fails its recomputation.
        let edited = base.join("resealed-identity");
        reseal(&clean, &edited, |root| {
            edit_record(root, |record| {
                record["protocol_identity"] = json!(format!("blake3:{}", "0".repeat(64)))
            })
        });
        assert!(matches!(
            load_error(&edited),
            StackCheckpointError::Identity(_)
        ));
        load_checkpoint(&clean, &Device::Cpu).expect("the clean checkpoint loads");
        fs::remove_dir_all(&base).expect("clean");
    }

    #[test]
    fn a_missing_file_is_rejected() {
        let (base, identity) = fixture("missing");
        let (root, _, _) = saved(&base, "checkpoint", &identity);
        // Files absent from a sealed set: the file-set check.
        for (name, missing) in [
            ("no-memory", MEMORY_FILE),
            ("no-model", MODEL_FILE),
            ("no-identity", IDENTITY_FILE),
            ("no-config", CONFIG_FILE),
        ] {
            let target = base.join(name);
            reseal(&root, &target, |root| {
                fs::remove_file(root.join(missing)).expect("remove")
            });
            assert!(
                matches!(load_error(&target), StackCheckpointError::FileSet(_)),
                "{missing}"
            );
        }
        // An unexpected file beside a checkpoint is refused too.
        let target = base.join("extra");
        reseal(&root, &target, |root| {
            fs::write(root.join(ERROR_FILE), b"{}").expect("write")
        });
        assert!(matches!(
            load_error(&target),
            StackCheckpointError::FileSet(_)
        ));
        // A sealed file deleted after sealing: the seal check.
        fs::remove_file(root.join(MEMORY_FILE)).expect("remove");
        match load_error(&root) {
            StackCheckpointError::Seal(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::NotFound)
            }
            other => panic!("{other}"),
        }
        fs::remove_dir_all(&base).expect("clean");
    }

    #[test]
    fn a_wrong_schema_is_rejected() {
        let (base, identity) = fixture("schema");
        let (root, _, _) = saved(&base, "checkpoint", &identity);
        let target = base.join("older");
        reseal(&root, &target, |root| {
            edit_record(root, |record| {
                record["schema"] = json!("uor-r4.stack-checkpoint/0")
            })
        });
        match load_error(&target) {
            StackCheckpointError::Schema { found } => {
                assert_eq!(found, "uor-r4.stack-checkpoint/0")
            }
            other => panic!("{other}"),
        }
        let target = base.join("unnamed");
        reseal(&root, &target, |root| {
            edit_record(root, |record| {
                record.as_object_mut().expect("object").remove("schema");
            })
        });
        assert!(matches!(
            load_error(&target),
            StackCheckpointError::Schema { found } if found == "none"
        ));
        // Schema 1 is an inference checkpoint and has no optimizer state.
        let target = base.join("resume");
        reseal(&root, &target, |root| {
            edit_record(root, |record| record["restore"] = json!("training"))
        });
        assert!(matches!(
            load_error(&target),
            StackCheckpointError::Identity(_)
        ));
        let target = base.join("optimizer");
        reseal(&root, &target, |root| {
            edit_record(root, |record| record["optimizer"] = json!("adamw"))
        });
        assert!(matches!(load_error(&target), StackCheckpointError::Json(_)));
        fs::remove_dir_all(&base).expect("clean");
    }

    #[test]
    fn a_foreign_lineage_memory_is_rejected() {
        let (base, identity) = fixture("lineage");
        let (root, _, store) = saved(&base, "checkpoint", &identity);
        // The same records under another lineage, with the recorded digest
        // and size updated, so that only the lineage differs.
        let mut memory = store.memory().clone();
        memory.lineage = 42;
        let foreign = memory.to_bytes().expect("bytes");
        let target = base.join("foreign-file");
        reseal(&root, &target, |root| {
            fs::write(root.join(MEMORY_FILE), &foreign).expect("write");
            edit_record(root, |record| {
                record["memory"]["sha256"] = json!(sha256_hex(&foreign));
                record["memory"]["bytes"] = json!(foreign.len());
            });
        });
        assert!(matches!(
            load_error(&target),
            StackCheckpointError::Store(StackStoreError::ForeignLineage {
                expected: 41,
                found: 42
            })
        ));
        // A record that declares another session's lineage for this file.
        let target = base.join("foreign-record");
        reseal(&root, &target, |root| {
            edit_record(root, |record| record["memory"]["lineage"] = json!(40))
        });
        assert!(matches!(
            load_error(&target),
            StackCheckpointError::Store(StackStoreError::ForeignLineage {
                expected: 40,
                found: 41
            })
        ));
        fs::remove_dir_all(&base).expect("clean");
    }

    #[test]
    fn a_model_in_served_mode_is_refused_before_claiming() {
        let (base, identity) = fixture("served");
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: VOCAB,
            width: 32,
            heads: 2,
            mlp_hidden: 24,
            context: 12,
            pattern: "rar".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 7,
            memory: None,
            select: None,
            pointer: None,
        };
        let mut model = StackModel::new(config, &Device::Cpu).expect("stack");
        model
            .set_served_representation(Some(std::sync::Arc::new(
                crate::geometric_stack::D11Interim,
            )))
            .expect("served mode");
        let root = base.join("served-checkpoint");
        match save_checkpoint(&root, &model, &identity, None) {
            Err(StackCheckpointError::ServedMode) => {}
            other => panic!("{other:?}"),
        }
        assert!(!root.exists(), "no root is claimed for a served-mode model");
    }

    #[test]
    fn a_model_with_a_transport_snap_is_refused_before_claiming() {
        let (base, identity) = fixture("snapped");
        let mut model = tiny_model();
        model
            .set_transport_snap(Some(crate::geometric_stack::TransportSnap::Icosian))
            .expect("transport snap");
        let root = base.join("snapped-checkpoint");
        match save_checkpoint(&root, &model, &identity, None) {
            Err(StackCheckpointError::TransportSnap) => {}
            other => panic!("{other:?}"),
        }
        assert!(!root.exists(), "no root is claimed for a snapped model");
        // Without the snap the same model saves.
        model.set_transport_snap(None).expect("no snap");
        save_checkpoint(&root, &model, &identity, None).expect("checkpoint");
    }

    #[test]
    fn save_refuses_an_existing_root_and_invalid_inputs_before_claiming() {
        let (base, identity) = fixture("refusals");
        let (root, model, store) = saved(&base, "checkpoint", &identity);
        let manifest = fs::read(root.join(report_output::MANIFEST_FILE)).expect("manifest");
        match save_checkpoint(&root, &model, &identity, Some(&store)) {
            Err(StackCheckpointError::Claim(error)) => {
                assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists)
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            fs::read(root.join(report_output::MANIFEST_FILE)).expect("manifest"),
            manifest
        );
        load_checkpoint(&root, &Device::Cpu).expect("the first checkpoint still loads");
        // Invalid identities are refused before any root is claimed.
        let mut upper = identity.clone();
        upper.tokenizer_sha256 = "A".repeat(64);
        let mut other_tokenizer = identity.clone();
        other_tokenizer.protocol.tokenizer_cid = format!("blake3:{}", "0".repeat(64));
        let mut outside = identity.clone();
        outside.protocol.eos_id = VOCAB as u32;
        let mut no_data = identity.clone();
        no_data.data.clear();
        let mut repeated = identity.clone();
        repeated.data.push(identity.data[0].clone());
        for (name, candidate) in [
            ("uppercase", upper),
            ("cid", other_tokenizer),
            ("eos", outside),
            ("no-data", no_data),
            ("repeated-data", repeated),
        ] {
            let target = base.join(name);
            assert!(
                matches!(
                    save_checkpoint(&target, &model, &candidate, None),
                    Err(StackCheckpointError::Identity(_))
                ),
                "{name}"
            );
            assert!(!target.exists(), "{name}: no root is claimed");
        }
        // A store whose tokens the model cannot represent is refused too.
        let mut wide = StackStore::new(41, 2).expect("store");
        wide.write(b"s", &[1], 1, &[VOCAB as u32], Update::Assert)
            .expect("write");
        let target = base.join("vocabulary");
        assert!(matches!(
            save_checkpoint(&target, &model, &identity, Some(&wide)),
            Err(StackCheckpointError::Store(
                StackStoreError::TokenOutOfVocabulary { .. }
            ))
        ));
        assert!(!target.exists());
        // The reloaded store answers the previous-distinct view it was saved with.
        let loaded = load_checkpoint(&root, &Device::Cpu).expect("load");
        let memory = loaded.memory.expect("memory");
        assert_eq!(
            memory.read(b"alice", &[3, 4], 1, HistoryView::PreviousDistinctValue),
            store.read(b"alice", &[3, 4], 1, HistoryView::PreviousDistinctValue)
        );
        fs::remove_dir_all(&base).expect("clean");
    }
}
