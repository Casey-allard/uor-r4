//! Application-level CID bundle management for R⁴ transformerless models.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

const MANIFEST_SCHEMA: u32 = 1;

/// Schema tag of the versioned source-snapshot manifest (#597).
pub const SOURCE_MANIFEST_SCHEMA: &str = "uor-r4-source-manifest/1";

/// File name the source-snapshot manifest is written under inside a
/// snapshot directory. The manifest is excluded from its own file list.
pub const SOURCE_MANIFEST_FILE_NAME: &str = "source_manifest.json";

/// The only source-execution mode this stack ships today: the downloaded
/// open-weight snapshot is executed exclusively as an offline compiler
/// input (the teacher forward pass); the deployed runtime never runs it.
pub const SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT: &str = "offline-compiler-input";

/// Default CID-manifest name selected when neither CLI nor environment chooses one.
pub const DEFAULT_CHAT_MODEL: &str = "smollm2-135m-instruct";

/// Select the most recently modified model descriptor in `models/`.
///
/// `TLESS_MODEL` always wins. The static default is used when discovery is
/// unavailable, such as when a binary runs outside the repository checkout.
pub fn default_model_reference() -> String {
    std::env::var("TLESS_MODEL")
        .ok()
        .or_else(|| latest_descriptor_name(Path::new("models")))
        .unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelCapability {
    Continuation,
    InstructionChat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelObject {
    pub cid: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityAttestation {
    pub instruction_eval_passed: bool,
    pub grounded_answer_rate: f32,
    pub repetition_rate: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelManifest {
    pub schema: u32,
    pub name: String,
    pub source_model: String,
    pub capability: ModelCapability,
    pub artifacts: ModelObject,
    pub store: ModelObject,
    pub tokenizer: ModelObject,
    pub evaluation_report: Option<ModelObject>,
    pub quality: QualityAttestation,
}

impl ModelManifest {
    pub fn validate_for_chat(&self) -> Result<(), ModelError> {
        if self.schema != MANIFEST_SCHEMA {
            return Err(ModelError::UnsupportedSchema(self.schema));
        }
        if self.capability != ModelCapability::InstructionChat {
            return Err(ModelError::NotChatCapable);
        }
        if !self.quality.instruction_eval_passed {
            return Err(ModelError::QualityGateFailed);
        }
        if self.evaluation_report.is_none() {
            return Err(ModelError::MissingEvaluationReport);
        }
        if !(0.0..=1.0).contains(&self.quality.grounded_answer_rate)
            || !(0.0..=1.0).contains(&self.quality.repetition_rate)
        {
            return Err(ModelError::InvalidQualityMetrics);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ModelError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidCid(String),
    InvalidRegionObject(String),
    SizeMismatch {
        cid: String,
        expected: u64,
        actual: u64,
    },
    UnsupportedSchema(u32),
    NotChatCapable,
    QualityGateFailed,
    MissingEvaluationReport,
    InvalidQualityMetrics,
    InvalidSourceName(String),
    InvalidRepository(String),
    UnpinnedRevision(String),
    DownloadToolMissing,
    DownloadFailed(Option<i32>),
    ManifestNotFound {
        reference: String,
        root: PathBuf,
    },
    SourceNotCompiled(PathBuf),
    CompiledNotImported(PathBuf),
    UnsupportedSourceManifestSchema(String),
    NonPortableSnapshotPath(PathBuf),
    ManifestAddressing(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "model storage I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "invalid model manifest: {error}"),
            Self::InvalidCid(cid) => {
                write!(formatter, "model object failed CID verification: {cid}")
            }
            Self::InvalidRegionObject(error) => {
                write!(formatter, "region object failed canonical verification: {error}")
            }
            Self::SizeMismatch {
                cid,
                expected,
                actual,
            } => write!(
                formatter,
                "model object {cid} has {actual} bytes; manifest declares {expected}"
            ),
            Self::UnsupportedSchema(schema) => {
                write!(formatter, "unsupported model manifest schema {schema}")
            }
            Self::NotChatCapable => formatter
                .write_str("model is continuation-only; ask requires an instruction-chat bundle"),
            Self::QualityGateFailed => formatter
                .write_str("model has not passed its instruction/grounding evaluation gate"),
            Self::MissingEvaluationReport => {
                formatter.write_str("chat model has no CID-addressed instruction evaluation report")
            }
            Self::InvalidQualityMetrics => {
                formatter.write_str("model quality metrics must be between zero and one")
            }
            Self::InvalidSourceName(name) => {
                write!(formatter, "invalid portable model source name: {name}")
            }
            Self::InvalidRepository(repository) => {
                write!(formatter, "invalid Hugging Face repository: {repository}")
            }
            Self::UnpinnedRevision(revision) => write!(
                formatter,
                "model revision must be a full 40-character commit hash, got: {revision}"
            ),
            Self::DownloadToolMissing => formatter.write_str(
                "the Hugging Face CLI is required for offline model downloads; install `hf`",
            ),
            Self::DownloadFailed(code) => {
                write!(formatter, "model download failed with exit code {code:?}")
            }
            Self::ManifestNotFound { reference, root } => write!(
                formatter,
                "compiled model manifest '{reference}' was not found under {}; run `cargo run --release -- compile` and optionally `cargo run -- import` first",
                root.display()
            ),
            Self::SourceNotCompiled(path) => write!(
                formatter,
                "{} is downloaded source data, not a compiled transformerless chat bundle; compile it with `cargo run --release -- compile --source {}` before using `ask`",
                path.display(),
                path.display()
            ),
            Self::CompiledNotImported(path) => write!(
                formatter,
                "compiled transformerless bundle found at {} but it has no imported manifest; direct local chat may load it, or use `cargo run -- import --help` to attach a quality attestation and persist a named manifest",
                path.display()
            ),
            Self::UnsupportedSourceManifestSchema(schema) => write!(
                formatter,
                "unsupported source-snapshot manifest schema '{schema}'; expected '{SOURCE_MANIFEST_SCHEMA}'"
            ),
            Self::NonPortableSnapshotPath(path) => write!(
                formatter,
                "snapshot file path is not portable UTF-8: {}",
                path.display()
            ),
            Self::ManifestAddressing(reason) => write!(
                formatter,
                "source-snapshot manifest could not be κ-addressed: {reason}"
            ),
        }
    }
}

impl std::error::Error for ModelError {}

impl From<std::io::Error> for ModelError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ModelError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone)]
pub struct ModelStore {
    root: PathBuf,
}

impl ModelStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn from_env() -> Self {
        let root = std::env::var_os("UOR_MODEL_STORE")
            .map_or_else(|| PathBuf::from(".uor-models"), PathBuf::from);
        Self::new(root)
    }

    pub fn put(&self, bytes: &[u8]) -> Result<ModelObject, ModelError> {
        let cid = address_container(bytes);
        let path = self.object_path(&cid)?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, bytes)?;
        }
        Ok(ModelObject {
            cid,
            bytes: bytes.len() as u64,
        })
    }

    pub fn get(&self, object: &ModelObject) -> Result<Vec<u8>, ModelError> {
        let bytes = std::fs::read(self.object_path(&object.cid)?)?;
        if bytes.len() as u64 != object.bytes {
            return Err(ModelError::SizeMismatch {
                cid: object.cid.clone(),
                expected: object.bytes,
                actual: bytes.len() as u64,
            });
        }
        let actual = address_container(&bytes);
        if actual != object.cid {
            return Err(ModelError::InvalidCid(object.cid.clone()));
        }
        Ok(bytes)
    }

    /// Store one canonical region object under its UOR κ-label.
    pub fn put_region_object(
        &self,
        object: &uor_r4_core::transformerless::region_store::RegionObject,
    ) -> Result<ModelObject, ModelError> {
        let bytes = uor_r4_core::transformerless::region_store::canonical_region_bytes(object)
            .ok_or_else(|| ModelError::InvalidRegionObject("canonical region bytes".to_string()))?;
        let cid = uor_r4_core::transformerless::region_store::region_kappa(object)
            .ok_or_else(|| ModelError::InvalidRegionObject("region addressing".to_string()))?;
        self.put_addressed_bytes(&cid, &bytes)
    }

    /// Load and verify one region object from the local CAS. A missing object
    /// is a normal resolver miss; malformed or tampered bytes are errors.
    pub fn get_region_object(
        &self,
        kappa: &str,
    ) -> Result<Option<uor_r4_core::transformerless::region_store::RegionObject>, ModelError> {
        let path = self.object_path(kappa)?;
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(ModelError::Io(error)),
        };
        let object = uor_r4_core::transformerless::region_store::decode_region_bytes(&bytes)
            .ok_or_else(|| {
                ModelError::InvalidRegionObject("malformed region object".to_string())
            })?;
        let actual = uor_r4_core::transformerless::region_store::region_kappa(&object)
            .ok_or_else(|| ModelError::InvalidRegionObject("region addressing".to_string()))?;
        if actual != kappa {
            return Err(ModelError::InvalidCid(kappa.to_owned()));
        }
        Ok(Some(object))
    }

    /// Store a canonical region manifest under its manifest κ-label.
    pub fn put_region_manifest(
        &self,
        manifest: &uor_r4_core::transformerless::region_store::RegionManifest,
    ) -> Result<ModelObject, ModelError> {
        let expected =
            uor_r4_core::transformerless::region_store::manifest_kappa_for(&manifest.regions)
                .ok_or_else(|| ModelError::InvalidRegionObject("region manifest".to_string()))?;
        if expected != manifest.manifest_kappa {
            return Err(ModelError::InvalidCid(manifest.manifest_kappa.clone()));
        }
        let bytes = uor_r4_core::transformerless::region_store::canonical_manifest_bytes(manifest)
            .ok_or_else(|| ModelError::InvalidRegionObject("region manifest".to_string()))?;
        self.put_addressed_bytes(&manifest.manifest_kappa, &bytes)
    }

    /// Load and verify a region manifest from the local CAS.
    pub fn get_region_manifest(
        &self,
        kappa: &str,
    ) -> Result<Option<uor_r4_core::transformerless::region_store::RegionManifest>, ModelError>
    {
        let path = self.object_path(kappa)?;
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(ModelError::Io(error)),
        };
        let manifest = uor_r4_core::transformerless::region_store::decode_manifest_bytes(&bytes)
            .ok_or_else(|| ModelError::InvalidRegionObject("region manifest".to_string()))?;
        if manifest.manifest_kappa != kappa {
            return Err(ModelError::InvalidCid(kappa.to_owned()));
        }
        Ok(Some(manifest))
    }

    pub fn write_manifest(&self, manifest: &ModelManifest) -> Result<String, ModelError> {
        let bytes = serde_json::to_vec_pretty(manifest)?;
        let object = self.put(&bytes)?;
        let manifests = self.root.join("manifests");
        std::fs::create_dir_all(&manifests)?;
        std::fs::write(
            manifests.join(format!("{}.json", safe_name(&manifest.name))),
            bytes,
        )?;
        Ok(object.cid)
    }

    pub fn read_manifest(&self, reference: &str) -> Result<ModelManifest, ModelError> {
        let supplied_path = Path::new(reference);
        if supplied_path.exists() {
            if is_compiled_bundle(supplied_path) {
                return Err(ModelError::CompiledNotImported(supplied_path.to_path_buf()));
            }
            return Err(ModelError::SourceNotCompiled(supplied_path.to_path_buf()));
        }
        let bytes = if reference.starts_with("blake3:") {
            let object = ModelObject {
                cid: reference.to_owned(),
                bytes: std::fs::metadata(self.object_path(reference)?)?.len(),
            };
            self.get(&object)?
        } else {
            let path = self
                .root
                .join("manifests")
                .join(format!("{}.json", safe_name(reference)));
            match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let compiled = self.root.join("compiled").join(safe_name(reference));
                    if is_compiled_bundle(&compiled) {
                        return Err(ModelError::CompiledNotImported(compiled));
                    }
                    let source = self.root.join("sources").join(safe_name(reference));
                    if source.is_dir() {
                        return Err(ModelError::SourceNotCompiled(source));
                    }
                    return Err(ModelError::ManifestNotFound {
                        reference: reference.to_owned(),
                        root: self.root.clone(),
                    });
                }
                Err(error) => return Err(ModelError::Io(error)),
            }
        };
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn object_path(&self, cid: &str) -> Result<PathBuf, ModelError> {
        let hash = cid
            .strip_prefix("blake3:")
            .ok_or_else(|| ModelError::InvalidCid(cid.to_owned()))?;
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ModelError::InvalidCid(cid.to_owned()));
        }
        Ok(self.root.join("objects").join("blake3").join(hash))
    }

    fn put_addressed_bytes(&self, cid: &str, bytes: &[u8]) -> Result<ModelObject, ModelError> {
        let path = self.object_path(cid)?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, bytes)?;
        }
        Ok(ModelObject {
            cid: cid.to_owned(),
            bytes: bytes.len() as u64,
        })
    }
}

impl uor_r4_core::transformerless::region_store::RegionResolver for ModelStore {
    fn resolve(
        &self,
        kappa: &str,
    ) -> Option<uor_r4_core::transformerless::region_store::RegionObject> {
        // Total resolver: a backend/IO failure or a miss both resolve to `None`.
        self.get_region_object(kappa).ok().flatten()
    }
}

fn is_compiled_bundle(path: &Path) -> bool {
    path.is_dir()
        && ["tless_artifacts.bin", "tless_store.bin", "tokenizer.bin"]
            .iter()
            .all(|name| path.join(name).is_file())
}

fn address_container(bytes: &[u8]) -> String {
    let mut prefix = [0u8; 9];
    let prefix_len = cbor_byte_string_header(bytes.len(), &mut prefix);
    let mut hasher = blake3::Hasher::new();
    hasher.update(&prefix[..prefix_len]);
    hasher.update(bytes);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn cbor_byte_string_header(length: usize, out: &mut [u8; 9]) -> usize {
    if length < 24 {
        out[0] = 0x40 | length as u8;
        1
    } else if length < 256 {
        out[0] = 0x58;
        out[1] = length as u8;
        2
    } else if length < 65_536 {
        out[0] = 0x59;
        out[1..3].copy_from_slice(&(length as u16).to_be_bytes());
        3
    } else if u32::try_from(length).is_ok() {
        out[0] = 0x5a;
        out[1..5].copy_from_slice(&(length as u32).to_be_bytes());
        5
    } else {
        out[0] = 0x5b;
        out[1..9].copy_from_slice(&(length as u64).to_be_bytes());
        9
    }
}

fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn latest_descriptor_name(directory: &Path) -> Option<String> {
    std::fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            let name = entry.path().file_stem()?.to_str()?.to_owned();
            Some((modified, name))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, name)| name)
}

/// A pinned open-weight model source used only by offline compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDownload {
    pub repository: String,
    pub revision: String,
    pub name: String,
    /// Destination directory. When omitted, uses
    /// `<model-store>/sources/<name>`.
    pub output: Option<PathBuf>,
    /// SPDX license identifier recorded in the #597 source-snapshot
    /// manifest, when the caller knows it (e.g. from a pinned
    /// `models/*.json` descriptor). `None` leaves the manifest's license
    /// field null; the license *file* is digested either way.
    pub license: Option<String>,
}

/// Download a pinned model source into the local compiler-input cache.
///
/// This function is intentionally absent from `ask`; the native HTTP server
/// exposes it only through an explicit user-triggered download job. It invokes
/// the `hf` CLI without a shell, so repository and revision values are passed
/// as opaque arguments rather than executable text.
pub fn download_source(source: &SourceDownload) -> Result<PathBuf, ModelError> {
    let name = portable_source_name(&source.name)?;
    if !valid_repository(&source.repository) {
        return Err(ModelError::InvalidRepository(source.repository.clone()));
    }
    if source.revision.len() != 40 || !source.revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ModelError::UnpinnedRevision(source.revision.clone()));
    }
    let destination = source
        .output
        .clone()
        .unwrap_or_else(|| ModelStore::from_env().root.join("sources").join(name));
    std::fs::create_dir_all(&destination)?;
    eprintln!("download: {}@{}", source.repository, &source.revision[..12]);
    eprintln!("destination: {}", destination.display());
    eprintln!("starting Hugging Face download...");
    let status = run_download(build_download_command(source, &destination), &destination)?;
    if !status.success() {
        return Err(ModelError::DownloadFailed(status.code()));
    }
    let stats = directory_stats(&destination);
    eprintln!(
        "download complete: {} files, {}",
        stats.files,
        ByteCount(stats.bytes)
    );
    // #597: bind the whole admitted snapshot in one canonical,
    // schema-versioned manifest with a root κ.
    let manifest = build_source_manifest(
        &destination,
        &SourceSnapshotInfo {
            repository: source.repository.clone(),
            revision: source.revision.clone(),
            license: source.license.clone(),
            source_execution_mode: SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT.to_owned(),
        },
    )?;
    let manifest_kappa = write_source_manifest(&destination, &manifest)?;
    eprintln!(
        "source manifest: {SOURCE_MANIFEST_FILE_NAME} ({} admitted files), root κ {manifest_kappa}",
        manifest.files.len()
    );
    Ok(destination)
}

fn build_download_command(source: &SourceDownload, destination: &Path) -> Command {
    let mut command = Command::new("hf");
    command
        .arg("download")
        .arg(&source.repository)
        .arg("--revision")
        .arg(&source.revision)
        .arg("--local-dir")
        .arg(destination)
        .args([
            "--include",
            "*.safetensors",
            "--include",
            "*.json",
            "--include",
            "*.model",
            "--include",
            "merges.txt",
            "--include",
            "LICENSE*",
            "--include",
            "README.md",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
}

fn run_download(mut command: Command, destination: &Path) -> Result<ExitStatus, ModelError> {
    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ModelError::DownloadToolMissing
        } else {
            ModelError::Io(error)
        }
    })?;
    let started = Instant::now();
    let mut last_report = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if last_report.elapsed() >= Duration::from_secs(2) {
            let stats = directory_stats(destination);
            eprintln!(
                "progress: downloaded {} files, {} ({}s elapsed)",
                stats.files,
                ByteCount(stats.bytes),
                started.elapsed().as_secs()
            );
            let _ = std::io::stderr().flush();
            last_report = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct DirectoryStats {
    files: u64,
    bytes: u64,
}

fn directory_stats(directory: &Path) -> DirectoryStats {
    let mut stats = DirectoryStats::default();
    accumulate_directory_stats(directory, &mut stats);
    stats
}

fn accumulate_directory_stats(directory: &Path, stats: &mut DirectoryStats) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            accumulate_directory_stats(&entry.path(), stats);
        } else if file_type.is_file() {
            stats.files = stats.files.saturating_add(1);
            if let Ok(metadata) = entry.metadata() {
                stats.bytes = stats.bytes.saturating_add(metadata.len());
            }
        }
    }
}

struct ByteCount(u64);

impl fmt::Display for ByteCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KIB: u64 = 1024;
        const MIB: u64 = KIB * 1024;
        const GIB: u64 = MIB * 1024;
        match self.0 {
            bytes if bytes >= GIB => write!(formatter, "{:.2} GiB", bytes as f64 / GIB as f64),
            bytes if bytes >= MIB => write!(formatter, "{:.2} MiB", bytes as f64 / MIB as f64),
            bytes if bytes >= KIB => write!(formatter, "{:.2} KiB", bytes as f64 / KIB as f64),
            bytes => write!(formatter, "{bytes} B"),
        }
    }
}

fn portable_source_name(name: &str) -> Result<String, ModelError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(ModelError::InvalidSourceName(name.to_owned()));
    }
    Ok(name.to_owned())
}

fn valid_repository(repository: &str) -> bool {
    let mut parts = repository.split('/');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(owner), Some(model), None) if valid_part(owner) && valid_part(model)
    )
}

// --------------------------------------------------------------------------
// #597: versioned source-snapshot manifest.
//
// One canonical, schema-versioned manifest binds ALL open-weight model
// semantics of a downloaded snapshot: repository, immutable revision,
// license, compiler/adapter version, source-execution mode, and every
// admitted file's path + byte length + blake3 digest. Its root κ is the
// canonical-JSON address of the manifest bytes
// (`uor_addr::json::address_blake3`), so the κ uniquely identifies the
// exact snapshot. Pre-#597 descriptor κs with
// `source_kappa_scope = "model.safetensors"` remain weight-only
// identities and are NOT relabeled.

/// One admitted file of a source snapshot: path relative to the snapshot
/// root (`/`-separated), byte length, and raw `blake3:<hex>` digest of
/// the file bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceManifestFile {
    pub path: String,
    pub bytes: u64,
    pub kappa: String,
}

/// The versioned source-snapshot manifest (#597). Field order is the
/// canonical serialization order; `files` is sorted by path byte order in
/// the canonical form. `license` is the SPDX identifier when the caller
/// knows it (`null` otherwise; the license *file* is always digested in
/// `files` when present).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceManifest {
    pub schema: String,
    pub repository: String,
    pub revision: String,
    pub license: Option<String>,
    pub compiler_version: String,
    pub source_execution_mode: String,
    pub files: Vec<SourceManifestFile>,
}

/// Snapshot-level provenance the directory walk cannot derive by itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSnapshotInfo {
    pub repository: String,
    pub revision: String,
    /// SPDX license identifier, when known to the caller.
    pub license: Option<String>,
    /// How the snapshot is executed; today always
    /// [`SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT`].
    pub source_execution_mode: String,
}

/// Whether a file name is admitted into the snapshot manifest. Mirrors
/// exactly what [`download_source`] admits: `*.safetensors` (including
/// sharded weights), `*.json` (config, tokenizer, chat template,
/// generation config, `model.safetensors.index.json`), `*.model` +
/// `merges.txt` (tokenizer files), `LICENSE*`, and `README*`. The
/// manifest never lists itself.
fn admitted_source_file(name: &str) -> bool {
    if name == SOURCE_MANIFEST_FILE_NAME {
        return false;
    }
    let extension = Path::new(name).extension().and_then(|ext| ext.to_str());
    matches!(extension, Some("safetensors" | "json" | "model"))
        || name == "merges.txt"
        || name.starts_with("LICENSE")
        || name.starts_with("README")
}

/// Build the source-snapshot manifest of a local snapshot directory by
/// walking its admitted files (hidden entries such as the `hf` CLI's
/// `.cache` metadata are skipped). Files are digested with raw blake3
/// and listed sorted by path byte order.
pub fn build_source_manifest(
    snapshot_dir: &Path,
    info: &SourceSnapshotInfo,
) -> Result<SourceManifest, ModelError> {
    let mut files = Vec::new();
    collect_admitted_files(snapshot_dir, snapshot_dir, &mut files)?;
    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(SourceManifest {
        schema: SOURCE_MANIFEST_SCHEMA.to_owned(),
        repository: info.repository.clone(),
        revision: info.revision.clone(),
        license: info.license.clone(),
        compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_execution_mode: info.source_execution_mode.clone(),
        files,
    })
}

fn collect_admitted_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<SourceManifestFile>,
) -> Result<(), ModelError> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(ModelError::NonPortableSnapshotPath(path));
        };
        if name.starts_with('.') {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_admitted_files(root, &path, files)?;
        } else if file_type.is_file() && admitted_source_file(name) {
            let relative = relative_manifest_path(root, &path)?;
            let (bytes, kappa) = file_length_and_kappa(&path)?;
            files.push(SourceManifestFile {
                path: relative,
                bytes,
                kappa,
            });
        }
    }
    Ok(())
}

fn relative_manifest_path(root: &Path, path: &Path) -> Result<String, ModelError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ModelError::NonPortableSnapshotPath(path.to_path_buf()))?;
    let mut portable = String::new();
    for component in relative.components() {
        let Some(part) = component.as_os_str().to_str() else {
            return Err(ModelError::NonPortableSnapshotPath(path.to_path_buf()));
        };
        if !portable.is_empty() {
            portable.push('/');
        }
        portable.push_str(part);
    }
    Ok(portable)
}

/// Streamed raw blake3 of one file's bytes plus its length, as
/// `blake3:<hex>` (per-file digests stay raw, unlike the manifest root κ).
fn file_length_and_kappa(path: &Path) -> Result<(u64, String), ModelError> {
    use std::io::Read as _;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65_536];
    let mut length = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        length += read as u64;
        hasher.update(&buffer[..read]);
    }
    Ok((length, format!("blake3:{}", hasher.finalize().to_hex())))
}

/// Canonical manifest bytes: stable field order (struct declaration
/// order, which `serde_json::to_vec` preserves), `files` sorted by path
/// byte order, no floats. Two manifests describing the same snapshot
/// serialize to identical bytes regardless of file insertion order.
pub fn canonical_source_manifest_bytes(manifest: &SourceManifest) -> Result<Vec<u8>, ModelError> {
    if manifest.schema != SOURCE_MANIFEST_SCHEMA {
        return Err(ModelError::UnsupportedSourceManifestSchema(
            manifest.schema.clone(),
        ));
    }
    let mut canonical = manifest.clone();
    canonical
        .files
        .sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(serde_json::to_vec(&canonical)?)
}

/// Root κ of a source-snapshot manifest: the canonical-JSON blake3
/// address (`uor_addr::json::address_blake3`) of the canonical manifest
/// bytes. This κ uniquely identifies the exact snapshot — any admitted
/// byte change, file addition/removal, or provenance change relabels it.
pub fn source_manifest_kappa(manifest: &SourceManifest) -> Result<String, ModelError> {
    let bytes = canonical_source_manifest_bytes(manifest)?;
    match uor_addr::json::address_blake3(&bytes) {
        Ok(outcome) => Ok(outcome.address.to_string()),
        // Unreachable for serde-produced JSON; kept to honor the
        // no-panic-on-recoverable-paths convention.
        Err(failure) => Err(ModelError::ManifestAddressing(format!("{failure:?}"))),
    }
}

/// Parse and schema-gate source-snapshot manifest bytes. Any `schema`
/// value other than [`SOURCE_MANIFEST_SCHEMA`] is rejected.
pub fn parse_source_manifest(bytes: &[u8]) -> Result<SourceManifest, ModelError> {
    let manifest: SourceManifest = serde_json::from_slice(bytes)?;
    if manifest.schema != SOURCE_MANIFEST_SCHEMA {
        return Err(ModelError::UnsupportedSourceManifestSchema(manifest.schema));
    }
    Ok(manifest)
}

/// Read the source-snapshot manifest of a snapshot directory.
pub fn read_source_manifest(snapshot_dir: &Path) -> Result<SourceManifest, ModelError> {
    parse_source_manifest(&std::fs::read(
        snapshot_dir.join(SOURCE_MANIFEST_FILE_NAME),
    )?)
}

/// Write the canonical manifest as `source_manifest.json` inside the
/// snapshot directory (the manifest excludes itself from its file list)
/// and return the manifest root κ.
pub fn write_source_manifest(
    snapshot_dir: &Path,
    manifest: &SourceManifest,
) -> Result<String, ModelError> {
    let bytes = canonical_source_manifest_bytes(manifest)?;
    let kappa = source_manifest_kappa(manifest)?;
    std::fs::write(snapshot_dir.join(SOURCE_MANIFEST_FILE_NAME), bytes)?;
    Ok(kappa)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(capability: ModelCapability, passed: bool) -> ModelManifest {
        let object = ModelObject {
            cid: format!("blake3:{}", "0".repeat(64)),
            bytes: 1,
        };
        ModelManifest {
            schema: MANIFEST_SCHEMA,
            name: "test".to_owned(),
            source_model: "test-source".to_owned(),
            capability,
            artifacts: object.clone(),
            store: object.clone(),
            tokenizer: object,
            evaluation_report: if capability == ModelCapability::InstructionChat {
                Some(ModelObject {
                    cid: format!("blake3:{}", "1".repeat(64)),
                    bytes: 1,
                })
            } else {
                None
            },
            quality: QualityAttestation {
                instruction_eval_passed: passed,
                grounded_answer_rate: 0.8,
                repetition_rate: 0.01,
            },
        }
    }

    #[test]
    fn chat_requires_capability_and_quality_attestation() {
        assert!(matches!(
            manifest(ModelCapability::Continuation, true).validate_for_chat(),
            Err(ModelError::NotChatCapable)
        ));
        assert!(matches!(
            manifest(ModelCapability::InstructionChat, false).validate_for_chat(),
            Err(ModelError::QualityGateFailed)
        ));
        assert!(manifest(ModelCapability::InstructionChat, true)
            .validate_for_chat()
            .is_ok());
    }

    #[test]
    fn names_are_portable_across_filesystems() {
        assert_eq!(safe_name("org/model:v1"), "org-model-v1");
        assert!(portable_source_name("smollm2-135m").is_ok());
        assert!(portable_source_name("../escape").is_err());
        assert!(valid_repository("org/model"));
        assert!(!valid_repository("https://example.com/model"));
    }

    #[test]
    fn download_command_is_pinned_filtered_and_streamed() {
        let source = SourceDownload {
            repository: "org/model".to_owned(),
            revision: "a".repeat(40),
            name: "model".to_owned(),
            output: None,
            license: None,
        };
        let command = build_download_command(&source, Path::new("models/model"));
        let arguments: Vec<_> = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();
        assert_eq!(command.get_program(), "hf");
        assert_eq!(arguments[0], "download");
        assert!(arguments
            .windows(2)
            .any(|pair| pair == ["--revision", &source.revision]));
        assert!(arguments
            .windows(2)
            .any(|pair| pair == ["--local-dir", "models/model"]));
        assert!(arguments
            .windows(2)
            .any(|pair| pair == ["--include", "*.safetensors"]));
    }

    #[test]
    fn byte_counts_are_readable() {
        assert_eq!(ByteCount(0).to_string(), "0 B");
        assert_eq!(ByteCount(1024).to_string(), "1.00 KiB");
        assert_eq!(ByteCount(1024 * 1024).to_string(), "1.00 MiB");
    }

    #[test]
    fn downloaded_source_is_distinguished_from_missing_manifest() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-downloaded-source-test-{}",
            std::process::id()
        ));
        let source = root.join("sources").join("downloaded-model");
        std::fs::create_dir_all(&source).unwrap();
        let error = ModelStore::new(&root)
            .read_manifest("downloaded-model")
            .unwrap_err();
        assert!(matches!(error, ModelError::SourceNotCompiled(path) if path == source));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn compiled_bundle_is_distinguished_from_downloaded_source() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-compiled-bundle-test-{}",
            std::process::id()
        ));
        let source = root.join("sources").join("compiled-model");
        let compiled = root.join("compiled").join("compiled-model");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&compiled).unwrap();
        for name in ["tless_artifacts.bin", "tless_store.bin", "tokenizer.bin"] {
            std::fs::write(compiled.join(name), []).unwrap();
        }
        let error = ModelStore::new(&root)
            .read_manifest("compiled-model")
            .unwrap_err();
        assert!(matches!(error, ModelError::CompiledNotImported(path) if path == compiled));
        std::fs::remove_dir_all(root).unwrap();
    }

    fn snapshot_info() -> SourceSnapshotInfo {
        SourceSnapshotInfo {
            repository: "org/model".to_owned(),
            revision: "a".repeat(40),
            license: Some("Apache-2.0".to_owned()),
            source_execution_mode: SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT.to_owned(),
        }
    }

    /// A manifest value constructed directly (no directory walk).
    fn in_memory_manifest(files: Vec<SourceManifestFile>) -> SourceManifest {
        let info = snapshot_info();
        SourceManifest {
            schema: SOURCE_MANIFEST_SCHEMA.to_owned(),
            repository: info.repository,
            revision: info.revision,
            license: info.license,
            compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
            source_execution_mode: info.source_execution_mode,
            files,
        }
    }

    /// A unique temp snapshot directory populated with one file per
    /// admitted downloader pattern plus two files that must be excluded.
    fn snapshot_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-source-manifest-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("nested")).unwrap();
        for (name, contents) in [
            ("model.safetensors", "weights"),
            ("model.safetensors.index.json", "{\"weight_map\":{}}"),
            ("config.json", "{\"hidden_size\":576}"),
            ("tokenizer.json", "{\"model\":{}}"),
            ("tokenizer.model", "sentencepiece"),
            ("merges.txt", "a b"),
            ("LICENSE", "Apache License 2.0"),
            ("README.md", "# model"),
            ("nested/extra.json", "{}"),
            ("notes.txt", "not admitted"),
            (".gitattributes", "hidden, not admitted"),
        ] {
            std::fs::write(dir.join(name), contents).unwrap();
        }
        dir
    }

    #[test]
    fn canonical_manifest_bytes_are_insertion_order_independent() {
        let file = |path: &str| SourceManifestFile {
            path: path.to_owned(),
            bytes: 1,
            kappa: format!("blake3:{}", "2".repeat(64)),
        };
        let forward =
            in_memory_manifest(vec![file("a.json"), file("b.safetensors"), file("z.model")]);
        let mut reversed = forward.clone();
        reversed.files.reverse();
        assert_eq!(
            canonical_source_manifest_bytes(&forward).unwrap(),
            canonical_source_manifest_bytes(&reversed).unwrap()
        );
        assert_eq!(
            source_manifest_kappa(&forward).unwrap(),
            source_manifest_kappa(&reversed).unwrap()
        );
    }

    #[test]
    fn manifest_covers_every_admitted_file_exactly_once_and_excludes_itself() {
        let dir = snapshot_dir("coverage");
        let manifest = build_source_manifest(&dir, &snapshot_info()).unwrap();
        write_source_manifest(&dir, &manifest).unwrap();
        // Rebuild AFTER the manifest file exists on disk: it must not
        // admit itself.
        let rebuilt = build_source_manifest(&dir, &snapshot_info()).unwrap();
        let paths: Vec<&str> = rebuilt.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "LICENSE",
                "README.md",
                "config.json",
                "merges.txt",
                "model.safetensors",
                "model.safetensors.index.json",
                "nested/extra.json",
                "tokenizer.json",
                "tokenizer.model",
            ],
            "sorted by path byte order, each admitted file exactly once"
        );
        assert!(!paths.contains(&SOURCE_MANIFEST_FILE_NAME));
        assert!(!paths.contains(&"notes.txt"));
        assert!(!paths.contains(&".gitattributes"));
        for file in &rebuilt.files {
            assert_eq!(
                file.bytes,
                std::fs::metadata(dir.join(&file.path)).unwrap().len()
            );
            assert!(file.kappa.starts_with("blake3:") && file.kappa.len() == 71);
        }
        assert_eq!(rebuilt, read_source_manifest(&dir).unwrap());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn double_build_is_deterministic_in_bytes_and_kappa() {
        let dir = snapshot_dir("determinism");
        let first = build_source_manifest(&dir, &snapshot_info()).unwrap();
        let second = build_source_manifest(&dir, &snapshot_info()).unwrap();
        assert_eq!(
            canonical_source_manifest_bytes(&first).unwrap(),
            canonical_source_manifest_bytes(&second).unwrap()
        );
        assert_eq!(
            source_manifest_kappa(&first).unwrap(),
            source_manifest_kappa(&second).unwrap()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn tampering_with_one_admitted_byte_changes_the_root_kappa() {
        let dir = snapshot_dir("tamper");
        let before =
            source_manifest_kappa(&build_source_manifest(&dir, &snapshot_info()).unwrap()).unwrap();
        let target = dir.join("model.safetensors");
        let mut bytes = std::fs::read(&target).unwrap();
        bytes[0] ^= 0x01;
        std::fs::write(&target, bytes).unwrap();
        let after =
            source_manifest_kappa(&build_source_manifest(&dir, &snapshot_info()).unwrap()).unwrap();
        assert_ne!(before, after);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn wrong_manifest_schema_is_rejected() {
        let mut manifest = in_memory_manifest(Vec::new());
        let bytes = canonical_source_manifest_bytes(&manifest).unwrap();
        assert!(parse_source_manifest(&bytes).is_ok());
        manifest.schema = "uor-r4-source-manifest/999".to_owned();
        let tampered = serde_json::to_vec(&manifest).unwrap();
        assert!(matches!(
            parse_source_manifest(&tampered),
            Err(ModelError::UnsupportedSourceManifestSchema(schema))
                if schema == "uor-r4-source-manifest/999"
        ));
        assert!(matches!(
            canonical_source_manifest_bytes(&manifest),
            Err(ModelError::UnsupportedSourceManifestSchema(_))
        ));
    }

    #[test]
    fn model_store_resolves_region_objects_from_manifest_only() {
        use std::collections::BTreeMap;
        use uor_r4_core::transformerless::region_store::{
            export_store, predict_witness_with_resolver, RegionResolver,
        };
        use uor_r4_core::transformerless::runtime::{predict_witness_plain, Store};

        let root =
            std::env::temp_dir().join(format!("uor-r4-region-store-test-{}", std::process::id()));
        let mut store: Store = (0..=4).map(|_| BTreeMap::new()).collect();
        store[0].insert(vec![], BTreeMap::from([(7, 2), (9, 1)]));
        store[1].insert(vec![1], BTreeMap::from([(11, 4), (12, 3)]));
        store[2].insert(vec![1, 2], BTreeMap::from([(21, 5)]));

        let export = export_store(&store).expect("region export");
        let model_store = ModelStore::new(&root);
        for object in &export.objects {
            model_store
                .put_region_object(object)
                .expect("region object write");
        }
        model_store
            .put_region_manifest(&export.manifest)
            .expect("manifest write");

        let manifest = model_store
            .get_region_manifest(&export.manifest.manifest_kappa)
            .expect("manifest read")
            .expect("manifest exists");
        let code = [1, 2, 3, 4];
        let expected = predict_witness_plain(&store, &code);
        let actual = predict_witness_with_resolver(&model_store, &manifest, &code)
            .expect("resolver prediction");
        assert_eq!(actual, expected);
        assert!(
            <ModelStore as RegionResolver>::resolve(&model_store, &manifest.regions[0].kappa)
                .is_some()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
