//! One coherent, no-following capture of a recorded corpus and the source
//! execution identity that produced it.
//!
//! New dense-present generations publish [`RECORDED_CORPUS_BINDING_FILE`]
//! last. That canonical commit record binds every provenance/corpus member by
//! exact filename, typed presence, length, and BLAKE3 content address.
//! Compilation consumes the captured bytes directly, so arbitrary cross-file
//! replacement cannot assemble an operator era and corpus from two committed
//! generations. Markerless legacy and attention-only corpora remain readable
//! through the explicitly weaker compatibility path.

use crate::observation::{self, ObservationManifest};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::{Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use uor_r4_model_source::SourceUnavailable;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::dense::DenseOperatorSpec;

pub const ATTENTION_OPERATOR_BINDING_FILE: &str = "attention_operator.json";
pub const DENSE_OPERATOR_BINDING_FILE: &str = "dense_operator.json";
pub const RECORDED_CORPUS_BINDING_FILE: &str = "recorded_corpus_binding.json";
pub const RECORDED_CORPUS_BINDING_SCHEMA: &str = "uor-r4-recorded-corpus-binding/1";
pub const RECORDED_CORPUS_COMPILE_ATTEMPT_FILE: &str = "recorded_corpus_compile_attempt.json";
pub const RECORDED_CORPUS_COMPILE_ATTEMPT_SCHEMA: &str = "uor-r4-recorded-corpus-compile-attempt/1";
pub const PLANNED_OUTPUT_RESERVED_PREFIX: &str = ".uor-r4-planned-output--";
pub const RECORDED_CORPUS_PRODUCER_COORDINATION_DIR: &str = ".uor-r4-recorded-corpus-producers";
const STREAM_BINDING_BUFFER_BYTES: usize = 64 * 1024;
const BINDING_CONTROL_MAX_BYTES: u64 = 16 * 1024;
const COMPILE_ATTEMPT_CONTROL_MAX_BYTES: u64 = 1024;
const PROVENANCE_CONTROL_MAX_BYTES: u64 = 1024 * 1024;
const METADATA_CONTROL_MAX_BYTES: u64 = 1024 * 1024;
const RESERVED_RESIDUE_MAX_ENTRIES: usize = 64;
const RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES: usize = 16;

static RECORDED_CORPUS_BINDING_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static RECORDED_CORPUS_COMPILE_ATTEMPT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Stable classification for nonblocking recorded-corpus coordination.
///
/// Callers should not duplicate the protocol's error wording when deciding
/// whether a retry is appropriate. Malformed or identity-raced coordination
/// remains terminal and deliberately does not satisfy this predicate.
pub fn is_recorded_corpus_busy(error: &SourceUnavailable) -> bool {
    is_recorded_corpus_busy_message(&error.reason)
}

/// String-boundary companion for servers which preserve their historical
/// public error type while mapping only the lower protocol's stable transient
/// classification.
pub fn is_recorded_corpus_busy_message(error: &str) -> bool {
    error.contains(" is BUSY under another active producer session")
}

/// The two canonical member pairs that a recorded-corpus binding may commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedCorpusRole {
    /// Compiler/subsample corpus: `corpus.meta` plus `corpus.records`.
    Compile,
    /// Observation corpus: `state.bin` plus `merged.bin`.
    Observation,
}

impl RecordedCorpusRole {
    fn member_names(self) -> (&'static str, &'static str) {
        match self {
            Self::Compile => ("corpus.meta", "corpus.records"),
            Self::Observation => (observation::STATE_FILE, "merged.bin"),
        }
    }
}

/// A deterministic compile-style output member with an owner-only staging
/// namespace. The enum and parser live here so every producer agrees on the
/// complete reserved namespace, even when the publisher itself lives in the
/// graph CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedOutputMember {
    AttentionOperator,
    DenseOperator,
    Artifact,
    Store,
    Calibration,
    HierarchicalCodes,
    Records,
    Hidden,
    Metadata,
}

impl PlannedOutputMember {
    pub const ALL: [Self; 7] = [
        Self::Artifact,
        Self::Store,
        Self::Calibration,
        Self::HierarchicalCodes,
        Self::Records,
        Self::Hidden,
        Self::Metadata,
    ];

    pub const REGISTERED: [Self; 9] = [
        Self::AttentionOperator,
        Self::DenseOperator,
        Self::Artifact,
        Self::Store,
        Self::Calibration,
        Self::HierarchicalCodes,
        Self::Records,
        Self::Hidden,
        Self::Metadata,
    ];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::AttentionOperator => ATTENTION_OPERATOR_BINDING_FILE,
            Self::DenseOperator => DENSE_OPERATOR_BINDING_FILE,
            Self::Artifact => "tless_artifacts.bin",
            Self::Store => "tless_store.bin",
            Self::Calibration => "hamming_calibration.json",
            Self::HierarchicalCodes => "hierarchical_codes.json",
            Self::Records => "corpus.records",
            Self::Hidden => "corpus.records.hidden",
            Self::Metadata => "corpus.meta",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCorpusFileBinding {
    name: String,
    present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blake3: Option<String>,
}

impl RecordedCorpusFileBinding {
    fn from_bytes(name: &str, bytes: Option<&[u8]>) -> Result<Self, SourceUnavailable> {
        let length = bytes
            .map(|bytes| {
                u64::try_from(bytes.len()).map_err(|_| {
                    SourceUnavailable::new(format!(
                        "recorded corpus member {name} is too large for its u64 binding length"
                    ))
                })
            })
            .transpose()?;
        Ok(Self {
            name: name.to_owned(),
            present: bytes.is_some(),
            length,
            blake3: bytes.map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex())),
        })
    }

    fn validate(
        &self,
        expected_name: &str,
        bytes: Option<&[u8]>,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if self.name != expected_name {
            return Err(SourceUnavailable::new(format!(
                "{context} binds filename {:?}, expected exact member {expected_name:?}",
                self.name
            )));
        }
        let expected = Self::from_bytes(expected_name, bytes)?;
        if self != &expected {
            return Err(SourceUnavailable::new(format!(
                "{context} does not match the typed presence, length, and BLAKE3 of {expected_name}"
            )));
        }
        Ok(())
    }

    fn validate_record(
        &self,
        expected_name: &str,
        actual: &Self,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if self.name != expected_name || actual.name != expected_name || self != actual {
            return Err(SourceUnavailable::new(format!(
                "{context} does not match the exact streamed presence, length, and BLAKE3 of {expected_name}"
            )));
        }
        Ok(())
    }

    fn validate_shape(&self, expected_name: &str, context: &str) -> Result<(), SourceUnavailable> {
        if self.name != expected_name {
            return Err(SourceUnavailable::new(format!(
                "{context} binds filename {:?}, expected exact member {expected_name:?}",
                self.name
            )));
        }
        match (self.present, self.length, self.blake3.as_deref()) {
            (false, None, None) => Ok(()),
            (true, Some(_), Some(digest))
                if digest.len() == "blake3:".len() + 64
                    && digest.starts_with("blake3:")
                    && digest["blake3:".len()..]
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) =>
            {
                Ok(())
            }
            _ => Err(SourceUnavailable::new(format!(
                "{context} has an invalid typed presence/length/BLAKE3 tuple for {expected_name}"
            ))),
        }
    }
}

/// Canonical commit record for one recorded-corpus generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedCorpusBinding {
    schema: String,
    manifest: RecordedCorpusFileBinding,
    attention_operator: RecordedCorpusFileBinding,
    dense_operator: RecordedCorpusFileBinding,
    metadata: RecordedCorpusFileBinding,
    records: RecordedCorpusFileBinding,
    hidden: RecordedCorpusFileBinding,
}

impl RecordedCorpusBinding {
    fn canonical_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(SourceUnavailable::new)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Content address of the canonical binding bytes. This is a derived
    /// corpus identity, never a source-manifest kappa.
    pub fn declared_digest(&self) -> Result<String, SourceUnavailable> {
        Ok(format!(
            "blake3:{}",
            blake3::hash(&self.canonical_bytes()?).to_hex()
        ))
    }

    fn validate_shape(&self, context: &str) -> Result<RecordedCorpusRole, SourceUnavailable> {
        self.manifest
            .validate_shape(observation::MANIFEST_FILE, context)?;
        self.attention_operator
            .validate_shape(ATTENTION_OPERATOR_BINDING_FILE, context)?;
        self.dense_operator
            .validate_shape(DENSE_OPERATOR_BINDING_FILE, context)?;
        let role = match (self.metadata.name.as_str(), self.records.name.as_str()) {
            ("corpus.meta", "corpus.records") => RecordedCorpusRole::Compile,
            (metadata, "merged.bin") if metadata == observation::STATE_FILE => {
                RecordedCorpusRole::Observation
            }
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "{context} does not name one exact registered metadata/records pair"
                )));
            }
        };
        let (metadata, records) = role.member_names();
        self.metadata.validate_shape(metadata, context)?;
        self.records.validate_shape(records, context)?;
        let expected_hidden = hidden_path(Path::new(records));
        let expected_hidden = expected_hidden.to_str().ok_or_else(|| {
            SourceUnavailable::new("registered hidden member name is not Unicode")
        })?;
        self.hidden.validate_shape(expected_hidden, context)?;
        if role == RecordedCorpusRole::Observation && self.hidden.present {
            return Err(SourceUnavailable::new(format!(
                "{context} declares a hidden stream for the observation role, which has no registered hidden member"
            )));
        }
        Ok(role)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCorpusCompileAttempt {
    schema: String,
    role: String,
}

impl RecordedCorpusCompileAttempt {
    fn compile() -> Self {
        Self {
            schema: RECORDED_CORPUS_COMPILE_ATTEMPT_SCHEMA.to_owned(),
            role: "compile".to_owned(),
        }
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(SourceUnavailable::new)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

/// Canonical bytes of the lower compile-role coordination marker. External
/// managed publishers use this exact record when classifying/excluding the
/// transient marker; re-serializing an ad-hoc JSON map could change field
/// order and falsely reject the lower protocol's own bytes.
pub fn recorded_corpus_compile_attempt_bytes() -> Result<Vec<u8>, SourceUnavailable> {
    RecordedCorpusCompileAttempt::compile().canonical_bytes()
}

fn validate_binding_role(
    binding: &RecordedCorpusBinding,
    role: RecordedCorpusRole,
    path: &Path,
) -> Result<(), SourceUnavailable> {
    let (metadata, records) = role.member_names();
    if binding.metadata.name != metadata || binding.records.name != records {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} commits canonical pair {}/{}, but this producer requires {metadata}/{records}; refusing cross-role mutation",
            path.display(),
            binding.metadata.name,
            binding.records.name
        )));
    }
    Ok(())
}

fn validate_role_inventory(root: &Path, role: RecordedCorpusRole) -> Result<(), SourceUnavailable> {
    const COMPILE_ONLY: [&str; 18] = [
        "corpus.meta",
        "corpus.records",
        "corpus.records.hidden",
        "tless_artifacts.bin",
        "tless_store.bin",
        "hamming_calibration.json",
        "hierarchical_codes.json",
        "space_manifest.json",
        "compile_report.json",
        "tokenizer_adapter.json",
        RECORDED_CORPUS_COMPILE_ATTEMPT_FILE,
        "source_compile_preflight.json",
        "source_manifest_kappa.json",
        "compiled_bundle_completion.json",
        ".compiled_bundle_stage.json",
        "graph",
        "graph-cover",
        "instruction-eval.json",
    ];
    fn compile_only_name(name: &str) -> bool {
        if name.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX) {
            return true;
        }
        if COMPILE_ONLY.contains(&name) {
            return true;
        }
        // Server/source publishers use the same stable-name + numeric owner
        // convention for their recoverable identity/completion temporaries.
        // A temp-only crash prefix is just as role-defining as its stable
        // destination and must keep an observation writer out.
        COMPILE_ONLY.iter().any(|stable| {
            let prefix = format!(".{stable}.");
            name.starts_with(&prefix)
                && (name.ends_with(".tmp")
                    || name.ends_with(".writing")
                    || name.ends_with(".replace.tmp"))
        })
    }

    if role == RecordedCorpusRole::Compile {
        for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let raw_name = entry.file_name();
            let Some(name) = raw_name.to_str().map(str::to_owned) else {
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt;
                    let bytes = raw_name.as_bytes();
                    if [
                        b"manifest.json".as_slice(),
                        b"state.bin".as_slice(),
                        b"raw-committed.bin".as_slice(),
                        b"committed.bin".as_slice(),
                        b"merged.bin".as_slice(),
                        b"stories.jsonl".as_slice(),
                        b"stories.tmp".as_slice(),
                        b"shard-".as_slice(),
                        b".manifest.json.tmp".as_slice(),
                        b".raw-committed.bin.tmp".as_slice(),
                        b".state.bin.tmp".as_slice(),
                        b".committed.bin.tmp".as_slice(),
                    ]
                    .iter()
                    .any(|prefix| bytes.starts_with(prefix))
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus root {} contains a non-UTF-8 entry in a reserved observation-role namespace",
                            root.display()
                        )));
                    }
                }
                continue;
            };
            if observation::is_observation_exclusive_entry_name(&name) {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains foreign observation-role member {name}; refusing compile mutation",
                    root.display()
                )));
            }
        }
    } else {
        for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                // An unrelated non-Unicode file remains compatible, but a
                // reserved ASCII prefix represented by non-Unicode bytes is
                // terminal on Unix. The lossy form is used only as a prefix
                // classifier and never as a path to follow.
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt;
                    let raw = entry.file_name();
                    if raw
                        .as_bytes()
                        .starts_with(PLANNED_OUTPUT_RESERVED_PREFIX.as_bytes())
                        || COMPILE_ONLY.iter().any(|stable| {
                            raw.as_bytes() == stable.as_bytes()
                                || raw.as_bytes().starts_with(format!(".{stable}.").as_bytes())
                        })
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus root {} contains a non-UTF-8 entry in a reserved compile-role namespace",
                            root.display()
                        )));
                    }
                }
                continue;
            };
            if compile_only_name(name) {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains foreign compile-role member {name}; refusing observation mutation",
                    root.display()
                )));
            }
        }
    }
    Ok(())
}

/// Parse a JSON document solely to reject duplicate object keys at every
/// nesting level. `serde_json::Value` is last-key-wins, which is unsuitable
/// for provenance records where duplicated fields could spell two arithmetic
/// eras in one document.
pub struct DuplicateRejectingJson;

impl<'de> serde::Deserialize<'de> for DuplicateRejectingJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DuplicateRejectingJson;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }

            fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(DuplicateRejectingJson)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <DuplicateRejectingJson as serde::Deserialize>::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while sequence.next_element::<DuplicateRejectingJson>()?.is_some() {}
                Ok(DuplicateRejectingJson)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut keys = std::collections::BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON field {key:?}"
                        )));
                    }
                    map.next_value::<DuplicateRejectingJson>()?;
                }
                Ok(DuplicateRejectingJson)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Reject duplicate object keys recursively before any typed/`Value` parse.
pub fn reject_duplicate_json(bytes: &[u8], context: &str) -> Result<(), SourceUnavailable> {
    serde_json::from_slice::<DuplicateRejectingJson>(bytes)
        .map(|_| ())
        .map_err(|error| SourceUnavailable::new(format!("{context}: malformed JSON: {error}")))
}

/// Exact registered source-execution pair captured beside a recorded corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusExecutionIdentity {
    pub attention_operator: Option<AttentionOperatorSpec>,
    pub dense_operator: Option<DenseOperatorSpec>,
}

/// Corpus bytes and execution provenance captured and reverified as one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusSnapshot {
    pub execution: RecordedCorpusExecutionIdentity,
    /// Exact captured `attention_operator.json` bytes, when present. These
    /// bytes are the same generation that was registry- and binding-checked;
    /// callers must not reopen the pathname to derive durable evidence.
    pub attention_operator_bytes: Option<Vec<u8>>,
    /// Exact captured `dense_operator.json` bytes, when present.
    pub dense_operator_bytes: Option<Vec<u8>>,
    pub meta_bytes: Vec<u8>,
    pub records_bytes: Vec<u8>,
    pub hidden_bytes: Option<Vec<u8>>,
    /// Exact canonical generation binding, when the producer committed one.
    pub binding: Option<RecordedCorpusBinding>,
    /// BLAKE3 CID of the exact canonical binding bytes. This is derived
    /// corpus provenance and is deliberately distinct from source kappa.
    pub binding_cid: Option<String>,
}

/// One no-follow corpus member whose complete content address was validated
/// while this exact handle remained open. Consumers may seek/read it without
/// materializing the body and must call [`Self::verify_generation`] after the
/// final read before committing derived output.
#[derive(Debug)]
pub struct VerifiedCorpusMember {
    path: PathBuf,
    file: std::fs::File,
    initial: Metadata,
    binding: RecordedCorpusFileBinding,
    position: u64,
    context: &'static str,
}

/// Bounded-memory length/content address of one exact regular member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusMemberSummary {
    pub length: u64,
    pub blake3: String,
}

impl VerifiedCorpusMember {
    pub fn len(&self) -> u64 {
        self.binding.length.unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn declared_digest(&self) -> &str {
        self.binding.blake3.as_deref().unwrap_or("")
    }

    pub fn summary(&self) -> RecordedCorpusMemberSummary {
        RecordedCorpusMemberSummary {
            length: self.len(),
            blake3: self.declared_digest().to_owned(),
        }
    }

    pub fn verify_generation(&self) -> Result<(), SourceUnavailable> {
        let final_file = self.file.metadata().map_err(SourceUnavailable::new)?;
        let final_path = std::fs::symlink_metadata(&self.path).map_err(|error| {
            SourceUnavailable::new(format!(
                "{} {} changed or disappeared: {error}",
                self.context,
                self.path.display()
            ))
        })?;
        if !final_path.file_type().is_file()
            || !opened_file_generation_matches(&self.initial, &final_file)
            || !opened_file_identity_matches(&final_path, &final_file)
        {
            return Err(SourceUnavailable::new(format!(
                "{} {} changed generation after its verified stream was opened",
                self.context,
                self.path.display()
            )));
        }
        Ok(())
    }
}

impl std::io::Read for VerifiedCorpusMember {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.len().saturating_sub(self.position);
        if remaining == 0 || buffer.is_empty() {
            return Ok(0);
        }
        let limit = usize::try_from(remaining.min(buffer.len() as u64)).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "verified corpus member read length does not fit usize",
            )
        })?;
        let read = self.file.read(&mut buffer[..limit])?;
        self.position = self
            .position
            .checked_add(read as u64)
            .ok_or_else(|| std::io::Error::other("verified corpus member position overflow"))?;
        Ok(read)
    }
}

impl std::io::Seek for VerifiedCorpusMember {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        let target = match position {
            SeekFrom::Start(target) => Some(target),
            SeekFrom::End(offset) => bounded_seek_target(self.len(), offset),
            SeekFrom::Current(offset) => bounded_seek_target(self.position, offset),
        }
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "verified corpus member seek is outside the u64 address space",
            )
        })?;
        self.file.seek(SeekFrom::Start(target))?;
        self.position = target;
        Ok(target)
    }
}

fn bounded_seek_target(base: u64, offset: i64) -> Option<u64> {
    let target = i128::from(base) + i128::from(offset);
    u64::try_from(target).ok()
}

/// Small provenance/metadata plus retained verified handles for large corpus
/// members. This is the bounded-memory source for derivation/status paths.
#[derive(Debug)]
pub struct RecordedCorpusStreamSnapshot {
    pub execution: RecordedCorpusExecutionIdentity,
    pub attention_operator_bytes: Option<Vec<u8>>,
    pub dense_operator_bytes: Option<Vec<u8>>,
    pub meta_bytes: Vec<u8>,
    pub records: VerifiedCorpusMember,
    pub hidden: Option<VerifiedCorpusMember>,
    pub binding_cid: Option<String>,
    root: PathBuf,
    provenance: ProvenanceBytes,
    meta_path: PathBuf,
    binding_path: PathBuf,
    binding_bytes: Option<Vec<u8>>,
    marker_path: PathBuf,
    marker_bytes: Option<Vec<u8>>,
    hidden_path: PathBuf,
}

impl RecordedCorpusStreamSnapshot {
    /// Materialize the record stream required by the existing compiler while
    /// retaining opaque model-width hidden rows as a verified summary only.
    /// The compiler consumes hidden rows exclusively at its fixed `D` width;
    /// GPT-2 source rows (for example width 768) are provenance-bound but are
    /// not duplicated into a multi-gigabyte `Vec` that the compiler discards.
    pub fn materialize_compiler_corpus_bytes(
        &mut self,
    ) -> Result<(Vec<u8>, Option<Vec<u8>>), SourceUnavailable> {
        fn read_member(
            member: &mut VerifiedCorpusMember,
            label: &str,
        ) -> Result<Vec<u8>, SourceUnavailable> {
            let length = usize::try_from(member.len()).map_err(|_| {
                SourceUnavailable::new(format!(
                    "{label} length {} cannot be represented on this host",
                    member.len()
                ))
            })?;
            member
                .seek(SeekFrom::Start(0))
                .map_err(SourceUnavailable::new)?;
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(length).map_err(|error| {
                SourceUnavailable::new(format!(
                    "cannot reserve {length} bytes for {label}: {error}"
                ))
            })?;
            bytes.resize(length, 0);
            member.read_exact(&mut bytes).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{label} ended before its declared {length} bytes: {error}"
                ))
            })?;
            let mut extra = [0u8; 1];
            if member.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
                return Err(SourceUnavailable::new(format!(
                    "{label} grew beyond its declared {length} bytes"
                )));
            }
            Ok(bytes)
        }

        let records = read_member(&mut self.records, "recorded corpus records")?;
        let hidden = match self.hidden.as_mut() {
            Some(hidden) => {
                let rows = self
                    .meta_bytes
                    .get(0..8)
                    .and_then(|bytes| bytes.try_into().ok())
                    .map(u64::from_le_bytes);
                let runtime_hidden_length = rows.and_then(|rows| {
                    rows.checked_mul(uor_r4_core::transformerless::compiler::D as u64)?
                        .checked_mul(std::mem::size_of::<f32>() as u64)
                });
                if runtime_hidden_length == Some(hidden.len()) {
                    Some(read_member(
                        hidden,
                        "recorded corpus compiler-width hidden rows",
                    )?)
                } else {
                    None
                }
            }
            None => None,
        };
        self.verify_generation()?;
        Ok((records, hidden))
    }

    /// Final generation check after a consumer finishes reading/seeking every
    /// retained member and before it publishes any derived generation.
    pub fn verify_generation(&self) -> Result<(), SourceUnavailable> {
        self.records.verify_generation()?;
        if let Some(hidden) = self.hidden.as_ref() {
            hidden.verify_generation()?;
        } else {
            verify_captured_optional_regular_file(
                &self.hidden_path,
                "recorded corpus hidden stream",
                None,
            )?;
        }
        verify_captured_optional_control_file(
            &self.meta_path,
            "recorded corpus metadata",
            Some(&self.meta_bytes),
            METADATA_CONTROL_MAX_BYTES,
        )?;
        verify_provenance(&self.root, &self.provenance)?;
        verify_captured_optional_control_file(
            &self.binding_path,
            "recorded corpus generation binding",
            self.binding_bytes.as_deref(),
            BINDING_CONTROL_MAX_BYTES,
        )?;
        verify_captured_optional_control_file(
            &self.marker_path,
            "recorded corpus compile-attempt marker",
            self.marker_bytes.as_deref(),
            COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        )?;
        require_no_binding_temporaries(&self.root)
    }
}

/// Nonblocking, process-crash-safe exclusive guard for one canonical recorded
/// corpus root.
///
/// The permanent sibling inode is coordination metadata, not corpus payload.
/// Every producer acquires any writer-specific outer session first and this
/// guard second, then holds both from the first corpus/provenance mutation
/// through final [`publish_binding`]. No producer may acquire another outer
/// session or an independent second recorded-corpus guard while this value is
/// live. The sole multi-root exception is
/// [`RecordedCorpusProducerHandoff`], which canonicalizes and acquires its
/// complete set in one sorted, failure-atomic operation.
///
/// This is a cooperative repository-writer protocol, not a claim that every
/// legacy pathname write is mutation-proof against an arbitrary process that
/// ignores the coordination inode. Supported writers must hold the guard for
/// their complete mutation interval. Retained parent/root handles make the
/// transaction commit primitives and verified reads no-follow and
/// generation-bound, so a pathname replacement is detected or the commit
/// remains attached to the retained inode.
#[derive(Debug)]
pub struct RecordedCorpusProducerGuard {
    root: PathBuf,
    _file: std::fs::File,
    root_file: Option<std::fs::File>,
    _coordination_file: std::fs::File,
    _parent_file: std::fs::File,
}

impl RecordedCorpusProducerGuard {
    /// Try to acquire exclusive producer ownership without waiting. A live
    /// cooperating producer returns an explicit `BUSY` error; malformed,
    /// symlinked, special, or identity-raced coordination entries fail closed.
    pub fn try_acquire(root: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        Self::try_acquire_with_root_hook(root.as_ref(), || {})
    }

    fn try_acquire_with_root_hook<F>(
        root: &Path,
        after_root_open: F,
    ) -> Result<Self, SourceUnavailable>
    where
        F: FnOnce(),
    {
        let root_name = root.file_name().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus producer subject {} must end in one exact directory name, not '.', '..', or a filesystem root",
                root.display()
            ))
        })?;
        if root_name == OsStr::new(".") || root_name == OsStr::new("..") {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer subject {} cannot use '.' or '..' as its root name",
                root.display()
            )));
        }
        if root_name == OsStr::new(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer root name {RECORDED_CORPUS_PRODUCER_COORDINATION_DIR:?} is reserved for sibling coordination"
            )));
        }
        let requested_parent = root
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let canonical_parent = std::fs::canonicalize(requested_parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer parent {} cannot be canonicalized: {error}",
                requested_parent.display()
            ))
        })?;
        let parent_file = open_directory_nofollow(
            &canonical_parent,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        let requested_root = canonical_parent.join(root_name);
        let root_file = match std::fs::symlink_metadata(&requested_root) {
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &requested_root,
                "recorded corpus producer root",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus producer root {} is not a real non-symlink directory",
                    requested_root.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
        };
        if let Some(file) = root_file.as_ref() {
            verify_directory_handle(&requested_root, file, "recorded corpus producer root")?;
        }
        after_root_open();
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_optional_root_generation(&requested_root, root_file.as_ref())?;
        let canonical = match root_file.as_ref() {
            Some(file) => canonical_existing_root_path(&canonical_parent, &requested_root, file)?,
            None => requested_root,
        };
        let canonical_name = canonical.file_name().ok_or_else(|| {
            SourceUnavailable::new("canonical recorded corpus root has no final component")
        })?;
        let coordination_path = canonical_parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        let coordination_file =
            open_or_create_coordination_directory(&canonical_parent, &parent_file)?;
        let lock_path = coordination_path.join(canonical_name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options
                .mode(0o600)
                .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(&lock_path).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} cannot be opened without following links: {error}",
                lock_path.display()
            ))
        })?;
        let initial_path = std::fs::symlink_metadata(&lock_path).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} cannot be inspected: {error}",
                lock_path.display()
            ))
        })?;
        let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
        if !initial_path.file_type().is_file()
            || !initial_file.file_type().is_file()
            || !opened_file_identity_matches(&initial_path, &initial_file)
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} is not one stable regular non-symlink inode",
                lock_path.display()
            )));
        }
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} is BUSY under another active producer session",
                    canonical.display()
                )));
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus producer coordination {} cannot be locked: {error}",
                    lock_path.display()
                )));
            }
        }
        let final_path = std::fs::symlink_metadata(&lock_path).map_err(SourceUnavailable::new)?;
        let final_file = file.metadata().map_err(SourceUnavailable::new)?;
        if !final_path.file_type().is_file()
            || !final_file.file_type().is_file()
            || !opened_file_identity_matches(&initial_file, &final_file)
            || !opened_file_identity_matches(&final_path, &final_file)
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination {} changed identity or type while its lock was acquired",
                lock_path.display()
            )));
        }
        verify_directory_handle(
            &canonical_parent,
            &parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &coordination_path,
            &coordination_file,
            "recorded corpus producer coordination directory",
        )?;
        verify_optional_root_generation(&canonical, root_file.as_ref())?;
        Ok(Self {
            root: canonical,
            _file: file,
            root_file,
            _coordination_file: coordination_file,
            _parent_file: parent_file,
        })
    }

    /// Canonical root protected by this guard.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the logical root existed when acquired (or has since been
    /// created by [`Self::ensure_root`]).
    pub fn root_exists(&self) -> bool {
        self.root_file.is_some()
    }

    /// Validate every stable/reserved binding entry before a caller mutates
    /// any generation member. Returns `true` when a canonical, fully parsed
    /// publication temporary needs exact-generation recovery by
    /// [`publish_binding`]. Non-authoritative `.writing` residue may be
    /// reclaimed only by that publisher while this guard remains live.
    pub fn preflight_publication_namespace(&self) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        let stable_path = binding_path(&self.root);
        if let Some(bytes) = capture_optional_control_file(
            &stable_path,
            "recorded corpus generation binding",
            BINDING_CONTROL_MAX_BYTES,
        )? {
            let _ = parse_binding_bytes(&stable_path, &bytes)?;
        }
        let residues = binding_publication_residues(&self.root, true)?;
        self.verify_root()?;
        Ok(!residues.temporaries.is_empty())
    }

    /// Validate the binding namespace and require every stable or recoverable
    /// commit record to name the caller's canonical member pair. This keeps an
    /// observation writer from mutating a compile/subsample root (and vice
    /// versa) before binding-last publication discovers the role conflict.
    pub fn preflight_publication_namespace_for(
        &self,
        role: RecordedCorpusRole,
    ) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        validate_role_inventory(&self.root, role)?;
        validate_compile_attempt_namespace(&self.root, role)?;
        let stable_path = binding_path(&self.root);
        if let Some(bytes) = capture_optional_control_file(
            &stable_path,
            "recorded corpus generation binding",
            BINDING_CONTROL_MAX_BYTES,
        )? {
            let binding = parse_binding_bytes(&stable_path, &bytes)?;
            validate_binding_role(&binding, role, &stable_path)?;
        }
        let residues = binding_publication_residues(&self.root, true)?;
        for (path, bytes) in &residues.temporaries {
            let binding = parse_binding_bytes(path, bytes)?;
            validate_binding_role(&binding, role, path)?;
        }
        self.verify_root()?;
        Ok(!residues.temporaries.is_empty())
    }

    /// Reclaim only non-authoritative binding `.writing` inodes after the
    /// complete role namespace has been validated under this retained root.
    /// A canonical `.tmp` is an authoritative recoverable commit candidate
    /// and is never discarded here; the caller must promote it through
    /// [`publish_binding`] before deciding whether the generation is bound.
    pub fn reclaim_uncommitted_binding_writings_for(
        &self,
        role: RecordedCorpusRole,
    ) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        if self.preflight_publication_namespace_for(role)? {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains an authoritative recoverable binding temporary; refusing to discard publication evidence",
                self.root.display()
            )));
        }
        let residues = binding_publication_residues(&self.root, true)?;
        for (path, bytes) in residues.writings {
            remove_exact_owned_binding_residue(
                self,
                &path,
                &bytes,
                "non-authoritative binding staging",
            )?;
        }
        self.sync_owned_root()?;
        self.verify_root()
    }

    /// Durably declare a compile-style mutation attempt before the first
    /// sidecar or corpus member is published. The fixed marker disambiguates
    /// sidecar-only crash prefixes from observation roots without changing
    /// the canonical recorded-corpus binding schema.
    pub fn begin_compile_attempt(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        self.preflight_planned_output_scope(&PlannedOutputMember::REGISTERED)?;
        let _ = self.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
        publish_compile_attempt_marker(self)?;
        self.verify_root()
    }

    /// Remove the compile-attempt marker only after a stable same-role corpus
    /// binding exists. A crash before removal leaves a readable committed
    /// generation plus explicit retry evidence; removal and directory sync
    /// are idempotent under the retained guard.
    pub fn finish_compile_attempt(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        let stable_binding = binding_path(&self.root);
        let bytes = capture_optional_control_file(
            &stable_binding,
            "recorded corpus generation binding",
            BINDING_CONTROL_MAX_BYTES,
        )?
        .ok_or_else(|| {
            SourceUnavailable::new(
                "cannot finish recorded-corpus compile attempt before its stable generation binding",
            )
        })?;
        let binding = parse_binding_bytes(&stable_binding, &bytes)?;
        validate_binding_role(&binding, RecordedCorpusRole::Compile, &stable_binding)?;
        preflight_binding_evidence_matches_current(
            self,
            &self.root.join("corpus.meta"),
            &self.root.join("corpus.records"),
        )?;
        finish_compile_attempt_marker(self)?;
        self.verify_root()
    }

    /// Whether this compile-role root carries the exact durable attempt
    /// marker. The namespace is fully validated before the answer is
    /// returned, so malformed or special residue remains terminal.
    pub fn compile_attempt_active(&self) -> Result<bool, SourceUnavailable> {
        self.verify_root()?;
        validate_compile_attempt_namespace(&self.root, RecordedCorpusRole::Compile)?;
        let path = compile_attempt_path(&self.root);
        let active = capture_optional_control_file(
            &path,
            "recorded corpus compile-attempt marker",
            COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        )?
        .map(|bytes| parse_compile_attempt_bytes(&path, &bytes))
        .transpose()?
        .is_some();
        self.verify_root()?;
        Ok(active)
    }

    /// Validate the complete shared planned-output staging namespace and
    /// reject residue owned by any member outside `allowed`. All recognized
    /// entries are opened nofollow and bound to one regular inode before the
    /// owner filter is applied; malformed or special entries are terminal.
    pub fn preflight_planned_output_scope(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        let residues = self.planned_output_residues()?;
        if let Some(residue) = residues
            .iter()
            .find(|residue| !allowed.contains(&residue.member))
        {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains reserved staging residue {} for foreign member {}; refusing a partial-plan publication",
                self.root.display(),
                residue.path.display(),
                residue.member.stable_name()
            )));
        }
        Ok(())
    }

    /// Reject stable deterministic members outside a partial producer's
    /// declared plan. Staging ownership alone is insufficient: a subsample
    /// must not commit new corpus bytes beside stale compile artifacts.
    pub fn preflight_planned_output_stable_scope(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        for member in PlannedOutputMember::ALL {
            if allowed.contains(&member) {
                continue;
            }
            let path = self.root.join(member.stable_name());
            match std::fs::symlink_metadata(&path) {
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus root {} contains foreign stable planned member {}; refusing a partial-plan publication",
                        self.root.display(),
                        member.stable_name()
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(SourceUnavailable::new(error)),
            }
        }
        self.verify_root()
    }

    /// Require a deterministic compile-style destination to contain only the
    /// exact members that this command can reproduce, plus the shared
    /// execution/binding/attempt namespaces. This is deliberately stricter
    /// than the broad Compile role: source bundles and server stages carry
    /// additional valid compile-role leaves that a corpus derivation must not
    /// silently preserve beside a newly committed generation.
    pub fn preflight_deterministic_compile_inventory(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        self.preflight_deterministic_compile_inventory_inner(allowed, false)
    }

    /// Ready-only inventory for an already exact, stable deterministic
    /// compile generation. The two well-known downstream graph directories
    /// may coexist because this path performs no corpus/member recovery or
    /// publication. Every other foreign leaf remains terminal, and either
    /// graph entry must itself be a real non-symlink directory.
    pub fn preflight_ready_deterministic_compile_inventory(
        &self,
        allowed: &[PlannedOutputMember],
    ) -> Result<(), SourceUnavailable> {
        self.preflight_deterministic_compile_inventory_inner(allowed, true)
    }

    fn preflight_deterministic_compile_inventory_inner(
        &self,
        allowed: &[PlannedOutputMember],
        allow_ready_graph_outputs: bool,
    ) -> Result<(), SourceUnavailable> {
        self.verify_root()?;
        self.preflight_planned_output_scope(allowed)?;
        let _ = self.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
        for entry in std::fs::read_dir(&self.root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let os_name = entry.file_name();
            let Some(name) = os_name.to_str() else {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-Unicode entry outside this deterministic compile plan",
                    self.root.display()
                )));
            };
            let stable_allowed = allowed.iter().any(|member| member.stable_name() == name);
            let shared_stable = matches!(
                name,
                ATTENTION_OPERATOR_BINDING_FILE
                    | DENSE_OPERATOR_BINDING_FILE
                    | RECORDED_CORPUS_BINDING_FILE
                    | RECORDED_CORPUS_COMPILE_ATTEMPT_FILE
            );
            let shared_reserved = name.starts_with(&format!(".{RECORDED_CORPUS_BINDING_FILE}."))
                || name.starts_with(&format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}."))
                || name.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX);
            if stable_allowed || shared_stable || shared_reserved {
                continue;
            }
            if allow_ready_graph_outputs && matches!(name, "graph" | "graph-cover") {
                let metadata =
                    std::fs::symlink_metadata(entry.path()).map_err(SourceUnavailable::new)?;
                if metadata.file_type().is_dir() {
                    continue;
                }
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} contains ready-only downstream entry {name}, but it is not a real non-symlink directory",
                    self.root.display()
                )));
            }
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unowned compile-style entry {name}; refusing deterministic corpus publication",
                self.root.display()
            )));
        }
        self.verify_root()
    }

    /// Whether a validated regular staging residue exists for `member`.
    pub fn has_planned_output_residue(
        &self,
        member: PlannedOutputMember,
    ) -> Result<bool, SourceUnavailable> {
        Ok(self
            .planned_output_residues()?
            .iter()
            .any(|residue| residue.member == member))
    }

    /// Reclaim only validated regular residue owned by `member`. The initial
    /// generation is rechecked through a nofollow handle immediately before
    /// unlink so process death recovery is O(1) in staged payload size.
    pub fn reclaim_planned_output_residues(
        &self,
        member: PlannedOutputMember,
    ) -> Result<(), SourceUnavailable> {
        self.reclaim_planned_output_residues_with_hook(member, || {})
    }

    fn reclaim_planned_output_residues_with_hook<F>(
        &self,
        member: PlannedOutputMember,
        before_unlink: F,
    ) -> Result<(), SourceUnavailable>
    where
        F: FnOnce(),
    {
        let residues = self.planned_output_residues()?;
        before_unlink();
        for residue in residues
            .into_iter()
            .filter(|residue| residue.member == member)
        {
            self.remove_owned_planned_output_residue(&residue)?;
        }
        self.sync_owned_root()?;
        self.verify_root()
    }

    fn remove_owned_planned_output_residue(
        &self,
        residue: &PlannedOutputResidue,
    ) -> Result<(), SourceUnavailable> {
        let name = residue.path.file_name().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "planned-output staging entry {} has no leaf name",
                residue.path.display()
            ))
        })?;
        let file = self.open_owned_entry_for_read(name)?;
        let metadata = file.metadata().map_err(SourceUnavailable::new)?;
        if !metadata.file_type().is_file()
            || !opened_file_generation_matches(&residue.metadata, &metadata)
        {
            return Err(SourceUnavailable::new(format!(
                "planned-output staging entry {} changed type or generation before guarded recovery",
                residue.path.display()
            )));
        }
        self.unlink_owned_entry(name)
    }

    fn planned_output_residues(&self) -> Result<Vec<PlannedOutputResidue>, SourceUnavailable> {
        self.verify_root()?;
        let mut residues = Vec::new();
        let mut reserved_count = 0usize;
        for entry in std::fs::read_dir(&self.root).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let Some(name) = reserved_planned_output_entry_name(&self.root, &entry.file_name())?
            else {
                continue;
            };
            let member = PlannedOutputMember::REGISTERED
                .into_iter()
                .find(|member| planned_output_staging_name(*member, &name))
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus root {} contains unrecognized reserved planned-output staging entry {name:?}",
                        self.root.display()
                    ))
                })?;
            reserved_count = reserved_count
                .checked_add(1)
                .ok_or_else(|| SourceUnavailable::new("planned-output residue count overflow"))?;
            if reserved_count > RESERVED_RESIDUE_MAX_ENTRIES {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} exceeds the fixed {RESERVED_RESIDUE_MAX_ENTRIES}-entry planned-output residue ceiling",
                    self.root.display()
                )));
            }
            let path = entry.path();
            let mut options = OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let file = options.open(&path).map_err(|error| {
                SourceUnavailable::new(format!(
                    "planned-output staging entry {} cannot be opened without following links: {error}",
                    path.display()
                ))
            })?;
            let path_metadata = std::fs::symlink_metadata(&path).map_err(SourceUnavailable::new)?;
            let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
            if !path_metadata.file_type().is_file()
                || !file_metadata.file_type().is_file()
                || !opened_file_identity_matches(&path_metadata, &file_metadata)
            {
                return Err(SourceUnavailable::new(format!(
                    "planned-output staging entry {} is not one stable regular non-symlink inode",
                    path.display()
                )));
            }
            residues.push(PlannedOutputResidue {
                member,
                path,
                metadata: file_metadata,
            });
        }
        residues.sort_by(|left, right| left.path.cmp(&right.path));
        self.verify_root()?;
        Ok(residues)
    }

    /// Create and bind a previously absent logical root after all caller
    /// semantic/input preflights have succeeded. Existing roots are merely
    /// reverified. This is the only supported transition from a path lock to
    /// a mutable recorded-corpus directory.
    pub fn ensure_root(&mut self) -> Result<&Path, SourceUnavailable> {
        self.ensure_root_with_hook(|| {})
    }

    fn ensure_root_with_hook<F>(
        &mut self,
        after_parent_check: F,
    ) -> Result<&Path, SourceUnavailable>
    where
        F: FnOnce(),
    {
        self.verify_parent()?;
        after_parent_check();
        if self.root_file.is_none() {
            #[cfg(unix)]
            {
                use std::os::fd::{AsRawFd, FromRawFd};

                let root_name = self.root.file_name().ok_or_else(|| {
                    SourceUnavailable::new("recorded corpus producer root has no leaf name")
                })?;
                let root_name = owned_leaf_cstring(root_name)?;
                // SAFETY: the retained parent is a live directory descriptor
                // and `root_name` is one validated NUL-free leaf.
                let result = unsafe {
                    libc::mkdirat(self._parent_file.as_raw_fd(), root_name.as_ptr(), 0o700)
                };
                if result != 0 {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus producer root {} cannot be created under exclusive ownership: {}",
                        self.root.display(),
                        std::io::Error::last_os_error()
                    )));
                }
                self._parent_file
                    .sync_all()
                    .map_err(SourceUnavailable::new)?;
                // SAFETY: the same retained parent and leaf identify the
                // directory just created; a successful fd is immediately
                // transferred into `File` ownership.
                let fd = unsafe {
                    libc::openat(
                        self._parent_file.as_raw_fd(),
                        root_name.as_ptr(),
                        libc::O_RDONLY
                            | libc::O_DIRECTORY
                            | libc::O_NOFOLLOW
                            | libc::O_CLOEXEC
                            | libc::O_NONBLOCK,
                    )
                };
                if fd < 0 {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus producer root {} cannot be reopened through its retained parent: {}",
                        self.root.display(),
                        std::io::Error::last_os_error()
                    )));
                }
                // SAFETY: `openat` returned a new owned descriptor.
                let root_file = unsafe { std::fs::File::from_raw_fd(fd) };
                let metadata = root_file.metadata().map_err(SourceUnavailable::new)?;
                if !metadata.file_type().is_dir() {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus producer root {} is not a retained directory",
                        self.root.display()
                    )));
                }
                self.root_file = Some(root_file);
            }
            #[cfg(not(unix))]
            {
                return Err(owned_root_relative_unsupported());
            }
        }
        self.verify_root()?;
        Ok(&self.root)
    }

    fn verify_parent(&self) -> Result<(), SourceUnavailable> {
        verify_directory_handle(
            self.root.parent().ok_or_else(|| {
                SourceUnavailable::new("recorded corpus producer root has no canonical parent")
            })?,
            &self._parent_file,
            "recorded corpus producer canonical parent",
        )?;
        verify_directory_handle(
            &self
                .root
                .parent()
                .expect("validated canonical root parent")
                .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR),
            &self._coordination_file,
            "recorded corpus producer coordination directory",
        )
    }

    fn verify_root(&self) -> Result<(), SourceUnavailable> {
        let file = self.root_file.as_ref().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus producer root {} is still absent; call ensure_root only after semantic preflight and before mutation",
                self.root.display()
            ))
        })?;
        verify_directory_handle(&self.root, file, "recorded corpus producer root")
    }

    /// Revalidate the guarded root handle before a higher-level producer
    /// publishes another generation member.
    pub fn verify_owned_root(&self) -> Result<(), SourceUnavailable> {
        self.verify_root()
    }

    /// Create one new regular staging leaf relative to the retained root
    /// directory handle. This cannot be redirected by replacing `self.root`.
    pub fn create_new_owned_entry(&self, name: &OsStr) -> Result<std::fs::File, SourceUnavailable> {
        validate_owned_leaf(name)?;
        #[cfg(unix)]
        {
            use std::os::fd::{AsRawFd, FromRawFd};

            let name = owned_leaf_cstring(name)?;
            let root = self
                .root_file
                .as_ref()
                .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?;
            // SAFETY: `root` is a live directory descriptor, `name` is a
            // NUL-free single leaf, and a successful fd is immediately owned.
            let fd = unsafe {
                libc::openat(
                    root.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC
                        | libc::O_NONBLOCK,
                    0o600,
                )
            };
            if fd < 0 {
                return Err(SourceUnavailable::new(std::io::Error::last_os_error()));
            }
            // SAFETY: `openat` returned a new owned descriptor.
            let file = unsafe { std::fs::File::from_raw_fd(fd) };
            let metadata = file.metadata().map_err(SourceUnavailable::new)?;
            if !metadata.file_type().is_file() {
                return Err(SourceUnavailable::new(format!(
                    "owned staging entry {name:?} is not a regular file"
                )));
            }
            Ok(file)
        }
        #[cfg(not(unix))]
        {
            Err(owned_root_relative_unsupported())
        }
    }

    /// Create a no-clobber hard link between two retained-root leaves.
    pub fn link_owned_entry(&self, from: &OsStr, to: &OsStr) -> Result<(), SourceUnavailable> {
        validate_owned_leaf(from)?;
        validate_owned_leaf(to)?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;

            let from = owned_leaf_cstring(from)?;
            let to = owned_leaf_cstring(to)?;
            let root = self
                .root_file
                .as_ref()
                .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?;
            // SAFETY: both names are validated single leaves under one live
            // retained directory descriptor.
            let result = unsafe {
                libc::linkat(
                    root.as_raw_fd(),
                    from.as_ptr(),
                    root.as_raw_fd(),
                    to.as_ptr(),
                    0,
                )
            };
            if result != 0 {
                return Err(SourceUnavailable::new(std::io::Error::last_os_error()));
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(owned_root_relative_unsupported())
        }
    }

    /// Atomically rename one retained-root leaf over another.
    pub fn rename_owned_entry(&self, from: &OsStr, to: &OsStr) -> Result<(), SourceUnavailable> {
        validate_owned_leaf(from)?;
        validate_owned_leaf(to)?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;

            let from = owned_leaf_cstring(from)?;
            let to = owned_leaf_cstring(to)?;
            let root = self
                .root_file
                .as_ref()
                .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?;
            // SAFETY: both names are validated leaves under the retained fd.
            let result = unsafe {
                libc::renameat(
                    root.as_raw_fd(),
                    from.as_ptr(),
                    root.as_raw_fd(),
                    to.as_ptr(),
                )
            };
            if result != 0 {
                return Err(SourceUnavailable::new(std::io::Error::last_os_error()));
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(owned_root_relative_unsupported())
        }
    }

    /// Unlink one retained-root leaf without following it.
    pub fn unlink_owned_entry(&self, name: &OsStr) -> Result<(), SourceUnavailable> {
        validate_owned_leaf(name)?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;

            let name = owned_leaf_cstring(name)?;
            let root = self
                .root_file
                .as_ref()
                .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?;
            // SAFETY: `name` is one validated leaf and flags=0 refuses dirs.
            let result = unsafe { libc::unlinkat(root.as_raw_fd(), name.as_ptr(), 0) };
            if result != 0 {
                return Err(SourceUnavailable::new(std::io::Error::last_os_error()));
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(owned_root_relative_unsupported())
        }
    }

    /// Durably synchronize the retained root directory inode.
    pub fn sync_owned_root(&self) -> Result<(), SourceUnavailable> {
        self.root_file
            .as_ref()
            .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?
            .sync_all()
            .map_err(SourceUnavailable::new)
    }

    /// Allocation-free exact-byte verification through the retained root.
    pub fn verify_owned_entry_bytes(
        &self,
        name: &OsStr,
        expected: &[u8],
        max_bytes: u64,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if !self.verify_optional_owned_entry_bytes(name, expected, max_bytes, context)? {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} is absent"
            )));
        }
        Ok(())
    }

    /// Verify an optional exact-byte leaf. `false` means only exact ENOENT.
    pub fn verify_optional_owned_entry_bytes(
        &self,
        name: &OsStr,
        expected: &[u8],
        max_bytes: u64,
        context: &str,
    ) -> Result<bool, SourceUnavailable> {
        if expected.len() as u64 > max_bytes {
            return Err(SourceUnavailable::new(format!(
                "{context} exceeds the registered {max_bytes}-byte schema cap"
            )));
        }
        let Some(mut file) = self.open_optional_owned_entry_for_read(name)? else {
            return Ok(false);
        };
        let initial = file.metadata().map_err(SourceUnavailable::new)?;
        if !initial.file_type().is_file() || initial.len() != expected.len() as u64 {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} has the wrong type or length"
            )));
        }
        verify_file_bytes_without_path(&mut file, expected, &initial, context)?;
        let current = self.open_owned_entry_for_read(name)?;
        let current = current.metadata().map_err(SourceUnavailable::new)?;
        if !opened_file_identity_matches(&initial, &current) {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} changed identity after verification"
            )));
        }
        Ok(true)
    }

    /// Streaming length+BLAKE3 verification through the retained root.
    pub fn verify_owned_entry_summary(
        &self,
        name: &OsStr,
        expected: &RecordedCorpusMemberSummary,
        context: &str,
    ) -> Result<(), SourceUnavailable> {
        if !self.verify_optional_owned_entry_summary(name, expected, context)? {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} is absent"
            )));
        }
        Ok(())
    }

    /// Verify an optional streaming summary. `false` means only exact ENOENT.
    pub fn verify_optional_owned_entry_summary(
        &self,
        name: &OsStr,
        expected: &RecordedCorpusMemberSummary,
        context: &str,
    ) -> Result<bool, SourceUnavailable> {
        let Some(mut file) = self.open_optional_owned_entry_for_read(name)? else {
            return Ok(false);
        };
        let initial = file.metadata().map_err(SourceUnavailable::new)?;
        if !initial.file_type().is_file() || initial.len() != expected.length {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} has the wrong type or length"
            )));
        }
        let mut remaining = expected.length;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; STREAM_BINDING_BUFFER_BYTES];
        while remaining != 0 {
            let limit = usize::try_from(remaining.min(buffer.len() as u64))
                .map_err(SourceUnavailable::new)?;
            let read = file
                .read(&mut buffer[..limit])
                .map_err(SourceUnavailable::new)?;
            if read == 0 {
                return Err(SourceUnavailable::new(format!(
                    "{context} ended before its declared length"
                )));
            }
            hasher.update(&buffer[..read]);
            remaining -= read as u64;
        }
        let mut extra = [0u8; 1];
        if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} grew beyond its declared length"
            )));
        }
        let final_metadata = file.metadata().map_err(SourceUnavailable::new)?;
        let digest = format!("blake3:{}", hasher.finalize().to_hex());
        if !opened_file_generation_matches(&initial, &final_metadata) || digest != expected.blake3 {
            return Err(SourceUnavailable::new(format!(
                "{context} does not match the exact retained-root generation summary"
            )));
        }
        let current = self.open_owned_entry_for_read(name)?;
        let current = current.metadata().map_err(SourceUnavailable::new)?;
        if !opened_file_identity_matches(&initial, &current) {
            return Err(SourceUnavailable::new(format!(
                "{context} retained-root entry {name:?} changed identity after verification"
            )));
        }
        Ok(true)
    }

    fn open_owned_entry_for_read(&self, name: &OsStr) -> Result<std::fs::File, SourceUnavailable> {
        self.open_optional_owned_entry_for_read(name)?
            .ok_or_else(|| {
                SourceUnavailable::new(format!("retained-root entry {name:?} is absent"))
            })
    }

    fn open_optional_owned_entry_for_read(
        &self,
        name: &OsStr,
    ) -> Result<Option<std::fs::File>, SourceUnavailable> {
        validate_owned_leaf(name)?;
        #[cfg(unix)]
        {
            use std::os::fd::{AsRawFd, FromRawFd};

            let name = owned_leaf_cstring(name)?;
            let root = self
                .root_file
                .as_ref()
                .ok_or_else(|| SourceUnavailable::new("recorded corpus producer root is absent"))?;
            // SAFETY: `root` and `name` satisfy the openat contract; the new
            // descriptor is transferred immediately into `File`.
            let fd = unsafe {
                libc::openat(
                    root.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
                )
            };
            if fd < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(SourceUnavailable::new(error));
            }
            // SAFETY: `openat` returned a new owned descriptor.
            Ok(Some(unsafe { std::fs::File::from_raw_fd(fd) }))
        }
        #[cfg(not(unix))]
        {
            Err(owned_root_relative_unsupported())
        }
    }

    /// Compare another directory to the guarded root by opened directory
    /// identity, not caller spelling. This catches case/Unicode aliases on
    /// filesystems whose canonicalize operation preserves the input case.
    pub fn protects_directory(&self, directory: &Path) -> Result<bool, SourceUnavailable> {
        if self.root_file.is_none() {
            self.verify_parent()?;
            verify_optional_root_generation(&self.root, None)?;
            return Ok(false);
        }
        self.verify_root()?;
        let other = open_directory_nofollow(directory, "recorded corpus comparison directory")?;
        #[cfg(unix)]
        {
            let guarded = self
                .root_file
                .as_ref()
                .expect("verified recorded corpus root")
                .metadata()
                .map_err(SourceUnavailable::new)?;
            let other = other.metadata().map_err(SourceUnavailable::new)?;
            Ok(opened_file_identity_matches(&guarded, &other))
        }
        #[cfg(not(unix))]
        {
            let canonical = std::fs::canonicalize(directory).map_err(SourceUnavailable::new)?;
            Ok(canonical == self.root)
        }
    }
}

/// One canonical logical-root generation captured without creating producer
/// coordination state.
///
/// `exists() == false` is a typed absence pinned by the retained parent
/// directory handle. An existing generation retains the exact directory inode,
/// so a same-byte remove/recreate is still a failed compare-and-swap.
#[derive(Debug)]
pub struct RecordedCorpusRootGeneration {
    root: PathBuf,
    root_file: Option<std::fs::File>,
    parent: PathBuf,
    parent_file: std::fs::File,
}

impl RecordedCorpusRootGeneration {
    /// Capture one canonical root or its typed absence using read-only,
    /// no-follow handles. The parent must already exist.
    pub fn capture(root: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        let requested = root.as_ref();
        let root_name = requested.file_name().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus generation subject {} has no final component",
                requested.display()
            ))
        })?;
        validate_owned_leaf(root_name)?;
        if root_name == OsStr::new(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root name {RECORDED_CORPUS_PRODUCER_COORDINATION_DIR:?} is reserved for sibling coordination"
            )));
        }
        let requested_parent = requested
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = std::fs::canonicalize(requested_parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus generation parent {} cannot be canonicalized: {error}",
                requested_parent.display()
            ))
        })?;
        let parent_file =
            open_directory_nofollow(&parent, "recorded corpus generation canonical parent")?;
        verify_directory_handle(
            &parent,
            &parent_file,
            "recorded corpus generation canonical parent",
        )?;
        let requested_root = parent.join(root_name);
        let root_file = match std::fs::symlink_metadata(&requested_root) {
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &requested_root,
                "recorded corpus generation root",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus generation root {} is not a real non-symlink directory",
                    requested_root.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
        };
        let root = match root_file.as_ref() {
            Some(file) => canonical_existing_root_path(&parent, &requested_root, file)?,
            None => requested_root,
        };
        let generation = Self {
            root,
            root_file,
            parent,
            parent_file,
        };
        generation.verify()?;
        Ok(generation)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn exists(&self) -> bool {
        self.root_file.is_some()
    }

    /// Recheck the exact directory inode or typed absence captured initially.
    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        verify_directory_handle(
            &self.parent,
            &self.parent_file,
            "recorded corpus generation canonical parent",
        )?;
        verify_optional_root_generation(&self.root, self.root_file.as_ref())
    }

    fn matches_guard(
        &self,
        guard: &RecordedCorpusProducerGuard,
    ) -> Result<bool, SourceUnavailable> {
        self.verify()?;
        guard.verify_parent()?;
        verify_optional_root_generation(&guard.root, guard.root_file.as_ref())?;
        if self.root != guard.root {
            return Ok(false);
        }
        let expected_parent = self
            .parent_file
            .metadata()
            .map_err(SourceUnavailable::new)?;
        let guarded_parent = guard
            ._parent_file
            .metadata()
            .map_err(SourceUnavailable::new)?;
        if !opened_file_identity_matches(&expected_parent, &guarded_parent) {
            return Ok(false);
        }
        match (self.root_file.as_ref(), guard.root_file.as_ref()) {
            (Some(expected), Some(guarded)) => {
                let expected = expected.metadata().map_err(SourceUnavailable::new)?;
                let guarded = guarded.metadata().map_err(SourceUnavailable::new)?;
                Ok(opened_file_identity_matches(&expected, &guarded))
            }
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }

    fn same_generation(&self, other: &Self) -> Result<bool, SourceUnavailable> {
        self.verify()?;
        other.verify()?;
        if self.root != other.root {
            return Ok(false);
        }
        match (self.root_file.as_ref(), other.root_file.as_ref()) {
            (Some(left), Some(right)) => {
                let left = left.metadata().map_err(SourceUnavailable::new)?;
                let right = right.metadata().map_err(SourceUnavailable::new)?;
                Ok(opened_file_identity_matches(&left, &right))
            }
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }

    fn protects_directory(&self, directory: &Path) -> Result<bool, SourceUnavailable> {
        let Some(root_file) = self.root_file.as_ref() else {
            self.verify()?;
            return Ok(false);
        };
        self.verify()?;
        let other = open_directory_nofollow(directory, "recorded corpus pinned comparison root")?;
        let root = root_file.metadata().map_err(SourceUnavailable::new)?;
        let other = other.metadata().map_err(SourceUnavailable::new)?;
        Ok(opened_file_identity_matches(&root, &other))
    }
}

fn collect_bounded_recorded_corpus_root_paths<I, P>(
    roots: I,
    acquisition: &str,
) -> Result<Vec<PathBuf>, SourceUnavailable>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut bounded = Vec::with_capacity(RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES);
    for root in roots {
        if bounded.len() == RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES {
            return Err(SourceUnavailable::new(format!(
                "{acquisition} exceeds the fixed {RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES}-entry logical-root ceiling"
            )));
        }
        bounded.push(root.as_ref().to_path_buf());
    }
    Ok(bounded)
}

/// Canonically sorted, failure-atomic exclusive ownership of a complete root
/// selection set during compare-and-swap publication.
///
/// This is the protocol's only supported multi-producer-guard acquisition. It
/// must itself follow every writer-specific outer session. The standalone stage
/// compiler releases its own guard before this handoff is acquired, avoiding
/// self-contention while every conventional/current/composite candidate plus
/// the private stage generation is revalidated by inode.
#[derive(Debug)]
pub struct RecordedCorpusProducerHandoff {
    guards: Vec<RecordedCorpusProducerGuard>,
    final_guard_index: usize,
    stage_guard_index: usize,
    promoted: bool,
}

impl RecordedCorpusProducerHandoff {
    /// Acquire every expected selection root in canonical lexical order.
    ///
    /// `final_index` and `stage_index` address the caller's unsorted witness
    /// slice. Canonical aliases are deduplicated only after proving that they
    /// name the same captured generation. The designated final and stage must
    /// remain distinct logical roots.
    pub fn try_acquire(
        expected_roots: &[RecordedCorpusRootGeneration],
        final_index: usize,
        stage_index: usize,
    ) -> Result<Self, SourceUnavailable> {
        if expected_roots.len() > RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus handoff exceeds the fixed {RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES}-entry logical-root ceiling"
            )));
        }
        if expected_roots.len() < 2 {
            return Err(SourceUnavailable::new(
                "recorded corpus handoff requires at least a final and stage root generation",
            ));
        }
        let expected_final = expected_roots.get(final_index).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus handoff final index {final_index} is outside {} root generations",
                expected_roots.len()
            ))
        })?;
        let expected_stage = expected_roots.get(stage_index).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus handoff stage index {stage_index} is outside {} root generations",
                expected_roots.len()
            ))
        })?;
        for expected in expected_roots {
            expected.verify()?;
        }
        if expected_final.root == expected_stage.root {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus handoff final and stage resolve to the same canonical root {}",
                expected_final.root.display()
            )));
        }

        fn acquire_expected(
            expected: &RecordedCorpusRootGeneration,
        ) -> Result<RecordedCorpusProducerGuard, SourceUnavailable> {
            let guard = RecordedCorpusProducerGuard::try_acquire(&expected.root)?;
            if !expected.matches_guard(&guard)? {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} changed generation while its producer handoff was acquired",
                    expected.root.display()
                )));
            }
            Ok(guard)
        }

        let mut ordered = expected_roots.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.root.cmp(&right.root));
        let mut unique = Vec::<&RecordedCorpusRootGeneration>::new();
        for expected in ordered {
            if let Some(previous) = unique.last()
                && previous.root == expected.root
            {
                if !previous.same_generation(expected)? {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus handoff root {} changed generation while aliases were canonicalized",
                        expected.root.display()
                    )));
                }
                continue;
            }
            unique.push(expected);
        }

        let mut guards = Vec::with_capacity(unique.len());
        for expected in unique {
            guards.push(acquire_expected(expected)?);
        }
        let locate_guard = |expected: &RecordedCorpusRootGeneration| {
            guards
                .iter()
                .position(|guard| guard.root == expected.root)
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus handoff lost canonical root {} while acquiring its selection set",
                        expected.root.display()
                    ))
                })
        };
        let final_guard_index = locate_guard(expected_final)?;
        let stage_guard_index = locate_guard(expected_stage)?;
        for expected in expected_roots {
            let guard = &guards[locate_guard(expected)?];
            if !expected.matches_guard(guard)? {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} changed generation while its complete producer handoff was acquired",
                    expected.root.display()
                )));
            }
        }
        let handoff = Self {
            guards,
            final_guard_index,
            stage_guard_index,
            promoted: false,
        };
        handoff.verify()?;
        Ok(handoff)
    }

    pub fn final_guard(&self) -> &RecordedCorpusProducerGuard {
        &self.guards[self.final_guard_index]
    }

    pub fn stage_guard(&self) -> &RecordedCorpusProducerGuard {
        &self.guards[self.stage_guard_index]
    }

    pub fn guards(
        &self,
    ) -> impl ExactSizeIterator<Item = &RecordedCorpusProducerGuard> + DoubleEndedIterator {
        self.guards.iter()
    }

    /// Recheck every current logical name against its retained root handle (or
    /// retained typed absence).
    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        for guard in &self.guards {
            guard.verify_parent()?;
            verify_optional_root_generation(&guard.root, guard.root_file.as_ref())?;
        }
        Ok(())
    }

    /// Atomically promote the guarded stage name to the guarded final name.
    /// Existing finals are exchanged; typed-absent finals use no-replace rename.
    /// Afterward the final guard adopts the promoted directory handle, so its
    /// path-keyed exclusive lock and inode ownership remain valid until drop.
    pub fn promote_stage(&mut self) -> Result<(), SourceUnavailable> {
        self.promote_stage_if(|_| Ok(()))
    }

    /// Validate a caller-defined content/selection compare-and-swap predicate
    /// under the complete sorted guard set immediately before the atomic
    /// namespace operation. [`RecordedCorpusRootGeneration`] deliberately
    /// pins directory identity/absence only; a publisher whose logical
    /// generation includes member content must supply that typed predicate
    /// here so an in-place same-directory commit cannot be overwritten.
    pub fn promote_stage_if<F>(&mut self, validate_content: F) -> Result<(), SourceUnavailable>
    where
        F: FnOnce(&Self) -> Result<(), SourceUnavailable>,
    {
        self.promote_stage_with_hook(validate_content)
    }

    fn promote_stage_with_hook<F>(
        &mut self,
        before_final_compare: F,
    ) -> Result<(), SourceUnavailable>
    where
        F: FnOnce(&Self) -> Result<(), SourceUnavailable>,
    {
        if self.promoted {
            return Err(SourceUnavailable::new(
                "recorded corpus stage handoff was already promoted",
            ));
        }
        self.verify()?;
        if self.stage_guard().root_file.is_none() {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus handoff stage {} is absent",
                self.stage_guard().root.display()
            )));
        }
        before_final_compare(self)?;
        // This is the final userspace CAS immediately before the one atomic
        // namespace operation. Cooperative writers cannot cross the held lock;
        // a process ignoring it and racing inside the syscall is outside the
        // documented repository-writer threat boundary.
        self.verify()?;
        let final_existed = self.final_guard().root_file.is_some();
        atomic_promote_recorded_corpus_roots(
            self.final_guard(),
            self.stage_guard(),
            final_existed,
        )?;

        let previous_final = self.guards[self.final_guard_index].root_file.take();
        let promoted = self.guards[self.stage_guard_index]
            .root_file
            .take()
            .ok_or_else(|| {
                SourceUnavailable::new("recorded corpus handoff lost its retained stage handle")
            })?;
        self.guards[self.final_guard_index].root_file = Some(promoted);
        self.guards[self.stage_guard_index].root_file = previous_final;
        self.promoted = true;

        let final_sync = self.guards[self.final_guard_index]
            ._parent_file
            .sync_all()
            .map_err(SourceUnavailable::new);
        let stage_sync = self.guards[self.stage_guard_index]
            ._parent_file
            .sync_all()
            .map_err(SourceUnavailable::new);
        match (final_sync, stage_sync) {
            (Ok(()), Ok(())) => {}
            (Err(error), Ok(())) | (Ok(()), Err(error)) => return Err(error),
            (Err(final_error), Err(stage_error)) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus handoff parent synchronization failed: {final_error}; {stage_error}"
                )));
            }
        }
        self.verify()
    }
}

/// Canonically sorted, non-promoting ownership of one source/destination
/// derivation pair.
///
/// A derivation reads one complete source generation while publishing a
/// different destination generation. Acquiring the two ordinary producer
/// guards independently would make lock order caller-dependent, while exposing
/// [`RecordedCorpusProducerHandoff`] directly would also expose an unrelated
/// root-promotion capability. This narrow facade reuses the handoff's sorted,
/// failure-atomic acquisition and deliberately provides only stable
/// source/destination guard access.
#[derive(Debug)]
pub struct RecordedCorpusDerivationGuards {
    handoff: RecordedCorpusProducerHandoff,
}

impl RecordedCorpusDerivationGuards {
    /// Capture and exclusively acquire a distinct source and destination root.
    ///
    /// Both parents must already exist. Existing roots are pinned by directory
    /// inode and an absent destination is pinned as typed absence until the
    /// destination guard creates it. Canonical aliases, symlink roots, and an
    /// in-place source/destination pair fail before corpus-member mutation.
    pub fn try_acquire(
        source_root: impl AsRef<Path>,
        destination_root: impl AsRef<Path>,
    ) -> Result<Self, SourceUnavailable> {
        let expected = [
            RecordedCorpusRootGeneration::capture(source_root)?,
            RecordedCorpusRootGeneration::capture(destination_root)?,
        ];
        let handoff = RecordedCorpusProducerHandoff::try_acquire(&expected, 0, 1)?;
        handoff.verify()?;
        Ok(Self { handoff })
    }

    /// Exclusive guard for the immutable derivation source.
    pub fn source_guard(&self) -> &RecordedCorpusProducerGuard {
        self.handoff.final_guard()
    }

    /// Exclusive guard for the destination publication transaction.
    pub fn destination_guard(&self) -> &RecordedCorpusProducerGuard {
        self.handoff.stage_guard()
    }

    /// Mutable destination access required to create a previously absent
    /// output root. No mutable source access and no promotion API are exposed.
    pub fn destination_guard_mut(&mut self) -> &mut RecordedCorpusProducerGuard {
        let index = self.handoff.stage_guard_index;
        &mut self.handoff.guards[index]
    }

    /// Recheck both retained root generations and coordination handles.
    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        self.handoff.verify()
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn atomic_promote_recorded_corpus_roots(
    final_guard: &RecordedCorpusProducerGuard,
    stage_guard: &RecordedCorpusProducerGuard,
    final_existed: bool,
) -> Result<(), SourceUnavailable> {
    use std::os::fd::AsRawFd;

    let final_name = owned_leaf_cstring(final_guard.root.file_name().ok_or_else(|| {
        SourceUnavailable::new("recorded corpus final handoff root has no leaf name")
    })?)?;
    let stage_name = owned_leaf_cstring(stage_guard.root.file_name().ok_or_else(|| {
        SourceUnavailable::new("recorded corpus stage handoff root has no leaf name")
    })?)?;
    let flags = if final_existed {
        libc::RENAME_EXCHANGE
    } else {
        libc::RENAME_NOREPLACE
    };
    // SAFETY: both retained descriptors are live directory handles; both C
    // strings are validated single leaves and remain live for the call.
    let result = unsafe {
        libc::renameat2(
            stage_guard._parent_file.as_raw_fd(),
            stage_name.as_ptr(),
            final_guard._parent_file.as_raw_fd(),
            final_name.as_ptr(),
            flags,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(SourceUnavailable::new(format!(
            "recorded corpus atomic stage promotion {} -> {} failed: {}",
            stage_guard.root.display(),
            final_guard.root.display(),
            std::io::Error::last_os_error()
        )))
    }
}

#[cfg(target_vendor = "apple")]
fn atomic_promote_recorded_corpus_roots(
    final_guard: &RecordedCorpusProducerGuard,
    stage_guard: &RecordedCorpusProducerGuard,
    final_existed: bool,
) -> Result<(), SourceUnavailable> {
    use std::os::fd::AsRawFd;

    let final_name = owned_leaf_cstring(final_guard.root.file_name().ok_or_else(|| {
        SourceUnavailable::new("recorded corpus final handoff root has no leaf name")
    })?)?;
    let stage_name = owned_leaf_cstring(stage_guard.root.file_name().ok_or_else(|| {
        SourceUnavailable::new("recorded corpus stage handoff root has no leaf name")
    })?)?;
    let flags = if final_existed {
        libc::RENAME_SWAP
    } else {
        libc::RENAME_EXCL
    };
    // SAFETY: both retained descriptors are live directory handles; both C
    // strings are validated single leaves and remain live for the call.
    let result = unsafe {
        libc::renameatx_np(
            stage_guard._parent_file.as_raw_fd(),
            stage_name.as_ptr(),
            final_guard._parent_file.as_raw_fd(),
            final_name.as_ptr(),
            flags,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(SourceUnavailable::new(format!(
            "recorded corpus atomic stage promotion {} -> {} failed: {}",
            stage_guard.root.display(),
            final_guard.root.display(),
            std::io::Error::last_os_error()
        )))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
fn atomic_promote_recorded_corpus_roots(
    _final_guard: &RecordedCorpusProducerGuard,
    _stage_guard: &RecordedCorpusProducerGuard,
    _final_existed: bool,
) -> Result<(), SourceUnavailable> {
    Err(SourceUnavailable::new(
        "atomic recorded-corpus root promotion is unsupported without a no-replace/exchange *at primitive",
    ))
}

/// Non-mutating shared coordination for a bounded recorded-corpus read.
///
/// When a producer coordination inode already exists, readers take its shared
/// lock and therefore return `BUSY` while an exclusive producer is live. A
/// copied/read-only legacy archive may have no coordination directory or lock;
/// this guard never creates either and instead pins their typed absence plus
/// the parent/root directory handles for a final recheck.
#[derive(Debug)]
pub struct RecordedCorpusReaderGuard {
    root: PathBuf,
    root_file: std::fs::File,
    parent: PathBuf,
    parent_file: std::fs::File,
    coordination_path: PathBuf,
    coordination_file: Option<std::fs::File>,
    lock_path: PathBuf,
    lock_file: Option<std::fs::File>,
}

impl RecordedCorpusReaderGuard {
    pub fn try_acquire(root: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        let requested = root.as_ref();
        let root_name = requested.file_name().ok_or_else(|| {
            SourceUnavailable::new("recorded corpus reader root has no final component")
        })?;
        let requested_parent = requested
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = std::fs::canonicalize(requested_parent).map_err(SourceUnavailable::new)?;
        let parent_file =
            open_directory_nofollow(&parent, "recorded corpus reader canonical parent")?;
        let requested_root = parent.join(root_name);
        let root_file = open_directory_nofollow(&requested_root, "recorded corpus reader root")?;
        let root = canonical_existing_root_path(&parent, &requested_root, &root_file)?;
        verify_directory_handle(&root, &root_file, "recorded corpus reader root")?;

        let coordination_path = parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        let coordination_file = match std::fs::symlink_metadata(&coordination_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &coordination_path,
                "recorded corpus reader coordination directory",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus reader coordination {} is not a real non-symlink directory",
                    coordination_path.display()
                )));
            }
        };
        let lock_path = coordination_path.join(root.file_name().ok_or_else(|| {
            SourceUnavailable::new("canonical recorded corpus reader root has no name")
        })?);
        let lock_file = if coordination_file.is_some() {
            match std::fs::symlink_metadata(&lock_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(metadata) if metadata.file_type().is_file() => {
                    let mut options = OpenOptions::new();
                    options.read(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
                    }
                    let file = options.open(&lock_path).map_err(SourceUnavailable::new)?;
                    let opened = file.metadata().map_err(SourceUnavailable::new)?;
                    let path =
                        std::fs::symlink_metadata(&lock_path).map_err(SourceUnavailable::new)?;
                    if !opened.file_type().is_file()
                        || !path.file_type().is_file()
                        || !opened_file_identity_matches(&path, &opened)
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus reader coordination {} changed identity or type",
                            lock_path.display()
                        )));
                    }
                    match file.try_lock_shared() {
                        Ok(()) => Some(file),
                        Err(std::fs::TryLockError::WouldBlock) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus root {} is BUSY under another active producer session",
                                root.display()
                            )));
                        }
                        Err(std::fs::TryLockError::Error(error)) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus reader coordination {} cannot be locked: {error}",
                                lock_path.display()
                            )));
                        }
                    }
                }
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} is not a regular non-symlink file",
                        lock_path.display()
                    )));
                }
            }
        } else {
            None
        };

        let guard = Self {
            root,
            root_file,
            parent,
            parent_file,
            coordination_path,
            coordination_file,
            lock_path,
            lock_file,
        };
        guard.verify()?;
        Ok(guard)
    }

    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        verify_directory_handle(
            &self.parent,
            &self.parent_file,
            "recorded corpus reader canonical parent",
        )?;
        verify_directory_handle(&self.root, &self.root_file, "recorded corpus reader root")?;
        match self.coordination_file.as_ref() {
            Some(file) => verify_directory_handle(
                &self.coordination_path,
                file,
                "recorded corpus reader coordination directory",
            )?,
            None => match std::fs::symlink_metadata(&self.coordination_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} appeared during an uncoordinated legacy read",
                        self.coordination_path.display()
                    )));
                }
            },
        }
        match self.lock_file.as_ref() {
            Some(file) => {
                let path =
                    std::fs::symlink_metadata(&self.lock_path).map_err(SourceUnavailable::new)?;
                let opened = file.metadata().map_err(SourceUnavailable::new)?;
                if !path.file_type().is_file()
                    || !opened.file_type().is_file()
                    || !opened_file_identity_matches(&path, &opened)
                {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader coordination {} changed identity or type",
                        self.lock_path.display()
                    )));
                }
            }
            None if self.coordination_file.is_some() => {
                match std::fs::symlink_metadata(&self.lock_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(SourceUnavailable::new(error)),
                    Ok(_) => {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus reader coordination {} appeared during an uncoordinated legacy read",
                            self.lock_path.display()
                        )));
                    }
                }
            }
            None => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RecordedCorpusReaderPin {
    generation: RecordedCorpusRootGeneration,
    coordination_path: PathBuf,
    coordination_file: Option<std::fs::File>,
    lock_path: PathBuf,
    lock_file: Option<std::fs::File>,
}

impl RecordedCorpusReaderPin {
    fn try_acquire(generation: RecordedCorpusRootGeneration) -> Result<Self, SourceUnavailable> {
        generation.verify()?;
        let coordination_path = generation
            .parent
            .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        let coordination_file = match std::fs::symlink_metadata(&coordination_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(SourceUnavailable::new(error)),
            Ok(metadata) if metadata.file_type().is_dir() => Some(open_directory_nofollow(
                &coordination_path,
                "recorded corpus pinned-reader coordination directory",
            )?),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus pinned-reader coordination {} is not a real non-symlink directory",
                    coordination_path.display()
                )));
            }
        };
        let lock_path = coordination_path.join(generation.root.file_name().ok_or_else(|| {
            SourceUnavailable::new("canonical recorded corpus pinned-reader root has no name")
        })?);
        let lock_file = if coordination_file.is_some() {
            match std::fs::symlink_metadata(&lock_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(metadata) if metadata.file_type().is_file() => {
                    let mut options = OpenOptions::new();
                    options.read(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
                    }
                    let file = options.open(&lock_path).map_err(SourceUnavailable::new)?;
                    let opened = file.metadata().map_err(SourceUnavailable::new)?;
                    let path =
                        std::fs::symlink_metadata(&lock_path).map_err(SourceUnavailable::new)?;
                    if !opened.file_type().is_file()
                        || !path.file_type().is_file()
                        || !opened_file_identity_matches(&path, &opened)
                    {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus pinned-reader coordination {} changed identity or type",
                            lock_path.display()
                        )));
                    }
                    match file.try_lock_shared() {
                        Ok(()) => Some(file),
                        Err(std::fs::TryLockError::WouldBlock) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus root {} is BUSY under another active producer session",
                                generation.root.display()
                            )));
                        }
                        Err(std::fs::TryLockError::Error(error)) => {
                            return Err(SourceUnavailable::new(format!(
                                "recorded corpus pinned-reader coordination {} cannot be locked: {error}",
                                lock_path.display()
                            )));
                        }
                    }
                }
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus pinned-reader coordination {} is not a regular non-symlink file",
                        lock_path.display()
                    )));
                }
            }
        } else {
            None
        };
        let pin = Self {
            generation,
            coordination_path,
            coordination_file,
            lock_path,
            lock_file,
        };
        pin.verify()?;
        Ok(pin)
    }

    fn verify(&self) -> Result<(), SourceUnavailable> {
        self.generation.verify()?;
        match self.coordination_file.as_ref() {
            Some(file) => verify_directory_handle(
                &self.coordination_path,
                file,
                "recorded corpus pinned-reader coordination directory",
            )?,
            None => match std::fs::symlink_metadata(&self.coordination_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(SourceUnavailable::new(error)),
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus pinned-reader coordination {} appeared during an uncoordinated legacy read",
                        self.coordination_path.display()
                    )));
                }
            },
        }
        match self.lock_file.as_ref() {
            Some(file) => {
                let path =
                    std::fs::symlink_metadata(&self.lock_path).map_err(SourceUnavailable::new)?;
                let opened = file.metadata().map_err(SourceUnavailable::new)?;
                if !path.file_type().is_file()
                    || !opened.file_type().is_file()
                    || !opened_file_identity_matches(&path, &opened)
                {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus pinned-reader coordination {} changed identity or type",
                        self.lock_path.display()
                    )));
                }
            }
            None if self.coordination_file.is_some() => {
                match std::fs::symlink_metadata(&self.lock_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(SourceUnavailable::new(error)),
                    Ok(_) => {
                        return Err(SourceUnavailable::new(format!(
                            "recorded corpus pinned-reader coordination {} appeared during an uncoordinated legacy read",
                            self.lock_path.display()
                        )));
                    }
                }
            }
            None => {}
        }
        self.generation.verify()
    }
}

/// Canonically sorted, failure-atomic shared pins for one or more logical
/// recorded-corpus roots.
///
/// Existing roots and typed absence are both retained by inode-bearing parent
/// handles. Acquisition is read-only: copied archives and read-only parents
/// without producer coordination remain usable. When coordination exists,
/// each permanent lock inode is held shared and therefore contends with every
/// cooperating producer until this value is dropped.
#[derive(Debug)]
pub struct RecordedCorpusReaderPins {
    pins: Vec<RecordedCorpusReaderPin>,
}

impl RecordedCorpusReaderPins {
    pub fn try_acquire<I, P>(roots: I) -> Result<Self, SourceUnavailable>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let roots = collect_bounded_recorded_corpus_root_paths(
            roots,
            "recorded corpus reader-pin acquisition",
        )?;
        let mut generations = Vec::with_capacity(roots.len());
        for root in roots {
            generations.push(RecordedCorpusRootGeneration::capture(root)?);
        }
        if generations.is_empty() {
            return Err(SourceUnavailable::new(
                "recorded corpus reader pins require at least one logical root",
            ));
        }
        generations.sort_by(|left, right| left.root.cmp(&right.root));

        let mut unique = Vec::<RecordedCorpusRootGeneration>::new();
        for generation in generations {
            if let Some(previous) = unique.last()
                && previous.root == generation.root
            {
                if !previous.same_generation(&generation)? {
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus reader pin root {} changed generation while aliases were canonicalized",
                        generation.root.display()
                    )));
                }
                continue;
            }
            unique.push(generation);
        }

        let mut pins = Vec::with_capacity(unique.len());
        for generation in unique {
            pins.push(RecordedCorpusReaderPin::try_acquire(generation)?);
        }
        let result = Self { pins };
        result.verify()?;
        Ok(result)
    }

    pub fn generations(
        &self,
    ) -> impl ExactSizeIterator<Item = &RecordedCorpusRootGeneration> + DoubleEndedIterator {
        self.pins.iter().map(|pin| &pin.generation)
    }

    pub fn verify(&self) -> Result<(), SourceUnavailable> {
        for pin in &self.pins {
            pin.verify()?;
        }
        Ok(())
    }

    fn protects_directory(&self, directory: &Path) -> Result<bool, SourceUnavailable> {
        for pin in &self.pins {
            if pin.generation.protects_directory(directory)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Debug)]
struct PlannedOutputResidue {
    member: PlannedOutputMember,
    path: PathBuf,
    metadata: Metadata,
}

fn validate_owned_leaf(name: &OsStr) -> Result<(), SourceUnavailable> {
    let path = Path::new(name);
    let mut components = path.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
        || name == OsStr::new(".")
        || name == OsStr::new("..")
    {
        return Err(SourceUnavailable::new(format!(
            "owned recorded-corpus entry {name:?} must be one exact basename"
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        if name.as_bytes().contains(&0) || name.as_bytes().contains(&b'/') {
            return Err(SourceUnavailable::new(format!(
                "owned recorded-corpus entry {name:?} contains a forbidden slash or NUL"
            )));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn owned_leaf_cstring(name: &OsStr) -> Result<std::ffi::CString, SourceUnavailable> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(name.as_bytes()).map_err(SourceUnavailable::new)
}

#[cfg(not(unix))]
fn owned_root_relative_unsupported() -> SourceUnavailable {
    SourceUnavailable::new(
        "retained-root relative mutation is unsupported without platform file-ID and *at equivalents",
    )
}

fn verify_file_bytes_without_path(
    file: &mut std::fs::File,
    expected: &[u8],
    initial: &Metadata,
    context: &str,
) -> Result<(), SourceUnavailable> {
    let mut offset = 0usize;
    let mut buffer = [0u8; 8192];
    while offset != expected.len() {
        let limit = (expected.len() - offset).min(buffer.len());
        let read = file
            .read(&mut buffer[..limit])
            .map_err(SourceUnavailable::new)?;
        if read == 0 || expected.get(offset..offset + read) != Some(&buffer[..read]) {
            return Err(SourceUnavailable::new(format!(
                "{context} changed content during retained-root verification"
            )));
        }
        offset += read;
    }
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} grew during retained-root verification"
        )));
    }
    let final_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !opened_file_generation_matches(initial, &final_metadata) {
        return Err(SourceUnavailable::new(format!(
            "{context} changed generation during retained-root verification"
        )));
    }
    Ok(())
}

fn planned_output_staging_name(member: PlannedOutputMember, name: &str) -> bool {
    let prefix = format!("{PLANNED_OUTPUT_RESERVED_PREFIX}{}--", member.stable_name());
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_planned_output_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX.as_bytes()) {
            return Ok(None);
        }
        std::str::from_utf8(bytes)
            .map(|name| Some(name.to_owned()))
            .map_err(|_| {
                SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-UTF-8 entry in the reserved planned-output staging namespace",
                    root.display()
                ))
            })
    }

    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX) {
            return Ok(None);
        }
        name.to_str().map(|name| Some(name.to_owned())).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved planned-output staging namespace",
                root.display()
            ))
        })
    }
}

fn open_directory_nofollow(path: &Path, context: &str) -> Result<std::fs::File, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.custom_flags(
            libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        );
    }
    let file = options.open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be opened without following links: {error}",
            path.display()
        ))
    })?;
    if !file
        .metadata()
        .map_err(SourceUnavailable::new)?
        .file_type()
        .is_dir()
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not a directory",
            path.display()
        )));
    }
    Ok(file)
}

fn verify_directory_handle(
    path: &Path,
    file: &std::fs::File,
    context: &str,
) -> Result<(), SourceUnavailable> {
    let path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !path_metadata.file_type().is_dir()
        || !file_metadata.file_type().is_dir()
        || !opened_file_identity_matches(&path_metadata, &file_metadata)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity or is not a real non-symlink directory",
            path.display()
        )));
    }
    Ok(())
}

fn open_or_create_coordination_directory(
    canonical_parent: &Path,
    parent_file: &std::fs::File,
) -> Result<std::fs::File, SourceUnavailable> {
    verify_directory_handle(
        canonical_parent,
        parent_file,
        "recorded corpus producer canonical parent",
    )?;
    let path = canonical_parent.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
    match std::fs::create_dir(&path) {
        Ok(()) => parent_file.sync_all().map_err(SourceUnavailable::new)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer coordination directory {} cannot be created: {error}",
                path.display()
            )));
        }
    }
    let file = open_directory_nofollow(&path, "recorded corpus producer coordination directory")?;
    verify_directory_handle(
        &path,
        &file,
        "recorded corpus producer coordination directory",
    )?;
    verify_directory_handle(
        canonical_parent,
        parent_file,
        "recorded corpus producer canonical parent",
    )?;
    Ok(file)
}

fn verify_optional_root_generation(
    root: &Path,
    file: Option<&std::fs::File>,
) -> Result<(), SourceUnavailable> {
    match file {
        Some(file) => verify_directory_handle(root, file, "recorded corpus producer root"),
        None => match std::fs::symlink_metadata(root) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SourceUnavailable::new(error)),
            Ok(_) => Err(SourceUnavailable::new(format!(
                "recorded corpus producer root {} appeared while its absent logical path was being locked",
                root.display()
            ))),
        },
    }
}

fn canonical_existing_root_path(
    canonical_parent: &Path,
    requested_root: &Path,
    file: &std::fs::File,
) -> Result<PathBuf, SourceUnavailable> {
    #[cfg(unix)]
    {
        let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
        let mut matches = Vec::new();
        for entry in std::fs::read_dir(canonical_parent).map_err(SourceUnavailable::new)? {
            let entry = entry.map_err(SourceUnavailable::new)?;
            let metadata = match std::fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata,
                // Unrelated siblings may be removed concurrently. The opened
                // target handle and its exact requested path are rechecked
                // below, so a vanished nonmatching sibling is irrelevant.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(SourceUnavailable::new(error)),
            };
            if metadata.file_type().is_dir()
                && opened_file_identity_matches(&metadata, &file_metadata)
            {
                matches.push(entry.path());
            }
        }
        matches.sort();
        if matches.len() != 1 {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus producer root {} resolves to {} real directory entries in {}; refusing an ambiguous OS-equivalence class",
                requested_root.display(),
                matches.len(),
                canonical_parent.display()
            )));
        }
        Ok(matches.remove(0))
    }

    #[cfg(not(unix))]
    std::fs::canonicalize(requested_root).map_err(SourceUnavailable::new)
}

#[cfg(test)]
fn producer_lock_path(canonical_parent: &Path, root_name: &OsStr) -> PathBuf {
    canonical_parent
        .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR)
        .join(root_name)
}

#[cfg(unix)]
fn opened_file_identity_matches(path: &Metadata, file: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    path.dev() == file.dev() && path.ino() == file.ino()
}

#[cfg(not(unix))]
fn opened_file_identity_matches(_path: &Metadata, _file: &Metadata) -> bool {
    // The portable Metadata API exposes no stable file ID. Treating two
    // snapshots as identical from length/timestamps alone permits ABA. A
    // platform implementation must supply real volume/file IDs before this
    // transaction protocol is enabled there.
    false
}

#[cfg(unix)]
fn opened_file_generation_matches(initial: &Metadata, final_metadata: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.mtime() == final_metadata.mtime()
        && initial.mtime_nsec() == final_metadata.mtime_nsec()
        && initial.ctime() == final_metadata.ctime()
        && initial.ctime_nsec() == final_metadata.ctime_nsec()
}

#[cfg(not(unix))]
fn opened_file_generation_matches(initial: &Metadata, final_metadata: &Metadata) -> bool {
    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.modified().ok() == final_metadata.modified().ok()
}

fn capture_optional_regular_file(
    path: &Path,
    context: &str,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} is not a regular file or cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} opened handle cannot be inspected: {error}",
            path.display()
        ))
    })?;
    let initial_path = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be inspected after open: {error}",
            path.display()
        ))
    })?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not the opened regular non-symlink file",
            path.display()
        )));
    }

    let length = usize::try_from(initial_file.len()).map_err(|_| {
        SourceUnavailable::new(format!(
            "{context} {} length cannot be represented on this host",
            path.display()
        ))
    })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot reserve its declared {length} bytes: {error}",
            path.display()
        ))
    })?;
    bytes.resize(length, 0);
    file.read_exact(&mut bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} changed or ended before its declared length: {error}",
            path.display()
        ))
    })?;
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its captured length",
            path.display()
        )));
    }
    file.seek(SeekFrom::Start(0)).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be rewound for verification: {error}",
            path.display()
        ))
    })?;
    verify_opened_bytes(&mut file, path, context, &bytes, &initial_file)?;
    Ok(Some(bytes))
}

fn capture_optional_control_file(
    path: &Path,
    context: &str,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} is not a regular file or cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not the opened regular non-symlink file",
            path.display()
        )));
    }
    if initial_file.len() > max_bytes {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is {} bytes, exceeding the registered {max_bytes}-byte schema cap",
            path.display(),
            initial_file.len()
        )));
    }
    let length = usize::try_from(initial_file.len()).map_err(|_| {
        SourceUnavailable::new(format!(
            "{context} {} length cannot be represented on this host",
            path.display()
        ))
    })?;
    let mut bytes = vec![0u8; length];
    file.read_exact(&mut bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} changed or ended before its declared length: {error}",
            path.display()
        ))
    })?;
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its captured length",
            path.display()
        )));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    verify_opened_bytes(&mut file, path, context, &bytes, &initial_file)?;
    Ok(Some(bytes))
}

/// Capture only the typed presence/length/content address of a regular file.
/// The payload is streamed through one no-follow handle and never retained;
/// initial/final path and handle generation checks bind the digest to that
/// exact inode generation.
struct StreamedGeneration {
    path: PathBuf,
    opened: Option<(std::fs::File, Metadata)>,
}

impl StreamedGeneration {
    fn verify(&self, context: &str) -> Result<(), SourceUnavailable> {
        let Some((file, initial)) = self.opened.as_ref() else {
            return match std::fs::symlink_metadata(&self.path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(SourceUnavailable::new(error)),
                Ok(_) => Err(SourceUnavailable::new(format!(
                    "{context} {} appeared after typed absence was captured",
                    self.path.display()
                ))),
            };
        };
        let current_file = file.metadata().map_err(SourceUnavailable::new)?;
        let current_path = std::fs::symlink_metadata(&self.path).map_err(SourceUnavailable::new)?;
        if !current_file.file_type().is_file()
            || !current_path.file_type().is_file()
            || !opened_file_generation_matches(initial, &current_file)
            || !opened_file_identity_matches(&current_path, &current_file)
        {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed generation before binding commit",
                self.path.display()
            )));
        }
        Ok(())
    }
}

fn stream_regular_file_binding_retained(
    path: &Path,
    expected_name: &str,
    context: &str,
) -> Result<(RecordedCorpusFileBinding, StreamedGeneration), SourceUnavailable> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((
                RecordedCorpusFileBinding::from_bytes(expected_name, None)?,
                StreamedGeneration {
                    path: path.to_path_buf(),
                    opened: None,
                },
            ));
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not one regular non-symlink inode",
            path.display()
        )));
    }
    let mut hasher = blake3::Hasher::new();
    let mut length = 0u64;
    let mut remaining = initial_file.len();
    let mut buffer = [0u8; STREAM_BINDING_BUFFER_BYTES];
    while remaining != 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| SourceUnavailable::new(format!("{context} length overflows usize")))?;
        let read = file.read(&mut buffer[..limit]).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be streamed: {error}",
                path.display()
            ))
        })?;
        if read == 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} {} ended before its initial declared length",
                path.display()
            )));
        }
        length = length
            .checked_add(read as u64)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} length overflows u64")))?;
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its initial declared length",
            path.display()
        )));
    }
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} changed or disappeared while hashed: {error}",
            path.display()
        ))
    })?;
    if length != initial_file.len()
        || !final_path.file_type().is_file()
        || !opened_file_generation_matches(&initial_file, &final_file)
        || !opened_file_identity_matches(&final_path, &final_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed generation while it was streamed",
            path.display()
        )));
    }
    Ok((
        RecordedCorpusFileBinding {
            name: expected_name.to_owned(),
            present: true,
            length: Some(length),
            blake3: Some(format!("blake3:{}", hasher.finalize().to_hex())),
        },
        StreamedGeneration {
            path: path.to_path_buf(),
            opened: Some((file, initial_file)),
        },
    ))
}

fn open_verified_corpus_member(
    path: &Path,
    expected_name: &str,
    context: &'static str,
    expected: Option<&RecordedCorpusFileBinding>,
) -> Result<Option<VerifiedCorpusMember>, SourceUnavailable> {
    open_verified_corpus_member_with_hook(path, expected_name, context, expected, || {})
}

fn open_verified_corpus_member_with_hook<F>(
    path: &Path,
    expected_name: &str,
    context: &'static str,
    expected: Option<&RecordedCorpusFileBinding>,
    after_metadata: F,
) -> Result<Option<VerifiedCorpusMember>, SourceUnavailable>
where
    F: FnOnce(),
{
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let actual = RecordedCorpusFileBinding::from_bytes(expected_name, None)?;
            if let Some(expected) = expected {
                expected.validate_record(expected_name, &actual, context)?;
            }
            return Ok(None);
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let initial = file.metadata().map_err(SourceUnavailable::new)?;
    let path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial.file_type().is_file()
        || !path_metadata.file_type().is_file()
        || !opened_file_identity_matches(&path_metadata, &initial)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not one regular non-symlink inode",
            path.display()
        )));
    }
    if let Some(expected) = expected {
        if !expected.present {
            return Err(SourceUnavailable::new(format!(
                "{context} {} is present, but the canonical generation binding declares it absent",
                path.display()
            )));
        }
        let expected_length = expected.length.ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{context} canonical generation binding omits the required present length"
            ))
        })?;
        if initial.len() != expected_length {
            return Err(SourceUnavailable::new(format!(
                "{context} {} has length {}, but the canonical generation binding declares {expected_length}; refusing before body hashing",
                path.display(),
                initial.len()
            )));
        }
    }
    after_metadata();
    let mut hasher = blake3::Hasher::new();
    let length = initial.len();
    let mut remaining = length;
    let mut buffer = [0u8; STREAM_BINDING_BUFFER_BYTES];
    while remaining != 0 {
        let limit =
            usize::try_from(remaining.min(buffer.len() as u64)).map_err(SourceUnavailable::new)?;
        let read = file
            .read(&mut buffer[..limit])
            .map_err(SourceUnavailable::new)?;
        if read == 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} {} ended before its initial {length}-byte generation",
                path.display()
            )));
        }
        remaining -= read as u64;
        hasher.update(&buffer[..read]);
    }
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its initial {length}-byte generation",
            path.display()
        )));
    }
    let binding = RecordedCorpusFileBinding {
        name: expected_name.to_owned(),
        present: true,
        length: Some(length),
        blake3: Some(format!("blake3:{}", hasher.finalize().to_hex())),
    };
    if let Some(expected) = expected {
        expected.validate_record(expected_name, &binding, context)?;
    }
    file.seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    let member = VerifiedCorpusMember {
        path: path.to_path_buf(),
        file,
        initial,
        binding,
        position: 0,
        context,
    };
    member.verify_generation()?;
    Ok(Some(member))
}

fn verify_opened_bytes(
    file: &mut std::fs::File,
    path: &Path,
    context: &str,
    expected: &[u8],
    initial_file: &Metadata,
) -> Result<(), SourceUnavailable> {
    let mut offset = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be verified: {error}",
                path.display()
            ))
        })?;
        if count == 0 {
            break;
        }
        let end = offset
            .checked_add(count)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} byte count overflow")))?;
        if expected.get(offset..end) != Some(&buffer[..count]) {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed content during capture",
                path.display()
            )));
        }
        offset = end;
    }
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if offset != expected.len()
        || !final_file.file_type().is_file()
        || !final_path.file_type().is_file()
        || !opened_file_generation_matches(initial_file, &final_file)
        || !opened_file_identity_matches(&final_path, &final_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity, type, length, content, or generation during capture",
            path.display()
        )));
    }
    Ok(())
}

fn verify_captured_optional_regular_file(
    path: &Path,
    context: &str,
    expected: Option<&[u8]>,
) -> Result<(), SourceUnavailable> {
    let Some(expected) = expected else {
        return match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SourceUnavailable::new(format!(
                "{context} {} cannot be reinspected: {error}",
                path.display()
            ))),
            Ok(_) => Err(SourceUnavailable::new(format!(
                "{context} {} appeared after typed absence was captured",
                path.display()
            ))),
        };
    };
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} disappeared or cannot be reopened without following links: {error}",
            path.display()
        ))
    })?;
    let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
        || initial_file.len() != expected.len() as u64
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity, type, or length after capture",
            path.display()
        )));
    }
    verify_opened_bytes(&mut file, path, context, expected, &initial_file)
}

fn verify_captured_optional_control_file(
    path: &Path,
    context: &str,
    expected: Option<&[u8]>,
    max_bytes: u64,
) -> Result<(), SourceUnavailable> {
    if expected.is_some_and(|bytes| bytes.len() as u64 > max_bytes) {
        return Err(SourceUnavailable::new(format!(
            "{context} {} exceeds the registered {max_bytes}-byte schema cap",
            path.display()
        )));
    }
    verify_captured_optional_regular_file(path, context, expected)
}

fn canonical_parent(path: &Path, label: &str) -> Result<PathBuf, SourceUnavailable> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be inspected: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{label} {} is not a regular non-symlink file",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical = std::fs::canonicalize(parent).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} parent {} cannot be resolved: {error}",
            parent.display()
        ))
    })?;
    if !std::fs::metadata(&canonical)
        .map_err(SourceUnavailable::new)?
        .is_dir()
    {
        return Err(SourceUnavailable::new(format!(
            "{label} parent {} is not a directory",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn resolved_corpus_paths(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), SourceUnavailable> {
    let meta_root = canonical_parent(corpus_meta, "corpus metadata")?;
    let records_root = canonical_parent(corpus_records, "corpus records")?;
    if meta_root != records_root {
        return Err(SourceUnavailable::new(format!(
            "corpus metadata and records have different canonical parent roots ({} versus {}); no cryptographic cross-directory pairing exists",
            meta_root.display(),
            records_root.display()
        )));
    }
    let meta_name = corpus_meta
        .file_name()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path has no filename"))?;
    let records_name = corpus_records
        .file_name()
        .ok_or_else(|| SourceUnavailable::new("corpus records path has no filename"))?;
    Ok((
        meta_root.clone(),
        meta_root.join(meta_name),
        meta_root.join(records_name),
    ))
}

fn is_canonical_pair(corpus_meta: &Path, corpus_records: &Path) -> bool {
    matches!(
        (corpus_meta.file_name(), corpus_records.file_name()),
        (Some(meta), Some(records))
            if (meta == OsStr::new("corpus.meta") && records == OsStr::new("corpus.records"))
                || (meta == OsStr::new(observation::STATE_FILE)
                    && records == OsStr::new("merged.bin"))
    )
}

fn hidden_path(corpus_records: &Path) -> PathBuf {
    uor_r4_core::transformerless::compiler::corpus_hidden_path(corpus_records)
}

type ProvenanceBytes = [Option<Vec<u8>>; 3];

fn provenance_paths(root: &Path) -> [PathBuf; 3] {
    [
        root.join(observation::MANIFEST_FILE),
        root.join(ATTENTION_OPERATOR_BINDING_FILE),
        root.join(DENSE_OPERATOR_BINDING_FILE),
    ]
}

fn capture_provenance(root: &Path) -> Result<ProvenanceBytes, SourceUnavailable> {
    let paths = provenance_paths(root);
    let mut bytes: ProvenanceBytes = [None, None, None];
    for (slot, path) in paths.iter().enumerate() {
        bytes[slot] = capture_optional_control_file(
            path,
            "recorded corpus provenance",
            PROVENANCE_CONTROL_MAX_BYTES,
        )?;
    }
    Ok(bytes)
}

fn verify_provenance(root: &Path, bytes: &ProvenanceBytes) -> Result<(), SourceUnavailable> {
    for (path, expected) in provenance_paths(root).iter().zip(bytes.iter()) {
        verify_captured_optional_control_file(
            path,
            "recorded corpus provenance",
            expected.as_deref(),
            PROVENANCE_CONTROL_MAX_BYTES,
        )?;
    }
    Ok(())
}

fn utf8_file_name(path: &Path, label: &str) -> Result<String, SourceUnavailable> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{label} {} has no UTF-8 filename for canonical binding",
                path.display()
            ))
        })
}

fn binding_path(root: &Path) -> PathBuf {
    root.join(RECORDED_CORPUS_BINDING_FILE)
}

fn parse_binding_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<RecordedCorpusBinding, SourceUnavailable> {
    reject_duplicate_json(
        bytes,
        &format!("recorded corpus binding {}", path.display()),
    )?;
    let binding: RecordedCorpusBinding = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "invalid recorded corpus binding {}: {error}",
            path.display()
        ))
    })?;
    if binding.schema != RECORDED_CORPUS_BINDING_SCHEMA {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} has unsupported schema {:?}",
            path.display(),
            binding.schema
        )));
    }
    let _ = binding.validate_shape(&format!("recorded corpus binding {}", path.display()))?;
    if binding.canonical_bytes()? != bytes {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding {} is not canonical pretty JSON with one trailing newline",
            path.display()
        )));
    }
    Ok(binding)
}

fn compile_attempt_path(root: &Path) -> PathBuf {
    root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
}

fn parse_compile_attempt_bytes(path: &Path, bytes: &[u8]) -> Result<(), SourceUnavailable> {
    reject_duplicate_json(
        bytes,
        &format!("recorded corpus compile-attempt marker {}", path.display()),
    )?;
    let marker: RecordedCorpusCompileAttempt = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "invalid recorded corpus compile-attempt marker {}: {error}",
            path.display()
        ))
    })?;
    let expected = RecordedCorpusCompileAttempt::compile();
    if marker != expected || marker.canonical_bytes()? != bytes {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus compile-attempt marker {} is not the exact canonical compile/1 record",
            path.display()
        )));
    }
    Ok(())
}

fn compile_attempt_writing_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_compile_attempt_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    let prefix = format!(".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bytes = name.as_bytes();
        if !bytes.starts_with(prefix.as_bytes()) {
            return Ok(None);
        }
        std::str::from_utf8(bytes)
            .map(|name| Some(name.to_owned()))
            .map_err(|_| {
                SourceUnavailable::new(format!(
                    "recorded corpus root {} contains a non-UTF-8 entry in the reserved compile-attempt namespace",
                    root.display()
                ))
            })
    }
    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(&prefix) {
            return Ok(None);
        }
        name.to_str().map(|name| Some(name.to_owned())).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved compile-attempt namespace",
                root.display()
            ))
        })
    }
}

fn compile_attempt_residues(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, SourceUnavailable> {
    let mut residues = Vec::new();
    let mut reserved_count = 0usize;
    for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = reserved_compile_attempt_entry_name(root, &entry.file_name())? else {
            continue;
        };
        if !compile_attempt_writing_name(&name) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unrecognized reserved compile-attempt entry {name:?}",
                root.display()
            )));
        }
        reserved_count = reserved_count
            .checked_add(1)
            .ok_or_else(|| SourceUnavailable::new("compile-attempt residue count overflow"))?;
        if reserved_count > RESERVED_RESIDUE_MAX_ENTRIES {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} exceeds the fixed {RESERVED_RESIDUE_MAX_ENTRIES}-entry compile-attempt residue ceiling",
                root.display()
            )));
        }
        let path = entry.path();
        let bytes = capture_optional_control_file(
            &path,
            "recorded corpus compile-attempt staging",
            COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        )?
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus compile-attempt staging {} disappeared",
                path.display()
            ))
        })?;
        residues.push((path, bytes));
    }
    residues.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(residues)
}

fn validate_compile_attempt_namespace(
    root: &Path,
    role: RecordedCorpusRole,
) -> Result<(), SourceUnavailable> {
    let stable = compile_attempt_path(root);
    let stable_present = capture_optional_control_file(
        &stable,
        "recorded corpus compile-attempt marker",
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
    )?;
    if let Some(bytes) = stable_present.as_deref() {
        parse_compile_attempt_bytes(&stable, bytes)?;
    }
    let residues = compile_attempt_residues(root)?;
    if role == RecordedCorpusRole::Observation && (stable_present.is_some() || !residues.is_empty())
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus root {} contains compile-attempt evidence; refusing observation mutation",
            root.display()
        )));
    }
    Ok(())
}

fn publish_compile_attempt_marker(
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    publish_compile_attempt_marker_with_hook(guard, || {})
}

fn publish_compile_attempt_marker_with_hook<F>(
    guard: &RecordedCorpusProducerGuard,
    before_link: F,
) -> Result<(), SourceUnavailable>
where
    F: FnOnce(),
{
    guard.verify_root()?;
    validate_compile_attempt_namespace(&guard.root, RecordedCorpusRole::Compile)?;
    let expected = RecordedCorpusCompileAttempt::compile().canonical_bytes()?;
    let stable = compile_attempt_path(&guard.root);
    let stable_name = OsStr::new(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE);
    let stable_present = guard.verify_optional_owned_entry_bytes(
        stable_name,
        &expected,
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        "recorded corpus compile-attempt marker",
    )?;
    let residues = compile_attempt_residues(&guard.root)?;
    guard.verify_root()?;
    if stable_present {
        for (path, bytes) in residues {
            remove_exact_owned_binding_residue(guard, &path, &bytes, "compile-attempt staging")?;
        }
        guard.sync_owned_root()?;
        return guard.verify_root();
    }
    for (path, bytes) in residues {
        remove_exact_owned_binding_residue(guard, &path, &bytes, "compile-attempt staging")?;
    }
    let (staging, mut file) = loop {
        let sequence = RECORDED_CORPUS_COMPILE_ATTEMPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = std::ffi::OsString::from(format!(
            ".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.{}.{}.writing",
            std::process::id(),
            sequence
        ));
        match guard.create_new_owned_entry(&candidate) {
            Ok(file) => break (candidate, file),
            Err(error) => {
                if guard
                    .open_optional_owned_entry_for_read(&candidate)?
                    .is_some()
                {
                    continue;
                }
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus compile-attempt staging {candidate:?} cannot be created: {error}"
                )));
            }
        }
    };
    file.write_all(&expected)
        .and_then(|()| file.sync_all())
        .map_err(SourceUnavailable::new)?;
    drop(file);
    guard.verify_root()?;
    before_link();
    if let Err(error) = guard.link_owned_entry(&staging, stable_name) {
        match guard.verify_optional_owned_entry_bytes(
            stable_name,
            &expected,
            COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
            "recorded corpus compile-attempt marker",
        ) {
            Ok(true) => {}
            Ok(false) => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus compile-attempt publication failed: {error}"
                )));
            }
            Err(conflict) => return Err(conflict),
        }
    }
    guard.unlink_owned_entry(&staging)?;
    guard.sync_owned_root()?;
    guard.verify_owned_entry_bytes(
        stable_name,
        &expected,
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        "recorded corpus compile-attempt marker",
    )?;
    parse_compile_attempt_bytes(&stable, &expected)?;
    guard.verify_root()
}

fn finish_compile_attempt_marker(
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    finish_compile_attempt_marker_with_hook(guard, || {})
}

fn finish_compile_attempt_marker_with_hook<F>(
    guard: &RecordedCorpusProducerGuard,
    before_unlink: F,
) -> Result<(), SourceUnavailable>
where
    F: FnOnce(),
{
    guard.verify_root()?;
    validate_compile_attempt_namespace(&guard.root, RecordedCorpusRole::Compile)?;
    let stable = compile_attempt_path(&guard.root);
    let expected = RecordedCorpusCompileAttempt::compile().canonical_bytes()?;
    let stable_name = OsStr::new(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE);
    let stable_present = guard.verify_optional_owned_entry_bytes(
        stable_name,
        &expected,
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        "recorded corpus compile-attempt marker",
    )?;
    let residues = compile_attempt_residues(&guard.root)?;
    guard.verify_root()?;
    before_unlink();
    if stable_present {
        guard.unlink_owned_entry(stable_name)?;
    }
    for (path, bytes) in residues {
        remove_exact_owned_binding_residue(guard, &path, &bytes, "compile-attempt staging")?;
    }
    guard.sync_owned_root()?;
    if guard.verify_optional_owned_entry_bytes(
        stable_name,
        &expected,
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        "recorded corpus compile-attempt marker",
    )? {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus compile-attempt marker {} remained after guarded finish",
            stable.display()
        )));
    }
    guard.verify_root()
}

fn binding_temporary_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".tmp"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn binding_writing_name(name: &str) -> bool {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".writing"))
    else {
        return false;
    };
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn reserved_binding_entry_name(
    root: &Path,
    name: &OsStr,
) -> Result<Option<String>, SourceUnavailable> {
    let prefix = format!(".{RECORDED_CORPUS_BINDING_FILE}.");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.starts_with(prefix.as_bytes()) {
            return Ok(None);
        }
        let name = std::str::from_utf8(bytes).map_err(|_| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-UTF-8 entry in the reserved binding publication namespace",
                root.display()
            ))
        })?;
        Ok(Some(name.to_owned()))
    }

    #[cfg(not(unix))]
    {
        let lossy = name.to_string_lossy();
        if !lossy.starts_with(&prefix) {
            return Ok(None);
        }
        let name = name.to_str().ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus root {} contains a non-Unicode entry in the reserved binding publication namespace",
                root.display()
            ))
        })?;
        Ok(Some(name.to_owned()))
    }
}

#[derive(Default)]
struct BindingPublicationResidues {
    temporaries: Vec<(PathBuf, Vec<u8>)>,
    writings: Vec<(PathBuf, Vec<u8>)>,
}

fn binding_publication_residues(
    root: &Path,
    capture_writings: bool,
) -> Result<BindingPublicationResidues, SourceUnavailable> {
    let mut residues = BindingPublicationResidues::default();
    let mut reserved_count = 0usize;
    for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let name = entry.file_name();
        let Some(name) = reserved_binding_entry_name(root, &name)? else {
            continue;
        };
        if !binding_temporary_name(&name) && !binding_writing_name(&name) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unrecognized entry {name:?} in the reserved binding publication namespace",
                root.display()
            )));
        }
        reserved_count = reserved_count
            .checked_add(1)
            .ok_or_else(|| SourceUnavailable::new("binding residue count overflow"))?;
        if reserved_count > RESERVED_RESIDUE_MAX_ENTRIES {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} exceeds the fixed {RESERVED_RESIDUE_MAX_ENTRIES}-entry binding residue ceiling",
                root.display()
            )));
        }
        if binding_temporary_name(&name) {
            let path = entry.path();
            let bytes = capture_optional_control_file(
                &path,
                "recorded corpus binding temporary",
                BINDING_CONTROL_MAX_BYTES,
            )?
            .ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "recorded corpus binding temporary {} disappeared during validation",
                    path.display()
                ))
            })?;
            let _ = parse_binding_bytes(&path, &bytes)?;
            residues.temporaries.push((path, bytes));
        } else if binding_writing_name(&name) {
            // A `.writing` inode is deliberately non-authoritative: readers
            // ignore it, while the exclusive publisher may validate and
            // reclaim it before beginning a new publication attempt.
            if capture_writings {
                let path = entry.path();
                let bytes = capture_optional_control_file(
                    &path,
                    "recorded corpus non-authoritative binding staging file",
                    BINDING_CONTROL_MAX_BYTES,
                )?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding staging file {} disappeared during validation",
                        path.display()
                    ))
                })?;
                residues.writings.push((path, bytes));
            }
        }
    }
    residues
        .temporaries
        .sort_by(|left, right| left.0.cmp(&right.0));
    residues
        .writings
        .sort_by(|left, right| left.0.cmp(&right.0));
    Ok(residues)
}

#[cfg(test)]
fn binding_temporaries(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, SourceUnavailable> {
    Ok(binding_publication_residues(root, false)?.temporaries)
}

fn require_no_binding_temporaries(root: &Path) -> Result<(), SourceUnavailable> {
    let mut reserved_count = 0usize;
    for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let raw_name = entry.file_name();
        let Some(name) = reserved_binding_entry_name(root, &raw_name)? else {
            continue;
        };
        if !binding_temporary_name(&name) && !binding_writing_name(&name) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains unrecognized entry {name:?} in the reserved binding publication namespace",
                root.display()
            )));
        }
        reserved_count = reserved_count
            .checked_add(1)
            .ok_or_else(|| SourceUnavailable::new("binding residue count overflow"))?;
        if reserved_count > RESERVED_RESIDUE_MAX_ENTRIES {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} exceeds the fixed {RESERVED_RESIDUE_MAX_ENTRIES}-entry binding residue ceiling",
                root.display()
            )));
        }
        if binding_temporary_name(&name) {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus root {} contains an unpublished canonical binding temporary {}; writer recovery is required before reading",
                root.display(),
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn make_streaming_binding(
    meta_path: &Path,
    records_path: &Path,
    hidden_path: &Path,
    provenance: &ProvenanceBytes,
    meta_bytes: &[u8],
) -> Result<RecordedCorpusBinding, SourceUnavailable> {
    make_streaming_binding_retained(
        meta_path,
        records_path,
        hidden_path,
        provenance,
        meta_bytes,
        || {},
    )
    .map(|(binding, _, _)| binding)
}

fn make_streaming_binding_retained<F>(
    meta_path: &Path,
    records_path: &Path,
    hidden_path: &Path,
    provenance: &ProvenanceBytes,
    meta_bytes: &[u8],
    after_records_capture: F,
) -> Result<
    (
        RecordedCorpusBinding,
        StreamedGeneration,
        StreamedGeneration,
    ),
    SourceUnavailable,
>
where
    F: FnOnce(),
{
    let metadata_name = utf8_file_name(meta_path, "recorded corpus metadata")?;
    let records_name = utf8_file_name(records_path, "recorded corpus records")?;
    let hidden_name = utf8_file_name(hidden_path, "recorded corpus hidden stream")?;
    let (records, records_generation) = stream_regular_file_binding_retained(
        records_path,
        &records_name,
        "recorded corpus records",
    )?;
    after_records_capture();
    let (hidden, hidden_generation) = stream_regular_file_binding_retained(
        hidden_path,
        &hidden_name,
        "recorded corpus hidden stream",
    )?;
    let binding = RecordedCorpusBinding {
        schema: RECORDED_CORPUS_BINDING_SCHEMA.to_owned(),
        manifest: RecordedCorpusFileBinding::from_bytes(
            observation::MANIFEST_FILE,
            provenance[0].as_deref(),
        )?,
        attention_operator: RecordedCorpusFileBinding::from_bytes(
            ATTENTION_OPERATOR_BINDING_FILE,
            provenance[1].as_deref(),
        )?,
        dense_operator: RecordedCorpusFileBinding::from_bytes(
            DENSE_OPERATOR_BINDING_FILE,
            provenance[2].as_deref(),
        )?,
        metadata: RecordedCorpusFileBinding::from_bytes(&metadata_name, Some(meta_bytes))?,
        records,
        hidden,
    };
    Ok((binding, records_generation, hidden_generation))
}

fn validate_finalized_corpus_shape(
    meta_bytes: &[u8],
    records: &RecordedCorpusFileBinding,
) -> Result<(), SourceUnavailable> {
    if meta_bytes.len() != 25 || meta_bytes[24] != 1 {
        return Err(SourceUnavailable::new(
            "recorded corpus binding requires one exact finalized 25-byte metadata checkpoint",
        ));
    }
    let count = u64::from_le_bytes(
        meta_bytes[0..8]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid finalized corpus count"))?,
    );
    let length = records.length.ok_or_else(|| {
        SourceUnavailable::new("recorded corpus records are absent during binding publication")
    })?;
    if count == 0
        || ![88u64, 48, 32, 12]
            .into_iter()
            .any(|width| count.checked_mul(width) == Some(length))
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus has {count} rows and {length} record bytes, not one exact registered finalized layout"
        )));
    }
    Ok(())
}

struct CapturedBindingMembers<'a> {
    meta_path: &'a Path,
    records_path: &'a Path,
    hidden_path: &'a Path,
    provenance: &'a ProvenanceBytes,
    meta_bytes: &'a [u8],
    records_bytes: &'a [u8],
    hidden_bytes: Option<&'a [u8]>,
}

fn validate_binding_generation(
    binding: &RecordedCorpusBinding,
    members: &CapturedBindingMembers<'_>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    binding.manifest.validate(
        observation::MANIFEST_FILE,
        members.provenance[0].as_deref(),
        context,
    )?;
    binding.attention_operator.validate(
        ATTENTION_OPERATOR_BINDING_FILE,
        members.provenance[1].as_deref(),
        context,
    )?;
    binding.dense_operator.validate(
        DENSE_OPERATOR_BINDING_FILE,
        members.provenance[2].as_deref(),
        context,
    )?;
    binding.metadata.validate(
        &utf8_file_name(members.meta_path, "recorded corpus metadata")?,
        Some(members.meta_bytes),
        context,
    )?;
    binding.records.validate(
        &utf8_file_name(members.records_path, "recorded corpus records")?,
        Some(members.records_bytes),
        context,
    )?;
    binding.hidden.validate(
        &utf8_file_name(members.hidden_path, "recorded corpus hidden stream")?,
        members.hidden_bytes,
        context,
    )
}

fn parse_execution_identity(
    root: &Path,
    corpus_meta: &Path,
    corpus_records: &Path,
    bytes: &ProvenanceBytes,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let manifest_path = root.join(observation::MANIFEST_FILE);
    let manifest = bytes[0]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("observation manifest {}", manifest_path.display()),
            )
            .map_err(|mut error| {
                error.reason = format!(
                    "{}: malformed observation manifest: {}",
                    manifest_path.display(),
                    error.reason
                );
                error
            })?;
            ObservationManifest::parse_captured_execution_identity(root, &manifest_path, bytes)
                .map_err(|mut error| {
                    error.reason = format!(
                        "{}: malformed observation manifest: {}",
                        manifest_path.display(),
                        error.reason
                    );
                    error
                })
        })
        .transpose()?;
    let manifest_present = manifest.is_some();

    let attention_path = root.join(ATTENTION_OPERATOR_BINDING_FILE);
    let sidecar_attention = bytes[1]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("attention operator sidecar {}", attention_path.display()),
            )?;
            let operator: AttentionOperatorSpec =
                serde_json::from_slice(bytes).map_err(|error| {
                    SourceUnavailable::new(format!(
                        "invalid attention operator sidecar {}: {error}",
                        attention_path.display()
                    ))
                })?;
            observation::validate_registered_source_attention_operator(&operator)?;
            Ok::<_, SourceUnavailable>(operator)
        })
        .transpose()?;
    let dense_path = root.join(DENSE_OPERATOR_BINDING_FILE);
    let sidecar_dense = bytes[2]
        .as_deref()
        .map(|bytes| {
            reject_duplicate_json(
                bytes,
                &format!("dense operator sidecar {}", dense_path.display()),
            )?;
            let operator: DenseOperatorSpec = serde_json::from_slice(bytes).map_err(|error| {
                SourceUnavailable::new(format!(
                    "invalid dense operator sidecar {}: {error}",
                    dense_path.display()
                ))
            })?;
            observation::validate_registered_source_dense_operator(&operator)?;
            Ok::<_, SourceUnavailable>(operator)
        })
        .transpose()?;
    let manifest_attention = manifest
        .as_ref()
        .and_then(|manifest| manifest.attention_operator.clone());
    let manifest_dense = manifest
        .as_ref()
        .and_then(|manifest| manifest.dense_operator.clone());

    if (manifest_present || sidecar_attention.is_some() || sidecar_dense.is_some())
        && !is_canonical_pair(corpus_meta, corpus_records)
    {
        return Err(SourceUnavailable::new(format!(
            "recorded-corpus provenance in {} applies only to the canonical corpus.meta/corpus.records or state.bin/merged.bin pair; refusing to attach it to {}/{}",
            root.display(),
            corpus_meta.display(),
            corpus_records.display()
        )));
    }
    if let Some(sidecar) = sidecar_attention.as_ref()
        && manifest_present
        && manifest_attention.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares attention operator {}/{} but {} records the legacy operatorless era",
            attention_path.display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display()
        )));
    }
    if let Some(sidecar) = sidecar_dense.as_ref()
        && manifest_present
        && manifest_dense.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares dense operator {}/{} but {} records the dense-operator-absent era",
            dense_path.display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display()
        )));
    }

    let attention_operator = match (sidecar_attention, manifest_attention) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different attention operators",
                attention_path.display(),
                manifest_path.display()
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, manifest) => manifest,
    };
    let dense_operator = match (sidecar_dense, manifest_dense) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different dense operators",
                dense_path.display(),
                manifest_path.display()
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, manifest) => manifest,
    };
    if attention_operator.is_none()
        && let Some(dense) = dense_operator.as_ref()
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus declares dense operator {}/{} without an attention operator",
            dense.id, dense.version
        )));
    }
    observation::validate_source_execution_identity(
        attention_operator.as_ref(),
        dense_operator.as_ref(),
        "recorded corpus provenance",
    )?;
    Ok(RecordedCorpusExecutionIdentity {
        attention_operator,
        dense_operator,
    })
}

/// Require the caller-declared pair to equal the exact recorded pair. Typed
/// absence is significant: a current dense corpus cannot be relabeled by
/// omitting `--dense-operator`, and a legacy corpus cannot acquire one.
pub fn require_execution_identity(
    recorded: &RecordedCorpusExecutionIdentity,
    attention: Option<&AttentionOperatorSpec>,
    dense: Option<&DenseOperatorSpec>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    if let Some(attention) = attention {
        observation::validate_registered_source_attention_operator(attention)?;
    }
    if let Some(dense) = dense {
        observation::validate_registered_source_dense_operator(dense)?;
    }
    observation::validate_source_execution_identity(attention, dense, context)?;
    if recorded.attention_operator.as_ref() != attention {
        return Err(SourceUnavailable::new(format!(
            "{context}: requested attention execution identity does not match the recorded corpus; refusing before output mutation"
        )));
    }
    if recorded.dense_operator.as_ref() != dense {
        return Err(SourceUnavailable::new(format!(
            "{context}: requested dense execution identity does not match the recorded corpus; refusing before output mutation"
        )));
    }
    Ok(())
}

pub(crate) fn capture_with_hooks<F, G>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_provenance_capture: F,
    after_corpus_capture: G,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    require_no_binding_temporaries(&root)?;
    let stable_binding_path = binding_path(&root);
    let binding_bytes = capture_optional_control_file(
        &stable_binding_path,
        "recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?;
    let binding = binding_bytes
        .as_deref()
        .map(|bytes| parse_binding_bytes(&stable_binding_path, bytes))
        .transpose()?;
    let compile_attempt_path = compile_attempt_path(&root);
    let compile_attempt = capture_optional_control_file(
        &compile_attempt_path,
        "recorded corpus compile-attempt marker",
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
    )?;
    if let Some(bytes) = compile_attempt.as_deref() {
        parse_compile_attempt_bytes(&compile_attempt_path, bytes)?;
        match binding.as_ref() {
            Some(binding) => {
                validate_binding_role(binding, RecordedCorpusRole::Compile, &stable_binding_path)?
            }
            None => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} has an unfinished compile attempt without a stable generation binding",
                    root.display()
                )));
            }
        }
    }

    let provenance = capture_provenance(&root)?;
    after_provenance_capture();

    let meta_bytes = capture_optional_control_file(
        &resolved_meta,
        "recorded corpus metadata",
        METADATA_CONTROL_MAX_BYTES,
    )?
    .ok_or_else(|| {
        SourceUnavailable::new(format!(
            "recorded corpus metadata {} disappeared during capture",
            resolved_meta.display()
        ))
    })?;
    let records_bytes =
        capture_optional_regular_file(&resolved_records, "recorded corpus records")?.ok_or_else(
            || {
                SourceUnavailable::new(format!(
                    "recorded corpus records {} disappeared during capture",
                    resolved_records.display()
                ))
            },
        )?;
    let hidden_path = hidden_path(&resolved_records);
    let hidden_bytes =
        capture_optional_regular_file(&hidden_path, "recorded corpus hidden stream")?;

    after_corpus_capture();
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;

    let binding_cid = if let (Some(binding), Some(bytes)) = (&binding, binding_bytes.as_deref()) {
        validate_binding_generation(
            binding,
            &CapturedBindingMembers {
                meta_path: &resolved_meta,
                records_path: &resolved_records,
                hidden_path: &hidden_path,
                provenance: &provenance,
                meta_bytes: &meta_bytes,
                records_bytes: &records_bytes,
                hidden_bytes: hidden_bytes.as_deref(),
            },
            &format!("recorded corpus binding {}", stable_binding_path.display()),
        )?;
        Some(format!("blake3:{}", blake3::hash(bytes).to_hex()))
    } else {
        // Markerless compatibility retains the historical repeated-read
        // defense. It is intentionally not advertised as an adversarial
        // cross-file transaction: only the canonical binding supplies that
        // guarantee.
        verify_provenance(&root, &provenance)?;
        verify_captured_optional_control_file(
            &resolved_meta,
            "recorded corpus metadata",
            Some(meta_bytes.as_slice()),
            METADATA_CONTROL_MAX_BYTES,
        )?;
        verify_captured_optional_regular_file(
            &resolved_records,
            "recorded corpus records",
            Some(records_bytes.as_slice()),
        )?;
        verify_captured_optional_regular_file(
            &hidden_path,
            "recorded corpus hidden stream",
            hidden_bytes.as_deref(),
        )?;
        verify_captured_optional_control_file(
            &stable_binding_path,
            "recorded corpus generation binding",
            None,
            BINDING_CONTROL_MAX_BYTES,
        )?;
        require_no_binding_temporaries(&root)?;
        if let Some(dense) = execution.dense_operator.as_ref() {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus in {} declares dense operator {}/{} but has no canonical {}; dense-present generations must be committed last before use",
                root.display(),
                dense.id,
                dense.version,
                RECORDED_CORPUS_BINDING_FILE
            )));
        }
        None
    };
    Ok(RecordedCorpusSnapshot {
        execution,
        attention_operator_bytes: provenance[1].clone(),
        dense_operator_bytes: provenance[2].clone(),
        meta_bytes,
        records_bytes,
        hidden_bytes,
        binding,
        binding_cid,
    })
}

/// Capture and reverify provenance and corpus bytes as one transaction.
pub fn capture(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable> {
    capture_with_hooks(corpus_meta, corpus_records, || {}, || {})
}

/// Open one bounded-memory recorded-corpus generation.
///
/// Large records and hidden-state members are hashed through retained
/// no-follow handles and rewound; their bodies are never accumulated here.
/// A consumer may seek/read those handles and must call
/// [`RecordedCorpusStreamSnapshot::verify_generation`] after its final read
/// and immediately before publishing any derived generation. A canonical
/// binding supplies the cross-file generation commit. Markerless snapshots
/// retain and recheck every opened generation, but cannot by themselves
/// exclude a non-cooperating cross-file ABA publisher; production derivations
/// therefore hold a source [`RecordedCorpusProducerGuard`] across open, read,
/// and final verification for that compatibility lane.
pub fn open_stream(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusStreamSnapshot, SourceUnavailable> {
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    require_no_binding_temporaries(&root)?;
    let stable_path = binding_path(&root);
    let binding_bytes = capture_optional_control_file(
        &stable_path,
        "recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?;
    let binding = binding_bytes
        .as_deref()
        .map(|bytes| parse_binding_bytes(&stable_path, bytes))
        .transpose()?;
    let marker_path = compile_attempt_path(&root);
    let marker_bytes = capture_optional_control_file(
        &marker_path,
        "recorded corpus compile-attempt marker",
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
    )?;
    if let Some(bytes) = marker_bytes.as_deref() {
        parse_compile_attempt_bytes(&marker_path, bytes)?;
        match binding.as_ref() {
            Some(binding) => {
                validate_binding_role(binding, RecordedCorpusRole::Compile, &stable_path)?
            }
            None => {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus root {} has an unfinished compile attempt without a stable generation binding",
                    root.display()
                )));
            }
        }
    }

    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_control_file(
        &resolved_meta,
        "recorded corpus metadata",
        METADATA_CONTROL_MAX_BYTES,
    )?
    .ok_or_else(|| SourceUnavailable::new("recorded corpus metadata is absent"))?;
    let hidden = hidden_path(&resolved_records);
    let records_name = utf8_file_name(&resolved_records, "recorded corpus records")?;
    let hidden_name = utf8_file_name(&hidden, "recorded corpus hidden stream")?;
    let records = open_verified_corpus_member(
        &resolved_records,
        &records_name,
        "recorded corpus records",
        binding.as_ref().map(|binding| &binding.records),
    )?
    .ok_or_else(|| SourceUnavailable::new("recorded corpus records are absent"))?;
    let hidden_member = open_verified_corpus_member(
        &hidden,
        &hidden_name,
        "recorded corpus hidden stream",
        binding.as_ref().map(|binding| &binding.hidden),
    )?;
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;

    let binding_cid = if let (Some(binding), Some(bytes)) =
        (binding.as_ref(), binding_bytes.as_deref())
    {
        let context = format!("recorded corpus binding {}", stable_path.display());
        binding.manifest.validate(
            observation::MANIFEST_FILE,
            provenance[0].as_deref(),
            &context,
        )?;
        binding.attention_operator.validate(
            ATTENTION_OPERATOR_BINDING_FILE,
            provenance[1].as_deref(),
            &context,
        )?;
        binding.dense_operator.validate(
            DENSE_OPERATOR_BINDING_FILE,
            provenance[2].as_deref(),
            &context,
        )?;
        binding.metadata.validate(
            &utf8_file_name(&resolved_meta, "recorded corpus metadata")?,
            Some(&meta_bytes),
            &context,
        )?;
        Some(format!("blake3:{}", blake3::hash(bytes).to_hex()))
    } else {
        if let Some(dense) = execution.dense_operator.as_ref() {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus in {} declares dense operator {}/{} but has no canonical {}; dense-present generations must be committed last before use",
                root.display(),
                dense.id,
                dense.version,
                RECORDED_CORPUS_BINDING_FILE
            )));
        }
        None
    };

    let snapshot = RecordedCorpusStreamSnapshot {
        execution,
        attention_operator_bytes: provenance[1].clone(),
        dense_operator_bytes: provenance[2].clone(),
        meta_bytes,
        records,
        hidden: hidden_member,
        binding_cid,
        root,
        provenance,
        meta_path: resolved_meta,
        binding_path: stable_path,
        binding_bytes,
        marker_path,
        marker_bytes,
        hidden_path: hidden,
    };
    snapshot.verify_generation()?;
    Ok(snapshot)
}

/// Bounded-memory identity resolver for status/copy callers. It validates the
/// complete bound generation while retaining no corpus body in memory.
pub fn execution_identity(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    let source_guard = RecordedCorpusReaderGuard::try_acquire(&root)?;
    let snapshot = open_stream(corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    source_guard.verify()?;
    Ok(snapshot.execution)
}

/// Open a stream while the caller already owns the exact producer root. This
/// avoids self-contention in source/server publication paths while preserving
/// the same root-identity and final-generation checks.
pub fn open_stream_under_guard(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusStreamSnapshot, SourceUnavailable> {
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    if !guard.protects_directory(&root)? {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard for {} does not own stream root {}",
            guard.root().display(),
            root.display()
        )));
    }
    let snapshot = open_stream(corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    guard.verify_owned_root()?;
    Ok(snapshot)
}

/// Resolve only the execution identity while the caller retains exclusive
/// ownership of the exact producer root. This is the identity-only companion
/// to [`open_stream_under_guard`] and cannot self-contend on the same lock.
pub fn execution_identity_under_producer_guard(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let snapshot = open_stream_under_guard(guard, corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    guard.verify_owned_root()?;
    Ok(snapshot.execution)
}

/// Open a stream under a caller-owned sorted reader-pin set. At least one pin
/// must identify the exact corpus directory; unrelated or typed-absent pins do
/// not authorize a pathname read.
pub fn open_stream_under_reader_pins(
    pins: &RecordedCorpusReaderPins,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusStreamSnapshot, SourceUnavailable> {
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    if !pins.protects_directory(&root)? {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus reader pins do not protect stream root {}",
            root.display()
        )));
    }
    let snapshot = open_stream(corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    pins.verify()?;
    Ok(snapshot)
}

/// Resolve only the execution identity under one retained shared-pin
/// authority, avoiding a nested reader acquisition and its avoidable race.
pub fn execution_identity_under_reader_pins(
    pins: &RecordedCorpusReaderPins,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let snapshot = open_stream_under_reader_pins(pins, corpus_meta, corpus_records)?;
    snapshot.verify_generation()?;
    pins.verify()?;
    Ok(snapshot.execution)
}

/// Open a source generation for a production derivation. Bound generations
/// need no writer lock because the canonical binding commits every captured
/// member. Markerless legacy/attention-only inputs are reopened under an
/// exclusive producer guard, which the caller retains until its final source
/// verification and derived-byte staging are complete.
pub fn open_stream_for_derivation(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<
    (
        RecordedCorpusStreamSnapshot,
        Option<RecordedCorpusProducerGuard>,
    ),
    SourceUnavailable,
> {
    let initial = open_stream(corpus_meta, corpus_records)?;
    if initial.binding_cid.is_some() {
        return Ok((initial, None));
    }
    drop(initial);
    let (root, _, _) = resolved_corpus_paths(corpus_meta, corpus_records)?;
    let guard = RecordedCorpusProducerGuard::try_acquire(&root)?;
    let snapshot = open_stream_under_guard(&guard, corpus_meta, corpus_records)?;
    if snapshot.binding_cid.is_some() {
        // A cooperating writer cannot publish while this exclusive guard is
        // held; reaching a bound generation here therefore means the first
        // markerless read raced an uncoordinated publisher. Fail closed rather
        // than silently changing the derivation contract mid-open.
        return Err(SourceUnavailable::new(format!(
            "recorded corpus {} changed from markerless to binding-committed while derivation ownership was acquired",
            root.display()
        )));
    }
    Ok((snapshot, Some(guard)))
}

fn create_binding_temporary(
    guard: &RecordedCorpusProducerGuard,
    bytes: &[u8],
) -> Result<std::ffi::OsString, SourceUnavailable> {
    loop {
        let sequence = RECORDED_CORPUS_BINDING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let writing = std::ffi::OsString::from(format!(
            ".{RECORDED_CORPUS_BINDING_FILE}.{}.{}.writing",
            std::process::id(),
            sequence
        ));
        let temporary = std::ffi::OsString::from(format!(
            ".{RECORDED_CORPUS_BINDING_FILE}.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        if guard
            .open_optional_owned_entry_for_read(&temporary)?
            .is_some()
        {
            continue;
        }
        match guard.create_new_owned_entry(&writing) {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
                    let _ = guard.unlink_owned_entry(&writing);
                    return Err(SourceUnavailable::new(format!(
                        "recorded corpus binding staging file {writing:?} cannot be written: {error}",
                    )));
                }
                drop(file);
                guard.rename_owned_entry(&writing, &temporary).map_err(|error| {
                    SourceUnavailable::new(format!(
                        "recorded corpus binding staging promotion {writing:?} -> {temporary:?} failed: {error}",
                    ))
                })?;
                guard.sync_owned_root()?;
                return Ok(temporary);
            }
            Err(error) => {
                if guard
                    .open_optional_owned_entry_for_read(&writing)?
                    .is_some()
                {
                    continue;
                }
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus binding staging file {writing:?} cannot be created: {error}",
                )));
            }
        }
    }
}

fn remove_exact_owned_binding_residue(
    guard: &RecordedCorpusProducerGuard,
    path: &Path,
    expected: &[u8],
    context: &str,
) -> Result<(), SourceUnavailable> {
    let name = path.file_name().ok_or_else(|| {
        SourceUnavailable::new(format!("{context} {} has no leaf name", path.display()))
    })?;
    let max_bytes = u64::try_from(expected.len())
        .map_err(|_| SourceUnavailable::new(format!("{context} length overflows u64")))?;
    guard.verify_owned_entry_bytes(name, expected, max_bytes, context)?;
    guard.unlink_owned_entry(name)
}

/// Read-only verification that every stable/recoverable binding record, when
/// present, is the canonical binding for the exact current member bytes.
/// Deterministic producers call this after all destination-member preflights
/// and before publishing their compile-attempt marker, so a terminal recovery
/// conflict leaves the corpus root byte-identical.
pub fn preflight_binding_evidence_matches_current(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<(), SourceUnavailable> {
    preflight_binding_evidence_matches_current_inner(guard, corpus_meta, corpus_records, true)
}

/// Publication state accepted for a resumable source-generation update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceUpdatePublicationState {
    /// An exact canonical binding temporary commits the current member bytes
    /// and must be promoted before the generator changes them again.
    pub recoverable_binding: bool,
    /// A stable compile-attempt marker proves an interrupted same-role update.
    pub attempt_active: bool,
}

/// Validate source-update binding evidence without confusing an honest
/// binding-last crash with a deterministic-output conflict.
///
/// With no active compile marker, every stable binding and temporary must
/// commit the exact current member bytes. With an active marker, a stable
/// binding may remain the canonical last-good generation while the
/// authoritative resume checkpoint has advanced; canonical temporaries must
/// still match the current bytes exactly. The caller is responsible for
/// validating that checkpoint before invoking this seam.
pub fn preflight_source_update_publication(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<SourceUpdatePublicationState, SourceUnavailable> {
    guard.verify_root()?;
    let recoverable_binding =
        guard.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
    let marker = compile_attempt_path(&guard.root);
    let attempt_active = capture_optional_control_file(
        &marker,
        "recorded corpus compile-attempt marker",
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
    )?
    .map(|bytes| parse_compile_attempt_bytes(&marker, &bytes))
    .transpose()?
    .is_some();
    let stable = capture_optional_control_file(
        &binding_path(&guard.root),
        "recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?
    .is_some();
    if recoverable_binding || (stable && !attempt_active) {
        preflight_binding_evidence_matches_current_inner(
            guard,
            corpus_meta,
            corpus_records,
            !attempt_active,
        )?;
    }
    if stable && attempt_active {
        // The compile-attempt marker records only the producer role.  It is
        // deliberately not an authority to splice a different manifest or
        // arithmetic era onto corpus bytes advanced by a resume checkpoint.
        // A legitimate A -> B source resume may change metadata/records while
        // retaining the last-good binding A, but every provenance member must
        // still equal the generation that authorized that resume.
        preflight_stable_binding_provenance_matches_current(guard, corpus_meta, corpus_records)?;
    }
    Ok(SourceUpdatePublicationState {
        recoverable_binding,
        attempt_active,
    })
}

fn preflight_stable_binding_provenance_matches_current(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<(), SourceUnavailable> {
    guard.verify_root()?;
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    if root != guard.root {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard owns {}, but source-resume provenance preflight resolved to {}",
            guard.root.display(),
            root.display()
        )));
    }
    if !is_canonical_pair(&resolved_meta, &resolved_records) {
        return Err(SourceUnavailable::new(
            "source-resume provenance preflight requires canonical corpus.meta/corpus.records",
        ));
    }
    let stable_path = binding_path(&root);
    let stable_bytes = capture_optional_control_file(
        &stable_path,
        "stable recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?
    .ok_or_else(|| {
        SourceUnavailable::new(
            "source-resume provenance preflight requires the observed stable binding",
        )
    })?;
    let stable = parse_binding_bytes(&stable_path, &stable_bytes)?;
    validate_binding_role(&stable, RecordedCorpusRole::Compile, &stable_path)?;
    let provenance = capture_provenance(&root)?;
    let context = format!(
        "stable recorded corpus binding {} source-resume provenance",
        stable_path.display()
    );
    stable.manifest.validate(
        observation::MANIFEST_FILE,
        provenance[0].as_deref(),
        &context,
    )?;
    stable.attention_operator.validate(
        ATTENTION_OPERATOR_BINDING_FILE,
        provenance[1].as_deref(),
        &context,
    )?;
    stable.dense_operator.validate(
        DENSE_OPERATOR_BINDING_FILE,
        provenance[2].as_deref(),
        &context,
    )?;
    guard.verify_root()
}

fn preflight_binding_evidence_matches_current_inner(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
    check_stable: bool,
) -> Result<(), SourceUnavailable> {
    guard.verify_root()?;
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    if root != guard.root {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard owns {}, but binding preflight resolved to {}",
            guard.root.display(),
            root.display()
        )));
    }
    if !is_canonical_pair(&resolved_meta, &resolved_records) {
        return Err(SourceUnavailable::new(
            "binding preflight requires a canonical corpus.meta/corpus.records or state.bin/merged.bin pair",
        ));
    }
    let role = if resolved_meta.file_name() == Some(OsStr::new("corpus.meta")) {
        RecordedCorpusRole::Compile
    } else {
        RecordedCorpusRole::Observation
    };
    validate_role_inventory(&root, role)?;
    validate_compile_attempt_namespace(&root, role)?;
    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_control_file(
        &resolved_meta,
        "recorded corpus metadata",
        METADATA_CONTROL_MAX_BYTES,
    )?
    .ok_or_else(|| SourceUnavailable::new("binding preflight metadata is absent"))?;
    let resolved_hidden = hidden_path(&resolved_records);
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;
    let binding = make_streaming_binding(
        &resolved_meta,
        &resolved_records,
        &resolved_hidden,
        &provenance,
        &meta_bytes,
    )?;
    if !binding.records.present {
        return Err(SourceUnavailable::new(
            "binding preflight records are absent",
        ));
    }
    if execution.dense_operator.is_some() {
        validate_finalized_corpus_shape(&meta_bytes, &binding.records)?;
    }
    let expected = binding.canonical_bytes()?;
    let stable_path = binding_path(&root);
    if let Some(bytes) = capture_optional_control_file(
        &stable_path,
        "recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?
    .filter(|_| check_stable)
    {
        let binding = parse_binding_bytes(&stable_path, &bytes)?;
        validate_binding_role(&binding, role, &stable_path)?;
        if bytes != expected {
            return Err(SourceUnavailable::new(format!(
                "stable recorded corpus binding {} does not commit the exact planned generation",
                stable_path.display()
            )));
        }
    }
    let residues = binding_publication_residues(&root, true)?;
    for (path, bytes) in residues.temporaries {
        let binding = parse_binding_bytes(&path, &bytes)?;
        validate_binding_role(&binding, role, &path)?;
        if bytes != expected {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus binding temporary {} does not commit the exact planned generation",
                path.display()
            )));
        }
    }
    guard.verify_root()
}

/// Publish or repair the canonical generation binding after a producer has
/// completed a canonical corpus pair. The binding is the final commit point.
///
/// Canonical publisher temporaries are recovered only when every byte exactly
/// matches the intended current generation. Malformed, nonregular, unknown,
/// or conflicting canonical residue remains untouched and terminal.
///
/// The reserved `.{RECORDED_CORPUS_BINDING_FILE}.<pid>.<sequence>.writing`
/// namespace is non-authoritative and ignored by readers. This function may
/// reclaim registry-shaped regular `.writing` residue, including zero or
/// partial bytes left by process death, only under this API's hard precondition
/// that its caller holds exclusive producer ownership of the corpus root for
/// the entire publication attempt. Callers must not use this function as an
/// unlocked cleanup utility.
pub fn publish_binding(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<String, SourceUnavailable> {
    publish_binding_with_hooks(guard, corpus_meta, corpus_records, || {}, || {})
}

fn verify_publication_generation(
    guard: &RecordedCorpusProducerGuard,
    provenance: &ProvenanceBytes,
    meta_path: &Path,
    meta_bytes: &[u8],
    marker_bytes: Option<&[u8]>,
    records_generation: &StreamedGeneration,
    hidden_generation: &StreamedGeneration,
) -> Result<(), SourceUnavailable> {
    guard.verify_root()?;
    verify_provenance(&guard.root, provenance)?;
    verify_captured_optional_control_file(
        meta_path,
        "recorded corpus metadata",
        Some(meta_bytes),
        METADATA_CONTROL_MAX_BYTES,
    )?;
    verify_captured_optional_control_file(
        &compile_attempt_path(&guard.root),
        "recorded corpus compile-attempt marker",
        marker_bytes,
        COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
    )?;
    records_generation.verify("recorded corpus records")?;
    hidden_generation.verify("recorded corpus hidden stream")?;
    guard.verify_root()
}

fn publish_binding_with_hooks<F, G>(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
    after_records_capture: F,
    before_commit: G,
) -> Result<String, SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce(),
{
    guard.verify_root()?;
    let (root, resolved_meta, resolved_records) =
        resolved_corpus_paths(corpus_meta, corpus_records)?;
    if root != guard.root {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus producer guard owns {}, but binding publication resolved to {}; refusing cross-root publication before mutation",
            guard.root.display(),
            root.display()
        )));
    }
    guard.verify_root()?;
    if !is_canonical_pair(&resolved_meta, &resolved_records) {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus generation bindings are supported only for canonical corpus.meta/corpus.records or state.bin/merged.bin pairs, not {}/{}",
            resolved_meta.display(),
            resolved_records.display()
        )));
    }
    let intended_role = if resolved_meta.file_name() == Some(OsStr::new("corpus.meta")) {
        RecordedCorpusRole::Compile
    } else {
        RecordedCorpusRole::Observation
    };
    validate_role_inventory(&root, intended_role)?;
    validate_compile_attempt_namespace(&root, intended_role)?;
    let marker_bytes = if intended_role == RecordedCorpusRole::Compile {
        let marker_path = compile_attempt_path(&root);
        let marker = capture_optional_control_file(
            &marker_path,
            "recorded corpus compile-attempt marker",
            COMPILE_ATTEMPT_CONTROL_MAX_BYTES,
        )?
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "compile-role binding publication in {} requires an exact stable {RECORDED_CORPUS_COMPILE_ATTEMPT_FILE} marker before any commit mutation",
                root.display()
            ))
        })?;
        parse_compile_attempt_bytes(&marker_path, &marker)?;
        Some(marker)
    } else {
        None
    };
    let provenance = capture_provenance(&root)?;
    let meta_bytes = capture_optional_control_file(
        &resolved_meta,
        "recorded corpus metadata",
        METADATA_CONTROL_MAX_BYTES,
    )?
    .ok_or_else(|| {
        SourceUnavailable::new(format!(
            "recorded corpus metadata {} is absent during binding publication",
            resolved_meta.display()
        ))
    })?;
    let resolved_hidden = hidden_path(&resolved_records);
    let execution =
        parse_execution_identity(&root, &resolved_meta, &resolved_records, &provenance)?;
    let (binding, records_generation, hidden_generation) = make_streaming_binding_retained(
        &resolved_meta,
        &resolved_records,
        &resolved_hidden,
        &provenance,
        &meta_bytes,
        after_records_capture,
    )?;
    verify_publication_generation(
        guard,
        &provenance,
        &resolved_meta,
        &meta_bytes,
        marker_bytes.as_deref(),
        &records_generation,
        &hidden_generation,
    )?;
    if !binding.records.present {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus records {} are absent during binding publication",
            resolved_records.display()
        )));
    }
    if execution.dense_operator.is_some() {
        validate_finalized_corpus_shape(&meta_bytes, &binding.records).map_err(|mut error| {
            error.reason = format!(
                "dense-present recorded corpus in {} is incomplete or malformed: {}",
                root.display(),
                error.reason
            );
            error
        })?;
    }
    let expected = binding.canonical_bytes()?;
    let expected_cid = format!("blake3:{}", blake3::hash(&expected).to_hex());
    let stable_path = binding_path(&root);
    let stable = capture_optional_control_file(
        &stable_path,
        "recorded corpus generation binding",
        BINDING_CONTROL_MAX_BYTES,
    )?;
    if let Some(bytes) = stable.as_deref() {
        let existing = parse_binding_bytes(&stable_path, bytes)?;
        validate_binding_role(&existing, intended_role, &stable_path)?;
    }
    let residues = binding_publication_residues(&root, true)?;
    let temporaries = residues.temporaries;
    let writings = residues.writings;
    if let Some((path, _)) = temporaries
        .iter()
        .find(|(_, bytes)| bytes.as_slice() != expected.as_slice())
    {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus binding temporary {} conflicts with the intended current generation; refusing recovery before mutation",
            path.display()
        )));
    }

    verify_publication_generation(
        guard,
        &provenance,
        &resolved_meta,
        &meta_bytes,
        marker_bytes.as_deref(),
        &records_generation,
        &hidden_generation,
    )?;

    let mut before_commit = Some(before_commit);
    if stable.as_deref() == Some(expected.as_slice()) {
        before_commit
            .take()
            .expect("publication hook is called exactly once")();
        verify_publication_generation(
            guard,
            &provenance,
            &resolved_meta,
            &meta_bytes,
            marker_bytes.as_deref(),
            &records_generation,
            &hidden_generation,
        )?;
        for (path, bytes) in temporaries {
            remove_exact_owned_binding_residue(
                guard,
                &path,
                &bytes,
                "recorded corpus binding temporary",
            )?;
        }
        for (path, bytes) in writings {
            remove_exact_owned_binding_residue(
                guard,
                &path,
                &bytes,
                "recorded corpus non-authoritative binding staging file",
            )?;
        }
        guard.sync_owned_root()?;
        guard.verify_owned_entry_bytes(
            OsStr::new(RECORDED_CORPUS_BINDING_FILE),
            &expected,
            BINDING_CONTROL_MAX_BYTES,
            "stable recorded corpus generation binding",
        )?;
        verify_publication_generation(
            guard,
            &provenance,
            &resolved_meta,
            &meta_bytes,
            marker_bytes.as_deref(),
            &records_generation,
            &hidden_generation,
        )?;
        return Ok(expected_cid);
    }

    for (path, bytes) in writings {
        remove_exact_owned_binding_residue(
            guard,
            &path,
            &bytes,
            "recorded corpus non-authoritative binding staging file",
        )?;
    }

    let selected = match temporaries.first() {
        Some((path, bytes)) => {
            let name = path.file_name().ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "recorded corpus binding temporary {} has no leaf name",
                    path.display()
                ))
            })?;
            guard.verify_owned_entry_bytes(
                name,
                bytes,
                BINDING_CONTROL_MAX_BYTES,
                "recorded corpus binding temporary",
            )?;
            if bytes.as_slice() != expected.as_slice() {
                return Err(SourceUnavailable::new(format!(
                    "recorded corpus binding temporary {} changed before publication",
                    path.display()
                )));
            }
            name.to_os_string()
        }
        None => create_binding_temporary(guard, &expected)?,
    };

    verify_publication_generation(
        guard,
        &provenance,
        &resolved_meta,
        &meta_bytes,
        marker_bytes.as_deref(),
        &records_generation,
        &hidden_generation,
    )?;

    before_commit
        .take()
        .expect("publication hook is called exactly once")();

    guard
        .rename_owned_entry(&selected, OsStr::new(RECORDED_CORPUS_BINDING_FILE))
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "recorded corpus binding publication {selected:?} -> {} failed: {error}",
                stable_path.display(),
            ))
        })?;
    for (path, bytes) in temporaries {
        if path.file_name() != Some(selected.as_os_str()) {
            remove_exact_owned_binding_residue(
                guard,
                &path,
                &bytes,
                "recorded corpus binding temporary",
            )?;
        }
    }
    guard.sync_owned_root()?;
    guard.verify_owned_entry_bytes(
        OsStr::new(RECORDED_CORPUS_BINDING_FILE),
        &expected,
        BINDING_CONTROL_MAX_BYTES,
        "published recorded corpus generation binding",
    )?;
    verify_publication_generation(
        guard,
        &provenance,
        &resolved_meta,
        &meta_bytes,
        marker_bytes.as_deref(),
        &records_generation,
        &hidden_generation,
    )?;
    Ok(expected_cid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "uor-r4-recorded-corpus-{label}-{}-{}",
            std::process::id(),
            NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_operator<T: serde::Serialize>(path: &Path, operator: &T) {
        let mut bytes = serde_json::to_vec_pretty(operator).expect("operator JSON");
        bytes.push(b'\n');
        std::fs::write(path, bytes).expect("operator sidecar");
    }

    fn write_corpus_pair(root: &Path, meta_name: &str, records_name: &str) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(root).expect("root");
        let meta = root.join(meta_name);
        let records = root.join(records_name);
        std::fs::write(&meta, b"captured metadata").expect("metadata");
        std::fs::write(&records, b"captured records").expect("records");
        (meta, records)
    }

    fn complete_source_corpus(root: &Path, next: u32) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(root).expect("root");
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        let mut meta_bytes = [0u8; 25];
        meta_bytes[0..8].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[16..24].copy_from_slice(&7u64.to_le_bytes());
        meta_bytes[24] = 1;
        let mut record = [0u8; 48];
        record[4..8].copy_from_slice(&next.to_le_bytes());
        record[20..24].copy_from_slice(&34u32.to_le_bytes());
        record[24..28].copy_from_slice(&33u32.to_le_bytes());
        record[28..32].copy_from_slice(&33u32.to_le_bytes());
        record[36..40].copy_from_slice(&1u32.to_le_bytes());
        record[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        record[44..48].copy_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&meta, meta_bytes).expect("complete metadata");
        std::fs::write(&records, record).expect("complete records");
        (meta, records)
    }

    fn write_execution_pair(root: &Path, version: u32) {
        let attention = if version == 1 {
            AttentionOperatorSpec::learned_absolute_v1()
        } else {
            AttentionOperatorSpec::learned_absolute_v2()
        };
        let dense = if version == 1 {
            DenseOperatorSpec::gpt2_v1()
        } else {
            DenseOperatorSpec::gpt2_v2()
        };
        write_operator(&root.join(ATTENTION_OPERATOR_BINDING_FILE), &attention);
        write_operator(&root.join(DENSE_OPERATOR_BINDING_FILE), &dense);
    }

    fn producer_guard(root: &Path) -> RecordedCorpusProducerGuard {
        std::fs::create_dir_all(root).expect("producer root");
        RecordedCorpusProducerGuard::try_acquire(root).expect("producer guard")
    }

    fn publish_compile_binding(
        guard: &RecordedCorpusProducerGuard,
        meta: &Path,
        records: &Path,
    ) -> String {
        guard
            .begin_compile_attempt()
            .expect("begin compile attempt");
        publish_binding(guard, meta, records).expect("publish compile binding")
    }

    #[test]
    fn canonical_binding_v1_bytes_and_cid_are_literal_pinned() {
        let binding = RecordedCorpusBinding {
            schema: "uor-r4-recorded-corpus-binding/1".to_owned(),
            manifest: RecordedCorpusFileBinding {
                name: "observation.manifest.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            attention_operator: RecordedCorpusFileBinding {
                name: "attention_operator.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            dense_operator: RecordedCorpusFileBinding {
                name: "dense_operator.json".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
            metadata: RecordedCorpusFileBinding {
                name: "corpus.meta".to_owned(),
                present: true,
                length: Some(1),
                blake3: Some(
                    "blake3:83d9dab06011479163c7c4d5fa735f911bac86729f56aaa115b7eed2eb66e022"
                        .to_owned(),
                ),
            },
            records: RecordedCorpusFileBinding {
                name: "corpus.records".to_owned(),
                present: true,
                length: Some(1),
                blake3: Some(
                    "blake3:b2dea48d667b2821a9bcf69eded39a2458a1d8165ca7fcac64c3557b69a7ea08"
                        .to_owned(),
                ),
            },
            hidden: RecordedCorpusFileBinding {
                name: "corpus.records.hidden".to_owned(),
                present: false,
                length: None,
                blake3: None,
            },
        };
        let expected = b"{\n  \"schema\": \"uor-r4-recorded-corpus-binding/1\",\n  \"manifest\": {\n    \"name\": \"observation.manifest.json\",\n    \"present\": false\n  },\n  \"attention_operator\": {\n    \"name\": \"attention_operator.json\",\n    \"present\": false\n  },\n  \"dense_operator\": {\n    \"name\": \"dense_operator.json\",\n    \"present\": false\n  },\n  \"metadata\": {\n    \"name\": \"corpus.meta\",\n    \"present\": true,\n    \"length\": 1,\n    \"blake3\": \"blake3:83d9dab06011479163c7c4d5fa735f911bac86729f56aaa115b7eed2eb66e022\"\n  },\n  \"records\": {\n    \"name\": \"corpus.records\",\n    \"present\": true,\n    \"length\": 1,\n    \"blake3\": \"blake3:b2dea48d667b2821a9bcf69eded39a2458a1d8165ca7fcac64c3557b69a7ea08\"\n  },\n  \"hidden\": {\n    \"name\": \"corpus.records.hidden\",\n    \"present\": false\n  }\n}\n";
        assert_eq!(
            binding.canonical_bytes().expect("canonical bytes"),
            expected
        );
        assert_eq!(
            binding.declared_digest().expect("binding CID"),
            "blake3:aea2551ba82356375580a480d338b076488354c4b6bcab59bb22c26af9bcdafc"
        );
    }

    #[test]
    fn producer_guard_is_alias_stable_nonblocking_and_reacquirable() {
        let root = unique_root("producer-guard-busy");
        std::fs::create_dir_all(&root).expect("producer root");
        let first = RecordedCorpusProducerGuard::try_acquire(&root).expect("first guard");
        let before: Vec<_> = std::fs::read_dir(&root)
            .expect("root inventory")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        let alias = root
            .parent()
            .expect("root parent")
            .join(".")
            .join(root.file_name().expect("root name"));
        let busy = RecordedCorpusProducerGuard::try_acquire(&alias)
            .expect_err("alias contender is nonblocking");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        let after: Vec<_> = std::fs::read_dir(&root)
            .expect("root inventory after BUSY")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        assert_eq!(after, before, "BUSY refusal cannot mutate corpus root");
        drop(first);
        let retry = RecordedCorpusProducerGuard::try_acquire(&alias).expect("retry after drop");
        assert_eq!(retry.root(), std::fs::canonicalize(&root).unwrap());
        drop(retry);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn producer_guard_case_aliases_contend_when_the_filesystem_equates_them() {
        let root = unique_root("producer-guard-case-alias");
        std::fs::create_dir_all(&root).expect("producer root");
        let alias = root.parent().expect("root parent").join(
            root.file_name()
                .expect("root name")
                .to_string_lossy()
                .to_uppercase(),
        );
        let Ok(alias_metadata) = std::fs::symlink_metadata(&alias) else {
            // Case-sensitive filesystems have no equivalent alias to test.
            let _ = std::fs::remove_dir_all(root);
            return;
        };
        let root_metadata = std::fs::symlink_metadata(&root).expect("root metadata");
        if !opened_file_identity_matches(&root_metadata, &alias_metadata) {
            let _ = std::fs::remove_dir_all(root);
            return;
        }

        let first = RecordedCorpusProducerGuard::try_acquire(&root).expect("first spelling");
        let busy = RecordedCorpusProducerGuard::try_acquire(&alias)
            .expect_err("equivalent final-component spelling must contend");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        assert_eq!(first.root(), std::fs::canonicalize(&root).unwrap());
        drop(first);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn producer_guard_refuses_cross_root_publication_before_mutation() {
        let owned = unique_root("producer-guard-owned");
        let other = unique_root("producer-guard-other");
        let guard = producer_guard(&owned);
        let (meta, records) = complete_source_corpus(&other, 37);
        write_execution_pair(&other, 2);
        let meta_before = std::fs::read(&meta).expect("metadata before");
        let records_before = std::fs::read(&records).expect("records before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("cross-root guard is terminal");
        assert!(error.reason.contains("cross-root"), "{error}");
        assert_eq!(std::fs::read(&meta).unwrap(), meta_before);
        assert_eq!(std::fs::read(&records).unwrap(), records_before);
        assert!(!other.join(RECORDED_CORPUS_BINDING_FILE).exists());
        let _ = std::fs::remove_dir_all(owned);
        let _ = std::fs::remove_dir_all(other);
    }

    #[cfg(unix)]
    #[test]
    fn producer_guard_rejects_symlink_coordination_without_following() {
        let root = unique_root("producer-guard-symlink");
        std::fs::create_dir_all(&root).expect("producer root");
        let canonical = std::fs::canonicalize(&root).expect("canonical root");
        let lock_path = producer_lock_path(
            canonical.parent().expect("canonical parent"),
            canonical.file_name().expect("canonical root name"),
        );
        std::fs::create_dir_all(lock_path.parent().expect("coordination parent"))
            .expect("coordination directory");
        let sentinel = root.parent().expect("root parent").join(format!(
            "{}.sentinel",
            root.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&sentinel, b"sentinel").expect("sentinel");
        std::os::unix::fs::symlink(&sentinel, &lock_path).expect("coordination symlink");
        let error = RecordedCorpusProducerGuard::try_acquire(&root)
            .expect_err("coordination symlink is terminal");
        assert!(error.reason.contains("without following"), "{error}");
        assert_eq!(
            std::fs::read(&sentinel).expect("sentinel preserved"),
            b"sentinel"
        );
        assert!(
            std::fs::symlink_metadata(&lock_path)
                .expect("symlink preserved")
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(lock_path).expect("remove test coordination symlink");
        let _ = std::fs::remove_file(sentinel);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn producer_guard_rejects_root_replaced_by_symlink_during_acquisition() {
        let root = unique_root("producer-guard-root-swap");
        let displaced = root.with_extension("displaced");
        let other = unique_root("producer-guard-root-swap-target");
        std::fs::create_dir_all(&root).expect("producer root");
        std::fs::create_dir_all(&other).expect("other root");
        std::fs::write(other.join("sentinel"), b"other").expect("other sentinel");

        let error = RecordedCorpusProducerGuard::try_acquire_with_root_hook(&root, || {
            std::fs::rename(&root, &displaced).expect("displace opened root");
            std::os::unix::fs::symlink(&other, &root).expect("replace root with symlink");
        })
        .expect_err("root replacement is terminal");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert_eq!(std::fs::read(other.join("sentinel")).unwrap(), b"other");
        assert!(
            std::fs::symlink_metadata(&root)
                .expect("replacement preserved")
                .file_type()
                .is_symlink()
        );

        std::fs::remove_file(&root).expect("remove test symlink");
        std::fs::rename(&displaced, &root).expect("restore root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(other);
    }

    #[test]
    fn producer_guard_locks_absent_logical_root_before_exact_creation() {
        let root = unique_root("producer-guard-absent");
        assert!(!root.exists());
        let mut first =
            RecordedCorpusProducerGuard::try_acquire(&root).expect("lock absent logical root");
        assert!(!root.exists(), "lock acquisition alone is failure-atomic");
        let busy = RecordedCorpusProducerGuard::try_acquire(&root)
            .expect_err("absent-root contender is nonblocking");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        assert!(!root.exists(), "BUSY cannot create the logical root");
        assert_eq!(
            first.ensure_root().expect("create root after preflight"),
            std::fs::canonicalize(&root).unwrap()
        );
        drop(first);
        let retry = RecordedCorpusProducerGuard::try_acquire(&root).expect("retry existing root");
        assert_eq!(retry.root(), std::fs::canonicalize(&root).unwrap());
        drop(retry);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn producer_guard_absent_root_creation_stays_with_retained_parent() {
        let parent = unique_root("producer-guard-parent-generation");
        let replacement = unique_root("producer-guard-parent-replacement");
        std::fs::create_dir_all(&parent).expect("original parent");
        std::fs::create_dir_all(&replacement).expect("replacement template");
        std::fs::write(replacement.join("sentinel"), b"untouched").expect("sentinel");
        let root = parent.join("corpus");
        let mut guard = RecordedCorpusProducerGuard::try_acquire(&root).expect("absent guard");
        let displaced = parent.with_extension("displaced");

        let error = guard
            .ensure_root_with_hook(|| {
                std::fs::rename(&parent, &displaced).expect("displace retained parent");
                std::fs::rename(&replacement, &parent).expect("install replacement parent");
            })
            .expect_err("replacement parent cannot receive root creation");
        assert!(
            error.reason.contains("changed")
                || error.reason.contains("cannot be inspected")
                || error.reason.contains("No such file"),
            "{error}"
        );
        assert!(
            displaced.join("corpus").is_dir(),
            "mkdirat creates only below the retained parent inode"
        );
        assert!(!parent.join("corpus").exists());
        assert_eq!(
            std::fs::read(parent.join("sentinel")).unwrap(),
            b"untouched"
        );

        drop(guard);
        std::fs::rename(&parent, &replacement).expect("restore replacement template");
        std::fs::rename(&displaced, &parent).expect("restore original parent");
        let _ = std::fs::remove_dir_all(parent);
        let _ = std::fs::remove_dir_all(replacement);
    }

    #[test]
    fn dense_generation_requires_and_roundtrips_canonical_binding() {
        let root = unique_root("dense-binding-roundtrip");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 17);
        write_execution_pair(&root, 2);

        let missing = capture(&meta, &records).expect_err("dense without commit is terminal");
        assert!(
            missing.reason.contains(RECORDED_CORPUS_BINDING_FILE),
            "{missing}"
        );
        let cid = publish_compile_binding(&guard, &meta, &records);
        let snapshot = capture(&meta, &records).expect("capture committed generation");
        assert_eq!(snapshot.binding_cid.as_deref(), Some(cid.as_str()));
        assert_eq!(
            snapshot.execution,
            RecordedCorpusExecutionIdentity {
                attention_operator: Some(AttentionOperatorSpec::learned_absolute_v2()),
                dense_operator: Some(DenseOperatorSpec::gpt2_v2()),
            }
        );
        assert_eq!(
            snapshot
                .binding
                .as_ref()
                .expect("typed binding")
                .declared_digest()
                .expect("binding digest"),
            cid
        );

        let bytes = std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE))
            .expect("canonical binding bytes");
        assert_eq!(
            parse_binding_bytes(&root.join(RECORDED_CORPUS_BINDING_FILE), &bytes)
                .expect("canonical binding")
                .canonical_bytes()
                .expect("canonical bytes"),
            bytes
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_shape_rejects_invalid_fixed_slots_before_recovery() {
        let root = unique_root("binding-shape");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 19);
        let _ = publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);
        let canonical = std::fs::read(&stable).expect("canonical binding");

        let mut invalid: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        invalid["records"]["blake3"] = serde_json::json!("blake3:NOT-A-DIGEST");
        let mut invalid_bytes = serde_json::to_vec_pretty(&invalid).unwrap();
        invalid_bytes.push(b'\n');
        let error = parse_binding_bytes(&stable, &invalid_bytes)
            .expect_err("digest syntax is structural, not recovery-time");
        assert!(error.reason.contains("invalid typed"), "{error}");

        let mut invalid: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        invalid["hidden"]["name"] = serde_json::json!("wrong.hidden");
        let mut invalid_bytes = serde_json::to_vec_pretty(&invalid).unwrap();
        invalid_bytes.push(b'\n');
        let error = parse_binding_bytes(&stable, &invalid_bytes)
            .expect_err("hidden slot name is role-specific");
        assert!(error.reason.contains("expected exact member"), "{error}");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_publication_rechecks_records_and_hidden_as_one_generation() {
        let root = unique_root("binding-joint-generation");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 23);
        let hidden = hidden_path(&records);
        std::fs::write(&hidden, b"hidden generation B").expect("hidden B");
        guard.begin_compile_attempt().expect("compile attempt");
        let old_records = root.join("records-generation-a");
        let error = publish_binding_with_hooks(
            &guard,
            &meta,
            &records,
            || {
                std::fs::rename(&records, &old_records).expect("retain records A");
                std::fs::write(&records, b"records generation B").expect("publish records B");
            },
            || {},
        )
        .expect_err("records A plus hidden B cannot commit");
        assert!(error.reason.contains("changed generation"), "{error}");
        assert!(!root.join(RECORDED_CORPUS_BINDING_FILE).exists());
        assert!(binding_temporaries(&root).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn binding_publication_commit_cannot_be_redirected_after_last_check() {
        let root = unique_root("binding-root-generation");
        let wrong = unique_root("binding-wrong-root");
        std::fs::create_dir_all(&wrong).expect("wrong root");
        std::fs::write(wrong.join("sentinel"), b"untouched").expect("wrong sentinel");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 29);
        guard.begin_compile_attempt().expect("compile attempt");
        let displaced = root.with_extension("displaced");
        let error = publish_binding_with_hooks(
            &guard,
            &meta,
            &records,
            || {},
            || {
                std::fs::rename(&root, &displaced).expect("displace guarded root");
                std::os::unix::fs::symlink(&wrong, &root).expect("redirect root path");
            },
        )
        .expect_err("replacement root cannot receive a commit");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert!(!root.join(RECORDED_CORPUS_BINDING_FILE).exists());
        assert!(
            displaced.join(RECORDED_CORPUS_BINDING_FILE).exists(),
            "retained-dirfd commit stays with the guarded inode"
        );
        assert_eq!(std::fs::read(wrong.join("sentinel")).unwrap(), b"untouched");
        assert_eq!(std::fs::read_dir(&wrong).unwrap().count(), 1);
        std::fs::remove_file(&root).expect("remove redirect symlink");
        std::fs::rename(&displaced, &root).expect("restore guarded root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(wrong);
    }

    #[test]
    fn idempotent_binding_publication_rechecks_stable_bytes_after_hook() {
        let root = unique_root("binding-idempotent-stable-recheck");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 31);
        let _ = publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);

        let error = publish_binding_with_hooks(
            &guard,
            &meta,
            &records,
            || {},
            || std::fs::write(&stable, b"conflicting stable binding").expect("replace stable"),
        )
        .expect_err("idempotent publication cannot bless replaced stable bytes");
        assert!(
            error.reason.contains("wrong type or length")
                || error.reason.contains("changed content"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(&stable).unwrap(),
            b"conflicting stable binding"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn bounded_binding_control_refuses_oversize_residue_without_mutation() {
        let root = unique_root("binding-control-cap");
        let guard = producer_guard(&root);
        let residue = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.91.7.writing"));
        let bytes = vec![b'x'; BINDING_CONTROL_MAX_BYTES as usize + 1];
        std::fs::write(&residue, &bytes).expect("oversize residue");
        let error = guard
            .preflight_publication_namespace()
            .expect_err("oversize control residue is terminal");
        assert!(error.reason.contains("schema cap"), "{error}");
        assert_eq!(std::fs::read(&residue).unwrap(), bytes);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reserved_residue_namespaces_have_fixed_aggregate_ceilings() {
        let binding_root = unique_root("binding-residue-count-cap");
        let binding_guard = producer_guard(&binding_root);
        let mut binding_paths = Vec::new();
        for sequence in 0..=RESERVED_RESIDUE_MAX_ENTRIES {
            let path = binding_root.join(format!(
                ".{RECORDED_CORPUS_BINDING_FILE}.901.{sequence}.writing"
            ));
            std::fs::write(&path, b"partial binding staging").expect("binding residue");
            binding_paths.push(path);
        }
        let error = binding_guard
            .preflight_publication_namespace()
            .expect_err("binding residue count is bounded");
        assert!(error.reason.contains("binding residue ceiling"), "{error}");
        for path in &binding_paths {
            assert_eq!(std::fs::read(path).unwrap(), b"partial binding staging");
        }

        let attempt_root = unique_root("attempt-residue-count-cap");
        let attempt_guard = producer_guard(&attempt_root);
        let mut attempt_paths = Vec::new();
        for sequence in 0..=RESERVED_RESIDUE_MAX_ENTRIES {
            let path = attempt_root.join(format!(
                ".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.902.{sequence}.writing"
            ));
            std::fs::write(&path, b"partial attempt staging").expect("attempt residue");
            attempt_paths.push(path);
        }
        let error = attempt_guard
            .begin_compile_attempt()
            .expect_err("compile-attempt residue count is bounded");
        assert!(
            error.reason.contains("compile-attempt residue ceiling"),
            "{error}"
        );
        assert!(
            !attempt_root
                .join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists()
        );
        for path in &attempt_paths {
            assert_eq!(std::fs::read(path).unwrap(), b"partial attempt staging");
        }

        let planned_root = unique_root("planned-residue-count-cap");
        let planned_guard = producer_guard(&planned_root);
        let mut planned_paths = Vec::new();
        for sequence in 0..=RESERVED_RESIDUE_MAX_ENTRIES {
            let path = planned_root.join(format!(
                "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--903.{sequence}.writing",
                PlannedOutputMember::Records.stable_name()
            ));
            std::fs::write(&path, b"partial planned staging").expect("planned residue");
            planned_paths.push(path);
        }
        let error = planned_guard
            .preflight_planned_output_scope(&[PlannedOutputMember::Records])
            .expect_err("planned-output residue count is bounded");
        assert!(
            error.reason.contains("planned-output residue ceiling"),
            "{error}"
        );
        for path in &planned_paths {
            assert_eq!(std::fs::read(path).unwrap(), b"partial planned staging");
        }

        let reader_root = unique_root("binding-reader-first-temp");
        std::fs::create_dir_all(&reader_root).expect("reader root");
        let temporary = reader_root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.904.1.tmp"));
        let file = std::fs::File::create(&temporary).expect("reader temporary");
        file.set_len(1u64 << 40).expect("sparse reader temporary");
        let error = require_no_binding_temporaries(&reader_root)
            .expect_err("reader rejects the first temporary without reading its body");
        assert!(
            error.reason.contains("writer recovery is required"),
            "{error}"
        );
        assert_eq!(std::fs::metadata(&temporary).unwrap().len(), 1u64 << 40);

        let _ = std::fs::remove_dir_all(binding_root);
        let _ = std::fs::remove_dir_all(attempt_root);
        let _ = std::fs::remove_dir_all(planned_root);
        let _ = std::fs::remove_dir_all(reader_root);
    }

    #[test]
    fn final_control_rechecks_reject_oversize_post_capture_replacements() {
        for (label, name, replacement_bytes) in [
            (
                "manifest",
                observation::MANIFEST_FILE,
                PROVENANCE_CONTROL_MAX_BYTES + 1,
            ),
            (
                "attention",
                ATTENTION_OPERATOR_BINDING_FILE,
                PROVENANCE_CONTROL_MAX_BYTES + 1,
            ),
            (
                "dense",
                DENSE_OPERATOR_BINDING_FILE,
                PROVENANCE_CONTROL_MAX_BYTES + 1,
            ),
            ("metadata", "corpus.meta", METADATA_CONTROL_MAX_BYTES + 1),
            (
                "binding",
                RECORDED_CORPUS_BINDING_FILE,
                BINDING_CONTROL_MAX_BYTES + 1,
            ),
            (
                "marker",
                RECORDED_CORPUS_COMPILE_ATTEMPT_FILE,
                COMPILE_ATTEMPT_CONTROL_MAX_BYTES + 1,
            ),
        ] {
            let root = unique_root(&format!("oversize-final-{label}"));
            let guard = producer_guard(&root);
            let (meta, records) = complete_source_corpus(&root, 73);
            write_execution_pair(&root, 2);
            let _ = publish_compile_binding(&guard, &meta, &records);
            let snapshot = open_stream(&meta, &records).expect("open exact generation");
            let replacement = vec![b'x'; replacement_bytes as usize];
            std::fs::write(root.join(name), replacement).expect("oversize replacement");

            let error = snapshot
                .verify_generation()
                .expect_err("oversize control replacement is terminal");
            assert!(
                error.reason.contains("appeared")
                    || error.reason.contains("length")
                    || error.reason.contains("generation"),
                "{label}: {error}"
            );
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[cfg(unix)]
    #[test]
    fn observation_role_rejects_non_utf8_planned_output_prefix() {
        use std::os::unix::ffi::OsStringExt;

        let root = unique_root("observation-nonutf-planned");
        let guard = producer_guard(&root);
        let mut name = PLANNED_OUTPUT_RESERVED_PREFIX.as_bytes().to_vec();
        name.extend_from_slice(b"corpus.records--12.3.writing");
        name.push(0xff);
        let residue = root.join(std::ffi::OsString::from_vec(name));
        if let Err(error) = std::fs::write(&residue, b"partial") {
            // APFS may reject non-UTF-8 directory entries at the VFS boundary;
            // Linux permits them and exercises the classifier below.
            if error.raw_os_error() == Some(92) {
                let _ = std::fs::remove_dir_all(root);
                return;
            }
            panic!("non-UTF-8 planned residue: {error}");
        }
        let error = guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("observation role owns no planned-output residue");
        assert!(error.reason.contains("non-UTF-8"), "{error}");
        assert_eq!(std::fs::read(&residue).unwrap(), b"partial");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn role_preflight_and_publication_reject_cross_role_generations() {
        let compile_root = unique_root("compile-role-binding");
        let compile_guard = producer_guard(&compile_root);
        let (compile_meta, compile_records) = complete_source_corpus(&compile_root, 41);
        publish_compile_binding(&compile_guard, &compile_meta, &compile_records);
        compile_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Compile)
            .expect("matching compile role");
        let error = compile_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("observation cannot enter a compile root");
        assert!(error.reason.contains("compile-role"), "{error}");
        let compile_binding =
            std::fs::read(compile_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let (state, merged) =
            write_corpus_pair(&compile_root, observation::STATE_FILE, "merged.bin");
        let error = publish_binding(&compile_guard, &state, &merged)
            .expect_err("public publisher cannot replace a compile binding with observation role");
        assert!(
            error.reason.contains("cross-role") || error.reason.contains("compile-role"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(compile_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            compile_binding
        );

        let observation_root = unique_root("observation-role-binding");
        let observation_guard = producer_guard(&observation_root);
        let (state, merged) =
            write_corpus_pair(&observation_root, observation::STATE_FILE, "merged.bin");
        publish_binding(&observation_guard, &state, &merged).expect("observation-role binding");
        observation_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect("matching observation role");
        let error = observation_guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Compile)
            .expect_err("compile cannot enter an observation root");
        assert!(error.reason.contains("observation-role"), "{error}");
        let observation_binding =
            std::fs::read(observation_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let marker = RecordedCorpusCompileAttempt::compile()
            .canonical_bytes()
            .expect("marker bytes");
        std::fs::write(
            observation_root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE),
            marker,
        )
        .expect("inject wrong-role marker");
        let error = capture(&state, &merged)
            .expect_err("an observation binding plus compile marker is not one generation");
        assert!(
            error.reason.contains("requires corpus.meta/corpus.records"),
            "{error}"
        );
        std::fs::remove_file(observation_root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE))
            .expect("remove injected marker");
        let (meta, records) = complete_source_corpus(&observation_root, 43);
        let error = publish_binding(&observation_guard, &meta, &records)
            .expect_err("public publisher cannot replace an observation binding with compile role");
        assert!(
            error.reason.contains("cross-role") || error.reason.contains("observation-role"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(observation_root.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            observation_binding
        );

        let _ = std::fs::remove_dir_all(compile_root);
        let _ = std::fs::remove_dir_all(observation_root);
    }

    #[test]
    fn tokenizer_payload_is_shared_between_compile_and_observation_roles() {
        let root = unique_root("shared-tokenizer-role-inventory");
        let guard = producer_guard(&root);
        std::fs::write(root.join("tokenizer.bin"), b"shared tokenizer payload")
            .expect("tokenizer payload");

        guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Compile)
            .expect("compile role accepts its tokenizer payload");
        guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect("observation resume accepts its tokenizer payload");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn compile_attempt_marker_bytes_digest_and_lifecycle_are_pinned() {
        let expected = b"{\n  \"schema\": \"uor-r4-recorded-corpus-compile-attempt/1\",\n  \"role\": \"compile\"\n}\n";
        let canonical = RecordedCorpusCompileAttempt::compile()
            .canonical_bytes()
            .expect("canonical marker");
        assert_eq!(canonical, expected);
        assert_eq!(
            format!("blake3:{}", blake3::hash(&canonical).to_hex()),
            "blake3:06c2257b2d374860717cac12285e7ba0cf9a414f88d89596d5380168e1e0ac52"
        );

        let root = unique_root("compile-attempt-lifecycle");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 53);
        guard.begin_compile_attempt().expect("begin marker");
        assert_eq!(
            std::fs::read(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)).unwrap(),
            expected
        );
        let error = capture(&meta, &records).expect_err("attempt without commit is terminal");
        assert!(
            error.reason.contains("unfinished compile attempt"),
            "{error}"
        );
        let cid = publish_binding(&guard, &meta, &records).expect("publish under marker");
        assert_eq!(
            capture(&meta, &records)
                .expect("committed generation remains readable before cleanup")
                .binding_cid
                .as_deref(),
            Some(cid.as_str())
        );
        guard.finish_compile_attempt().expect("finish marker");
        assert!(!root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());
        assert_eq!(
            capture(&meta, &records)
                .expect("finished generation")
                .binding_cid,
            Some(cid)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn compile_attempt_begin_commit_cannot_be_redirected_after_last_check() {
        let root = unique_root("compile-attempt-begin-root-generation");
        let wrong = unique_root("compile-attempt-begin-wrong-root");
        std::fs::create_dir_all(&wrong).expect("wrong root");
        std::fs::write(wrong.join("sentinel"), b"untouched").expect("wrong sentinel");
        let guard = producer_guard(&root);
        let displaced = root.with_extension("displaced");

        let error = publish_compile_attempt_marker_with_hook(&guard, || {
            std::fs::rename(&root, &displaced).expect("displace guarded root");
            std::os::unix::fs::symlink(&wrong, &root).expect("redirect root path");
        })
        .expect_err("replacement root cannot receive the attempt marker");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert!(
            displaced
                .join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists(),
            "retained-dirfd marker commit stays with the guarded inode"
        );
        assert_eq!(std::fs::read(wrong.join("sentinel")).unwrap(), b"untouched");
        assert_eq!(std::fs::read_dir(&wrong).unwrap().count(), 1);

        std::fs::remove_file(&root).expect("remove redirect symlink");
        std::fs::rename(&displaced, &root).expect("restore guarded root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(wrong);
    }

    #[cfg(unix)]
    #[test]
    fn compile_attempt_finish_commit_cannot_be_redirected_after_last_check() {
        let root = unique_root("compile-attempt-finish-root-generation");
        let wrong = unique_root("compile-attempt-finish-wrong-root");
        std::fs::create_dir_all(&wrong).expect("wrong root");
        std::fs::write(wrong.join("sentinel"), b"untouched").expect("wrong sentinel");
        let guard = producer_guard(&root);
        guard.begin_compile_attempt().expect("begin attempt");
        let displaced = root.with_extension("displaced");

        let error = finish_compile_attempt_marker_with_hook(&guard, || {
            std::fs::rename(&root, &displaced).expect("displace guarded root");
            std::os::unix::fs::symlink(&wrong, &root).expect("redirect root path");
        })
        .expect_err("replacement root cannot receive marker cleanup");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert!(
            !displaced
                .join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists(),
            "retained-dirfd marker cleanup stays with the guarded inode"
        );
        assert_eq!(std::fs::read(wrong.join("sentinel")).unwrap(), b"untouched");
        assert_eq!(std::fs::read_dir(&wrong).unwrap().count(), 1);

        std::fs::remove_file(&root).expect("remove redirect symlink");
        std::fs::rename(&displaced, &root).expect("restore guarded root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(wrong);
    }

    #[test]
    fn finish_compile_attempt_requires_binding_for_exact_current_members() {
        let root = unique_root("finish-exact-current");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 31);
        let _ = publish_compile_binding(&guard, &meta, &records);
        std::fs::write(&records, b"different current records").expect("advance records");
        let error = guard
            .finish_compile_attempt()
            .expect_err("stale stable binding cannot finish current attempt");
        assert!(error.reason.contains("exact planned generation"), "{error}");
        assert!(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_update_requires_marker_for_same_role_generation_transition() {
        let interrupted = unique_root("source-update-interrupted");
        let guard = producer_guard(&interrupted);
        let (meta, records) = complete_source_corpus(&interrupted, 53);
        let old_cid = publish_compile_binding(&guard, &meta, &records);
        guard.finish_compile_attempt().expect("finish generation A");
        guard.begin_compile_attempt().expect("begin generation B");
        let _ = complete_source_corpus(&interrupted, 71);
        let state = preflight_source_update_publication(&guard, &meta, &records)
            .expect("active marker permits canonical last-good binding A beside resumed members B");
        assert!(state.attempt_active);
        assert!(!state.recoverable_binding);
        let new_cid = publish_binding(&guard, &meta, &records).expect("commit generation B");
        assert_ne!(new_cid, old_cid);
        guard.finish_compile_attempt().expect("finish generation B");
        assert_eq!(capture(&meta, &records).unwrap().binding_cid, Some(new_cid));

        let unowned = unique_root("source-update-without-marker");
        let guard = producer_guard(&unowned);
        let (meta, records) = complete_source_corpus(&unowned, 53);
        publish_compile_binding(&guard, &meta, &records);
        guard.finish_compile_attempt().expect("finish generation A");
        let _ = complete_source_corpus(&unowned, 71);
        let before_binding = std::fs::read(unowned.join(RECORDED_CORPUS_BINDING_FILE)).unwrap();
        let before_meta = std::fs::read(&meta).unwrap();
        let before_records = std::fs::read(&records).unwrap();
        let error = preflight_source_update_publication(&guard, &meta, &records)
            .expect_err("member drift without an active attempt marker is terminal");
        assert!(error.reason.contains("does not commit"), "{error}");
        assert_eq!(
            std::fs::read(unowned.join(RECORDED_CORPUS_BINDING_FILE)).unwrap(),
            before_binding
        );
        assert_eq!(std::fs::read(&meta).unwrap(), before_meta);
        assert_eq!(std::fs::read(&records).unwrap(), before_records);

        let _ = std::fs::remove_dir_all(interrupted);
        let _ = std::fs::remove_dir_all(unowned);
    }

    #[test]
    fn active_source_update_never_authorizes_mixed_provenance() {
        let root = unique_root("source-update-mixed-provenance");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 53);
        publish_compile_binding(&guard, &meta, &records);
        guard.finish_compile_attempt().expect("finish generation A");
        guard.begin_compile_attempt().expect("begin generation B");
        let _ = complete_source_corpus(&root, 71);

        let mut attention = serde_json::to_vec_pretty(&AttentionOperatorSpec::standard_v2())
            .expect("serialize injected provenance");
        attention.push(b'\n');
        std::fs::write(root.join(ATTENTION_OPERATOR_BINDING_FILE), attention)
            .expect("inject different current provenance");
        let binding_before =
            std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE)).expect("binding before");
        let records_before = std::fs::read(&records).expect("records before");

        let error = preflight_source_update_publication(&guard, &meta, &records)
            .expect_err("role-only attempt marker cannot authorize mixed provenance");
        assert!(error.reason.contains("does not match"), "{error}");
        assert_eq!(
            std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE)).expect("binding after"),
            binding_before
        );
        assert_eq!(
            std::fs::read(&records).expect("records after"),
            records_before
        );
        assert!(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn compile_attempt_crash_residue_is_recovered_but_observation_refuses_it() {
        let root = unique_root("compile-attempt-writing");
        let guard = producer_guard(&root);
        let writing = root.join(format!(
            ".{RECORDED_CORPUS_COMPILE_ATTEMPT_FILE}.999.1.writing"
        ));
        std::fs::write(&writing, b"partial").expect("partial marker staging");
        let error = guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("observation rejects compile staging evidence");
        assert!(error.reason.contains("compile-attempt"), "{error}");
        assert_eq!(std::fs::read(&writing).unwrap(), b"partial");
        guard
            .begin_compile_attempt()
            .expect("compile retry reclaims staging");
        assert!(!writing.exists());
        assert!(root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE).exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn role_preflight_rejects_foreign_prefix_and_temporary_without_mutation() {
        for foreign in [
            "shard-00.bin",
            "shard-not-a-number.bin",
            observation::RAW_COMMITTED_FILE,
            "tokenizer_adapter.json",
            ".source_compile_preflight.json.800.1.tmp",
            ".compiled_bundle_completion.json.801.2.tmp",
        ] {
            let root = unique_root(&format!("foreign-role-{}", foreign.replace('.', "-")));
            let guard = producer_guard(&root);
            let path = root.join(foreign);
            std::fs::write(&path, b"foreign crash prefix").expect("foreign prefix");
            let before = std::fs::read(&path).unwrap();
            let role =
                if foreign.starts_with("shard-") || foreign == observation::RAW_COMMITTED_FILE {
                    RecordedCorpusRole::Compile
                } else {
                    RecordedCorpusRole::Observation
                };
            let error = guard
                .preflight_publication_namespace_for(role)
                .expect_err("foreign role prefix is terminal");
            assert!(error.reason.contains("foreign"), "{error}");
            assert_eq!(std::fs::read(&path).unwrap(), before);
            let _ = std::fs::remove_dir_all(root);
        }

        let root = unique_root("cross-role-binding-temp");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 47);
        publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);
        let temporary = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.777.1.tmp"));
        std::fs::rename(&stable, &temporary).expect("binding crash temp");
        let before = std::fs::read(&temporary).unwrap();
        let error = guard
            .preflight_publication_namespace_for(RecordedCorpusRole::Observation)
            .expect_err("cross-role canonical temp is terminal");
        assert!(error.reason.contains("cross-role") || error.reason.contains("compile-role"));
        assert_eq!(std::fs::read(&temporary).unwrap(), before);
        assert!(!stable.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn shared_planned_output_scope_is_global_and_recovery_is_owner_exact() {
        let root = unique_root("planned-output-scope");
        let guard = producer_guard(&root);
        let records_residue = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.1.writing",
            PlannedOutputMember::Records.stable_name()
        ));
        std::fs::write(&records_residue, b"partial records").expect("records residue");
        guard
            .preflight_planned_output_scope(&[PlannedOutputMember::Records])
            .expect("declared owner accepts its regular residue");
        assert!(
            guard
                .has_planned_output_residue(PlannedOutputMember::Records)
                .unwrap()
        );
        let error = guard
            .preflight_planned_output_scope(&[])
            .expect_err("partial-plan writer owns no residue");
        assert!(error.reason.contains("foreign member"), "{error}");
        assert_eq!(std::fs::read(&records_residue).unwrap(), b"partial records");
        guard
            .reclaim_planned_output_residues(PlannedOutputMember::Records)
            .expect("owner reclaims exact regular residue");
        assert!(!records_residue.exists());

        let artifact_residue = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.2.writing",
            PlannedOutputMember::Artifact.stable_name()
        ));
        std::fs::write(&artifact_residue, b"foreign artifact").expect("artifact residue");
        let error = guard
            .preflight_planned_output_scope(&[
                PlannedOutputMember::Records,
                PlannedOutputMember::Hidden,
                PlannedOutputMember::Metadata,
            ])
            .expect_err("subsample cannot ignore another plan's residue");
        assert!(error.reason.contains("tless_artifacts.bin"), "{error}");
        assert_eq!(
            std::fs::read(&artifact_residue).unwrap(),
            b"foreign artifact"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn planned_output_recovery_cannot_be_redirected_after_last_check() {
        let root = unique_root("planned-output-recovery-root-generation");
        let wrong = unique_root("planned-output-recovery-wrong-root");
        std::fs::create_dir_all(&wrong).expect("wrong root");
        std::fs::write(wrong.join("sentinel"), b"untouched").expect("wrong sentinel");
        let guard = producer_guard(&root);
        let residue_name = format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.77.writing",
            PlannedOutputMember::Records.stable_name()
        );
        std::fs::write(root.join(&residue_name), b"partial records").expect("residue");
        let displaced = root.with_extension("displaced");

        let error = guard
            .reclaim_planned_output_residues_with_hook(PlannedOutputMember::Records, || {
                std::fs::rename(&root, &displaced).expect("displace guarded root");
                std::os::unix::fs::symlink(&wrong, &root).expect("redirect root path");
            })
            .expect_err("replacement root cannot receive residue cleanup");
        assert!(
            error.reason.contains("changed identity")
                || error.reason.contains("real non-symlink directory"),
            "{error}"
        );
        assert!(
            !displaced.join(&residue_name).exists(),
            "retained-dirfd cleanup unlinks only from the guarded inode"
        );
        assert_eq!(std::fs::read(wrong.join("sentinel")).unwrap(), b"untouched");
        assert_eq!(std::fs::read_dir(&wrong).unwrap().count(), 1);

        std::fs::remove_file(&root).expect("remove redirect symlink");
        std::fs::rename(&displaced, &root).expect("restore guarded root");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(wrong);
    }

    #[cfg(unix)]
    #[test]
    fn shared_planned_output_scope_rejects_symlink_and_nonregular_entries() {
        let root = unique_root("planned-output-special");
        let guard = producer_guard(&root);
        let symlink = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.3.writing",
            PlannedOutputMember::Hidden.stable_name()
        ));
        std::os::unix::fs::symlink("missing-target", &symlink).expect("reserved symlink");
        let error = guard
            .preflight_planned_output_scope(&PlannedOutputMember::ALL)
            .expect_err("reserved symlink is terminal");
        assert!(error.reason.contains("without following links"), "{error}");
        assert!(
            std::fs::symlink_metadata(&symlink)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(&symlink).unwrap();

        let directory = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.4.writing",
            PlannedOutputMember::Metadata.stable_name()
        ));
        std::fs::create_dir(&directory).expect("reserved directory");
        let error = guard
            .preflight_planned_output_scope(&PlannedOutputMember::ALL)
            .expect_err("reserved directory is terminal");
        assert!(
            error.reason.contains("without following links") || error.reason.contains("regular")
        );
        assert!(
            std::fs::symlink_metadata(&directory)
                .unwrap()
                .file_type()
                .is_dir()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_rejects_stronger_pair_corpus_aba_even_when_pair_and_marker_restore() {
        let root = unique_root("binding-aba");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 11);
        write_execution_pair(&root, 1);
        publish_compile_binding(&guard, &meta, &records);
        let attention_a =
            std::fs::read(root.join(ATTENTION_OPERATOR_BINDING_FILE)).expect("attention A");
        let dense_a = std::fs::read(root.join(DENSE_OPERATOR_BINDING_FILE)).expect("dense A");
        let binding_a = std::fs::read(root.join(RECORDED_CORPUS_BINDING_FILE)).expect("binding A");

        let error = capture_with_hooks(
            &meta,
            &records,
            || {
                write_execution_pair(&root, 2);
                let _ = complete_source_corpus(&root, 29);
                publish_binding(&guard, &meta, &records).expect("publish B");
            },
            || {
                std::fs::write(root.join(ATTENTION_OPERATOR_BINDING_FILE), &attention_a)
                    .expect("restore attention A");
                std::fs::write(root.join(DENSE_OPERATOR_BINDING_FILE), &dense_a)
                    .expect("restore dense A");
                std::fs::write(root.join(RECORDED_CORPUS_BINDING_FILE), &binding_a)
                    .expect("restore binding A");
            },
        )
        .expect_err("binding A cannot authorize corpus B");
        assert!(
            error.reason.contains("does not match")
                && (error.reason.contains("metadata") || error.reason.contains("records")),
            "unexpected error: {error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn markerless_attention_only_remains_explicit_compatibility() {
        let root = unique_root("markerless-attention-only");
        let (meta, records) = complete_source_corpus(&root, 5);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let snapshot = capture(&meta, &records).expect("attention-only compatibility");
        assert!(snapshot.binding.is_none());
        assert!(snapshot.binding_cid.is_none());
        assert_eq!(snapshot.execution.dense_operator, None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_temporary_recovery_is_exact_and_conflicts_are_preserved() {
        let root = unique_root("binding-temp-recovery");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 7);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let stable = root.join(RECORDED_CORPUS_BINDING_FILE);
        let expected = std::fs::read(&stable).expect("stable binding");
        let recoverable = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.1.tmp"));
        std::fs::rename(&stable, &recoverable).expect("simulate pre-publish crash");
        let recovered_cid =
            publish_binding(&guard, &meta, &records).expect("recover exact temporary");
        assert_eq!(
            recovered_cid,
            format!("blake3:{}", blake3::hash(&expected).to_hex())
        );
        assert!(!recoverable.exists());

        let conflicting = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.2.tmp"));
        let mut conflict = expected.clone();
        let schema = RECORDED_CORPUS_BINDING_SCHEMA.as_bytes();
        let offset = conflict
            .windows(schema.len())
            .position(|window| window == schema)
            .expect("schema bytes");
        conflict[offset] = b'X';
        std::fs::write(&conflicting, &conflict).expect("conflicting temporary");
        let before = std::fs::read(&conflicting).expect("conflict before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("malformed temp is terminal");
        assert!(error.reason.contains("binding"), "{error}");
        assert_eq!(
            std::fs::read(&conflicting).expect("conflict preserved"),
            before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binding_writing_crash_residue_is_ignored_and_reclaimed_by_exact_retry() {
        let root = unique_root("binding-writing-recovery");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 13);
        write_execution_pair(&root, 2);
        let zero = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.10.writing"));
        let partial = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.11.writing"));
        std::fs::write(&zero, b"").expect("zero-byte staging residue");
        std::fs::write(&partial, b"partial canonical JSON").expect("partial staging residue");

        publish_compile_binding(&guard, &meta, &records);
        assert!(!zero.exists());
        assert!(!partial.exists());

        let ignored = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.12.writing"));
        std::fs::write(&ignored, b"unfinished").expect("reader-ignored staging residue");
        capture(&meta, &records).expect("reader ignores non-authoritative staging residue");
        assert_eq!(
            std::fs::read(&ignored).expect("reader preserves staging residue"),
            b"unfinished"
        );
        publish_binding(&guard, &meta, &records).expect("publisher reclaims ignored residue");
        assert!(!ignored.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unrecognized_binding_publication_residue_is_terminal_and_preserved() {
        let root = unique_root("binding-unrecognized-residue");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 19);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let unrecognized = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.unowned.writing"));
        std::fs::write(&unrecognized, b"unowned").expect("unrecognized residue");
        let before = std::fs::read(&unrecognized).expect("residue before");
        let error =
            publish_binding(&guard, &meta, &records).expect_err("unrecognized residue is terminal");
        assert!(error.reason.contains("reserved"), "{error}");
        assert_eq!(
            std::fs::read(&unrecognized).expect("residue preserved"),
            before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn binding_namespace_ignores_unrelated_non_utf8_but_rejects_reserved_non_utf8() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let root = unique_root("binding-non-utf8");
        let (meta, records) = complete_source_corpus(&root, 23);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let unrelated = root.join(OsString::from_vec(vec![b'u', 0xff, b'x']));
        std::fs::write(&unrelated, b"unrelated").expect("unrelated non-UTF-8 entry");
        capture(&meta, &records).expect("unrelated non-UTF-8 entry is outside namespace");

        let mut reserved = format!(".{RECORDED_CORPUS_BINDING_FILE}.").into_bytes();
        reserved.extend_from_slice(&[0xff, b'.', b't', b'm', b'p']);
        let reserved = root.join(OsString::from_vec(reserved));
        std::fs::write(&reserved, b"reserved").expect("reserved non-UTF-8 entry");
        let error = capture(&meta, &records).expect_err("reserved non-UTF-8 entry is terminal");
        assert!(error.reason.contains("non-UTF-8"), "{error}");
        assert_eq!(
            std::fs::read(&reserved).expect("reserved entry preserved"),
            b"reserved"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn publisher_rejects_symlink_binding_writing_residue_without_mutation() {
        let root = unique_root("binding-writing-symlink");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 31);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        let writing = root.join(format!(".{RECORDED_CORPUS_BINDING_FILE}.999.20.writing"));
        std::os::unix::fs::symlink("missing-target", &writing).expect("staging symlink");
        capture(&meta, &records).expect("reader ignores non-authoritative staging symlink");
        let error = publish_binding(&guard, &meta, &records)
            .expect_err("publisher rejects staging symlink");
        assert!(
            error.reason.contains("not a regular file")
                || error.reason.contains("without following links"),
            "{error}"
        );
        assert!(
            std::fs::symlink_metadata(&writing)
                .expect("symlink preserved")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read_link(&writing).expect("symlink target preserved"),
            PathBuf::from("missing-target")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lower_hidden_path_changes_only_the_filename() {
        let records = Path::new("/tmp/parent_recs.bin/corpus_recs.bin");
        assert_eq!(
            hidden_path(records),
            PathBuf::from("/tmp/parent_recs.bin/corpus_hidden.bin")
        );
    }

    #[test]
    fn composite_capture_rejects_execution_swap_between_provenance_and_corpus() {
        let root = unique_root("inter-phase-swap");
        std::fs::create_dir_all(&root).expect("root");
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        std::fs::write(&meta, b"corpus A metadata").expect("metadata A");
        std::fs::write(&records, b"corpus A records").expect("records A");
        let attention = root.join(ATTENTION_OPERATOR_BINDING_FILE);
        let dense = root.join(DENSE_OPERATOR_BINDING_FILE);
        write_operator(&attention, &AttentionOperatorSpec::learned_absolute_v1());
        write_operator(&dense, &DenseOperatorSpec::gpt2_v1());

        let error = capture_with_hooks(
            &meta,
            &records,
            || {
                write_operator(&attention, &AttentionOperatorSpec::learned_absolute_v2());
                write_operator(&dense, &DenseOperatorSpec::gpt2_v2());
                std::fs::write(&meta, b"corpus B metadata").expect("metadata B");
                std::fs::write(&records, b"corpus B records").expect("records B");
            },
            || {},
        )
        .expect_err("provenance A cannot label corpus B");
        assert!(
            error.reason.contains("changed after capture")
                || error
                    .reason
                    .contains("changed identity, type, or length after capture"),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn exact_pair_reconciliation_treats_absence_and_version_as_identity() {
        let recorded = RecordedCorpusExecutionIdentity {
            attention_operator: Some(AttentionOperatorSpec::learned_absolute_v2()),
            dense_operator: Some(DenseOperatorSpec::gpt2_v2()),
        };
        assert!(
            require_execution_identity(
                &recorded,
                Some(&AttentionOperatorSpec::learned_absolute_v2()),
                Some(&DenseOperatorSpec::gpt2_v2()),
                "test compile",
            )
            .is_ok()
        );
        let absent_dense = require_execution_identity(
            &recorded,
            Some(&AttentionOperatorSpec::learned_absolute_v2()),
            None,
            "test compile",
        )
        .expect_err("dense absence cannot relabel v2");
        assert!(absent_dense.reason.contains("dense execution identity"));
        let wrong_era = require_execution_identity(
            &recorded,
            Some(&AttentionOperatorSpec::learned_absolute_v1()),
            Some(&DenseOperatorSpec::gpt2_v1()),
            "test compile",
        )
        .expect_err("v1 cannot relabel v2");
        assert!(wrong_era.reason.contains("attention execution identity"));
    }

    #[test]
    fn recursive_duplicate_keys_are_terminal_in_both_sidecars_and_manifest() {
        let attention_v2 = AttentionOperatorSpec::learned_absolute_v2();
        let dense_v2 = DenseOperatorSpec::gpt2_v2();

        let attention_root = unique_root("duplicate-attention-sidecar");
        let (meta, records) = write_corpus_pair(&attention_root, "corpus.meta", "corpus.records");
        let attention_json = serde_json::to_string(&attention_v2).expect("attention JSON");
        let duplicate_attention =
            attention_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_attention, attention_json);
        std::fs::write(
            attention_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            duplicate_attention,
        )
        .expect("duplicate attention sidecar");
        write_operator(&attention_root.join(DENSE_OPERATOR_BINDING_FILE), &dense_v2);
        let error = capture(&meta, &records).expect_err("duplicate attention key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let dense_root = unique_root("duplicate-dense-sidecar");
        let (meta, records) = write_corpus_pair(&dense_root, "corpus.meta", "corpus.records");
        write_operator(
            &dense_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &attention_v2,
        );
        let dense_json = serde_json::to_string(&dense_v2).expect("dense JSON");
        let duplicate_dense =
            dense_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_dense, dense_json);
        std::fs::write(
            dense_root.join(DENSE_OPERATOR_BINDING_FILE),
            duplicate_dense,
        )
        .expect("duplicate dense sidecar");
        let error = capture(&meta, &records).expect_err("duplicate dense key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let nested_manifest_root = unique_root("duplicate-nested-manifest");
        let (state, merged) =
            write_corpus_pair(&nested_manifest_root, observation::STATE_FILE, "merged.bin");
        let mut manifest = ObservationManifest::new(1);
        manifest.attention_operator = Some(attention_v2.clone());
        manifest.dense_operator = Some(dense_v2.clone());
        let manifest_json = serde_json::to_string(&manifest).expect("manifest JSON");
        let duplicate_nested =
            manifest_json.replacen("\"version\":2", "\"version\":2,\"version\":1", 1);
        assert_ne!(duplicate_nested, manifest_json);
        std::fs::write(
            nested_manifest_root.join(observation::MANIFEST_FILE),
            duplicate_nested,
        )
        .expect("nested duplicate manifest");
        let error = capture(&state, &merged).expect_err("nested duplicate key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        let top_manifest_root = unique_root("duplicate-top-manifest");
        let (state, merged) =
            write_corpus_pair(&top_manifest_root, observation::STATE_FILE, "merged.bin");
        let mut duplicate_top = manifest_json
            .strip_suffix(char::from(125))
            .expect("manifest object")
            .to_owned();
        duplicate_top.push_str(",\"dense_operator\":");
        duplicate_top.push_str(&dense_json);
        duplicate_top.push(char::from(125));
        std::fs::write(
            top_manifest_root.join(observation::MANIFEST_FILE),
            duplicate_top,
        )
        .expect("top-level duplicate manifest");
        let error = capture(&state, &merged).expect_err("top-level duplicate key is terminal");
        assert!(error.reason.contains("duplicate JSON field"), "{error}");

        for root in [
            attention_root,
            dense_root,
            nested_manifest_root,
            top_manifest_root,
        ] {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn open_stream_rewinds_retained_records_and_hidden_handles() {
        let root = unique_root("open-stream-rewind");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 41);
        let hidden_path = hidden_path(&records);
        let hidden_bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        std::fs::write(&hidden_path, hidden_bytes).expect("hidden rows");
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);

        let expected_records = std::fs::read(&records).expect("expected records");
        let mut stream = open_stream_under_guard(&guard, &meta, &records).expect("open stream");
        let mut captured_records = Vec::new();
        stream
            .records
            .read_to_end(&mut captured_records)
            .expect("read records from offset zero");
        let mut captured_hidden = Vec::new();
        stream
            .hidden
            .as_mut()
            .expect("hidden handle")
            .read_to_end(&mut captured_hidden)
            .expect("read hidden from offset zero");
        assert_eq!(captured_records, expected_records);
        assert_eq!(captured_hidden, hidden_bytes);
        stream.verify_generation().expect("final generation");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn verified_member_read_and_end_seek_remain_bounded_after_append() {
        let root = unique_root("open-stream-bounded-live-eof");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 41);
        let expected = std::fs::read(&records).expect("initial records");
        let mut stream = open_stream_under_guard(&guard, &meta, &records).expect("open stream");

        let mut append = OpenOptions::new()
            .append(true)
            .open(&records)
            .expect("append handle");
        append
            .write_all(b"bytes beyond the committed generation")
            .expect("append records");
        append.sync_all().expect("sync append");

        assert_eq!(
            stream
                .records
                .seek(SeekFrom::End(0))
                .expect("bounded logical end"),
            expected.len() as u64
        );
        stream
            .records
            .seek(SeekFrom::Start(0))
            .expect("rewind bounded member");
        let mut captured = Vec::new();
        stream
            .records
            .read_to_end(&mut captured)
            .expect("bounded read to end");
        assert_eq!(
            captured, expected,
            "live append is outside the bounded view"
        );
        let error = stream
            .verify_generation()
            .expect_err("final generation check still rejects the append");
        assert!(error.reason.contains("changed generation"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn verified_member_hashing_is_bounded_to_the_initial_length() {
        let root = unique_root("open-stream-initial-length");
        let (_meta, records) = complete_source_corpus(&root, 42);
        let error = open_verified_corpus_member_with_hook(
            &records,
            "corpus.records",
            "recorded corpus records",
            None,
            || {
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(&records)
                    .expect("append records");
                file.write_all(b"growth after metadata")
                    .expect("append growth");
                file.sync_all().expect("sync growth");
            },
        )
        .expect_err("post-metadata growth cannot extend the hashed generation");
        assert!(error.reason.contains("grew beyond its initial"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn verified_member_rejects_binding_presence_and_length_before_hashing() {
        let root = unique_root("open-stream-prehash-binding-shape");
        std::fs::create_dir_all(&root).expect("root");
        let records = root.join("corpus.records");
        std::fs::write(&records, b"present").expect("records");

        let absent =
            RecordedCorpusFileBinding::from_bytes("corpus.records", None).expect("absent binding");
        let entered = std::cell::Cell::new(false);
        let error = open_verified_corpus_member_with_hook(
            &records,
            "corpus.records",
            "recorded corpus records",
            Some(&absent),
            || entered.set(true),
        )
        .expect_err("declared absence rejects a present file before hashing");
        assert!(error.reason.contains("declares it absent"), "{error}");
        assert!(!entered.get(), "declared-absence mismatch entered hashing");

        let expected = RecordedCorpusFileBinding::from_bytes("corpus.records", Some(b"small"))
            .expect("small binding");
        let sparse = OpenOptions::new()
            .write(true)
            .open(&records)
            .expect("sparse records");
        sparse.set_len(1u64 << 40).expect("sparse oversized file");
        sparse.sync_all().expect("sync sparse length");
        let entered = std::cell::Cell::new(false);
        let error = open_verified_corpus_member_with_hook(
            &records,
            "corpus.records",
            "recorded corpus records",
            Some(&expected),
            || entered.set(true),
        )
        .expect_err("declared length rejects an oversized sparse file before hashing");
        assert!(error.reason.contains("before body hashing"), "{error}");
        assert!(!entered.get(), "length mismatch entered hashing");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn open_stream_final_check_covers_every_member_and_typed_hidden_absence() {
        fn assert_change_rejected<F>(label: &str, with_hidden: bool, mutate: F)
        where
            F: FnOnce(&Path, &Path, &Path),
        {
            let root = unique_root(label);
            let guard = producer_guard(&root);
            let (meta, records) = complete_source_corpus(&root, 43);
            let hidden = hidden_path(&records);
            if with_hidden {
                std::fs::write(&hidden, b"initial hidden").expect("initial hidden");
            }
            write_execution_pair(&root, 2);
            publish_compile_binding(&guard, &meta, &records);
            let stream = open_stream_under_guard(&guard, &meta, &records).expect("open stream");
            mutate(&root, &meta, &records);
            let error = stream
                .verify_generation()
                .expect_err("changed member must invalidate stream");
            assert!(
                error.reason.contains("changed")
                    || error.reason.contains("appeared")
                    || error.reason.contains("after capture"),
                "{label}: {error}"
            );
            let _ = std::fs::remove_dir_all(root);
        }

        assert_change_rejected("stream-records-change", false, |_, _, records| {
            std::fs::write(records, b"changed records").expect("change records");
        });
        assert_change_rejected("stream-meta-change", false, |_, meta, _| {
            std::fs::write(meta, b"changed metadata").expect("change metadata");
        });
        assert_change_rejected("stream-hidden-change", true, |_, _, records| {
            std::fs::write(hidden_path(records), b"changed hidden").expect("change hidden");
        });
        assert_change_rejected("stream-hidden-appearance", false, |_, _, records| {
            std::fs::write(hidden_path(records), b"appeared hidden").expect("add hidden");
        });
        assert_change_rejected("stream-provenance-change", false, |root, _, _| {
            write_operator(
                &root.join(ATTENTION_OPERATOR_BINDING_FILE),
                &AttentionOperatorSpec::learned_absolute_v1(),
            );
        });
        assert_change_rejected("stream-binding-change", false, |root, _, _| {
            std::fs::write(root.join(RECORDED_CORPUS_BINDING_FILE), b"changed binding")
                .expect("change binding");
        });
        assert_change_rejected("stream-marker-change", false, |root, _, _| {
            std::fs::write(
                root.join(RECORDED_CORPUS_COMPILE_ATTEMPT_FILE),
                b"changed marker",
            )
            .expect("change marker");
        });
    }

    #[test]
    fn open_stream_hashes_sparse_hidden_without_materializing_its_body() {
        let root = unique_root("open-stream-sparse-hidden");
        let guard = producer_guard(&root);
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        let rows = 4_096u64;
        let mut meta_bytes = [0u8; 25];
        meta_bytes[0..8].copy_from_slice(&rows.to_le_bytes());
        meta_bytes[8..16].copy_from_slice(&8u64.to_le_bytes());
        meta_bytes[16..24].copy_from_slice(&7u64.to_le_bytes());
        meta_bytes[24] = 1;
        std::fs::write(&meta, meta_bytes).expect("metadata");
        let records_file = std::fs::File::create(&records).expect("records file");
        records_file.set_len(rows * 88).expect("sparse records");
        let hidden = hidden_path(&records);
        let hidden_file = std::fs::File::create(&hidden).expect("hidden file");
        let hidden_len = rows * 4_096;
        hidden_file.set_len(hidden_len).expect("sparse hidden");
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);

        let mut stream = open_stream_under_guard(&guard, &meta, &records).expect("bounded stream");
        assert_eq!(stream.records.len(), rows * 88);
        assert_eq!(stream.hidden.as_ref().expect("hidden").len(), hidden_len);
        assert_eq!(stream.meta_bytes, meta_bytes);
        let (records_bytes, compiler_hidden) = stream
            .materialize_compiler_corpus_bytes()
            .expect("materialize compiler inputs");
        assert_eq!(records_bytes.len() as u64, rows * 88);
        assert!(
            compiler_hidden.is_none(),
            "opaque non-D hidden rows stay stream-only"
        );
        stream
            .verify_generation()
            .expect("stable sparse generation");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn identity_reader_is_non_mutating_for_markerless_archives_and_busy_for_writers() {
        let container = unique_root("identity-reader-container");
        let root = container.join("archive");
        let (meta, records) = complete_source_corpus(&root, 47);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let coordination = container.join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR);
        assert!(!coordination.exists());
        let identity = execution_identity(&meta, &records).expect("markerless identity");
        assert_eq!(
            identity.attention_operator,
            Some(AttentionOperatorSpec::learned_absolute_v2())
        );
        assert_eq!(identity.dense_operator, None);
        assert!(
            !coordination.exists(),
            "read-only identity resolution cannot create coordination state"
        );

        let producer = RecordedCorpusProducerGuard::try_acquire(&root).expect("producer");
        let busy = execution_identity(&meta, &records).expect_err("writer excludes reader");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        drop(producer);
        execution_identity(&meta, &records).expect("reader after writer drop");
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn derivation_reader_locks_only_markerless_sources() {
        let markerless = unique_root("derivation-reader-markerless");
        let (meta, records) = complete_source_corpus(&markerless, 59);
        write_operator(
            &markerless.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        let (snapshot, source_guard) =
            open_stream_for_derivation(&meta, &records).expect("markerless derivation");
        assert!(source_guard.is_some(), "markerless source needs ownership");
        let busy = RecordedCorpusProducerGuard::try_acquire(&markerless)
            .expect_err("retained markerless guard excludes writers");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        snapshot.verify_generation().expect("markerless generation");
        drop(source_guard);

        let bound = unique_root("derivation-reader-bound");
        let guard = producer_guard(&bound);
        let (meta, records) = complete_source_corpus(&bound, 61);
        write_execution_pair(&bound, 2);
        publish_compile_binding(&guard, &meta, &records);
        drop(guard);
        let (snapshot, source_guard) =
            open_stream_for_derivation(&meta, &records).expect("bound derivation");
        assert!(
            source_guard.is_none(),
            "binding commits the bound generation"
        );
        snapshot.verify_generation().expect("bound generation");

        let _ = std::fs::remove_dir_all(markerless);
        let _ = std::fs::remove_dir_all(bound);
    }

    #[test]
    fn ready_deterministic_inventory_allows_only_real_graph_directories() {
        let root = unique_root("ready-deterministic-inventory");
        let guard = producer_guard(&root);
        let _ = complete_source_corpus(&root, 67);
        std::fs::create_dir(root.join("graph")).expect("graph directory");
        std::fs::create_dir(root.join("graph-cover")).expect("cover directory");
        guard
            .preflight_ready_deterministic_compile_inventory(&[
                PlannedOutputMember::Records,
                PlannedOutputMember::Metadata,
            ])
            .expect("exact ready generation may retain downstream graph directories");

        let foreign = root.join("compile_report.json");
        std::fs::write(&foreign, b"foreign").expect("foreign report");
        let error = guard
            .preflight_ready_deterministic_compile_inventory(&[
                PlannedOutputMember::Records,
                PlannedOutputMember::Metadata,
            ])
            .expect_err("foreign root report is terminal even on a ready generation");
        assert!(error.reason.contains("unowned"), "{error}");
        assert_eq!(std::fs::read(&foreign).unwrap(), b"foreign");
        std::fs::remove_file(&foreign).expect("remove foreign report");

        std::fs::remove_dir(root.join("graph")).expect("remove graph directory");
        std::fs::write(root.join("graph"), b"not a directory").expect("graph file");
        let error = guard
            .preflight_ready_deterministic_compile_inventory(&[
                PlannedOutputMember::Records,
                PlannedOutputMember::Metadata,
            ])
            .expect_err("ready graph name must be a real directory");
        assert!(
            error.reason.contains("not a real non-symlink directory"),
            "{error}"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn producer_handoff_sorts_full_selection_failure_atomically_and_contends() {
        let container = unique_root("producer-handoff-selection");
        std::fs::create_dir_all(&container).expect("container");
        let conventional = container.join("z-conventional");
        let current = container.join("m-current");
        let composite = container.join("b-composite");
        let stage = container.join("a-stage");
        for root in [&conventional, &current, &composite, &stage] {
            std::fs::create_dir(root).expect("selection root");
        }
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&conventional).expect("conventional"),
            RecordedCorpusRootGeneration::capture(&current).expect("current"),
            RecordedCorpusRootGeneration::capture(&composite).expect("composite"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage"),
            RecordedCorpusRootGeneration::capture(&current).expect("current alias"),
        ];

        let blocker = RecordedCorpusProducerGuard::try_acquire(&conventional).expect("blocker");
        let busy = RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3)
            .expect_err("last sorted root blocks the complete acquisition");
        assert!(is_recorded_corpus_busy(&busy), "{busy}");
        for root in [&stage, &composite, &current] {
            let probe = RecordedCorpusProducerGuard::try_acquire(root)
                .expect("earlier sorted acquisition was released on failure");
            drop(probe);
        }
        drop(blocker);

        let handoff =
            RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3).expect("complete handoff");
        let guarded = handoff
            .guards()
            .map(|guard| guard.root().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            guarded,
            vec![
                expected[3].root().to_owned(),
                expected[2].root().to_owned(),
                expected[1].root().to_owned(),
                expected[0].root().to_owned(),
            ]
        );
        assert_eq!(handoff.final_guard().root(), expected[1].root());
        assert_eq!(handoff.stage_guard().root(), expected[3].root());
        for root in [&conventional, &current, &composite, &stage] {
            let busy = RecordedCorpusProducerGuard::try_acquire(root)
                .expect_err("handoff excludes an independent producer");
            assert!(is_recorded_corpus_busy(&busy), "{busy}");
        }
        drop(handoff);
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn derivation_guards_sort_source_and_destination_without_promotion_capability() {
        let container = unique_root("derivation-guards");
        std::fs::create_dir_all(&container).expect("container");
        let source = container.join("z-source");
        let destination = container.join("a-destination");
        std::fs::create_dir(&source).expect("source");

        let mut guards = RecordedCorpusDerivationGuards::try_acquire(&source, &destination)
            .expect("sorted derivation authority");
        assert_eq!(
            guards.source_guard().root(),
            std::fs::canonicalize(&source).expect("canonical source")
        );
        assert_eq!(
            guards.destination_guard().root(),
            std::fs::canonicalize(&container)
                .expect("canonical container")
                .join("a-destination")
        );
        assert!(!guards.destination_guard().root_exists());
        guards
            .destination_guard_mut()
            .ensure_root()
            .expect("create guarded destination");
        guards.verify().expect("both generations remain guarded");

        for root in [&source, &destination] {
            let busy = RecordedCorpusProducerGuard::try_acquire(root)
                .expect_err("derivation retains both exclusive guards");
            assert!(is_recorded_corpus_busy(&busy), "{busy}");
        }
        drop(guards);
        for root in [&source, &destination] {
            let probe = RecordedCorpusProducerGuard::try_acquire(root)
                .expect("derivation releases both guards together");
            drop(probe);
        }
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn derivation_guards_reject_aliases_and_release_partial_sorted_acquisition() {
        let container = unique_root("derivation-guards-alias");
        std::fs::create_dir_all(&container).expect("container");
        let source = container.join("z-source");
        let destination = container.join("a-destination");
        std::fs::create_dir(&source).expect("source");
        std::fs::create_dir(&destination).expect("destination");

        let alias = RecordedCorpusDerivationGuards::try_acquire(
            &source,
            container.join(".").join("z-source"),
        )
        .expect_err("in-place canonical alias is refused");
        assert!(alias.reason.contains("same canonical root"), "{alias}");

        let blocker = RecordedCorpusProducerGuard::try_acquire(&source).expect("source blocker");
        let busy = RecordedCorpusDerivationGuards::try_acquire(&source, &destination)
            .expect_err("later sorted source contention fails atomically");
        assert!(is_recorded_corpus_busy(&busy), "{busy}");
        let destination_probe = RecordedCorpusProducerGuard::try_acquire(&destination)
            .expect("earlier sorted destination acquisition was released");
        drop(destination_probe);
        drop(blocker);

        #[cfg(unix)]
        {
            let link = container.join("source-link");
            std::os::unix::fs::symlink(&source, &link).expect("source symlink");
            let error = RecordedCorpusDerivationGuards::try_acquire(&link, &destination)
                .expect_err("symlink root is terminal");
            assert!(error.reason.contains("not a real non-symlink"), "{error}");
        }
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn producer_handoff_root_ceiling_counts_unique_and_duplicate_entries_before_locking() {
        let unique_container = unique_root("producer-handoff-root-ceiling-unique");
        std::fs::create_dir_all(&unique_container).expect("unique container");
        let unique_roots = (0..=RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES)
            .map(|index| unique_container.join(format!("root-{index:02}")))
            .collect::<Vec<_>>();
        for root in &unique_roots {
            std::fs::create_dir(root).expect("unique root");
        }
        let unique_generations = unique_roots
            .iter()
            .map(RecordedCorpusRootGeneration::capture)
            .collect::<Result<Vec<_>, _>>()
            .expect("unique generations");
        let error = RecordedCorpusProducerHandoff::try_acquire(&unique_generations, 0, 1)
            .expect_err("unique over-limit handoff must fail closed");
        assert!(
            error.reason.contains("fixed 16-entry logical-root ceiling"),
            "{error}"
        );
        assert!(
            !unique_container
                .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR)
                .exists(),
            "over-limit handoff cannot create or retain coordination state"
        );
        for root in &unique_roots {
            let probe = RecordedCorpusProducerGuard::try_acquire(root)
                .expect("over-limit handoff retained no unique-root lock");
            drop(probe);
        }
        drop(unique_generations);
        let _ = std::fs::remove_dir_all(unique_container);

        let duplicate_container = unique_root("producer-handoff-root-ceiling-duplicate");
        std::fs::create_dir_all(&duplicate_container).expect("duplicate container");
        let duplicate_root = duplicate_container.join("root");
        std::fs::create_dir(&duplicate_root).expect("duplicate root");
        let duplicate_generations = (0..=RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES)
            .map(|_| RecordedCorpusRootGeneration::capture(&duplicate_root))
            .collect::<Result<Vec<_>, _>>()
            .expect("duplicate generations");
        let error = RecordedCorpusProducerHandoff::try_acquire(&duplicate_generations, 0, 1)
            .expect_err("duplicates cannot bypass the raw handoff ceiling");
        assert!(
            error.reason.contains("fixed 16-entry logical-root ceiling"),
            "{error}"
        );
        assert!(
            !duplicate_container
                .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR)
                .exists(),
            "duplicate over-limit handoff cannot acquire a lock"
        );
        let probe = RecordedCorpusProducerGuard::try_acquire(&duplicate_root)
            .expect("duplicate over-limit handoff retained no lock");
        drop(probe);
        drop(duplicate_generations);
        let _ = std::fs::remove_dir_all(duplicate_container);
    }

    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    #[test]
    fn producer_handoff_promotes_into_absent_final_and_adopts_inode() {
        let container = unique_root("producer-handoff-absent-final");
        std::fs::create_dir_all(&container).expect("container");
        let conventional = container.join("conventional");
        let final_root = container.join("current");
        let composite = container.join("composite");
        let stage = container.join("stage");
        for root in [&conventional, &composite, &stage] {
            std::fs::create_dir(root).expect("selection root");
        }
        std::fs::write(stage.join("sentinel"), b"promoted").expect("stage sentinel");
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&conventional).expect("conventional"),
            RecordedCorpusRootGeneration::capture(&final_root).expect("absent final"),
            RecordedCorpusRootGeneration::capture(&composite).expect("composite"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage"),
        ];
        assert!(!expected[1].exists());

        let mut handoff =
            RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3).expect("complete handoff");
        handoff.promote_stage().expect("no-replace promotion");
        assert_eq!(
            std::fs::read(final_root.join("sentinel")).unwrap(),
            b"promoted"
        );
        assert!(!stage.exists(), "typed-absent final leaves stage absent");
        handoff
            .verify()
            .expect("adopted post-promotion generations");
        handoff
            .final_guard()
            .verify_owned_root()
            .expect("final guard adopted promoted inode");
        let busy = RecordedCorpusProducerGuard::try_acquire(&final_root)
            .expect_err("adopted final ownership remains exclusive");
        assert!(is_recorded_corpus_busy(&busy), "{busy}");
        drop(handoff);
        let probe = RecordedCorpusProducerGuard::try_acquire(&final_root)
            .expect("final becomes available after handoff drop");
        drop(probe);
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    #[test]
    fn producer_handoff_exchanges_existing_roots_and_rebinds_both_guards() {
        let container = unique_root("producer-handoff-exchange");
        std::fs::create_dir_all(&container).expect("container");
        let conventional = container.join("conventional");
        let final_root = container.join("current");
        let composite = container.join("composite");
        let stage = container.join("stage");
        for root in [&conventional, &final_root, &composite, &stage] {
            std::fs::create_dir(root).expect("selection root");
        }
        std::fs::write(final_root.join("sentinel"), b"old-final").expect("old final");
        std::fs::write(stage.join("sentinel"), b"new-stage").expect("new stage");
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&conventional).expect("conventional"),
            RecordedCorpusRootGeneration::capture(&final_root).expect("final"),
            RecordedCorpusRootGeneration::capture(&composite).expect("composite"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage"),
        ];

        let mut handoff =
            RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3).expect("complete handoff");
        handoff.promote_stage().expect("atomic exchange");
        assert_eq!(
            std::fs::read(final_root.join("sentinel")).unwrap(),
            b"new-stage"
        );
        assert_eq!(std::fs::read(stage.join("sentinel")).unwrap(), b"old-final");
        handoff.verify().expect("exchanged generations adopted");
        handoff
            .final_guard()
            .verify_owned_root()
            .expect("final guard rebound");
        handoff
            .stage_guard()
            .verify_owned_root()
            .expect("stage guard rebound");
        let repeated = handoff
            .promote_stage()
            .expect_err("one handoff cannot promote twice");
        assert!(repeated.reason.contains("already promoted"), "{repeated}");
        drop(handoff);
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    #[test]
    fn producer_handoff_cas_rechecks_non_designated_selection_root_before_exchange() {
        let container = unique_root("producer-handoff-selection-cas");
        std::fs::create_dir_all(&container).expect("container");
        let conventional = container.join("conventional");
        let final_root = container.join("current");
        let composite = container.join("composite");
        let stage = container.join("stage");
        for root in [&conventional, &final_root, &composite, &stage] {
            std::fs::create_dir(root).expect("selection root");
        }
        std::fs::write(final_root.join("sentinel"), b"old-final").expect("old final");
        std::fs::write(stage.join("sentinel"), b"new-stage").expect("new stage");
        std::fs::write(composite.join("sentinel"), b"selected-old").expect("selected old");
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&conventional).expect("conventional"),
            RecordedCorpusRootGeneration::capture(&final_root).expect("final"),
            RecordedCorpusRootGeneration::capture(&composite).expect("composite"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage"),
        ];
        let displaced = container.join("displaced-composite");
        let mut handoff =
            RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3).expect("complete handoff");
        let conflict = handoff
            .promote_stage_with_hook(|_| {
                std::fs::rename(&composite, &displaced).expect("replace selected root");
                std::fs::create_dir(&composite).expect("replacement selected root");
                std::fs::write(composite.join("sentinel"), b"selected-new")
                    .expect("replacement sentinel");
                Ok(())
            })
            .expect_err("selection replacement must fail the final CAS");
        assert!(
            conflict.reason.contains("changed identity")
                || conflict.reason.contains("changed generation"),
            "{conflict}"
        );
        assert_eq!(
            std::fs::read(final_root.join("sentinel")).unwrap(),
            b"old-final"
        );
        assert_eq!(std::fs::read(stage.join("sentinel")).unwrap(), b"new-stage");
        assert_eq!(
            std::fs::read(composite.join("sentinel")).unwrap(),
            b"selected-new"
        );
        drop(handoff);
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn producer_handoff_rejects_a_preexisting_generation_conflict_without_mutation() {
        let container = unique_root("producer-handoff-initial-cas");
        std::fs::create_dir_all(&container).expect("container");
        let conventional = container.join("conventional");
        let final_root = container.join("current");
        let composite = container.join("composite");
        let stage = container.join("stage");
        for root in [&conventional, &final_root, &composite, &stage] {
            std::fs::create_dir(root).expect("selection root");
        }
        std::fs::write(final_root.join("sentinel"), b"old-final").expect("old final");
        std::fs::write(stage.join("sentinel"), b"new-stage").expect("new stage");
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&conventional).expect("conventional"),
            RecordedCorpusRootGeneration::capture(&final_root).expect("final"),
            RecordedCorpusRootGeneration::capture(&composite).expect("composite"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage"),
        ];
        let displaced = container.join("displaced-conventional");
        std::fs::rename(&conventional, &displaced).expect("displace conventional");
        std::fs::create_dir(&conventional).expect("replacement conventional");

        let conflict = RecordedCorpusProducerHandoff::try_acquire(&expected, 1, 3)
            .expect_err("stale root witness is terminal");
        assert!(
            conflict.reason.contains("changed identity")
                || conflict.reason.contains("changed generation"),
            "{conflict}"
        );
        assert_eq!(
            std::fs::read(final_root.join("sentinel")).unwrap(),
            b"old-final"
        );
        assert_eq!(std::fs::read(stage.join("sentinel")).unwrap(), b"new-stage");
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    #[test]
    fn producer_handoff_content_predicate_refuses_same_directory_commit() {
        let container = unique_root("producer-handoff-content-cas");
        std::fs::create_dir_all(&container).expect("container");
        let final_root = container.join("current");
        let stage = container.join("stage");
        std::fs::create_dir(&final_root).expect("final root");
        std::fs::create_dir(&stage).expect("stage root");
        std::fs::write(final_root.join("binding"), b"generation-a").expect("base A");
        std::fs::write(stage.join("binding"), b"generation-stage").expect("stage");
        let expected = vec![
            RecordedCorpusRootGeneration::capture(&final_root).expect("final witness"),
            RecordedCorpusRootGeneration::capture(&stage).expect("stage witness"),
        ];
        // A cooperating writer committed B in-place during the long build;
        // the directory inode is intentionally unchanged.
        std::fs::write(final_root.join("binding"), b"generation-b").expect("commit B");
        let mut handoff =
            RecordedCorpusProducerHandoff::try_acquire(&expected, 0, 1).expect("inode handoff");
        let error = handoff
            .promote_stage_if(|_| {
                if std::fs::read(final_root.join("binding")).unwrap() != b"generation-a" {
                    return Err(SourceUnavailable::new(
                        "typed selection content changed from generation A",
                    ));
                }
                Ok(())
            })
            .expect_err("content predicate must preserve B");
        assert!(error.reason.contains("content changed"), "{error}");
        assert_eq!(
            std::fs::read(final_root.join("binding")).unwrap(),
            b"generation-b"
        );
        assert_eq!(
            std::fs::read(stage.join("binding")).unwrap(),
            b"generation-stage"
        );
        drop(handoff);
        drop(expected);
        let _ = std::fs::remove_dir_all(container);
    }

    #[cfg(unix)]
    #[test]
    fn reader_pins_cover_absence_read_only_parents_shared_locks_and_busy() {
        use std::os::unix::fs::PermissionsExt;

        let container = unique_root("reader-pins");
        std::fs::create_dir_all(&container).expect("container");
        let existing = container.join("z-existing");
        let absent = container.join("a-absent");
        std::fs::create_dir(&existing).expect("existing root");

        let existing_seed =
            RecordedCorpusProducerGuard::try_acquire(&existing).expect("seed existing lock");
        drop(existing_seed);
        let absent_seed =
            RecordedCorpusProducerGuard::try_acquire(&absent).expect("seed absent lock");
        drop(absent_seed);
        std::fs::set_permissions(&container, std::fs::Permissions::from_mode(0o555))
            .expect("read-only parent");

        let pins = RecordedCorpusReaderPins::try_acquire([&existing, &absent])
            .expect("read-only multi-root pins");
        let generations = pins.generations().collect::<Vec<_>>();
        assert_eq!(generations[0].root().file_name(), absent.file_name());
        assert!(!generations[0].exists());
        assert_eq!(generations[1].root().file_name(), existing.file_name());
        assert!(generations[1].exists());
        pins.verify().expect("stable pinned generations");
        let second = RecordedCorpusReaderPins::try_acquire([&absent, &existing])
            .expect("shared pins coexist");
        for root in [&absent, &existing] {
            let busy = RecordedCorpusProducerGuard::try_acquire(root)
                .expect_err("shared pin excludes producer");
            assert!(is_recorded_corpus_busy(&busy), "{busy}");
        }
        drop(second);
        drop(pins);
        std::fs::set_permissions(&container, std::fs::Permissions::from_mode(0o755))
            .expect("restore parent");

        let producer =
            RecordedCorpusProducerGuard::try_acquire(&existing).expect("active producer");
        let busy = RecordedCorpusReaderPins::try_acquire([&existing, &absent])
            .expect_err("producer excludes complete shared-pin acquisition");
        assert!(is_recorded_corpus_busy(&busy), "{busy}");
        let absent_probe = RecordedCorpusProducerGuard::try_acquire(&absent)
            .expect("earlier shared pin released on later-root failure");
        drop(absent_probe);
        drop(producer);
        let _ = std::fs::remove_dir_all(container);
    }

    #[test]
    fn reader_pin_mode_root_ceilings_reject_duplicates_and_unique_roots_without_locks() {
        let duplicate_container = unique_root("reader-pin-root-ceiling-duplicate");
        std::fs::create_dir_all(&duplicate_container).expect("duplicate container");
        let duplicate_root = duplicate_container.join("root");
        std::fs::create_dir(&duplicate_root).expect("duplicate root");
        let duplicate_roots =
            vec![duplicate_root.clone(); RECORDED_CORPUS_MULTI_ROOT_MAX_ENTRIES + 1];
        let error = RecordedCorpusReaderPins::try_acquire(&duplicate_roots)
            .expect_err("duplicates cannot bypass the read-only reader-pin ceiling");
        assert!(
            error.reason.contains("fixed 16-entry logical-root ceiling"),
            "{error}"
        );
        assert!(
            !duplicate_container
                .join(RECORDED_CORPUS_PRODUCER_COORDINATION_DIR)
                .exists(),
            "reader ceiling is checked before coordination capture"
        );
        let probe = RecordedCorpusProducerGuard::try_acquire(&duplicate_root)
            .expect("read-only over-limit pins retained no lock");
        drop(probe);
        let _ = std::fs::remove_dir_all(duplicate_container);
    }

    #[test]
    fn retained_authority_stream_and_identity_helpers_avoid_self_busy() {
        let container = unique_root("retained-authority-helpers");
        let root = container.join("corpus");
        let guard = producer_guard(&root);
        let (meta, records) = complete_source_corpus(&root, 71);
        write_execution_pair(&root, 2);
        publish_compile_binding(&guard, &meta, &records);
        guard
            .finish_compile_attempt()
            .expect("finish compile attempt");

        let identity = execution_identity_under_producer_guard(&guard, &meta, &records)
            .expect("identity under producer authority");
        assert_eq!(identity.dense_operator, Some(DenseOperatorSpec::gpt2_v2()));
        let busy = execution_identity(&meta, &records).expect_err("nested reader self-contends");
        assert!(is_recorded_corpus_busy(&busy), "{busy}");
        drop(guard);

        let pins = RecordedCorpusReaderPins::try_acquire([&root]).expect("reader pins");
        let stream = open_stream_under_reader_pins(&pins, &meta, &records)
            .expect("stream under retained pins");
        stream.verify_generation().expect("stream generation");
        let identity = execution_identity_under_reader_pins(&pins, &meta, &records)
            .expect("identity under retained pins");
        assert_eq!(identity.dense_operator, Some(DenseOperatorSpec::gpt2_v2()));
        pins.verify().expect("reader authority remains valid");
        drop(pins);
        let _ = std::fs::remove_dir_all(container);
    }

    #[cfg(unix)]
    #[test]
    fn identity_reader_accepts_a_read_only_existing_archive() {
        use std::os::unix::fs::PermissionsExt;

        let container = unique_root("identity-reader-read-only");
        let root = container.join("archive");
        let (meta, records) = complete_source_corpus(&root, 53);
        write_operator(
            &root.join(ATTENTION_OPERATOR_BINDING_FILE),
            &AttentionOperatorSpec::learned_absolute_v2(),
        );
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o555))
            .expect("read-only root");
        std::fs::set_permissions(&container, std::fs::Permissions::from_mode(0o555))
            .expect("read-only parent");
        let identity = execution_identity(&meta, &records).expect("read-only identity");
        assert_eq!(
            identity.attention_operator,
            Some(AttentionOperatorSpec::learned_absolute_v2())
        );
        std::fs::set_permissions(&container, std::fs::Permissions::from_mode(0o755))
            .expect("restore parent");
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755))
            .expect("restore root");
        let _ = std::fs::remove_dir_all(container);
    }
}
