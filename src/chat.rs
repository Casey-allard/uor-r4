//! Example direct-chat application built on R⁴ transformerless inference.
//!
//! Chat is a consumer of the core runtime, not a separate inference layer.

use std::fmt;
use std::io::{BufRead, Read, Write};

use crate::model::{default_model_reference, ModelError, ModelObject, ModelStore};
use uor_r4_api::{NormativeServingDecision, NormativeStepAdapter};
use uor_r4_core::transformerless::compiler::{self, Compiled};
use uor_r4_core::transformerless::runtime::{self, Runtime, SampleRng, Store};
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;

const MAX_CHAT_TOKENS: usize = 256;
const MAX_CHAT_HISTORY: usize = 4096;
const MAX_ANSWER_BYTES: usize = 16 * 1024;

/// The pinned default sampling seed (#655 decode-default decision,
/// 2026-08-19): sampled decode is the CLI/serving default everywhere,
/// seeded with this constant so default behavior stays reproducible run
/// to run; greedy decode is the explicit opt-in (`--greedy` on the CLI,
/// `temperature: 0` on the wire). 42 is the seed the #655 F-p2 canary
/// measured 15/15 valid completions with (vs 0/15 greedy,
/// issuecomment-5335796517).
pub const DEFAULT_SAMPLE_SEED: u32 = 42;

/// A completed local chat turn.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatAnswer {
    /// Generated assistant text.
    pub text: String,
    /// Number of tokens generated for this turn.
    pub generated_tokens: usize,
    /// Fraction of generated tokens that already appeared earlier in
    /// this same generation (see [`recent_window_repetition_rate`],
    /// called with the full generated length as its window — a bounded
    /// chat turn is short enough that "recent" can mean "the whole
    /// answer so far"). A fixed 32-token window, the convention Gate C's
    /// `generate_greedy_repetition_rate` uses for its much longer
    /// generations, empirically under-detects real degenerate output
    /// here: `#744`'s own verification against the actual `#745`
    /// word-salad bundle measured only 0.10-0.25 at window 32, because
    /// the small pool of recurring fragments (`tetra`/`gru`/`caption`/…)
    /// mixes with enough distinct short filler subword pieces that few
    /// *exact same-token* recurrences land inside any single 32-token
    /// slice, even though the visible vocabulary is obviously collapsed
    /// to a human reader. Widening the window to the full answer raised
    /// the same transcripts to 0.31-0.41, which is why
    /// [`crate::model::evaluate_live_quality`]'s repetition bar was
    /// recalibrated down from 0.5 to 0.25 alongside this change — see
    /// that function's bar constant for the honest caveat on how that
    /// number was chosen. Unlike
    /// [`repeated_suffix`]'s exact-block detector (this module's other
    /// repetition guard), this also catches small-vocabulary cycling
    /// through several distinct tokens in varying order, which never
    /// forms an identical repeated block.
    pub repeated_token_rate: f64,
    /// #785 C3: which decode surface served this turn and how the
    /// context resolved — the observability the #759 scoping asked for.
    pub witness: DecodeWitness,
    /// #811: `Some` when this turn ended in a typed D4 abstention
    /// instead of served text — `text` is empty, no token is served,
    /// and any partially generated tokens were dropped (the server
    /// tier's exact contract). `None` on every served answer.
    pub abstention: Option<ChatAbstention>,
}

/// #811: the CLI's typed honest abstention — the same D4 outcome the
/// server's R4G1 tier surfaces (issue #78 decision D4), produced by the
/// same deployed policy engine. No token is guessed; the status names
/// how the context resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAbstention {
    /// The abstaining resolution status label (`"novel"`,
    /// `"contradictory"`, …) from the deployed policy.
    pub status: String,
    /// Whether the policy widened once before abstaining.
    pub widened: bool,
    /// #839 phase 1 (RF-30): the typed spec-§2 outcome label — always
    /// `"abstention"` on this record (an abstention is a successful,
    /// honest outcome; spec §5 CLI row).
    pub outcome: &'static str,
    /// The typed abstention cause. In legacy-coverage mode the only legal
    /// cause is `"distributionally-novel"` (spec §6); calibrated-mode
    /// causes are phase-2 vocabulary and are never minted here.
    pub cause: &'static str,
    /// The coverage-axis reading of the abstaining window (spec §2).
    pub coverage: &'static str,
}

/// #785 C3: the depth/fallback-tier witness for one chat turn.
///
/// `ask`/`chat` previously discarded which engine served an answer and
/// how the context resolved, so degenerate output could not be
/// attributed to a tier from the outside. The fields are additive
/// observability — they change no decode behavior.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecodeWitness {
    /// Which decode surface produced the answer: `"r4g1-beam"`,
    /// `"r4g1-sampled"`, `"tla-plain-greedy"`, or `"tla-plain-sampled"`.
    /// Empty only for
    /// answers constructed outside [`hologram_answer`] (test stubs).
    pub engine: &'static str,
    /// R4G1 path: candidate queries that resolved **without** consulting
    /// node evidence — the node-score buffer stayed all-MIN (the #785-C1
    /// contract). On a scored graph this means a context-row hit (the
    /// converter-flavor last resort is gated off); on a converter-flavor
    /// graph it can also be the global last-resort table or no candidate
    /// at all. Finer attribution needs an engine-level tier tag
    /// (recorded limitation, #785-C3). Zero on the plain path.
    pub non_node_queries: usize,
    /// R4G1 path: candidate queries resolved through node evidence (the
    /// engine published at least one node score). Zero on the plain path.
    pub node_path_queries: usize,
    /// Plain path: per-emitted-token resolution depth (0..=STAGES) from
    /// [`runtime::Prediction`]. Empty on the R4G1 path, whose engine does
    /// not report per-tier depth (recorded limitation, #785-C3).
    pub plain_depths: Vec<u8>,
}

/// Failure to load or run the local transformerless chat engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ChatError {
    /// A required file could not be read.
    Io(std::io::Error),
    /// The compiled artifact container was invalid.
    InvalidArtifacts,
    /// The graded store container was invalid.
    InvalidStore,
    /// Generation produced no tokens, or the question/answer could not be
    /// tokenized or decoded into its caller-owned buffer.
    EmptyGeneration,
    /// The bundle carries a tagged decode-only tokenizer, but direct chat has
    /// no exact registered host encoder for that family/version.
    TokenizerUnavailable { family: String, version: u32 },
    /// Generation entered a repeated-token loop and was rejected.
    RepetitiveGeneration,
    /// No CID-addressed, capability-attested model was selected.
    MissingModel,
    /// The model bundle or its CID verification failed.
    Model(ModelError),
    /// #839 phase 1 (RF-30), spec §6: the bundle carries selective-prediction
    /// calibration data (`selective_calibration.bin`), but no executable
    /// calibrated mode exists — the typed `hard-incompatibility` outcome,
    /// fail-closed. Present calibration never silently degrades to the
    /// always-serve legacy surface.
    SelectiveCalibrationPresent { path: std::path::PathBuf },
    /// A bundle presented a release envelope but could not reproduce strict
    /// schema-2 production admission from the exact chat bytes.
    ProductionAdmission(String),
}

impl fmt::Display for ChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to load chat data: {error}"),
            Self::InvalidArtifacts => formatter.write_str("invalid transformerless artifacts"),
            Self::InvalidStore => formatter.write_str("invalid transformerless store"),
            Self::EmptyGeneration => formatter.write_str("transformerless produced no text"),
            Self::TokenizerUnavailable { family, version } => write!(
                formatter,
                "tokenizer unavailable: exact host adapter {family}/{version} is required"
            ),
            Self::RepetitiveGeneration => formatter.write_str(
                "transformerless generation became repetitive; refusing a low-quality answer",
            ),
            Self::MissingModel => {
                formatter.write_str("no chat model selected; set TLESS_MODEL or pass --model")
            }
            Self::Model(error) => error.fmt(formatter),
            Self::SelectiveCalibrationPresent { path } => write!(
                formatter,
                "hard-incompatibility: selective-prediction calibration data is present at \
                 {} but no executable calibrated mode exists (#839 phase 1); present \
                 calibration never degrades to legacy serving",
                path.display()
            ),
            Self::ProductionAdmission(reason) => {
                write!(formatter, "production chat admission unavailable: {reason}")
            }
        }
    }
}

impl std::error::Error for ChatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidArtifacts
            | Self::InvalidStore
            | Self::EmptyGeneration
            | Self::TokenizerUnavailable { .. }
            | Self::RepetitiveGeneration
            | Self::MissingModel
            | Self::SelectiveCalibrationPresent { .. }
            | Self::ProductionAdmission(_) => None,
            Self::Model(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ChatError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ModelError> for ChatError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// Builder for a direct local [`ChatEngine`].
#[derive(Debug, Clone)]
pub struct ChatEngineBuilder {
    max_tokens: usize,
    model: Option<String>,
    sample_seed: Option<u32>,
}

impl Default for ChatEngineBuilder {
    fn default() -> Self {
        Self {
            max_tokens: 96,
            model: Some(default_model_reference()),
            sample_seed: None,
        }
    }
}

impl ChatEngineBuilder {
    /// Set the maximum number of generated tokens per turn.
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens.clamp(1, MAX_CHAT_TOKENS);
        self
    }

    /// Opt into seeded weighted sampling instead of the default
    /// deterministic decode, on both generation paths. Strictly additive:
    /// omitting this call (the default) keeps every turn on the existing
    /// deterministic paths (`generate_greedy_into` on the plain path, the
    /// greedy beam on the R4G1 path), byte-for-byte unchanged. When set,
    /// the plain path samples per issue #762 lever 2, and the R4G1 path
    /// samples one seeded trajectory over the same candidate queries the
    /// beam uses with the same #762 weighting scheme (#785-C2,
    /// maintainer-approved parity). The seeded RNG persists and advances
    /// across turns within one `ChatEngine`, so a fixed seed reproduces
    /// an entire session's sampled output, not just one turn.
    #[must_use]
    pub fn sample_seed(mut self, seed: u32) -> Self {
        self.sample_seed = Some(seed);
        self
    }

    /// Select a CID-addressed model manifest by name or UOR CID.
    #[must_use]
    pub fn model(mut self, reference: impl Into<String>) -> Self {
        self.model = Some(reference.into());
        self
    }

    /// Load all local data and construct a production-admitted engine. A
    /// pre-schema-2 bundle is rejected rather than silently downgraded.
    pub fn build(self) -> Result<ChatEngine, ChatError> {
        self.build_with_admission(false)
    }

    /// Explicitly permit a pre-schema-2 research bundle. The typed warning is
    /// returned with the engine so callers cannot mistake this compatibility
    /// path for production admission.
    pub fn build_for_research(self) -> Result<(ChatEngine, ResearchServingWarning), ChatError> {
        let engine = self.build_with_admission(true)?;
        Ok((engine, ResearchServingWarning::PreSchema2Compatibility))
    }

    fn build_with_admission(self, allow_research: bool) -> Result<ChatEngine, ChatError> {
        let reference = self.model.as_deref().ok_or(ChatError::MissingModel)?;
        let model_store = ModelStore::from_env();
        let manifest = match model_store.read_manifest(reference) {
            Ok(manifest) => manifest,
            Err(ModelError::CompiledNotImported(path)) => {
                return build_local_compiled_engine(
                    &model_store,
                    &path,
                    reference,
                    self.max_tokens,
                    self.sample_seed,
                    allow_research,
                );
            }
            Err(error) => return Err(error.into()),
        };
        manifest.validate_for_chat()?;
        if let Some(report) = &manifest.evaluation_report {
            let _ = model_store.get(report)?;
        }
        let artifact_bytes = model_store.get(&manifest.artifacts)?;
        let artifacts =
            compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
        let store_bytes = model_store.get(&manifest.store)?;
        let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;
        // #790 item 6: resolve against the store root with the same name
        // sanitization the manifest read applied — the raw name was
        // previously interpolated into a hardcoded `.uor-models` path, so
        // it could disagree with an env-relocated store or escape it.
        let model_dir = crate::model::compiled_model_dir(&manifest.name);
        // #839 phase 1 (RF-30), spec §6: present selective-calibration data
        // fails closed before the engine constructs — never a legacy serve.
        let calibration_path = model_dir.join(crate::selective::SELECTIVE_CALIBRATION_FILE);
        if calibration_path.is_file() {
            return Err(ChatError::SelectiveCalibrationPresent {
                path: calibration_path,
            });
        }
        let r4g1_bytes = read_preferred_chat_graph(&model_dir)?;
        validate_chat_r4g1_structure(r4g1_bytes.as_deref())?;
        let bundled_tokenizer = match model_store.get(&manifest.tokenizer) {
            Ok(bytes) => bytes,
            Err(error) => match matching_ref_tokenizer(&manifest.tokenizer, r4g1_bytes.as_deref())?
            {
                Some(bytes) => bytes,
                None => return Err(error.into()),
            },
        };
        let tokenizer_bytes = select_chat_tokenizer_bytes(&bundled_tokenizer)?;
        let r4g1_bytes = bind_chat_r4g1(r4g1_bytes, &tokenizer_bytes)?;
        let tokenizer = parse_chat_tokenizer(&tokenizer_bytes)?;
        tracing::info!(
            model = %manifest.name,
            source_model = %manifest.source_model,
            artifact_cid = %manifest.artifacts.cid,
            store_cid = %manifest.store.cid,
            r4g1_loaded = r4g1_bytes.is_some(),
            max_tokens = self.max_tokens,
            "transformerless chat engine loaded"
        );
        let policy_engine = load_chat_policy_engine(
            &model_dir,
            r4g1_bytes.as_deref(),
            &artifact_bytes,
            &tokenizer_bytes,
            read_chat_score_report(&model_dir).as_deref(),
            allow_research,
        )?;
        Ok(ChatEngine {
            artifacts,
            store,
            r4g1_bytes,
            tokenizer,
            history: [0; MAX_CHAT_HISTORY],
            history_len: 0,
            max_tokens: self.max_tokens,
            sample_rng: self.sample_seed.map(SampleRng::new),
            policy_engine: policy_engine.engine,
        })
    }
}

/// Typed evidence that a caller explicitly selected the non-production
/// compatibility path. This is a warning, not an admission token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchServingWarning {
    PreSchema2Compatibility,
}

impl std::fmt::Display for ResearchServingWarning {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreSchema2Compatibility => formatter.write_str(
                "RESEARCH ONLY: pre-schema-2 bundle loading was explicitly enabled; this engine is not production-admitted",
            ),
        }
    }
}

/// #811: the optional D4 policy override beside the graph
/// (`graph/score_report.json`), pre-validated as JSON exactly like the
/// server-side loader (`src/r4g1.rs`): an unparseable report is ignored
/// (D4 defaults apply), never an error.
fn read_chat_score_report(model_dir: &std::path::Path) -> Option<Vec<u8>> {
    let bytes = std::fs::read(model_dir.join("graph").join("score_report.json")).ok()?;
    serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    Some(bytes)
}

struct LoadedChatPolicyEngine {
    engine: Option<ChatPolicyEngine>,
    production_admitted: bool,
}

enum ChatPolicyEngine {
    Production(uor_r4_api::ProductionPolicyEngine),
    Research(uor_r4_api::engine::R4Engine),
}

impl ChatPolicyEngine {
    #[cfg(test)]
    fn policy_counters(&self) -> uor_r4_api::PolicyCounters {
        match self {
            Self::Production(policy) => policy.policy_counters(),
            Self::Research(policy) => policy.policy_counters(),
        }
    }
}

/// Load the D4 half of the shared serving adapter. Any present release
/// envelope is authoritative: all schema-2 bytes and the deployed-quality
/// report must reproduce before chat can construct. Pre-schema-2 bundles use
/// the explicitly named, loudly warned research compatibility path.
fn load_chat_policy_engine(
    model_dir: &std::path::Path,
    r4g1_bytes: Option<&[u8]>,
    artifact_bytes: &[u8],
    tokenizer_bytes: &[u8],
    score_report: Option<&[u8]>,
    allow_research: bool,
) -> Result<LoadedChatPolicyEngine, ChatError> {
    let release_manifest =
        model_dir.join(crate::release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME);
    let has_release_envelope = match std::fs::symlink_metadata(&release_manifest) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(ChatError::Io(error)),
    };
    if !has_release_envelope {
        if !allow_research {
            return Err(ChatError::ProductionAdmission(format!(
                "{} has no release-bundle.json schema-2 production envelope; rerun with the explicit research compatibility option only for non-production investigation",
                model_dir.display()
            )));
        }
        tracing::warn!(
            directory = %model_dir.display(),
            "release-bundle.json is absent; chat is using the explicit pre-schema-2 research compatibility loader and is not production-admitted"
        );
        return Ok(LoadedChatPolicyEngine {
            engine: load_research_chat_policy_engine(
                r4g1_bytes,
                artifact_bytes,
                tokenizer_bytes,
                score_report,
            ),
            production_admitted: false,
        });
    }

    let graph = r4g1_bytes.ok_or_else(|| {
        ChatError::ProductionAdmission(
            "release-bundle.json is present but graph/score.r4g1 is unavailable".to_owned(),
        )
    })?;
    let graph_path = model_dir.join("graph/score.r4g1");
    let signature_artifact_path = model_dir.join("tless_artifacts.bin");
    crate::release_bundle_loader::production_bundle_root(&graph_path, &signature_artifact_path)
        .map_err(ChatError::ProductionAdmission)?;
    require_exact_production_chat_file(&graph_path, graph)?;
    require_exact_production_chat_file(&signature_artifact_path, artifact_bytes)?;
    require_exact_production_chat_file(&model_dir.join("tokenizer.bin"), tokenizer_bytes)?;
    let captured = crate::release_bundle_loader::capture_production_admission(model_dir)
        .map_err(ChatError::ProductionAdmission)?;
    let verified = crate::release_bundle_loader::verify_production_admission(
        graph,
        artifact_bytes,
        Some(tokenizer_bytes),
        &captured,
    )
    .map_err(ChatError::ProductionAdmission)?;
    if uor_r4_graph_certify::GraphScorer::from_artifact(
        graph,
        Some(artifact_bytes),
        uor_r4_graph_certify::DEFAULT_ROOT_TOP_B,
        uor_r4_graph_certify::DEFAULT_EXCT_TOP_X,
    )
    .is_none()
    {
        return Err(ChatError::ProductionAdmission(
            "the deployed scorer does not rebuild from the schema-2 graph".to_owned(),
        ));
    }
    let report_cid = verified
        .manifest()
        .components
        .deployed_quality_report
        .as_deref()
        .ok_or_else(|| {
            ChatError::ProductionAdmission(
                "schema-2 release manifest omitted deployed-quality report CID".to_owned(),
            )
        })?;
    let engine = uor_r4_api::load_production_policy_engine(uor_r4_api::ProductionServingParts {
        engine: uor_r4_api::engine::EngineParts {
            graph,
            signature_artifact: artifact_bytes,
            tokenizer: Some(tokenizer_bytes),
            score_report: Some(&captured.score_report),
        },
        deployed_quality_report: &verified.deployed_quality_report,
        verified_envelope: &verified.envelope,
    })
    .map_err(|error| ChatError::ProductionAdmission(error.to_string()))?;
    tracing::info!(
        directory = %model_dir.display(),
        report_cid,
        "chat admitted the exact schema-2 production envelope"
    );
    Ok(LoadedChatPolicyEngine {
        engine: Some(ChatPolicyEngine::Production(engine)),
        production_admitted: true,
    })
}

fn require_exact_production_chat_file(
    path: &std::path::Path,
    selected_bytes: &[u8],
) -> Result<(), ChatError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        ChatError::ProductionAdmission(format!(
            "required production component {}: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(ChatError::ProductionAdmission(format!(
            "required production component {} is not a regular non-symlink file",
            path.display()
        )));
    }
    let bytes = std::fs::read(path).map_err(|error| {
        ChatError::ProductionAdmission(format!(
            "required production component {}: {error}",
            path.display()
        ))
    })?;
    if bytes != selected_bytes {
        return Err(ChatError::ProductionAdmission(format!(
            "selected chat bytes do not equal required production component {}",
            path.display()
        )));
    }
    Ok(())
}

/// Explicit compatibility loader for legacy/synthetic research inputs. It
/// never claims schema-2 production admission and may return no gate.
fn load_research_chat_policy_engine(
    r4g1_bytes: Option<&[u8]>,
    artifact_bytes: &[u8],
    tokenizer_bytes: &[u8],
    score_report: Option<&[u8]>,
) -> Option<ChatPolicyEngine> {
    let graph = r4g1_bytes?;
    // The deployed scorer itself must accept these bytes first: the
    // engine's own load treats a scorer-rebuild refusal on CID-verified
    // bytes as a self-produced-defect panic (R5, #510) — correct for the
    // pipeline's own artifacts, but the chat walk also serves
    // converter-era/legacy graphs the deployed engine has no scorer for
    // (#655-C1e recognize-don't-migrate). Probe first so those bundles
    // get the documented gate-absent warn instead of that panic.
    if uor_r4_graph_certify::GraphScorer::from_artifact(
        graph,
        Some(artifact_bytes),
        uor_r4_graph_certify::DEFAULT_ROOT_TOP_B,
        uor_r4_graph_certify::DEFAULT_EXCT_TOP_X,
    )
    .is_none()
    {
        tracing::warn!(
            "the deployed scorer does not rebuild from this graph (legacy/converter-era \
             bundle); the ask-path D4 abstention gate (#811) is ABSENT this session and \
             the graph walk runs ungated"
        );
        return None;
    }
    // This bypass is confined to the loudly named research path above. A
    // present schema-2 envelope never reaches it.
    match uor_r4_api::engine::R4Engine::load_accepting_quality(uor_r4_api::engine::EngineParts {
        graph,
        signature_artifact: artifact_bytes,
        tokenizer: Some(tokenizer_bytes),
        score_report,
    }) {
        Ok(engine) => Some(ChatPolicyEngine::Research(engine)),
        Err(error) => {
            tracing::warn!(
                %error,
                "deployed policy engine could not load this bundle; the ask-path \
                 D4 abstention gate (#811) is ABSENT this session and the graph \
                 walk runs ungated (the serving tier would refuse this bundle)"
            );
            None
        }
    }
}

fn build_local_compiled_engine(
    model_store: &ModelStore,
    directory: &std::path::Path,
    reference: &str,
    max_tokens: usize,
    sample_seed: Option<u32>,
    allow_research: bool,
) -> Result<ChatEngine, ChatError> {
    // #839 phase 1 (RF-30), spec §6: present selective-calibration data
    // fails closed before the engine constructs — never a legacy serve.
    let calibration_path = directory.join(crate::selective::SELECTIVE_CALIBRATION_FILE);
    if calibration_path.is_file() {
        return Err(ChatError::SelectiveCalibrationPresent {
            path: calibration_path,
        });
    }
    let artifact_bytes = std::fs::read(directory.join("tless_artifacts.bin"))?;
    let r4g1_bytes = read_preferred_chat_graph(directory)?;
    validate_chat_r4g1_structure(r4g1_bytes.as_deref())?;
    // #655-C1e: the plain-path store is optional for an R4G1-era bundle —
    // the release-packaged component set (#655-D) is graph + signature
    // artifact + tokenizer. Absent store with a present graph leaves the
    // plain fallback tier EMPTY (it declines honestly instead of serving;
    // the decode witness still names whichever engine ran, #785-C3).
    // Absent store with NO graph stays the required-file error it always
    // was — that directory serves nothing.
    let store_bytes = match std::fs::read(directory.join("tless_store.bin")) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && r4g1_bytes.is_some() => {
            tracing::warn!(
                directory = %directory.display(),
                "R4G1-era bundle without a plain-path store (tless_store.bin); \
                 the plain fallback tier is empty and will decline honestly (#655-C1e)"
            );
            None
        }
        Err(error) => return Err(error.into()),
    };
    let tok_file = directory.join("tokenizer.bin");
    tracing::debug!(?tok_file, "resolved chat tokenizer path");
    let bundled_tokenizer = std::fs::read(&tok_file)?;
    let tokenizer_bytes = select_chat_tokenizer_bytes(&bundled_tokenizer)?;
    let r4g1_bytes = bind_chat_r4g1(r4g1_bytes, &tokenizer_bytes)?;
    let artifacts =
        compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = match store_bytes.as_deref() {
        Some(bytes) => runtime::parse_store(bytes).ok_or(ChatError::InvalidStore)?,
        None => (0..=compiler::STAGES).map(|_| Default::default()).collect(),
    };

    // Content-address all local compiler outputs immediately. A manifest and
    // quality report remain optional metadata; integrity does not.
    let artifact_object = model_store.put(&artifact_bytes)?;
    let store_cid = match store_bytes.as_deref() {
        Some(bytes) => model_store.put(bytes)?.cid,
        None => "absent (R4G1-era bundle, #655-C1e)".to_owned(),
    };
    let tokenizer_object = model_store.put(&tokenizer_bytes)?;
    let tokenizer = parse_chat_tokenizer(&tokenizer_bytes)?;
    let policy_engine = load_chat_policy_engine(
        directory,
        r4g1_bytes.as_deref(),
        &artifact_bytes,
        &tokenizer_bytes,
        read_chat_score_report(directory).as_deref(),
        allow_research,
    )?;
    if policy_engine.production_admitted {
        tracing::info!(
            model = reference,
            directory = %directory.display(),
            artifact_cid = %artifact_object.cid,
            store_cid = %store_cid,
            tokenizer_cid = %tokenizer_object.cid,
            "using a production-admitted locally compiled bundle"
        );
    } else {
        tracing::warn!(
            model = reference,
            directory = %directory.display(),
            artifact_cid = %artifact_object.cid,
            store_cid = %store_cid,
            tokenizer_cid = %tokenizer_object.cid,
            "using a pre-schema-2 local research bundle without production admission"
        );
    }
    Ok(ChatEngine {
        artifacts,
        store,
        r4g1_bytes,
        tokenizer,
        history: [0; MAX_CHAT_HISTORY],
        history_len: 0,
        max_tokens,
        sample_rng: sample_seed.map(SampleRng::new),
        policy_engine: policy_engine.engine,
    })
}

/// Stateful local chat engine with no HTTP server or background worker.
pub struct ChatEngine {
    artifacts: Compiled,
    store: Store,
    r4g1_bytes: Option<Vec<u8>>,
    tokenizer: Tokenizer,
    history: [u32; MAX_CHAT_HISTORY],
    history_len: usize,
    max_tokens: usize,
    /// `Some` opts every turn into weighted sampling on whichever path
    /// serves it — #762 lever 2 on the plain path, the #785-C2 sampled
    /// decode on the R4G1 path (see `ChatEngineBuilder::sample_seed`);
    /// the RNG advances turn to turn so a session is reproducible end to
    /// end from its initial seed. `None` (the default) leaves both
    /// deterministic paths completely untouched.
    sample_rng: Option<SampleRng>,
    /// #811: the deployed D4 policy engine over this same bundle — the
    /// ask-path abstention gate. Stateful across turns (widen-once
    /// bookkeeping, novel-seen FIFO), like the server tier's own engine.
    /// `None` only when the bundle has no graph or the engine refused
    /// the bytes (loudly warned at build).
    policy_engine: Option<ChatPolicyEngine>,
}

impl ChatEngine {
    /// Start configuring a local chat engine.
    #[must_use]
    pub fn builder() -> ChatEngineBuilder {
        ChatEngineBuilder::default()
    }

    /// Generate one answer and retain its tokens as context for the next turn.
    pub fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
        let span = tracing::debug_span!("ask", question_bytes = question.len());
        let _guard = span.enter();
        hologram_answer(
            &self.artifacts,
            &self.store,
            self.r4g1_bytes.as_deref(),
            &self.tokenizer,
            &mut self.history,
            &mut self.history_len,
            question,
            self.max_tokens,
            self.sample_rng.as_mut(),
            self.policy_engine.as_mut(),
        )
    }
}

/// One CLI-chat call into the shared token-authoritative adapter. A missing
/// policy object remains an explicit research-era fallback; a present policy
/// must compose successfully and may never be bypassed on an error.
fn normative_chat_step(
    policy_engine: &mut Option<&mut ChatPolicyEngine>,
    runtime: &R4G1Runtime<'_>,
    node_scores: &mut [ScoreQ],
    context_tokens: &[u32],
    session_signature: Option<&[u8]>,
) -> Result<Option<NormativeServingDecision>, ChatError> {
    let Some(policy) = policy_engine.as_deref_mut() else {
        return Ok(None);
    };
    let mut adapter = match policy {
        ChatPolicyEngine::Production(policy) => {
            NormativeStepAdapter::new(policy, runtime, node_scores)
        }
        ChatPolicyEngine::Research(policy) => {
            NormativeStepAdapter::new_with_reference_policy(policy, runtime, node_scores)
        }
    };
    adapter
        .select(context_tokens, session_signature)
        .map(Some)
        .map_err(|error| {
            ChatError::Io(std::io::Error::other(format!(
                "normative CLI-chat step rejected the context: {error}"
            )))
        })
}

/// Speculatively execute the same composed D4 + `R4G1Runtime` step without
/// retaining D4 counters or widen-once memory. Beam hypotheses use this seam;
/// the winning path is subsequently committed through [`normative_chat_step`]
/// before any token is exposed to the caller.
fn replay_normative_chat_step(
    policy_engine: &mut Option<&mut ChatPolicyEngine>,
    runtime: &R4G1Runtime<'_>,
    node_scores: &mut [ScoreQ],
    context_tokens: &[u32],
    session_signature: Option<&[u8]>,
) -> Result<Option<NormativeServingDecision>, ChatError> {
    let Some(policy) = policy_engine.as_deref_mut() else {
        return Ok(None);
    };
    let mut adapter = match policy {
        ChatPolicyEngine::Production(policy) => {
            NormativeStepAdapter::new(policy, runtime, node_scores)
        }
        ChatPolicyEngine::Research(policy) => {
            NormativeStepAdapter::new_with_reference_policy(policy, runtime, node_scores)
        }
    };
    adapter
        .replay_select(context_tokens, session_signature)
        .map(Some)
        .map_err(|error| {
            ChatError::Io(std::io::Error::other(format!(
                "normative CLI-chat beam replay rejected the context: {error}"
            )))
        })
}

/// Commit one deterministic beam prefix through the stateful production
/// adapter. Each selected token must still belong to that step's
/// runtime-owned shortlist. This runs only for the winning prefix (including
/// the best prefix at a terminal abstention), never for losing hypotheses.
fn commit_normative_chat_beam_prefix(
    policy_engine: &mut Option<&mut ChatPolicyEngine>,
    runtime: &R4G1Runtime<'_>,
    node_scores: &mut [ScoreQ],
    committed_context: &mut Vec<u32>,
    selected_tokens: &[u32],
    session_signature: Option<&[u8]>,
) -> Result<(), ChatError> {
    for &selected_token in selected_tokens {
        match normative_chat_step(
            policy_engine,
            runtime,
            node_scores,
            committed_context,
            session_signature,
        )? {
            Some(NormativeServingDecision::Serve(serve))
                if serve
                    .candidates
                    .ranked()
                    .iter()
                    .any(|candidate| candidate.token == selected_token) => {}
            Some(NormativeServingDecision::Serve(_)) => {
                return Err(ChatError::Io(std::io::Error::other(
                    "CLI beam selected a token outside the normative R4G1Runtime shortlist",
                )));
            }
            Some(NormativeServingDecision::Abstain(_)) => {
                return Err(ChatError::Io(std::io::Error::other(
                    "CLI beam replay/commit diverged before the selected prefix ended",
                )));
            }
            Some(NormativeServingDecision::Decline(_)) => {
                return Err(ChatError::Io(std::io::Error::other(
                    "D4 permitted CLI beam but R4G1Runtime produced no candidate",
                )));
            }
            None => {
                return Err(ChatError::Io(std::io::Error::other(
                    "CLI beam lost its production policy adapter before commit",
                )));
            }
        }
        committed_context.push(selected_token);
    }
    Ok(())
}

/// Evidence-only replay of the exact shared production step used by CLI
/// chat. This is not a serving entry point: it accepts already-tokenized,
/// bounded context solely so the canonical cross-surface producer can
/// mechanically execute the same `normative_chat_step` for beam-first and
/// sampled policies without claiming that it exercised prompt tokenization,
/// terminal I/O, HTTP routing, or a browser wrapper. Tokenizer bytes are still
/// required to authenticate a nonzero `HEAD.tokenizer_cid`; this helper does
/// not use them to encode the already-tokenized context.
pub(crate) fn replayable_normative_chat_step_for_evidence(
    graph: &[u8],
    signature_artifact: &[u8],
    tokenizer: Option<&[u8]>,
    score_report: Option<&[u8]>,
    context_tokens: &[u32],
    session_signature: &[u8],
    sample_seed: Option<u32>,
) -> Result<(u32, uor_r4_graph_runtime::ServedCandidates), String> {
    let runtime = R4G1Runtime::parse(graph)
        .map_err(|error| format!("cross-surface CLI-chat graph: {error:?}"))?;
    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let policy =
        uor_r4_api::engine::R4Engine::load_accepting_quality(uor_r4_api::engine::EngineParts {
            graph,
            signature_artifact,
            tokenizer,
            score_report,
        })
        .map_err(|error| error.to_string())?;
    let mut policy = ChatPolicyEngine::Research(policy);
    let mut policy_ref = Some(&mut policy);
    let decision = normative_chat_step(
        &mut policy_ref,
        &runtime,
        &mut node_scores,
        context_tokens,
        Some(session_signature),
    )
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "cross-surface CLI-chat policy adapter is absent".to_owned())?;
    match decision {
        NormativeServingDecision::Serve(serve) => {
            let token = match sample_seed {
                Some(seed) => {
                    let mut rng = SampleRng::new(seed);
                    serve.select_sampled_token(&[], &mut rng)
                }
                None => serve.token,
            };
            Ok((token, serve.candidates))
        }
        NormativeServingDecision::Abstain(_) => {
            Err("cross-surface CLI-chat position abstained".to_owned())
        }
        NormativeServingDecision::Decline(_) => {
            Err("cross-surface CLI-chat position declined".to_owned())
        }
    }
}

fn chat_abstention(outcome: uor_r4_api::engine::AbstainOutcome) -> ChatAbstention {
    let label = uor_r4_api::engine::PolicyStatus::from(outcome.status).label();
    ChatAbstention {
        status: label.to_owned(),
        widened: outcome.widened,
        outcome: crate::selective::STATUS_ABSTENTION,
        cause: crate::selective::CAUSE_DISTRIBUTIONALLY_NOVEL,
        coverage: crate::selective::coverage_for_policy_label(label)
            .unwrap_or(crate::selective::COVERAGE_DISTRIBUTIONALLY_NOVEL),
    }
}

/// #811: the typed abstention answer — empty text, zero tokens served,
/// partial generation dropped (the server tier's exact contract), the
/// decode witness naming which surface the gate fired on.
fn abstention_answer(
    engine: &'static str,
    non_node_queries: usize,
    node_path_queries: usize,
    abstention: ChatAbstention,
) -> ChatAnswer {
    let witness = DecodeWitness {
        engine,
        non_node_queries,
        node_path_queries,
        plain_depths: Vec::new(),
    };
    tracing::info!(
        engine = witness.engine,
        status = %abstention.status,
        widened = abstention.widened,
        "D4 abstention on the ask path (#811): no token served"
    );
    ChatAnswer {
        text: String::new(),
        generated_tokens: 0,
        repeated_token_rate: 0.0,
        witness,
        abstention: Some(abstention),
    }
}

#[allow(clippy::too_many_arguments)]
fn hologram_answer(
    artifacts: &Compiled,
    store: &Store,
    r4g1_bytes: Option<&[u8]>,
    tokenizer: &Tokenizer,
    history: &mut [u32; MAX_CHAT_HISTORY],
    history_len: &mut usize,
    question: &str,
    max_tokens: usize,
    mut sample_rng: Option<&mut SampleRng>,
    mut policy_engine: Option<&mut ChatPolicyEngine>,
) -> Result<ChatAnswer, ChatError> {
    ensure_chat_prompt_encoder(tokenizer)?;
    let mut question_tokens = [0u32; MAX_CHAT_HISTORY];
    let question_count = tokenizer
        .encode_into(question, &mut question_tokens)
        .ok_or(ChatError::EmptyGeneration)?;
    let question_tokens = if *history_len == 0 {
        &question_tokens[..question_count]
    } else {
        &question_tokens[1..question_count]
    };
    append_history(history, history_len, question_tokens);
    let session_signature = uor_r4_router::session_signature_from_tokens(&history[..*history_len]);

    if let Some(bytes) = r4g1_bytes {
        match uor_r4_graph_runtime::R4G1Runtime::parse(bytes) {
            Err(error) => {
                // #790 item 7: a graph that was present but unusable used
                // to fall through to the plain path with no trace at all —
                // after import required both paths to pass, that silence
                // hid a real regression. The downgrade is now named, and
                // the served answer's witness says which engine ran.
                tracing::warn!(
                    ?error,
                    "compiled R4G1 graph failed to parse at ask time; \
                     serving from the plain transformerless path instead"
                );
            }
            Ok(r4g1) => {
                use uor_r4_core::transformerless::score_q::ScoreQ;
                let rot = compiler::derive_rotations();
                let num_nodes = r4g1.node_count() as usize;
                let mut node_scores = vec![ScoreQ::MIN; num_nodes];

                // #785-C2 (maintainer-approved 2026-08-18): opt-in sampling
                // parity on the R4G1 path. One seeded trajectory over the
                // same candidate queries the beam uses, weighted by the
                // #762 plain-path sampler's exact scheme — order-preserving
                // shift to positive weights, the same ~1000-per-occurrence
                // soft repetition penalty with floor 1 (a penalized token
                // stays reachable, never excluded), and the shared
                // division-free `SampleRng::draw`. Greedy beam decoding
                // below remains byte-identical when no seed is supplied.
                if let Some(rng) = sample_rng.as_deref_mut() {
                    let mut generated: Vec<u32> = Vec::new();
                    let mut non_node_queries = 0usize;
                    let mut node_path_queries = 0usize;
                    let steps = max_tokens.min(MAX_CHAT_TOKENS);
                    for _ in 0..steps {
                        let mut context = history[..*history_len].to_vec();
                        context.extend_from_slice(&generated);
                        let len = core::cmp::min(context.len(), compiler::WINDOW);
                        let window = &context[context.len() - len..];
                        let (candidates, fallback_token) = match normative_chat_step(
                            &mut policy_engine,
                            &r4g1,
                            &mut node_scores,
                            &context,
                            Some(&session_signature),
                        )? {
                            Some(NormativeServingDecision::Serve(serve)) => {
                                (serve.candidates, serve.token)
                            }
                            Some(NormativeServingDecision::Abstain(outcome)) => {
                                return Ok(abstention_answer(
                                    "r4g1-sampled",
                                    non_node_queries,
                                    node_path_queries,
                                    chat_abstention(outcome),
                                ));
                            }
                            Some(NormativeServingDecision::Decline(_)) => {
                                return Err(ChatError::Io(std::io::Error::other(
                                    "D4 permitted CLI chat but R4G1Runtime produced no candidate",
                                )));
                            }
                            None => {
                                // Explicit research-era fallback: there is no
                                // D4 object to compose, but candidates still
                                // come only from the normative runtime.
                                node_scores.fill(ScoreQ::MIN);
                                let bundle = runtime::bundle_window_plain(artifacts, &rot, window);
                                let sig = runtime::sig_plain(artifacts, &bundle);
                                let candidates = r4g1
                                    .predict_served_candidates_with_signature_lanes(
                                        &context,
                                        Some(&sig),
                                        Some(&session_signature),
                                        &mut node_scores,
                                    );
                                let Some(winner) = candidates.winner() else {
                                    break;
                                };
                                (candidates, winner.token)
                            }
                        };
                        if node_scores.iter().all(|s| s.raw() == ScoreQ::MIN.raw()) {
                            non_node_queries += 1;
                        } else {
                            node_path_queries += 1;
                        }
                        let chosen = uor_r4_api::select_sampled_runtime_candidate(
                            &candidates,
                            &generated,
                            fallback_token,
                            rng,
                        );
                        if chosen == 0 || chosen == 2 {
                            // End-of-sequence: stop without emitting the
                            // terminal token into the visible answer.
                            break;
                        }
                        generated.push(chosen);
                    }
                    if generated.is_empty() {
                        return Err(ChatError::EmptyGeneration);
                    }
                    append_history(history, history_len, &generated);
                    let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
                    let answer_len = tokenizer
                        .decode_into(&generated, &mut answer_bytes)
                        .ok_or(ChatError::EmptyGeneration)?;
                    let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
                        .trim()
                        .to_owned();
                    if text.is_empty() {
                        return Err(ChatError::EmptyGeneration);
                    }
                    let witness = DecodeWitness {
                        engine: "r4g1-sampled",
                        non_node_queries,
                        node_path_queries,
                        plain_depths: Vec::new(),
                    };
                    tracing::info!(
                        engine = witness.engine,
                        non_node_queries,
                        node_path_queries,
                        "decode witness (#785 C3)"
                    );
                    return Ok(ChatAnswer {
                        text,
                        generated_tokens: generated.len(),
                        repeated_token_rate: recent_window_repetition_rate(
                            &generated,
                            generated.len(),
                        ),
                        witness,
                        abstention: None,
                    });
                }

                struct BeamHypothesis {
                    tokens: Vec<u32>,
                    score: i32,
                    terminated: bool,
                }

                let mut beams = vec![BeamHypothesis {
                    tokens: Vec::new(),
                    score: 0,
                    terminated: false,
                }];

                // #785 C3 witness counters, classified per candidate query
                // by the #785-C1 buffer contract: node-evidence tiers
                // publish at least one node score; everything else (row
                // hit, converter last resort, or no candidate) leaves the
                // buffer all-MIN.
                let mut non_node_queries = 0usize;
                let mut node_path_queries = 0usize;

                let steps = max_tokens.min(MAX_CHAT_TOKENS);
                for _ in 0..steps {
                    let mut all_candidates = Vec::new();
                    let mut any_active = false;
                    let mut replayed_abstention = None;

                    for beam in &beams {
                        if beam.terminated {
                            all_candidates.push(BeamHypothesis {
                                tokens: beam.tokens.clone(),
                                score: beam.score,
                                terminated: true,
                            });
                            continue;
                        }
                        any_active = true;

                        let mut beam_history = history[..*history_len].to_vec();
                        beam_history.extend_from_slice(&beam.tokens);

                        let candidates = match replay_normative_chat_step(
                            &mut policy_engine,
                            &r4g1,
                            &mut node_scores,
                            &beam_history,
                            Some(&session_signature),
                        )? {
                            Some(NormativeServingDecision::Serve(serve)) => serve.candidates,
                            Some(NormativeServingDecision::Abstain(outcome)) => {
                                replayed_abstention
                                    .get_or_insert_with(|| (outcome, beam.tokens.clone()));
                                continue;
                            }
                            Some(NormativeServingDecision::Decline(_)) => {
                                return Err(ChatError::Io(std::io::Error::other(
                                    "D4 permitted CLI beam but R4G1Runtime produced no candidate",
                                )));
                            }
                            None => {
                                // Explicit research-era fallback: candidate
                                // authority remains the normative runtime,
                                // but no D4 object exists to replay or commit.
                                let len = core::cmp::min(beam_history.len(), compiler::WINDOW);
                                let window = &beam_history[beam_history.len() - len..];
                                let bundle = runtime::bundle_window_plain(artifacts, &rot, window);
                                let sig = runtime::sig_plain(artifacts, &bundle);

                                // #785 C1: reset per-query scratch so one
                                // beam's evidence never leaks into another.
                                node_scores.fill(ScoreQ::MIN);
                                r4g1.predict_served_candidates_with_signature_lanes(
                                    &beam_history,
                                    Some(&sig),
                                    Some(&session_signature),
                                    &mut node_scores,
                                )
                            }
                        };

                        if node_scores
                            .iter()
                            .all(|score| score.raw() == ScoreQ::MIN.raw())
                        {
                            non_node_queries += 1;
                        } else {
                            node_path_queries += 1;
                        }

                        for candidate in candidates.ranked() {
                            let cand_tok = candidate.token;
                            let cand_score = candidate.score;
                            let is_eos = cand_tok == 0 || cand_tok == 2;
                            let mut new_tokens = beam.tokens.clone();

                            let mut repeat_count = 0i32;
                            for &t in new_tokens.iter().rev() {
                                if t == cand_tok {
                                    repeat_count += 1;
                                } else {
                                    break;
                                }
                            }
                            let repeat_penalty = if repeat_count > 0 {
                                repeat_count * 3000
                            } else {
                                0
                            };

                            let adjusted_score = beam
                                .score
                                .saturating_add(cand_score.raw())
                                .saturating_sub(repeat_penalty);
                            new_tokens.push(cand_tok);

                            all_candidates.push(BeamHypothesis {
                                tokens: new_tokens,
                                score: adjusted_score,
                                terminated: is_eos,
                            });
                        }
                    }

                    if !any_active {
                        break;
                    }
                    if all_candidates.is_empty() {
                        if let Some((mut outcome, blocked_prefix)) = replayed_abstention {
                            if policy_engine.is_some() {
                                let mut committed_context = history[..*history_len].to_vec();
                                commit_normative_chat_beam_prefix(
                                    &mut policy_engine,
                                    &r4g1,
                                    &mut node_scores,
                                    &mut committed_context,
                                    &blocked_prefix,
                                    Some(&session_signature),
                                )?;
                                outcome = match normative_chat_step(
                                    &mut policy_engine,
                                    &r4g1,
                                    &mut node_scores,
                                    &committed_context,
                                    Some(&session_signature),
                                )? {
                                    Some(NormativeServingDecision::Abstain(committed)) => committed,
                                    Some(NormativeServingDecision::Serve(_)) => {
                                        return Err(ChatError::Io(std::io::Error::other(
                                            "CLI beam replay/commit diverged at its terminal abstention",
                                        )));
                                    }
                                    Some(NormativeServingDecision::Decline(_)) => {
                                        return Err(ChatError::Io(std::io::Error::other(
                                            "D4 permitted CLI beam but R4G1Runtime produced no candidate",
                                        )));
                                    }
                                    None => {
                                        return Err(ChatError::Io(std::io::Error::other(
                                            "CLI beam lost its production policy adapter before abstention commit",
                                        )));
                                    }
                                };
                            }
                            return Ok(abstention_answer(
                                "r4g1-beam",
                                non_node_queries,
                                node_path_queries,
                                chat_abstention(outcome),
                            ));
                        }
                        break;
                    }

                    all_candidates.sort_by_key(|b| std::cmp::Reverse(b.score));
                    all_candidates.truncate(4);
                    beams = all_candidates;
                }

                let best_beam = beams
                    .into_iter()
                    .max_by_key(|b| b.score)
                    .unwrap_or_else(|| BeamHypothesis {
                        tokens: Vec::new(),
                        score: 0,
                        terminated: false,
                    });

                let generated_tokens_buf = best_beam.tokens;
                let generated = generated_tokens_buf.as_slice();

                // Speculation above retained no D4 semantic state. Commit the
                // exact winning path now, and prove each beam-selected token
                // belonged to that step's runtime-owned shortlist before any
                // output reaches history or the caller.
                if policy_engine.is_some() {
                    let mut committed_context = history[..*history_len].to_vec();
                    commit_normative_chat_beam_prefix(
                        &mut policy_engine,
                        &r4g1,
                        &mut node_scores,
                        &mut committed_context,
                        generated,
                        Some(&session_signature),
                    )?;
                }
                append_history(history, history_len, generated);
                if generated.is_empty() {
                    return Err(ChatError::EmptyGeneration);
                }
                let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
                let answer_len = tokenizer
                    .decode_into(generated, &mut answer_bytes)
                    .ok_or(ChatError::EmptyGeneration)?;
                let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
                    .trim()
                    .to_owned();
                if text.is_empty() {
                    return Err(ChatError::EmptyGeneration);
                }
                let witness = DecodeWitness {
                    engine: "r4g1-beam",
                    non_node_queries,
                    node_path_queries,
                    plain_depths: Vec::new(),
                };
                tracing::info!(
                    engine = witness.engine,
                    non_node_queries,
                    node_path_queries,
                    "decode witness (#785 C3)"
                );
                tracing::debug!(generated_tokens = generated.len(), "R4G1 answer generated");
                return Ok(ChatAnswer {
                    text,
                    generated_tokens: generated.len(),
                    repeated_token_rate: recent_window_repetition_rate(generated, generated.len()),
                    witness,
                    abstention: None,
                });
            }
        }
    }

    let mut runtime = Runtime::new(artifacts);
    let mut predictions = [runtime::Prediction::default(); MAX_CHAT_TOKENS];
    let sampled = sample_rng.is_some();
    let prediction_count = match sample_rng {
        // Issue #762 lever 2: opt-in weighted sampling on this plain
        // path. The R4G1 path above applies the same seed via its own
        // #785-C2 sampled decode and always returns before this point
        // when a graph is present, so a given turn samples on exactly
        // one path (see the builder doc comment).
        Some(rng) => runtime.generate_sampled_into(
            store,
            &history[..*history_len],
            rng,
            &mut predictions[..max_tokens.min(MAX_CHAT_TOKENS)],
        ),
        None => runtime.generate_greedy_into(
            store,
            &history[..*history_len],
            &mut predictions[..max_tokens.min(MAX_CHAT_TOKENS)],
        ),
    };
    let mut generated = [0u32; MAX_CHAT_TOKENS];
    let mut generated_count = 0usize;
    for prediction in &predictions[..prediction_count] {
        if prediction.token == 1 {
            break;
        }
        generated[generated_count] = prediction.token;
        generated_count += 1;
        if repeated_suffix(&generated[..generated_count], 8) {
            generated_count -= 1;
            break;
        }
    }
    let generated = &generated[..generated_count];
    if generated.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
    let answer_len = tokenizer
        .decode_into(generated, &mut answer_bytes)
        .ok_or(ChatError::EmptyGeneration)?;
    let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
        .trim()
        .to_owned();
    if text.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    append_history(history, history_len, generated);
    // #785 C3: the per-token resolution depths were always computed by
    // the plain runtime and then discarded here; carry them out.
    let witness = DecodeWitness {
        engine: if sampled {
            "tla-plain-sampled"
        } else {
            "tla-plain-greedy"
        },
        non_node_queries: 0,
        node_path_queries: 0,
        plain_depths: predictions[..generated.len()]
            .iter()
            .map(|prediction| prediction.depth)
            .collect(),
    };
    tracing::info!(
        engine = witness.engine,
        depth_min = witness.plain_depths.iter().min().copied().unwrap_or(0),
        depth_max = witness.plain_depths.iter().max().copied().unwrap_or(0),
        "decode witness (#785 C3)"
    );
    tracing::debug!(generated_tokens = generated.len(), "answer generated");
    Ok(ChatAnswer {
        text,
        generated_tokens: generated.len(),
        repeated_token_rate: recent_window_repetition_rate(generated, generated.len()),
        witness,
        abstention: None,
    })
}

fn ensure_chat_prompt_encoder(tokenizer: &Tokenizer) -> Result<(), ChatError> {
    match tokenizer.adapter_key() {
        Some((family, version)) if tokenizer.is_decode_only() => {
            Err(ChatError::TokenizerUnavailable {
                family: family.to_owned(),
                version,
            })
        }
        _ => Ok(()),
    }
}

fn invalid_chat_data(message: impl Into<String>) -> ChatError {
    ChatError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

/// Resolve the model directory's servable R4G1 graph, preferring the
/// scored artifact over the converter carryover.
///
/// `graph/score.r4g1` is the graph the HTTP server (`primary_graph`) and
/// the release-bundle packager (`GRAPH_RELATIVE_PATH`) already treat as
/// the servable one: it carries per-node emissions, context rows, and
/// residual EXCT tables. `compiled.r4g1` is the converter carryover
/// (empty per-node emissions, raw TLS1 EXCT) and stays as the fallback
/// for bundles that never ran score (#785 C1c).
fn read_preferred_chat_graph(directory: &std::path::Path) -> Result<Option<Vec<u8>>, ChatError> {
    if let Some(bytes) = read_optional_chat_file(&directory.join("graph/score.r4g1"))? {
        return Ok(Some(bytes));
    }
    read_optional_chat_file(&directory.join("compiled.r4g1"))
}

fn read_optional_chat_file(path: &std::path::Path) -> Result<Option<Vec<u8>>, ChatError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ChatError::Io(error)),
    };
    let is_regular = if metadata.file_type().is_symlink() {
        std::fs::metadata(path)?.is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(invalid_chat_data(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    std::fs::read(path).map(Some).map_err(ChatError::Io)
}

fn validate_chat_r4g1_structure(graph: Option<&[u8]>) -> Result<(), ChatError> {
    let Some(graph) = graph else {
        return Ok(());
    };
    let view = uor_r4_graph_format::GraphView::parse(graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.reason)))?;
    view.verify_cids()
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.as_format())))?;
    if view.head().is_none() {
        return Err(invalid_chat_data(
            "invalid R4G1 graph: missing HEAD section",
        ));
    }
    uor_r4_graph_runtime::R4G1Runtime::parse(graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 runtime: {}", error.reason)))?;
    Ok(())
}

fn bind_chat_r4g1(
    graph: Option<Vec<u8>>,
    tokenizer_bytes: &[u8],
) -> Result<Option<Vec<u8>>, ChatError> {
    let Some(graph) = graph else {
        return Ok(None);
    };
    let view = uor_r4_graph_format::GraphView::parse(&graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.reason)))?;
    view.verify_cids()
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.as_format())))?;
    let expected = view
        .head()
        .ok_or_else(|| invalid_chat_data("invalid R4G1 graph: missing HEAD section"))?
        .tokenizer_cid()
        .0;
    let actual = blake3::hash(tokenizer_bytes);
    if expected != [0; 32] && expected != *actual.as_bytes() {
        return Err(invalid_chat_data(format!(
            "R4G1 tokenizer CID mismatch: header expected blake3:{}, selected blake3:{actual}",
            blake3::Hash::from(expected).to_hex()
        )));
    }
    if expected == [0; 32] && Tokenizer::is_tagged_container_bytes(tokenizer_bytes) {
        return Err(invalid_chat_data(
            "a tagged tokenizer requires a nonzero R4G1 header tokenizer CID",
        ));
    }
    Ok(Some(graph))
}

/// Validate and retain the exact tokenizer object selected by the bundle.
/// Present bytes are never replaced by `/tmp/ref/tokenizer.bin`: doing so
/// would write different content under the manifest CID and could cross
/// tokenizer id spaces.
fn select_chat_tokenizer_bytes(bundled: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    if Tokenizer::is_tagged_container_bytes(bundled) {
        let tokenizer = Tokenizer::from_bytes(bundled).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed tagged tokenizer.bin",
            )
        })?;
        if !tokenizer.is_decode_only() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "tagged tokenizer.bin did not declare decode-only semantics",
            ));
        }
        return Ok(bundled.to_vec());
    }
    if Tokenizer::from_bytes(bundled).is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "malformed tokenizer.bin",
        ));
    }
    Ok(bundled.to_vec())
}

/// Compatibility path for historical 32k bundles whose content-addressed
/// object is temporarily unavailable. The old `/tmp/ref` shortcut is admitted
/// only when its bytes exactly match the manifest object, so an unavailable
/// tagged object can never be replaced by unrelated legacy bytes.
fn matching_ref_tokenizer(
    expected: &ModelObject,
    r4g1_bytes: Option<&[u8]>,
) -> Result<Option<Vec<u8>>, ChatError> {
    let is_32k_graph = r4g1_bytes.is_some_and(|bytes| {
        uor_r4_graph_runtime::R4G1Runtime::parse(bytes)
            .is_ok_and(|runtime| runtime.node_count() > 0)
    });
    if !is_32k_graph {
        return Ok(None);
    }
    let Some(bytes) = read_optional_chat_file(std::path::Path::new("/tmp/ref/tokenizer.bin"))?
    else {
        return Ok(None);
    };
    if u64::try_from(bytes.len()).ok() != Some(expected.bytes) {
        return Ok(None);
    }
    let actual = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    Ok((actual == expected.cid).then_some(bytes))
}

fn append_history(history: &mut [u32; MAX_CHAT_HISTORY], len: &mut usize, tokens: &[u32]) {
    let tokens = &tokens[tokens.len().saturating_sub(MAX_CHAT_HISTORY)..];
    let overflow = len
        .saturating_add(tokens.len())
        .saturating_sub(MAX_CHAT_HISTORY);
    if overflow > 0 {
        history.copy_within(overflow..*len, 0);
        *len -= overflow;
    }
    history[*len..*len + tokens.len()].copy_from_slice(tokens);
    *len += tokens.len();
}

fn repeated_suffix(tokens: &[u32], width: usize) -> bool {
    if tokens.len() < width * 2 {
        return false;
    }
    let suffix = &tokens[tokens.len() - width..];
    tokens[..tokens.len() - width]
        .windows(width)
        .any(|window| window == suffix)
}

/// Fraction of `tokens` that already appeared among the preceding
/// `window` tokens of the same generation — the same recent-window
/// repetition definition Gate C's `generate_greedy_repetition_rate` uses
/// (`uor-r4-graph-certify::score`), reimplemented here for the live
/// serving path (`chat`'s `Compiled`/`Store` types are private to
/// `uor-r4-core::transformerless`, a different crate boundary than
/// graph-certify's `GraphScorer`, so the definition is duplicated rather
/// than shared code — see `#744`). Empty input is defined as
/// non-repetitive (`0.0`), matching the natural reading of "no
/// repetition observed" rather than the alternative "undefined".
fn recent_window_repetition_rate(tokens: &[u32], window: usize) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let mut recent: std::collections::VecDeque<u32> =
        std::collections::VecDeque::with_capacity(window);
    let mut duplicate_count = 0usize;
    for &token in tokens {
        if recent.contains(&token) {
            duplicate_count += 1;
        }
        if recent.len() == window {
            recent.pop_front();
        }
        recent.push_back(token);
    }
    duplicate_count as f64 / tokens.len() as f64
}

/// Build a chat engine directly from raw artifact/store/tokenizer bytes,
/// bypassing manifest lookup and the on-disk compiled-bundle directory
/// convention (`build_local_compiled_engine`'s fixed file names). Used by
/// `model::evaluate_live_quality` (`#744`) to measure generation quality
/// against the exact bytes being imported, before any manifest or
/// `.uor-models/compiled/<name>` directory exists for them. No R4G1
/// graph is attempted (`r4g1_bytes: None`) — this always exercises the
/// plain TLA/TLS1 runtime path, the common baseline every bundle must
/// clear regardless of which runtime ultimately serves it, and the one
/// whose repetition guard (`repeated_suffix`) is weaker, making it the
/// conservative choice for a pass/fail gate.
///
/// Thin wrapper over [`engine_from_bytes_with_r4g1`] with `r4g1_bytes:
/// None` — kept as its own function so existing callers that only ever
/// want the plain path (and any external callers of this crate) are
/// unaffected by #750's addition of R4G1-path probing.
pub fn engine_from_bytes(
    artifact_bytes: &[u8],
    store_bytes: &[u8],
    tokenizer_bytes: &[u8],
    max_tokens: usize,
) -> Result<ChatEngine, ChatError> {
    engine_from_bytes_with_r4g1(
        artifact_bytes,
        store_bytes,
        tokenizer_bytes,
        None,
        max_tokens,
    )
}

/// Same as [`engine_from_bytes`], but accepts an optional R4G1 graph
/// (`compiled.r4g1` bytes) so a caller — currently only
/// `model::evaluate_live_quality` (#750) — can construct an engine that
/// exercises the R4G1 beam-search path (`hologram_answer`'s `r4g1_bytes:
/// Some(...)` branch) instead of the plain TLA/TLS1 path, the same
/// validation and CID-binding `ChatEngineBuilder::build()` and
/// `build_local_compiled_engine` apply to a discovered
/// `compiled.r4g1` file.
pub fn engine_from_bytes_with_r4g1(
    artifact_bytes: &[u8],
    store_bytes: &[u8],
    tokenizer_bytes: &[u8],
    r4g1_bytes: Option<&[u8]>,
    max_tokens: usize,
) -> Result<ChatEngine, ChatError> {
    let artifacts = compiler::parse_artifacts(artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = runtime::parse_store(store_bytes).ok_or(ChatError::InvalidStore)?;
    validate_chat_r4g1_structure(r4g1_bytes)?;
    let r4g1_bytes = bind_chat_r4g1(r4g1_bytes.map(<[u8]>::to_vec), tokenizer_bytes)?;
    let tokenizer = parse_chat_tokenizer(tokenizer_bytes)?;
    Ok(ChatEngine {
        artifacts,
        store,
        r4g1_bytes,
        tokenizer,
        history: [0; MAX_CHAT_HISTORY],
        history_len: 0,
        max_tokens: max_tokens.clamp(1, MAX_CHAT_TOKENS),
        // The live quality gate (#750) must stay fully deterministic --
        // this constructor has no seed input, so sampling is never on.
        sample_rng: None,
        // #811: deliberately ungated. This constructor exists for the
        // #750 live quality gate, which measures the raw decode; wiring
        // the D4 abstention gate here would silently change what that
        // gate measures. The product ask/chat surfaces (builder +
        // local-compiled paths) are the gated ones.
        policy_engine: None,
    })
}

fn parse_chat_tokenizer(bytes: &[u8]) -> Result<Tokenizer, ChatError> {
    Tokenizer::from_bytes(bytes).ok_or_else(|| {
        ChatError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid tokenizer.bin bytes",
        ))
    })
}

struct SlashCommandDef {
    cmd: &'static str,
    desc: &'static str,
}

const COMMAND_DEFS: &[SlashCommandDef] = &[
    SlashCommandDef {
        cmd: "/help",
        desc: "Display available client slash commands",
    },
    SlashCommandDef {
        cmd: "/status",
        desc: "View R4G1 sub-millisecond 4-stage pipeline readiness",
    },
    SlashCommandDef {
        cmd: "/models",
        desc: "List supported teacher models & disk compilation status",
    },
    SlashCommandDef {
        cmd: "/switch",
        desc: "Dynamically switch active teacher model in-session",
    },
    SlashCommandDef {
        cmd: "/engine",
        desc: "Select synthesis engine (r4g1, attention, r4-attention, geometric)",
    },
    SlashCommandDef {
        cmd: "/corpus",
        desc: "Manage extra reading corpus datasets & server index",
    },
    SlashCommandDef {
        cmd: "/compile",
        desc: "Trigger full automated 4-stage graph compilation",
    },
    SlashCommandDef {
        cmd: "/audit",
        desc: "Audit Q&A token trace, UOR coordinates & R4 geometry",
    },
    SlashCommandDef {
        cmd: "/clear",
        desc: "Clear terminal screen & session history",
    },
    SlashCommandDef {
        cmd: "/reset",
        desc: "Reset history, corpus & geometric manifold state back to base",
    },
    SlashCommandDef {
        cmd: "/export",
        desc: "Export manifold state & corpus to .uor-models/exported/exported_manifold.json",
    },
    SlashCommandDef {
        cmd: "/quit",
        desc: "Exit client session",
    },
    SlashCommandDef {
        cmd: "/exit",
        desc: "Exit client session",
    },
];
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
use rustyline::completion::Completer;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::highlight::Highlighter;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::hint::Hinter;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::validate::Validator;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::Helper;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct SlashCommandHelper;

#[cfg(not(target_arch = "wasm32"))]
impl Completer for SlashCommandHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        if line.starts_with('/') {
            let candidates: Vec<String> = COMMAND_DEFS
                .iter()
                .filter(|d| d.cmd.starts_with(&line[..pos]))
                .map(|d| d.cmd.to_string())
                .collect();
            Ok((0, candidates))
        } else {
            Ok((0, Vec::new()))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Hinter for SlashCommandHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Highlighter for SlashCommandHelper {}
#[cfg(not(target_arch = "wasm32"))]
impl Validator for SlashCommandHelper {}
#[cfg(not(target_arch = "wasm32"))]
impl Helper for SlashCommandHelper {}

fn read_line_with_history<W: Write>(
    prompt: &str,
    history: &mut Vec<String>,
    input: &mut impl BufRead,
    output: &mut W,
) -> Result<Option<String>, std::io::Error> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if unsafe { libc::isatty(libc::STDIN_FILENO) } != 0 {
            let config = rustyline::Config::builder()
                .auto_add_history(true)
                .completion_type(rustyline::CompletionType::List)
                .build();
            let mut rl = rustyline::Editor::<SlashCommandHelper, _>::with_config(config)
                .map_err(std::io::Error::other)?;
            rl.set_helper(Some(SlashCommandHelper));
            for entry in history.iter() {
                let _ = rl.add_history_entry(entry);
            }

            match rl.readline(prompt) {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
                    if !trimmed.is_empty() {
                        history.push(trimmed.clone());
                    }
                    return Ok(Some(trimmed));
                }
                Err(rustyline::error::ReadlineError::Interrupted) => return Ok(None),
                Err(rustyline::error::ReadlineError::Eof) => return Ok(None),
                Err(e) => return Err(std::io::Error::other(e)),
            }
        }
    }

    write!(output, "{}", prompt)?;
    output.flush()?;

    let mut line_bytes = Vec::new();
    loop {
        let mut buf = [0u8; 1];
        let n = input.read(&mut buf)?;
        if n == 0 {
            if line_bytes.is_empty() {
                return Ok(None);
            }
            break;
        }
        let b = buf[0];
        if b == b'\r' || b == b'\n' {
            break;
        }
        line_bytes.push(b);
    }

    let line = String::from_utf8_lossy(&line_bytes);
    let trimmed = line.trim().to_string();
    if !trimmed.is_empty() {
        history.push(trimmed.clone());
    }
    Ok(Some(trimmed))
}

fn select_menu_interactive<W: Write>(
    title: &str,
    options: &[(&str, &str)],
    output: &mut W,
) -> Result<Option<usize>, std::io::Error> {
    writeln!(output, "\n\x1b[1m{}\x1b[0m", title)?;
    for (idx, (item_name, item_desc)) in options.iter().enumerate() {
        writeln!(
            output,
            "  \x1b[1;36m[{}]\x1b[0m \x1b[1m{:<24}\x1b[0m {}",
            idx + 1,
            item_name,
            item_desc
        )?;
    }
    write!(
        output,
        "\x1b[1;33mSelect option [1-{}]: \x1b[0m",
        options.len()
    )?;
    output.flush()?;

    let mut history_dummy = Vec::new();
    let mut stdin_buf = std::io::BufReader::new(std::io::stdin());
    let resp = read_line_with_history("", &mut history_dummy, &mut stdin_buf, output)?;
    if let Some(line) = resp {
        let trimmed = line.trim();
        if let Ok(num) = trimmed.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                return Ok(Some(num - 1));
            }
        }
        for (idx, (item_name, _)) in options.iter().enumerate() {
            if item_name
                .trim_start_matches('/')
                .eq_ignore_ascii_case(trimmed.trim_start_matches('/'))
            {
                return Ok(Some(idx));
            }
        }
    }
    Ok(None)
}

fn check_model_artifact_status(model_id: &str) -> (bool, bool) {
    let target_key = match model_id {
        "smollm2-135m-instruct" => "smollm2-135m",
        "smollm2-360m-instruct" => "smollm2-360m",
        "smollm2-1-7b-instruct" => "smollm2-1-7b",
        other => other,
    };

    let downloaded = if let Ok(entries) = std::fs::read_dir(".uor-models/sources") {
        entries.filter_map(|e| e.ok()).any(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            entry.path().is_dir() && name.contains(target_key)
        })
    } else {
        false
    };

    let compiled = if let Ok(entries) = std::fs::read_dir(".uor-models/compiled") {
        entries.filter_map(|e| e.ok()).any(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            // #655-C1e: one recognition predicate, shared with the model
            // store — this probe's previous any-single-file check drifted
            // from `ModelStore`'s triple (a dir with only
            // `tless_artifacts.bin` counted here but served nothing).
            name.contains(target_key) && crate::model::is_compiled_bundle(&path)
        })
    } else {
        false
    };

    (downloaded, compiled)
}

fn trigger_in_client_compilation<W: Write>(
    target_model: &str,
    host: &str,
    port: u16,
    output: &mut W,
) -> Result<bool, std::io::Error> {
    let (repo, rev) = match target_model {
        "smollm2-360m-instruct" => (
            "HuggingFaceTB/SmolLM2-360M-Instruct",
            "9d9ff7299a9a3b6d289ff100d0246a48d88c0326",
        ),
        // #790: pinned 40-hex revision — the download path refuses branch
        // names by design, so "main" made this arm unreachable on a fresh
        // machine. Resolved from the repo's main on 2026-08-18.
        "smollm2-1-7b-instruct" => (
            "HuggingFaceTB/SmolLM2-1.7B-Instruct",
            "31b70e2e869a7173562077fd711b654946d38674",
        ),
        _ => (
            "HuggingFaceTB/SmolLM2-135M-Instruct",
            "7e27bd9f95328f0f3b08261d1252705110c806f8",
        ),
    };

    let r4_exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./target/release/r4"));
    let source_dir = format!(".uor-models/sources/{}", target_model);
    let compiled_dir = format!(".uor-models/compiled/{}", target_model);
    let graph_dir = format!("{}/graph", compiled_dir);
    let score_file = format!("{}/score.r4g1", graph_dir);

    writeln!(
        output,
        "\n\x1b[1;36m[*] Initiating automated 4-stage graph compilation for '{}'...\x1b[0m",
        target_model
    )?;

    // Stage 1: Download if missing
    if !std::path::Path::new(&source_dir).is_dir() {
        writeln!(
            output,
            "[*] [Stage 1/4] Downloading HF teacher weights ({})",
            repo
        )?;
        output.flush()?;
        let status = std::process::Command::new(&r4_exe)
            .args([
                "download",
                "--repository",
                repo,
                "--revision",
                rev,
                "--name",
                target_model,
            ])
            .status()?;
        if !status.success() {
            writeln!(output, "\x1b[31m[!] Stage 1 download failed.\x1b[0m")?;
            return Ok(false);
        }
    } else {
        writeln!(
            output,
            "\x1b[32m[✓] [Stage 1/4] Pinned teacher source ready: {}\x1b[0m",
            source_dir
        )?;
    }

    // Stage 2: Compile observation corpus. Projection ownership is fixed by
    // the loaded architecture; the legacy --exact-scalar switch is a no-op.
    writeln!(
        output,
        "[*] [Stage 2/4] Compiling zero-multiply observation corpus (exact uor-matmul projections; certified-exact current attention)..."
    )?;
    output.flush()?;
    std::fs::create_dir_all(&compiled_dir).ok();
    std::fs::create_dir_all(&graph_dir).ok();

    let (target_tokens, compile_seconds) = match target_model {
        "smollm2-135m-instruct" => ("176800", "300"),
        "smollm2-360m-instruct" => ("500000", "600"),
        "smollm2-1-7b-instruct" => ("1768000", "1200"),
        _ => ("176800", "300"),
    };

    let compile_cmd_args = vec![
        "compile".to_string(),
        "--source".to_string(),
        source_dir.clone(),
        "--output".to_string(),
        compiled_dir.clone(),
        "--seconds".to_string(),
        compile_seconds.to_string(),
        "--target".to_string(),
        target_tokens.to_string(),
        "--sequence-length".to_string(),
        "128".to_string(),
    ];
    let status = std::process::Command::new(&r4_exe)
        .args(&compile_cmd_args)
        .status()?;
    if !status.success() {
        writeln!(
            output,
            "\x1b[31m[!] Stage 2 bundle compilation failed.\x1b[0m"
        )?;
        return Ok(false);
    }
    writeln!(
        output,
        "\x1b[32m[✓] [Stage 2/4] Transformerless bundle compiled successfully.\x1b[0m"
    )?;

    // Stage 3: Score residual graph
    writeln!(
        output,
        "[*] [Stage 3/4] Inducing multiresolution cover & scoring R4G1 residual graph..."
    )?;
    output.flush()?;
    let c_meta = if std::path::Path::new(&format!("{}/corpus.meta", compiled_dir)).is_file() {
        format!("{}/corpus.meta", compiled_dir)
    } else {
        format!("{}/c_meta.bin", compiled_dir)
    };

    let c_recs = if std::path::Path::new(&format!("{}/corpus.records", compiled_dir)).is_file() {
        format!("{}/corpus.records", compiled_dir)
    } else {
        format!("{}/c_recs.bin", compiled_dir)
    };
    let tless_artifacts = format!("{}/tless_artifacts.bin", compiled_dir);

    let mut status = std::process::Command::new(&r4_exe)
        .args([
            "transformerless",
            "score",
            "--corpus-meta",
            &c_meta,
            "--corpus-recs",
            &c_recs,
            "--artifacts",
            &tless_artifacts,
            "--quality-profile",
            "relative_tla",
            "--out",
            &graph_dir,
        ])
        .status()?;

    // If corpus was incomplete, re-run compile to finish remaining tokens then retry score
    if !status.success() {
        writeln!(
            output,
            "\x1b[33m[*] Finishing corpus generation to complete all required tokens...\x1b[0m"
        )?;
        output.flush()?;
        let _ = std::process::Command::new(&r4_exe)
            .args([
                "compile",
                "--source",
                &source_dir,
                "--output",
                &compiled_dir,
                "--seconds",
                compile_seconds,
                "--target",
                target_tokens,
                "--sequence-length",
                "128",
            ])
            .status()?;
        status = std::process::Command::new(&r4_exe)
            .args([
                "transformerless",
                "score",
                "--corpus-meta",
                &c_meta,
                "--corpus-recs",
                &c_recs,
                "--artifacts",
                &tless_artifacts,
                "--quality-profile",
                "relative_tla",
                "--out",
                &graph_dir,
            ])
            .status()?;
    }

    if !status.success() {
        writeln!(output, "\x1b[31m[!] Stage 3 graph scoring failed.\x1b[0m")?;
        return Ok(false);
    }
    writeln!(
        output,
        "\x1b[32m[✓] [Stage 3/4] Scored R4G1 residual graph ready: {}\x1b[0m",
        score_file
    )?;

    // Stage 4: Reload server
    writeln!(
        output,
        "[*] [Stage 4/4] Reloading server runtime with new R4G1 graph..."
    )?;
    output.flush()?;
    let req_body = serde_json::json!({ "model": target_model });
    match send_server_post_request(host, port, "/v1/reload", &req_body) {
        Ok(res) if res["status"] == "success" => {
            writeln!(
                output,
                "\x1b[1;32m[+] Compilation complete! Successfully loaded model '{}' in-session.\x1b[0m\n",
                target_model
            )?;
            Ok(true)
        }
        _ => {
            writeln!(
                output,
                "\x1b[31m[!] Server reload failed after compilation.\x1b[0m\n"
            )?;
            Ok(false)
        }
    }
}

fn handle_model_switch_with_remediation<W: Write>(
    target_model: &str,
    host: &str,
    port: u16,
    current_active_model: &mut String,
    current_active_engine: &mut String,
    output: &mut W,
) -> Result<(), std::io::Error> {
    writeln!(
        output,
        "\n[*] Requesting in-session server reload for model '{}'...",
        target_model
    )?;
    let req_body = serde_json::json!({ "model": target_model });
    match send_server_post_request(host, port, "/v1/reload", &req_body) {
        Ok(res) => {
            if res["status"] == "success" {
                *current_active_model = target_model.to_string();
                if let Err(error) = persist_state_file(
                    &crate::model::store_state_file("last_model_name.txt"),
                    current_active_model.as_str(),
                ) {
                    writeln!(
                        output,
                        "\x1b[33m[-] model preference not persisted: {error}\x1b[0m"
                    )?;
                }
                writeln!(
                    output,
                    "\x1b[32m[+] {}\x1b[0m\n",
                    res["message"]
                        .as_str()
                        .unwrap_or("Model reloaded successfully")
                )?;
            } else {
                let err_msg = res["message"].as_str().unwrap_or("Failed to reload model");
                writeln!(
                    output,
                    "\n\x1b[1;31m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m│ [!] MODEL RELOAD FAILURE & DIAGNOSTIC REMEDIATION:                         │\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m│\x1b[0m Target Model : \x1b[1m{:<60}\x1b[0m \x1b[1;31m│\x1b[0m",
                    target_model
                )?;
                let display_err = if err_msg.len() > 60 {
                    &err_msg[..60]
                } else {
                    err_msg
                };
                writeln!(
                    output,
                    "\x1b[1;31m│\x1b[0m Error        : \x1b[33m{:<60}\x1b[0m \x1b[1;31m│\x1b[0m",
                    display_err
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n"
                )?;

                let remediation_options = [
                    (
                        "1) Re-compile Model Graph",
                        "Re-run 4-stage compilation in-client to fix CID mismatch / out-of-date graph",
                    ),
                    (
                        "2) Switch to Oracle Mode",
                        "Switch engine to 'attention' oracle mode (runs model without graph)",
                    ),
                    (
                        "3) Keep Active Model",
                        "Cancel reload and stay on current working model",
                    ),
                ];

                if let Ok(Some(rem_idx)) = select_menu_interactive(
                    "Select Remediation Action:",
                    &remediation_options,
                    output,
                ) {
                    match rem_idx {
                        0 => {
                            if trigger_in_client_compilation(target_model, host, port, output)
                                .unwrap_or(false)
                            {
                                *current_active_model = target_model.to_string();
                                *current_active_engine = "r4g1".to_string();
                                if let Err(error) = persist_state_file(
                                    &crate::model::store_state_file("last_model_name.txt"),
                                    current_active_model.as_str(),
                                )
                                .and_then(|()| {
                                    persist_state_file(
                                        &crate::model::store_state_file("last_engine.txt"),
                                        current_active_engine.as_str(),
                                    )
                                }) {
                                    writeln!(
                                        output,
                                        "\x1b[33m[-] preference not persisted: {error}\x1b[0m"
                                    )?;
                                }
                            }
                        }
                        1 => {
                            *current_active_engine = "attention".to_string();
                            if let Err(error) = persist_state_file(
                                &crate::model::store_state_file("last_engine.txt"),
                                current_active_engine.as_str(),
                            ) {
                                writeln!(
                                    output,
                                    "\x1b[33m[-] engine preference not persisted: {error}\x1b[0m"
                                )?;
                            }
                            writeln!(
                                output,
                                "\x1b[32m[+] Engine switched to 'attention' oracle fallback mode.\x1b[0m\n"
                            )?;
                        }
                        _ => {
                            writeln!(
                                output,
                                "[*] Staying on active model: {}\n",
                                current_active_model
                            )?;
                        }
                    }
                }
            }
        }
        Err(e) => {
            writeln!(output, "[!] Error communicating with server: {}\n", e)?;
        }
    }
    Ok(())
}

/// #790 item 8: persist a small CLI state file atomically and surface the
/// outcome instead of swallowing it.
///
/// The previous `let _ = std::fs::write(...)` sites reported success in
/// the UI while nothing persisted (read-only store, missing directory,
/// full disk), and a crash mid-write could leave a half-written
/// preference for every later request to read. Writes go to a
/// same-directory temp file and rename over the target (atomic on one
/// filesystem); callers report a failure to the user.
fn persist_state_file(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    let directory = path.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(directory) = directory {
        std::fs::create_dir_all(directory)?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state");
    let temp = path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    std::fs::write(&temp, contents)?;
    match std::fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temp);
            Err(error)
        }
    }
}

/// Run an interactive client chat session against a remote local HTTP vendor endpoint.
pub fn remote_interactive_chat(
    remote_url: &str,
    model: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let (host, port, path) = parse_remote_url(remote_url);

    // Read initial active model and engine from disk if present
    let mut current_active_model = if let Ok(m) =
        std::fs::read_to_string(crate::model::store_state_file("last_model_name.txt"))
    {
        let trimmed = m.trim().to_string();
        if !trimmed.is_empty() {
            trimmed
        } else {
            model.to_string()
        }
    } else {
        model.to_string()
    };

    let mut current_active_engine =
        if let Ok(e) = std::fs::read_to_string(crate::model::store_state_file("last_engine.txt")) {
            let trimmed = e.trim().to_string();
            if !trimmed.is_empty() {
                trimmed
            } else {
                "r4g1".to_string()
            }
        } else {
            "r4g1".to_string()
        };

    // Render Rich Intro Banner
    writeln!(output, "\x1b[1;36m")?;
    writeln!(
        output,
        "██╗   ██╗██████╗ ██████╗        ██████╗ ██╗  ██╗        ██████╗██╗     ██╗\n\
         ██║   ██║██╔══██╗██╔══██╗       ██╔══██╗██║  ██║       ██╔════╝██║     ██║\n\
         ██║   ██║██║  ██║██████╔╝ ████╗ ██████╔╝███████║ ████╗ ██║     ██║     ██║\n\
         ██║   ██║██║  ██║██╔══██╗ ╚═══╝ ██╔══██╗╚════██║ ╚═══╝ ██║     ██║     ██║\n\
         ╚██████╔╝██████╔╝██║  ██║       ██║  ██║     ██║       ╚██████╗███████╗██║\n\
          ╚═════╝ ╚═════╝ ╚═╝  ╚═╝       ╚═╝  ╚═╝     ╚═╝        ╚═════╝╚══════╝╚═╝"
    )?;
    writeln!(output, "\x1b[0m")?;
    writeln!(
        output,
        "\x1b[1mUOR-R4 Holographic Graph & Transformerless Engine v0.1.0\x1b[0m"
    )?;
    writeln!(
        output,
        "Zero-Multiply Local Intelligence Runtime • Pinned Multiplication-Free Execution\n"
    )?;
    writeln!(
        output,
        "Connected to local vendor endpoint: \x1b[36mhttp://{}:{}{}\x1b[0m",
        host, port, path
    )?;
    writeln!(
        output,
        "Active teacher model             : \x1b[32m{}\x1b[0m",
        current_active_model
    )?;
    writeln!(
        output,
        "Active synthesis engine          : \x1b[36m{}\x1b[0m\n",
        current_active_engine
    )?;
    writeln!(output, "\x1b[1mCommands & Shortcuts:\x1b[0m")?;
    writeln!(
        output,
        "  • Type \x1b[33m/help\x1b[0m to view available slash commands (/status, /models, /engine, /corpus, /export, /reset, /clear, /quit)"
    )?;
    writeln!(
        output,
        "  • Type \x1b[33m/\x1b[0m for interactive slash command suggestions & autocomplete"
    )?;
    writeln!(
        output,
        "  • Type \x1b[33mexit\x1b[0m or press \x1b[33mCtrl-D\x1b[0m to quit session\n"
    )?;
    output.flush()?;

    let mut history: Vec<String> = Vec::new();
    let mut audit_history: Vec<(String, String, Option<crate::server::UorAuditTrace>)> = Vec::new();

    loop {
        let prompt_lbl = format!(
            "\x1b[1;36mr4\x1b[0m \x1b[33m[model: {} | engine: {}]\x1b[0m \x1b[1;32m>\x1b[0m ",
            current_active_model, current_active_engine
        );
        let line_opt = match read_line_with_history(&prompt_lbl, &mut history, input, output) {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(e) => {
                writeln!(output, "[!] Input error: {}", e)?;
                break;
            }
        };

        let question = line_opt.trim();
        if matches!(question, "exit" | "quit") {
            break;
        }
        if question.is_empty() {
            continue;
        }

        if question.starts_with('/') {
            let mut input_cmd = question.trim();
            if input_cmd == "/" {
                let menu_options = [
                    ("/models", "Manage & switch active teacher model in-session"),
                    (
                        "/engine",
                        "Manage & switch synthesis engine (r4g1, attention, etc.)",
                    ),
                    (
                        "/status",
                        "View R4G1 sub-millisecond 4-stage pipeline readiness",
                    ),
                    (
                        "/corpus",
                        "Manage, import & paste extra reading corpus datasets into manifold",
                    ),
                    (
                        "/compile",
                        "Trigger full automated 4-stage graph compilation",
                    ),
                    (
                        "/audit",
                        "Audit Q&A token trace, UOR coordinates & R4 geometry",
                    ),
                    (
                        "/export",
                        "Export manifold state & corpus to .uor-models/exported/exported_manifold.json",
                    ),
                    (
                        "/reset",
                        "Reset chat history, corpus & geometric state back to base",
                    ),
                    ("/clear", "Clear terminal screen & session history"),
                    ("/quit", "Exit client session"),
                ];

                if let Ok(Some(idx)) = select_menu_interactive(
                    "R⁴ Interactive Slash Command Selector:",
                    &menu_options,
                    output,
                ) {
                    input_cmd = menu_options[idx].0;
                } else {
                    output.flush()?;
                    continue;
                }
            }

            let parts: Vec<&str> = input_cmd.split_whitespace().collect();
            let primary_token = parts[0];

            let matches: Vec<&SlashCommandDef> = COMMAND_DEFS
                .iter()
                .filter(|def| def.cmd.starts_with(primary_token))
                .collect();

            let target_cmd = match matches.len() {
                1 => matches[0].cmd,
                0 => {
                    writeln!(
                        output,
                        "[!] Unknown command: '{}'. Type '/help' or '/' for suggestions.\n",
                        input_cmd
                    )?;
                    output.flush()?;
                    continue;
                }
                _ => {
                    writeln!(output, "\nMultiple matching commands for '{}':", input_cmd)?;
                    for m in &matches {
                        writeln!(output, "  \x1b[33m{:<10}\x1b[0m - {}", m.cmd, m.desc)?;
                    }
                    writeln!(output)?;
                    output.flush()?;
                    continue;
                }
            };

            match target_cmd {
                "/help" => {
                    writeln!(output, "\n\x1b[1mR⁴ Interactive Slash Commands:\x1b[0m")?;
                    for def in COMMAND_DEFS {
                        writeln!(output, "  \x1b[33m{:<10}\x1b[0m - {}", def.cmd, def.desc)?;
                    }
                    writeln!(output)?;
                    output.flush()?;
                    continue;
                }
                "/models" | "/switch" => {
                    let target_model_opt = if parts.len() > 1 {
                        let sel = parts[1];
                        match sel {
                            "1" | "135m" => Some("smollm2-135m-instruct"),
                            "2" | "360m" => Some("smollm2-360m-instruct"),
                            "3" | "1.7b" | "1-7b" => Some("smollm2-1-7b-instruct"),
                            other => Some(other),
                        }
                    } else {
                        None
                    };

                    let target_model = match target_model_opt {
                        Some(m) => m.to_string(),
                        None => {
                            let (d1, c1) = check_model_artifact_status("smollm2-135m-instruct");
                            let (d2, c2) = check_model_artifact_status("smollm2-360m-instruct");
                            let (d3, c3) = check_model_artifact_status("smollm2-1-7b-instruct");

                            let desc1 = format!(
                                "Fast & Light (~270MB) [DL: {} | CP: {}]",
                                if d1 { "✓" } else { " " },
                                if c1 { "✓" } else { " " }
                            );
                            let desc2 = format!(
                                "Balanced Quality (~720MB) [DL: {} | CP: {}]",
                                if d2 { "✓" } else { " " },
                                if c2 { "✓" } else { " " }
                            );
                            let desc3 = format!(
                                "High-Fidelity (~3.4GB) [DL: {} | CP: {}]",
                                if d3 { "✓" } else { " " },
                                if c3 { "✓" } else { " " }
                            );

                            let model_options = [
                                ("smollm2-135m-instruct", desc1.as_str()),
                                ("smollm2-360m-instruct", desc2.as_str()),
                                ("smollm2-1-7b-instruct", desc3.as_str()),
                            ];
                            match select_menu_interactive(
                                "R⁴ Interactive Model Selector & Engine Manager:",
                                &model_options,
                                output,
                            )? {
                                Some(idx) => model_options[idx].0.to_string(),
                                None => {
                                    output.flush()?;
                                    continue;
                                }
                            }
                        }
                    };

                    handle_model_switch_with_remediation(
                        &target_model,
                        &host,
                        port,
                        &mut current_active_model,
                        &mut current_active_engine,
                        output,
                    )?;
                    output.flush()?;
                    continue;
                }
                "/engine" => {
                    let target_engine_opt = if parts.len() > 1 {
                        match parts[1] {
                            "1" | "r4g1" => Some("r4g1"),
                            "2" | "attention" => Some("attention"),
                            "3" | "r4-attention" => Some("r4-attention"),
                            "4" | "geometric" => Some("geometric"),
                            "5" | "legacy" | "transformerless-legacy" => {
                                Some("transformerless-legacy")
                            }
                            other => Some(other),
                        }
                    } else {
                        None
                    };

                    let target_engine = match target_engine_opt {
                        Some(eng) => eng.to_string(),
                        None => {
                            let engine_options = [
                                ("r4g1", "Sub-ms Zero-Multiply Residual Graph Engine"),
                                ("attention", "Full Attention Teacher Oracle Fallback"),
                                (
                                    "r4-attention",
                                    "Llama Certified Leading-4 Attention (GPT-2 Learned-Absolute)",
                                ),
                                ("geometric", "f64 Geometric Router Engine"),
                                ("transformerless-legacy", "Legacy Table Store Kernel"),
                            ];
                            match select_menu_interactive(
                                "R⁴ Interactive Synthesis Engine Manager:",
                                &engine_options,
                                output,
                            )? {
                                Some(idx) => engine_options[idx].0.to_string(),
                                None => {
                                    output.flush()?;
                                    continue;
                                }
                            }
                        }
                    };

                    current_active_engine = target_engine.clone();
                    if let Err(error) = persist_state_file(
                        &crate::model::store_state_file("last_engine.txt"),
                        &current_active_engine,
                    ) {
                        writeln!(
                            output,
                            "\x1b[33m[-] engine preference not persisted: {error}\x1b[0m"
                        )?;
                    }
                    writeln!(
                        output,
                        "\x1b[32m[+] Active synthesis engine set to '{}'\x1b[0m\n",
                        current_active_engine
                    )?;
                    output.flush()?;
                    continue;
                }
                "/export" => {
                    let req_body = serde_json::json!({ "action": "export" });
                    match send_server_post_request(&host, port, "/v1/corpus", &req_body) {
                        Ok(res) => {
                            let msg = res["message"]
                                .as_str()
                                .unwrap_or("Exported manifold state to .uor-models/exported/exported_manifold.json");
                            writeln!(output, "\x1b[32m[✓] {}\x1b[0m\n", msg)?;
                        }
                        Err(e) => {
                            writeln!(output, "[!] Export error: {}\n", e)?;
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/corpus" => {
                    if parts.len() > 2 && parts[1] == "add" {
                        let file_path = parts[2];
                        match std::fs::read_to_string(file_path) {
                            Ok(content) => {
                                let filename = std::path::Path::new(file_path)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "custom_corpus.txt".to_string());

                                let req_body = serde_json::json!({
                                    "action": "add",
                                    "filename": filename,
                                    "content": content
                                });

                                match send_server_post_request(&host, port, "/v1/corpus", &req_body)
                                {
                                    Ok(res) => {
                                        writeln!(
                                            output,
                                            "\x1b[32m[+] {}\x1b[0m\n",
                                            res["message"].as_str().unwrap_or("Corpus added")
                                        )?;
                                    }
                                    Err(e) => {
                                        writeln!(output, "[!] Error updating corpus: {}\n", e)?;
                                    }
                                }
                            }
                            Err(e) => {
                                writeln!(
                                    output,
                                    "[!] Failed to read file '{}': {}\n",
                                    file_path, e
                                )?;
                            }
                        }
                    } else {
                        let corpus_options = [
                            (
                                "1. List Indexed Files",
                                "View reading corpus datasets indexed on server",
                            ),
                            (
                                "2. Import Local File",
                                "Browse and select local text file to index into manifold",
                            ),
                            (
                                "3. Paste Plain Text",
                                "Paste raw text content to index into geometric manifold hashes",
                            ),
                            (
                                "4. Export Manifold",
                                "Export manifold state to .uor-models/exported/exported_manifold.json",
                            ),
                        ];
                        if let Ok(Some(opt_idx)) = select_menu_interactive(
                            "R⁴ Corpus & Geometric Manifold Management:",
                            &corpus_options,
                            output,
                        ) {
                            match opt_idx {
                                0 => {
                                    let req_body = serde_json::json!({ "action": "list" });
                                    match send_server_post_request(
                                        &host,
                                        port,
                                        "/v1/corpus",
                                        &req_body,
                                    ) {
                                        Ok(res) => {
                                            writeln!(
                                                output,
                                                "\n\x1b[1mR⁴ Extra Reading Corpus Datasets:\x1b[0m"
                                            )?;
                                            if let Some(files) = res["files"].as_array() {
                                                if files.is_empty() {
                                                    writeln!(
                                                        output,
                                                        "  (No extra reading corpus files indexed yet)"
                                                    )?;
                                                } else {
                                                    for f in files {
                                                        writeln!(
                                                            output,
                                                            "  • {}",
                                                            f.as_str().unwrap_or("")
                                                        )?;
                                                    }
                                                }
                                            }
                                            writeln!(output)?;
                                        }
                                        Err(e) => {
                                            writeln!(output, "[!] Error listing corpus: {}\n", e)?;
                                        }
                                    }
                                }
                                1 => {
                                    let mut candidates = Vec::new();
                                    if let Ok(entries) = std::fs::read_dir(".uor-models/sources") {
                                        for entry in entries.filter_map(|e| e.ok()) {
                                            let p = entry.path();
                                            if p.is_file() {
                                                candidates.push(p.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                    if let Ok(entries) = std::fs::read_dir(".") {
                                        for entry in entries.filter_map(|e| e.ok()) {
                                            let p = entry.path();
                                            if p.is_file()
                                                && (p.extension()
                                                    == Some(std::ffi::OsStr::new("txt"))
                                                    || p.extension()
                                                        == Some(std::ffi::OsStr::new("md")))
                                            {
                                                candidates.push(p.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                    candidates.sort();
                                    candidates.dedup();

                                    let mut menu_items: Vec<(&str, &str)> = candidates
                                        .iter()
                                        .map(|path| (path.as_str(), "Local corpus document"))
                                        .collect();
                                    menu_items.push((
                                        "Custom File Path...",
                                        "Enter arbitrary file path manually",
                                    ));

                                    if let Ok(Some(file_idx)) = select_menu_interactive(
                                        "Select File to Import into Geometric Manifold:",
                                        &menu_items,
                                        output,
                                    ) {
                                        let target_path = if file_idx < candidates.len() {
                                            candidates[file_idx].clone()
                                        } else {
                                            writeln!(output, "Enter file path to import: ")?;
                                            output.flush()?;
                                            let mut path_buf = String::new();
                                            std::io::stdin().read_line(&mut path_buf).ok();
                                            path_buf.trim().to_string()
                                        };

                                        if !target_path.is_empty() {
                                            match std::fs::read_to_string(&target_path) {
                                                Ok(content) => {
                                                    let filename =
                                                        std::path::Path::new(&target_path)
                                                            .file_name()
                                                            .map(|f| {
                                                                f.to_string_lossy().to_string()
                                                            })
                                                            .unwrap_or_else(|| {
                                                                "imported_corpus.txt".to_string()
                                                            });

                                                    let req_body = serde_json::json!({
                                                        "action": "add",
                                                        "filename": filename,
                                                        "content": content
                                                    });

                                                    match send_server_post_request(
                                                        &host,
                                                        port,
                                                        "/v1/corpus",
                                                        &req_body,
                                                    ) {
                                                        Ok(res) => {
                                                            writeln!(
                                                                output,
                                                                "\x1b[32m[✓] {}\x1b[0m\n",
                                                                res["message"]
                                                                    .as_str()
                                                                    .unwrap_or("Corpus imported")
                                                            )?;
                                                        }
                                                        Err(e) => {
                                                            writeln!(
                                                                output,
                                                                "[!] Error importing corpus: {}\n",
                                                                e
                                                            )?;
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    writeln!(
                                                        output,
                                                        "[!] Failed to read '{}': {}\n",
                                                        target_path, e
                                                    )?;
                                                }
                                            }
                                        }
                                    }
                                }
                                2 => {
                                    write!(
                                        output,
                                        "\x1b[1mEnter or paste text content to index into R⁴ geometric manifold:\x1b[0m\n\x1b[1;36mcorpus-text > \x1b[0m"
                                    )?;
                                    output.flush()?;

                                    let mut input_buf = String::new();
                                    std::io::stdin().read_line(&mut input_buf).ok();
                                    let content = input_buf.trim().to_string();

                                    if !content.is_empty() {
                                        let ts = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();
                                        let filename = format!("pasted_corpus_{}.txt", ts);

                                        let req_body = serde_json::json!({
                                            "action": "add",
                                            "filename": filename,
                                            "content": content
                                        });

                                        match send_server_post_request(
                                            &host,
                                            port,
                                            "/v1/corpus",
                                            &req_body,
                                        ) {
                                            Ok(res) => {
                                                writeln!(
                                                    output,
                                                    "\x1b[32m[✓] {}\x1b[0m\n",
                                                    res["message"]
                                                        .as_str()
                                                        .unwrap_or("Pasted corpus indexed")
                                                )?;
                                            }
                                            Err(e) => {
                                                writeln!(
                                                    output,
                                                    "[!] Error indexing pasted corpus: {}\n",
                                                    e
                                                )?;
                                            }
                                        }
                                    } else {
                                        writeln!(
                                            output,
                                            "[!] Empty text content submitted. Nothing indexed.\n"
                                        )?;
                                    }
                                }
                                3 => {
                                    let req_body = serde_json::json!({ "action": "export" });
                                    match send_server_post_request(
                                        &host,
                                        port,
                                        "/v1/corpus",
                                        &req_body,
                                    ) {
                                        Ok(res) => {
                                            let msg = res["message"]
                                                .as_str()
                                                .unwrap_or("Exported manifold state to .uor-models/exported/exported_manifold.json");
                                            writeln!(output, "\x1b[32m[✓] {}\x1b[0m\n", msg)?;
                                        }
                                        Err(e) => {
                                            writeln!(output, "[!] Export error: {}\n", e)?;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/clear" => {
                    history.clear();
                    audit_history.clear();
                    write!(output, "\x1b[2J\x1b[1H")?;
                    output.flush()?;
                    continue;
                }
                "/reset" => {
                    history.clear();
                    audit_history.clear();
                    let _ = send_vendor_reset(&host, port);
                    write!(output, "\x1b[2J\x1b[1H")?;
                    writeln!(
                        output,
                        "\x1b[32m[✓] Chat history, extra corpus index & geometric manifold state reset back to base defaults.\x1b[0m\n"
                    )?;
                    output.flush()?;
                    continue;
                }
                "/quit" | "/exit" => {
                    break;
                }
                "/compile" => {
                    let model_options = [
                        ("smollm2-135m-instruct", "Fast & Ultra-Light (~270MB)"),
                        ("smollm2-360m-instruct", "Balanced Quality (~720MB)"),
                        ("smollm2-1-7b-instruct", "High-Fidelity Teacher (~3.4GB)"),
                    ];
                    if let Ok(Some(m_idx)) = select_menu_interactive(
                        "Select Model to Compile into R4G1 Zero-Multiply Graph:",
                        &model_options,
                        output,
                    ) {
                        let target_model = model_options[m_idx].0;
                        if trigger_in_client_compilation(target_model, &host, port, output)
                            .unwrap_or(false)
                        {
                            current_active_model = target_model.to_string();
                            current_active_engine = "r4g1".to_string();
                            if let Err(error) = persist_state_file(
                                &crate::model::store_state_file("last_model_name.txt"),
                                &current_active_model,
                            )
                            .and_then(|()| {
                                persist_state_file(
                                    &crate::model::store_state_file("last_engine.txt"),
                                    &current_active_engine,
                                )
                            }) {
                                writeln!(
                                    output,
                                    "\x1b[33m[-] preference not persisted: {error}\x1b[0m"
                                )?;
                            }
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/status" => {
                    writeln!(
                        output,
                        "[*] Querying R4G1 sub-millisecond pipeline status..."
                    )?;
                    match fetch_server_status(&host, port) {
                        Ok(st) => {
                            let model_name =
                                st["model_name"].as_str().unwrap_or("smollm2-135m-instruct");
                            let s1 = st["stages"]["stage_1_download"].as_bool().unwrap_or(false);
                            let s2 = st["stages"]["stage_2_compile"].as_bool().unwrap_or(false);
                            let s3 = st["stages"]["stage_3_graph_score"]
                                .as_bool()
                                .unwrap_or(false);
                            let s4 = st["stages"]["stage_4_r4g1_active"]
                                .as_bool()
                                .unwrap_or(false);

                            let mark = |b| if b { "[✓]" } else { "[ ]" };

                            writeln!(
                                output,
                                "\n\x1b[1mR⁴ Sub-Millisecond R4G1 Compilation Pipeline Status ({})\x1b[0m",
                                model_name
                            )?;
                            writeln!(
                                output,
                                "┌───────┬───────────────────────────────────┬────────┬──────────────────────────────────────────────┐"
                            )?;
                            writeln!(
                                output,
                                "│ Stage │ Description                       │ Status │ Target Artifact / Location                   │"
                            )?;
                            writeln!(
                                output,
                                "├───────┼───────────────────────────────────┼────────┼──────────────────────────────────────────────┤"
                            )?;
                            writeln!(
                                output,
                                "│   1   │ Pinned Teacher Source Download    │  {:^5} │ .uor-models/sources/{:<25} │",
                                mark(s1),
                                model_name
                            )?;
                            writeln!(
                                output,
                                "│   2   │ Transformerless Bundle Compile    │  {:^5} │ .uor-models/compiled/{:<24} │",
                                mark(s2),
                                model_name
                            )?;
                            writeln!(
                                output,
                                "│   3   │ Scored R4G1 Graph Cover & Score   │  {:^5} │ .../{:<35} │",
                                mark(s3),
                                format!("{}/graph/score.r4g1", model_name)
                            )?;
                            writeln!(
                                output,
                                "│   4   │ Sub-ms Zero-Multiply Engine       │  {:^5} │ Active (R4G1 Scored Graph Runtime)           │",
                                mark(s4)
                            )?;
                            writeln!(
                                output,
                                "└───────┴───────────────────────────────────┴────────┴──────────────────────────────────────────────┘"
                            )?;
                            writeln!(
                                output,
                                "Target Performance Goal: < 1.0 ms / token (Zero-Multiply Table-Native Kernel)\n"
                            )?;
                        }
                        Err(e) => {
                            writeln!(output, "[!] Error fetching pipeline status: {}\n", e)?;
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/audit" => {
                    if audit_history.is_empty() {
                        writeln!(
                            output,
                            "\n\x1b[33m[!] No Q&A turns audited in this session yet.\x1b[0m"
                        )?;
                        writeln!(
                            output,
                            "    Ask a question first, then run '/audit' to inspect UOR coordinates.\n"
                        )?;
                    } else {
                        let audit_options = [
                            (
                                "1) View Last Q&A Audit Trace",
                                "Inspect UOR coordinates, kappa pass status, and token provenance for last turn",
                            ),
                            (
                                "2) List Audit History",
                                "Browse and select from recent session Q&A turns",
                            ),
                            (
                                "3) Export Audit Log",
                                "Export full session audit trace to .uor-models/audit_log.json",
                            ),
                        ];

                        if let Ok(Some(a_idx)) = select_menu_interactive(
                            "R⁴ UOR Auditability & Tracing Inspector:",
                            &audit_options,
                            output,
                        ) {
                            match a_idx {
                                0 => {
                                    let last_rec = audit_history.last().unwrap();
                                    render_audit_trace_record(last_rec, output)?;
                                }
                                1 => {
                                    let history_options: Vec<(String, String)> = audit_history
                                        .iter()
                                        .enumerate()
                                        .map(|(i, (q, _, audit))| {
                                            let kappa_str = audit
                                                .as_ref()
                                                .map(|a| format!("κ={:.4}", a.kappa))
                                                .unwrap_or_else(|| "N/A".to_string());
                                            let short_q = if q.len() > 35 {
                                                format!("{}...", &q[..35])
                                            } else {
                                                q.clone()
                                            };
                                            (
                                                format!("Turn #{}", i + 1),
                                                format!("{} [{}]", short_q, kappa_str),
                                            )
                                        })
                                        .collect();

                                    let view_refs: Vec<(&str, &str)> = history_options
                                        .iter()
                                        .map(|(label, desc)| (label.as_str(), desc.as_str()))
                                        .collect();

                                    if let Ok(Some(h_idx)) = select_menu_interactive(
                                        "Select Q&A Turn to Audit:",
                                        &view_refs,
                                        output,
                                    ) {
                                        render_audit_trace_record(&audit_history[h_idx], output)?;
                                    }
                                }
                                2 => {
                                    let export_path = ".uor-models/audit_log.json";
                                    if let Ok(json_str) =
                                        serde_json::to_string_pretty(&audit_history)
                                    {
                                        if std::fs::write(export_path, json_str).is_ok() {
                                            writeln!(
                                                output,
                                                "\x1b[32m[+] Successfully exported session audit trace ({} turns) to {}\x1b[0m\n",
                                                audit_history.len(),
                                                export_path
                                            )?;
                                        } else {
                                            writeln!(
                                                output,
                                                "[!] Failed to write audit log file.\n"
                                            )?;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    output.flush()?;
                    continue;
                }
                _ => {}
            }
        }

        let start_time = std::time::Instant::now();
        let (host_c, port_c, path_c, model_c, engine_c, q_c) = (
            host.clone(),
            port,
            path.clone(),
            current_active_model.clone(),
            current_active_engine.clone(),
            question.to_string(),
        );

        let worker_handle = std::thread::spawn(move || {
            send_vendor_chat_completion(&host_c, port_c, &path_c, &model_c, &engine_c, &q_c)
        });

        writeln!(output)?;
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut frame_idx = 0;

        while !worker_handle.is_finished() {
            let elapsed_secs = start_time.elapsed().as_secs();
            let frame = frames[frame_idx % frames.len()];
            write!(
                output,
                "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m {} lifting... ({}s)\x1b[K",
                frame, elapsed_secs
            )?;
            output.flush()?;
            frame_idx += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let res = worker_handle
            .join()
            .unwrap_or_else(|_| Err("Worker thread panicked".to_string()));

        match res {
            Ok((answer_text, completion_tokens, engine_mode, uor_audit)) => {
                audit_history.push((question.to_string(), answer_text.clone(), uor_audit));
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let latency_ms = elapsed_secs * 1000.0;
                let tok_per_sec = if elapsed_secs > 0.0001 {
                    (completion_tokens as f64) / elapsed_secs
                } else {
                    0.0
                };
                write!(
                    output,
                    "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m {}\x1b[K\n",
                    answer_text
                )?;
                writeln!(
                    output,
                    "\x1b[90m[stats: {} tokens | {:.2} ms | {:.1} tok/s | mode: {} | model: {}]\x1b[0m\n",
                    completion_tokens, latency_ms, tok_per_sec, engine_mode, current_active_model
                )?;
                output.flush()?;
            }
            Err(err) => {
                write!(
                    output,
                    "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m [!] Error communicating with local server: {}\x1b[K\n\n",
                    err
                )?;
                output.flush()?;
            }
        }
    }
    Ok(())
}

fn parse_remote_url(raw_url: &str) -> (String, u16, String) {
    let clean = raw_url
        .trim()
        .strip_prefix("http://")
        .or_else(|| raw_url.trim().strip_prefix("https://"))
        .unwrap_or(raw_url.trim());

    let (host_port, path_part) = match clean.find('/') {
        Some(idx) => (&clean[..idx], &clean[idx..]),
        None => (clean, ""),
    };

    let mut parts = host_port.split(':');
    let host = parts.next().unwrap_or("127.0.0.1").trim();
    let host_str = if host.is_empty() { "127.0.0.1" } else { host };
    let port: u16 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(8000);

    let path = if path_part.contains("/chat/completions") {
        path_part.to_string()
    } else if path_part.ends_with("/v1") || path_part.ends_with("/v1/") {
        let base = path_part.trim_end_matches('/');
        format!("{}/chat/completions", base)
    } else if path_part.is_empty() || path_part == "/" {
        "/v1/chat/completions".to_string()
    } else {
        format!("{}/chat/completions", path_part.trim_end_matches('/'))
    };

    (host_str.to_string(), port, path)
}

fn send_vendor_chat_completion(
    host: &str,
    port: u16,
    path: &str,
    model: &str,
    engine: &str,
    user_message: &str,
) -> Result<(String, usize, String, Option<crate::server::UorAuditTrace>), String> {
    let payload = serde_json::json!({
        "model": model,
        "engine": engine,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ],
        "max_tokens": 384,
        "temperature": 0.7
    });
    let body_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("Serialization error: {}", e))?;

    let req_str = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        path,
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(300)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request headers: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send request body: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    let parsed: serde_json::Value = serde_json::from_str(json_body)
        .map_err(|e| format!("Invalid response JSON: {} (body: {:?})", e, json_body))?;

    let choice = parsed["choices"]
        .get(0)
        .ok_or_else(|| "Missing choices in response".to_string())?;
    let content = choice["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let completion_tokens = parsed["usage"]["completion_tokens"]
        .as_u64()
        .unwrap_or_else(|| content.split_whitespace().count() as u64)
        as usize;
    // #655-F: fingerprints are `r4-{mode}` post-flip; the pre-flip
    // `uor-r4-{mode}` prefix stays parseable for the deprecation window
    // (an old server behind a new client).
    let fingerprint = parsed["system_fingerprint"].as_str().unwrap_or("r4");
    let mode = fingerprint
        .strip_prefix("r4-")
        .or_else(|| fingerprint.strip_prefix("uor-r4-"))
        .unwrap_or(fingerprint)
        .to_string();

    let uor_audit: Option<crate::server::UorAuditTrace> = parsed
        .get("uor_audit")
        .and_then(|val| serde_json::from_value(val.clone()).ok());

    Ok((content, completion_tokens, mode, uor_audit))
}

fn fetch_server_status(host: &str, port: u16) -> Result<serde_json::Value, String> {
    let req_str = format!(
        "GET /v1/status HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Connection: close\r\n\r\n",
        host, port
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send status request: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read status response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    serde_json::from_str(json_body).map_err(|e| format!("Invalid status response JSON: {}", e))
}

pub fn send_vendor_reset(host: &str, port: u16) -> Result<(), String> {
    let payload = serde_json::json!({});
    let body_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("Serialization error: {}", e))?;
    let req_str = format!(
        "POST /api/reset HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send body: {}", e))?;
    stream.flush().ok();

    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).ok();
    Ok(())
}

fn send_server_post_request(
    host: &str,
    port: u16,
    path: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let body_bytes =
        serde_json::to_vec(payload).map_err(|e| format!("Serialization error: {}", e))?;
    let req_str = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        path,
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request headers: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send request body: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    serde_json::from_str(json_body)
        .map_err(|e| format!("Invalid response JSON: {} (body: {:?})", e, json_body))
}

fn render_audit_trace_record(
    record: &(String, String, Option<crate::server::UorAuditTrace>),
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let (question, answer, audit_opt) = record;
    writeln!(
        output,
        "\n\x1b[1;36m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m"
    )?;
    writeln!(
        output,
        "\x1b[1;36m│  R⁴ UOR Auditability & Tracing Inspector                                    │\x1b[0m"
    )?;
    writeln!(
        output,
        "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
    )?;
    let short_q = if question.len() > 65 {
        format!("{}...", &question[..65])
    } else {
        question.clone()
    };
    let short_a = if answer.len() > 65 {
        format!("{}...", &answer[..65])
    } else {
        answer.clone()
    };
    writeln!(output, "  \x1b[1mQuery Prompt \x1b[0m : {}", short_q)?;
    writeln!(output, "  \x1b[1mGenerated Ans\x1b[0m : {}", short_a)?;
    writeln!(
        output,
        "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
    )?;

    if let Some(audit) = audit_opt {
        let pass_badge = if audit.kappa_pass {
            "\x1b[32m[✓ PASS]\x1b[0m"
        } else {
            "\x1b[33m[! DRIFT]\x1b[0m"
        };
        writeln!(
            output,
            "  \x1b[1mUOR Address     \x1b[0m : \x1b[36m{}\x1b[0m",
            audit.uor_address
        )?;
        writeln!(
            output,
            "  \x1b[1mCurvature κ     \x1b[0m : {} {}",
            audit.kappa, pass_badge
        )?;
        writeln!(
            output,
            "  \x1b[1mDeficit Angle θd\x1b[0m : {} rad",
            audit.deficit_angle
        )?;
        writeln!(
            output,
            "  \x1b[1mQIMC Bias uor_b \x1b[0m : {}",
            audit.entropy_bias
        )?;
        writeln!(output, "  \x1b[1mDampening γ     \x1b[0m : {}", audit.gamma)?;
        writeln!(
            output,
            "  \x1b[1mTemperature T   \x1b[0m : {}",
            audit.temperature
        )?;
        writeln!(
            output,
            "  \x1b[1mEngine Mode     \x1b[0m : \x1b[32m{}\x1b[0m",
            audit.generation_mode
        )?;
        writeln!(
            output,
            "  \x1b[1mTotal Latency   \x1b[0m : {:.2} ms",
            audit.total_latency_ms
        )?;
        writeln!(
            output,
            "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
        )?;
        writeln!(
            output,
            "  \x1b[1mToken Provenance Trace ({} tokens):\x1b[0m",
            audit.tokens_detail.len()
        )?;
        for t in audit.tokens_detail.iter().take(20) {
            writeln!(
                output,
                "   [{:>2}] '{:<15}' -> {:<38} ({:.2} ms)",
                t.token_id, t.text, t.origin_rule, t.latency_ms
            )?;
        }
        if audit.tokens_detail.len() > 20 {
            writeln!(
                output,
                "   ... ({} remaining tokens omitted for display)",
                audit.tokens_detail.len() - 20
            )?;
        }
    } else {
        writeln!(output, "  (No UOR audit trace payload returned by backend)")?;
    }
    writeln!(
        output,
        "\x1b[1;36m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n"
    )?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        bind_chat_r4g1, engine_from_bytes_with_r4g1, ensure_chat_prompt_encoder, parse_remote_url,
        recent_window_repetition_rate, repeated_suffix, select_chat_tokenizer_bytes, ChatError,
    };
    use uor_r4_core::transformerless::scenarios::{
        export_runtime_tokenizer_table, RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable,
        RuntimeTokenizerEncodePolicy, RuntimeTokenizerIdentity, Tokenizer,
    };
    use uor_r4_router::session_signature_from_tokens;

    fn minimal_graph(tokenizer_cid: [u8; 32]) -> Vec<u8> {
        use uor_r4_graph_format::{ArtifactBuilder, SectionId};

        let mut head = Vec::with_capacity(224);
        head.extend_from_slice(&[0x11; 32]);
        head.extend_from_slice(&tokenizer_cid);
        head.extend_from_slice(&[0x33; 32]);
        head.extend_from_slice(&[0x44; 32]);
        head.extend_from_slice(b"0123456789abcdef0123");
        head.extend_from_slice(&[0x55; 32]);
        head.extend_from_slice(&32u16.to_le_bytes());
        head.extend_from_slice(&16u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.push(1);
        head.extend_from_slice(&[0; 5]);
        head.extend_from_slice(&[0; 2]);
        head.extend_from_slice(&64u16.to_le_bytes());
        head.extend_from_slice(&1u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&100u32.to_le_bytes());
        assert_eq!(head.len(), 224);
        let mut builder = ArtifactBuilder::new(3);
        builder.add_section(SectionId::HEAD, 0, &head);
        builder.build().expect("minimal graph")
    }

    #[test]
    fn repetition_guard_detects_repeated_token_windows() {
        assert!(repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 4], 4));
        assert!(!repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 5], 4));
    }

    /// `repeated_suffix`'s exact-block detector is blind to the #745
    /// failure mode: cycling through a handful of distinct tokens in
    /// varying (non-block-repeating) order. `recent_window_repetition_rate`
    /// (#744) is built specifically to catch it.
    #[test]
    fn recent_window_repetition_rate_catches_small_vocabulary_cycling_that_repeated_suffix_misses()
    {
        let cycling = [1u32, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3];
        assert!(
            !repeated_suffix(&cycling, 8),
            "too short for the exact 8-token block detector to see anything"
        );
        assert!(
            recent_window_repetition_rate(&cycling, 8) > 0.5,
            "but a small-vocabulary cycle is mostly recently-seen tokens"
        );

        let varied: Vec<u32> = (0..40).collect();
        assert_eq!(
            recent_window_repetition_rate(&varied, 8),
            0.0,
            "monotonically novel tokens carry no repetition"
        );

        assert_eq!(
            recent_window_repetition_rate(&[], 8),
            0.0,
            "empty generation is defined as non-repetitive, not undefined"
        );
    }

    #[test]
    fn direct_chat_reports_decode_only_tokenizer_as_unavailable() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-chat-decode-only-{}.bin",
            std::process::id()
        ));
        let table = RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: "future-sentencepiece-family".to_owned(),
                version: 41,
                tokenizer_cid: format!("blake3:{}", "3".repeat(64)),
                adapter_digest: format!("blake3:{}", "4".repeat(64)),
            },
            pieces: vec![Vec::new(), b"piece".to_vec()],
            encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        export_runtime_tokenizer_table(&table, &path).expect("tagged export");
        let bytes = std::fs::read(&path).expect("read tagged");
        assert_eq!(
            select_chat_tokenizer_bytes(&bytes).expect("valid tagged bytes stay selected"),
            bytes
        );
        let mut malformed = bytes.clone();
        malformed.pop();
        let error = select_chat_tokenizer_bytes(&malformed)
            .expect_err("malformed tagged bytes cannot downgrade to a legacy tokenizer");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

        let tokenizer = Tokenizer::from_bytes(&bytes).expect("parse tagged");
        let error = ensure_chat_prompt_encoder(&tokenizer).expect_err("encoder is unavailable");
        assert!(matches!(
            error,
            ChatError::TokenizerUnavailable {
                ref family,
                version: 41,
            } if family == "future-sentencepiece-family"
        ));
        assert!(error.to_string().contains("exact host adapter"));
        let error = bind_chat_r4g1(Some(minimal_graph([0; 32])), &bytes)
            .expect_err("zero-CID graph cannot admit a tagged tokenizer");
        assert!(error.to_string().contains("nonzero"), "{error}");
        assert!(bind_chat_r4g1(
            Some(minimal_graph(*blake3::hash(&bytes).as_bytes())),
            &bytes,
        )
        .expect("nonzero exact tagged binding")
        .is_some());
        let _ = std::fs::remove_file(path);
    }

    /// #655-C1e: an R4G1-era bundle (graph + signature artifact +
    /// tokenizer, NO `tless_store.bin`) builds a working engine with the
    /// graph preferred and an EMPTY plain-fallback store — while removing
    /// the graph too restores the legacy required-file error (falsifier:
    /// a store-less, graph-less directory serves nothing and must not
    /// silently build).
    #[test]
    fn r4g1_era_bundle_without_plain_store_builds_and_prefers_the_graph() {
        let root =
            std::env::temp_dir().join(format!("uor-r4-c1e-modern-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("bundle");
        std::fs::create_dir_all(dir.join("graph")).unwrap();

        let artifact_bytes = std::fs::read("crates/uor-r4-core/tests/fixtures/tless_artifacts.bin")
            .expect("fixture artifacts");
        std::fs::write(dir.join("tless_artifacts.bin"), &artifact_bytes).unwrap();

        let mut tokenizer_bytes = Vec::new();
        for piece in [
            b"<unk>".as_slice(),
            b"<s>".as_slice(),
            b"</s>".as_slice(),
            b"a".as_slice(),
        ] {
            tokenizer_bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            tokenizer_bytes.extend_from_slice(piece);
        }
        std::fs::write(dir.join("tokenizer.bin"), &tokenizer_bytes).unwrap();

        let graph = minimal_graph(*blake3::hash(&tokenizer_bytes).as_bytes());
        std::fs::write(dir.join("graph").join("score.r4g1"), &graph).unwrap();

        let model_store = crate::model::ModelStore::new(root.join("store-root"));
        let strict_error = match super::build_local_compiled_engine(
            &model_store,
            &dir,
            "c1e-modern",
            8,
            None,
            false,
        ) {
            Err(error) => error,
            Ok(_) => panic!("a pre-schema-2 bundle must not load without explicit research mode"),
        };
        assert!(matches!(strict_error, ChatError::ProductionAdmission(_)));
        let engine =
            super::build_local_compiled_engine(&model_store, &dir, "c1e-modern", 8, None, true)
                .expect(
                    "an explicitly research-only R4G1-era bundle builds without tless_store.bin",
                );
        assert_eq!(
            engine.r4g1_bytes.as_deref(),
            Some(graph.as_slice()),
            "the graph is preferred and bound"
        );
        assert!(
            engine.store.iter().all(|stage| stage.is_empty()),
            "the plain fallback store is empty, never fabricated"
        );

        // A presented production envelope is authoritative. It cannot fall
        // back to the permissive legacy/research loader when incomplete.
        std::fs::write(dir.join("release-bundle.json"), b"{}\n").unwrap();
        let error = match super::build_local_compiled_engine(
            &model_store,
            &dir,
            "c1e-modern",
            8,
            None,
            true,
        ) {
            Err(error) => error,
            Ok(_) => panic!("an incomplete schema-2 envelope must fail closed"),
        };
        assert!(matches!(error, ChatError::ProductionAdmission(_)));
        std::fs::remove_file(dir.join("release-bundle.json")).unwrap();

        // Falsifier: no store AND no graph — the directory serves nothing
        // and the legacy required-file error returns.
        std::fs::remove_file(dir.join("graph").join("score.r4g1")).unwrap();
        assert!(
            super::build_local_compiled_engine(&model_store, &dir, "c1e-modern", 8, None, true,)
                .is_err(),
            "a store-less, graph-less directory must not build"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn present_untagged_chat_tokenizer_keeps_its_exact_bytes() {
        let mut bytes = Vec::new();
        for piece in [b"<unk>".as_slice(), b"a".as_slice(), b"b".as_slice()] {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        assert!(!Tokenizer::is_tagged_container_bytes(&bytes));
        assert_eq!(
            select_chat_tokenizer_bytes(&bytes).expect("valid historical tokenizer"),
            bytes
        );
    }

    #[test]
    fn direct_chat_rejects_a_graph_tokenizer_swap_and_preserves_zero_cid_legacy() {
        let tokenizer_a = b"\x01\0\0\0a".to_vec();
        let tokenizer_b = b"\x01\0\0\0b".to_vec();
        let graph = minimal_graph(*blake3::hash(&tokenizer_a).as_bytes());
        let error = bind_chat_r4g1(Some(graph.clone()), &tokenizer_b)
            .expect_err("graph A plus tokenizer B must be rejected");
        assert!(
            error.to_string().contains("tokenizer CID mismatch"),
            "{error}"
        );
        assert_eq!(
            bind_chat_r4g1(Some(graph.clone()), &tokenizer_a)
                .expect("exact graph/tokenizer pair")
                .as_deref(),
            Some(graph.as_slice())
        );

        let legacy = minimal_graph([0; 32]);
        assert_eq!(
            bind_chat_r4g1(Some(legacy.clone()), &tokenizer_b)
                .expect("zero-CID graph retains legacy compatibility")
                .as_deref(),
            Some(legacy.as_slice())
        );
    }

    /// #750 falsifier: `engine_from_bytes_with_r4g1` must actually thread
    /// the supplied graph bytes into the resulting `ChatEngine` (bound and
    /// validated the same way `ChatEngineBuilder::build()` does), not
    /// silently drop them the way `engine_from_bytes` always has. A
    /// plumbing bug here would defeat #750's entire point: the quality
    /// gate could believe it probed the R4G1 path while actually still
    /// only exercising the plain path.
    #[test]
    fn engine_from_bytes_with_r4g1_actually_threads_the_graph_bytes_through() {
        use uor_r4_core::transformerless::{compiler, runtime};

        let art_bytes = std::fs::read("crates/uor-r4-core/tests/fixtures/tless_artifacts.bin")
            .expect("fixture artifacts present");
        let mut store: runtime::Store =
            (0..=compiler::STAGES).map(|_| Default::default()).collect();
        runtime::add_evidence(&mut store, &[3, 1, 4, 1], 7, 5);
        let store_bytes = runtime::store_bytes(&store);

        let mut tokenizer_bytes = Vec::new();
        for piece in [b"<unk>".as_slice(), b"a".as_slice(), b"b".as_slice()] {
            tokenizer_bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            tokenizer_bytes.extend_from_slice(piece);
        }

        // No graph supplied: behaves exactly like the original
        // `engine_from_bytes` (r4g1_bytes stays None).
        let plain_only =
            engine_from_bytes_with_r4g1(&art_bytes, &store_bytes, &tokenizer_bytes, None, 8)
                .expect("plain engine builds");
        assert!(plain_only.r4g1_bytes.is_none());

        // A graph is supplied: it must survive validation/binding and end
        // up populated on the engine, so `ask()` actually takes the R4G1
        // branch of `hologram_answer` rather than silently falling back.
        let graph = minimal_graph([0; 32]);
        let with_graph = engine_from_bytes_with_r4g1(
            &art_bytes,
            &store_bytes,
            &tokenizer_bytes,
            Some(&graph),
            8,
        )
        .expect("engine with graph builds");
        assert_eq!(with_graph.r4g1_bytes.as_deref(), Some(graph.as_slice()));

        // A structurally invalid graph is rejected up front rather than
        // silently ignored (mirrors `ChatEngineBuilder::build()`'s own
        // `validate_chat_r4g1_structure` gate).
        let result = engine_from_bytes_with_r4g1(
            &art_bytes,
            &store_bytes,
            &tokenizer_bytes,
            Some(&[0u8; 4]),
            8,
        );
        match result {
            Err(error) => assert!(matches!(error, ChatError::Io(_))),
            Ok(_) => panic!("truncated graph bytes must be rejected"),
        }
    }

    /// #785 C3: the decode witness names the engine that actually served
    /// and carries the tier evidence each path really produced — the
    /// R4G1 beam classifies every candidate query by the #785-C1 buffer
    /// contract, and the plain path finally carries out the per-token
    /// resolution depths it always computed.
    #[test]
    fn decode_witness_names_the_serving_engine() {
        use uor_r4_core::transformerless::{compiler, convert_r4g1, runtime};

        let art_bytes = std::fs::read("crates/uor-r4-core/tests/fixtures/tless_artifacts.bin")
            .expect("fixture artifacts present");
        let artifacts = compiler::parse_artifacts(&art_bytes).expect("fixture artifacts parse");
        let mut store: runtime::Store =
            (0..=compiler::STAGES).map(|_| Default::default()).collect();
        // Mirror the graph-runtime tests' synthetic store: six codes whose
        // evidence tokens (1..=6) give the converted graph a real root
        // prior to emit from.
        let codes: [[u8; 4]; 6] = [
            [3, 1, 4, 1],
            [3, 1, 4, 2],
            [3, 5, 9, 2],
            [7, 5, 9, 2],
            [7, 5, 8, 2],
            [11, 5, 8, 7],
        ];
        for (i, code) in codes.iter().enumerate() {
            runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
        }
        // One dominant non-reserved token so both decode paths have an
        // unambiguous, non-terminating winner to emit.
        runtime::add_evidence(&mut store, &[3, 1, 4, 2], 3, 9);
        let store_bytes = runtime::store_bytes(&store);
        let mut tokenizer_bytes = Vec::new();
        // The non-BPE encoder prepends a space, so the vocab needs a " "
        // piece; ids stay small and in-range for the store's tokens.
        for piece in [
            b"<unk>".as_slice(),
            b" ".as_slice(),
            b"a".as_slice(),
            b"b".as_slice(),
            b"c".as_slice(),
            b"d".as_slice(),
            b"e".as_slice(),
            b"f".as_slice(),
            b"g".as_slice(),
        ] {
            tokenizer_bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            tokenizer_bytes.extend_from_slice(piece);
        }

        let (graph, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
            .expect("convert fixture graph");
        let mut engine = engine_from_bytes_with_r4g1(
            &art_bytes,
            &store_bytes,
            &tokenizer_bytes,
            Some(&graph),
            8,
        )
        .expect("engine with runnable graph");
        let answer = engine.ask("g").expect("graph-backed ask serves");
        assert_eq!(answer.witness.engine, "r4g1-beam");
        assert!(
            answer.witness.non_node_queries + answer.witness.node_path_queries > 0,
            "every candidate query is classified into exactly one bucket"
        );
        // This converter fixture resolves through the global last-resort
        // table (its leaf collection yields empty per-node lists), so the
        // queries land in the non-node bucket.
        assert!(answer.witness.non_node_queries > 0);
        assert!(answer.witness.plain_depths.is_empty());

        let mut plain =
            engine_from_bytes_with_r4g1(&art_bytes, &store_bytes, &tokenizer_bytes, None, 8)
                .expect("plain engine builds");
        let answer = plain.ask("g").expect("plain ask serves");
        assert_eq!(answer.witness.engine, "tla-plain-greedy");
        assert_eq!(answer.witness.plain_depths.len(), answer.generated_tokens);
        assert_eq!(answer.witness.non_node_queries, 0);
        assert_eq!(answer.witness.node_path_queries, 0);
    }

    /// #785-C2: the sampled R4G1 decode is seed-reproducible, names
    /// itself in the witness, and leaves the no-seed greedy beam
    /// untouched. Two engines with the same seed must produce identical
    /// answers; the unseeded engine keeps the `r4g1-beam` tag.
    #[test]
    fn sampled_r4g1_decode_is_seeded_and_witnessed() {
        use uor_r4_core::transformerless::{compiler, convert_r4g1, runtime};

        let art_bytes = std::fs::read("crates/uor-r4-core/tests/fixtures/tless_artifacts.bin")
            .expect("fixture artifacts present");
        let artifacts = compiler::parse_artifacts(&art_bytes).expect("fixture artifacts parse");
        let mut store: runtime::Store =
            (0..=compiler::STAGES).map(|_| Default::default()).collect();
        let codes: [[u8; 4]; 6] = [
            [3, 1, 4, 1],
            [3, 1, 4, 2],
            [3, 5, 9, 2],
            [7, 5, 9, 2],
            [7, 5, 8, 2],
            [11, 5, 8, 7],
        ];
        for (i, code) in codes.iter().enumerate() {
            runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
        }
        runtime::add_evidence(&mut store, &[3, 1, 4, 2], 3, 9);
        let store_bytes = runtime::store_bytes(&store);
        let mut tokenizer_bytes = Vec::new();
        for piece in [
            b"<unk>".as_slice(),
            b" ".as_slice(),
            b"a".as_slice(),
            b"b".as_slice(),
            b"c".as_slice(),
            b"d".as_slice(),
            b"e".as_slice(),
            b"f".as_slice(),
            b"g".as_slice(),
        ] {
            tokenizer_bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            tokenizer_bytes.extend_from_slice(piece);
        }
        let (graph, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
            .expect("convert fixture graph");

        let ask_sampled = |seed: u32| {
            let mut engine = engine_from_bytes_with_r4g1(
                &art_bytes,
                &store_bytes,
                &tokenizer_bytes,
                Some(&graph),
                8,
            )
            .expect("sampled engine builds");
            // In-module test: seed the same field ChatEngineBuilder::
            // sample_seed sets.
            engine.sample_rng = Some(super::SampleRng::new(seed));
            engine.ask("g").expect("sampled ask serves")
        };
        let first = ask_sampled(41);
        let second = ask_sampled(41);
        assert_eq!(first.witness.engine, "r4g1-sampled");
        assert_eq!(
            first.text, second.text,
            "one seed must reproduce one answer"
        );
        assert_eq!(first.witness, second.witness);
        assert!(
            first.witness.non_node_queries + first.witness.node_path_queries > 0,
            "sampled queries are classified like beam queries"
        );

        // No seed: the greedy beam is untouched and says so.
        let mut greedy = engine_from_bytes_with_r4g1(
            &art_bytes,
            &store_bytes,
            &tokenizer_bytes,
            Some(&graph),
            8,
        )
        .expect("greedy engine builds");
        assert_eq!(
            greedy.ask("g").expect("greedy ask serves").witness.engine,
            "r4g1-beam"
        );
    }

    /// #790 item 8: state-file persists are atomic (no temp residue, full
    /// content lands) and failures are surfaced, not swallowed — the
    /// falsifier drives a parent path that is a file, which the old
    /// `let _ = std::fs::write` sites would have silently ignored.
    #[test]
    fn persist_state_file_is_atomic_and_fails_loudly() {
        let dir = std::env::temp_dir().join(format!("uor-r4-790-8-persist-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture dir");

        let target = dir.join("nested/last_engine.txt");
        super::persist_state_file(&target, "r4g1").expect("persist succeeds");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "r4g1");
        super::persist_state_file(&target, "attention").expect("overwrite succeeds");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "attention");
        let residue: Vec<_> = std::fs::read_dir(target.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(residue.is_empty(), "no temp files may remain: {residue:?}");

        let blocker = dir.join("blocker");
        std::fs::write(&blocker, b"file, not a directory").unwrap();
        let err = super::persist_state_file(&blocker.join("x.txt"), "y")
            .expect_err("a file in the parent path must surface an error");
        assert!(!err.to_string().is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_remote_url_parses_various_formats() {
        let (host, port, path) = parse_remote_url("http://127.0.0.1:8000/v1");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8000);
        assert_eq!(path, "/v1/chat/completions");

        let (host, port, path) = parse_remote_url("http://localhost:9000/v1/chat/completions");
        assert_eq!(host, "localhost");
        assert_eq!(port, 9000);
        assert_eq!(path, "/v1/chat/completions");
    }

    #[test]
    fn session_lane_sees_history_beyond_the_shared_eight_token_context() {
        let shared_context = [10, 11, 12, 13, 14, 15, 16, 17];
        let mut first_history = vec![1, 2, 3, 4];
        first_history.extend_from_slice(&shared_context);
        let mut second_history = vec![91, 92, 93, 94];
        second_history.extend_from_slice(&shared_context);

        assert_eq!(
            &first_history[first_history.len() - 8..],
            &second_history[second_history.len() - 8..]
        );
        assert_ne!(
            session_signature_from_tokens(&first_history),
            session_signature_from_tokens(&second_history)
        );
    }

    /// #811: end-to-end ask-path D4 abstention over a real scored
    /// two-region bundle (the `tests/status_policy_common` fixture
    /// shape, rebuilt in-module because integration fixtures cannot be
    /// imported here). Questions go through the real chat tokenizer
    /// (whose encoder prepends BOS + a leading space), so the gated
    /// windows are exactly what `ask` derives: a Novel question
    /// abstains with a typed outcome on both decode paths, a covered
    /// question serves byte-identically to the ungated engine, and an
    /// engine without a policy engine keeps the pre-#811 behavior.
    pub(crate) mod d4_gate_tests {
        use crate::chat::{
            replayable_normative_chat_step_for_evidence, ChatEngine, ChatPolicyEngine, SampleRng,
            DEFAULT_SAMPLE_SEED, MAX_CHAT_HISTORY,
        };
        use uor_r4_core::transformerless::compiler::{self, D, K, SIG_BYTES, STAGES};
        use uor_r4_core::transformerless::runtime;
        use uor_r4_core::transformerless::scenarios::Tokenizer;
        use uor_r4_graph_certify::{
            self as score, ContextRow, EmissionTables, RegionParams, Smoothing, StructuralEdge,
        };
        use uor_r4_graph_format::ScoreQ;

        fn xorshift(state: &mut u64) -> u64 {
            let mut x = *state;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *state = x;
            x
        }

        /// A minimal deterministic `Compiled` (mirrors
        /// `tests/status_policy_common::synthetic_compiled`).
        fn synthetic_compiled() -> compiler::Compiled {
            let vocab = 64usize;
            let mut rng = 0xC0DE42u64;
            let mut rand_bytes = |n: usize| -> Vec<u8> {
                (0..n).map(|_| (xorshift(&mut rng) & 0xff) as u8).collect()
            };
            compiler::Compiled {
                token_codes: rand_bytes(vocab * STAGES),
                stage_books: (0..STAGES)
                    .map(|_| rand_bytes(K * D).iter().map(|&b| b as i8).collect())
                    .collect(),
                stage_shifts: vec![0; STAGES],
                thresholds: vec![0; D],
                class_sigs: (0..STAGES).map(|_| rand_bytes(K * SIG_BYTES)).collect(),
                ctx_cb: Vec::new(),
                token_stage_kappas: Vec::new(),
                dot_cb: Vec::new(),
                resid_cb: Vec::new(),
                resid_scale_shifts: Vec::new(),
                norm_fold_const: 0,
            }
        }

        /// The 64-piece chat tokenizer: `<unk>`/`<s>`/`</s>`, a space
        /// piece (the encoder prepends BOS + one space), then 60 unique
        /// single characters at ids 4..=63.
        fn fixture_tokenizer_bytes() -> Vec<u8> {
            let alphabet: Vec<u8> = (b'A'..=b'Z')
                .chain(b'a'..=b'z')
                .chain(b'0'..=b'7')
                .collect();
            assert_eq!(alphabet.len(), 60);
            let mut bytes = Vec::new();
            for piece in [
                b"<unk>".as_slice(),
                b"<s>".as_slice(),
                b"</s>".as_slice(),
                b" ".as_slice(),
            ] {
                bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
                bytes.extend_from_slice(piece);
            }
            for &ch in &alphabet {
                bytes.extend_from_slice(&1i32.to_le_bytes());
                bytes.push(ch);
            }
            bytes
        }

        /// Encode a question exactly as the chat path will (BOS + space
        /// prepended by the encoder itself).
        pub(crate) fn encode_question(tokenizer: &Tokenizer, question: &str) -> Vec<u32> {
            let mut buffer = [0u32; 16];
            let count = tokenizer
                .encode_into(question, &mut buffer)
                .expect("fixture question encodes");
            buffer[..count].to_vec()
        }

        fn policy_engine(graph: &[u8], teacher: &[u8]) -> uor_r4_api::engine::R4Engine {
            uor_r4_api::engine::R4Engine::load(uor_r4_api::engine::EngineParts {
                graph,
                signature_artifact: teacher,
                tokenizer: None,
                score_report: None,
            })
            .expect("the deployed policy engine loads the scored fixture")
        }

        /// The complete fixture: a scored graph whose region 0 is
        /// anchored at the derived signature of the COVERED question's
        /// real encoded window, a bigram context row so the packed
        /// walk has candidates to serve on that window, and a scanned
        /// NOVEL question whose encoded window the deployed policy
        /// abstains on. Returns
        /// `(graph, teacher, covered question, novel question)`.
        pub(crate) fn fixture() -> (Vec<u8>, Vec<u8>, String, String) {
            fixture_with_lane_shape(false, false)
        }

        fn fixture_with_extra_lower_ranked_candidate(
            extra_lower_ranked_candidate: bool,
        ) -> (Vec<u8>, Vec<u8>, String, String) {
            fixture_with_lane_shape(extra_lower_ranked_candidate, false)
        }

        fn fixture_with_psib_partner() -> (Vec<u8>, Vec<u8>, String, String) {
            fixture_with_lane_shape(false, true)
        }

        fn fixture_with_lane_shape(
            extra_lower_ranked_candidate: bool,
            psib_partner: bool,
        ) -> (Vec<u8>, Vec<u8>, String, String) {
            let artifacts = synthetic_compiled();
            let teacher = compiler::artifact_bytes(&artifacts);
            let tokenizer =
                Tokenizer::from_bytes(&fixture_tokenizer_bytes()).expect("fixture tokenizer");
            let covered_question = "C".to_owned();
            let covered_window = encode_question(&tokenizer, &covered_question);
            let covered_last = *covered_window.last().expect("nonempty window");
            let rotations = compiler::derive_rotations();
            let window_tail =
                &covered_window[covered_window.len().saturating_sub(compiler::WINDOW)..];
            let bundle = runtime::bundle_window_plain(&artifacts, &rotations, window_tail);
            let covered_sig = runtime::sig_plain(&artifacts, &bundle);

            let emissions = EmissionTables {
                root_prior: [(10u32, 100i32), (20, 200), (30, 300), (40, 50)]
                    .into_iter()
                    .map(|(token, raw)| (token, ScoreQ::from_raw(raw)))
                    .collect(),
                root_floor: ScoreQ::from_raw(-7000),
                root_total: 1000,
                region_lists: vec![
                    vec![(10, ScoreQ::from_raw(1000)), (20, ScoreQ::from_raw(-500))],
                    vec![(20, ScoreQ::from_raw(2000)), (30, ScoreQ::from_raw(100))],
                ],
                smoothing: Smoothing::AddOne,
                root_prior_quantization: score::QuantizationErrorStats::default(),
                emission_quantization: score::QuantizationErrorStats::default(),
                selection_stats: score::EmissionSelectionStats::default(),
            };
            let regions = [
                RegionParams {
                    node: 1,
                    depth: 1,
                    radius: 4,
                    sig: covered_sig,
                    trajectory_sig: None,
                    trajectory_radius: None,
                    parent: None,
                },
                RegionParams {
                    node: 2,
                    depth: 1,
                    radius: 4,
                    sig: [0xFF; SIG_BYTES],
                    trajectory_sig: None,
                    trajectory_radius: None,
                    parent: None,
                },
            ];
            // The packed walk on a scored graph serves candidates from
            // context rows (the canary's own witness: non-node queries);
            // one bigram row keyed on the covered question's last token
            // gives the walk something real to emit.
            let mut context_entries =
                vec![(10, ScoreQ::from_raw(900)), (20, ScoreQ::from_raw(400))];
            if extra_lower_ranked_candidate {
                context_entries.push((30, ScoreQ::from_raw(100)));
            }
            let context_rows = [ContextRow {
                context_len: 1,
                key0: covered_last,
                key1: 0,
                entries: context_entries,
            }];
            // A runtime-only partner for #933's default-sampled base-bypass
            // falsifier. Token 50 is absent from every base emission list;
            // only the normative SKMX/PSIB lane can place it in the shortlist.
            let (skipmix_rows, psi_bag_rows) = if psib_partner {
                (
                    Vec::new(),
                    vec![(covered_last, vec![(50u32, 1_500_000_000i32)])],
                )
            } else {
                (
                    vec![(covered_last, covered_last, vec![(50u32, 1_500_000_000i32)])],
                    Vec::new(),
                )
            };
            let store: runtime::Store = (0..=STAGES).map(|_| Default::default()).collect();
            let tls1 = runtime::store_bytes(&store);
            let (graph, _) = score::emit_scored_r4g1(
                &teacher,
                (b"chat-d4-meta", b"chat-d4-recs"),
                64,
                &score::ScoredGraphSections {
                    regions: &regions,
                    structural: &[
                        StructuralEdge {
                            src: 0,
                            kind: 0,
                            dst: 1,
                            score_q: ScoreQ::ZERO,
                        },
                        StructuralEdge {
                            src: 0,
                            kind: 0,
                            dst: 2,
                            score_q: ScoreQ::ZERO,
                        },
                    ],
                    transitions: &[],
                    transition_quantization: score::QuantizationErrorStats::default(),
                    emissions: &emissions,
                    context_rows: &context_rows,
                    fwd_rows: &[],
                    skipmix_rows: &skipmix_rows,
                    psi_bag_rows: &psi_bag_rows,
                    exct_tls1: &tls1,
                    exct_top_x: score::ScoreConfig::default().exct_top_x,
                },
            );
            uor_r4_graph_runtime::R4G1Runtime::parse(&graph)
                .expect("the packed runtime parses the scored fixture");

            // Sanity + scan with a throwaway policy engine (probing
            // pollutes widen-once bookkeeping; test engines are fresh).
            use uor_r4_api::engine::PredictDecision;
            let mut scan = policy_engine(&graph, &teacher);
            assert!(
                matches!(
                    scan.predict_decision(&covered_window),
                    Ok(PredictDecision::Serve(_))
                ),
                "the covered question's own window must resolve Serve"
            );
            let alphabet: Vec<u8> = (b'A'..=b'Z')
                .chain(b'a'..=b'z')
                .chain(b'0'..=b'7')
                .collect();
            let novel_question = alphabet
                .iter()
                .map(|&ch| (ch as char).to_string())
                .filter(|question| question != &covered_question)
                .find(|question| {
                    let window = encode_question(&tokenizer, question);
                    matches!(
                        scan.predict_decision(&window),
                        Ok(PredictDecision::Abstain(_))
                    )
                })
                .expect("some single-character question resolves Novel");
            (graph, teacher, covered_question, novel_question)
        }

        fn chat_engine(
            graph: &[u8],
            teacher: &[u8],
            gated: bool,
            seed: Option<u32>,
            max_tokens: usize,
        ) -> ChatEngine {
            ChatEngine {
                artifacts: compiler::parse_artifacts(teacher).expect("fixture artifacts parse"),
                store: (0..=STAGES).map(|_| Default::default()).collect(),
                r4g1_bytes: Some(graph.to_vec()),
                tokenizer: Tokenizer::from_bytes(&fixture_tokenizer_bytes())
                    .expect("fixture tokenizer parses"),
                history: [0; MAX_CHAT_HISTORY],
                history_len: 0,
                max_tokens,
                sample_rng: seed.map(SampleRng::new),
                policy_engine: gated
                    .then(|| ChatPolicyEngine::Research(policy_engine(graph, teacher))),
            }
        }

        pub(crate) fn r4g1_state(graph: &[u8], teacher: &[u8]) -> crate::r4g1::R4g1State {
            let captured = crate::r4g1::CapturedR4g1Bundle {
                graph: graph.to_vec(),
                signature_artifact: teacher.to_vec(),
                tokenizer: Some(fixture_tokenizer_bytes()),
                score_report: None,
                production_admission: None,
            };
            crate::r4g1::R4g1State::load_captured_for_research_with_source(
                std::path::Path::new("cross-surface/graph/score.r4g1"),
                std::path::Path::new("cross-surface/tless_artifacts.bin"),
                &captured,
                None,
            )
            .expect("R4g1State loads the exact cross-surface bytes")
        }

        fn authoritative_step(
            graph: &[u8],
            teacher: &[u8],
            context: &[u32],
            session_signature: Option<&[u8]>,
        ) -> uor_r4_api::NormativeServingDecision {
            let mut engine =
                uor_r4_api::NormativeServingEngine::load_for_research(uor_r4_api::EngineParts {
                    graph,
                    signature_artifact: teacher,
                    tokenizer: None,
                    score_report: None,
                })
                .expect("direct API loads the exact cross-surface bytes");
            engine
                .predict_with_session_signature(context, session_signature)
                .expect("covered context is in bounds")
        }

        fn served(decision: uor_r4_api::NormativeServingDecision) -> uor_r4_api::NormativeServe {
            match decision {
                uor_r4_api::NormativeServingDecision::Serve(serve) => serve,
                uor_r4_api::NormativeServingDecision::Abstain(_) => {
                    panic!("covered cross-surface context abstained")
                }
                uor_r4_api::NormativeServingDecision::Decline(_) => {
                    panic!("covered cross-surface context declined")
                }
            }
        }

        #[test]
        fn novel_question_abstains_on_both_decode_paths_and_serves_ungated() {
            let (graph, teacher, _, novel_question) = fixture();

            // Default (sampled) path: typed abstention, no token served.
            let mut sampled = chat_engine(&graph, &teacher, true, Some(42), 8);
            let answer = sampled
                .ask(&novel_question)
                .expect("abstention is an answer, not an error");
            let abstention = answer.abstention.expect("novel question abstains");
            assert_eq!(abstention.status, "novel");
            assert!(answer.text.is_empty(), "no text on abstention");
            assert_eq!(answer.generated_tokens, 0, "no token served");
            assert_eq!(answer.witness.engine, "r4g1-sampled");

            // Greedy beam opt-out: the question-boundary gate fires too.
            let mut beam = chat_engine(&graph, &teacher, true, None, 8);
            let answer = beam.ask(&novel_question).expect("abstention is an answer");
            assert!(answer.abstention.is_some(), "beam question gate fires");
            assert_eq!(answer.witness.engine, "r4g1-beam");

            // No policy engine (pre-#811 shape): the same question runs
            // ungated — whatever the walk did before, it still does.
            let mut ungated = chat_engine(&graph, &teacher, false, Some(42), 8);
            if let Ok(answer) = ungated.ask(&novel_question) {
                assert!(
                    answer.abstention.is_none(),
                    "no gate without a policy engine"
                );
            }
        }

        #[test]
        fn covered_question_serves_byte_identically_to_the_ungated_walk() {
            let (graph, teacher, covered_question, _) = fixture();

            // One served step (max_tokens 1): the gate decides Serve and
            // the walk's own selection is untouched — gated and ungated
            // engines produce the identical answer under the same seed.
            let mut gated = chat_engine(&graph, &teacher, true, Some(42), 1);
            let mut ungated = chat_engine(&graph, &teacher, false, Some(42), 1);
            let gated_answer = gated
                .ask(&covered_question)
                .expect("covered question serves");
            let ungated_answer = ungated
                .ask(&covered_question)
                .expect("covered question serves ungated");
            assert!(gated_answer.abstention.is_none());
            assert_eq!(gated_answer.text, ungated_answer.text);
            assert_eq!(
                gated_answer.generated_tokens,
                ungated_answer.generated_tokens
            );
            assert_eq!(gated_answer.witness, ungated_answer.witness);
            assert!(!gated_answer.text.is_empty(), "a real token was served");
        }

        /// #933 regression: beam width must not turn speculative hypotheses
        /// into a D4 bypass. The planted covered first context has real
        /// runtime candidates, while every one-token continuation is Novel
        /// under the bound policy. A first-step-only gate would return a
        /// partial multi-token answer; the composed beam must instead prune
        /// every blocked hypothesis and expose a typed abstention with no
        /// generated token.
        #[test]
        fn beam_replays_d4_at_later_steps_and_never_serves_a_blocked_partial() {
            let (graph, teacher, covered_question, _) = fixture();
            let tokenizer =
                Tokenizer::from_bytes(&fixture_tokenizer_bytes()).expect("fixture tokenizer");
            let context = encode_question(&tokenizer, &covered_question);
            let session_signature = uor_r4_router::session_signature_from_tokens(&context);
            let first = served(authoritative_step(
                &graph,
                &teacher,
                &context,
                Some(&session_signature),
            ));
            assert!(!first.candidates.ranked().is_empty());
            for candidate in first.candidates.ranked() {
                let mut extended = context.clone();
                extended.push(candidate.token);
                assert!(
                    matches!(
                        authoritative_step(&graph, &teacher, &extended, Some(&session_signature),),
                        uor_r4_api::NormativeServingDecision::Abstain(_)
                    ),
                    "fixture candidate {} must exercise the later-step D4 block",
                    candidate.token
                );
            }

            let mut beam = chat_engine(&graph, &teacher, true, None, 8);
            let answer = beam
                .ask(&covered_question)
                .expect("later-step policy block is a typed answer");
            assert!(answer.abstention.is_some());
            assert!(answer.text.is_empty());
            assert_eq!(answer.generated_tokens, 0);
            assert_eq!(answer.witness.engine, "r4g1-beam");
            assert_eq!(
                beam.history_len,
                context.len(),
                "no speculative or partially generated token enters chat history"
            );
            let beam_counters = beam
                .policy_engine
                .as_ref()
                .expect("beam policy")
                .policy_counters();
            assert_eq!((beam_counters.predicts, beam_counters.serves), (2, 1));
            assert_eq!(beam_counters.abstains, 1);
            assert_eq!(
                beam_counters.widen_attempts, 1,
                "only the best blocked prefix is committed; speculative beam replays retain no counters or widen-once state"
            );

            let mut sampled = chat_engine(&graph, &teacher, true, Some(42), 8);
            let answer = sampled
                .ask(&covered_question)
                .expect("sampled later-step policy block is typed");
            assert!(answer.abstention.is_some());
            assert!(answer.text.is_empty());
            assert_eq!(answer.generated_tokens, 0);
            assert_eq!(answer.witness.engine, "r4g1-sampled");
            assert_eq!(
                sampled.history_len,
                context.len(),
                "sampled decode also drops a blocked partial trajectory"
            );
            let sampled_counters = sampled
                .policy_engine
                .as_ref()
                .expect("sampled policy")
                .policy_counters();
            assert_eq!((sampled_counters.predicts, sampled_counters.serves), (2, 1));
            assert_eq!(sampled_counters.abstains, 1);
        }

        /// #933: execute the same real graph/teacher bytes through the direct
        /// API, the server's `R4g1State` greedy/default-sampled methods, and
        /// both CLI-chat policies. The emitted JSON bytes are directly usable
        /// as `ServingReportEvidence.cross_surface_evidence`; counts come from
        /// the artifact itself, never a parallel handwritten claim.
        #[test]
        fn cross_surface_parity_evidence_uses_real_bytes_and_catches_base_bypass() {
            use uor_r4_api::{
                CrossSurfaceDisposition, CrossSurfaceParityEvidence,
                CrossSurfaceParityEvidenceBuilder, CrossSurfaceParityObservation,
                NormativeServingDecision,
            };
            use uor_r4_graph_runtime::ServedCandidateSource;

            let (graph, teacher, covered_question, _) = fixture();
            let tokenizer =
                Tokenizer::from_bytes(&fixture_tokenizer_bytes()).expect("fixture tokenizer");
            let context = encode_question(&tokenizer, &covered_question);
            let session_signature = uor_r4_router::session_signature_from_tokens(&context);
            let mut builder = CrossSurfaceParityEvidenceBuilder::new(&graph, &teacher);

            let direct = authoritative_step(&graph, &teacher, &context, None);
            let direct_token = served(direct).token;
            builder
                .record(CrossSurfaceParityObservation {
                    surface: "direct-api",
                    decode_policy: "greedy",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative: direct,
                    authoritative_token: Some(direct_token),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(direct_token),
                    observed_candidates: Some(served(direct).candidates),
                })
                .expect("record direct API parity");

            let state = r4g1_state(&graph, &teacher);
            let mut greedy_out = [0u32; 1];
            let (greedy_status, greedy_observed) = state
                .generate_into_status_with_first_step(&context, &mut greedy_out)
                .expect("R4g1State greedy serves");
            assert_eq!(greedy_status.count, 1);
            let state_authoritative = authoritative_step(&graph, &teacher, &context, None);
            let state_expected = served(state_authoritative).token;
            builder
                .record(CrossSurfaceParityObservation {
                    surface: "r4g1-state",
                    decode_policy: "greedy",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative: state_authoritative,
                    authoritative_token: Some(state_expected),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(greedy_out[0]),
                    observed_candidates: Some(
                        served(greedy_observed.expect("state captured first step")).candidates,
                    ),
                })
                .expect("record state greedy parity");

            let sampled_authoritative = authoritative_step(&graph, &teacher, &context, None);
            let sampled_serve = served(sampled_authoritative);
            let mut expected_rng = SampleRng::new(DEFAULT_SAMPLE_SEED);
            let sampled_expected = sampled_serve.select_sampled_token(&[], &mut expected_rng);
            let sampled_candidate = sampled_serve
                .candidates
                .ranked()
                .iter()
                .find(|candidate| candidate.token == sampled_expected)
                .expect("sampled token belongs to runtime shortlist");
            assert_eq!(
                sampled_candidate.source,
                ServedCandidateSource::Skipmix,
                "the pinned default seed must select the runtime-only planted partner"
            );
            assert!(sampled_candidate.skmx_contributed);
            assert!(!sampled_candidate.psib_contributed);
            assert_ne!(
                sampled_expected, sampled_serve.base_token,
                "a base/reference-candidate bypass would erase this falsifier"
            );
            let sampled_state = r4g1_state(&graph, &teacher);
            let mut sampled_out = [0u32; 1];
            let mut sampled_rng = SampleRng::new(DEFAULT_SAMPLE_SEED);
            let (sampled_status, sampled_observed) = sampled_state
                .generate_sampled_into_status_with_first_step(
                    &context,
                    &mut sampled_out,
                    &mut sampled_rng,
                )
                .expect("server-default state sampling serves");
            assert_eq!(sampled_status.count, 1);
            builder
                .record(CrossSurfaceParityObservation {
                    surface: "server-r4g1-state",
                    decode_policy: "default-sampled-seed-42",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative: sampled_authoritative,
                    authoritative_token: Some(sampled_expected),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(sampled_out[0]),
                    observed_candidates: Some(
                        served(sampled_observed.expect("state captured sampled first step"))
                            .candidates,
                    ),
                })
                .expect("record server default-sampled parity");

            let chat_sample_authoritative =
                authoritative_step(&graph, &teacher, &context, Some(&session_signature));
            let mut chat_expected_rng = SampleRng::new(DEFAULT_SAMPLE_SEED);
            let chat_sample_expected =
                served(chat_sample_authoritative).select_sampled_token(&[], &mut chat_expected_rng);
            let mut sampled_chat =
                chat_engine(&graph, &teacher, true, Some(DEFAULT_SAMPLE_SEED), 1);
            let sampled_answer = sampled_chat
                .ask(&covered_question)
                .expect("CLI sampled chat serves");
            assert_eq!(sampled_answer.generated_tokens, 1);
            let sampled_chat_token = sampled_chat.history[sampled_chat.history_len - 1];
            let (_, sampled_chat_candidates) = replayable_normative_chat_step_for_evidence(
                &graph,
                &teacher,
                None,
                None,
                &context,
                &session_signature,
                Some(DEFAULT_SAMPLE_SEED),
            )
            .expect("capture sampled CLI shortlist");
            builder
                .record(CrossSurfaceParityObservation {
                    surface: "cli-chat",
                    decode_policy: "default-sampled-seed-42",
                    context_tokens: &context,
                    session_signature: Some(&session_signature),
                    authoritative: chat_sample_authoritative,
                    authoritative_token: Some(chat_sample_expected),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(sampled_chat_token),
                    observed_candidates: Some(sampled_chat_candidates),
                })
                .expect("record CLI sampled parity");

            let chat_beam_authoritative =
                authoritative_step(&graph, &teacher, &context, Some(&session_signature));
            let chat_beam_expected = served(chat_beam_authoritative).token;
            let mut beam_chat = chat_engine(&graph, &teacher, true, None, 1);
            let beam_answer = beam_chat
                .ask(&covered_question)
                .expect("CLI beam first step serves");
            assert_eq!(beam_answer.generated_tokens, 1);
            let beam_chat_token = beam_chat.history[beam_chat.history_len - 1];
            let (_, beam_chat_candidates) = replayable_normative_chat_step_for_evidence(
                &graph,
                &teacher,
                None,
                None,
                &context,
                &session_signature,
                None,
            )
            .expect("capture beam CLI shortlist");
            builder
                .record(CrossSurfaceParityObservation {
                    surface: "cli-chat",
                    decode_policy: "beam-first-step",
                    context_tokens: &context,
                    session_signature: Some(&session_signature),
                    authoritative: chat_beam_authoritative,
                    authoritative_token: Some(chat_beam_expected),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(beam_chat_token),
                    observed_candidates: Some(beam_chat_candidates),
                })
                .expect("record CLI beam parity");

            let evidence = builder.finish().expect("finish parity evidence");
            assert_eq!(evidence.checks, 5);
            assert_eq!(evidence.mismatches, 0);
            let bytes = evidence
                .deterministic_json_bytes()
                .expect("deterministic parity JSON");
            assert_eq!(
                bytes,
                evidence
                    .clone()
                    .deterministic_json_bytes()
                    .expect("repeat deterministic parity JSON")
            );
            let reparsed = CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                &bytes, &graph, &teacher,
            )
            .expect("parity artifact parses, reproduces, and binds real bytes");
            assert_eq!((reparsed.checks, reparsed.mismatches), (5, 0));
            assert!(
                CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                    &bytes,
                    b"a different graph cannot inherit this evidence",
                    &teacher,
                )
                .is_err()
            );

            let mut old_schema: serde_json::Value =
                serde_json::from_slice(&bytes).expect("parse evidence for schema control");
            old_schema["schema"] = serde_json::json!("uor-r4-normative-selector-cross-surface/3");
            let mut old_schema_bytes =
                serde_json::to_vec_pretty(&old_schema).expect("serialize old-schema control");
            old_schema_bytes.push(b'\n');
            assert!(
                CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                    &old_schema_bytes,
                    &graph,
                    &teacher,
                )
                .is_err(),
                "version-3 parity evidence cannot inherit version-4 provenance credit"
            );

            let mut tampered_counts: serde_json::Value =
                serde_json::from_slice(&bytes).expect("parse evidence for planted tamper");
            tampered_counts["checks"] = serde_json::json!(6);
            let mut tampered_count_bytes =
                serde_json::to_vec_pretty(&tampered_counts).expect("serialize planted tamper");
            tampered_count_bytes.push(b'\n');
            assert!(
                CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                    &tampered_count_bytes,
                    &graph,
                    &teacher,
                )
                .is_err()
            );

            let mut tampered_verdict: serde_json::Value =
                serde_json::from_slice(&bytes).expect("parse evidence for planted row tamper");
            tampered_verdict["records"][0]["matched"] = serde_json::json!(false);
            tampered_verdict["mismatches"] = serde_json::json!(1);
            let mut tampered_verdict_bytes =
                serde_json::to_vec_pretty(&tampered_verdict).expect("serialize planted row tamper");
            tampered_verdict_bytes.push(b'\n');
            assert!(
                CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                    &tampered_verdict_bytes,
                    &graph,
                    &teacher,
                )
                .is_err()
            );

            if let Some(path) = std::env::var_os("R4_CROSS_SURFACE_EVIDENCE_OUT") {
                let path = std::path::PathBuf::from(path);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).expect("create evidence output directory");
                }
                std::fs::write(&path, &bytes).expect("write cross-surface evidence bytes");
                eprintln!(
                    "cross-surface evidence: {} checks, {} mismatches, blake3:{}, {}",
                    evidence.checks,
                    evidence.mismatches,
                    blake3::hash(&bytes).to_hex(),
                    path.display()
                );
            }

            // Planted divergence: substituting the internal base token must
            // turn the same real-byte observation into an exact mismatch.
            let divergence_authoritative = authoritative_step(&graph, &teacher, &context, None);
            let divergence_serve = served(divergence_authoritative);
            let mut divergence = CrossSurfaceParityEvidenceBuilder::new(&graph, &teacher);
            divergence
                .record(CrossSurfaceParityObservation {
                    surface: "planted-divergence",
                    decode_policy: "default-sampled-seed-42",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative: divergence_authoritative,
                    authoritative_token: Some(sampled_expected),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(divergence_serve.base_token),
                    observed_candidates: Some(divergence_serve.candidates),
                })
                .expect("record planted divergence");
            let divergence = divergence.finish().expect("finish planted divergence");
            assert_eq!((divergence.checks, divergence.mismatches), (1, 1));
            assert!(!divergence.records[0].matched);

            // Candidate-only negative control: an alternate real runtime has
            // one extra lower-ranked context candidate, while retaining the
            // same greedy winner. Token-only parity would incorrectly pass;
            // the independently captured shortlist CID must fail it.
            let (alternate_graph, alternate_teacher, alternate_question, _) =
                fixture_with_extra_lower_ranked_candidate(true);
            assert_eq!(alternate_teacher, teacher);
            assert_eq!(alternate_question, covered_question);
            let authoritative = authoritative_step(&graph, &teacher, &context, None);
            let authoritative_serve = served(authoritative);
            let alternate = authoritative_step(&alternate_graph, &teacher, &context, None);
            let alternate_serve = served(alternate);
            assert_eq!(alternate_serve.token, authoritative_serve.token);
            assert_ne!(
                alternate_serve.candidates, authoritative_serve.candidates,
                "planted control must change only the bounded shortlist"
            );
            let mut candidate_divergence = CrossSurfaceParityEvidenceBuilder::new(&graph, &teacher);
            candidate_divergence
                .record(CrossSurfaceParityObservation {
                    surface: "planted-candidate-only-divergence",
                    decode_policy: "greedy",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative,
                    authoritative_token: Some(authoritative_serve.token),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(alternate_serve.token),
                    observed_candidates: Some(alternate_serve.candidates),
                })
                .expect("record candidate-only divergence");
            let candidate_divergence = candidate_divergence
                .finish()
                .expect("finish candidate-only divergence");
            assert_eq!(
                (candidate_divergence.checks, candidate_divergence.mismatches),
                (1, 1)
            );
            assert!(!candidate_divergence.records[0].matched);
            assert!(candidate_divergence
                .validate_canonical_production_inventory()
                .is_err());
            let mismatch_root = std::env::temp_dir().join(format!(
                "uor-r4-candidate-divergence-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&mismatch_root);
            let mismatch_path = crate::cross_surface_parity::write_canonical_cross_surface_parity(
                &mismatch_root,
                &candidate_divergence,
            )
            .expect("persist deterministic candidate mismatch rows before STOP");
            let mismatch_bytes =
                std::fs::read(&mismatch_path).expect("read persisted candidate mismatch");
            let replayed = CrossSurfaceParityEvidence::parse_and_validate_for_artifacts(
                &mismatch_bytes,
                &graph,
                &teacher,
            )
            .expect("persisted mismatch remains replayable evidence");
            assert_eq!((replayed.checks, replayed.mismatches), (1, 1));
            let _ = std::fs::remove_dir_all(&mismatch_root);

            // Provenance-only negative control: SKMX and PSIB produce the
            // same token/score/source shortlist and winner, but their exact
            // contribution flags differ. Version-4 candidate CIDs must retain
            // that distinction.
            let (psib_graph, psib_teacher, psib_question, _) = fixture_with_psib_partner();
            assert_eq!(psib_teacher, teacher);
            assert_eq!(psib_question, covered_question);
            let skmx_authoritative = authoritative_step(&graph, &teacher, &context, None);
            let skmx_serve = served(skmx_authoritative);
            let psib_observed = authoritative_step(&psib_graph, &teacher, &context, None);
            let psib_serve = served(psib_observed);
            let projection = |serve: uor_r4_api::NormativeServe| {
                serve
                    .candidates
                    .ranked()
                    .iter()
                    .map(|candidate| (candidate.token, candidate.score, candidate.source))
                    .collect::<Vec<_>>()
            };
            assert_eq!(skmx_serve.token, psib_serve.token);
            assert_eq!(projection(skmx_serve), projection(psib_serve));
            let skmx_partner = skmx_serve
                .candidates
                .ranked()
                .iter()
                .find(|candidate| candidate.token == 50)
                .expect("SKMX partner");
            let psib_partner = psib_serve
                .candidates
                .ranked()
                .iter()
                .find(|candidate| candidate.token == 50)
                .expect("PSIB partner");
            assert_eq!(
                (skmx_partner.skmx_contributed, skmx_partner.psib_contributed),
                (true, false)
            );
            assert_eq!(
                (psib_partner.skmx_contributed, psib_partner.psib_contributed),
                (false, true)
            );
            let mut provenance_divergence =
                CrossSurfaceParityEvidenceBuilder::new(&graph, &teacher);
            provenance_divergence
                .record(CrossSurfaceParityObservation {
                    surface: "planted-provenance-only-divergence",
                    decode_policy: "greedy",
                    context_tokens: &context,
                    session_signature: None,
                    authoritative: skmx_authoritative,
                    authoritative_token: Some(skmx_serve.token),
                    observed_disposition: CrossSurfaceDisposition::Serve,
                    observed_token: Some(psib_serve.token),
                    observed_candidates: Some(psib_serve.candidates),
                })
                .expect("record provenance-only divergence");
            let provenance_divergence = provenance_divergence
                .finish()
                .expect("finish provenance-only divergence");
            assert_eq!(
                (
                    provenance_divergence.checks,
                    provenance_divergence.mismatches
                ),
                (1, 1)
            );
            assert!(!provenance_divergence.records[0].matched);

            assert!(matches!(direct, NormativeServingDecision::Serve(_)));
        }

        #[test]
        fn canonical_bundle_producer_executes_eight_same_input_candidate_rows() {
            use crate::cross_surface_parity::{
                produce_canonical_cross_surface_parity, CanonicalCrossSurfaceMaterial,
                CanonicalCrossSurfaceSpec,
            };

            let (unbound_graph, teacher, covered_question, _) = fixture();
            let tokenizer_bytes = fixture_tokenizer_bytes();
            let view = uor_r4_graph_format::GraphView::parse(&unbound_graph)
                .expect("cross-surface fixture graph parses");
            let mut builder =
                uor_r4_graph_format::ArtifactBuilder::new(view.header().alignment_log2)
                    .with_flags(view.header().flags);
            for section in view.sections() {
                if section.id == uor_r4_graph_format::SectionId::HEAD {
                    let mut head = section.payload.to_vec();
                    head[32..64].copy_from_slice(blake3::hash(&tokenizer_bytes).as_bytes());
                    builder.add_section(section.id, section.flags, &head);
                } else {
                    builder.add_section(section.id, section.flags, section.payload);
                }
            }
            let graph = builder
                .build()
                .expect("tokenizer-bound cross-surface fixture graph");
            let bound_view = uor_r4_graph_format::GraphView::parse(&graph)
                .expect("tokenizer-bound fixture parses");
            assert_eq!(
                bound_view
                    .head()
                    .expect("tokenizer-bound fixture HEAD")
                    .tokenizer_cid()
                    .0,
                *blake3::hash(&tokenizer_bytes).as_bytes()
            );
            let tokenizer = Tokenizer::from_bytes(&tokenizer_bytes).expect("fixture tokenizer");
            let context = encode_question(&tokenizer, &covered_question);
            assert_eq!(context.len(), 3, "fixture corpus encoding contract");

            let n = context.len();
            let mut corpus_meta = Vec::with_capacity(25);
            corpus_meta.extend_from_slice(&(n as u64).to_le_bytes());
            corpus_meta.extend_from_slice(&1u64.to_le_bytes());
            corpus_meta.extend_from_slice(&42u64.to_le_bytes());
            corpus_meta.push(1);
            let next = [context[1], context[2], 10];
            let mut corpus_records = Vec::with_capacity(n * 12);
            for token in next {
                corpus_records.extend_from_slice(&0u32.to_le_bytes());
                corpus_records.extend_from_slice(&(token as u16).to_le_bytes());
                corpus_records.extend_from_slice(&(token as u16).to_le_bytes());
                corpus_records.extend_from_slice(&(-0.1f32).to_le_bytes());
            }

            let positions = [2u64];
            let score_report = br#"{}"#;
            let spec = CanonicalCrossSurfaceSpec {
                material: CanonicalCrossSurfaceMaterial {
                    graph: &graph,
                    signature_artifact: &teacher,
                    tokenizer: Some(&tokenizer_bytes),
                    score_report: Some(score_report),
                    corpus_meta: &corpus_meta,
                    corpus_records: &corpus_records,
                },
                evaluated_positions: &positions,
                sample_seed: DEFAULT_SAMPLE_SEED,
            };
            let first = produce_canonical_cross_surface_parity(spec)
                .expect("canonical producer executes every shared adapter");
            let second = produce_canonical_cross_surface_parity(spec)
                .expect("canonical producer is deterministic");
            assert_eq!(first, second);
            assert_eq!(first.selected_position, 2);
            assert_eq!((first.evidence.checks, first.evidence.mismatches), (8, 0));
            assert!(first.evidence.records.iter().all(|record| {
                record.context_tokens == context
                    && record.authoritative_ranked_candidates_cid.is_some()
                    && record.authoritative_ranked_candidates_cid
                        == record.observed_ranked_candidates_cid
            }));
            assert!(first.evidence.records.iter().any(|record| {
                record.surface == "cli-chat-shared-production-step"
                    && record.decode_policy == "beam-first-step"
            }));
            assert!(first.evidence.records.iter().any(|record| {
                record.surface == "r4g1-state-native-host-adapter"
                    && record.decode_policy == "default-sampled-seed-42"
            }));
            first
                .evidence
                .validate_canonical_production_inventory()
                .expect("all four same-input cohorts reproduce candidates and tokens");

            let bytes = first
                .evidence
                .deterministic_json_bytes()
                .expect("canonical evidence bytes");
            let mut tampered: serde_json::Value =
                serde_json::from_slice(&bytes).expect("parse evidence for context tamper");
            tampered["records"][0]["context_tokens"][0] = serde_json::json!(63);
            let mut tampered_bytes =
                serde_json::to_vec_pretty(&tampered).expect("serialize context tamper");
            tampered_bytes.push(b'\n');
            assert!(
                uor_r4_api::CrossSurfaceParityEvidence::parse_and_validate_for_bundle(
                    &tampered_bytes,
                    &graph,
                    &teacher,
                    Some(&tokenizer_bytes),
                    None,
                )
                .is_err()
            );
            assert!(
                uor_r4_api::CrossSurfaceParityEvidence::parse_and_validate_for_bundle(
                    &bytes, &graph, &teacher, None, None,
                )
                .is_err(),
                "a tokenizer-bound artifact must reject a missing tokenizer"
            );
            assert!(
                uor_r4_api::CrossSurfaceParityEvidence::parse_and_validate_for_bundle(
                    &bytes,
                    &graph,
                    &teacher,
                    Some(b"wrong tokenizer generation"),
                    None,
                )
                .is_err(),
                "a tokenizer-bound artifact must reject different tokenizer bytes"
            );
        }
    }
}
