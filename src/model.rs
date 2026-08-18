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

/// Select the default chat model from the descriptors in `models/`.
///
/// `TLESS_MODEL` always wins. Discovery considers only chat-capable
/// descriptors (declaring `architecture` + `weight_format`; tokenizer-only
/// pins are skipped), prefers one whose compiled bundle exists locally,
/// then newest, then name (#790). The static default is used when
/// discovery is unavailable, such as when a binary runs outside the
/// repository checkout.
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

/// Whether an imported bundle may serve `r4 ask`, and how it earned that.
///
/// Before `#744`, these three fields were plain CLI floats on `r4 import`
/// (`--instruction-eval-passed`, `--grounded-answer-rate`,
/// `--repetition-rate`) — an operator-typed attestation with no
/// connection to any computation. Two manifests over byte-identical
/// compiled artifacts could (and did) carry opposite numbers, both
/// "passing". [`evaluate_live_quality`] replaces that: for
/// instruction-chat imports, these fields are now derived from actually
/// running the exact bytes being imported against a fixed probe set
/// through the same generation path `r4 ask` uses (see
/// `chat::engine_from_bytes`), not accepted as operator input.
///
/// `grounded_answer_rate` measures **non-degeneracy**, not semantic fact-
/// checking — this codebase has no mechanism to verify an answer's
/// factual content, so the honest thing is not to claim it does. A probe
/// counts toward `grounded_answer_rate` when it produces a non-empty
/// answer whose [`repeated_token_rate`](crate::chat::ChatAnswer::repeated_token_rate)
/// stays at or below [`LIVE_QUALITY_REPETITION_BAR`] — i.e. the answer is
/// not empty and not dominated by recently-repeated tokens, the failure
/// mode observed in `#745`'s word-salad transcripts.
///
/// `grounded_answer_rate`/`repetition_rate` are always the **plain
/// TLA/TLS1 path**'s results, the baseline every bundle must clear
/// regardless of which runtime ultimately serves it. `#750`: when the
/// import also supplies an R4G1 graph (`--r4g1`), that path is probed
/// too and its results land in `r4g1_grounded_answer_rate`/
/// `r4g1_repetition_rate` — `None` when no R4G1 graph was supplied,
/// distinct from a probed-and-degenerate `Some(0.0)`.
/// `instruction_eval_passed` requires **both** paths to independently
/// clear the bar when both were probed (see `combine_path_quality`) —
/// `ask`/`chat` will actually serve whichever path is available at
/// runtime for a given manifest name, so a pass on one path cannot
/// paper over a fail on the other.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityAttestation {
    pub instruction_eval_passed: bool,
    pub grounded_answer_rate: f32,
    pub repetition_rate: f32,
    #[serde(default)]
    pub r4g1_grounded_answer_rate: Option<f32>,
    #[serde(default)]
    pub r4g1_repetition_rate: Option<f32>,
}

/// Fixed probe set for [`evaluate_live_quality`] (`#744`): ordinary
/// questions with no single expected answer string, used only to check
/// that generation is non-degenerate — not to verify factual
/// correctness, which nothing in this codebase currently checks. Kept
/// small and fixed so the gate stays fast and its verdict reproducible
/// run to run on the same bytes.
const LIVE_QUALITY_PROBES: &[&str] = &[
    "What is the capital of France?",
    "Why is the sky blue?",
    "What is two plus two?",
    "Name a primary color.",
    "What day comes after Monday?",
    "What is water made of?",
    "How many legs does a dog have?",
    "What is the opposite of hot?",
];

/// A probe answer counts as non-degenerate when its full-generation
/// repetition rate ([`chat::ChatAnswer::repeated_token_rate`]) stays at
/// or below this bar.
///
/// Calibrated empirically against the real `#745` bundle
/// (`smollm2-1-7b-instruct`, artifact `blake3:fca5bdfb…` — the same
/// bytes the `#655` sweep found producing word-salad): its actual probe
/// answers measured 0.31-0.41 on this metric, well above ordinary
/// short-answer word reuse. `0.25` catches that entire observed range
/// with margin.
///
/// Honest limitation, stated plainly: this bar is calibrated only
/// against known-*bad* evidence. No genuinely coherent local bundle
/// exists yet to confirm it does not also reject good output — that
/// positive calibration point does not exist until `#745` produces one.
/// Revisit this constant against real positive evidence once it does,
/// rather than treating `0.25` as load-bearing precision.
const LIVE_QUALITY_REPETITION_BAR: f64 = 0.25;

/// An imported bundle passes when at least this fraction of the probe
/// set produces a non-degenerate answer. A bar below `1.0` tolerates an
/// occasional empty/degenerate probe without failing an otherwise-working
/// bundle outright; `0.5` requires a majority.
const LIVE_QUALITY_PASS_BAR: f32 = 0.5;

/// One probe's live-generation outcome, reduced to just what the gate
/// needs. Kept separate from `chat::ChatAnswer`/`ChatError` so
/// [`aggregate_probe_outcomes`] — the actual pass/fail policy — can be
/// exercised by a fast, deterministic unit test (`#744`'s falsifier)
/// without loading a real compiled bundle.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProbeOutcome {
    /// Generation produced some text.
    Answered {
        non_empty: bool,
        repeated_token_rate: f64,
    },
    /// `ask` returned an error (empty/repetitive generation, or a load
    /// failure) — scored as the worst case (maximal repetition) so it
    /// cannot be diluted by averaging with healthier probes.
    Failed,
}

/// One code path's reduced probe results, before combination across
/// paths (`#750`). Kept separate from the final [`QualityAttestation`]
/// so [`combine_path_quality`] — the actual cross-path pass/fail policy
/// — can be exercised by a fast, deterministic unit test without
/// loading a real compiled bundle, the same discipline
/// `aggregate_probe_outcomes`'s `#744` falsifier already established.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PathQuality {
    passed: bool,
    grounded_answer_rate: f32,
    repetition_rate: f32,
}

fn aggregate_probe_outcomes(outcomes: &[ProbeOutcome]) -> PathQuality {
    debug_assert!(!outcomes.is_empty(), "probe set must be non-empty");
    let mut non_degenerate = 0usize;
    let mut repetition_sum = 0.0f64;
    for outcome in outcomes {
        match outcome {
            ProbeOutcome::Answered {
                non_empty: true,
                repeated_token_rate,
            } if *repeated_token_rate <= LIVE_QUALITY_REPETITION_BAR => {
                non_degenerate += 1;
                repetition_sum += repeated_token_rate;
            }
            ProbeOutcome::Answered {
                repeated_token_rate,
                ..
            } => repetition_sum += repeated_token_rate,
            ProbeOutcome::Failed => repetition_sum += 1.0,
        }
    }
    let probe_count = outcomes.len().max(1);
    let grounded_answer_rate = non_degenerate as f32 / probe_count as f32;
    let repetition_rate = (repetition_sum / probe_count as f64) as f32;
    PathQuality {
        passed: grounded_answer_rate >= LIVE_QUALITY_PASS_BAR,
        grounded_answer_rate,
        repetition_rate,
    }
}

/// Combine the plain-path and (when a bundle carries a `compiled.r4g1`
/// graph) R4G1-path probe results into the manifest's final
/// [`QualityAttestation`] (`#750`). `ask`/`chat` will actually serve
/// whichever path is available at runtime for a given manifest name, so
/// silently averaging the two paths, or taking whichever is better,
/// would let a bundle that is degenerate under one of them still pass —
/// both paths must independently clear the bar when both were probed.
fn combine_path_quality(plain: PathQuality, r4g1: Option<PathQuality>) -> QualityAttestation {
    QualityAttestation {
        instruction_eval_passed: plain.passed && r4g1.is_none_or(|path| path.passed),
        grounded_answer_rate: plain.grounded_answer_rate,
        repetition_rate: plain.repetition_rate,
        r4g1_grounded_answer_rate: r4g1.map(|path| path.grounded_answer_rate),
        r4g1_repetition_rate: r4g1.map(|path| path.repetition_rate),
    }
}

/// Run [`LIVE_QUALITY_PROBES`] against the exact bytes being imported,
/// through the plain TLA/TLS1 path (`r4g1_bytes: None`) or through the
/// R4G1 beam-search path when `r4g1_bytes` is supplied, and reduce the
/// results to a [`PathQuality`] (`#744`, path parameter added `#750`).
/// Fails fast with a real error if the bytes cannot even be loaded,
/// rather than scoring repeated identical load failures into the
/// metrics below — an unloadable bundle is a defect in the import
/// inputs, not a generation-quality signal for this gate to describe.
///
/// A fresh engine is built per probe: the probes are independent
/// ordinary questions, not turns of one conversation, so no chat history
/// should carry between them.
fn run_live_quality_probe_set(
    artifact_bytes: &[u8],
    store_bytes: &[u8],
    tokenizer_bytes: &[u8],
    r4g1_bytes: Option<&[u8]>,
) -> Result<PathQuality, ModelError> {
    const PROBE_MAX_TOKENS: usize = 64;
    crate::chat::engine_from_bytes_with_r4g1(
        artifact_bytes,
        store_bytes,
        tokenizer_bytes,
        r4g1_bytes,
        PROBE_MAX_TOKENS,
    )
    .map_err(|error| ModelError::Io(std::io::Error::other(error.to_string())))?;

    let mut outcomes = Vec::with_capacity(LIVE_QUALITY_PROBES.len());
    for probe in LIVE_QUALITY_PROBES {
        let mut engine = crate::chat::engine_from_bytes_with_r4g1(
            artifact_bytes,
            store_bytes,
            tokenizer_bytes,
            r4g1_bytes,
            PROBE_MAX_TOKENS,
        )
        .map_err(|error| ModelError::Io(std::io::Error::other(error.to_string())))?;
        outcomes.push(match engine.ask(probe) {
            Ok(answer) => ProbeOutcome::Answered {
                non_empty: !answer.text.trim().is_empty(),
                repeated_token_rate: answer.repeated_token_rate,
            },
            Err(_) => ProbeOutcome::Failed,
        });
    }
    Ok(aggregate_probe_outcomes(&outcomes))
}

/// Derive a [`QualityAttestation`] for the exact bytes being imported
/// (`#744`). When `r4g1_bytes` is supplied (`#750`, `r4 import --r4g1
/// <path>`), the probe set is run a second time through the R4G1
/// beam-search path and both paths must independently pass — see
/// [`combine_path_quality`] — since `ask`/`chat` will transparently
/// prefer an R4G1 graph over the plain path at serving time when one
/// exists at the manifest-name-keyed convention path, and the two paths
/// have been observed to diverge sharply on the same underlying bytes
/// (one bundle: `"cut cut cut ..."` under R4G1 vs. word-salad on the
/// plain path).
pub fn evaluate_live_quality(
    artifact_bytes: &[u8],
    store_bytes: &[u8],
    tokenizer_bytes: &[u8],
    r4g1_bytes: Option<&[u8]>,
) -> Result<QualityAttestation, ModelError> {
    let plain = run_live_quality_probe_set(artifact_bytes, store_bytes, tokenizer_bytes, None)?;
    let r4g1 = match r4g1_bytes {
        Some(bytes) => Some(run_live_quality_probe_set(
            artifact_bytes,
            store_bytes,
            tokenizer_bytes,
            Some(bytes),
        )?),
        None => None,
    };
    Ok(combine_path_quality(plain, r4g1))
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
        if let Some(rate) = self.quality.r4g1_grounded_answer_rate {
            if !(0.0..=1.0).contains(&rate) {
                return Err(ModelError::InvalidQualityMetrics);
            }
        }
        if let Some(rate) = self.quality.r4g1_repetition_rate {
            if !(0.0..=1.0).contains(&rate) {
                return Err(ModelError::InvalidQualityMetrics);
            }
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
    // #790 item 3: only chat-capable model descriptors participate in
    // default-model discovery. The mtime-newest-json rule used to pick the
    // tokenizer-only `t5-base-tokenizer.json`, breaking bare `r4 ask` with
    // "compiled model manifest 't5-base-tokenizer' was not found". A
    // descriptor is chat-capable when it declares both `architecture` and
    // `weight_format`; tokenizer pins declare neither. Newest still wins,
    // with the name as a deterministic tiebreak (a fresh clone gives every
    // descriptor the same mtime, which made the old pick arbitrary).
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
            let path = entry.path();
            let descriptor: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
            if descriptor.get("architecture").is_none() || descriptor.get("weight_format").is_none()
            {
                return None;
            }
            let modified = entry.metadata().ok()?.modified().ok()?;
            let name = path.file_stem()?.to_str()?.to_owned();
            // A descriptor whose bundle is actually compiled locally beats
            // any uncompiled one: bare `r4 ask` should reach a servable
            // model when one exists rather than erroring on the newest pin.
            let compiled = ModelStore::from_env()
                .root
                .join("compiled")
                .join(safe_name(&name))
                .join("tless_artifacts.bin")
                .is_file();
            Some((compiled, modified, name))
        })
        .max()
        .map(|(_, _, name)| name)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceTreeScope {
    ManifestAdmitted,
    ManifestlessAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VerifiedSourceTreeEntry {
    Directory {
        path: String,
    },
    File {
        path: String,
        bytes: u64,
        kappa: String,
    },
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
    let mut files = verified_source_tree(snapshot_dir, SourceTreeScope::ManifestAdmitted)?
        .into_iter()
        .filter_map(|entry| match entry {
            VerifiedSourceTreeEntry::File { path, bytes, kappa } => {
                Some(SourceManifestFile { path, bytes, kappa })
            }
            VerifiedSourceTreeEntry::Directory { .. } => None,
        })
        .collect::<Vec<_>>();
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

pub(crate) fn verified_source_tree(
    snapshot_dir: &Path,
    scope: SourceTreeScope,
) -> Result<Vec<VerifiedSourceTreeEntry>, ModelError> {
    verified_source_tree_with_hook(snapshot_dir, scope, |_| {})
}

#[cfg(unix)]
pub(crate) fn verified_source_tree_with_hook<F>(
    snapshot_dir: &Path,
    scope: SourceTreeScope,
    mut before_open: F,
) -> Result<Vec<VerifiedSourceTreeEntry>, ModelError>
where
    F: FnMut(&Path),
{
    let root = open_snapshot_directory_nofollow(snapshot_dir)?;
    let root_metadata = root.metadata()?;
    let mut entries = Vec::new();
    walk_source_tree_directory(
        snapshot_dir,
        &root,
        "",
        scope,
        &mut before_open,
        &mut entries,
    )?;
    verify_opened_snapshot_entry(snapshot_dir, &root_metadata)?;
    entries.sort_by(|left, right| {
        verified_source_tree_entry_path(left)
            .as_bytes()
            .cmp(verified_source_tree_entry_path(right).as_bytes())
    });
    Ok(entries)
}

#[cfg(not(unix))]
pub(crate) fn verified_source_tree_with_hook<F>(
    snapshot_dir: &Path,
    _scope: SourceTreeScope,
    _before_open: F,
) -> Result<Vec<VerifiedSourceTreeEntry>, ModelError>
where
    F: FnMut(&Path),
{
    Err(ModelError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "handle-bound source snapshot traversal is unavailable on this platform: {}",
            snapshot_dir.display()
        ),
    )))
}

fn verified_source_tree_entry_path(entry: &VerifiedSourceTreeEntry) -> &str {
    match entry {
        VerifiedSourceTreeEntry::Directory { path }
        | VerifiedSourceTreeEntry::File { path, .. } => path,
    }
}

#[cfg(unix)]
fn open_snapshot_directory_nofollow(path: &Path) -> Result<std::fs::File, ModelError> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = std::fs::OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    let directory = options.open(path)?;
    if !directory.metadata()?.file_type().is_dir() {
        return Err(ModelError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "source snapshot {} is not a regular non-symlink directory",
                path.display()
            ),
        )));
    }
    Ok(directory)
}

#[cfg(unix)]
fn same_snapshot_inode(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(unix)]
fn same_snapshot_metadata_version(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    same_snapshot_inode(left, right)
        && left.mode() == right.mode()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn source_tree_errno_pointer() -> *mut libc::c_int {
    // SAFETY: caller uses the target libc's thread-local errno accessor.
    unsafe { libc::__errno_location() }
}

#[cfg(target_vendor = "apple")]
unsafe fn source_tree_errno_pointer() -> *mut libc::c_int {
    // SAFETY: caller uses the target libc's thread-local errno accessor.
    unsafe { libc::__error() }
}

#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
fn read_opened_directory_names(_directory: &std::fs::File) -> Result<Vec<String>, ModelError> {
    Err(ModelError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "handle-bound directory enumeration is not implemented for this Unix target",
    )))
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn read_opened_directory_names(directory: &std::fs::File) -> Result<Vec<String>, ModelError> {
    use std::os::fd::AsRawFd;

    // SAFETY: `fcntl` duplicates the live directory descriptor. Ownership of
    // the duplicate is transferred to `fdopendir` below.
    let duplicate = unsafe { libc::fcntl(directory.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(ModelError::Io(std::io::Error::last_os_error()));
    }
    // SAFETY: `duplicate` is a fresh owned directory descriptor.
    let stream = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        // SAFETY: fdopendir did not take ownership on failure.
        unsafe { libc::close(duplicate) };
        return Err(ModelError::Io(std::io::Error::last_os_error()));
    }
    struct DirectoryStream(*mut libc::DIR);
    impl Drop for DirectoryStream {
        fn drop(&mut self) {
            // SAFETY: this guard uniquely owns the DIR pointer.
            unsafe { libc::closedir(self.0) };
        }
    }
    let stream = DirectoryStream(stream);
    let mut names = Vec::new();
    loop {
        // SAFETY: the target accessor returns this thread's errno cell.
        unsafe { *source_tree_errno_pointer() = 0 };
        // SAFETY: `stream` stays live and uniquely owned during enumeration.
        let entry = unsafe { libc::readdir(stream.0) };
        if entry.is_null() {
            // SAFETY: the target accessor returns this thread's errno cell.
            let errno = unsafe { *source_tree_errno_pointer() };
            if errno != 0 {
                return Err(ModelError::Io(std::io::Error::from_raw_os_error(errno)));
            }
            break;
        }
        // SAFETY: POSIX dirent d_name is NUL-terminated for a successful
        // readdir result and remains valid until the next readdir call.
        let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| ModelError::NonPortableSnapshotPath(PathBuf::from("<non-utf8>")))?;
        if name != "." && name != ".." {
            names.push(name.to_owned());
        }
    }
    names.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(names)
}

#[cfg(unix)]
fn verify_opened_snapshot_entry(
    path: &Path,
    opened_metadata: &std::fs::Metadata,
) -> Result<(), ModelError> {
    let current = std::fs::symlink_metadata(path)?;
    if current.file_type().is_symlink()
        || !same_snapshot_metadata_version(&current, opened_metadata)
        || current.file_type().is_dir() != opened_metadata.file_type().is_dir()
        || current.file_type().is_file() != opened_metadata.file_type().is_file()
    {
        return Err(ModelError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "source snapshot entry {} changed identity or type during handle-bound traversal",
                path.display()
            ),
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn open_snapshot_child_nofollow(
    parent: &std::fs::File,
    name: &str,
    display_path: &Path,
) -> Result<std::fs::File, ModelError> {
    use std::os::fd::{AsRawFd, FromRawFd};

    let name = std::ffi::CString::new(name)
        .map_err(|_| ModelError::NonPortableSnapshotPath(display_path.to_path_buf()))?;
    // SAFETY: `parent` and `name` remain live for the call, the C string is
    // NUL-terminated, and a successful descriptor is transferred exactly
    // once into `File` below.
    let descriptor = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        )
    };
    if descriptor < 0 {
        let error = std::io::Error::last_os_error();
        return Err(ModelError::Io(std::io::Error::new(
            error.kind(),
            format!(
                "source snapshot entry {} cannot be opened without following links: {}",
                display_path.display(),
                error
            ),
        )));
    }
    // SAFETY: `openat` returned a new owned descriptor and no other owner is
    // constructed for it.
    Ok(unsafe { std::fs::File::from_raw_fd(descriptor) })
}

#[cfg(unix)]
fn verify_opened_snapshot_child(
    parent: &std::fs::File,
    name: &str,
    display_path: &Path,
    opened_metadata: &std::fs::Metadata,
) -> Result<(), ModelError> {
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;

    let name = std::ffi::CString::new(name)
        .map_err(|_| ModelError::NonPortableSnapshotPath(display_path.to_path_buf()))?;
    let mut status = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: parent/name are live, status points to writable storage, and
    // AT_SYMLINK_NOFOLLOW makes the final component itself authoritative.
    let result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            name.as_ptr(),
            status.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        return Err(ModelError::Io(std::io::Error::new(
            error.kind(),
            format!(
                "source snapshot entry {} cannot be rebound after open: {}",
                display_path.display(),
                error
            ),
        )));
    }
    // SAFETY: fstatat initialized status on success.
    let status = unsafe { status.assume_init() };
    let kind = status.st_mode & libc::S_IFMT;
    let expected_kind = if opened_metadata.file_type().is_dir() {
        libc::S_IFDIR
    } else if opened_metadata.file_type().is_file() {
        libc::S_IFREG
    } else {
        0
    };
    // `libc::dev_t` is already `u64` on Linux but is narrower on Darwin;
    // normalize the platform ABI before comparing it with `MetadataExt::dev`.
    #[allow(clippy::unnecessary_cast)]
    let status_device = status.st_dev as u64;
    if kind != expected_kind
        || status_device != opened_metadata.dev()
        || status.st_ino != opened_metadata.ino()
    {
        return Err(ModelError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "source snapshot entry {} changed identity or type during handle-bound traversal",
                display_path.display()
            ),
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn walk_source_tree_directory<F>(
    snapshot_root: &Path,
    directory: &std::fs::File,
    relative_directory: &str,
    scope: SourceTreeScope,
    before_open: &mut F,
    output: &mut Vec<VerifiedSourceTreeEntry>,
) -> Result<(), ModelError>
where
    F: FnMut(&Path),
{
    use std::io::Read as _;

    let names = read_opened_directory_names(directory)?;

    for name in names {
        if scope == SourceTreeScope::ManifestAdmitted && name.starts_with('.') {
            continue;
        }
        let relative = if relative_directory.is_empty() {
            name.clone()
        } else {
            format!("{relative_directory}/{name}")
        };
        let display_path = snapshot_root.join(&relative);
        before_open(&display_path);
        let mut child = open_snapshot_child_nofollow(directory, &name, &display_path)?;
        let opened_metadata = child.metadata()?;
        verify_opened_snapshot_child(directory, &name, &display_path, &opened_metadata)?;

        if opened_metadata.file_type().is_dir() {
            if scope == SourceTreeScope::ManifestlessAll && name == ".cache" {
                continue;
            }
            if scope == SourceTreeScope::ManifestlessAll {
                output.push(VerifiedSourceTreeEntry::Directory {
                    path: relative.clone(),
                });
            }
            walk_source_tree_directory(
                snapshot_root,
                &child,
                &relative,
                scope,
                before_open,
                output,
            )?;
            let final_metadata = child.metadata()?;
            if !same_snapshot_metadata_version(&opened_metadata, &final_metadata) {
                return Err(ModelError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "source snapshot directory {} changed while being traversed",
                        display_path.display()
                    ),
                )));
            }
            verify_opened_snapshot_child(directory, &name, &display_path, &final_metadata)?;
            continue;
        }
        if !opened_metadata.file_type().is_file() {
            return Err(ModelError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "source snapshot entry {} is not a regular non-symlink file or directory",
                    display_path.display()
                ),
            )));
        }
        if scope == SourceTreeScope::ManifestlessAll && name == ".cache" {
            return Err(ModelError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "source snapshot entry {} uses reserved .cache transport name but is not a directory",
                    display_path.display()
                ),
            )));
        }
        if scope == SourceTreeScope::ManifestAdmitted && !admitted_source_file(&name) {
            verify_opened_snapshot_child(directory, &name, &display_path, &opened_metadata)?;
            continue;
        }

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 65_536];
        let mut length = 0u64;
        loop {
            let read = child.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            length = length.checked_add(read as u64).ok_or_else(|| {
                ModelError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "source snapshot entry {} is too large",
                        display_path.display()
                    ),
                ))
            })?;
            hasher.update(&buffer[..read]);
        }
        let final_metadata = child.metadata()?;
        if !same_snapshot_metadata_version(&opened_metadata, &final_metadata)
            || !final_metadata.file_type().is_file()
            || final_metadata.len() != length
        {
            return Err(ModelError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "source snapshot entry {} changed while being digested",
                    display_path.display()
                ),
            )));
        }
        verify_opened_snapshot_child(directory, &name, &display_path, &final_metadata)?;
        output.push(VerifiedSourceTreeEntry::File {
            path: relative,
            bytes: length,
            kappa: format!("blake3:{}", hasher.finalize().to_hex()),
        });
    }
    Ok(())
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

    /// #790 item 3 falsifier: a tokenizer-only descriptor must never win
    /// default-model discovery, even when it is the mtime-newest json —
    /// the pre-fix rule picked `t5-base-tokenizer` and broke bare
    /// `r4 ask` ("compiled model manifest 't5-base-tokenizer' was not
    /// found", verified live in the 2026-08-18 audit).
    #[test]
    fn default_model_discovery_skips_tokenizer_only_descriptors() {
        let dir = std::env::temp_dir().join("uor-r4-790-descriptor-pick");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture dir");
        std::fs::write(
            dir.join("aaa-chat-model.json"),
            r#"{"architecture":"LlamaForCausalLM","weight_format":"safetensors"}"#,
        )
        .expect("chat descriptor");
        // Written second, so it is at least as new as the chat descriptor.
        std::fs::write(
            dir.join("zzz-tokenizer-only.json"),
            r#"{"tokenizer_family":"sentencepiece-unigram"}"#,
        )
        .expect("tokenizer descriptor");
        assert_eq!(
            latest_descriptor_name(&dir).as_deref(),
            Some("aaa-chat-model")
        );
        // Deterministic across repeated calls.
        assert_eq!(latest_descriptor_name(&dir), latest_descriptor_name(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Falsifier for `#744`'s live quality gate: demonstrates
    /// `aggregate_probe_outcomes` correctly rejects the two known-bad
    /// cases the `#655` coherence sweep actually observed (all-empty
    /// output, and small-vocabulary word-salad), and accepts a
    /// genuinely healthy probe set. Exercises the pass/fail policy
    /// directly, without loading a real compiled bundle, so it stays
    /// fast and deterministic in CI.
    #[test]
    fn live_quality_gate_rejects_known_bad_probe_sets_and_accepts_a_healthy_one() {
        let probe_count = LIVE_QUALITY_PROBES.len();

        // Every probe fails outright -- mirrors `smollm2-135m-instruct`'s
        // observed empty `<|im_end|>` output in the #655 sweep.
        let all_failed = vec![ProbeOutcome::Failed; probe_count];
        let quality = aggregate_probe_outcomes(&all_failed);
        assert!(!quality.passed);
        assert_eq!(quality.grounded_answer_rate, 0.0);
        assert_eq!(quality.repetition_rate, 1.0);

        // Every probe "answers" but collapses into a small repeated-token
        // cycle -- mirrors the #745 word-salad transcripts.
        let all_degenerate = vec![
            ProbeOutcome::Answered {
                non_empty: true,
                repeated_token_rate: 0.9,
            };
            probe_count
        ];
        let quality = aggregate_probe_outcomes(&all_degenerate);
        assert!(!quality.passed);
        assert_eq!(quality.grounded_answer_rate, 0.0);
        assert!((quality.repetition_rate - 0.9).abs() < 1e-6);

        // A genuinely healthy probe set passes.
        let all_healthy = vec![
            ProbeOutcome::Answered {
                non_empty: true,
                repeated_token_rate: 0.1,
            };
            probe_count
        ];
        let quality = aggregate_probe_outcomes(&all_healthy);
        assert!(quality.passed);
        assert_eq!(quality.grounded_answer_rate, 1.0);

        // Confirm the documented 0.5 pass bar is exactly where it's
        // declared to be: exactly half non-degenerate passes (`>=`), one
        // probe fewer does not.
        let outcomes_with_healthy_count = |healthy: usize| -> Vec<ProbeOutcome> {
            (0..probe_count)
                .map(|i| {
                    if i < healthy {
                        ProbeOutcome::Answered {
                            non_empty: true,
                            repeated_token_rate: 0.1,
                        }
                    } else {
                        ProbeOutcome::Failed
                    }
                })
                .collect()
        };
        let half = probe_count / 2;
        let at_bar = aggregate_probe_outcomes(&outcomes_with_healthy_count(half));
        assert!(
            at_bar.passed,
            "exactly half non-degenerate should clear the >= 0.5 bar"
        );
        let below_bar = aggregate_probe_outcomes(&outcomes_with_healthy_count(half - 1));
        assert!(
            !below_bar.passed,
            "one fewer non-degenerate probe should miss the 0.5 bar"
        );
    }

    /// Falsifier for `#750`'s cross-path combination policy: a bundle
    /// that is healthy on the plain path but degenerate on R4G1 (the
    /// exact divergence observed on a real local bundle -- "cut cut
    /// cut ..." under R4G1 vs. word-salad on plain, both bad but not
    /// identically bad) must NOT pass overall, since `ask`/`chat` will
    /// serve the R4G1 path whenever one is present. Exercises
    /// `combine_path_quality` directly, without loading a real compiled
    /// bundle, following the same discipline as `#744`'s falsifier
    /// above.
    #[test]
    fn combine_path_quality_requires_both_probed_paths_to_pass() {
        let healthy = PathQuality {
            passed: true,
            grounded_answer_rate: 0.9,
            repetition_rate: 0.05,
        };
        let degenerate = PathQuality {
            passed: false,
            grounded_answer_rate: 0.0,
            repetition_rate: 0.95,
        };

        // No R4G1 graph supplied: only the plain path's verdict matters,
        // and the r4g1_* fields stay None (not a probed-and-degenerate
        // Some(0.0)) so a manifest without a graph is distinguishable
        // from one that was probed and found wanting.
        let plain_only = combine_path_quality(healthy, None);
        assert!(plain_only.instruction_eval_passed);
        assert_eq!(plain_only.r4g1_grounded_answer_rate, None);
        assert_eq!(plain_only.r4g1_repetition_rate, None);

        // Plain healthy, R4G1 degenerate: must fail overall -- this is
        // the exact scenario #750 exists to catch, since #744's gate
        // alone would have looked only at the plain path and passed it.
        let plain_healthy_r4g1_degenerate = combine_path_quality(healthy, Some(degenerate));
        assert!(!plain_healthy_r4g1_degenerate.instruction_eval_passed);
        assert_eq!(
            plain_healthy_r4g1_degenerate.r4g1_grounded_answer_rate,
            Some(0.0)
        );

        // Both paths healthy: passes.
        let both_healthy = combine_path_quality(healthy, Some(healthy));
        assert!(both_healthy.instruction_eval_passed);

        // Plain degenerate, R4G1 healthy: still fails -- a healthy R4G1
        // graph cannot paper over a degenerate plain path either, since
        // a manifest without an R4G1 graph present at serving time (or
        // one that's later removed) falls back to the plain path.
        let plain_degenerate_r4g1_healthy = combine_path_quality(degenerate, Some(healthy));
        assert!(!plain_degenerate_r4g1_healthy.instruction_eval_passed);
    }

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
                r4g1_grounded_answer_rate: None,
                r4g1_repetition_rate: None,
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

    #[cfg(unix)]
    #[test]
    fn handle_bound_snapshot_walk_refuses_file_symlink_and_fifo_swaps() {
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;

        fn fifo(path: &Path) {
            let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
            // SAFETY: the C string is live for the call and mkfifo does not
            // retain its pointer.
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        }

        for (scope, scope_name) in [
            (SourceTreeScope::ManifestAdmitted, "manifest"),
            (SourceTreeScope::ManifestlessAll, "manifestless"),
        ] {
            for kind in ["symlink", "fifo"] {
                let dir = snapshot_dir(&format!("handle-file-{scope_name}-{kind}"));
                let target = dir.join("config.json");
                let original = dir.join("config.original");
                let mut swapped = false;
                let result = verified_source_tree_with_hook(&dir, scope, |path| {
                    if !swapped && path == target {
                        std::fs::rename(&target, &original).unwrap();
                        match kind {
                            "symlink" => symlink(&original, &target).unwrap(),
                            "fifo" => fifo(&target),
                            _ => unreachable!(),
                        }
                        swapped = true;
                    }
                });
                let error = result.expect_err("a post-enumeration file swap is terminal");
                assert!(error.to_string().contains("config.json"), "{error}");
                assert_eq!(
                    std::fs::read(&original).expect("original file remains bound"),
                    b"{\"hidden_size\":576}"
                );
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn handle_bound_snapshot_walk_refuses_directory_symlink_escape_and_cycle() {
        use std::os::unix::fs::symlink;

        for kind in ["outside", "cycle"] {
            let dir = snapshot_dir(&format!("handle-directory-{kind}"));
            let child = dir.join("nested");
            let original = dir.join("nested-original");
            let outside = snapshot_dir(&format!("handle-directory-{kind}-outside"));
            std::fs::write(outside.join("escaped.json"), b"outside").unwrap();
            let mut swapped = false;
            let result =
                verified_source_tree_with_hook(&dir, SourceTreeScope::ManifestlessAll, |path| {
                    if !swapped && path == child {
                        std::fs::rename(&child, &original).unwrap();
                        match kind {
                            "outside" => symlink(&outside, &child).unwrap(),
                            "cycle" => symlink(&dir, &child).unwrap(),
                            _ => unreachable!(),
                        }
                        swapped = true;
                    }
                });
            let error = result.expect_err("a directory link swap cannot escape or cycle");
            assert!(error.to_string().contains("nested"), "{error}");
            assert!(outside.join("escaped.json").is_file());
            let _ = std::fs::remove_dir_all(dir);
            let _ = std::fs::remove_dir_all(outside);
        }
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
