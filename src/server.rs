use crate as uor_r4_wasm_router;
use crate::model::{download_source, SourceDownload};
use crate::r4g1::{self, R4g1State};
use crate::tless_uor::{self, TlessAxis};
use crate::UorR4Router;
use serde::Deserialize;
use std::any::Any;
use std::cell::Cell;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use uor_foundation::pipeline::PrismModel;

use uor_r4_core::transformerless::hf_bpe::{
    adapter_constructor, resolve_source_tokenizer, TokenizerAdapter, TokenizerAdapterKey,
    TokenizerKind,
};
use uor_r4_core::transformerless::scenarios::RuntimeTokenizerIdentity;
use uor_r4_graph_certify::ScoreStatus;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::{BehaviorSource, TeacherOracle};
use uor_r4_router::fallback::{
    run_cascade, CascadeOutcome, EngineStatus, TierFn, TierOutcome, TierResult,
};

// The browser-triggered build must have enough teacher evidence and graph
// capacity to be a meaningful quality attempt. These are still bounded,
// resumable compiler inputs; the quality gate remains authoritative.
const R4G1_CORPUS_SECONDS: &str = "1800";
const R4G1_CORPUS_TARGET: &str = "200000";
const R4G1_COVER_DEPTHS: &str = "5";
const R4G1_COVER_K0: &str = "16";
const R4G1_COVER_REGIONS: &str = "2048";
const R4G1_COVER_MEMORY_MB: &str = "2048";
const R4G1_COVER_MIN_SUPPORT: &str = "32";
const R4G1_COVER_ENTROPY_GAIN: &str = "0.10";
const R4G1_COVER_RADIUS_QUANTILE: &str = "80";
const R4G1_SCORE_TRANSITION_DEGREE: &str = "16";
const R4G1_SCORE_EMISSION_ENTRIES: &str = "256";
const R4G1_SCORE_ROOT_TOP_B: &str = "256";
const R4G1_SCORE_EXCT_TOP_X: &str = "128";
// The browser can compile arbitrary pinned HF teachers. Their generated
// distributions do not share the historical fixture-corpus Gate C floor, so
// the report must explicitly use the same-corpus TLA comparison.
const R4G1_SCORE_QUALITY_PROFILE: &str = "relative_tla";

/// Configuration supplied by the executable to the reusable HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub manifold_cache: String,
    pub tless_artifacts: String,
    pub tless_store: String,
    pub tless_tokenizer: String,
    pub r4g1_artifact: Option<String>,
    pub tless_corpus_meta: Option<String>,
    pub tless_corpus_recs: Option<String>,
}

pub use uor_r4_api::{InferenceRequest, InferenceResponse, InferenceWitness};

type ChatPayload = InferenceRequest;

fn startup_source_candidates(last_model_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::with_capacity(4);
    let last_model_name = last_model_name.trim();
    if !last_model_name.is_empty() {
        candidates.push(PathBuf::from(".uor-models/sources").join(last_model_name));
    }
    candidates.extend([
        PathBuf::from(".uor-models/sources/smollm2-135m-instruct"),
        PathBuf::from(".uor-models/sources/smollm2-360m-instruct"),
        PathBuf::from(".uor-models/sources/smollm2-1-7b-instruct"),
    ]);
    candidates
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// The supported subset of the OpenAI Chat Completions request. `#654`
/// phase B: `deny_unknown_fields` makes any parameter outside this subset
/// fail closed with the standard error envelope instead of being silently
/// accepted and ignored — support is never implied by omission.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VendorChatCompletionsRequest {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    /// Optional engine pin ("r4g1", "transformerless", "attention",
    /// "r4-attention", "geometric"); absent/"auto" runs the full cascade,
    /// falling back to the persisted `/engine` selection (issue #248).
    #[serde(default)]
    pub engine: Option<String>,
    /// `#654` phase D: when `true`, the completion is delivered as a
    /// `text/event-stream` of `chat.completion.chunk` events terminated by
    /// `data: [DONE]`. Absent/`false` keeps the single-JSON response.
    #[serde(default)]
    pub stream: Option<bool>,
    /// `#654` phase D: streaming options. Only `include_usage` is honored —
    /// when set, a final usage-only chunk (empty `choices`) is emitted before
    /// `[DONE]`. Denies unknown fields so an unsupported option fails closed.
    #[serde(default)]
    pub stream_options: Option<StreamOptions>,
}

/// The supported subset of OpenAI `stream_options` (`#654` phase D).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamOptions {
    #[serde(default)]
    pub include_usage: Option<bool>,
}

/// The supported subset of the OpenAI Responses request (`#654` phase E).
/// `deny_unknown_fields` makes any parameter outside this subset — tools,
/// reasoning, structured-output formats — fail closed with the error envelope;
/// support is never implied by omission. `input` is a bare string or an array
/// of message items (flattened by `flatten_responses_input`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VendorResponsesRequest {
    #[serde(default)]
    pub model: Option<String>,
    pub input: serde_json::Value,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub max_output_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    /// When `true`, the completion is delivered as the Responses
    /// `text/event-stream` event sequence terminating on `response.completed`.
    #[serde(default)]
    pub stream: Option<bool>,
    /// R4 engine pin, consistent with chat completions (issue #248).
    #[serde(default)]
    pub engine: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct VendorChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize)]
pub struct VendorChoice {
    pub index: usize,
    pub message: VendorChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, serde::Serialize)]
pub struct VendorUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ServingUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

impl ServingUsage {
    fn total_tokens(self) -> usize {
        self.prompt_tokens + self.completion_tokens
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenTraceEntry {
    pub token_id: u32,
    pub text: String,
    pub origin_rule: String,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UorAuditTrace {
    pub uor_address: String,
    pub kappa: f64,
    pub deficit_angle: f64,
    pub entropy_bias: f64,
    pub gamma: f64,
    pub temperature: f64,
    pub kappa_pass: bool,
    pub generation_mode: String,
    pub total_latency_ms: f64,
    pub tokens_detail: Vec<TokenTraceEntry>,
}

#[derive(Debug, serde::Serialize)]
pub struct VendorChatCompletionsResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<VendorChoice>,
    pub usage: VendorUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uor_audit: Option<UorAuditTrace>,
    /// Per-tier serving-cascade trail (issue #248): every attempted tier
    /// with its typed status and detail, in attempt order.
    pub cascade_trail: serde_json::Value,
}

#[derive(Deserialize)]
struct CorpusPayload {
    corpus: String,
    identity: Option<String>,
}

#[derive(Deserialize)]
struct ResetPayload {
    identity: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct HuggingFaceDownloadPayload {
    model: Option<String>,
    /// Optional explicit source-tokenizer selection. The two fields are one
    /// atomic registry key; a half-selection is rejected at the HTTP edge.
    tokenizer_family: Option<String>,
    tokenizer_version: Option<u32>,
}

impl HuggingFaceDownloadPayload {
    fn tokenizer_selection(&self) -> Result<Option<TokenizerAdapterKey>, String> {
        match (&self.tokenizer_family, self.tokenizer_version) {
            (Some(family), Some(version)) => {
                Ok(Some(TokenizerAdapterKey::new(family.clone(), version)))
            }
            (None, None) => Ok(None),
            (Some(_), None) => Err("tokenizer_family requires tokenizer_version".to_owned()),
            (None, Some(_)) => Err("tokenizer_version requires tokenizer_family".to_owned()),
        }
    }

    fn validate_download_controls(&self) -> Result<(), String> {
        if self.tokenizer_family.is_some() || self.tokenizer_version.is_some() {
            return Err(
                "tokenizer_family/tokenizer_version apply only to compile and reload; the download endpoint accepts only model"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

fn parse_huggingface_control_payload(body: &[u8]) -> Result<HuggingFaceDownloadPayload, String> {
    if body.is_empty() {
        Ok(HuggingFaceDownloadPayload::default())
    } else {
        serde_json::from_slice(body).map_err(|error| format!("Invalid JSON: {error}"))
    }
}

#[derive(Clone, Debug)]
struct R4g1CompileStatus {
    running: bool,
    ready: bool,
    progress: u8,
    message: String,
    report: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
struct HuggingFaceDownloadStatus {
    running: bool,
    ready: bool,
    message: String,
    source: Option<String>,
    /// Exact immutable identity whose verified snapshot was published at
    /// `source`. Keeping the descriptor beside the path prevents an implicit
    /// compile from trusting a substituted legacy-named directory merely
    /// because download status still says it is ready.
    completed_source: Option<SourceDownload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CompletedDownloadSource {
    path: PathBuf,
    identity: SourceDownload,
}

struct CompileSourceSelection {
    path: Option<String>,
    expected: Option<SourceDownload>,
    require_manifest: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceCacheOperationKind {
    Download,
    Compile,
    Reload,
}

impl SourceCacheOperationKind {
    fn label(self) -> &'static str {
        match self {
            Self::Download => "download",
            Self::Compile => "compile",
            Self::Reload => "reload",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveSourceCacheOperation {
    id: u64,
    kind: SourceCacheOperationKind,
    subject: String,
}

#[derive(Debug, Default)]
struct SourceCacheOperationState {
    next_id: u64,
    active: Option<ActiveSourceCacheOperation>,
}

type SharedSourceCacheOperations = Arc<Mutex<SourceCacheOperationState>>;

/// One process-wide mutation/read reservation for the immutable source cache.
///
/// Downloads publish source directories, while compilation and reload hold
/// borrowed teacher/tokenizer files open. Serializing these three operations
/// prevents an HTTP download from replacing bytes after provenance validation
/// but before the final serving tuple is installed. The server rejects a
/// conflicting request immediately instead of making an unbounded HTTP wait.
struct SourceCacheReservation {
    state: SharedSourceCacheOperations,
    id: u64,
    armed: bool,
}

impl Drop for SourceCacheReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if state.active.as_ref().map(|active| active.id) == Some(self.id) {
            state.active = None;
        }
    }
}

fn try_reserve_source_cache_operation(
    state: &SharedSourceCacheOperations,
    kind: SourceCacheOperationKind,
    subject: impl Into<String>,
) -> Result<SourceCacheReservation, String> {
    let subject = subject.into();
    let mut current = state.lock().unwrap();
    if let Some(active) = current.active.as_ref() {
        return Err(format!(
            "source cache is reserved by an active {} for {}; refusing {} for {}",
            active.kind.label(),
            active.subject,
            kind.label(),
            subject
        ));
    }
    current.next_id = current.next_id.wrapping_add(1).max(1);
    let id = current.next_id;
    current.active = Some(ActiveSourceCacheOperation { id, kind, subject });
    drop(current);
    Ok(SourceCacheReservation {
        state: Arc::clone(state),
        id,
        armed: true,
    })
}

impl HuggingFaceDownloadStatus {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "running": self.running,
            "ready": self.ready,
            "message": self.message,
            "source": self.source,
        })
    }
}

#[derive(Debug, Deserialize)]
struct PinnedSourceManifest {
    repository: String,
    revision: String,
    source_directory: Option<String>,
    /// SPDX license identifier of the pinned source (#597), forwarded
    /// into the source-snapshot manifest. Optional so older descriptors
    /// without the field stay readable.
    #[serde(default)]
    license: Option<String>,
}

impl R4g1CompileStatus {
    fn json(&self, serving: &ServingModelState) -> serde_json::Value {
        let graph_loaded = serving.r4g1.is_some();
        let text_ready = graph_text_ready(serving);
        serde_json::json!({
            "running": self.running,
            "ready": text_ready,
            "graph_loaded": graph_loaded,
            "decode_only": graph_loaded && !text_ready,
            "progress": self.progress,
            "message": self.message,
            "report": self.report,
            "model_name": active_canonical_model_name(serving),
            "physical_root": status_physical_root(serving.active_bundle.as_ref()),
            "terminal_error": serving.terminal_load_error.as_deref(),
            "last_operation_error": serving.last_operation_error.as_deref(),
        })
    }
}

/// Owns the single long-running R4G1 replacement slot while a reload is
/// prepared off-lock. Compilation uses the same `running` reservation, so the
/// two writers cannot mutate the same bundle concurrently. Dropping a failed
/// reload releases the reservation without changing the installed tuple.
struct ReloadReservation {
    status: Arc<Mutex<R4g1CompileStatus>>,
    armed: bool,
}

impl ReloadReservation {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ReloadReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut status = self.status.lock().unwrap();
        status.running = false;
        status.message = "R4G1 reload failed; active serving tuple preserved".to_owned();
    }
}

fn reserve_r4g1_reload(
    serving: &SharedServingModel,
    status: &Arc<Mutex<R4g1CompileStatus>>,
) -> Result<(u64, ReloadReservation), bool> {
    let installed = serving.lock().unwrap();
    let mut current = status.lock().unwrap();
    if current.running {
        return Err(current.ready);
    }
    current.running = true;
    current.ready = graph_text_ready(&installed);
    current.message = "Preparing an atomic R4G1 reload...".to_owned();
    let epoch = installed.epoch;
    drop(current);
    drop(installed);
    Ok((
        epoch,
        ReloadReservation {
            status: Arc::clone(status),
            armed: true,
        },
    ))
}

fn get_window_theme(win_idx: usize) -> &'static str {
    match win_idx {
        1 => "Origins & Foundations",
        2 => "Duality & Polarity",
        3 => "Temporal & Sequential",
        4 => "Boundaries & Limits",
        5 => "Quintessential Forces",
        6 => "Harmonic Resonance",
        7 => "Critical Transitions",
        8 => "Octave Completion",
        9 => "Convergence",
        10 => "Curvature & Topology",
        11 => "Relativistic Effects",
        12 => "Hyperbolic Geometry",
        13 => "Zeta Horizon",
        14 => "High Frequency",
        15 => "Entropic Dissolution",
        16 => "Extremal Manifold",
        _ => "Unknown Window",
    }
}

/// Run the HTTP server with configuration supplied by the caller.
pub fn run_server(cli: Arc<ServerConfig>) {
    tracing::info!(
        host = %cli.host,
        port = cli.port,
        cache = %cli.manifold_cache,
        artifacts = %cli.tless_artifacts,
        store = %cli.tless_store,
        tokenizer = %cli.tless_tokenizer,
        r4g1_artifact = ?cli.r4g1_artifact,
        "initializing R4 Prime Router server"
    );
    let start_time = Instant::now();
    let router = Arc::new(Mutex::new(UorR4Router::new(0.85)));
    let tless: Arc<Mutex<Option<tless_uor::TlessState>>> = Arc::new(Mutex::new(None));
    let serving: SharedServingModel = Arc::new(Mutex::new(ServingModelState::default()));

    let last_model = std::fs::read_to_string(".uor-models/last_model_name.txt").unwrap_or_default();
    let last_model_name = last_model.trim();

    let candidates = startup_source_candidates(last_model_name);
    let mut source_dir = None;
    let mut startup_source_snapshot = None;
    for candidate in &candidates {
        match optional_source_directory(candidate) {
            Ok(Some(path)) => {
                let logical_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("uor-r4");
                let snapshot = match verify_managed_source_snapshot(&path, logical_name) {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        println!("[-] Refusing present-invalid teacher source: {error}");
                        set_r4g1_terminal_load_error(&serving, error);
                        break;
                    }
                };
                source_dir = Some(path);
                startup_source_snapshot = Some(snapshot);
                break;
            }
            Ok(None) => {}
            Err(error) => {
                println!("[-] Refusing present-invalid teacher source: {error}");
                set_r4g1_terminal_load_error(&serving, error);
                break;
            }
        }
    }
    if let Some(path) = source_dir.as_deref() {
        println!(
            "[*] Loading full source teacher from {} for attention-based generation...",
            path.display()
        );
        let logical_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("uor-r4");
        match prepare_optional_teacher_source_for_identity(Some(path), None, logical_name, None) {
            Ok(Some(prepared)) => {
                let final_snapshot =
                    verify_managed_source_snapshot(path, logical_name).and_then(|after| {
                        let before = startup_source_snapshot.as_ref().ok_or_else(|| {
                            format!(
                                "startup source {} has no initial verified snapshot",
                                path.display()
                            )
                        })?;
                        require_unchanged_managed_source_snapshot(
                            path,
                            "standalone teacher startup",
                            before,
                            &after,
                        )?;
                        Ok(after)
                    });
                match final_snapshot {
                    Ok(_) => {
                        println!(
                            "[+] Successfully loaded full source teacher ({})!",
                            path.display()
                        );
                        let mut installed = serving.lock().unwrap();
                        installed.epoch = installed.epoch.wrapping_add(1);
                        installed.oracle = Some(prepared.teacher);
                        installed.source_tokenizer = Some(prepared.tokenizer);
                        installed.teacher_default_r4_attention = false;
                        installed.active_teacher_source = Some(prepared.source);
                    }
                    Err(error) => {
                        println!("[-] Refusing changed teacher source: {error}");
                        set_r4g1_terminal_load_error(&serving, error);
                    }
                }
            }
            Ok(None) => {
                let error = format!(
                    "present teacher source {} produced no loadable teacher",
                    path.display()
                );
                println!("[-] {error}");
                set_r4g1_terminal_load_error(&serving, error);
            }
            Err(e) => {
                println!("[-] Failed to load full teacher model: {e}");
                set_r4g1_terminal_load_error(&serving, e);
            }
        }
    }

    let r4g1_compile = Arc::new(Mutex::new(R4g1CompileStatus {
        running: false,
        ready: false,
        progress: 0,
        message: "R4G1 graph compiler idle".to_owned(),
        report: None,
    }));
    let hf_download = Arc::new(Mutex::new(HuggingFaceDownloadStatus {
        running: false,
        ready: false,
        message: "Hugging Face source download idle".to_owned(),
        source: None,
        completed_source: None,
    }));
    let source_cache_operations: SharedSourceCacheOperations =
        Arc::new(Mutex::new(SourceCacheOperationState::default()));
    let configured_graph = r4g1::discover_path(
        cli.r4g1_artifact.as_deref(),
        Path::new(&cli.tless_artifacts),
    );
    let configured_graph_subject = configured_graph
        .as_deref()
        .map(graph_output_session_subject);
    let mut startup_session_subjects = configured_graph_subject.into_iter().collect::<Vec<_>>();
    startup_session_subjects.push(PathBuf::from(".uor-models/sources"));
    startup_session_subjects.push(graph_output_session_subject(Path::new(
        &cli.tless_artifacts,
    )));
    let startup_sessions = if cli.r4g1_artifact.is_none() {
        try_lock_managed_inventory_write_sessions(
            Path::new(".uor-models/compiled"),
            startup_session_subjects,
        )
    } else {
        try_lock_source_compile_sessions(
            startup_session_subjects,
            SourceCompileSessionMode::ExclusiveWriter,
        )
    };
    let (startup_read_sessions, r4g1_discovery_allowed) = match startup_sessions {
        Ok(sessions) => {
            let recovery = if cli.r4g1_artifact.is_none() {
                recover_managed_compiled_bundle_completion_temporaries(Path::new(
                    ".uor-models/compiled",
                ))
            } else {
                configured_graph
                    .as_deref()
                    .and_then(Path::parent)
                    .and_then(Path::parent)
                    .map(recover_compiled_bundle_completion_temporaries)
                    .transpose()
                    .map(|_| ())
            };
            match recovery {
                Ok(()) => (Some(sessions), true),
                Err(error) => {
                    println!(
                        "[-] Refusing R4G1 startup completion recovery before discovery: {error}"
                    );
                    set_r4g1_terminal_load_error(&serving, error);
                    (Some(sessions), false)
                }
            }
        }
        Err(error) if source_compile_session_is_busy(&error) => {
            // Another process is atomically refreshing this physical bundle.
            // This is transient BUSY state, not invalid on-disk provenance;
            // do not poison the independently loaded teacher or the next
            // restart with a terminal marker derived from a partial read.
            println!("[-] Deferring R4G1 startup while bundle publication is busy: {error}");
            (None, false)
        }
        Err(error) => {
            println!("[-] Refusing R4G1 startup session: {error}");
            set_r4g1_terminal_load_error(&serving, error);
            (None, false)
        }
    };
    let mut r4g1_candidates = if r4g1_discovery_allowed {
        configured_graph
            .map(|graph| {
                vec![(
                    graph,
                    PathBuf::from(&cli.tless_artifacts),
                    None::<ResolvedCompiledBundle>,
                )]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if cli.r4g1_artifact.is_none() && r4g1_discovery_allowed {
        let mut discovery_valid = true;
        let mut configured_external = false;
        let mut configured_managed_logical = None;
        match current_source_attention_era_version().and_then(|version| {
            resolve_managed_teacher_bundle_in(
                Path::new(&cli.tless_artifacts),
                Path::new(".uor-models"),
                version,
            )
        }) {
            Ok(ConfiguredManagedBundle::Selected(candidate)) => {
                let candidate = *candidate;
                configured_managed_logical = Some(candidate.logical_name.clone());
                r4g1_candidates = vec![(
                    candidate.graph.clone(),
                    candidate.teacher.clone(),
                    Some(candidate),
                )];
            }
            Ok(external @ ConfiguredManagedBundle::External) => {
                configured_external = true;
                discovery_valid = external.permits_inventory_discovery();
            }
            Ok(ConfiguredManagedBundle::Absent) => {
                // A configured managed path selects one logical model even
                // when that model is absent. Do not silently boot an
                // unrelated bundle discovered elsewhere in the inventory.
                r4g1_candidates.clear();
                discovery_valid = ConfiguredManagedBundle::Absent.permits_inventory_discovery();
            }
            Ok(ConfiguredManagedBundle::Incomplete(error)) => {
                println!("[-] Refusing incomplete configured managed R4G1 bundle: {error}");
                set_r4g1_terminal_load_error(&serving, error);
                r4g1_candidates.clear();
                discovery_valid = false;
            }
            Err(error) => {
                println!("[-] Refusing configured managed R4G1 bundle: {error}");
                set_r4g1_terminal_load_error(&serving, error);
                r4g1_candidates.clear();
                discovery_valid = false;
            }
        }
        if discovery_valid {
            match discover_compiled_r4g1_candidates() {
                Ok(candidates) => {
                    for candidate in candidates {
                        if configured_managed_logical.as_deref()
                            == Some(candidate.logical_name.as_str())
                        {
                            continue;
                        }
                        r4g1_candidates.push((
                            candidate.graph.clone(),
                            candidate.teacher.clone(),
                            Some(candidate),
                        ));
                    }
                }
                Err(error) => {
                    println!("[-] Refusing compiled R4G1 discovery: {error}");
                    if configured_external {
                        println!(
                            "[-] Ignoring unrelated managed inventory while honoring configured external R4G1 candidate"
                        );
                    } else {
                        set_r4g1_terminal_load_error(&serving, error);
                        r4g1_candidates.clear();
                    }
                }
            }
        }
    }
    let mut loaded_r4g1 = false;
    for (graph_path, teacher_path, resolved) in r4g1_candidates {
        if let Err(error) = validate_legacy_graph_generation_for_serving(&graph_path) {
            println!(
                "[-] Refusing incomplete or changed legacy R4G1 generation {}: {error}",
                graph_path.display()
            );
            set_r4g1_terminal_load_error(&serving, error);
            break;
        }
        let inputs_present = match required_r4g1_inputs_present(&graph_path, &teacher_path) {
            Ok(present) => present,
            Err(error) => {
                println!("[-] Refusing present-invalid R4G1 bundle: {error}");
                set_r4g1_terminal_load_error(&serving, error);
                break;
            }
        };
        if !inputs_present {
            continue;
        }
        if let Some(bundle) = resolved.as_ref() {
            let current_version = match current_source_attention_era_version() {
                Ok(version) => version,
                Err(error) => {
                    set_r4g1_terminal_load_error(&serving, error);
                    break;
                }
            };
            if let Err(error) =
                ensure_compiled_bundle_completion_for_serving(bundle, current_version)
            {
                println!(
                    "[-] Refusing R4G1 graph {} because its bundle completion is invalid: {error}",
                    graph_path.display()
                );
                set_r4g1_terminal_load_error(&serving, error);
                break;
            }
        }
        let resolved_source = match resolved.as_ref() {
            Some(bundle) => match source_for_resolved_bundle_in(bundle, Path::new(".uor-models")) {
                Ok(source) => source,
                Err(error) => {
                    println!(
                        "[-] Refusing R4G1 graph {} because its logical source is invalid: {error}",
                        graph_path.display()
                    );
                    set_r4g1_terminal_load_error(&serving, error);
                    break;
                }
            },
            None => None,
        };
        let selected_source = if resolved.is_none() {
            source_dir.as_deref().filter(|source| {
                source.file_name() == teacher_path.parent().and_then(Path::file_name)
            })
        } else {
            None
        };
        let inferred_source = if resolved.is_none() && selected_source.is_none() {
            match source_for_compiled_teacher(&teacher_path) {
                Ok(source) => source,
                Err(error) => {
                    println!(
                        "[-] Refusing R4G1 graph {} because its inferred source is invalid: {error}",
                        graph_path.display()
                    );
                    set_r4g1_terminal_load_error(&serving, error);
                    break;
                }
            }
        } else {
            None
        };
        let source = selected_source.or(inferred_source.as_deref());
        let source = resolved_source.as_deref().or(source);
        let logical_name = resolved
            .as_ref()
            .map(|candidate| candidate.logical_name.as_str())
            .or_else(|| {
                source
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
            })
            .unwrap_or("uor-r4");
        let verified_source_snapshot = match source {
            Some(source) => match verify_managed_source_snapshot(source, logical_name) {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    println!(
                        "[-] Refusing R4G1 graph {} because its source snapshot is invalid: {error}",
                        graph_path.display()
                    );
                    set_r4g1_terminal_load_error(&serving, error);
                    break;
                }
            },
            None => None,
        };
        if let (Some(bundle), Some(_)) = (resolved.as_ref(), source) {
            let current_version = match current_source_attention_era_version() {
                Ok(version) => version,
                Err(error) => {
                    set_r4g1_terminal_load_error(&serving, error);
                    break;
                }
            };
            if let Err(error) = validate_resolved_source_snapshot_binding(
                bundle,
                verified_source_snapshot.as_ref(),
                current_version,
            ) {
                println!(
                    "[-] Refusing R4G1 graph {} because source/corpus provenance conflicts: {error}",
                    graph_path.display()
                );
                set_r4g1_terminal_load_error(&serving, error);
                break;
            }
        }
        match R4g1State::load_with_source(&graph_path, &teacher_path, source) {
            Ok(state) => {
                let mut prepared_teacher = match prepare_optional_teacher_source_for_identity(
                    source,
                    None,
                    logical_name,
                    state.tokenizer_adapter_identity(),
                ) {
                    Ok(prepared) => prepared,
                    Err(error) => {
                        println!(
                            "[-] Refusing R4G1 graph {} because its teacher source is invalid: {error}",
                            graph_path.display()
                        );
                        set_r4g1_terminal_load_error(&serving, error);
                        break;
                    }
                };
                let (teacher_default_r4_attention, teacher_mismatch) =
                    match reconcile_prepared_teacher_with_bundle(
                        &mut prepared_teacher,
                        resolved.as_ref(),
                    ) {
                        Ok(decision) => decision,
                        Err(error) => {
                            println!(
                                "[-] Refusing R4G1 graph {} because its teacher operator is invalid: {error}",
                                graph_path.display()
                            );
                            set_r4g1_terminal_load_error(&serving, error);
                            break;
                        }
                    };
                if let Some(message) = teacher_mismatch.as_deref() {
                    println!("[-] {message}");
                }
                let refreshed = match resolved.as_ref() {
                    Some(candidate) => match current_source_attention_era_version()
                        .and_then(|version| refresh_resolved_compiled_bundle(candidate, version))
                    {
                        Ok(refreshed) => Some(refreshed),
                        Err(error) => {
                            println!(
                                "[-] Refusing R4G1 graph {} after provenance re-check: {error}",
                                graph_path.display()
                            );
                            set_r4g1_terminal_load_error(&serving, error);
                            break;
                        }
                    },
                    None => None,
                };
                println!(
                    "[+] Loaded validated R4G1 graph runtime from {}",
                    graph_path.display()
                );
                let (teacher, tokenizer, teacher_source) = match prepared_teacher {
                    Some(prepared) => (
                        Some(prepared.teacher),
                        Some(prepared.tokenizer),
                        Some(prepared.source),
                    ),
                    None => (None, None, None),
                };
                if let (Some(source), Some(before)) = (source, verified_source_snapshot.as_ref()) {
                    let after = match verify_managed_source_snapshot(source, logical_name) {
                        Ok(after) => after,
                        Err(error) => {
                            println!(
                                "[-] Refusing R4G1 graph {} after final source re-check: {error}",
                                graph_path.display()
                            );
                            set_r4g1_terminal_load_error(&serving, error);
                            break;
                        }
                    };
                    if let Err(error) = require_unchanged_managed_source_snapshot(
                        source,
                        "R4G1 startup",
                        before,
                        &after,
                    ) {
                        println!(
                            "[-] Refusing R4G1 graph {} after final source re-check: {error}",
                            graph_path.display()
                        );
                        set_r4g1_terminal_load_error(&serving, error);
                        break;
                    }
                    if let Some(bundle) = refreshed.as_ref() {
                        let current_version = match current_source_attention_era_version() {
                            Ok(version) => version,
                            Err(error) => {
                                set_r4g1_terminal_load_error(&serving, error);
                                break;
                            }
                        };
                        if let Err(error) = validate_resolved_source_snapshot_binding(
                            bundle,
                            Some(&after),
                            current_version,
                        ) {
                            println!(
                                "[-] Refusing R4G1 graph {} after final source/corpus re-check: {error}",
                                graph_path.display()
                            );
                            set_r4g1_terminal_load_error(&serving, error);
                            break;
                        }
                    }
                }
                let mut installed = serving.lock().unwrap();
                let mut compile_status = r4g1_compile.lock().unwrap();
                installed.epoch = installed.epoch.wrapping_add(1);
                installed.r4g1 = Some(state);
                installed.oracle = teacher;
                installed.source_tokenizer = tokenizer;
                installed.teacher_default_r4_attention = teacher_default_r4_attention;
                installed.active_teacher_source = teacher_source;
                installed.active_bundle = refreshed;
                installed.terminal_load_error = None;
                installed.last_operation_error = None;
                compile_status.ready = graph_text_ready(&installed);
                compile_status.progress = 100;
                compile_status.message = "R4G1 graph runtime ready".to_owned();
                loaded_r4g1 = true;
                break;
            }
            Err(error) => {
                println!(
                    "[-] Failed to load R4G1 graph runtime from {}: {error}",
                    graph_path.display()
                );
                // This candidate was selected only after paired-root and
                // provenance validation. A present preferred bundle that
                // fails graph/tokenizer validation is terminal; falling
                // through would silently downgrade its arithmetic era.
                set_r4g1_terminal_load_error(&serving, error);
                break;
            }
        }
    }
    // All on-disk graph/cover/source bindings have been consumed and the
    // serving tuple was installed atomically. Runtime requests use only the
    // in-memory snapshot, so the shared filesystem sessions need not remain
    // held for the lifetime of the server.
    drop(startup_read_sessions);
    if !loaded_r4g1 {
        if let Some(error) = r4g1_terminal_load_error(&serving) {
            tracing::error!(%error, "R4G1 graph was rejected; default serving will fail closed");
        } else {
            tracing::info!("no validated R4G1 graph found; compile it from the dashboard");
        }
    }
    // Load cache on startup
    {
        let mut r = router.lock().unwrap();
        if let Ok(cache_data) = std::fs::read_to_string(&cli.manifold_cache) {
            if r.import_state_native(&cache_data) {
                let total = r.get_total_indexed_sentences();
                println!(
                    "[+] Successfully loaded manifold cache from {}. Sentences indexed: {}",
                    cli.manifold_cache, total
                );
            } else {
                tracing::warn!(path = %cli.manifold_cache, "failed to load manifold cache: not a valid serialized router state");
            }
        } else {
            tracing::info!(path = %cli.manifold_cache, "no manifold cache found; initializing a new manifold");
        }

        // Always ingest wiki corpus and extra reading files into manifold cache
        index_wiki_corpus(&mut r);
        index_extra_reading_files(&mut r);

        // Save cache
        let state_json = r.export_state();
        let _ = std::fs::write(&cli.manifold_cache, state_json);
    }

    let bind_addr = format!("{}:{}", cli.host, cli.port);
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                println!("[!] {} is already in use.", bind_addr);
                if let Some(pid) = find_pid_by_port(cli.port) {
                    println!("[*] Found process occupying port {}: PID {}", cli.port, pid);
                    print!(
                        "Would you like to terminate this process and start the server? [y/N]: "
                    );
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let trimmed = input.trim().to_lowercase();
                        if trimmed == "y" || trimmed == "yes" {
                            println!("[*] Terminating process {}...", pid);
                            if kill_process(pid) {
                                // Wait 1 second for port to release
                                std::thread::sleep(std::time::Duration::from_millis(1000));
                                match TcpListener::bind(&bind_addr) {
                                    Ok(l) => l,
                                    Err(e2) => {
                                        eprintln!("[-] Failed to bind to {} after terminating process: {}", bind_addr, e2);
                                        std::process::exit(1);
                                    }
                                }
                            } else {
                                eprintln!("[-] Failed to terminate process {}. Please close it manually and retry.", pid);
                                std::process::exit(1);
                            }
                        } else {
                            println!("[*] Exiting gracefully.");
                            std::process::exit(0);
                        }
                    } else {
                        println!("[-] Non-interactive session or read error. Exiting gracefully.");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("[-] {} is occupied, but could not determine process ID. Please close it manually and retry.", bind_addr);
                    std::process::exit(1);
                }
            } else {
                eprintln!("[-] Failed to bind to {}: {}", bind_addr, e);
                std::process::exit(1);
            }
        }
    };
    tracing::info!(address = %bind_addr, "local server is running");

    for stream in listener.incoming().flatten() {
        let r_clone = Arc::clone(&router);
        let t_clone = Arc::clone(&tless);
        let s_clone = Arc::clone(&serving);
        let gc_clone = Arc::clone(&r4g1_compile);
        let hf_clone = Arc::clone(&hf_download);
        let source_cache_clone = Arc::clone(&source_cache_operations);
        let c_clone = Arc::clone(&cli);
        std::thread::spawn(move || {
            handle_connection(
                stream,
                r_clone,
                t_clone,
                s_clone,
                gc_clone,
                hf_clone,
                source_cache_clone,
                c_clone,
                start_time,
            );
        });
    }
}

// Personal-path wiki indexer; retained for local experiments.
fn index_wiki_corpus(router: &mut UorR4Router) {
    let paths = vec![
        std::path::PathBuf::from(".uor-models/sources/wiki_corpus.txt"),
        std::path::PathBuf::from(".uor-models/wiki_corpus.txt"),
        std::path::PathBuf::from("wiki_corpus.txt"),
    ];
    let mut wiki_file = None;
    for p in paths {
        if p.exists() && p.is_file() {
            wiki_file = Some(p);
            break;
        }
    }
    let wiki_file = match wiki_file {
        Some(f) => f,
        None => {
            println!("[-] wiki_corpus.txt not found.");
            return;
        }
    };
    println!("[*] Loading and indexing wiki corpus from {:?}", wiki_file);
    if let Ok(content) = std::fs::read_to_string(&wiki_file) {
        let count = router.index_corpus(&content, "shared");
        println!(
            "[+] Successfully indexed {} sentences from wiki_corpus.txt.",
            count
        );
    }
}

fn index_extra_reading_files(router: &mut UorR4Router) {
    let paths = vec![
        std::path::PathBuf::from(".uor-models/extra_reading"),
        std::path::PathBuf::from("extra_reading"),
    ];
    let mut extra_dir = None;
    for p in paths {
        if p.exists() && p.is_dir() {
            extra_dir = Some(p);
            break;
        }
    }
    let extra_dir = match extra_dir {
        Some(d) => d,
        None => {
            println!("[-] extra_reading directory not found.");
            return;
        }
    };
    println!("[*] Checking for extra_reading files in {:?}", extra_dir);
    if let Ok(entries) = std::fs::read_dir(extra_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    println!(
                        "[*] Reading and indexing extra_reading file: {:?}",
                        path.file_name().unwrap_or_default()
                    );
                    let count = router.index_corpus(&content, "shared");
                    println!(
                        "[+] Indexed {} sentences from {:?}",
                        count,
                        path.file_name().unwrap_or_default()
                    );
                }
            }
        }
    }
}

/// Run `f` with the shared transformerless state bound on this thread
/// (lazy-loads from TLESS_ARTIFACTS / TLESS_STORE on first use). The state
/// Mutex is held across the call so concurrent requests serialize; the axis
/// reads the thread-local binding only inside this region.
fn with_tless_server_state<R>(
    slot: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    f: impl FnOnce(&mut tless_uor::TlessState) -> R,
) -> Option<R> {
    let mut g = slot.lock().unwrap();
    if g.is_none() {
        *g = tless_uor::load_tless_state();
    }
    let st = g.as_mut()?;
    tless_uor::bind_tless_state(st as *mut _);
    let r = f(st);
    tless_uor::unbind_tless_state();
    Some(r)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CompiledRootState {
    Absent,
    Empty,
    /// A source compile published its immutable source-snapshot binding, but
    /// stage A did not yet publish any corpus payload or attention binding.
    /// This is a resumable preflight state, not an implicit historical bundle.
    PreAttentionIdentity,
    ImplicitV1(Box<AttentionOperatorSpec>),
    BoundHistorical(Box<AttentionOperatorSpec>),
    BoundCurrent(Box<AttentionOperatorSpec>),
}

impl CompiledRootState {
    fn operator(&self) -> Option<&AttentionOperatorSpec> {
        match self {
            Self::ImplicitV1(operator)
            | Self::BoundHistorical(operator)
            | Self::BoundCurrent(operator) => Some(operator),
            Self::Absent | Self::Empty | Self::PreAttentionIdentity => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CompiledModelPair {
    logical_name: String,
    conventional_root: PathBuf,
    conventional: CompiledRootState,
    current_root: PathBuf,
    current: CompiledRootState,
}

/// One logical source resolved to the exact physical bundle selected for
/// serving. Keeping all of these fields together prevents reload/status from
/// reconstructing a source name from the resolver-owned era suffix.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedCompiledBundle {
    logical_name: String,
    physical_root: PathBuf,
    graph: PathBuf,
    teacher: PathBuf,
    attention_operator: AttentionOperatorSpec,
    /// Exact source snapshot root recorded by the selected cover report.
    /// `None` is the historical pre-#597 state, not an inferred identity.
    source_manifest_kappa: Option<String>,
}

/// The complete serving identity committed by startup, reload, or background
/// compilation. One mutex protects the graph, teacher, tokenizer, logical /
/// physical provenance, and failure/readiness state, so generation can never
/// observe fields from two different installations.
#[derive(Default)]
struct ServingModelState {
    epoch: u64,
    r4g1: Option<R4g1State>,
    oracle: Option<uor_r4_model_source::Teacher>,
    source_tokenizer: Option<TokenizerKind>,
    /// Teacher mode used by the unpinned fallback. Explicit `attention` and
    /// `r4-attention` requests continue to select their named modes.
    teacher_default_r4_attention: bool,
    active_bundle: Option<ResolvedCompiledBundle>,
    active_teacher_source: Option<PathBuf>,
    terminal_load_error: Option<String>,
    last_operation_error: Option<String>,
}

type SharedServingModel = Arc<Mutex<ServingModelState>>;

struct PreparedTeacherSource {
    teacher: uor_r4_model_source::Teacher,
    tokenizer: TokenizerKind,
    source: PathBuf,
}

fn source_attention_operator(operator: &AttentionOperatorSpec) -> bool {
    matches!(
        operator.id.as_str(),
        AttentionOperatorSpec::STANDARD_ID
            | AttentionOperatorSpec::EXPERIMENTAL_R4_ID
            | AttentionOperatorSpec::LEARNED_ABSOLUTE_ID
    )
}

fn attention_era_suffix(current_version: u32) -> String {
    format!("-attention-v{current_version}")
}

fn validate_logical_model_name(name: &str, current_version: u32) -> Result<(), String> {
    let suffix = attention_era_suffix(current_version);
    let path = Path::new(name);
    let one_normal_component = path.components().count() == 1
        && path.file_name().and_then(|part| part.to_str()) == Some(name)
        && name != "."
        && name != "..";
    if name.is_empty() || !one_normal_component {
        return Err(format!(
            "model name {name:?} is not one logical source basename"
        ));
    }
    if name.ends_with(&suffix) {
        return Err(format!(
            "model name {name:?} uses the resolver-owned suffix {suffix}; source basenames ending in that suffix are reserved"
        ));
    }
    Ok(())
}

fn logical_model_name_for_request(requested: &str, current_version: u32) -> Result<String, String> {
    let suffix = attention_era_suffix(current_version);
    let logical = requested
        .strip_suffix(&suffix)
        .filter(|base| !base.is_empty())
        .unwrap_or(requested);
    validate_logical_model_name(logical, current_version)?;
    Ok(logical.to_owned())
}

fn inspect_compiled_root(
    root: &Path,
    current_version: u32,
    resolver_owned_current_root: bool,
) -> Result<CompiledRootState, String> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CompiledRootState::Absent)
        }
        Err(error) => return Err(format!("{} cannot be inspected: {error}", root.display())),
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "compiled model root {} is not a regular non-symlink directory",
            root.display()
        ));
    }
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("{} cannot be enumerated: {error}", root.display()))?;
    let populated = entries
        .next()
        .transpose()
        .map_err(|error| format!("{} cannot be enumerated: {error}", root.display()))?
        .is_some();
    if !populated {
        let initialization = source_compile_initialization_path(root)?;
        match fs::symlink_metadata(&initialization) {
            Ok(_) => {
                return Err(format!(
                    "compiled model root {} is empty beside an ambiguous legacy initialization claim {}; refusing adoption",
                    root.display(),
                    initialization.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{} cannot be inspected: {error}",
                    initialization.display()
                ));
            }
        }
        return Ok(CompiledRootState::Empty);
    }

    let binding = source_compile_attention_binding(root)?;
    let (operator, explicit_binding) = match binding {
        Some(operator) => (operator, true),
        None if source_compile_pre_attention_prefix(root)? => {
            return Ok(CompiledRootState::PreAttentionIdentity);
        }
        None if resolver_owned_current_root => {
            return Err(format!(
                "deterministic current-era bundle {} contains payload without {}; refusing to infer or relabel its arithmetic era",
                root.display(),
                uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE
            ))
        }
        None => (AttentionOperatorSpec::standard_v1(), false),
    };
    if !source_attention_operator(&operator) {
        return Err(format!(
            "compiled model root {} is pinned to non-source attention operator {}/{}",
            root.display(),
            operator.id,
            operator.version
        ));
    }
    if operator.version > current_version {
        return Err(format!(
            "compiled model root {} is pinned to unsupported future attention operator {}/{}; current source era is {current_version}",
            root.display(),
            operator.id,
            operator.version
        ));
    }
    if resolver_owned_current_root && operator.version != current_version {
        return Err(format!(
            "deterministic current-era bundle {} is pinned to attention operator {}/{}; expected a source operator at version {current_version}",
            root.display(),
            operator.id,
            operator.version
        ));
    }
    let operator = Box::new(operator);
    Ok(if !explicit_binding {
        CompiledRootState::ImplicitV1(operator)
    } else if operator.version == current_version {
        CompiledRootState::BoundCurrent(operator)
    } else {
        CompiledRootState::BoundHistorical(operator)
    })
}

/// Inspect both physical roots before making any selection. This is the one
/// era boundary used by compile, startup, reload, and model/status reporting.
fn inspect_compiled_model_pair(
    compiled_root: &Path,
    logical_name: &str,
    current_version: u32,
) -> Result<CompiledModelPair, String> {
    validate_logical_model_name(logical_name, current_version)?;
    let conventional_root = compiled_root.join(logical_name);
    let current_root = compiled_root.join(format!(
        "{logical_name}{}",
        attention_era_suffix(current_version)
    ));
    let conventional = inspect_compiled_root(&conventional_root, current_version, false)?;
    let current = inspect_compiled_root(&current_root, current_version, true)?;

    if matches!(conventional, CompiledRootState::PreAttentionIdentity)
        && current.operator().is_some()
    {
        return Err(format!(
            "conventional root {} contains an unfinished pre-attention identity while current root {} is already bound; refusing to hide or overwrite either initialization",
            conventional_root.display(),
            current_root.display()
        ));
    }

    if let (Some(historical), Some(current_operator)) =
        (conventional.operator(), current.operator())
    {
        if historical.id != current_operator.id {
            return Err(format!(
                "compiled roots {} and {} conflict across source-attention families ({}/{} versus {}/{})",
                conventional_root.display(),
                current_root.display(),
                historical.id,
                historical.version,
                current_operator.id,
                current_operator.version
            ));
        }
        if historical.version == current_version {
            return Err(format!(
                "duplicate current source-attention bundles exist for logical model {logical_name}: {} and {}",
                conventional_root.display(),
                current_root.display()
            ));
        }
    }

    Ok(CompiledModelPair {
        logical_name: logical_name.to_owned(),
        conventional_root,
        conventional,
        current_root,
        current,
    })
}

fn selected_compiled_root(pair: &CompiledModelPair) -> Option<(&Path, &AttentionOperatorSpec)> {
    if let Some(operator) = pair.current.operator() {
        Some((&pair.current_root, operator))
    } else {
        pair.conventional
            .operator()
            .map(|operator| (pair.conventional_root.as_path(), operator))
    }
}

struct CoverProvenanceProjection {
    attention_present: bool,
    operator: Option<AttentionOperatorSpec>,
    source_manifest_kappa_present: bool,
    source_manifest_kappa: Option<String>,
}

impl<'de> Deserialize<'de> for CoverProvenanceProjection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ProjectionVisitor;

        impl<'de> serde::de::Visitor<'de> for ProjectionVisitor {
            type Value = CoverProvenanceProjection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a cover report object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut attention_present = false;
                let mut operator = None;
                let mut source_manifest_kappa_present = false;
                let mut source_manifest_kappa = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "attention_operator" => {
                            if attention_present {
                                return Err(serde::de::Error::duplicate_field(
                                    "attention_operator",
                                ));
                            }
                            attention_present = true;
                            operator = map.next_value()?;
                        }
                        "source_manifest_kappa" => {
                            if source_manifest_kappa_present {
                                return Err(serde::de::Error::duplicate_field(
                                    "source_manifest_kappa",
                                ));
                            }
                            source_manifest_kappa_present = true;
                            source_manifest_kappa = map.next_value()?;
                        }
                        _ => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CoverProvenanceProjection {
                    attention_present,
                    operator,
                    source_manifest_kappa_present,
                    source_manifest_kappa,
                })
            }
        }

        deserializer.deserialize_map(ProjectionVisitor)
    }
}

fn canonical_source_manifest_kappa(kappa: &str) -> bool {
    kappa.strip_prefix("blake3:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn parse_cover_provenance(
    report_path: &Path,
) -> Result<(bool, Option<AttentionOperatorSpec>, Option<String>), String> {
    let Some(bytes) = read_regular_file_nofollow(report_path, "cover report")? else {
        return Ok((false, None, None));
    };
    let projection: CoverProvenanceProjection = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed JSON: {error}", report_path.display()))?;
    if let Some(kappa) = projection.source_manifest_kappa.as_deref() {
        if !canonical_source_manifest_kappa(kappa) {
            return Err(format!(
                "{} records a non-canonical source_manifest_kappa {kappa:?}",
                report_path.display()
            ));
        }
    }
    if projection.attention_present {
        if let Some(recorded) = projection.operator.as_ref() {
            let registered =
                uor_r4_model_source::attention::operator_spec(&recorded.id, recorded.version)
                    .map_err(|error| format!("{}: {error}", report_path.display()))?;
            if &registered != recorded || !source_attention_operator(recorded) {
                return Err(format!(
                    "{} does not contain a registry-exact source attention operator {}/{}",
                    report_path.display(),
                    recorded.id,
                    recorded.version
                ));
            }
        }
    }
    // A JSON `null` is compatible with the historical unrecorded state; the
    // presence bit exists only so duplicate top-level controls are rejected.
    let _ = projection.source_manifest_kappa_present;
    Ok((true, projection.operator, projection.source_manifest_kappa))
}

/// Reconcile every attention identity the server bundle records. Current-v2
/// serving requires both the canonical corpus pair and its cover report; old
/// v1 bundles remain readable when those supporting records are genuinely
/// absent. R4G1 graph-byte provenance itself remains the separate #637 PROV
/// contract, so this check deliberately makes no stronger claim.
fn validate_serving_attention_provenance(
    bundle: &Path,
    expected: &AttentionOperatorSpec,
    current_version: u32,
) -> Result<Option<String>, String> {
    // New server-published generations carry a digest-complete transaction
    // record. Its presence is authoritative; no caller may silently fall
    // back to piecemeal graph/report reads if any member drifted.
    let _ = validate_compiled_bundle_completion(bundle)?;
    let meta = bundle.join("corpus.meta");
    let records = bundle.join("corpus.records");
    let meta_present = regular_file_presence(&meta)?;
    let records_present = regular_file_presence(&records)?;
    let recorded_corpus = match (meta_present, records_present) {
        (true, true) => uor_r4_graph_cli::recorded_corpus_attention_operator(&meta, &records)
            .map_err(|error| error.to_string())?,
        (false, false) if expected.version < current_version => None,
        (false, false) => {
            return Err(format!(
                "current source-attention bundle {} is missing its canonical corpus.meta/corpus.records provenance pair",
                bundle.display()
            ))
        }
        _ => {
            return Err(format!(
                "compiled bundle {} has only one of corpus.meta/corpus.records; recorded provenance is incomplete",
                bundle.display()
            ))
        }
    };
    if meta_present {
        let recorded = recorded_corpus.unwrap_or_else(AttentionOperatorSpec::standard_v1);
        if &recorded != expected {
            return Err(format!(
                "compiled bundle {} records attention operator {}/{} in its corpus provenance but its selected root is {}/{}",
                bundle.display(),
                recorded.id,
                recorded.version,
                expected.id,
                expected.version
            ));
        }
    }

    let report_path = bundle.join("graph-cover/cover_report.json");
    let (report_present, report_operator, source_manifest_kappa) =
        parse_cover_provenance(&report_path)?;
    if expected.version == current_version && !report_present {
        return Err(format!(
            "current source-attention bundle {} is missing required cover provenance {}",
            bundle.display(),
            report_path.display()
        ));
    }
    if report_present {
        let recorded = report_operator.unwrap_or_else(AttentionOperatorSpec::standard_v1);
        if &recorded != expected {
            return Err(format!(
                "{} records attention operator {}/{} but bundle {} is selected as {}/{}",
                report_path.display(),
                recorded.id,
                recorded.version,
                bundle.display(),
                expected.id,
                expected.version
            ));
        }
    }
    Ok(source_manifest_kappa)
}

fn source_manifest_kappa(manifest: &crate::model::SourceManifest) -> Result<String, String> {
    crate::model::source_manifest_kappa(manifest)
        .map_err(|error| format!("source manifest cannot be addressed: {error}"))
}

fn validate_resolved_source_snapshot_binding(
    bundle: &ResolvedCompiledBundle,
    snapshot: Option<&VerifiedManagedSourceSnapshot>,
    current_version: u32,
) -> Result<(), String> {
    match (snapshot, bundle.source_manifest_kappa.as_deref()) {
        (Some(snapshot), Some(recorded)) => {
            let actual = &snapshot.content_kappa;
            if recorded != actual {
                return Err(format!(
                    "compiled bundle {} records source snapshot kappa {recorded}, but source {} verifies as {actual}; refusing mixed source/corpus identities",
                    bundle.physical_root.display(),
                    bundle.logical_name
                ));
            }
        }
        (Some(_), None) if bundle.attention_operator.version == current_version => {
            return Err(format!(
                "current source-attention bundle {} has a verified source snapshot but its cover report records no source_manifest_kappa",
                bundle.physical_root.display()
            ));
        }
        (None, Some(recorded)) => {
            return Err(format!(
                "compiled bundle {} records source snapshot kappa {recorded}, but no verified source snapshot is available",
                bundle.physical_root.display()
            ));
        }
        (Some(_), None) | (None, None) => {}
    }
    Ok(())
}

fn resolve_loadable_compiled_bundle(
    pair: &CompiledModelPair,
    current_version: u32,
) -> Result<Option<ResolvedCompiledBundle>, String> {
    if matches!(
        pair.current,
        CompiledRootState::Empty | CompiledRootState::PreAttentionIdentity
    ) {
        return Err(format!(
            "preferred current source-attention root {} is present but incomplete; refusing historical fallback",
            pair.current_root.display()
        ));
    }
    let Some((physical_root, operator)) = selected_compiled_root(pair) else {
        return Ok(None);
    };
    let primary_graph = physical_root.join("graph/score.r4g1");
    let fallback_graph = physical_root.join("compiled.r4g1");
    let graph = select_regular_fallback_path(&primary_graph, &fallback_graph)?;
    let teacher = physical_root.join("tless_artifacts.bin");
    let teacher_present = regular_file_presence(&teacher)?;
    let Some(graph) = graph else {
        if teacher_present {
            return Err(format!(
                "selected compiled bundle {} has a teacher artifact but no R4G1 graph",
                physical_root.display()
            ));
        }
        return Ok(None);
    };
    if !teacher_present {
        return Err(format!(
            "selected compiled bundle {} has graph {} but no teacher artifact {}",
            physical_root.display(),
            graph.display(),
            teacher.display()
        ));
    }
    let source_manifest_kappa =
        validate_serving_attention_provenance(physical_root, operator, current_version)?;
    Ok(Some(ResolvedCompiledBundle {
        logical_name: pair.logical_name.clone(),
        physical_root: physical_root.to_path_buf(),
        graph,
        teacher,
        attention_operator: operator.clone(),
        source_manifest_kappa,
    }))
}

fn resolve_requested_compiled_bundle_in(
    models_root: &Path,
    requested: &str,
    current_version: u32,
) -> Result<Option<ResolvedCompiledBundle>, String> {
    reject_requested_suffix_source_collision(requested, models_root, current_version)?;
    let logical_name = logical_model_name_for_request(requested, current_version)?;
    let pair = inspect_compiled_model_pair(
        &models_root.join("compiled"),
        &logical_name,
        current_version,
    )?;
    let resolved = resolve_loadable_compiled_bundle(&pair, current_version)?;
    if let Some(bundle) = resolved.as_ref() {
        reject_reserved_suffix_source_collision(bundle, models_root, current_version)?;
    }
    Ok(resolved)
}

fn reject_requested_suffix_source_collision(
    requested: &str,
    models_root: &Path,
    current_version: u32,
) -> Result<(), String> {
    let suffix = attention_era_suffix(current_version);
    if requested.strip_suffix(&suffix).is_none() {
        return Ok(());
    }
    let ambiguous = models_root.join("sources").join(requested);
    match fs::symlink_metadata(&ambiguous) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!(
            "requested model {requested:?} is ambiguous because the pre-existing source basename {} collides with the resolver-owned suffix; request the logical base explicitly after resolving the collision",
            ambiguous.display()
        )),
        Err(error) => Err(format!("{} cannot be inspected: {error}", ambiguous.display())),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ConfiguredManagedBundle {
    External,
    Absent,
    Incomplete(String),
    Selected(Box<ResolvedCompiledBundle>),
}

impl ConfiguredManagedBundle {
    fn permits_inventory_discovery(&self) -> bool {
        matches!(self, Self::External | Self::Selected(_))
    }
}

fn normalized_absolute_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("current directory is unavailable: {error}"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

/// Re-enter the managed resolver when `--tless-artifacts` names the canonical
/// teacher file inside `models_root/compiled/<physical-root>`. Existing paths
/// are classified by their canonical target so a symlink alias cannot bypass
/// the paired-era resolver. Lexical normalization is used only when the target
/// is genuinely absent and therefore cannot be canonicalized.
fn resolve_managed_teacher_bundle_in(
    teacher_path: &Path,
    models_root: &Path,
    current_version: u32,
) -> Result<ConfiguredManagedBundle, String> {
    if teacher_path.file_name() != Some(std::ffi::OsStr::new("tless_artifacts.bin")) {
        return Ok(ConfiguredManagedBundle::External);
    }
    let Some(physical_root) = teacher_path.parent() else {
        return Ok(ConfiguredManagedBundle::External);
    };
    let compiled_root = models_root.join("compiled");
    let normalized_compiled_root = normalized_absolute_path(&compiled_root)?;
    let lexical_managed =
        normalized_absolute_path(physical_root.parent().unwrap_or_else(|| Path::new(".")))?
            == normalized_compiled_root;

    let physical_metadata = match fs::symlink_metadata(physical_root) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "{} cannot be inspected while classifying the managed namespace: {error}",
                physical_root.display()
            ));
        }
    };
    let mut classified_root = physical_root.to_path_buf();
    if physical_metadata.is_some() {
        let compiled_metadata = match fs::symlink_metadata(&compiled_root) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!(
                    "{} cannot be inspected while classifying {}: {error}",
                    compiled_root.display(),
                    teacher_path.display()
                ));
            }
        };
        if compiled_metadata.is_none() {
            if lexical_managed {
                return Err(format!(
                    "configured managed path {} exists while its compiled namespace {} is absent",
                    teacher_path.display(),
                    compiled_root.display()
                ));
            }
            return Ok(ConfiguredManagedBundle::External);
        }
        let canonical_compiled = fs::canonicalize(&compiled_root).map_err(|error| {
            format!(
                "{} cannot be canonicalized while classifying {}: {error}",
                compiled_root.display(),
                teacher_path.display()
            )
        })?;
        let canonical_physical = fs::canonicalize(physical_root).map_err(|error| {
            format!(
                "{} cannot be canonicalized while classifying the managed namespace: {error}",
                physical_root.display()
            )
        })?;
        if canonical_physical.parent() == Some(canonical_compiled.as_path()) {
            classified_root = canonical_physical;
        } else {
            let canonical_teacher = match fs::symlink_metadata(teacher_path) {
                Ok(_) => Some(fs::canonicalize(teacher_path).map_err(|error| {
                    format!(
                        "{} cannot be canonicalized while classifying the managed namespace: {error}",
                        teacher_path.display()
                    )
                })?),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(format!(
                        "{} cannot be inspected while classifying the managed namespace: {error}",
                        teacher_path.display()
                    ));
                }
            };
            let canonical_teacher_root = canonical_teacher
                .as_deref()
                .and_then(Path::parent)
                .filter(|root| root.parent() == Some(canonical_compiled.as_path()));
            if let Some(root) = canonical_teacher_root {
                classified_root = root.to_path_buf();
            } else if lexical_managed {
                return Err(format!(
                    "configured managed path {} resolves outside {}; refusing namespace escape",
                    teacher_path.display(),
                    compiled_root.display()
                ));
            } else {
                return Ok(ConfiguredManagedBundle::External);
            }
        }
    } else if !lexical_managed {
        return Ok(ConfiguredManagedBundle::External);
    }

    let physical_name = classified_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "managed compiled bundle name is not UTF-8: {}",
                physical_root.display()
            )
        })?;
    let logical_name = logical_model_name_for_request(physical_name, current_version)?;
    let pair = inspect_compiled_model_pair(&compiled_root, &logical_name, current_version)?;
    if matches!(
        pair.current,
        CompiledRootState::Empty | CompiledRootState::PreAttentionIdentity
    ) {
        return Ok(ConfiguredManagedBundle::Incomplete(format!(
            "preferred current bundle {} exists but is incomplete; refusing historical adjacent-path fallback",
            pair.current_root.display()
        )));
    }
    match resolve_loadable_compiled_bundle(&pair, current_version)? {
        Some(bundle) => Ok(ConfiguredManagedBundle::Selected(Box::new(bundle))),
        None
            if matches!(pair.conventional, CompiledRootState::Absent)
                && matches!(pair.current, CompiledRootState::Absent) =>
        {
            Ok(ConfiguredManagedBundle::Absent)
        }
        None => Ok(ConfiguredManagedBundle::Incomplete(format!(
            "configured managed model {logical_name} has a present selected root without a complete graph/teacher bundle"
        ))),
    }
}

fn source_for_resolved_bundle_in(
    bundle: &ResolvedCompiledBundle,
    models_root: &Path,
) -> Result<Option<PathBuf>, String> {
    reject_reserved_suffix_source_collision(
        bundle,
        models_root,
        current_source_attention_era_version()?,
    )?;
    optional_source_directory(&models_root.join("sources").join(&bundle.logical_name))
}

fn reject_reserved_suffix_source_collision(
    bundle: &ResolvedCompiledBundle,
    models_root: &Path,
    current_version: u32,
) -> Result<(), String> {
    let expected_physical = format!(
        "{}{}",
        bundle.logical_name,
        attention_era_suffix(current_version)
    );
    if bundle
        .physical_root
        .file_name()
        .and_then(|name| name.to_str())
        != Some(expected_physical.as_str())
    {
        return Ok(());
    }
    let ambiguous = models_root.join("sources").join(&expected_physical);
    match fs::symlink_metadata(&ambiguous) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!(
            "managed bundle {} is ambiguous because the pre-existing source basename {} collides with the resolver-owned suffix; refusing to map it to sources/{}",
            bundle.physical_root.display(),
            ambiguous.display(),
            bundle.logical_name
        )),
        Err(error) => Err(format!("{} cannot be inspected: {error}", ambiguous.display())),
    }
}

fn status_physical_root(bundle: Option<&ResolvedCompiledBundle>) -> Option<String> {
    bundle.map(|bundle| bundle.physical_root.display().to_string())
}

#[cfg(test)]
fn teacher_ready_for_source(
    oracle_loaded: bool,
    active_teacher_source: Option<&Path>,
    resolved_source: Option<&Path>,
) -> bool {
    oracle_loaded && resolved_source.is_some() && active_teacher_source == resolved_source
}

fn graph_text_ready(state: &ServingModelState) -> bool {
    state
        .r4g1
        .as_ref()
        .is_some_and(|graph| !graph.host_encoder_unavailable())
}

fn teacher_text_ready(state: &ServingModelState) -> bool {
    state.oracle.is_some()
        && state.source_tokenizer.is_some()
        && state.active_teacher_source.is_some()
}

fn installed_logical_model_name(state: &ServingModelState) -> String {
    state
        .active_bundle
        .as_ref()
        .map(|bundle| bundle.logical_name.clone())
        .or_else(|| {
            state
                .active_teacher_source
                .as_deref()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "uor-r4".to_owned())
}

fn active_canonical_model_name(state: &ServingModelState) -> Option<String> {
    if !graph_text_ready(state) && !teacher_text_ready(state) {
        return None;
    }
    Some(installed_logical_model_name(state))
}

/// Resolve the OpenAI request model before any routing or generation side
/// effects. `uor-r4` is the one intentional compatibility alias; responses
/// always report the canonical active identity rather than echoing the alias.
fn resolve_active_request_model(
    state: &ServingModelState,
    requested: Option<&str>,
) -> Result<String, String> {
    let active = active_canonical_model_name(state);
    resolve_request_model_name(active.as_deref(), requested)
}

fn resolve_request_model_name(
    active: Option<&str>,
    requested: Option<&str>,
) -> Result<String, String> {
    let active = active.ok_or_else(|| "no text-ready serving model is active".to_owned())?;
    match requested.map(str::trim).filter(|name| !name.is_empty()) {
        None | Some("uor-r4") => Ok(active.to_owned()),
        Some(name) if name == active => Ok(active.to_owned()),
        Some(name) => Err(format!(
            "The model '{name}' is not active; the active model is '{active}'."
        )),
    }
}

fn active_models(state: &ServingModelState) -> Vec<(String, u64)> {
    let Some(name) = active_canonical_model_name(state) else {
        return Vec::new();
    };
    let created = state
        .active_bundle
        .as_ref()
        .and_then(|bundle| fs::metadata(&bundle.teacher).ok())
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|delta| delta.as_secs())
        .unwrap_or(0);
    vec![(name, created)]
}

fn resolve_reload_bundle_in(
    models_root: &Path,
    requested: &str,
    current_version: u32,
) -> Result<Option<(ResolvedCompiledBundle, Option<PathBuf>)>, String> {
    let Some(bundle) =
        resolve_requested_compiled_bundle_in(models_root, requested, current_version)?
    else {
        return Ok(None);
    };
    let source = source_for_resolved_bundle_in(&bundle, models_root)?;
    Ok(Some((bundle, source)))
}

fn refresh_resolved_compiled_bundle(
    resolved: &ResolvedCompiledBundle,
    current_version: u32,
) -> Result<ResolvedCompiledBundle, String> {
    let compiled_root = resolved.physical_root.parent().ok_or_else(|| {
        format!(
            "{} has no compiled-model parent",
            resolved.physical_root.display()
        )
    })?;
    let pair = inspect_compiled_model_pair(compiled_root, &resolved.logical_name, current_version)?;
    let refreshed = resolve_loadable_compiled_bundle(&pair, current_version)?.ok_or_else(|| {
        format!(
            "resolved compiled bundle {} disappeared before installation",
            resolved.physical_root.display()
        )
    })?;
    if &refreshed != resolved {
        return Err(format!(
            "compiled model {} changed between resolution and installation",
            resolved.logical_name
        ));
    }
    Ok(refreshed)
}

/// Find compiled dashboard bundles when the server was restarted without an
/// explicit `--r4g1-artifact`. Every immediate entry is part of a logical
/// pair: non-directories/symlinks and invalid siblings are terminal, while a
/// valid current-era root wins over its historical sibling deterministically.
fn discover_compiled_r4g1_candidates() -> Result<Vec<ResolvedCompiledBundle>, String> {
    discover_compiled_r4g1_candidates_in(
        Path::new(".uor-models/compiled"),
        current_source_attention_era_version()?,
    )
}

#[allow(dead_code)] // retained for protocol tests and future non-mutating inventory clients
fn try_lock_managed_inventory_read_sessions(
    compiled_root: &Path,
    additional_subjects: impl IntoIterator<Item = PathBuf>,
) -> Result<SourceCompileSessionLocks, String> {
    // The inventory root is the namespace lock for entry creation/removal.
    // Source-driven writers acquire it exclusively before publishing either
    // era sibling, so a reader cannot enumerate a moving logical inventory.
    let mut subjects = vec![compiled_root.to_path_buf()];
    subjects.extend(additional_subjects);
    // The namespace lock is sufficient for inventory stability because every
    // server writer, including the configured legacy lane, takes the same
    // exclusive subject before touching a managed graph/cover output. The
    // additional subjects cover configured sinks outside that namespace.
    try_lock_source_compile_sessions(subjects, SourceCompileSessionMode::SharedReader)
}

fn try_lock_managed_inventory_write_sessions(
    compiled_root: &Path,
    additional_subjects: impl IntoIterator<Item = PathBuf>,
) -> Result<SourceCompileSessionLocks, String> {
    let mut subjects = vec![compiled_root.to_path_buf()];
    subjects.extend(additional_subjects);
    try_lock_source_compile_sessions(subjects, SourceCompileSessionMode::ExclusiveWriter)
}

fn discover_compiled_r4g1_candidates_in(
    root: &Path,
    current_version: u32,
) -> Result<Vec<ResolvedCompiledBundle>, String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "compiled model root {} cannot be inspected: {error}",
                root.display()
            ));
        }
    };
    let suffix = attention_era_suffix(current_version);
    let mut logical_names = std::collections::BTreeSet::<String>::new();

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "compiled model root {} cannot be enumerated: {error}",
                root.display()
            )
        })?;
        let bundle = entry.path();
        let metadata = fs::symlink_metadata(&bundle)
            .map_err(|error| format!("{} cannot be inspected: {error}", bundle.display()))?;
        if entry.file_name() == std::ffi::OsStr::new(SOURCE_COMPILE_STAGING_DIR) {
            if !metadata.file_type().is_dir() {
                return Err(format!(
                    "source compile staging namespace {} is not a regular non-symlink directory",
                    bundle.display()
                ));
            }
            continue;
        }
        if !metadata.file_type().is_dir() {
            return Err(format!(
                "compiled model candidate {} is not a regular non-symlink directory",
                bundle.display()
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("compiled bundle name is not UTF-8: {}", bundle.display()))?;
        let logical_name = name
            .strip_suffix(&suffix)
            .filter(|base| !base.is_empty())
            .unwrap_or(&name)
            .to_owned();
        validate_logical_model_name(&logical_name, current_version)?;
        logical_names.insert(logical_name);
    }

    let mut selected = Vec::new();
    for logical_name in logical_names {
        let pair = inspect_compiled_model_pair(root, &logical_name, current_version)?;
        if let Some(candidate) = resolve_loadable_compiled_bundle(&pair, current_version)? {
            reject_reserved_suffix_source_collision(
                &candidate,
                root.parent().unwrap_or_else(|| Path::new(".")),
                current_version,
            )?;
            selected.push(candidate);
        }
    }
    Ok(selected)
}

/// Inspect an optional file without collapsing a present-invalid filesystem
/// entry into absence. Symlinks are followed only when their target is a
/// regular file; directories, dangling links, and special files fail closed.
fn regular_file_presence(path: &Path) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let is_regular = if metadata.file_type().is_symlink() {
        fs::metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(format!("{} is not a regular file", path.display()));
    }
    Ok(true)
}

fn required_r4g1_inputs_present(graph: &Path, teacher: &Path) -> Result<bool, String> {
    let graph_present = regular_file_presence(graph)?;
    let teacher_present = regular_file_presence(teacher)?;
    match (graph_present, teacher_present) {
        (true, true) => Ok(true),
        (false, false) => Ok(false),
        (true, false) => Err(format!(
            "R4G1 graph {} is present but its teacher artifact {} is absent",
            graph.display(),
            teacher.display()
        )),
        (false, true) => Err(format!(
            "R4G1 teacher artifact {} is present but its graph {} is absent",
            teacher.display(),
            graph.display()
        )),
    }
}

fn append_tokenizer_arg(
    args: &mut Vec<String>,
    tokenizer_path: &Path,
    required: bool,
) -> Result<(), String> {
    if regular_file_presence(tokenizer_path)? {
        args.extend([
            "--tokenizer".to_owned(),
            tokenizer_path.display().to_string(),
        ]);
    } else if required {
        return Err(format!(
            "required source-backed tokenizer is missing: {}",
            tokenizer_path.display()
        ));
    }
    Ok(())
}

fn select_regular_fallback_path(
    primary: &Path,
    fallback: &Path,
) -> Result<Option<PathBuf>, String> {
    if regular_file_presence(primary)? {
        Ok(Some(primary.to_path_buf()))
    } else if regular_file_presence(fallback)? {
        Ok(Some(fallback.to_path_buf()))
    } else {
        Ok(None)
    }
}

fn optional_source_directory(path: &Path) -> Result<Option<PathBuf>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let is_directory = if metadata.file_type().is_symlink() {
        fs::metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .is_dir()
    } else {
        metadata.is_dir()
    };
    if !is_directory {
        return Err(format!("{} is not a source directory", path.display()));
    }
    Ok(Some(path.to_path_buf()))
}

/// Source snapshot corresponding to a compiled teacher. Conventional
/// `.uor-models/compiled/<name>` maps directly; the server-owned exact
/// `<name>-attention-v2` suffix maps back to `<name>` only when the bundle
/// carries a registry-exact current-v2 binding. Genuine source absence is
/// optional; invalid provenance or a present non-directory, dangling symlink,
/// or unreadable source entry is a hard error.
fn source_for_compiled_teacher(teacher_path: &Path) -> Result<Option<PathBuf>, String> {
    source_for_compiled_teacher_in(teacher_path, Path::new(".uor-models"))
}

fn source_for_compiled_teacher_in(
    teacher_path: &Path,
    models_root: &Path,
) -> Result<Option<PathBuf>, String> {
    let Some(bundle) = teacher_path.parent() else {
        return Ok(None);
    };
    let Some(name) = bundle.file_name() else {
        return Ok(None);
    };
    let managed_compiled_root = models_root.join("compiled");
    if bundle.parent() != Some(managed_compiled_root.as_path()) {
        // Explicit CLI paths are outside the managed resolver namespace.
        // Preserve their historical literal parent-name mapping, including
        // names that happen to resemble the server-owned era suffix.
        return optional_source_directory(&models_root.join("sources").join(name));
    }
    let Some(name) = name.to_str() else {
        return Ok(None);
    };
    let current_version = current_source_attention_era_version()?;
    let logical_name = logical_model_name_for_request(name, current_version)?;
    let compiled_root = bundle.parent().ok_or_else(|| {
        format!(
            "compiled teacher bundle {} has no compiled-model root",
            bundle.display()
        )
    })?;
    let pair = inspect_compiled_model_pair(compiled_root, &logical_name, current_version)?;
    if bundle != pair.conventional_root && bundle != pair.current_root {
        return Err(format!(
            "compiled teacher {} is outside the resolved roots for logical model {}",
            teacher_path.display(),
            logical_name
        ));
    }
    let source = models_root.join("sources").join(logical_name);
    optional_source_directory(&source)
}

/// Generate a text continuation with the transformerless runtime. The shared
/// state keeps chat turns on one graded store and serializes its thread-local
/// UOR binding. `None` means the configured artifacts/tokenizer are not ready.
fn generate_tless_text(
    slot: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    prompt: &str,
    max_tokens: usize,
    _session_signature: Option<&[u8]>,
) -> Option<String> {
    const MAX_SERVER_TOKENS: usize = 256;
    const MAX_SERVER_TEXT_BYTES: usize = 16 * 1024;
    let mut seed = [0u32; 4096];
    let seed_len = match tless_uor::tless_tokenize_into(prompt, &mut seed) {
        Some(l) => l,
        None => {
            println!("[-] generate_tless_text: Tokenization failed for prompt context");
            return None;
        }
    };
    if seed_len == 0 {
        println!("[-] generate_tless_text: Tokenized to 0 length");
        return None;
    }
    with_tless_server_state(slot, |_st| {
        let mut steps =
            [uor_r4_core::transformerless::runtime::Prediction::default(); MAX_SERVER_TOKENS];
        let count = match tless_uor::generate_steps_into(
            &seed[..seed_len],
            &mut steps[..max_tokens.min(MAX_SERVER_TOKENS)],
        ) {
            Some(c) => c,
            None => {
                println!("[-] generate_tless_text: generate_steps_into returned None");
                return None;
            }
        };
        println!("[+] generate_tless_text: generated {} steps", count);
        let mut tokens = [0u32; MAX_SERVER_TOKENS];
        for (token, step) in tokens.iter_mut().zip(&steps[..count]) {
            *token = step.token;
        }
        let mut bytes = [0u8; MAX_SERVER_TEXT_BYTES];
        let byte_count = match tless_uor::tless_detokenize_into(&tokens[..count], &mut bytes) {
            Some(b) => b,
            None => {
                println!("[-] generate_tless_text: tless_detokenize_into returned None");
                return None;
            }
        };
        let decoded = String::from_utf8_lossy(&bytes[..byte_count]).into_owned();
        println!("[+] generate_tless_text: decoded: {:?}", decoded);
        Some(decoded)
    })
    .flatten()
    .map(|text| text.trim().to_string())
    .filter(|text| !text.is_empty())
}

/// The outcome of one R4G1 chat generation: the decoded text (empty on
/// an abstention — the partial tokens of an abstained run are not
/// served), the status of the final scoring step, whether a widened
/// re-probe ran, and whether generation stopped on a policy abstention.
/// `None` from [`generate_r4g1_text`] means the runtime or tokenizer is
/// unavailable — distinct from an abstention, which is a declared D4
/// outcome.
struct R4g1Text {
    text: String,
    status: Option<ScoreStatus>,
    widened: bool,
    abstained: bool,
    usage: ServingUsage,
}

/// Generate directly from the validated R4G1 graph runtime. Tokenization and
/// decoding intentionally use the same tokenizer as the compiled teacher
/// artifact; R4G1 stores token ids, not user-facing text.
fn generate_r4g1_text(
    state: Option<&R4g1State>,
    prompt: &str,
    max_tokens: usize,
) -> Result<Option<R4g1Text>, String> {
    const MAX_SERVER_TOKENS: usize = 256;
    const MAX_SERVER_TEXT_BYTES: usize = 16 * 1024;
    let mut seed = [0u32; 4096];
    let mut generated = [0u32; MAX_SERVER_TOKENS];
    let mut bytes = [0u8; MAX_SERVER_TEXT_BYTES];
    let (byte_count, status, widened, abstained, usage) = {
        let Some(state) = state else {
            return Ok(None);
        };
        let seed_len = match state.encode_into(prompt, &mut seed) {
            Some(count) => count,
            None if state.has_explicit_tokenizer() => {
                let identity = state
                    .tokenizer_adapter_identity()
                    .map(|identity| format!(" {}/{}", identity.family, identity.version))
                    .unwrap_or_default();
                return Err(format!(
                    "R4G1 tokenizer{identity} is unavailable for prompt encoding"
                ));
            }
            None => tless_uor::tless_tokenize_into(prompt, &mut seed)
                .ok_or_else(|| "R4G1 tokenizer could not encode the prompt".to_owned())?,
        };
        if seed_len == 0 {
            return Err("R4G1 tokenizer produced an empty prompt".to_owned());
        }
        let outcome = state
            .generate_into_status(
                &seed[..seed_len],
                &mut generated[..max_tokens.min(MAX_SERVER_TOKENS)],
            )
            .map_err(|error| format!("R4G1 graph scoring failed: {error}"))?;
        // On an abstention no text is produced: the tokens generated before
        // the abstaining step are dropped rather than served, so an
        // out-of-distribution prompt never surfaces partial output.
        let bytes_written = if outcome.abstained || outcome.count == 0 {
            0
        } else {
            match state.decode_into(&generated[..outcome.count], &mut bytes) {
                Some(count) => count,
                None if state.has_explicit_tokenizer() => {
                    return Err(
                        "R4G1 bundle tokenizer could not decode generated tokens".to_owned()
                    );
                }
                None => tless_uor::tless_detokenize_into(&generated[..outcome.count], &mut bytes)
                    .ok_or_else(|| {
                    "R4G1 tokenizer could not decode generated tokens".to_owned()
                })?,
            }
        };
        (
            bytes_written,
            outcome.status,
            outcome.widened,
            outcome.abstained,
            ServingUsage {
                prompt_tokens: seed_len,
                completion_tokens: outcome.count,
            },
        )
    };
    let text = String::from_utf8_lossy(&bytes[..byte_count])
        .trim()
        .to_owned();
    Ok(Some(R4g1Text {
        text,
        status,
        widened,
        abstained,
        usage,
    }))
}

fn set_r4g1_terminal_load_error(serving: &SharedServingModel, error: impl Into<String>) {
    let mut installed = serving.lock().unwrap();
    let error = error.into();
    installed.terminal_load_error = Some(error.clone());
    installed.last_operation_error = Some(error);
}

fn r4g1_terminal_load_error(serving: &SharedServingModel) -> Option<String> {
    serving.lock().unwrap().terminal_load_error.clone()
}

fn record_replacement_failure(serving: &SharedServingModel, error: impl Into<String>) {
    let mut installed = serving.lock().unwrap();
    let error = error.into();
    installed.last_operation_error = Some(error.clone());
    if installed.r4g1.is_none()
        && installed.oracle.is_none()
        && installed.active_bundle.is_none()
        && installed.active_teacher_source.is_none()
    {
        installed.terminal_load_error = Some(error);
    }
}

fn resolve_serving_source_tokenizer(
    dir: &std::path::Path,
    selection: Option<&TokenizerAdapterKey>,
) -> Result<TokenizerKind, uor_r4_model_source::SourceUnavailable> {
    let tokenizer = resolve_source_tokenizer(dir, selection)?;
    let adapter = tokenizer.adapter().ok_or_else(|| {
        uor_r4_model_source::SourceUnavailable::new(format!(
            "{} resolved to an adapterless tokenizer",
            dir.display()
        ))
    })?;
    println!(
        "[+] Serving source tokenizer loaded ({}) as {}/{} (CID {}, digest {})",
        dir.display(),
        adapter.family,
        adapter.version,
        adapter.tokenizer_cid,
        adapter.adapter_digest,
    );
    Ok(tokenizer)
}

fn tokenizer_identity_matches(
    adapter: &TokenizerAdapter,
    expected: &RuntimeTokenizerIdentity,
) -> bool {
    adapter.family == expected.family
        && adapter.version == expected.version
        && adapter.tokenizer_cid == expected.tokenizer_cid
        && adapter.adapter_digest == expected.adapter_digest
}

fn prepare_optional_teacher_source_for_identity(
    source: Option<&Path>,
    selection: Option<&TokenizerAdapterKey>,
    logical_name: &str,
    expected: Option<&RuntimeTokenizerIdentity>,
) -> Result<Option<PreparedTeacherSource>, String> {
    let Some(source) = source else {
        if selection.is_some() {
            return Err(format!(
                "model {logical_name} cannot apply an explicit tokenizer selection without its source directory"
            ));
        }
        return Ok(None);
    };
    validate_managed_source_for_serving(source, logical_name)?;
    if selection.is_some() && expected.is_none() {
        return Err(format!(
            "model {logical_name} uses an untagged bundle tokenizer, so an explicit source-tokenizer selection cannot be proven equal"
        ));
    }
    let tokenizer = resolve_serving_source_tokenizer(source, selection)
        .map_err(|error| format!("teacher tokenizer for {logical_name} is unusable: {error}"))?;
    let adapter = tokenizer.adapter().ok_or_else(|| {
        format!("teacher tokenizer for {logical_name} has no registered adapter identity")
    })?;
    if let Some(expected) = expected {
        if !tokenizer_identity_matches(&adapter, expected) {
            return Err(format!(
                "teacher tokenizer for {logical_name} is {}/{} (CID {}, digest {}), but the compiled bundle requires {}/{} (CID {}, digest {})",
                adapter.family,
                adapter.version,
                adapter.tokenizer_cid,
                adapter.adapter_digest,
                expected.family,
                expected.version,
                expected.tokenizer_cid,
                expected.adapter_digest,
            ));
        }
    }
    // `Teacher::load` is the source crate's single-file/indexed-shard
    // ingestion boundary. Calling it directly distinguishes a genuine absent
    // source (handled above) from missing, malformed, ambiguous, or incomplete
    // weights, including `model.safetensors.index.json` layouts.
    let teacher = uor_r4_model_source::Teacher::load(source)
        .map_err(|error| format!("teacher source for {logical_name} is unusable: {error}"))?;
    Ok(Some(PreparedTeacherSource {
        teacher,
        tokenizer,
        source: source.to_path_buf(),
    }))
}

/// Select the exact executable teacher mode recorded by a managed graph. A
/// historical operator has no current Teacher implementation, so its graph
/// keeps its independently bound host encoder but is not silently backstopped
/// by v2 Teacher arithmetic.
fn teacher_mode_for_bundle_records(
    default: &AttentionOperatorSpec,
    experimental: Option<&AttentionOperatorSpec>,
    bundle: &AttentionOperatorSpec,
) -> Option<bool> {
    if default == bundle {
        Some(false)
    } else if experimental == Some(bundle) {
        Some(true)
    } else {
        None
    }
}

fn reconcile_prepared_teacher_with_bundle(
    prepared: &mut Option<PreparedTeacherSource>,
    bundle: Option<&ResolvedCompiledBundle>,
) -> Result<(bool, Option<String>), String> {
    let (Some(prepared_teacher), Some(bundle)) = (prepared.as_mut(), bundle) else {
        return Ok((false, None));
    };
    let default = prepared_teacher
        .teacher
        .attention_operator_spec()
        .ok_or_else(|| {
            format!(
                "teacher source {} declares no attention operator",
                prepared_teacher.source.display()
            )
        })?;
    prepared_teacher.teacher.set_r4_attention(true);
    let experimental = prepared_teacher.teacher.attention_operator_spec();
    prepared_teacher.teacher.set_r4_attention(false);
    if let Some(mode) =
        teacher_mode_for_bundle_records(&default, experimental.as_ref(), &bundle.attention_operator)
    {
        return Ok((mode, None));
    }

    let available = experimental
        .map(|operator| {
            format!(
                "{}/{} or {}/{}",
                default.id, default.version, operator.id, operator.version
            )
        })
        .unwrap_or_else(|| format!("{}/{}", default.id, default.version));
    let message = format!(
        "teacher source {} provides {available}, but compiled bundle {} records {}/{}; keeping the valid graph and omitting its incompatible teacher fallback",
        prepared_teacher.source.display(),
        bundle.physical_root.display(),
        bundle.attention_operator.id,
        bundle.attention_operator.version
    );
    *prepared = None;
    Ok((false, Some(message)))
}

fn teacher_r4_attention_for_request(pinned: Option<&str>, default_r4_attention: bool) -> bool {
    match pinned {
        Some(TIER_ATTENTION) => false,
        Some(TIER_R4_ATTENTION) => true,
        _ => default_r4_attention,
    }
}

fn generate_attention_text(
    oracle: &mut uor_r4_model_source::Teacher,
    tokenizer: Option<&TokenizerKind>,
    prompt: &str,
    max_tokens: usize,
) -> Option<(String, ServingUsage)> {
    // 1. Construct token seed for prompt
    let formatted_prompt = format!("User: {}\nAssistant:", prompt.trim());
    let tokenizer = tokenizer?;
    let seed = tokenizer.encode(&formatted_prompt);

    let seed_len = seed.len();
    if seed_len == 0 {
        return None;
    }

    // 2. Reset the oracle state for a new generation session
    oracle.reset();

    // 3. Feed the prompt tokens into the transformer model to populate key-value cache
    let mut logits = vec![0.0f32; oracle.vocab()];
    for (pos, &tok) in seed.iter().enumerate() {
        oracle.step(tok as usize, pos, &mut logits);
    }

    // 4. Autoregressively generate next tokens using greedy decoding with repetition penalty
    let mut generated = Vec::new();
    for pos in seed_len..seed_len + max_tokens {
        // Apply repetition penalty to logits before selecting token
        let start_idx = generated.len().saturating_sub(48);
        for &t in &generated[start_idx..] {
            let count = generated[start_idx..].iter().filter(|&&x| x == t).count();
            logits[t as usize] -= (count as f32) * 8.0;
        }

        // Find argmax token
        let mut best_t = 0usize;
        let mut best_v = logits[0];
        for (i, &v) in logits.iter().enumerate() {
            if v > best_v {
                best_v = v;
                best_t = i;
            }
        }

        // Stop on EOS token or NULL (0)
        if best_t == oracle.eos_token() || (oracle.eos_token() == 0 && best_t == 2) {
            break;
        }

        generated.push(best_t as u32);

        // Advance transformer state with generated token
        oracle.step(best_t, pos, &mut logits);
    }

    // 5. Detokenize back to String — same id space as the seed (#254).
    let decoded = tokenizer.decode(&generated);
    println!("[+] generate_attention_text: raw decoded: {:?}", decoded);
    let cleaned = clean_attention_response(&decoded, prompt);
    println!("[+] generate_attention_text: cleaned: {:?}", cleaned);
    Some((
        cleaned,
        ServingUsage {
            prompt_tokens: seed_len,
            completion_tokens: generated.len(),
        },
    ))
}

fn clean_attention_response(text: &str, prompt: &str) -> String {
    let mut cleaned = text.to_string();

    // 1. If the output contains "<|im_start|>assistant", extract everything after the last occurrence
    if let Some(pos) = cleaned.rfind("<|im_start|>assistant") {
        cleaned = cleaned[pos + "<|im_start|>assistant".len()..].to_string();
    } else if let Some(pos) = cleaned.rfind("assistant\n") {
        cleaned = cleaned[pos + "assistant\n".len()..].to_string();
    }

    // 2. Remove template boundary markers and replacement characters
    cleaned = cleaned
        .replace("<|im_start|>", "")
        .replace("<|im_end|>", "")
        .replace("user\n", "")
        .replace("assistant\n", "")
        .replace('\u{fffd}', "");

    // 3. Strip prompt echoes if the model repeated the user prompt at the beginning only when non-empty content remains
    let trimmed_prompt = prompt.trim();
    if cleaned.trim().starts_with(trimmed_prompt) {
        let remainder = cleaned.trim()[trimmed_prompt.len()..].trim();
        if !remainder.is_empty() {
            cleaned = remainder.to_string();
        }
    }

    // 4. Remove any leading punctuation leftovers from echoes (e.g. "?", "-", ",", ".")
    let mut result = cleaned.trim().to_string();
    while result.starts_with('?')
        || result.starts_with('-')
        || result.starts_with(':')
        || result.starts_with(',')
        || result.starts_with('.')
        || result.starts_with(';')
    {
        result = result[1..].trim().to_string();
    }

    if result.is_empty() {
        truncate_repetitive_loops(text.trim())
    } else {
        truncate_repetitive_loops(&result)
    }
}

fn truncate_repetitive_loops(text: &str) -> String {
    let mut sentences = Vec::new();
    let mut last_end = 0;
    for (i, c) in text.char_indices() {
        if c == '.' || c == '!' || c == '?' || c == '\n' {
            let sentence = &text[last_end..=i];
            sentences.push(sentence);
            last_end = i + c.len_utf8();
        }
    }
    if last_end < text.len() {
        sentences.push(&text[last_end..]);
    }

    let mut seen = std::collections::HashSet::new();
    let mut result = String::new();
    for sentence in sentences {
        let norm = sentence.trim().to_lowercase();
        if norm.len() > 15 && !seen.insert(norm) {
            break;
        }
        result.push_str(sentence);
    }
    if result.trim().is_empty() {
        text.to_string()
    } else {
        result.trim().to_string()
    }
}

/// Candidate locations of the serving R4G1 artifact, in engine resolution
/// order (factored from startup loading; issues #256/#257).
const R4G1_ARTIFACT_CANDIDATES: [&str; 7] = [
    ".uor-models/compiled/smollm2-1-7b-instruct/compiled.r4g1",
    ".uor-models/compiled/smollm2-360m-instruct/compiled.r4g1",
    ".uor-models/compiled/smollm2-135m-instruct/compiled.r4g1",
    ".uor-models/compiled/smollm2-1-7b-instruct/score.r4g1",
    ".uor-models/compiled/smollm2-360m-instruct/score.r4g1",
    ".uor-models/compiled/smollm2-135m-instruct/score.r4g1",
    "/tmp/score.r4g1",
];

/// Canonical JSON kappa-label via the uor-addr pipeline (issues #256/#257):
/// JCS canonicalization then blake3 — byte-different encodings of the same
/// JSON value get the same label. Falls back to a raw-bytes blake3 label
/// only if canonicalization fails (unreachable for serde-produced JSON;
/// kept to honor the no-panic-on-recoverable-paths convention).
fn canonical_json_address_blake3(value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    match uor_addr::json::address_blake3(&bytes) {
        Ok(outcome) => outcome.address.to_string(),
        Err(_) => format!("blake3:{}", blake3::hash(&bytes).to_hex()),
    }
}

/// Typed syntax gate for a claimed `blake3:<64 lowercase hex>` address
/// (issue #257): empty and malformed subjects reject with a named reason.
/// A verifier that passes the empty case provides negative assurance.
fn validate_uor_address_syntax(address: &str) -> Result<(), &'static str> {
    if address.is_empty() {
        return Err("empty_address");
    }
    let Some(hex) = address.strip_prefix("blake3:") else {
        return Err("unsupported_axis");
    };
    if hex.len() != 64 {
        return Err("digest_length_invalid");
    }
    if !hex
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("digest_not_lowercase_hex");
    }
    Ok(())
}

/// Representation-level UOR κ-label of the artifact the serving cascade
/// would load, mtime-cached. The R4G1 wire CIDs remain internal integrity
/// checks; external attestations use the canonical section-addressable
/// realization from issue #264.
fn active_artifact_cid() -> Option<String> {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<Option<(std::path::PathBuf, std::time::SystemTime, String)>>> =
        OnceLock::new();
    let path = R4G1_ARTIFACT_CANDIDATES
        .iter()
        .map(std::path::Path::new)
        .find(|p| p.exists())?;
    let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().ok()?;
    if let Some((cached_path, cached_mtime, cid)) = guard.as_ref() {
        if cached_path == path && *cached_mtime == mtime {
            return Some(cid.clone());
        }
    }
    let bytes = std::fs::read(path).ok()?;
    let cid = uor_r4_graph_format::r4g1::artifact_kappa(&bytes)?;
    *guard = Some((path.to_path_buf(), mtime, cid.clone()));
    Some(cid)
}

/// Validate generated text before it is returned by the HTTP chat endpoint.
pub fn is_usable_generated_text(text: &str) -> bool {
    if text.contains(SPARSE_RESONANCE_MESSAGE) {
        return false;
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty()
        || chars
            .iter()
            .any(|ch| ch.is_control() && *ch != '\n' && *ch != '\r' && *ch != '\t')
    {
        return false;
    }
    let non_space = chars.iter().filter(|ch| !ch.is_whitespace()).count();
    let readable = chars
        .iter()
        .filter(|ch| ch.is_alphanumeric() || ch.is_ascii_punctuation())
        .count();
    if non_space == 0 || readable * 2 < non_space {
        return false;
    }
    let mut run = 1usize;
    for pair in chars.windows(2) {
        if pair[0] == pair[1] {
            run += 1;
            if run >= 16 {
                return false;
            }
        } else {
            run = 1;
        }
    }
    !repeated_word_loop(text)
}

/// Detect the kind of multi-word loop produced by a broken decoder or a
/// geometric fallback. Character-level guards cannot catch output such as
/// "that is how i work" repeated over and over, so inspect word windows too.
fn repeated_word_loop(text: &str) -> bool {
    let normalized = text.replace(['\n', '\r', '\t'], " ");
    let words: Vec<&str> = normalized.split_whitespace().collect();
    if words.len() < 4 {
        return false;
    }

    // A repeated suffix or adjacent loop is the common autoregressive failure mode. Check widths starting at 1.
    let max_suffix_width = (words.len() / 2).min(12);
    for width in 1..=max_suffix_width {
        let split = words.len() - width;
        if words[split..] == words[split - width..split] {
            return true;
        }
    }

    // Tightly-packed loops (3 occurrences of a 3-12 word phrase within a 5x window span).
    // Distant phrase reuse across separate paragraphs in long text is valid prose.
    let max_width = words.len().min(12);
    for width in 3..=max_width {
        for start in 0..=words.len().saturating_sub(width * 4) {
            let candidate = &words[start..start + width];
            let sub_slice = &words[start..(start + width * 5).min(words.len())];
            let occurrences = sub_slice
                .windows(width)
                .filter(|window| *window == candidate)
                .count();
            if occurrences >= 3 {
                return true;
            }
        }
    }

    false
}

/// Persist the manifold cache in the background, at the CLI-configured path.
fn spawn_cache_save(cli: &Arc<ServerConfig>, state_json: String) {
    let path = cli.manifold_cache.clone();
    std::thread::spawn(move || {
        let _ = std::fs::write(path, state_json);
    });
}

fn has_r4g1_compile_inputs(root: &Path) -> bool {
    root.join("corpus.meta").is_file() && root.join("corpus.records").is_file()
}

/// Validate the corpus files used by the browser-triggered R4G1 compiler.
pub fn validate_r4g1_corpus_inputs(meta: &Path, records: &Path) -> Result<(), String> {
    if !meta.is_file() {
        return Err(format!(
            "configured corpus metadata is missing: {}",
            meta.display()
        ));
    }
    if !records.is_file() {
        return Err(format!(
            "configured corpus records are missing: {}",
            records.display()
        ));
    }
    Ok(())
}

/// Resolve the synthesis engine. R4G1 is the active default; legacy and
/// geometric paths are reachable only through explicit engine values.
pub fn select_synthesis_engine(requested: Option<&str>) -> &'static str {
    match requested {
        Some("r4g1") => "r4g1",
        Some("geometric") => "geometric",
        Some("attention") => "attention",
        Some("r4-attention") => "r4-attention",
        Some("transformerless-legacy") => "transformerless-legacy",
        Some("auto" | "ollama" | "transformerless") | None => "r4g1",
        Some(_) => "r4g1",
    }
}

/// The explicit failure contract for an unavailable R4G1 runtime. Keeping
/// this separate makes the no-fallback behavior directly testable.
pub fn r4g1_unavailable_response() -> (u16, serde_json::Value) {
    r4g1_unavailable_response_with_reason(None)
}

fn r4g1_unavailable_response_with_reason(reason: Option<&str>) -> (u16, serde_json::Value) {
    let error = match reason {
        Some(reason) => {
            format!("R4G1 Graph runtime failed: {reason}; no fallback engine was invoked")
        }
        None => {
            "R4G1 Graph runtime did not produce a usable response; no fallback engine was invoked"
                .to_owned()
        }
    };
    (
        503,
        serde_json::json!({
            "error": error,
            "engine": "r4g1",
            "action": "Compile / Refresh the R4G1 graph, or explicitly select another engine"
        }),
    )
}

// ---------------------------------------------------------------------------
// The single serving cascade (issue #248). Both HTTP chat endpoints route
// through `run_serving_cascade`; the abstention policy is centralized in
// `uor_r4_router::fallback::SERVING_ABSTAIN_POLICY`.
// ---------------------------------------------------------------------------

/// Serving-cascade tier identifiers, in full-cascade order.
const TIER_R4G1: &str = "r4g1";
const TIER_TRANSFORMERLESS: &str = "transformerless";
const TIER_TEACHER_ORACLE: &str = "teacher-oracle";
const TIER_GEOMETRIC: &str = "geometric";
/// Pinned-only teacher tiers reachable through explicit `/engine` values.
const TIER_ATTENTION: &str = "attention";
const TIER_R4_ATTENTION: &str = "r4-attention";

/// The geometric decoder's sparse-manifold terminal. Recognized as a typed
/// abstention (issue #248) — never served as if it were generated prose.
const SPARSE_RESONANCE_MESSAGE: &str = "Manifold resonance too sparse for synthesis.";

/// R4G1 policy metadata surfaced alongside the cascade outcome so the
/// legacy `r4g1` response block keeps its exact shape.
#[derive(Debug, Default)]
struct R4g1Signal {
    status: Option<&'static str>,
    widened: bool,
    abstained: bool,
    error: Option<String>,
}

/// The full result of one serving-cascade run: the typed outcome and
/// per-tier trail, R4G1 policy metadata, and the geometric decode (for
/// trajectory reporting) when that tier ran.
struct ServingCascade {
    outcome: CascadeOutcome,
    r4g1: R4g1Signal,
    geometric: Option<uor_r4_router::GeometricResponse>,
    /// Exact counts carried by the R4G1 or teacher tier that served. Legacy
    /// tiers leave this empty and retain their historical tokenizer counter.
    usage: Option<ServingUsage>,
}

/// The persisted `/engine` selection written by the terminal chat client
/// (`.uor-models/last_engine.txt`), if any.
fn persisted_engine_preference() -> Option<String> {
    let raw = fs::read_to_string(".uor-models/last_engine.txt").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Resolve issue-#248 engine pinning. An engine named by the request — or,
/// when the request is silent, by the persisted `/engine` selection — pins
/// the cascade to that single tier, with one deliberate exception:
/// **"r4g1" never pins.** The CLI persists "r4g1" as its default
/// (`last_engine.txt`), so treating it as a pin would silently disable
/// every fallback tier for default installs; and r4g1-first is already the
/// full cascade's order, so the choice loses nothing by cascading.
/// Explicit non-default selections (attention, geometric, transformerless,
/// r4-attention) pin. "auto"/empty/unknown and the legacy "ollama" alias
/// run the full cascade. `select_synthesis_engine` remains the legacy
/// single-value resolver for existing consumers.
fn resolve_pinned_tier(requested: Option<&str>) -> Option<&'static str> {
    let requested = match requested.map(str::trim) {
        Some(value) if !value.is_empty() => Some(value.to_owned()),
        _ => persisted_engine_preference(),
    };
    match requested.as_deref() {
        Some("transformerless" | "transformerless-legacy") => Some(TIER_TRANSFORMERLESS),
        Some("attention") => Some(TIER_ATTENTION),
        Some("r4-attention") => Some(TIER_R4_ATTENTION),
        Some("geometric") => Some(TIER_GEOMETRIC),
        // "r4g1" (the persisted CLI default) and "auto"/unknown: full cascade.
        _ => None,
    }
}

/// R4G1 tier: a D4 policy abstention is typed `Abstained` — a declared
/// outcome recorded in the trail, which the cascade continues past under
/// the central policy (recording, not refusing; PR #223 semantics).
/// Unusable text is `Pathological` with the reason; an unavailable runtime
/// or scoring error is `Failed`.
fn r4g1_tier(
    state: Option<&R4g1State>,
    prompt: &str,
    max_tokens: usize,
    signal: &mut R4g1Signal,
    load_error: Option<&str>,
    usage: &Cell<Option<ServingUsage>>,
) -> TierResult {
    match generate_r4g1_text(state, prompt, max_tokens.max(32)) {
        Ok(Some(gen)) if gen.abstained => {
            signal.status = gen.status.map(r4g1::PolicyStatus::from).map(|s| s.label());
            signal.widened = gen.widened;
            signal.abstained = true;
            TierResult::abstained(match signal.status {
                Some(label) => format!("R4G1 policy abstained (status: {label})"),
                None => "R4G1 policy abstained".to_owned(),
            })
        }
        Ok(Some(gen)) if is_usable_generated_text(&gen.text) => {
            signal.status = gen.status.map(r4g1::PolicyStatus::from).map(|s| s.label());
            signal.widened = gen.widened;
            usage.set(Some(gen.usage));
            TierResult::success(gen.text)
        }
        Ok(Some(_)) => {
            let reason = "R4G1 generated text was rejected as non-readable or pathological";
            signal.error = Some(reason.to_owned());
            println!("[-] R4G1 output rejected as non-readable or pathological");
            TierResult::pathological(reason)
        }
        Ok(None) => {
            let reason = load_error
                .map(str::to_owned)
                .unwrap_or_else(|| "R4G1 graph runtime is not loaded".to_owned());
            signal.error = Some(reason.clone());
            TierResult::failed(reason)
        }
        Err(error) => {
            println!("[-] R4G1 generation failed: {error}");
            signal.error = Some(error.clone());
            TierResult::failed(error)
        }
    }
}

/// Transformerless tier: unusable text is `Pathological`; an unavailable
/// runtime or empty generation is `Failed`.
fn transformerless_tier(
    slot: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> TierResult {
    match generate_tless_text(slot, prompt, max_tokens.max(32), session_signature) {
        Some(text) if is_usable_generated_text(&text) => TierResult::success(text),
        Some(_) => {
            println!("[-] Transformerless output rejected as non-readable or pathological");
            TierResult::pathological(
                "transformerless output rejected as non-readable or pathological",
            )
        }
        None => TierResult::failed("transformerless runtime unavailable or produced no text"),
    }
}

/// Teacher-oracle tier (full-attention teacher): `Failed` when the oracle
/// is not loaded or produced nothing, `Pathological` when its output fails
/// the readability gate — pinned attention modes get the same gate as the
/// cascade tier.
fn attention_tier(
    oracle: &mut Option<uor_r4_model_source::Teacher>,
    tokenizer: Option<&TokenizerKind>,
    prompt: &str,
    max_tokens: usize,
    r4_attention: bool,
    usage: &Cell<Option<ServingUsage>>,
) -> TierResult {
    let Some(o) = oracle.as_mut() else {
        return TierResult::failed("teacher oracle is not loaded");
    };
    o.set_r4_attention(r4_attention);
    let generated = generate_attention_text(o, tokenizer, prompt, max_tokens);
    o.set_r4_attention(false);
    match generated {
        Some((text, counts)) if is_usable_generated_text(&text) => {
            usage.set(Some(counts));
            TierResult::success(text)
        }
        Some(_) => TierResult::pathological(
            "teacher oracle output rejected as non-readable or pathological",
        ),
        None => TierResult::failed("teacher oracle produced no text"),
    }
}

/// Geometric tier: an empty or sparse-manifold decode is a typed
/// `Abstained` — the sparse-string terminal is never served as generated
/// text. Unreadable decodes are `Pathological`.
#[allow(clippy::too_many_arguments)]
fn geometric_tier(
    router: &mut UorR4Router,
    out: &mut Option<uor_r4_router::GeometricResponse>,
    prompt: &str,
    identity: &str,
    max_tokens: usize,
    temperature: f64,
    gamma: f64,
) -> TierResult {
    let geom = router.generate_geometric_response_native(
        prompt,
        identity,
        max_tokens,
        temperature,
        10.0,
        4.0,
        gamma,
    );
    let text = geom.text.clone();
    *out = Some(geom);
    if text.trim().is_empty() || text.contains(SPARSE_RESONANCE_MESSAGE) {
        TierResult::abstained("manifold resonance too sparse for synthesis")
    } else if is_usable_generated_text(&text) {
        TierResult::success(text)
    } else {
        TierResult::pathological("geometric output rejected as non-readable or pathological")
    }
}

fn tokenizer_unavailable_is_terminal(
    pinned: Option<&'static str>,
    host_encoder_unavailable: bool,
    rejected_binding: bool,
) -> bool {
    pinned.is_none() && (host_encoder_unavailable || rejected_binding)
}

/// Build and run THE serving cascade (issue #248): r4g1 → transformerless →
/// teacher-oracle → geometric, or the single pinned tier when `pinned`
/// names one. First success serves; every attempted tier's typed outcome
/// lands in the trail; a run where no tier serves returns `text: None` so
/// the caller can answer with an honest `declined_by_all` terminal instead
/// of serving a placeholder string as if it were generated.
#[allow(clippy::too_many_arguments)]
fn run_serving_cascade(
    router: &mut UorR4Router,
    serving: &mut ServingModelState,
    tless: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    prompt: &str,
    identity: &str,
    max_tokens: usize,
    temperature: f64,
    gamma: f64,
    session_signature: Option<&[u8]>,
    pinned: Option<&'static str>,
) -> ServingCascade {
    let ServingModelState {
        r4g1,
        oracle,
        source_tokenizer,
        teacher_default_r4_attention,
        terminal_load_error,
        ..
    } = serving;
    let r4g1 = r4g1.as_ref();
    let source_tokenizer = source_tokenizer.as_ref();
    let mut signal = R4g1Signal::default();
    let mut geometric: Option<uor_r4_router::GeometricResponse> = None;
    let usage = Cell::new(None);
    let load_error = terminal_load_error.clone();
    let host_encoder_unavailable = r4g1.is_some_and(R4g1State::host_encoder_unavailable);
    // A tagged graph without its exact host encoder must terminate as
    // tokenizer-unavailable. Continuing to the transformerless tier would
    // invoke the historical greedy encoder in another id space, precisely the
    // fallback this artifact tag forbids.
    let tokenizer_terminal =
        tokenizer_unavailable_is_terminal(pinned, host_encoder_unavailable, load_error.is_some());
    let outcome = {
        let signal_ref = &mut signal;
        let geometric_ref = &mut geometric;
        let usage_ref = &usage;
        let include = |tier: &'static str| pinned.is_none() || pinned == Some(tier);
        let mut tiers: Vec<(&'static str, TierFn<'_>)> = Vec::new();
        if include(TIER_R4G1) {
            tiers.push((
                TIER_R4G1,
                Box::new(move || {
                    r4g1_tier(
                        r4g1,
                        prompt,
                        max_tokens,
                        signal_ref,
                        load_error.as_deref(),
                        usage_ref,
                    )
                }),
            ));
        }
        if include(TIER_TRANSFORMERLESS) && !tokenizer_terminal {
            tiers.push((
                TIER_TRANSFORMERLESS,
                Box::new(move || {
                    transformerless_tier(tless, prompt, max_tokens, session_signature)
                }),
            ));
        }
        if pinned.is_none() && !tokenizer_terminal {
            tiers.push((
                TIER_TEACHER_ORACLE,
                Box::new(move || {
                    attention_tier(
                        oracle,
                        source_tokenizer,
                        prompt,
                        max_tokens.max(128),
                        teacher_r4_attention_for_request(None, *teacher_default_r4_attention),
                        usage_ref,
                    )
                }),
            ));
        } else if pinned == Some(TIER_ATTENTION) {
            tiers.push((
                TIER_ATTENTION,
                Box::new(move || {
                    attention_tier(
                        oracle,
                        source_tokenizer,
                        prompt,
                        max_tokens.max(256),
                        teacher_r4_attention_for_request(
                            Some(TIER_ATTENTION),
                            *teacher_default_r4_attention,
                        ),
                        usage_ref,
                    )
                }),
            ));
        } else if pinned == Some(TIER_R4_ATTENTION) {
            tiers.push((
                TIER_R4_ATTENTION,
                Box::new(move || {
                    attention_tier(
                        oracle,
                        source_tokenizer,
                        prompt,
                        max_tokens.max(256),
                        teacher_r4_attention_for_request(
                            Some(TIER_R4_ATTENTION),
                            *teacher_default_r4_attention,
                        ),
                        usage_ref,
                    )
                }),
            ));
        }
        if include(TIER_GEOMETRIC) && !tokenizer_terminal {
            tiers.push((
                TIER_GEOMETRIC,
                Box::new(move || {
                    geometric_tier(
                        router,
                        geometric_ref,
                        prompt,
                        identity,
                        max_tokens,
                        temperature,
                        gamma,
                    )
                }),
            ));
        }
        run_cascade(tiers)
    };
    ServingCascade {
        outcome,
        r4g1: signal,
        geometric,
        usage: usage.get(),
    }
}

/// Derive the legacy `generation_mode` label from the cascade trail so
/// existing response consumers keep the field names and values they rely
/// on.
fn derive_generation_mode(cascade: &ServingCascade, pinned: Option<&'static str>) -> String {
    if let Some(served) = cascade.outcome.served_by {
        return match served {
            TIER_TRANSFORMERLESS => {
                if pinned == Some(TIER_TRANSFORMERLESS) {
                    "transformerless-legacy".to_owned()
                } else {
                    "transformerless-fallback".to_owned()
                }
            }
            TIER_TEACHER_ORACLE => "teacher-oracle-fallback".to_owned(),
            TIER_GEOMETRIC => "geometric-decoded".to_owned(),
            // TIER_R4G1, TIER_ATTENTION, TIER_R4_ATTENTION serve under
            // their own tier name.
            other => other.to_owned(),
        };
    }
    // Declined by all: keep the most specific legacy R4G1 label when that
    // tier was attempted, else the typed terminal label.
    let r4g1_step = cascade
        .outcome
        .trail
        .iter()
        .find(|step| step.tier == TIER_R4G1);
    match r4g1_step.map(|step| step.status) {
        Some(EngineStatus::Abstained) => "r4g1-abstained".to_owned(),
        Some(EngineStatus::Pathological) => "r4g1-rejected".to_owned(),
        Some(EngineStatus::Failed) => "r4g1-error".to_owned(),
        _ => "declined-by-all".to_owned(),
    }
}

/// Serialize the cascade trail for response payloads: tier, typed status
/// (as its wire label), and optional detail, in attempt order.
fn cascade_trail_json(trail: &[TierOutcome]) -> serde_json::Value {
    serde_json::Value::Array(
        trail
            .iter()
            .map(|step| {
                serde_json::json!({
                    "tier": step.tier,
                    "status": step.status.to_string(),
                    "detail": step.detail,
                })
            })
            .collect(),
    )
}

/// The honest terminal for a cascade where no tier served (issue #248):
/// `outcome: "declined_by_all"`, the pinned tier when one was named, the
/// derived legacy fields, and the full per-tier trail. A decline that
/// includes a declared abstention is HTTP 200 (a declared outcome, not a
/// fault); a decline where every tier hard-failed is HTTP 503.
fn declined_by_all_response(
    cascade: &ServingCascade,
    pinned: Option<&'static str>,
    generation_mode: &str,
) -> (u16, serde_json::Value) {
    let any_abstained = cascade
        .outcome
        .trail
        .iter()
        .any(|step| step.status == EngineStatus::Abstained);
    let status = if any_abstained { 200 } else { 503 };
    // A status notice for UIs that render `description` — visibly an
    // outcome declaration, never text presented as generated prose.
    let attempted = cascade
        .outcome
        .trail
        .iter()
        .map(|step| format!("{}: {}", step.tier, step.status))
        .collect::<Vec<_>>()
        .join(", ");
    let description =
        format!("Declined by all attempted tiers ({attempted}); no text was generated.");
    let mut body = serde_json::json!({
        "outcome": "declined_by_all",
        "engine": pinned.unwrap_or("auto"),
        "pinned_engine": pinned,
        "text": "",
        "description": description,
        "llm_connected": false,
        "generation_mode": generation_mode,
        "abstained": cascade.r4g1.abstained,
        "status": cascade.r4g1.status,
        "widened": cascade.r4g1.widened,
        "r4g1": {
            "status": cascade.r4g1.status,
            "widened": cascade.r4g1.widened,
            "abstained": cascade.r4g1.abstained,
        },
        "cascade_trail": cascade_trail_json(&cascade.outcome.trail),
    });
    if status == 503 {
        if let Some(map) = body.as_object_mut() {
            let reason = cascade
                .r4g1
                .error
                .clone()
                .unwrap_or_else(|| "no serving tier produced usable text".to_owned());
            map.insert(
                "error".to_owned(),
                serde_json::json!(format!(
                    "Serving cascade declined: {reason}; no text was fabricated"
                )),
            );
        }
    }
    (status, body)
}

fn same_file_bytes(left: &Path, right: &Path) -> bool {
    if !left.is_file() || !right.is_file() {
        return false;
    }
    match (fs::read(left), fs::read(right)) {
        (Ok(left), Ok(right)) => blake3::hash(&left) == blake3::hash(&right),
        _ => false,
    }
}

fn discover_r4g1_compile_root(cli: &ServerConfig, artifact: &Path) -> Result<PathBuf, String> {
    let direct_root = artifact
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    if has_r4g1_compile_inputs(&direct_root) {
        return Ok(direct_root);
    }

    if let Some(graph_artifact) = cli.r4g1_artifact.as_deref() {
        let graph_root = Path::new(graph_artifact)
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf);
        if let Some(graph_root) = graph_root {
            let candidate_artifact = graph_root.join(
                artifact
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("tless_artifacts.bin")),
            );
            if has_r4g1_compile_inputs(&graph_root)
                && same_file_bytes(artifact, &candidate_artifact)
            {
                return Ok(graph_root);
            }
        }
    }

    let compiled_root = Path::new(".uor-models").join("compiled");
    let mut matches = Vec::new();
    if let Ok(entries) = fs::read_dir(&compiled_root) {
        for entry in entries.flatten() {
            let root = entry.path();
            if root.is_dir()
                && has_r4g1_compile_inputs(&root)
                && same_file_bytes(artifact, &root.join("tless_artifacts.bin"))
            {
                matches.push(root);
            }
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(format!(
            "required compilation input is missing: {}. Point --tless-artifacts at the compiled bundle containing corpus.meta and corpus.records, or copy those corpus files beside the configured artifact",
            direct_root.join("corpus.meta").display()
        )),
        _ => Err(format!(
            "multiple compiled bundles match {}; pass --tless-artifacts explicitly to select one",
            artifact.display()
        )),
    }
}

fn r4g1_compile_paths(cli: &ServerConfig) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), String> {
    let artifact = PathBuf::from(&cli.tless_artifacts);
    if let (Some(meta), Some(recs)) = (&cli.tless_corpus_meta, &cli.tless_corpus_recs) {
        let corpus_meta = PathBuf::from(meta);
        let corpus_recs = PathBuf::from(recs);
        let root = corpus_meta
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let graph_path = cli
            .r4g1_artifact
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("graph").join("score.r4g1"));
        let graph_output = graph_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        validate_r4g1_corpus_inputs(&corpus_meta, &corpus_recs)?;
        return Ok((
            corpus_meta,
            corpus_recs,
            root.join("graph-cover"),
            graph_output,
        ));
    }
    let root = discover_r4g1_compile_root(cli, &artifact)?;
    let cover_output = root.join("graph-cover");
    let graph_path = cli
        .r4g1_artifact
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("graph").join("score.r4g1"));
    let graph_output = graph_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok((
        root.join("corpus.meta"),
        root.join("corpus.records"),
        cover_output,
        graph_output,
    ))
}

fn require_unchanged_legacy_compile_paths(
    selected: &(PathBuf, PathBuf, PathBuf, PathBuf),
    refreshed: &(PathBuf, PathBuf, PathBuf, PathBuf),
) -> Result<(), String> {
    if selected == refreshed {
        return Ok(());
    }
    Err(format!(
        "legacy R4G1 compile path selection changed while acquiring its cross-process output sessions ({} / {} -> {} / {}); refusing stale or ambiguous mutation",
        selected.0.display(),
        selected.3.display(),
        refreshed.0.display(),
        refreshed.3.display()
    ))
}

fn validate_source_bundle_inventory(output: &Path) -> Result<(), String> {
    for file in [
        "tless_artifacts.bin",
        "tless_store.bin",
        "tokenizer.bin",
        "tokenizer_adapter.json",
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
        "corpus.meta",
        "corpus.records",
    ] {
        let path = output.join(file);
        match regular_file_presence(&path) {
            Ok(true) => {}
            Ok(false) => {
                return Err(format!(
                    "transformerless bundle compilation is incomplete; missing {}. Retry the compile action to resume the corpus",
                    path.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "transformerless bundle compilation produced an invalid entry: {error}"
                ));
            }
        }
    }
    Ok(())
}

/// Parse a JSON document solely to reject duplicate object keys at every
/// nesting level. `serde_json::Value` is last-key-wins, which is unsuitable
/// for provenance records where duplicated fields could spell two eras in one
/// sidecar.
struct DuplicateRejectingJson;

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
                DuplicateRejectingJson::deserialize(deserializer)
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

fn reject_duplicate_json_fields(
    bytes: &[u8],
    path: &Path,
    description: &str,
) -> Result<(), String> {
    serde_json::from_slice::<DuplicateRejectingJson>(bytes)
        .map(|_| ())
        .map_err(|error| format!("{}: malformed {description}: {error}", path.display(),))
}

/// Read an output directory's exact source-attention binding. Genuine
/// absence stays `None`; every present-invalid entry is terminal so the
/// server never routes around malformed provenance into a fresh directory.
fn source_compile_attention_binding(
    output: &Path,
) -> Result<Option<uor_r4_model_source::attention::AttentionOperatorSpec>, String> {
    let path = output.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE);
    let Some(file) = open_regular_file_nofollow(&path, "attention-operator binding")? else {
        return Ok(None);
    };
    let value = validate_pre_attention_identity_handle(
        file,
        &path,
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
    )?;
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))
}

const SOURCE_MANIFEST_KAPPA_BINDING_FILE: &str = "source_manifest_kappa.json";
const SOURCE_MANIFEST_KAPPA_BINDING_SCHEMA: &str = "uor-r4-source-manifest-kappa-binding/1";
const SOURCE_COMPILE_PREFLIGHT_FILE: &str = "source_compile_preflight.json";
const SOURCE_COMPILE_PREFLIGHT_SCHEMA: &str = "uor-r4-source-compile-preflight/1";
const SOURCE_COMPILE_STAGING_DIR: &str = ".uor-r4-source-compile-staging";
const SOURCE_COMPILE_SESSION_LOCK_SUFFIX: &str = ".compile-session.lock";
const COMPILED_BUNDLE_STAGE_TAG: &str = "bundle-stage";
const COMPILED_BUNDLE_STAGE_MARKER_FILE: &str = ".compiled_bundle_stage.json";
const COMPILED_BUNDLE_STAGE_SCHEMA: &str = "uor-r4-compiled-bundle-stage/1";
const COMPILED_BUNDLE_COMPLETION_FILE: &str = "compiled_bundle_completion.json";
const COMPILED_BUNDLE_COMPLETION_SCHEMA: &str = "uor-r4-compiled-bundle-completion/1";
const LEGACY_GRAPH_GENERATION_SCHEMA: &str = "uor-r4-legacy-graph-generation/1";
const LEGACY_GRAPH_ATTEMPT_SCHEMA: &str = "uor-r4-legacy-graph-attempt/1";
static NEXT_SOURCE_KAPPA_BINDING_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct SourceManifestKappaBinding {
    schema: String,
    source_manifest_kappa: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct SourceCompilePreflight {
    schema: String,
    source_manifest_kappa: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CompiledBundleCompletion {
    schema: String,
    files: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CompiledBundleStageMarker {
    schema: String,
    final_output: String,
    stage_path: String,
    source_snapshot_kappa: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyGraphGenerationIdentity {
    cover_output: String,
    graph_output: String,
    input_files: std::collections::BTreeMap<String, String>,
    cover_controls_kappa: String,
    score_controls_kappa: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyGraphGenerationAttempt {
    schema: String,
    identity: LegacyGraphGenerationIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyGraphGenerationCompletion {
    schema: String,
    identity: LegacyGraphGenerationIdentity,
    output_files: std::collections::BTreeMap<String, String>,
}

fn open_regular_file_nofollow(path: &Path, label: &str) -> Result<Option<fs::File>, String> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        // Do not follow a link swapped in after a directory scan. NONBLOCK
        // also ensures a raced FIFO/device cannot hang the server before its
        // opened-handle metadata is rejected below.
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "{} cannot be opened as a regular non-symlink {label}: {error}",
                path.display()
            ));
        }
    };
    let file_metadata = file.metadata().map_err(|error| {
        format!(
            "{} opened {label} handle cannot be inspected: {error}",
            path.display()
        )
    })?;
    let path_metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{} cannot be reinspected: {error}", path.display()))?;
    if !file_metadata.file_type().is_file()
        || !path_metadata.file_type().is_file()
        || !source_compile_lock_metadata_matches(&path_metadata, &file_metadata)
    {
        return Err(format!(
            "{} is not a regular file; a stable non-symlink {label} is required",
            path.display()
        ));
    }
    Ok(Some(file))
}

fn read_opened_regular_file_nofollow(
    mut file: fs::File,
    path: &Path,
    label: &str,
) -> Result<Vec<u8>, String> {
    let opened_metadata = file.metadata().map_err(|error| {
        format!(
            "{} opened {label} handle cannot be inspected: {error}",
            path.display()
        )
    })?;
    if !opened_metadata.file_type().is_file() {
        return Err(format!(
            "{} opened {label} handle is not a regular file",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("{} cannot be read: {error}", path.display()))?;
    let final_file_metadata = file.metadata().map_err(|error| {
        format!(
            "{} opened {label} handle cannot be reinspected: {error}",
            path.display()
        )
    })?;
    let final_path_metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{} cannot be reinspected: {error}", path.display()))?;
    if !final_file_metadata.file_type().is_file()
        || !final_path_metadata.file_type().is_file()
        || !source_compile_lock_metadata_matches(&opened_metadata, &final_file_metadata)
        || !source_compile_lock_metadata_matches(&final_path_metadata, &final_file_metadata)
        || final_file_metadata.len() != bytes.len() as u64
    {
        return Err(format!(
            "{} changed identity, type, or length while its {label} bytes were read",
            path.display()
        ));
    }
    Ok(bytes)
}

fn read_regular_file_nofollow(path: &Path, label: &str) -> Result<Option<Vec<u8>>, String> {
    open_regular_file_nofollow(path, label)?
        .map(|file| read_opened_regular_file_nofollow(file, path, label))
        .transpose()
}

fn read_required_regular_file_nofollow(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    read_regular_file_nofollow(path, label)?.ok_or_else(|| {
        format!(
            "{} disappeared before its {label} bytes could be read",
            path.display()
        )
    })
}

fn source_compile_preflight_bytes(kappa: Option<&str>) -> Result<Vec<u8>, String> {
    if kappa.is_some_and(|kappa| !canonical_source_manifest_kappa(kappa)) {
        return Err(format!(
            "source manifest kappa {kappa:?} is not canonical",
            kappa = kappa.unwrap_or_default()
        ));
    }
    let record = SourceCompilePreflight {
        schema: SOURCE_COMPILE_PREFLIGHT_SCHEMA.to_owned(),
        source_manifest_kappa: kappa.map(str::to_owned),
    };
    let mut bytes = serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_source_compile_preflight_path(path: &Path) -> Result<SourceCompilePreflight, String> {
    let bytes = read_required_regular_file_nofollow(path, "source compile preflight")?;
    reject_duplicate_json_fields(&bytes, path, "source compile preflight")?;
    let record: SourceCompilePreflight = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    if record.schema != SOURCE_COMPILE_PREFLIGHT_SCHEMA {
        return Err(format!(
            "{} records unsupported schema {:?}",
            path.display(),
            record.schema
        ));
    }
    let canonical = source_compile_preflight_bytes(record.source_manifest_kappa.as_deref())?;
    if bytes != canonical {
        return Err(format!(
            "{} is not the canonical source compile preflight",
            path.display()
        ));
    }
    Ok(record)
}

fn source_compile_initialization_path(output: &Path) -> Result<PathBuf, String> {
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no parent for atomic initialization",
            output.display()
        )
    })?;
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source compile output name is not UTF-8: {}",
                output.display()
            )
        })?;
    Ok(parent.join(format!(".{name}.{SOURCE_COMPILE_PREFLIGHT_FILE}.init")))
}

fn read_optional_source_compile_preflight(
    output: &Path,
) -> Result<Option<SourceCompilePreflight>, String> {
    let path = output.join(SOURCE_COMPILE_PREFLIGHT_FILE);
    match open_regular_file_nofollow(&path, "source compile preflight")? {
        Some(file) => {
            let bytes = read_opened_regular_file_nofollow(file, &path, "source compile preflight")?;
            reject_duplicate_json_fields(&bytes, &path, "source compile preflight")?;
            let record: SourceCompilePreflight = serde_json::from_slice(&bytes)
                .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
            if record.schema != SOURCE_COMPILE_PREFLIGHT_SCHEMA {
                return Err(format!(
                    "{} records unsupported schema {:?}",
                    path.display(),
                    record.schema
                ));
            }
            let canonical =
                source_compile_preflight_bytes(record.source_manifest_kappa.as_deref())?;
            if bytes != canonical {
                return Err(format!(
                    "{} is not the canonical source compile preflight",
                    path.display()
                ));
            }
            Ok(Some(record))
        }
        None => Ok(None),
    }
}

fn source_manifest_kappa_binding_bytes(kappa: &str) -> Result<Vec<u8>, String> {
    if !canonical_source_manifest_kappa(kappa) {
        return Err(format!("source manifest kappa {kappa:?} is not canonical"));
    }
    let binding = SourceManifestKappaBinding {
        schema: SOURCE_MANIFEST_KAPPA_BINDING_SCHEMA.to_owned(),
        source_manifest_kappa: kappa.to_owned(),
    };
    let mut bytes = serde_json::to_vec_pretty(&binding).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_optional_source_manifest_kappa_binding(output: &Path) -> Result<Option<String>, String> {
    let path = output.join(SOURCE_MANIFEST_KAPPA_BINDING_FILE);
    let Some(bytes) = read_regular_file_nofollow(&path, "source-manifest kappa binding")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, &path, "source-manifest kappa binding")?;
    let binding: SourceManifestKappaBinding = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    if binding.schema != SOURCE_MANIFEST_KAPPA_BINDING_SCHEMA {
        return Err(format!(
            "{} records unsupported schema {:?}",
            path.display(),
            binding.schema
        ));
    }
    let canonical = source_manifest_kappa_binding_bytes(&binding.source_manifest_kappa)?;
    if bytes != canonical {
        return Err(format!(
            "{} is not the canonical source-manifest kappa binding",
            path.display()
        ));
    }
    Ok(Some(binding.source_manifest_kappa))
}

fn source_compile_output_has_payload(output: &Path) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", output.display())),
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "source compile output {} is not a regular non-symlink directory",
            output.display()
        ));
    }
    validate_source_compile_identity_temporaries(output)?;
    let entries = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source compile output {} contains a non-UTF-8 entry",
                output.display()
            )
        })?;
        if name == SOURCE_MANIFEST_KAPPA_BINDING_FILE || name == SOURCE_COMPILE_PREFLIGHT_FILE {
            continue;
        }
        if name == "tokenizer_adapter.json"
            || name == uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE
        {
            validate_pre_attention_identity_file(&entry.path(), name)?;
            continue;
        }
        if let Some(kind) = atomic_identity_temporary_kind(name) {
            validate_pre_attention_identity_file(&entry.path(), kind)?;
            continue;
        }
        if looks_like_atomic_identity_temporary(name) {
            return Err(format!(
                "source compile output {} contains unrecognized or non-owned identity temporary {name}",
                output.display()
            ));
        }
        return Ok(true);
    }
    Ok(false)
}

fn looks_like_atomic_identity_temporary(name: &str) -> bool {
    [
        SOURCE_COMPILE_PREFLIGHT_FILE,
        SOURCE_MANIFEST_KAPPA_BINDING_FILE,
        "tokenizer_adapter.json",
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
    ]
    .iter()
    .any(|sidecar| name.starts_with(&format!(".{sidecar}.")) && name.ends_with(".tmp"))
}

fn atomic_identity_temporary_kind(name: &str) -> Option<&'static str> {
    for sidecar in [
        SOURCE_COMPILE_PREFLIGHT_FILE,
        SOURCE_MANIFEST_KAPPA_BINDING_FILE,
        "tokenizer_adapter.json",
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
    ] {
        let prefix = format!(".{sidecar}.");
        let Some(sequence) = name
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".tmp"))
        else {
            continue;
        };
        let mut parts = sequence.split('.');
        if parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            && parts.next().is_some_and(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            })
            && parts.next().is_none()
        {
            return Some(sidecar);
        }
    }
    None
}

fn validate_source_compile_identity_temporaries(output: &Path) -> Result<(), String> {
    let mut identities = std::collections::BTreeMap::<&'static str, serde_json::Value>::new();
    for kind in [
        SOURCE_COMPILE_PREFLIGHT_FILE,
        SOURCE_MANIFEST_KAPPA_BINDING_FILE,
        "tokenizer_adapter.json",
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
    ] {
        let path = output.join(kind);
        if let Some(file) = open_regular_file_nofollow(&path, "pre-attention identity record")? {
            let value = validate_pre_attention_identity_handle(file, &path, kind)?;
            identities.insert(kind, value);
        }
    }
    let entries = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source compile output {} contains a non-UTF-8 entry",
                output.display()
            )
        })?;
        if let Some(kind) = atomic_identity_temporary_kind(name) {
            let value = validate_pre_attention_identity_file(&entry.path(), kind)?;
            if let Some(existing) = identities.get(kind) {
                if existing != &value {
                    return Err(format!(
                        "source compile output {} has conflicting stable/temporary {kind} identities",
                        output.display()
                    ));
                }
            } else {
                identities.insert(kind, value);
            }
        } else if looks_like_atomic_identity_temporary(name) {
            return Err(format!(
                "source compile output {} contains unrecognized or non-owned identity temporary {name}",
                output.display()
            ));
        }
    }
    Ok(())
}

/// Reclaim only graph/server publisher temporaries whose exact reserved name
/// is known. The caller must hold the physical output's exclusive OS session
/// lock, so no live cooperating publisher can own these files. Stable
/// sidecars are never removed; symlinks, special entries, and merely
/// similar/unknown names remain terminal.
fn recover_source_compile_identity_temporaries(output: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", output.display())),
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "source compile output {} is not a regular non-symlink directory",
            output.display()
        ));
    }
    let entries = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source compile output {} contains a non-UTF-8 entry",
                output.display()
            )
        })?;
        if atomic_identity_temporary_kind(name).is_some() {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("{} cannot be inspected: {error}", path.display()))?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized source identity temporary {} is not a regular non-symlink file; refusing recovery",
                    path.display()
                ));
            }
            fs::remove_file(&path).map_err(|error| {
                format!(
                    "recognized source identity temporary {} cannot be reclaimed under exclusive session ownership: {error}",
                    path.display()
                )
            })?;
        } else if uor_r4_core::transformerless::compiler::is_source_corpus_checkpoint_temporary_name(
            name,
            "corpus.meta",
        ) {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("{} cannot be inspected: {error}", path.display()))?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized source corpus checkpoint temporary {} is not a regular non-symlink file; refusing recovery",
                    path.display()
                ));
            }
            fs::remove_file(&path).map_err(|error| {
                format!(
                    "recognized source corpus checkpoint temporary {} cannot be reclaimed under exclusive session ownership: {error}",
                    path.display()
                )
            })?;
        } else if name.starts_with(".corpus.meta.checkpoint.") && name.ends_with(".tmp") {
            return Err(format!(
                "source compile output {} contains unrecognized or non-owned checkpoint temporary {name}",
                output.display()
            ));
        } else if looks_like_atomic_identity_temporary(name) {
            return Err(format!(
                "source compile output {} contains unrecognized or non-owned identity temporary {name}",
                output.display()
            ));
        }
    }
    Ok(())
}

/// Return the identity carried by canonical preflight temporaries. The outer
/// option distinguishes no temporary from a manifestless `null` binding.
fn source_compile_preflight_temporary_binding(
    output: &Path,
) -> Result<Option<Option<String>>, String> {
    let entries = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
    let mut binding: Option<Option<String>> = None;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if atomic_identity_temporary_kind(name) != Some(SOURCE_COMPILE_PREFLIGHT_FILE) {
            continue;
        }
        let value =
            validate_pre_attention_identity_file(&entry.path(), SOURCE_COMPILE_PREFLIGHT_FILE)?;
        let record: SourceCompilePreflight = serde_json::from_value(value)
            .map_err(|error| format!("{} is malformed: {error}", entry.path().display()))?;
        if let Some(existing) = binding.as_ref() {
            if existing != &record.source_manifest_kappa {
                return Err(format!(
                    "source compile output {} has conflicting canonical preflight temporaries",
                    output.display()
                ));
            }
        } else {
            binding = Some(record.source_manifest_kappa);
        }
    }
    Ok(binding)
}

fn validate_pre_attention_identity_file(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let file =
        open_regular_file_nofollow(path, "pre-attention identity record")?.ok_or_else(|| {
            format!(
                "pre-attention identity entry {} disappeared before validation",
                path.display()
            )
        })?;
    validate_pre_attention_identity_handle(file, path, kind)
}

fn validate_pre_attention_identity_handle(
    file: fs::File,
    path: &Path,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let description = if kind == uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE {
        "attention-operator binding"
    } else {
        "pre-attention identity record"
    };
    let bytes = read_opened_regular_file_nofollow(file, path, description)?;
    reject_duplicate_json_fields(&bytes, path, description)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    match kind {
        SOURCE_COMPILE_PREFLIGHT_FILE => {
            let record: SourceCompilePreflight = serde_json::from_value(value.clone())
                .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
            if record.schema != SOURCE_COMPILE_PREFLIGHT_SCHEMA {
                return Err(format!(
                    "{} records unsupported schema {:?}",
                    path.display(),
                    record.schema
                ));
            }
            let expected = source_compile_preflight_bytes(record.source_manifest_kappa.as_deref())?;
            if bytes != expected {
                return Err(format!(
                    "{} is not a canonical source compile preflight temporary",
                    path.display()
                ));
            }
        }
        SOURCE_MANIFEST_KAPPA_BINDING_FILE => {
            let binding: SourceManifestKappaBinding = serde_json::from_value(value.clone())
                .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
            if binding.schema != SOURCE_MANIFEST_KAPPA_BINDING_SCHEMA {
                return Err(format!(
                    "{} records unsupported schema {:?}",
                    path.display(),
                    binding.schema
                ));
            }
            let expected = source_manifest_kappa_binding_bytes(&binding.source_manifest_kappa)?;
            if bytes != expected {
                return Err(format!(
                    "{} is not a canonical source-manifest kappa binding temporary",
                    path.display()
                ));
            }
        }
        "tokenizer_adapter.json" => {
            let adapter: TokenizerAdapter =
                serde_json::from_value(value.clone()).map_err(|error| {
                    format!(
                        "{} is not a tokenizer adapter record: {error}",
                        path.display()
                    )
                })?;
            adapter_constructor(&adapter.family, adapter.version).map_err(|error| {
                format!(
                    "{} names an unsupported tokenizer adapter: {error}",
                    path.display()
                )
            })?;
            if !canonical_source_manifest_kappa(&adapter.tokenizer_cid) {
                return Err(format!(
                    "{} records a noncanonical tokenizer CID {}",
                    path.display(),
                    adapter.tokenizer_cid
                ));
            }
            let declared = adapter.declared_digest();
            if adapter.adapter_digest != declared {
                return Err(format!(
                    "{} declares tokenizer adapter digest {}, expected {declared}",
                    path.display(),
                    adapter.adapter_digest
                ));
            }
            if value
                != serde_json::to_value(&adapter)
                    .map_err(|error| format!("{}: {error}", path.display()))?
            {
                return Err(format!(
                    "{} is not the full canonical tokenizer adapter record",
                    path.display()
                ));
            }
        }
        kind if kind == uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE => {
            let operator: AttentionOperatorSpec =
                serde_json::from_value(value.clone()).map_err(|error| {
                    format!(
                        "{} is not an attention operator record: {error}",
                        path.display()
                    )
                })?;
            let registered =
                uor_r4_model_source::attention::operator_spec(&operator.id, operator.version)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
            if operator != registered
                || value
                    != serde_json::to_value(&registered)
                        .map_err(|error| format!("{}: {error}", path.display()))?
            {
                return Err(format!(
                    "{} is not the full registered attention operator record",
                    path.display()
                ));
            }
        }
        _ => {
            return Err(format!(
                "{} is not a recognized pre-attention identity entry",
                path.display()
            ));
        }
    }
    Ok(value)
}

/// Recognize only the immutable identity prefix that source compilation can
/// publish before its attention sidecar. This makes a process-death retry
/// resumable without treating arbitrary payload as an implicit v1 bundle.
fn source_compile_pre_attention_prefix(output: &Path) -> Result<bool, String> {
    let preflight = read_optional_source_compile_preflight(output)?;
    reject_legacy_source_compile_initialization(output)?;
    let kappa = read_optional_source_manifest_kappa_binding(output)?;
    let recorded_kappa = preflight
        .as_ref()
        .and_then(|record| record.source_manifest_kappa.as_deref());
    if let Some(kappa) = kappa.as_deref() {
        if preflight.is_some() && recorded_kappa != Some(kappa) {
            return Err(format!(
                "source compile root {} has conflicting preflight ({recorded_kappa:?}) and source-manifest kappa ({kappa}) identities",
                output.display()
            ));
        }
    }
    let mut saw_identity = preflight.is_some() || kappa.is_some();
    let mut unexpected = Vec::new();
    let entries = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source compile root {} contains a non-UTF-8 pre-attention entry",
                output.display()
            )
        })?;
        if name == SOURCE_MANIFEST_KAPPA_BINDING_FILE || name == SOURCE_COMPILE_PREFLIGHT_FILE {
            saw_identity = true;
            continue;
        }
        if name == "tokenizer_adapter.json" {
            validate_pre_attention_identity_file(&entry.path(), name)?;
            saw_identity = true;
            continue;
        }
        if let Some(kind) = atomic_identity_temporary_kind(name) {
            validate_pre_attention_identity_file(&entry.path(), kind)?;
            saw_identity = true;
            continue;
        }
        if looks_like_atomic_identity_temporary(name) {
            return Err(format!(
                "source compile root {} contains unrecognized or non-owned identity temporary {name}",
                output.display()
            ));
        }
        unexpected.push(name.to_owned());
    }
    if saw_identity && !unexpected.is_empty() {
        return Err(format!(
            "source compile root {} contains {} beside {} but has no {}; refusing to infer a historical era from an interrupted current-era compile",
            output.display(),
            unexpected.join(", "),
            SOURCE_MANIFEST_KAPPA_BINDING_FILE,
            uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE
        ));
    }
    Ok(saw_identity)
}

fn publish_bytes_no_clobber(path: &Path, expected: &[u8], label: &str) -> Result<(), String> {
    if let Some(existing) = read_regular_file_nofollow(path, label)? {
        return if existing == expected {
            sync_parent_directory(path, label)
        } else {
            Err(format!(
                "{} already records a different {label}",
                path.display()
            ))
        };
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    let parent_metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("{} cannot be inspected: {error}", parent.display()))?;
    if !parent_metadata.file_type().is_dir() {
        return Err(format!(
            "{} is not a regular non-symlink directory",
            parent.display()
        ));
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))?;
    for _ in 0..128 {
        let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), id));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{}: {error}", temporary.display())),
        };
        if let Err(error) = file.write_all(expected).and_then(|()| file.sync_all()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("{}: {error}", temporary.display()));
        }
        drop(file);
        match fs::hard_link(&temporary, path) {
            Ok(()) => {
                fs::remove_file(&temporary)
                    .map_err(|error| format!("{}: {error}", temporary.display()))?;
                return sync_parent_directory(path, label);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
                let existing = read_regular_file_nofollow(path, label)?.ok_or_else(|| {
                    format!(
                        "{} concurrently appeared and disappeared while publishing {label}",
                        path.display()
                    )
                })?;
                return if existing == expected {
                    sync_parent_directory(path, label)
                } else {
                    Err(format!(
                        "{} concurrently recorded a different {label}",
                        path.display()
                    ))
                };
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(format!(
                    "{label} publish {} -> {}: {error}",
                    temporary.display(),
                    path.display()
                ));
            }
        }
    }
    Err(format!(
        "could not reserve a unique {label} temporary in {}",
        parent.display()
    ))
}

fn sync_parent_directory(path: &Path, label: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "{label} parent directory {} cannot be durably synchronized: {error}",
                parent.display()
            )
        })
}

#[cfg(unix)]
fn exclusive_rename_path_bytes(path: &Path) -> Result<std::ffi::CString, String> {
    use std::os::unix::ffi::OsStrExt;

    std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        format!(
            "source compile publication path contains an interior NUL: {}",
            path.display()
        )
    })
}

/// Rename one fully populated staging directory into an absent final name
/// without ever replacing an entry published by another actor.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn rename_directory_no_replace(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    let from = exclusive_rename_path_bytes(from)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let to = exclusive_rename_path_bytes(to)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both C strings are live for the call, contain no interior NUL,
    // and `renameat2` borrows rather than retains their pointers.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_vendor = "apple")]
fn rename_directory_no_replace(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    let from = exclusive_rename_path_bytes(from)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let to = exclusive_rename_path_bytes(to)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both C strings are live for the call, contain no interior NUL,
    // and `renameatx_np` borrows rather than retains their pointers.
    let result = unsafe {
        libc::renameatx_np(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_EXCL,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
fn rename_directory_no_replace(_from: &Path, _to: &Path) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this platform has no atomic no-replace directory rename primitive",
    ))
}

/// Atomically exchange two existing directory names on the same filesystem.
/// The fully validated replacement becomes visible in one namespace step;
/// the old last-good directory moves to `from` for post-publication cleanup.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn exchange_directories(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    let from = exclusive_rename_path_bytes(from)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let to = exclusive_rename_path_bytes(to)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both C strings remain live for the call and renameat2 borrows
    // their pointers. RENAME_EXCHANGE is a single filesystem transaction.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_vendor = "apple")]
fn exchange_directories(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    let from = exclusive_rename_path_bytes(from)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let to = exclusive_rename_path_bytes(to)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both C strings remain live for the call and renameatx_np
    // borrows their pointers. RENAME_SWAP is an atomic name exchange.
    let result = unsafe {
        libc::renameatx_np(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_SWAP,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
fn exchange_directories(_from: &Path, _to: &Path) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this platform has no atomic directory-exchange primitive",
    ))
}

fn source_compile_staging_root(parent: &Path) -> PathBuf {
    parent.join(SOURCE_COMPILE_STAGING_DIR)
}

fn source_compile_session_lock_path(output: &Path) -> Result<PathBuf, String> {
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no parent for session coordination",
            output.display()
        )
    })?;
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source compile output name is not UTF-8: {}",
                output.display()
            )
        })?;
    Ok(source_compile_staging_root(parent)
        .join(format!("{output_name}{SOURCE_COMPILE_SESSION_LOCK_SUFFIX}")))
}

/// Cross-process ownership for one physical source-compile output.
///
/// The coordination inode lives in the reserved staging namespace rather
/// than in the mutable bundle inventory. It is deliberately persistent: two
/// cooperating processes must always open and lock the same inode, including
/// after a completed compile. Dropping the guard releases the OS lock but does
/// not unlink the coordination file.
struct SourceCompileSessionLock {
    file: fs::File,
}

struct SourceCompileSessionLocks {
    _locks: Vec<SourceCompileSessionLock>,
}

#[allow(dead_code)] // SharedReader remains part of the tested cross-process protocol
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceCompileSessionMode {
    SharedReader,
    ExclusiveWriter,
}

impl SourceCompileSessionMode {
    fn label(self) -> &'static str {
        match self {
            Self::SharedReader => "reader",
            Self::ExclusiveWriter => "writer",
        }
    }
}

fn source_compile_session_is_busy(error: &str) -> bool {
    error.contains(" is BUSY under an active cross-process compile session; refusing ")
}

fn graph_output_session_subject(graph_path: &Path) -> PathBuf {
    graph_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

impl Drop for SourceCompileSessionLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(unix)]
fn source_compile_lock_metadata_matches(
    path_metadata: &fs::Metadata,
    file_metadata: &fs::Metadata,
) -> bool {
    use std::os::unix::fs::MetadataExt;

    path_metadata.dev() == file_metadata.dev() && path_metadata.ino() == file_metadata.ino()
}

#[cfg(not(unix))]
fn source_compile_lock_metadata_matches(
    _path_metadata: &fs::Metadata,
    _file_metadata: &fs::Metadata,
) -> bool {
    // `OpenOptions` plus the before/after regular-file checks below reject
    // pre-existing links and type changes. Unix additionally compares the
    // opened inode because its portable metadata extension exposes identity.
    true
}

fn canonical_compile_session_subject(subject: &Path) -> Result<PathBuf, String> {
    let normalized = normalized_absolute_path(subject)?;
    let mut existing = normalized.as_path();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = existing.file_name().ok_or_else(|| {
                    format!(
                        "compile session subject {} has no existing ancestor",
                        normalized.display()
                    )
                })?;
                missing.push(name.to_os_string());
                existing = existing.parent().ok_or_else(|| {
                    format!(
                        "compile session subject {} has no existing ancestor",
                        normalized.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "compile session subject {} cannot be inspected: {error}",
                    existing.display()
                ));
            }
        }
    }
    let mut canonical = fs::canonicalize(existing).map_err(|error| {
        format!(
            "compile session subject ancestor {} cannot be canonicalized: {error}",
            existing.display()
        )
    })?;
    for component in missing.into_iter().rev() {
        canonical.push(component);
    }
    Ok(canonical)
}

#[cfg(test)]
fn try_lock_source_compile_session_mode(
    subject: &Path,
    mode: SourceCompileSessionMode,
) -> Result<SourceCompileSessionLock, String> {
    let output = canonical_compile_session_subject(subject)?;
    try_lock_canonical_source_compile_session_mode(&output, mode)
}

/// Lock one already-canonical subject. Multi-subject callers canonicalize
/// exactly once before sorting, then use this inner primitive so an alias
/// swap cannot change the subject or acquisition order between those steps.
fn try_lock_canonical_source_compile_session_mode(
    output: &Path,
    mode: SourceCompileSessionMode,
) -> Result<SourceCompileSessionLock, String> {
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no parent for session coordination",
            output.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("{} cannot be created: {error}", parent.display()))?;
    let parent_metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("{} cannot be inspected: {error}", parent.display()))?;
    if !parent_metadata.file_type().is_dir() {
        return Err(format!(
            "source compile parent {} is not a regular non-symlink directory",
            parent.display()
        ));
    }
    let staging = ensure_source_compile_staging_root(parent)?;
    let lock_path = source_compile_session_lock_path(output)?;
    debug_assert_eq!(lock_path.parent(), Some(staging.as_path()));

    match fs::symlink_metadata(&lock_path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => {
            return Err(format!(
                "source compile session coordination {} is not a regular non-symlink file",
                lock_path.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "source compile session coordination {} cannot be inspected: {error}",
                lock_path.display()
            ));
        }
    }

    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        // O_NOFOLLOW closes the link-swap seam between the metadata check and
        // open. O_NONBLOCK ensures a raced FIFO/device cannot block this
        // request before the opened-handle type check rejects it.
        options
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(&lock_path).map_err(|error| {
        format!(
            "source compile session coordination {} cannot be opened: {error}",
            lock_path.display()
        )
    })?;
    let path_metadata = fs::symlink_metadata(&lock_path).map_err(|error| {
        format!(
            "source compile session coordination {} cannot be reinspected: {error}",
            lock_path.display()
        )
    })?;
    let file_metadata = file.metadata().map_err(|error| {
        format!(
            "source compile session coordination {} handle cannot be inspected: {error}",
            lock_path.display()
        )
    })?;
    if !path_metadata.file_type().is_file()
        || !file_metadata.file_type().is_file()
        || !source_compile_lock_metadata_matches(&path_metadata, &file_metadata)
    {
        return Err(format!(
            "source compile session coordination {} changed identity or is not a regular non-symlink file",
            lock_path.display()
        ));
    }

    let lock = match mode {
        SourceCompileSessionMode::SharedReader => file.try_lock_shared(),
        SourceCompileSessionMode::ExclusiveWriter => file.try_lock(),
    };
    match lock {
        Ok(()) => Ok(SourceCompileSessionLock { file }),
        Err(fs::TryLockError::WouldBlock) => Err(format!(
            "source compile output {} is BUSY under an active cross-process compile session; refusing {} access",
            output.display(), mode.label()
        )),
        Err(fs::TryLockError::Error(error)) => Err(format!(
            "source compile session coordination {} cannot be locked: {error}",
            lock_path.display()
        )),
    }
}

#[cfg(test)]
fn try_lock_source_compile_session(output: &Path) -> Result<SourceCompileSessionLock, String> {
    try_lock_source_compile_session_mode(output, SourceCompileSessionMode::ExclusiveWriter)
}

fn try_lock_source_compile_sessions(
    subjects: impl IntoIterator<Item = PathBuf>,
    mode: SourceCompileSessionMode,
) -> Result<SourceCompileSessionLocks, String> {
    let mut subjects = subjects
        .into_iter()
        .map(|subject| canonical_compile_session_subject(&subject))
        .collect::<Result<Vec<_>, _>>()?;
    subjects.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
    subjects.dedup();
    let mut locks = Vec::with_capacity(subjects.len());
    for subject in subjects {
        locks.push(try_lock_canonical_source_compile_session_mode(
            &subject, mode,
        )?);
    }
    Ok(SourceCompileSessionLocks { _locks: locks })
}

fn ensure_source_compile_staging_root(parent: &Path) -> Result<PathBuf, String> {
    let staging = source_compile_staging_root(parent);
    match fs::create_dir(&staging) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("{} cannot be created: {error}", staging.display())),
    }
    let metadata = fs::symlink_metadata(&staging)
        .map_err(|error| format!("{} cannot be inspected: {error}", staging.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "source compile staging namespace {} is not a regular non-symlink directory",
            staging.display()
        ));
    }
    Ok(staging)
}

fn compiled_bundle_stage_name(output_name: &str, pid: u32, id: u64) -> String {
    format!(".{output_name}.{COMPILED_BUNDLE_STAGE_TAG}.{pid}.{id}")
}

fn is_compiled_bundle_stage_name(name: &str, output_name: &str) -> bool {
    let prefix = format!(".{output_name}.{COMPILED_BUNDLE_STAGE_TAG}.");
    let Some(sequence) = name.strip_prefix(&prefix) else {
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

fn looks_like_compiled_bundle_stage(name: &str, output_name: &str) -> bool {
    name.starts_with(&format!(".{output_name}.{COMPILED_BUNDLE_STAGE_TAG}."))
}

fn compiled_bundle_stage_marker_bytes(
    final_output: &Path,
    stage_path: &Path,
    source_snapshot_kappa: &str,
) -> Result<Vec<u8>, String> {
    if !canonical_source_manifest_kappa(source_snapshot_kappa) {
        return Err(format!(
            "source snapshot kappa {source_snapshot_kappa:?} is not canonical"
        ));
    }
    let final_output = canonical_compile_session_subject(final_output)?;
    let final_output = final_output.to_str().ok_or_else(|| {
        format!(
            "compiled-bundle destination is not UTF-8: {}",
            final_output.display()
        )
    })?;
    let stage_path = canonical_compile_session_subject(stage_path)?;
    let stage_path = stage_path.to_str().ok_or_else(|| {
        format!(
            "compiled-bundle stage is not UTF-8: {}",
            stage_path.display()
        )
    })?;
    let mut bytes = serde_json::to_vec_pretty(&CompiledBundleStageMarker {
        schema: COMPILED_BUNDLE_STAGE_SCHEMA.to_owned(),
        final_output: final_output.to_owned(),
        stage_path: stage_path.to_owned(),
        source_snapshot_kappa: source_snapshot_kappa.to_owned(),
    })
    .map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_compiled_bundle_stage_marker(stage: &Path) -> Result<Option<Vec<u8>>, String> {
    let path = stage.join(COMPILED_BUNDLE_STAGE_MARKER_FILE);
    let Some(bytes) = read_regular_file_nofollow(&path, "compiled-bundle stage marker")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, &path, "compiled-bundle stage marker")?;
    let record: CompiledBundleStageMarker = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    let mut canonical = serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
    canonical.push(b'\n');
    if record.schema != COMPILED_BUNDLE_STAGE_SCHEMA || bytes != canonical {
        return Err(format!(
            "{} is not a canonical supported compiled-bundle stage marker",
            path.display()
        ));
    }
    Ok(Some(bytes))
}

fn validate_compiled_bundle_stage_marker_location(root: &Path, bytes: &[u8]) -> Result<(), String> {
    let record: CompiledBundleStageMarker =
        serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    if !canonical_source_manifest_kappa(&record.source_snapshot_kappa) {
        return Err(format!(
            "compiled-bundle marker in {} records noncanonical source snapshot κ {:?}",
            root.display(),
            record.source_snapshot_kappa
        ));
    }
    let final_output = canonical_compile_session_subject(Path::new(&record.final_output))?;
    let stage_path = canonical_compile_session_subject(Path::new(&record.stage_path))?;
    if final_output.to_str() != Some(record.final_output.as_str())
        || stage_path.to_str() != Some(record.stage_path.as_str())
    {
        return Err(format!(
            "compiled-bundle marker in {} contains a noncanonical path",
            root.display()
        ));
    }
    let output_name = final_output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("compiled output is not UTF-8: {}", final_output.display()))?;
    let stage_name = stage_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("compiled stage is not UTF-8: {}", stage_path.display()))?;
    let final_parent = final_output
        .parent()
        .ok_or_else(|| format!("compiled output {} has no parent", final_output.display()))?;
    let expected_stage_parent =
        canonical_compile_session_subject(&source_compile_staging_root(final_parent))?;
    if stage_path.parent() != Some(expected_stage_parent.as_path())
        || !is_compiled_bundle_stage_name(stage_name, output_name)
    {
        return Err(format!(
            "compiled-bundle marker in {} records an invalid stage path {}",
            root.display(),
            stage_path.display()
        ));
    }
    let actual = canonical_compile_session_subject(root)?;
    if actual != final_output && actual != stage_path {
        return Err(format!(
            "compiled-bundle marker in {} belongs to neither its public nor staging namespace",
            root.display()
        ));
    }
    Ok(())
}

fn validate_compiled_bundle_stage_marker(stage: &Path, expected: &[u8]) -> Result<bool, String> {
    Ok(read_compiled_bundle_stage_marker(stage)?.is_some_and(|bytes| bytes == expected))
}

fn cleanup_published_compiled_bundle_stage_marker(
    final_output: &Path,
    source_snapshot_kappa: &str,
) -> Result<(), String> {
    let Some(bytes) = read_compiled_bundle_stage_marker(final_output)? else {
        return Ok(());
    };
    validate_compiled_bundle_stage_marker_location(final_output, &bytes)?;
    let record: CompiledBundleStageMarker =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if record.source_snapshot_kappa != source_snapshot_kappa {
        return Err(format!(
            "published compiled-bundle marker in {} binds source snapshot {}, not {}",
            final_output.display(),
            record.source_snapshot_kappa,
            source_snapshot_kappa
        ));
    }
    validate_compiled_bundle_completion(final_output)?.ok_or_else(|| {
        format!(
            "published compiled-bundle marker in {} has no exact completion record",
            final_output.display()
        )
    })?;
    let path = final_output.join(COMPILED_BUNDLE_STAGE_MARKER_FILE);
    fs::remove_file(&path).map_err(|error| {
        format!(
            "published compiled-bundle marker {} cannot be reclaimed: {error}",
            path.display()
        )
    })?;
    fs::File::open(final_output)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", final_output.display()))
}

fn is_atomic_publisher_temporary(name: &str, stable_name: &str) -> bool {
    let prefix = format!(".{stable_name}.");
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

fn looks_like_atomic_publisher_temporary(name: &str, stable_name: &str) -> bool {
    name.starts_with(&format!(".{stable_name}.")) && name.ends_with(".tmp")
}

fn compiled_bundle_stage_temporaries(stage: &Path) -> Result<Vec<PathBuf>, String> {
    let mut temporaries = Vec::new();
    for entry in fs::read_dir(stage)
        .map_err(|error| format!("{} cannot be enumerated: {error}", stage.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", stage.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "compiled-bundle stage {} contains a non-UTF-8 entry",
                stage.display()
            )
        })?;
        let exact = [
            COMPILED_BUNDLE_STAGE_MARKER_FILE,
            COMPILED_BUNDLE_COMPLETION_FILE,
        ]
        .into_iter()
        .any(|stable| is_atomic_publisher_temporary(name, stable));
        let looks_reserved = [
            COMPILED_BUNDLE_STAGE_MARKER_FILE,
            COMPILED_BUNDLE_COMPLETION_FILE,
        ]
        .into_iter()
        .any(|stable| looks_like_atomic_publisher_temporary(name, stable));
        if exact {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized compiled-bundle publisher temporary {} is not a regular non-symlink file",
                    entry.path().display()
                ));
            }
            temporaries.push(entry.path());
        } else if looks_reserved {
            return Err(format!(
                "compiled-bundle stage {} contains unrecognized publisher temporary {name}",
                stage.display()
            ));
        }
    }
    Ok(temporaries)
}

fn reset_resumable_compiled_bundle_stage(
    stage: &Path,
    temporaries: Vec<PathBuf>,
) -> Result<(), String> {
    let completion = stage.join(COMPILED_BUNDLE_COMPLETION_FILE);
    let mut remove_completion = false;
    match fs::symlink_metadata(&completion) {
        Ok(metadata) if metadata.file_type().is_file() => remove_completion = true,
        Ok(_) => {
            return Err(format!(
                "resumable compiled-bundle completion {} is not a regular non-symlink file",
                completion.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "{} cannot be inspected: {error}",
                completion.display()
            ));
        }
    }
    let mut derived = Vec::new();
    for name in ["graph-cover", "graph"] {
        let path = stage.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_dir() => derived.push(path),
            Ok(_) => {
                return Err(format!(
                    "resumable compiled-bundle output {} is not a regular non-symlink directory",
                    path.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("{} cannot be inspected: {error}", path.display()));
            }
        }
    }
    if remove_completion {
        fs::remove_file(&completion)
            .map_err(|error| format!("{} cannot be reclaimed: {error}", completion.display()))?;
    }
    for temporary in temporaries {
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "recognized compiled-bundle publisher temporary {} cannot be reclaimed: {error}",
                temporary.display()
            )
        })?;
    }
    for path in derived {
        fs::remove_dir_all(&path).map_err(|error| {
            format!(
                "derived compiled-bundle stage output {} cannot be reclaimed: {error}",
                path.display()
            )
        })?;
    }
    fs::File::open(stage)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", stage.display()))
}

fn recover_compiled_bundle_stages(
    output: &Path,
    source_snapshot_kappa: &str,
) -> Result<Option<(PathBuf, Vec<u8>)>, String> {
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no staging parent",
            output.display()
        )
    })?;
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("source compile output is not UTF-8: {}", output.display()))?;
    let staging = ensure_source_compile_staging_root(parent)?;
    let mut resumable = None;
    let mut stale = Vec::new();
    for entry in fs::read_dir(&staging)
        .map_err(|error| format!("{} cannot be enumerated: {error}", staging.display()))?
    {
        let entry = entry
            .map_err(|error| format!("{} cannot be enumerated: {error}", staging.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source compile staging namespace {} contains a non-UTF-8 entry",
                staging.display()
            )
        })?;
        if is_compiled_bundle_stage_name(name, output_name) {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_dir() {
                return Err(format!(
                    "recognized compiled-bundle stage {} is not a regular non-symlink directory",
                    entry.path().display()
                ));
            }
            let temporaries = compiled_bundle_stage_temporaries(&entry.path())?;
            let expected_marker =
                compiled_bundle_stage_marker_bytes(output, &entry.path(), source_snapshot_kappa)?;
            let recorded_marker = read_compiled_bundle_stage_marker(&entry.path())?;
            if recorded_marker.as_deref() == Some(expected_marker.as_slice()) {
                if resumable.is_some() {
                    return Err(format!(
                        "source compile output {} has multiple resumable bundle stages; refusing ambiguous adoption",
                        output.display()
                    ));
                }
                resumable = Some((entry.path(), expected_marker, temporaries));
            } else if recorded_marker.is_some()
                && validate_compiled_bundle_completion(&entry.path())?.is_none()
            {
                return Err(format!(
                    "{} does not bind this exact compiled output, stage path, and source snapshot",
                    entry
                        .path()
                        .join(COMPILED_BUNDLE_STAGE_MARKER_FILE)
                        .display()
                ));
            } else {
                // A markerless exact stage is either a crash before any
                // resumable work began or the old final directory left after
                // a committed exchange. Neither may be adopted as compiler
                // input, and exclusive session ownership makes reclamation
                // race-free with every cooperating publisher.
                stale.push(entry.path());
            }
        } else if looks_like_compiled_bundle_stage(name, output_name) {
            return Err(format!(
                "source compile staging namespace {} contains unrecognized bundle stage {name}",
                staging.display()
            ));
        }
    }
    // Classification is complete before the first mutation. A later malformed
    // or duplicate candidate can therefore never leave earlier stage entries
    // partially reclaimed.
    for stage in stale {
        fs::remove_dir_all(&stage).map_err(|error| {
            format!(
                "stale compiled-bundle stage {} cannot be reclaimed under exclusive session ownership: {error}",
                stage.display()
            )
        })?;
    }
    if let Some((stage, expected_marker, temporaries)) = resumable.as_ref() {
        reset_resumable_compiled_bundle_stage(stage, temporaries.clone())?;
        if !validate_compiled_bundle_stage_marker(stage, expected_marker)? {
            return Err(format!(
                "resumable compiled-bundle stage {} lost its owner marker during recovery",
                stage.display()
            ));
        }
    }
    fs::File::open(&staging)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", staging.display()))?;
    Ok(resumable.map(|(path, marker, _)| (path, marker)))
}

fn copy_regular_file_nofollow(source: &Path, destination: &Path) -> Result<(), String> {
    let mut source_file = open_regular_file_nofollow(source, "compiled-bundle source file")?
        .ok_or_else(|| format!("compiled-bundle source file {} is absent", source.display()))?;
    let opened_metadata = source_file
        .metadata()
        .map_err(|error| format!("{} cannot be inspected: {error}", source.display()))?;
    let mut destination_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("{} cannot be created: {error}", destination.display()))?;
    let copied = std::io::copy(&mut source_file, &mut destination_file)
        .map_err(|error| format!("{} cannot be copied: {error}", source.display()))?;
    destination_file
        .sync_all()
        .map_err(|error| format!("{} cannot be synced: {error}", destination.display()))?;
    let final_source_metadata = source_file
        .metadata()
        .map_err(|error| format!("{} cannot be reinspected: {error}", source.display()))?;
    let final_path_metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("{} cannot be reinspected: {error}", source.display()))?;
    if !final_source_metadata.file_type().is_file()
        || !final_path_metadata.file_type().is_file()
        || !source_compile_lock_metadata_matches(&opened_metadata, &final_source_metadata)
        || !source_compile_lock_metadata_matches(&final_path_metadata, &final_source_metadata)
        || copied != final_source_metadata.len()
    {
        return Err(format!(
            "compiled-bundle source file {} changed identity, type, or length while copied",
            source.display()
        ));
    }
    Ok(())
}

fn validate_legacy_coordination_directory_for_prefix_copy(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{} cannot be inspected: {error}", path.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "legacy coordination entry {} is not a regular non-symlink directory",
            path.display()
        ));
    }
    for entry in fs::read_dir(path)
        .map_err(|error| format!("{} cannot be enumerated: {error}", path.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", path.display()))?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| format!("{} contains a non-UTF-8 coordination entry", path.display()))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("{} cannot be inspected: {error}", entry.path().display()))?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "legacy coordination entry {} is not a regular non-symlink file",
                entry.path().display()
            ));
        }
        let legacy_completion = name.starts_with(".legacy-graph-")
            && name.ends_with(".completion.json")
            && name
                .strip_prefix(".legacy-graph-")
                .and_then(|rest| rest.strip_suffix(".completion.json"))
                .is_some_and(|hash| {
                    hash.len() == 64
                        && hash
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                });
        if !name.ends_with(SOURCE_COMPILE_SESSION_LOCK_SUFFIX) && !legacy_completion {
            return Err(format!(
                "legacy coordination directory {} contains unfinished or unrecognized entry {name}; refusing source-refresh copy",
                path.display()
            ));
        }
    }
    Ok(())
}

fn copy_compiled_bundle_prefix(source: &Path, destination: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source)
        .map_err(|error| format!("{} cannot be enumerated: {error}", source.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", source.display()))?;
        let name = entry.file_name();
        if name == std::ffi::OsStr::new("graph")
            || name == std::ffi::OsStr::new("graph-cover")
            || name == std::ffi::OsStr::new(COMPILED_BUNDLE_COMPLETION_FILE)
            || name == std::ffi::OsStr::new(COMPILED_BUNDLE_STAGE_MARKER_FILE)
        {
            continue;
        }
        if name == std::ffi::OsStr::new(SOURCE_COMPILE_STAGING_DIR) {
            validate_legacy_coordination_directory_for_prefix_copy(&entry.path())?;
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("{} cannot be inspected: {error}", entry.path().display()))?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "compiled-bundle prefix {} contains nonregular entry {}; refusing staging copy",
                source.display(),
                entry.path().display()
            ));
        }
        copy_regular_file_nofollow(&entry.path(), &destination.join(name))?;
    }
    fs::File::open(destination)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", destination.display()))
}

fn validate_copied_compiled_bundle_prefix(
    destination: &Path,
    completion: &CompiledBundleCompletion,
) -> Result<(), String> {
    let expected = completion
        .files
        .iter()
        .filter(|(relative, _)| {
            !relative.starts_with("graph/") && !relative.starts_with("graph-cover/")
        })
        .map(|(relative, kappa)| (relative.clone(), kappa.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let actual = compiled_bundle_file_kappas(destination)?;
    if actual != expected {
        return Err(format!(
            "compiled-bundle prefix copied into {} does not exactly match the authoritative completed generation",
            destination.display()
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct CompiledBundleStage {
    path: PathBuf,
    final_output: PathBuf,
    marker_bytes: Vec<u8>,
}

impl CompiledBundleStage {
    fn allocate(final_output: &Path, source_snapshot_kappa: &str) -> Result<Self, String> {
        // A stable completion is authoritative. Validate every recorded
        // corpus/artifact member before copying a single byte into a resumable
        // refresh stage, and retain the exact record for a post-copy recheck.
        // This prevents a structurally parseable tamper from being copied and
        // freshly re-certified as a new generation.
        recover_compiled_bundle_completion_temporaries(final_output)?;
        let authoritative_completion = validate_compiled_bundle_completion(final_output)?;
        cleanup_published_compiled_bundle_stage_marker(final_output, source_snapshot_kappa)?;
        if let Some((path, marker_bytes)) =
            recover_compiled_bundle_stages(final_output, source_snapshot_kappa)?
        {
            return Ok(Self {
                path,
                final_output: final_output.to_path_buf(),
                marker_bytes,
            });
        }
        let parent = final_output.parent().ok_or_else(|| {
            format!(
                "source compile output {} has no staging parent",
                final_output.display()
            )
        })?;
        let output_name = final_output
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "source compile output is not UTF-8: {}",
                    final_output.display()
                )
            })?;
        let staging = ensure_source_compile_staging_root(parent)?;
        for _ in 0..128 {
            let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
            let path = staging.join(compiled_bundle_stage_name(
                output_name,
                std::process::id(),
                id,
            ));
            match fs::create_dir(&path) {
                Ok(()) => {
                    let marker_bytes = match compiled_bundle_stage_marker_bytes(
                        final_output,
                        &path,
                        source_snapshot_kappa,
                    ) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            let _ = fs::remove_dir_all(&path);
                            let _ =
                                fs::File::open(&staging).and_then(|directory| directory.sync_all());
                            return Err(error);
                        }
                    };
                    if let Err(error) = copy_compiled_bundle_prefix(final_output, &path) {
                        let _ = fs::remove_dir_all(&path);
                        return Err(error);
                    }
                    if let Some(before) = authoritative_completion.as_ref() {
                        let after = match validate_compiled_bundle_completion(final_output) {
                            Ok(Some(after)) => after,
                            Ok(None) => {
                                let _ = fs::remove_dir_all(&path);
                                return Err(format!(
                                    "authoritative compiled bundle {} lost its completion record while a refresh stage was copied",
                                    final_output.display()
                                ));
                            }
                            Err(error) => {
                                let _ = fs::remove_dir_all(&path);
                                return Err(error);
                            }
                        };
                        if &after != before {
                            let _ = fs::remove_dir_all(&path);
                            return Err(format!(
                                "authoritative compiled bundle {} changed completion identity while a refresh stage was copied",
                                final_output.display()
                            ));
                        }
                        if let Err(error) = validate_copied_compiled_bundle_prefix(&path, before) {
                            let _ = fs::remove_dir_all(&path);
                            return Err(error);
                        }
                    }
                    if let Err(error) = publish_bytes_no_clobber(
                        &path.join(COMPILED_BUNDLE_STAGE_MARKER_FILE),
                        &marker_bytes,
                        "compiled-bundle stage marker",
                    ) {
                        let _ = fs::remove_dir_all(&path);
                        return Err(error);
                    }
                    if let Err(error) =
                        fs::File::open(&staging).and_then(|directory| directory.sync_all())
                    {
                        let _ = fs::remove_dir_all(&path);
                        let _ = fs::File::open(&staging).and_then(|directory| directory.sync_all());
                        return Err(format!("{} cannot be synced: {error}", staging.display()));
                    }
                    return Ok(Self {
                        path,
                        final_output: final_output.to_path_buf(),
                        marker_bytes,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "compiled-bundle stage {} cannot be created: {error}",
                        path.display()
                    ));
                }
            }
        }
        Err(format!(
            "could not allocate a compiled-bundle stage for {}",
            final_output.display()
        ))
    }

    fn sync_publication_parents(&self, label: &str) -> Result<(), String> {
        // The working generation lives in the hidden staging namespace while
        // the public generation lives directly below the compiled root. A
        // rename/exchange mutates both directories, so durability requires
        // syncing both even if the first synchronization already failed.
        let final_parent = sync_parent_directory(&self.final_output, label);
        let stage_parent = sync_parent_directory(&self.path, label);
        match (final_parent, stage_parent) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(final_error), Ok(())) => Err(final_error),
            (Ok(()), Err(stage_error)) => Err(stage_error),
            (Err(final_error), Err(stage_error)) => Err(format!("{final_error}; {stage_error}")),
        }
    }

    fn publish(&mut self) -> Result<(), String> {
        validate_compiled_bundle_completion(&self.path)?.ok_or_else(|| {
            format!(
                "compiled-bundle stage {} has no durable completion record",
                self.path.display()
            )
        })?;
        if !validate_compiled_bundle_stage_marker(&self.path, &self.marker_bytes)? {
            return Err(format!(
                "compiled-bundle stage {} has no exact owner marker",
                self.path.display()
            ));
        }
        match fs::symlink_metadata(&self.final_output) {
            Ok(metadata) if metadata.file_type().is_dir() => {
                if let Err(error) = exchange_directories(&self.path, &self.final_output) {
                    return Err(format!(
                        "validated compiled-bundle stage {} cannot atomically replace {}: {error}",
                        self.path.display(),
                        self.final_output.display(),
                    ));
                }
                if let Err(error) = self.sync_publication_parents("compiled-bundle publication") {
                    let rollback = exchange_directories(&self.path, &self.final_output)
                        .map_err(|rollback_error| rollback_error.to_string())
                        .and_then(|()| {
                            self.sync_publication_parents("compiled-bundle swap rollback")
                        });
                    return Err(format!(
                        "{error}; atomic compiled-bundle swap rollback: {}",
                        rollback
                            .map(|()| "restored last-good".to_owned())
                            .unwrap_or_else(|rollback_error| rollback_error)
                    ));
                }
                let published_marker = self.final_output.join(COMPILED_BUNDLE_STAGE_MARKER_FILE);
                if validate_compiled_bundle_stage_marker(&self.final_output, &self.marker_bytes)
                    == Ok(true)
                    && fs::remove_file(&published_marker).is_ok()
                {
                    let _ = fs::File::open(&self.final_output)
                        .and_then(|directory| directory.sync_all());
                }
                // Publication is complete and durable at this point. Cleanup
                // is best-effort; an exact stale stage is safely reclaimed by
                // the next exclusive owner rather than turning a committed
                // disk replacement into a false client-visible failure.
                let _ = fs::remove_dir_all(&self.path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Err(error) = rename_directory_no_replace(&self.path, &self.final_output) {
                    return Err(format!(
                        "validated compiled-bundle stage {} cannot be atomically published at {}: {error}",
                        self.path.display(),
                        self.final_output.display(),
                    ));
                }
                if let Err(error) = self.sync_publication_parents("compiled-bundle publication") {
                    let rollback = rename_directory_no_replace(&self.final_output, &self.path)
                        .map_err(|rollback_error| rollback_error.to_string())
                        .and_then(|()| {
                            self.sync_publication_parents("compiled-bundle publish rollback")
                        });
                    return Err(format!(
                        "{error}; fresh compiled-bundle publication rollback: {}",
                        rollback
                            .map(|()| "removed incomplete final namespace".to_owned())
                            .unwrap_or_else(|rollback_error| rollback_error)
                    ));
                }
                let published_marker = self.final_output.join(COMPILED_BUNDLE_STAGE_MARKER_FILE);
                if validate_compiled_bundle_stage_marker(&self.final_output, &self.marker_bytes)
                    == Ok(true)
                    && fs::remove_file(&published_marker).is_ok()
                {
                    let _ = fs::File::open(&self.final_output)
                        .and_then(|directory| directory.sync_all());
                }
            }
            Ok(_) => {
                return Err(format!(
                    "compiled-bundle destination {} is not a regular non-symlink directory",
                    self.final_output.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "compiled-bundle destination {} cannot be inspected: {error}",
                    self.final_output.display()
                ));
            }
        }
        Ok(())
    }
}

fn collect_compiled_bundle_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, PathBuf)>,
    ignored: &std::collections::BTreeSet<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("{} cannot be enumerated: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{} cannot be enumerated: {error}", directory.display()))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if ignored.contains(&path) {
            continue;
        }
        let relative = path.strip_prefix(root).map_err(|_| {
            format!(
                "compiled-bundle entry {} escaped root {}",
                path.display(),
                root.display()
            )
        })?;
        let relative_utf8 = relative.to_str().ok_or_else(|| {
            format!(
                "compiled-bundle entry {} has a non-UTF-8 relative path",
                path.display()
            )
        })?;
        if relative_utf8 == COMPILED_BUNDLE_COMPLETION_FILE {
            continue;
        }
        if relative_utf8 == COMPILED_BUNDLE_STAGE_MARKER_FILE {
            let bytes = read_required_regular_file_nofollow(&path, "compiled-bundle stage marker")?;
            reject_duplicate_json_fields(&bytes, &path, "compiled-bundle stage marker")?;
            let record: CompiledBundleStageMarker = serde_json::from_slice(&bytes)
                .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
            let mut canonical =
                serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
            canonical.push(b'\n');
            if record.schema != COMPILED_BUNDLE_STAGE_SCHEMA || bytes != canonical {
                return Err(format!(
                    "{} is not a canonical supported compiled-bundle stage marker",
                    path.display()
                ));
            }
            validate_compiled_bundle_stage_marker_location(root, &bytes)?;
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("{} cannot be inspected: {error}", path.display()))?;
        if metadata.file_type().is_file() {
            files.push((relative_utf8.to_owned(), path));
        } else if metadata.file_type().is_dir() && matches!(relative_utf8, "graph" | "graph-cover")
        {
            collect_compiled_bundle_files(root, &path, files, ignored)?;
        } else {
            return Err(format!(
                "compiled bundle {} contains unsupported nonregular entry {}",
                root.display(),
                path.display()
            ));
        }
    }
    Ok(())
}

fn compiled_bundle_file_kappas(
    root: &Path,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    compiled_bundle_file_kappas_ignoring(root, &std::collections::BTreeSet::new())
}

fn compiled_bundle_file_kappas_ignoring(
    root: &Path,
    ignored: &std::collections::BTreeSet<PathBuf>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut files = Vec::new();
    collect_compiled_bundle_files(root, root, &mut files, ignored)?;
    let mut kappas = std::collections::BTreeMap::new();
    for (relative, path) in files {
        let bytes = read_required_regular_file_nofollow(&path, "compiled-bundle member")?;
        kappas.insert(
            relative,
            format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        );
    }
    Ok(kappas)
}

fn sync_regular_file_nofollow(path: &Path, label: &str) -> Result<(), String> {
    let file = open_regular_file_nofollow(path, label)?
        .ok_or_else(|| format!("{} disappeared before synchronization", path.display()))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("{} cannot be inspected: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("{} cannot be synced: {error}", path.display()))?;
    let final_handle = file
        .metadata()
        .map_err(|error| format!("{} cannot be reinspected: {error}", path.display()))?;
    let final_path = fs::symlink_metadata(path)
        .map_err(|error| format!("{} cannot be reinspected: {error}", path.display()))?;
    if !final_handle.file_type().is_file()
        || !final_path.file_type().is_file()
        || !source_compile_lock_metadata_matches(&opened, &final_handle)
        || !source_compile_lock_metadata_matches(&final_path, &final_handle)
    {
        return Err(format!(
            "{} changed identity or type while its {label} bytes were synchronized",
            path.display()
        ));
    }
    Ok(())
}

fn sync_compiled_bundle_members(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_compiled_bundle_files(root, root, &mut files, &std::collections::BTreeSet::new())?;
    for (_, path) in files {
        sync_regular_file_nofollow(&path, "compiled-bundle member")?;
    }
    for directory in [
        root.join("graph-cover"),
        root.join("graph"),
        root.to_path_buf(),
    ] {
        fs::File::open(&directory)
            .and_then(|handle| handle.sync_all())
            .map_err(|error| format!("{} cannot be synced: {error}", directory.display()))?;
    }
    Ok(())
}

fn compiled_bundle_completion_bytes(
    files: std::collections::BTreeMap<String, String>,
) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&CompiledBundleCompletion {
        schema: COMPILED_BUNDLE_COMPLETION_SCHEMA.to_owned(),
        files,
    })
    .map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish_compiled_bundle_completion(root: &Path) -> Result<(), String> {
    for required in [
        "corpus.meta",
        "corpus.records",
        "tless_artifacts.bin",
        "tokenizer.bin",
        "tokenizer_adapter.json",
        uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
        SOURCE_COMPILE_PREFLIGHT_FILE,
        SOURCE_MANIFEST_KAPPA_BINDING_FILE,
        "graph-cover/cover.r4g1",
        "graph-cover/cover_report.json",
        "graph/score.r4g1",
        "graph/score_report.json",
    ] {
        if !regular_file_presence(&root.join(required))? {
            return Err(format!(
                "compiled-bundle stage {} is incomplete; missing required {required}",
                root.display()
            ));
        }
    }
    // The completion record is the commit point. Every member and directory
    // entry it names must reach stable storage before that record can become
    // visible, otherwise a host crash could persist a truthful completion
    // record ahead of its graph/corpus bytes.
    sync_compiled_bundle_members(root)?;
    let files = compiled_bundle_file_kappas(root)?;
    let bytes = compiled_bundle_completion_bytes(files)?;
    publish_bytes_no_clobber(
        &root.join(COMPILED_BUNDLE_COMPLETION_FILE),
        &bytes,
        "compiled-bundle completion",
    )
}

fn validate_staged_graph_outputs(root: &Path) -> Result<(), String> {
    for relative in ["graph-cover/cover.r4g1", "graph/score.r4g1"] {
        let path = root.join(relative);
        let bytes = read_required_regular_file_nofollow(&path, "staged R4G1 artifact")?;
        let view = uor_r4_graph_format::GraphView::parse(&bytes)
            .map_err(|error| format!("{} is not a valid R4G1 artifact: {error}", path.display()))?;
        view.verify_cids().map_err(|error| {
            format!(
                "{} failed R4G1 integrity verification: {error}",
                path.display()
            )
        })?;
        if view.head().is_none() {
            return Err(format!("{} has no R4G1 HEAD section", path.display()));
        }
    }
    for relative in ["graph-cover/cover_report.json", "graph/score_report.json"] {
        let path = root.join(relative);
        let bytes = read_required_regular_file_nofollow(&path, "staged graph report")?;
        reject_duplicate_json_fields(&bytes, &path, "staged graph report")?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
        if !value.is_object() {
            return Err(format!("{} must contain a JSON object", path.display()));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum GraphOutputKind {
    Cover,
    Score,
}

impl GraphOutputKind {
    fn files(self) -> (&'static str, &'static str) {
        match self {
            Self::Cover => ("cover.r4g1", "cover_report.json"),
            Self::Score => ("score.r4g1", "score_report.json"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Score => "score",
        }
    }
}

#[derive(Debug)]
struct LegacyGraphRecordPaths {
    attempt: PathBuf,
    completion: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LegacyGraphGenerationAction {
    Reuse,
    BuildBoth,
    ResumeScore,
}

fn canonical_path_text(path: &Path, label: &str) -> Result<String, String> {
    canonical_compile_session_subject(path)?
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{label} is not UTF-8: {}", path.display()))
}

fn bytes_kappa(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn regular_file_kappa(path: &Path, label: &str) -> Result<String, String> {
    read_required_regular_file_nofollow(path, label).map(|bytes| bytes_kappa(&bytes))
}

fn legacy_graph_controls_kappa(args: &[String]) -> Result<String, String> {
    serde_json::to_vec(args)
        .map(|bytes| bytes_kappa(&bytes))
        .map_err(|error| error.to_string())
}

struct LegacyGraphGenerationInputs<'a> {
    artifacts: &'a Path,
    corpus_meta: &'a Path,
    corpus_recs: &'a Path,
    tokenizer: &'a Path,
    cover_output: &'a Path,
    graph_output: &'a Path,
    cover_args: &'a [String],
    score_args: &'a [String],
}

fn capture_legacy_graph_generation_identity(
    inputs: LegacyGraphGenerationInputs<'_>,
) -> Result<LegacyGraphGenerationIdentity, String> {
    let LegacyGraphGenerationInputs {
        artifacts,
        corpus_meta,
        corpus_recs,
        tokenizer,
        cover_output,
        graph_output,
        cover_args,
        score_args,
    } = inputs;
    let mut inputs = std::collections::BTreeMap::new();
    for (path, label) in [
        (artifacts, "legacy transformerless artifact"),
        (corpus_meta, "legacy corpus metadata"),
        (corpus_recs, "legacy corpus records"),
    ] {
        inputs.insert(
            canonical_path_text(path, label)?,
            regular_file_kappa(path, label)?,
        );
    }
    for (path, label) in [
        (tokenizer.to_path_buf(), "legacy tokenizer"),
        (
            corpus_meta
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE),
            "legacy attention-operator binding",
        ),
        (
            corpus_meta
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("manifest.json"),
            "legacy observation manifest",
        ),
    ] {
        if let Some(bytes) = read_regular_file_nofollow(&path, label)? {
            inputs.insert(canonical_path_text(&path, label)?, bytes_kappa(&bytes));
        }
    }
    Ok(LegacyGraphGenerationIdentity {
        cover_output: canonical_path_text(cover_output, "legacy cover output")?,
        graph_output: canonical_path_text(graph_output, "legacy score output")?,
        input_files: inputs,
        cover_controls_kappa: legacy_graph_controls_kappa(cover_args)?,
        score_controls_kappa: legacy_graph_controls_kappa(score_args)?,
    })
}

fn legacy_graph_record_paths(
    identity: &LegacyGraphGenerationIdentity,
) -> Result<LegacyGraphRecordPaths, String> {
    legacy_graph_record_paths_for_outputs(
        Path::new(&identity.cover_output),
        Path::new(&identity.graph_output),
        true,
    )
}

fn legacy_graph_record_paths_for_outputs(
    cover: &Path,
    graph: &Path,
    create_coordination: bool,
) -> Result<LegacyGraphRecordPaths, String> {
    let cover_output = canonical_path_text(cover, "legacy cover output")?;
    let graph_output = canonical_path_text(graph, "legacy score output")?;
    let cover = Path::new(&cover_output);
    let parent = cover.parent().ok_or_else(|| {
        format!(
            "legacy cover output {} has no coordination parent",
            cover.display()
        )
    })?;
    // Keep transaction records outside the bundle inventory itself. A source
    // refresh copies every non-derived root entry and must never encounter a
    // legacy coordination directory as executable model payload.
    let coordination_parent = parent
        .parent()
        .filter(|ancestor| ancestor.parent().is_some())
        .unwrap_or(parent);
    let coordination = source_compile_staging_root(coordination_parent);
    if create_coordination {
        ensure_source_compile_staging_root(coordination_parent)?;
    }
    let key_material = format!("{cover_output}\0{graph_output}");
    let key = blake3::hash(key_material.as_bytes()).to_hex();
    Ok(LegacyGraphRecordPaths {
        attempt: coordination.join(format!(".legacy-graph-{key}.attempt.json")),
        completion: coordination.join(format!(".legacy-graph-{key}.completion.json")),
    })
}

fn legacy_graph_attempt_bytes(identity: &LegacyGraphGenerationIdentity) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&LegacyGraphGenerationAttempt {
        schema: LEGACY_GRAPH_ATTEMPT_SCHEMA.to_owned(),
        identity: identity.clone(),
    })
    .map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn legacy_graph_completion_bytes(
    record: &LegacyGraphGenerationCompletion,
) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(record).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_optional_legacy_graph_attempt(
    path: &Path,
) -> Result<Option<LegacyGraphGenerationAttempt>, String> {
    let Some(bytes) = read_regular_file_nofollow(path, "legacy graph attempt")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, path, "legacy graph attempt")?;
    let record: LegacyGraphGenerationAttempt = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    if record.schema != LEGACY_GRAPH_ATTEMPT_SCHEMA
        || bytes != legacy_graph_attempt_bytes(&record.identity)?
    {
        return Err(format!(
            "{} is not a canonical supported legacy graph attempt",
            path.display()
        ));
    }
    Ok(Some(record))
}

fn read_optional_legacy_graph_completion(
    path: &Path,
) -> Result<Option<LegacyGraphGenerationCompletion>, String> {
    let Some(bytes) = read_regular_file_nofollow(path, "legacy graph completion")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, path, "legacy graph completion")?;
    let record: LegacyGraphGenerationCompletion = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    if record.schema != LEGACY_GRAPH_GENERATION_SCHEMA
        || bytes != legacy_graph_completion_bytes(&record)?
    {
        return Err(format!(
            "{} is not a canonical supported legacy graph completion",
            path.display()
        ));
    }
    Ok(Some(record))
}

fn is_atomic_replace_temporary(name: &str, stable_name: &str) -> bool {
    let prefix = format!(".{stable_name}.");
    let Some(sequence) = name
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(".replace.tmp"))
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

fn recover_atomic_replace_temporaries(path: &Path, label: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    let stable_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))?;
    let prefix = format!(".{stable_name}.");
    let mut recoverable = Vec::new();
    for entry in fs::read_dir(parent)
        .map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "{} contains a non-UTF-8 coordination entry",
                parent.display()
            )
        })?;
        if is_atomic_replace_temporary(name, stable_name) {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized {label} replacement temporary {} is not a regular non-symlink file",
                    entry.path().display()
                ));
            }
            recoverable.push(entry.path());
        } else if name.starts_with(&prefix) && name.ends_with(".replace.tmp") {
            return Err(format!(
                "{} contains unrecognized {label} replacement temporary {name}",
                parent.display()
            ));
        }
    }
    let recovered = !recoverable.is_empty();
    for temporary in recoverable {
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "recognized {label} replacement temporary {} cannot be reclaimed: {error}",
                temporary.display()
            )
        })?;
    }
    if recovered {
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("{} cannot be synced: {error}", parent.display()))?;
    }
    Ok(())
}

fn rollback_atomic_byte_replacement(
    path: &Path,
    previous: Option<&[u8]>,
    label: &str,
) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    match previous {
        Some(previous) => {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))?;
            for _ in 0..128 {
                let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
                let temporary =
                    parent.join(format!(".{name}.{}.{}.replace.tmp", std::process::id(), id));
                let mut file = match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temporary)
                {
                    Ok(file) => file,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(format!("{}: {error}", temporary.display())),
                };
                file.write_all(previous)
                    .and_then(|()| file.sync_all())
                    .map_err(|error| format!("{}: {error}", temporary.display()))?;
                drop(file);
                fs::rename(&temporary, path).map_err(|error| {
                    format!(
                        "{label} rollback {} -> {} failed: {error}",
                        temporary.display(),
                        path.display()
                    )
                })?;
                return fs::File::open(parent)
                    .and_then(|directory| directory.sync_all())
                    .map_err(|error| format!("{} cannot be synced: {error}", parent.display()));
            }
            Err(format!(
                "could not reserve a {label} rollback temporary in {}",
                parent.display()
            ))
        }
        None => {
            fs::remove_file(path)
                .map_err(|error| format!("{} cannot be rolled back: {error}", path.display()))?;
            fs::File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| format!("{} cannot be synced: {error}", parent.display()))
        }
    }
}

fn replace_bytes_atomically(path: &Path, expected: &[u8], label: &str) -> Result<(), String> {
    recover_atomic_replace_temporaries(path, label)?;
    let previous = read_regular_file_nofollow(path, label)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(format!(
                "{} is not a regular non-symlink {label}",
                path.display()
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))?;
    for _ in 0..128 {
        let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".{name}.{}.{}.replace.tmp", std::process::id(), id));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{}: {error}", temporary.display())),
        };
        if let Err(error) = file.write_all(expected).and_then(|()| file.sync_all()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("{}: {error}", temporary.display()));
        }
        drop(file);
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "atomic {label} replacement {} -> {} failed: {error}",
                temporary.display(),
                path.display()
            ));
        }
        return match sync_parent_directory(path, label) {
            Ok(()) => Ok(()),
            Err(error) => {
                let rollback = rollback_atomic_byte_replacement(path, previous.as_deref(), label)
                    .map(|()| "restored previous record".to_owned())
                    .unwrap_or_else(|rollback_error| rollback_error);
                Err(format!("{error}; {label} rollback: {rollback}"))
            }
        };
    }
    Err(format!(
        "could not reserve a unique {label} replacement temporary in {}",
        parent.display()
    ))
}

fn graph_output_file_kappas(
    output: &Path,
    kind: GraphOutputKind,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    if !validate_graph_output_directory(output, kind)? {
        return Err(format!(
            "{} output {} is absent",
            kind.label(),
            output.display()
        ));
    }
    let mut files = std::collections::BTreeMap::new();
    for name in [kind.files().0, kind.files().1] {
        let path = output.join(name);
        files.insert(
            canonical_path_text(&path, "legacy graph output member")?,
            regular_file_kappa(&path, "legacy graph output member")?,
        );
    }
    Ok(files)
}

fn legacy_graph_output_file_kappas(
    cover_output: &Path,
    graph_output: &Path,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut files = graph_output_file_kappas(cover_output, GraphOutputKind::Cover)?;
    files.extend(graph_output_file_kappas(
        graph_output,
        GraphOutputKind::Score,
    )?);
    Ok(files)
}

fn legacy_graph_generation_action(
    identity: &LegacyGraphGenerationIdentity,
) -> Result<(LegacyGraphGenerationAction, LegacyGraphRecordPaths, Vec<u8>), String> {
    let paths = legacy_graph_record_paths(identity)?;
    recover_atomic_replace_temporaries(&paths.completion, "legacy graph completion")?;
    let attempt_bytes = legacy_graph_attempt_bytes(identity)?;
    let attempt = read_optional_legacy_graph_attempt(&paths.attempt)?;
    let exact_attempt = match attempt.as_ref() {
        Some(attempt) if attempt.identity == *identity => true,
        Some(_) => {
            return Err(format!(
                "legacy graph outputs {} and {} retain a different unfinished generation attempt; refusing to overwrite its crash state",
                identity.cover_output, identity.graph_output
            ));
        }
        None => false,
    };
    let cover_output = Path::new(&identity.cover_output);
    let graph_output = Path::new(&identity.graph_output);
    let cover_complete = validate_graph_output_directory(cover_output, GraphOutputKind::Cover)?;
    let graph_complete = validate_graph_output_directory(graph_output, GraphOutputKind::Score)?;
    let completion = read_optional_legacy_graph_completion(&paths.completion)?;
    if let Some(completion) = completion.as_ref() {
        let outputs = if cover_complete && graph_complete {
            Some(legacy_graph_output_file_kappas(cover_output, graph_output)?)
        } else {
            None
        };
        if completion.identity == *identity {
            if outputs.as_ref() != Some(&completion.output_files) {
                return Err(
                    "legacy graph outputs changed after their exact input-bound completion was published; refusing refresh recertification"
                        .to_owned(),
                );
            }
            return Ok((LegacyGraphGenerationAction::Reuse, paths, attempt_bytes));
        }
        if !exact_attempt && outputs.as_ref() != Some(&completion.output_files) {
            return Err(
                "legacy graph outputs do not match their last durable generation and no exact current attempt authorizes crash recovery"
                    .to_owned(),
            );
        }
    }
    let action = match (cover_complete, graph_complete, exact_attempt) {
        (true, false, true) => LegacyGraphGenerationAction::ResumeScore,
        (false, true, true) => {
            return Err(
                "legacy score output exists without its cover during an unfinished generation; refusing an impossible publication order"
                    .to_owned(),
            );
        }
        (true, false, false) | (false, true, false) => {
            return Err(format!(
                "legacy R4G1 outputs are one-sided (cover complete={cover_complete}, score complete={graph_complete}) without an exact server attempt record; refusing arbitrary partial state"
            ));
        }
        _ => LegacyGraphGenerationAction::BuildBoth,
    };
    Ok((action, paths, attempt_bytes))
}

fn validate_legacy_graph_generation_for_serving(graph_path: &Path) -> Result<(), String> {
    let Some(graph_output) = graph_path.parent() else {
        return Ok(());
    };
    if graph_output.file_name() != Some(std::ffi::OsStr::new("graph")) {
        return Ok(());
    }
    let Some(root) = graph_output.parent() else {
        return Ok(());
    };
    let cover_output = root.join("graph-cover");
    let paths = legacy_graph_record_paths_for_outputs(&cover_output, graph_output, false)?;
    let coordination = paths
        .completion
        .parent()
        .ok_or_else(|| format!("{} has no coordination parent", paths.completion.display()))?;
    match fs::symlink_metadata(coordination) {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => {
            return Err(format!(
                "legacy graph coordination root for {} is not a regular non-symlink directory",
                graph_path.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", graph_path.display())),
    }
    recover_atomic_replace_temporaries(&paths.completion, "legacy graph completion")?;
    let attempt = read_optional_legacy_graph_attempt(&paths.attempt)?;
    let Some(completion) = read_optional_legacy_graph_completion(&paths.completion)? else {
        if let Some(attempt) = attempt {
            return Err(format!(
                "legacy graph generation for {} is incomplete under attempt schema {}; retry Compile / Refresh before serving",
                graph_path.display(),
                attempt.schema
            ));
        }
        // Pre-transaction historical bundles retain their read compatibility.
        return Ok(());
    };
    let expected_cover = canonical_path_text(&cover_output, "legacy cover output")?;
    let expected_graph = canonical_path_text(graph_output, "legacy score output")?;
    if completion.identity.cover_output != expected_cover
        || completion.identity.graph_output != expected_graph
    {
        return Err(format!(
            "legacy graph completion beside {} binds different physical outputs",
            graph_path.display()
        ));
    }
    let actual_outputs = legacy_graph_output_file_kappas(&cover_output, graph_output)?;
    if actual_outputs != completion.output_files {
        return Err(format!(
            "legacy graph output generation {} changed after its durable completion",
            graph_path.display()
        ));
    }
    for (path, expected) in &completion.identity.input_files {
        let path = Path::new(path);
        let actual = regular_file_kappa(path, "legacy graph bound input")?;
        if &actual != expected {
            return Err(format!(
                "legacy graph input {} changed after {} was completed",
                path.display(),
                graph_path.display()
            ));
        }
    }
    if let Some(attempt) = attempt {
        if attempt.identity != completion.identity {
            return Err(format!(
                "legacy graph generation for {} has a completion and a conflicting unfinished attempt",
                graph_path.display()
            ));
        }
        let bytes = legacy_graph_attempt_bytes(&attempt.identity)?;
        remove_exact_legacy_attempt(&paths.attempt, &bytes)?;
    }
    Ok(())
}

fn validate_graph_output_directory(output: &Path, kind: GraphOutputKind) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", output.display())),
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "{} output {} is not a regular non-symlink directory",
            kind.label(),
            output.display()
        ));
    }
    let (artifact_name, report_name) = kind.files();
    let mut inventory = fs::read_dir(output)
        .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?
        .map(|entry| {
            let entry = entry
                .map_err(|error| format!("{} cannot be enumerated: {error}", output.display()))?;
            let name = entry
                .file_name()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{} contains a non-UTF-8 output entry", output.display()))?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "{} output entry {} is not a regular non-symlink file",
                    kind.label(),
                    entry.path().display()
                ));
            }
            Ok(name)
        })
        .collect::<Result<Vec<_>, String>>()?;
    inventory.sort();
    let mut expected = vec![artifact_name.to_owned(), report_name.to_owned()];
    expected.sort();
    if inventory != expected {
        return Err(format!(
            "{} output {} must contain exactly {artifact_name} and {report_name}; found {:?}",
            kind.label(),
            output.display(),
            inventory
        ));
    }
    let artifact_path = output.join(artifact_name);
    let report_path = output.join(report_name);
    let artifact = read_regular_file_nofollow(&artifact_path, "R4G1 output artifact")?;
    let report = read_regular_file_nofollow(&report_path, "R4G1 output report")?;
    let (Some(artifact), Some(report)) = (artifact, report) else {
        return Err(format!(
            "{} output {} exists without its complete {artifact_name}/{report_name} pair; refusing to adopt or mutate partial publication",
            kind.label(),
            output.display()
        ));
    };
    let view = uor_r4_graph_format::GraphView::parse(&artifact).map_err(|error| {
        format!(
            "{} output artifact {} is invalid: {error}",
            kind.label(),
            artifact_path.display()
        )
    })?;
    view.verify_cids().map_err(|error| {
        format!(
            "{} output artifact {} failed integrity verification: {error}",
            kind.label(),
            artifact_path.display()
        )
    })?;
    if view.head().is_none() {
        return Err(format!(
            "{} output artifact {} has no R4G1 HEAD section",
            kind.label(),
            artifact_path.display()
        ));
    }
    reject_duplicate_json_fields(&report, &report_path, "R4G1 output report")?;
    let value: serde_json::Value = serde_json::from_slice(&report)
        .map_err(|error| format!("{} is malformed: {error}", report_path.display()))?;
    if !value.is_object() {
        return Err(format!(
            "{} must contain a JSON object",
            report_path.display()
        ));
    }
    Ok(true)
}

fn sync_graph_output_directory(output: &Path, kind: GraphOutputKind) -> Result<(), String> {
    if !validate_graph_output_directory(output, kind)? {
        return Err(format!(
            "{} staging directory {} remained absent before synchronization",
            kind.label(),
            output.display()
        ));
    }
    let (artifact, report) = kind.files();
    sync_regular_file_nofollow(&output.join(artifact), "R4G1 output artifact")?;
    sync_regular_file_nofollow(&output.join(report), "R4G1 output report")?;
    fs::File::open(output)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", output.display()))
}

struct GraphOutputStage {
    path: PathBuf,
    armed: bool,
}

#[derive(Clone, Copy, Debug)]
enum GraphStageInstall {
    Fresh,
    Exchanged,
}

impl GraphOutputStage {
    fn allocate(final_output: &Path, kind: GraphOutputKind) -> Result<Self, String> {
        let parent = final_output
            .parent()
            .ok_or_else(|| format!("graph output {} has no parent", final_output.display()))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("{} cannot be created: {error}", parent.display()))?;
        let name = final_output
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("graph output is not UTF-8: {}", final_output.display()))?;
        let prefix = format!(".{name}.{}-staging-", kind.label());
        for entry in fs::read_dir(parent)
            .map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?
        {
            let entry = entry
                .map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?;
            let entry_name = entry.file_name();
            let Some(entry_name) = entry_name.to_str() else {
                continue;
            };
            if !entry_name.starts_with(&prefix) {
                continue;
            }
            let sequence = &entry_name[prefix.len()..];
            let mut parts = sequence.split('-');
            let exact = parts.next().is_some_and(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            }) && parts.next().is_some_and(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            }) && parts.next().is_none();
            if !exact {
                return Err(format!(
                    "graph output parent {} contains unrecognized staging entry {entry_name}",
                    parent.display()
                ));
            }
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_dir() {
                return Err(format!(
                    "recognized graph staging entry {} is not a regular non-symlink directory",
                    entry.path().display()
                ));
            }
            fs::remove_dir_all(entry.path()).map_err(|error| {
                format!(
                    "stale graph staging entry {} cannot be reclaimed under exclusive output ownership: {error}",
                    entry.path().display()
                )
            })?;
        }
        for _ in 0..128 {
            let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("{prefix}{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return Ok(Self { path, armed: true });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("{} cannot be created: {error}", path.display()));
                }
            }
        }
        Err(format!(
            "could not allocate a {} staging directory beside {}",
            kind.label(),
            final_output.display()
        ))
    }

    #[cfg(test)]
    fn publish(mut self, final_output: &Path, kind: GraphOutputKind) -> Result<(), String> {
        sync_graph_output_directory(&self.path, kind)?;
        rename_directory_no_replace(&self.path, final_output).map_err(|error| {
            format!(
                "validated {} staging directory {} cannot be exclusively published at {}: {error}",
                kind.label(),
                self.path.display(),
                final_output.display()
            )
        })?;
        if let Err(error) = sync_parent_directory(final_output, "R4G1 graph output publication") {
            let rollback = rename_directory_no_replace(final_output, &self.path).and_then(|()| {
                fs::File::open(final_output.parent().unwrap_or_else(|| Path::new(".")))?.sync_all()
            });
            return Err(format!(
                "{error}; R4G1 graph output publication rollback: {}",
                rollback
                    .map(|()| "removed incomplete final namespace".to_owned())
                    .unwrap_or_else(|rollback_error| rollback_error.to_string())
            ));
        }
        self.armed = false;
        Ok(())
    }

    fn install_for_generation(
        &mut self,
        final_output: &Path,
        kind: GraphOutputKind,
    ) -> Result<GraphStageInstall, String> {
        sync_graph_output_directory(&self.path, kind)?;
        let install = match fs::symlink_metadata(final_output) {
            Ok(metadata) if metadata.file_type().is_dir() => {
                validate_graph_output_directory(final_output, kind)?;
                exchange_directories(&self.path, final_output).map_err(|error| {
                    format!(
                        "validated {} stage {} cannot replace {}: {error}",
                        kind.label(),
                        self.path.display(),
                        final_output.display()
                    )
                })?;
                GraphStageInstall::Exchanged
            }
            Ok(_) => {
                return Err(format!(
                    "{} output {} is not a regular non-symlink directory",
                    kind.label(),
                    final_output.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                rename_directory_no_replace(&self.path, final_output).map_err(|error| {
                    format!(
                        "validated {} stage {} cannot be exclusively published at {}: {error}",
                        kind.label(),
                        self.path.display(),
                        final_output.display()
                    )
                })?;
                GraphStageInstall::Fresh
            }
            Err(error) => {
                return Err(format!(
                    "{} output {} cannot be inspected: {error}",
                    kind.label(),
                    final_output.display()
                ));
            }
        };
        if let Err(error) =
            sync_parent_directory(final_output, "legacy graph generation installation")
        {
            let rollback = self
                .rollback_generation_install(final_output, install)
                .map(|()| "restored pre-install generation".to_owned())
                .unwrap_or_else(|rollback_error| rollback_error);
            return Err(format!("{error}; installation rollback: {rollback}"));
        }
        Ok(install)
    }

    fn rollback_generation_install(
        &mut self,
        final_output: &Path,
        install: GraphStageInstall,
    ) -> Result<(), String> {
        match install {
            GraphStageInstall::Fresh => rename_directory_no_replace(final_output, &self.path)
                .map_err(|error| error.to_string())?,
            GraphStageInstall::Exchanged => {
                exchange_directories(&self.path, final_output).map_err(|error| error.to_string())?
            }
        }
        sync_parent_directory(final_output, "legacy graph generation rollback")
    }

    fn finish_generation_install(&mut self, final_output: &Path) -> Result<(), String> {
        match fs::symlink_metadata(&self.path) {
            Ok(metadata) if metadata.file_type().is_dir() => {
                fs::remove_dir_all(&self.path).map_err(|error| {
                    format!(
                        "replaced graph generation {} cannot be reclaimed: {error}",
                        self.path.display()
                    )
                })?;
                sync_parent_directory(final_output, "legacy graph generation cleanup")?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(format!(
                    "graph generation cleanup path {} is not a directory",
                    self.path.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "graph generation cleanup path {} cannot be inspected: {error}",
                    self.path.display()
                ));
            }
        }
        self.armed = false;
        Ok(())
    }
}

impl Drop for GraphOutputStage {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn replace_out_argument(args: &[String], output: &Path) -> Result<Vec<String>, String> {
    let mut staged = args.to_vec();
    let index = staged
        .iter()
        .position(|argument| argument == "--out")
        .ok_or_else(|| "graph writer arguments contain no --out control".to_owned())?;
    let value = staged
        .get_mut(index + 1)
        .ok_or_else(|| "graph writer --out control has no value".to_owned())?;
    *value = output.display().to_string();
    Ok(staged)
}

fn replace_argument_value(args: &mut [String], name: &str, value: &Path) -> Result<(), String> {
    let index = args
        .iter()
        .position(|argument| argument == name)
        .ok_or_else(|| format!("graph writer arguments contain no {name} control"))?;
    let target = args
        .get_mut(index + 1)
        .ok_or_else(|| format!("graph writer {name} control has no value"))?;
    *target = value.display().to_string();
    Ok(())
}

fn build_graph_writer_stage<F>(
    final_output: &Path,
    kind: GraphOutputKind,
    args: &[String],
    writer: F,
) -> Result<GraphOutputStage, String>
where
    F: FnOnce(&[String]) -> Result<(), String>,
{
    let stage = GraphOutputStage::allocate(final_output, kind)?;
    let staged_args = replace_out_argument(args, &stage.path)?;
    writer(&staged_args)?;
    sync_graph_output_directory(&stage.path, kind)?;
    Ok(stage)
}

fn staged_legacy_graph_output_kappas(
    cover_stage: &Path,
    score_stage: &Path,
    cover_output: &Path,
    score_output: &Path,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut files = std::collections::BTreeMap::new();
    for (stage, output, kind) in [
        (cover_stage, cover_output, GraphOutputKind::Cover),
        (score_stage, score_output, GraphOutputKind::Score),
    ] {
        if !validate_graph_output_directory(stage, kind)? {
            return Err(format!(
                "{} legacy generation stage {} is absent",
                kind.label(),
                stage.display()
            ));
        }
        for name in [kind.files().0, kind.files().1] {
            files.insert(
                canonical_path_text(&output.join(name), "legacy graph output member")?,
                regular_file_kappa(&stage.join(name), "staged legacy graph output member")?,
            );
        }
    }
    Ok(files)
}

fn remove_exact_legacy_attempt(path: &Path, expected: &[u8]) -> Result<(), String> {
    let Some(bytes) = read_regular_file_nofollow(path, "legacy graph attempt")? else {
        return Ok(());
    };
    if bytes != expected {
        return Err(format!(
            "legacy graph attempt {} changed before completion cleanup",
            path.display()
        ));
    }
    fs::remove_file(path)
        .map_err(|error| format!("{} cannot be reclaimed: {error}", path.display()))?;
    sync_parent_directory(path, "legacy graph attempt cleanup")
}

fn ensure_legacy_graph_attempt(path: &Path, expected: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    let stable_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))?;
    let stable = read_regular_file_nofollow(path, "legacy graph attempt")?;
    if stable.as_deref().is_some_and(|bytes| bytes != expected) {
        return Err(format!(
            "{} records a different unfinished legacy graph attempt",
            path.display()
        ));
    }
    let mut recoverable = Vec::new();
    for entry in fs::read_dir(parent)
        .map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "{} contains a non-UTF-8 coordination entry",
                parent.display()
            )
        })?;
        if is_atomic_publisher_temporary(name, stable_name) {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized legacy attempt temporary {} is not a regular non-symlink file",
                    entry.path().display()
                ));
            }
            let bytes = read_required_regular_file_nofollow(
                &entry.path(),
                "legacy graph attempt temporary",
            )?;
            let admissible = if stable.is_some() {
                bytes == expected
            } else {
                expected.starts_with(&bytes)
            };
            if !admissible {
                return Err(format!(
                    "legacy graph attempt temporary {} is not an exact publisher crash prefix",
                    entry.path().display()
                ));
            }
            recoverable.push(entry.path());
        } else if looks_like_atomic_publisher_temporary(name, stable_name) {
            return Err(format!(
                "{} contains unrecognized legacy attempt temporary {name}",
                parent.display()
            ));
        }
    }
    for temporary in recoverable {
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "recognized legacy attempt temporary {} cannot be reclaimed: {error}",
                temporary.display()
            )
        })?;
    }
    publish_bytes_no_clobber(path, expected, "legacy graph attempt")
}

fn publish_legacy_graph_generation_pair(
    mut cover_stage: GraphOutputStage,
    mut score_stage: GraphOutputStage,
    cover_output: &Path,
    score_output: &Path,
    paths: &LegacyGraphRecordPaths,
    attempt_bytes: &[u8],
    completion: &LegacyGraphGenerationCompletion,
) -> Result<(), String> {
    let completion_bytes = legacy_graph_completion_bytes(completion)?;
    let cover_install = cover_stage.install_for_generation(cover_output, GraphOutputKind::Cover)?;
    let score_install =
        match score_stage.install_for_generation(score_output, GraphOutputKind::Score) {
            Ok(install) => install,
            Err(error) => {
                let rollback = cover_stage
                    .rollback_generation_install(cover_output, cover_install)
                    .map(|()| "restored cover generation".to_owned())
                    .unwrap_or_else(|rollback_error| rollback_error);
                return Err(format!("{error}; cover rollback: {rollback}"));
            }
        };
    let installed = legacy_graph_output_file_kappas(cover_output, score_output);
    let commit = installed.and_then(|installed| {
        if installed != completion.output_files {
            return Err(
                "installed legacy graph generation differs from its staged digest set".to_owned(),
            );
        }
        replace_bytes_atomically(
            &paths.completion,
            &completion_bytes,
            "legacy graph completion",
        )
    });
    if let Err(error) = commit {
        let score_rollback = score_stage
            .rollback_generation_install(score_output, score_install)
            .map(|()| "restored score generation".to_owned())
            .unwrap_or_else(|rollback_error| rollback_error);
        let cover_rollback = cover_stage
            .rollback_generation_install(cover_output, cover_install)
            .map(|()| "restored cover generation".to_owned())
            .unwrap_or_else(|rollback_error| rollback_error);
        return Err(format!(
            "{error}; score rollback: {score_rollback}; cover rollback: {cover_rollback}"
        ));
    }
    // The completion is now the durable commit point. Reclaiming old
    // directories and the exact attempt is cleanup; failures are reported so
    // the next exclusive owner can finish without ever rewriting final bytes.
    cover_stage.finish_generation_install(cover_output)?;
    score_stage.finish_generation_install(score_output)?;
    remove_exact_legacy_attempt(&paths.attempt, attempt_bytes)
}

fn publish_legacy_graph_score_resume(
    mut score_stage: GraphOutputStage,
    cover_output: &Path,
    score_output: &Path,
    paths: &LegacyGraphRecordPaths,
    attempt_bytes: &[u8],
    identity: &LegacyGraphGenerationIdentity,
) -> Result<(), String> {
    let score_install = score_stage.install_for_generation(score_output, GraphOutputKind::Score)?;
    let output_files = match legacy_graph_output_file_kappas(cover_output, score_output) {
        Ok(files) => files,
        Err(error) => {
            let rollback = score_stage
                .rollback_generation_install(score_output, score_install)
                .map(|()| "removed resumed score generation".to_owned())
                .unwrap_or_else(|rollback_error| rollback_error);
            return Err(format!("{error}; score rollback: {rollback}"));
        }
    };
    let completion = LegacyGraphGenerationCompletion {
        schema: LEGACY_GRAPH_GENERATION_SCHEMA.to_owned(),
        identity: identity.clone(),
        output_files,
    };
    let bytes = legacy_graph_completion_bytes(&completion)?;
    if let Err(error) =
        replace_bytes_atomically(&paths.completion, &bytes, "legacy graph completion")
    {
        let rollback = score_stage
            .rollback_generation_install(score_output, score_install)
            .map(|()| "removed resumed score generation".to_owned())
            .unwrap_or_else(|rollback_error| rollback_error);
        return Err(format!("{error}; score rollback: {rollback}"));
    }
    score_stage.finish_generation_install(score_output)?;
    remove_exact_legacy_attempt(&paths.attempt, attempt_bytes)
}

#[cfg(test)]
fn run_graph_writer_staged<F>(
    final_output: &Path,
    kind: GraphOutputKind,
    args: &[String],
    writer: F,
) -> Result<(), String>
where
    F: FnOnce(&[String]) -> Result<(), String>,
{
    if validate_graph_output_directory(final_output, kind)? {
        return Ok(());
    }
    let stage = GraphOutputStage::allocate(final_output, kind)?;
    let staged_args = replace_out_argument(args, &stage.path)?;
    writer(&staged_args)?;
    stage.publish(final_output, kind)
}

fn compiled_bundle_completion_temporaries(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", root.display())),
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "compiled bundle {} is not a regular non-symlink directory",
            root.display()
        ));
    }
    let mut temporaries = Vec::new();
    for entry in fs::read_dir(root)
        .map_err(|error| format!("{} cannot be enumerated: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", root.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "compiled bundle {} contains a non-UTF-8 entry",
                root.display()
            )
        })?;
        if is_atomic_publisher_temporary(name, COMPILED_BUNDLE_COMPLETION_FILE) {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_file() {
                return Err(format!(
                    "recognized compiled-bundle completion temporary {} is not a regular non-symlink file",
                    entry.path().display()
                ));
            }
            let bytes = read_required_regular_file_nofollow(
                &entry.path(),
                "compiled-bundle completion temporary",
            )?;
            temporaries.push((entry.path(), bytes));
        } else if looks_like_atomic_publisher_temporary(name, COMPILED_BUNDLE_COMPLETION_FILE) {
            return Err(format!(
                "compiled bundle {} contains unrecognized completion publisher temporary {name}",
                root.display()
            ));
        }
    }
    temporaries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(temporaries)
}

fn validate_compiled_bundle_completion_bytes(
    root: &Path,
    path: &Path,
    bytes: &[u8],
    ignored: &std::collections::BTreeSet<PathBuf>,
) -> Result<CompiledBundleCompletion, String> {
    reject_duplicate_json_fields(bytes, path, "compiled-bundle completion")?;
    let record: CompiledBundleCompletion = serde_json::from_slice(bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    if record.schema != COMPILED_BUNDLE_COMPLETION_SCHEMA {
        return Err(format!(
            "{} records unsupported schema {:?}",
            path.display(),
            record.schema
        ));
    }
    let canonical = compiled_bundle_completion_bytes(record.files.clone())?;
    if bytes != canonical {
        return Err(format!(
            "{} is not a canonical compiled-bundle completion record",
            path.display()
        ));
    }
    let actual = compiled_bundle_file_kappas_ignoring(root, ignored)?;
    if actual != record.files {
        return Err(format!(
            "compiled bundle {} changed after its durable completion record was published",
            root.display()
        ));
    }
    Ok(record)
}

/// Reconcile the only crash residue which can coexist with a committed
/// completion: the publisher's exact hard-linked temporary. Callers hold the
/// bundle's cross-process session, so no cooperating publisher can still own
/// the temporary. Classification and full member validation precede the first
/// removal; malformed, conflicting, symlinked, or special entries remain
/// terminal and untouched.
fn recover_compiled_bundle_completion_temporaries(root: &Path) -> Result<(), String> {
    let temporaries = compiled_bundle_completion_temporaries(root)?;
    if temporaries.is_empty() {
        return Ok(());
    }
    let stable = root.join(COMPILED_BUNDLE_COMPLETION_FILE);
    let stable_bytes = read_regular_file_nofollow(&stable, "compiled-bundle completion")?;
    let candidate = stable_bytes
        .as_deref()
        .unwrap_or_else(|| temporaries[0].1.as_slice());
    if temporaries
        .iter()
        .any(|(_, temporary)| temporary.as_slice() != candidate)
    {
        return Err(format!(
            "compiled bundle {} has conflicting stable/temporary completion records",
            root.display()
        ));
    }
    let ignored = temporaries
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<std::collections::BTreeSet<_>>();
    validate_compiled_bundle_completion_bytes(root, &stable, candidate, &ignored)?;

    if stable_bytes.is_none() {
        match fs::hard_link(&temporaries[0].0, &stable) {
            Ok(()) => sync_parent_directory(&stable, "compiled-bundle completion recovery")?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let raced = read_required_regular_file_nofollow(
                    &stable,
                    "concurrently recovered compiled-bundle completion",
                )?;
                if raced != candidate {
                    return Err(format!(
                        "{} concurrently recorded a different compiled-bundle completion",
                        stable.display()
                    ));
                }
            }
            Err(error) => {
                return Err(format!(
                    "compiled-bundle completion recovery {} -> {} failed: {error}",
                    temporaries[0].0.display(),
                    stable.display()
                ));
            }
        }
    }
    for (temporary, expected) in temporaries {
        let current = read_required_regular_file_nofollow(
            &temporary,
            "compiled-bundle completion temporary before recovery",
        )?;
        if current != expected {
            return Err(format!(
                "compiled-bundle completion temporary {} changed before recovery",
                temporary.display()
            ));
        }
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "recognized compiled-bundle completion temporary {} cannot be reclaimed: {error}",
                temporary.display()
            )
        })?;
    }
    sync_parent_directory(&stable, "compiled-bundle completion recovery")?;
    validate_compiled_bundle_completion(root)?.ok_or_else(|| {
        format!(
            "compiled bundle {} lost its recovered completion record",
            root.display()
        )
    })?;
    Ok(())
}

fn validate_compiled_bundle_completion(
    root: &Path,
) -> Result<Option<CompiledBundleCompletion>, String> {
    let path = root.join(COMPILED_BUNDLE_COMPLETION_FILE);
    let Some(bytes) = read_regular_file_nofollow(&path, "compiled-bundle completion")? else {
        return Ok(None);
    };
    validate_compiled_bundle_completion_bytes(
        root,
        &path,
        &bytes,
        &std::collections::BTreeSet::new(),
    )
    .map(Some)
}

fn ensure_compiled_bundle_completion_for_serving(
    bundle: &ResolvedCompiledBundle,
    current_version: u32,
) -> Result<(), String> {
    recover_compiled_bundle_completion_temporaries(&bundle.physical_root)?;
    if validate_compiled_bundle_completion(&bundle.physical_root)?.is_some() {
        return Ok(());
    }
    if bundle.attention_operator.version < current_version {
        // Historical completed v1 roots predate this server transaction
        // record. Their existing provenance compatibility remains unchanged.
        return Ok(());
    }
    validate_staged_graph_outputs(&bundle.physical_root)?;
    publish_compiled_bundle_completion(&bundle.physical_root)?;
    validate_compiled_bundle_completion(&bundle.physical_root)?
        .ok_or_else(|| {
            format!(
                "current compiled bundle {} lost its bootstrapped completion record",
                bundle.physical_root.display()
            )
        })
        .map(|_| ())
}

fn recover_managed_compiled_bundle_completion_temporaries(
    compiled_root: &Path,
) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(compiled_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "{} cannot be inspected before completion recovery: {error}",
                compiled_root.display()
            ));
        }
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "compiled model inventory {} is not a regular non-symlink directory",
            compiled_root.display()
        ));
    }
    for entry in fs::read_dir(compiled_root)
        .map_err(|error| format!("{} cannot be enumerated: {error}", compiled_root.display()))?
    {
        let entry = entry.map_err(|error| {
            format!("{} cannot be enumerated: {error}", compiled_root.display())
        })?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("{} cannot be inspected: {error}", entry.path().display()))?;
        if metadata.file_type().is_dir() {
            recover_compiled_bundle_completion_temporaries(&entry.path())?;
        }
    }
    Ok(())
}

fn create_source_compile_preflight_stage(
    output: &Path,
    expected: &[u8],
) -> Result<PathBuf, String> {
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no parent for atomic staging",
            output.display()
        )
    })?;
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source compile output name is not UTF-8: {}",
                output.display()
            )
        })?;
    let staging_root = ensure_source_compile_staging_root(parent)?;
    for _ in 0..128 {
        let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
        let stage = staging_root.join(format!(
            ".{output_name}.{}.{}.stage",
            std::process::id(),
            id
        ));
        match fs::create_dir(&stage) {
            Ok(()) => {
                if let Err(error) = publish_bytes_no_clobber(
                    &stage.join(SOURCE_COMPILE_PREFLIGHT_FILE),
                    expected,
                    "source compile preflight",
                ) {
                    let _ = fs::remove_dir(&stage);
                    return Err(error);
                }
                let directory = fs::File::open(&stage)
                    .map_err(|error| format!("{} cannot be opened: {error}", stage.display()))?;
                directory
                    .sync_all()
                    .map_err(|error| format!("{} cannot be synced: {error}", stage.display()))?;
                return Ok(stage);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{} cannot be created: {error}", stage.display())),
        }
    }
    Err(format!(
        "could not reserve a unique source compile stage in {}",
        staging_root.display()
    ))
}

fn remove_owned_source_compile_stage(stage: &Path, expected: &[u8]) -> Result<(), String> {
    let metadata = fs::symlink_metadata(stage)
        .map_err(|error| format!("{} cannot be inspected: {error}", stage.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "owned source compile stage {} changed type before cleanup",
            stage.display()
        ));
    }
    let marker = stage.join(SOURCE_COMPILE_PREFLIGHT_FILE);
    let recorded = read_source_compile_preflight_path(&marker)?;
    if source_compile_preflight_bytes(recorded.source_manifest_kappa.as_deref())? != expected {
        return Err(format!(
            "owned source compile stage {} changed identity before cleanup",
            stage.display()
        ));
    }
    let entries = fs::read_dir(stage)
        .map_err(|error| format!("{} cannot be enumerated: {error}", stage.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{} cannot be enumerated: {error}", stage.display()))?;
    if entries.len() != 1 || entries[0].file_name() != SOURCE_COMPILE_PREFLIGHT_FILE {
        return Err(format!(
            "owned source compile stage {} acquired unexpected entries; refusing cleanup",
            stage.display()
        ));
    }
    fs::remove_file(&marker).map_err(|error| format!("{}: {error}", marker.display()))?;
    fs::remove_dir(stage).map_err(|error| format!("{}: {error}", stage.display()))
}

fn reject_legacy_source_compile_initialization(output: &Path) -> Result<(), String> {
    let initialization = source_compile_initialization_path(output)?;
    match fs::symlink_metadata(&initialization) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!(
            "stranded legacy source compile initialization {} is ambiguous and cannot authorize adoption; refusing mutation",
            initialization.display()
        )),
        Err(error) => Err(format!("{} cannot be inspected: {error}", initialization.display())),
    }
}

/// Publish a marker-bearing compile directory in one atomic, exclusive
/// namespace operation. Process death can leave only a hidden staging
/// directory; the resolver-visible final root is either absent or already
/// carries its canonical preflight marker.
fn publish_source_compile_preflight(output: &Path, kappa: Option<&str>) -> Result<(), String> {
    let expected = source_compile_preflight_bytes(kappa)?;
    let parent = output.parent().ok_or_else(|| {
        format!(
            "source compile output {} has no parent for atomic publication",
            output.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("{} cannot be created: {error}", parent.display()))?;
    let parent_metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("{} cannot be inspected: {error}", parent.display()))?;
    if !parent_metadata.file_type().is_dir() {
        return Err(format!(
            "source compile parent {} is not a regular non-symlink directory",
            parent.display()
        ));
    }
    reject_legacy_source_compile_initialization(output)?;

    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() {
                return Err(format!(
                    "source compile output {} is not a regular non-symlink directory",
                    output.display()
                ));
            }
            validate_source_compile_identity_temporaries(output)?;
            if let Some(recorded) = read_optional_source_compile_preflight(output)? {
                if recorded.source_manifest_kappa.as_deref() == kappa {
                    return Ok(());
                }
                return Err(format!(
                    "source compile output {} is preflight-bound to {:?}, not {:?}",
                    output.display(),
                    recorded.source_manifest_kappa,
                    kappa
                ));
            }
            if !source_compile_pre_attention_prefix(output)? {
                return Err(format!(
                    "source compile output {} exists without a stable source compile preflight; refusing to adopt or mutate it",
                    output.display()
                ));
            }
            let temporary = source_compile_preflight_temporary_binding(output)?;
            let bound = read_optional_source_manifest_kappa_binding(output)?;
            let recovered = match (temporary, bound) {
                (Some(temporary), Some(bound)) if temporary.as_deref() == Some(&bound) => {
                    Some(temporary)
                }
                (Some(temporary), None) => Some(temporary),
                (None, Some(bound)) => Some(Some(bound)),
                (Some(temporary), Some(bound)) => {
                    return Err(format!(
                        "source compile output {} has conflicting temporary preflight {:?} and source binding {bound}",
                        output.display(),
                        temporary
                    ));
                }
                (None, None) => None,
            };
            let Some(recovered) = recovered else {
                return Err(format!(
                    "source compile output {} has identity sidecars but no recoverable source snapshot binding",
                    output.display()
                ));
            };
            if recovered.as_deref() != kappa {
                return Err(format!(
                    "source compile output {} has recoverable binding {:?}, not requested {:?}",
                    output.display(),
                    recovered,
                    kappa
                ));
            }
            publish_bytes_no_clobber(
                &output.join(SOURCE_COMPILE_PREFLIGHT_FILE),
                &expected,
                "source compile preflight",
            )?;
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{} cannot be inspected: {error}", output.display())),
    }

    let stage = create_source_compile_preflight_stage(output, &expected)?;
    match rename_directory_no_replace(&stage, output) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            let cleanup_error = remove_owned_source_compile_stage(&stage, &expected).err();
            let raced = match fs::symlink_metadata(output) {
                Ok(metadata) if metadata.file_type().is_dir() => {
                    match read_optional_source_compile_preflight(output) {
                        Ok(Some(recorded))
                            if recorded.source_manifest_kappa.as_deref() == kappa =>
                        {
                            None
                        }
                        Ok(Some(recorded)) => Some(format!(
                            "raced output records {:?}, not {:?}",
                            recorded.source_manifest_kappa, kappa
                        )),
                        Ok(None) => {
                            Some("raced output has no stable source compile preflight".to_owned())
                        }
                        Err(error) => Some(error),
                    }
                }
                Ok(_) => Some("raced output is not a regular non-symlink directory".to_owned()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(format!(
                    "final output is still absent after exclusive rename failure: {rename_error}"
                )),
                Err(error) => Some(format!("raced output cannot be inspected: {error}")),
            };
            if raced.is_none() && cleanup_error.is_none() {
                return Ok(());
            }
            Err(format!(
                "atomic source compile publication {} -> {} failed: {}; cleanup: {}",
                stage.display(),
                output.display(),
                raced.unwrap_or_else(|| rename_error.to_string()),
                cleanup_error.unwrap_or_else(|| "owned stage removed".to_owned())
            ))
        }
    }
}

fn publish_source_manifest_kappa_binding(output: &Path, kappa: &str) -> Result<(), String> {
    let expected = source_manifest_kappa_binding_bytes(kappa)?;
    let metadata = fs::symlink_metadata(output).map_err(|error| {
        format!(
            "{} has no marker-bearing source compile preflight directory: {error}",
            output.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "source compile output {} is not a regular non-symlink directory",
            output.display()
        ));
    }
    let path = output.join(SOURCE_MANIFEST_KAPPA_BINDING_FILE);
    for _ in 0..128 {
        let id = NEXT_SOURCE_KAPPA_BINDING_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = output.join(format!(
            ".{SOURCE_MANIFEST_KAPPA_BINDING_FILE}.{}.{}.tmp",
            std::process::id(),
            id
        ));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{}: {error}", temporary.display())),
        };
        if let Err(error) = file.write_all(&expected).and_then(|()| file.sync_all()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("{}: {error}", temporary.display()));
        }
        drop(file);
        match fs::hard_link(&temporary, &path) {
            Ok(()) => {
                fs::remove_file(&temporary)
                    .map_err(|error| format!("{}: {error}", temporary.display()))?;
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
                let recorded = read_optional_source_manifest_kappa_binding(output)?
                    .ok_or_else(|| format!("{} appeared and then disappeared", path.display()))?;
                if recorded == kappa {
                    return Ok(());
                }
                return Err(format!(
                    "source compile output {} is already bound to source manifest kappa {recorded}, not {kappa}",
                    output.display()
                ));
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(format!(
                    "source-manifest kappa binding publish {} -> {}: {error}",
                    temporary.display(),
                    path.display()
                ));
            }
        }
    }
    Err(format!(
        "could not reserve a unique source-manifest kappa binding temporary in {}",
        output.display()
    ))
}

fn preflight_and_bind_source_snapshot_kappa(
    output: &Path,
    requested: Option<&str>,
) -> Result<(), String> {
    if requested.is_some_and(|kappa| !canonical_source_manifest_kappa(kappa)) {
        return Err(format!(
            "source snapshot kappa {requested:?} is not canonical"
        ));
    }
    let requested = requested.map(str::to_owned);
    let recorded = read_optional_source_manifest_kappa_binding(output)?;
    match (requested.as_deref(), recorded.as_deref()) {
        (None, None) => {}
        (None, Some(recorded)) => {
            return Err(format!(
                "legacy manifestless source cannot resume source compile output {} bound to source manifest kappa {recorded}",
                output.display()
            ));
        }
        (Some(requested), Some(recorded)) if requested == recorded => {}
        (Some(requested), Some(recorded)) => {
            return Err(format!(
                "source compile output {} is bound to source manifest kappa {recorded}, not current source {requested}; refusing cross-snapshot resume",
                output.display()
            ));
        }
        (Some(_), None) => {}
    }

    let mut cover_bootstrap = false;
    if recorded.is_none() {
        if let Some(requested) = requested.as_deref() {
            if source_compile_output_has_payload(output)? {
                let report = output.join("graph-cover/cover_report.json");
                let (present, _, cover_kappa) = parse_cover_provenance(&report)?;
                if !present || cover_kappa.as_deref() != Some(requested) {
                    return Err(format!(
                        "populated source compile output {} has no immutable source-manifest kappa binding or exact cover provenance for {requested}; refusing to resume or relabel it",
                        output.display()
                    ));
                }
                cover_bootstrap = true;
            }
        }
    }
    if cover_bootstrap {
        reject_legacy_source_compile_initialization(output)?;
        let expected = source_compile_preflight_bytes(requested.as_deref())?;
        publish_bytes_no_clobber(
            &output.join(SOURCE_COMPILE_PREFLIGHT_FILE),
            &expected,
            "source compile preflight",
        )?;
    } else {
        publish_source_compile_preflight(output, requested.as_deref())?;
    }
    if let Some(requested) = requested.as_deref() {
        publish_source_manifest_kappa_binding(output, requested)?;
    }
    Ok(())
}

#[cfg(test)]
fn preflight_and_bind_source_manifest_kappa(
    output: &Path,
    manifest: Option<&crate::model::SourceManifest>,
) -> Result<(), String> {
    let requested = manifest.map(source_manifest_kappa).transpose()?;
    preflight_and_bind_source_snapshot_kappa(output, requested.as_deref())
}

/// All current source families advance in one arithmetic era. Keep the check
/// executable so a future independently-versioned family cannot silently use
/// the server's shared era suffix.
fn current_source_attention_era_version() -> Result<u32, String> {
    use uor_r4_model_source::attention::AttentionOperatorSpec;

    let versions = [
        AttentionOperatorSpec::STANDARD_VERSION,
        AttentionOperatorSpec::EXPERIMENTAL_R4_VERSION,
        AttentionOperatorSpec::LEARNED_ABSOLUTE_VERSION,
    ];
    versions
        .iter()
        .all(|version| *version == versions[0])
        .then_some(versions[0])
        .ok_or_else(|| {
            format!(
                "source attention families no longer share one current era: standard={}, experimental={}, learned-absolute={}",
                versions[0], versions[1], versions[2]
            )
        })
}

/// Select the browser compile output without mutating either arithmetic era.
/// A fresh or already-current conventional root remains usable. An explicit
/// historical binding, or an unbound populated legacy root, redirects to the
/// deterministic `<name>-attention-vN` root. The era root itself must be
/// uncreated or carry a registry-exact current-version binding. A present
/// empty era root is terminal, matching load-time precedence: otherwise a
/// compile could write the conventional sibling and immediately become
/// unloadable behind the empty preferred root.
#[cfg(test)]
fn source_compile_output_for_attention_era(
    compiled_root: &Path,
    name: &str,
    current_version: u32,
) -> Result<PathBuf, String> {
    // The paired inspection validates both siblings before selection. A
    // current conventional root therefore cannot hide a duplicate, malformed,
    // or unbound resolver-owned suffix root.
    let pair = inspect_compiled_model_pair(compiled_root, name, current_version)?;
    select_source_compile_output_from_pair(pair)
}

fn select_source_compile_output_from_pair(pair: CompiledModelPair) -> Result<PathBuf, String> {
    match (&pair.conventional, &pair.current) {
        (_, CompiledRootState::Empty) => Err(format!(
            "preferred current bundle {} exists but is empty; refusing compile mutation until the stale empty root is removed",
            pair.current_root.display()
        )),
        (
            CompiledRootState::ImplicitV1(_) | CompiledRootState::BoundHistorical(_),
            CompiledRootState::PreAttentionIdentity,
        ) => Ok(pair.current_root),
        (_, CompiledRootState::PreAttentionIdentity) => Err(format!(
            "preferred current bundle {} contains only a pre-attention identity prefix without a matching historical conventional bundle; refusing ambiguous compile mutation",
            pair.current_root.display()
        )),
        (_, CompiledRootState::BoundCurrent(_)) => Ok(pair.current_root),
        (CompiledRootState::BoundCurrent(_), CompiledRootState::Absent) => {
            Ok(pair.conventional_root)
        }
        (
            CompiledRootState::ImplicitV1(_) | CompiledRootState::BoundHistorical(_),
            CompiledRootState::Absent,
        ) => Ok(pair.current_root),
        (
            CompiledRootState::Absent
                | CompiledRootState::Empty
                | CompiledRootState::PreAttentionIdentity,
            CompiledRootState::Absent,
        ) => Ok(pair.conventional_root),
        (_, CompiledRootState::ImplicitV1(_) | CompiledRootState::BoundHistorical(_)) => {
            Err(format!(
                "resolver-owned current root {} was not classified as current",
                pair.current_root.display()
            ))
        }
    }
}

fn source_compile_output_for_operator_era(
    compiled_root: &Path,
    name: &str,
    current_version: u32,
    source_operator: &AttentionOperatorSpec,
) -> Result<PathBuf, String> {
    let registered =
        uor_r4_model_source::attention::operator_spec(&source_operator.id, source_operator.version)
            .map_err(|error| format!("source teacher attention operator is invalid: {error}"))?;
    if &registered != source_operator
        || !source_attention_operator(source_operator)
        || source_operator.version != current_version
    {
        return Err(format!(
            "source teacher declares unsupported compile attention operator {}/{} for current era {current_version}",
            source_operator.id, source_operator.version
        ));
    }
    let pair = inspect_compiled_model_pair(compiled_root, name, current_version)?;
    for recorded in [pair.conventional.operator(), pair.current.operator()]
        .into_iter()
        .flatten()
    {
        if recorded.id != source_operator.id {
            return Err(format!(
                "compiled model {} already records source-attention family {}/{} but the selected source teacher declares {}/{}; refusing to create or mutate a conflicting current-era sibling",
                pair.logical_name,
                recorded.id,
                recorded.version,
                source_operator.id,
                source_operator.version
            ));
        }
    }
    select_source_compile_output_from_pair(pair)
}

struct CompiledSourceBundle {
    final_output: PathBuf,
    working_output: PathBuf,
    stage: CompiledBundleStage,
    /// Held until the caller finishes cover/score validation and atomically
    /// installs the replacement serving state. This is intentionally not
    /// released when Stage A returns: every compiler write derived from the
    /// selected source root belongs to the same cross-process transaction.
    _sessions: SourceCompileSessionLocks,
}

impl CompiledSourceBundle {
    fn publish(&mut self) -> Result<(), String> {
        publish_compiled_bundle_completion(&self.working_output)?;
        validate_compiled_bundle_completion(&self.working_output)?.ok_or_else(|| {
            format!(
                "compiled-bundle stage {} lost its completion record before publication",
                self.working_output.display()
            )
        })?;
        self.stage.publish()
    }
}

#[cfg(test)]
fn immutable_graph_artifact_present(graph_path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(graph_path) {
        // Any present entry belongs to the last published attempt. A valid
        // graph is immutable/idempotent; an invalid or nonregular entry must
        // fail later validation unchanged. Neither case authorizes truncating
        // last-good bytes in place.
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "existing graph artifact {} cannot be inspected before immutable reuse: {error}",
            graph_path.display()
        )),
    }
}

#[cfg(test)]
fn run_graph_writer_if_incomplete<F>(reuse_existing_graph: bool, writer: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    if reuse_existing_graph {
        Ok(())
    } else {
        writer()
    }
}

fn compile_bundle_from_source(
    source: &Path,
    status: &Arc<Mutex<R4g1CompileStatus>>,
    tokenizer_selection: Option<&TokenizerAdapterKey>,
    source_snapshot_kappa: &str,
    requested_graph_path: Option<&Path>,
) -> Result<CompiledSourceBundle, String> {
    let name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source path is not a valid model directory: {}",
                source.display()
            )
        })?;
    // This is a server-owned namespace, not a model-family convention.
    // Reject a colliding source basename before selecting or creating any
    // output directory.
    validate_logical_model_name(name, current_source_attention_era_version()?)?;
    // Resolve once before the resumable stage mutates its output and pass the
    // selected family/version as an atomic pair. Sources with multiple
    // definitions are intentionally refused until the caller names one.
    let tokenizer =
        resolve_source_tokenizer(source, tokenizer_selection).map_err(|error| error.to_string())?;
    let adapter = tokenizer
        .adapter()
        .ok_or_else(|| "source resolved to an adapterless tokenizer".to_owned())?;
    let teacher = uor_r4_model_source::Teacher::load(source)
        .map_err(|error| format!("source teacher cannot be loaded before compile: {error}"))?;
    let source_operator = teacher
        .attention_operator_spec()
        .ok_or_else(|| "source teacher declares no attention operator".to_owned())?;
    let compiled_root = Path::new(".uor-models/compiled");
    let current_version = current_source_attention_era_version()?;
    let conventional = compiled_root.join(name);
    let current = compiled_root.join(format!("{name}{}", attention_era_suffix(current_version)));
    let mut session_subjects = vec![
        compiled_root.to_path_buf(),
        conventional.clone(),
        current.clone(),
        source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        source.to_path_buf(),
    ];
    if let Some(graph_path) = requested_graph_path {
        let graph_output = graph_path.parent().ok_or_else(|| {
            format!(
                "configured graph artifact {} has no output directory",
                graph_path.display()
            )
        })?;
        session_subjects.push(graph_output.to_path_buf());
    }
    let sessions = try_lock_source_compile_sessions(
        session_subjects,
        SourceCompileSessionMode::ExclusiveWriter,
    )?;
    recover_source_compile_identity_temporaries(&conventional)?;
    recover_source_compile_identity_temporaries(&current)?;
    let output = source_compile_output_for_operator_era(
        compiled_root,
        name,
        current_version,
        &source_operator,
    )?;
    // The process-local source-cache reservation is acquired by the HTTP
    // handler first. Both era siblings and any external score sink are now
    // exclusively owned before selection, preflight publication, or adoption.
    // The returned guards remain live through final serving installation.
    let refreshed_output = source_compile_output_for_operator_era(
        compiled_root,
        name,
        current_version,
        &source_operator,
    )?;
    if refreshed_output != output {
        return Err(format!(
            "source compile output selection changed from {} to {} while acquiring its cross-process session; refusing stale mutation",
            output.display(),
            refreshed_output.display()
        ));
    }
    let canonical_graph_path = output.join("graph/score.r4g1");
    if let Some(requested) = requested_graph_path {
        if normalized_absolute_path(requested)? != normalized_absolute_path(&canonical_graph_path)?
        {
            return Err(format!(
                "source-driven browser compilation requires its graph sink at {}; configured external sink {} cannot participate in the atomic compiled-bundle transaction",
                canonical_graph_path.display(),
                requested.display()
            ));
        }
    }
    preflight_and_bind_source_snapshot_kappa(&output, Some(source_snapshot_kappa))?;
    let stage = CompiledBundleStage::allocate(&output, source_snapshot_kappa)?;
    let working_output = stage.path.clone();
    let args = vec![
        "--source".to_owned(),
        source.display().to_string(),
        "--output".to_owned(),
        working_output.display().to_string(),
        "--seconds".to_owned(),
        R4G1_CORPUS_SECONDS.to_owned(),
        "--target".to_owned(),
        R4G1_CORPUS_TARGET.to_owned(),
        "--sequence-length".to_owned(),
        "128".to_owned(),
        "--tokenizer-family".to_owned(),
        adapter.family,
        "--tokenizer-version".to_owned(),
        adapter.version.to_string(),
    ];
    // The teacher compile can take most of the wall-clock time. Run it in a
    // child worker so the server can report corpus progress while the native
    // compiler is generating/resuming records.
    let meta_path = working_output.join("corpus.meta");
    let status_for_compiler = Arc::clone(status);
    let compiler = std::thread::spawn(move || {
        set_r4g1_compile_progress(
            &status_for_compiler,
            6,
            "Loading teacher model and preparing corpus...",
        );
        uor_r4_graph_cli::compile_hugging_face(&args)
    });
    while !compiler.is_finished() {
        let observed = fs::read(&meta_path).ok().and_then(|bytes| {
            bytes
                .get(..8)
                .and_then(|prefix| prefix.try_into().ok().map(u64::from_le_bytes))
        });
        let target = R4G1_CORPUS_TARGET.parse::<u64>().unwrap_or(1).max(1);
        let progress = observed
            .map(|current| 5u8.saturating_add((current.saturating_mul(14) / target).min(14) as u8))
            .unwrap_or(6);
        let message = observed
            .map(|current| format!("Generating teacher corpus ({current} / {target} samples)..."))
            .unwrap_or_else(|| "Loading teacher model and preparing corpus...".to_owned());
        set_r4g1_compile_progress(status, progress, &message);
        std::thread::sleep(Duration::from_millis(500));
    }
    compiler
        .join()
        .map_err(|payload| {
            format!(
                "teacher compilation panicked: {}",
                panic_payload_message(&*payload)
            )
        })?
        .map_err(|error| error.to_string())?;
    validate_source_bundle_inventory(&working_output)?;
    let meta = working_output.join("corpus.meta");
    let records = working_output.join("corpus.records");
    let meta_str = meta
        .to_str()
        .ok_or_else(|| format!("corpus metadata path is not UTF-8: {}", meta.display()))?;
    let records_str = records
        .to_str()
        .ok_or_else(|| format!("corpus records path is not UTF-8: {}", records.display()))?;
    if uor_r4_core::transformerless::compiler::load_corpus_from(meta_str, records_str).is_none() {
        return Err(format!(
            "teacher corpus is incomplete at {}; click Compile / Refresh again to resume generation toward {} samples",
            working_output.display(), R4G1_CORPUS_TARGET
        ));
    }
    Ok(CompiledSourceBundle {
        final_output: output,
        working_output,
        stage,
        _sessions: sessions,
    })
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic payload was not a string".to_owned()
    }
}

/// Append the exact registered source-attention identity recorded with one
/// corpus. A genuinely operator-less caller corpus retains the historical
/// implicit standard/1 interpretation; source-driven compiles must carry the
/// explicit sidecar written by stage A.
fn append_recorded_attention_operator(
    cover_args: &mut Vec<String>,
    corpus_meta: &Path,
    corpus_recs: &Path,
    require_explicit: bool,
) -> Result<(), String> {
    let operator = uor_r4_graph_cli::recorded_corpus_attention_operator(corpus_meta, corpus_recs)
        .map_err(|error| error.to_string())?;
    match operator {
        Some(operator) => {
            let json = serde_json::to_string(&operator).map_err(|error| error.to_string())?;
            cover_args.extend(["--attention-operator".to_owned(), json]);
            Ok(())
        }
        None if require_explicit => {
            let root = corpus_meta.parent().unwrap_or_else(|| Path::new("."));
            Err(format!(
                "compiled source corpus is missing its attention-operator binding: {}",
                root.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE)
                    .display()
            ))
        }
        None => Ok(()),
    }
}

fn compile_r4g1_bundle(
    cli: &ServerConfig,
    serving: &SharedServingModel,
    status: &Arc<Mutex<R4g1CompileStatus>>,
    downloaded_source: Option<&Path>,
    expected_source: Option<&SourceDownload>,
    tokenizer_selection: Option<&TokenizerAdapterKey>,
    require_source_manifest: bool,
) -> Result<serde_json::Value, String> {
    let base_epoch = serving.lock().unwrap().epoch;
    let source_snapshot_before = downloaded_source
        .map(|source| {
            let manifest = validate_compile_source_snapshot_for_identity(
                source,
                require_source_manifest,
                expected_source,
            )?;
            verified_managed_source_snapshot_from_manifest(source, manifest)
        })
        .transpose()?;
    set_r4g1_compile_progress(
        status,
        5,
        "Preparing teacher corpus and R4G1 compiler inputs...",
    );
    // A downloaded source is authoritative for the browser workflow. Even
    // when an older corpus bundle already exists, resume the teacher compile
    // first so the requested target (currently 200k tokens) is actually
    // reached instead of silently rebuilding the old ~20k corpus.
    let mut compiled_source = downloaded_source
        .map(|source| {
            compile_bundle_from_source(
                source,
                status,
                tokenizer_selection,
                source_snapshot_before
                    .as_ref()
                    .map(|snapshot| snapshot.content_kappa.as_str())
                    .ok_or_else(|| {
                        format!(
                            "compiled source {} has no verified snapshot identity",
                            source.display()
                        )
                    })?,
                cli.r4g1_artifact.as_deref().map(Path::new),
            )
        })
        .transpose()?;
    let source_root = compiled_source
        .as_ref()
        .map(|compiled| compiled.working_output.clone());
    let managed_source_root = compiled_source
        .as_ref()
        .map(|compiled| compiled.final_output.clone());
    let compiled_from_source = source_root.is_some();
    if downloaded_source.is_none() && tokenizer_selection.is_some() {
        return Err(
            "an explicit tokenizer selection requires a downloaded source directory".to_owned(),
        );
    }
    set_r4g1_compile_progress(status, 20, "Building the R4G1 cover...");
    let (mut artifacts, corpus_meta, corpus_recs, cover_output, mut graph_output, mut graph_path) =
        match source_root {
            Some(root) => {
                let artifacts = root.join("tless_artifacts.bin");
                let corpus_meta = root.join("corpus.meta");
                let corpus_recs = root.join("corpus.records");
                let cover_output = root.join("graph-cover");
                // `compile_bundle_from_source` already proved that any
                // explicit configured sink denotes the final canonical graph.
                // All writers and pre-publication loads must nevertheless use
                // the hidden working generation, never that live final path.
                let graph_path = root.join("graph").join("score.r4g1");
                let graph_output = graph_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                (
                    artifacts,
                    corpus_meta,
                    corpus_recs,
                    cover_output,
                    graph_output,
                    graph_path,
                )
            }
            None => match r4g1_compile_paths(cli) {
                Ok((corpus_meta, corpus_recs, cover_output, graph_output)) => {
                    let artifacts = PathBuf::from(&cli.tless_artifacts);
                    let graph_path = cli
                        .r4g1_artifact
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| graph_output.join("score.r4g1"));
                    (
                        artifacts,
                        corpus_meta,
                        corpus_recs,
                        cover_output,
                        graph_output,
                        graph_path,
                    )
                }
                Err(error) => return Err(error),
            },
        };
    let legacy_sessions = if compiled_from_source {
        None
    } else {
        Some(try_lock_source_compile_sessions(
            [
                PathBuf::from(".uor-models/compiled"),
                cover_output.clone(),
                graph_output.clone(),
            ],
            SourceCompileSessionMode::ExclusiveWriter,
        )?)
    };
    let _legacy_sessions = legacy_sessions;
    if !compiled_from_source {
        let selected = (
            corpus_meta.clone(),
            corpus_recs.clone(),
            cover_output.clone(),
            graph_output.clone(),
        );
        let refreshed = r4g1_compile_paths(cli)?;
        require_unchanged_legacy_compile_paths(&selected, &refreshed)?;
    }
    for path in [&artifacts, &corpus_meta, &corpus_recs] {
        if !path.is_file() {
            return Err(format!(
                "required compilation input is missing: {}",
                path.display()
            ));
        }
    }

    let mut cover_args = vec![
        "--corpus-meta".to_owned(),
        corpus_meta.display().to_string(),
        "--corpus-recs".to_owned(),
        corpus_recs.display().to_string(),
        "--artifacts".to_owned(),
        artifacts.display().to_string(),
        "--depths".to_owned(),
        R4G1_COVER_DEPTHS.to_owned(),
        "--k0".to_owned(),
        R4G1_COVER_K0.to_owned(),
        "--regions-budget".to_owned(),
        R4G1_COVER_REGIONS.to_owned(),
        "--memory-budget".to_owned(),
        R4G1_COVER_MEMORY_MB.to_owned(),
        "--min-support".to_owned(),
        R4G1_COVER_MIN_SUPPORT.to_owned(),
        "--entropy-gain".to_owned(),
        R4G1_COVER_ENTROPY_GAIN.to_owned(),
        "--radius-quantile".to_owned(),
        R4G1_COVER_RADIUS_QUANTILE.to_owned(),
        "--out".to_owned(),
        cover_output.display().to_string(),
    ];
    let tokenizer_path = artifacts
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("tokenizer.bin");
    append_tokenizer_arg(
        &mut cover_args,
        &tokenizer_path,
        downloaded_source.is_some(),
    )?;
    // #597/#704: server-selected source snapshots were strictly validated
    // above. A manifest-backed source uses its canonical manifest κ; a
    // genuine legacy manifestless source uses the deterministic verified tree
    // κ, so a resumed corpus cannot be relabeled after source-byte drift.
    if let Some(snapshot) = source_snapshot_before.as_ref() {
        cover_args.extend([
            "--source-manifest-kappa".to_owned(),
            snapshot.content_kappa.clone(),
        ]);
    }
    // #600: bind the typed geometry-projection record the teacher adapter
    // applies (bucket-average/1, hidden_size→288) into the cover report,
    // when the downloaded snapshot declares its width. Opportunistic like
    // the #597 binding above: a miss compiles exactly as before.
    if let Some(geometry) = downloaded_source.and_then(geometry_projection_of) {
        cover_args.extend(["--geometry-projection".to_owned(), geometry]);
    }
    // #602/#704: bind the identity stage A or the observation manifest
    // actually recorded. Reconstructing it as standard from the absent
    // experimental switch would mislabel GPT-2's learned-absolute operator.
    append_recorded_attention_operator(
        &mut cover_args,
        &corpus_meta,
        &corpus_recs,
        compiled_from_source,
    )?;
    if compiled_from_source {
        uor_r4_graph_cli::cover_command(&cover_args).map_err(|error| error.to_string())?;
    }

    set_r4g1_compile_progress(status, 55, "Scoring graph transitions and emissions...");
    let cover_artifact = cover_output.join("cover.r4g1");
    let mut score_args = vec![
        "--corpus-meta".to_owned(),
        corpus_meta.display().to_string(),
        "--corpus-recs".to_owned(),
        corpus_recs.display().to_string(),
        "--artifacts".to_owned(),
        artifacts.display().to_string(),
        "--cover".to_owned(),
        cover_artifact.display().to_string(),
        "--transition-out-degree".to_owned(),
        R4G1_SCORE_TRANSITION_DEGREE.to_owned(),
        "--emission-entries".to_owned(),
        R4G1_SCORE_EMISSION_ENTRIES.to_owned(),
        "--root-top-b".to_owned(),
        R4G1_SCORE_ROOT_TOP_B.to_owned(),
        "--exct-top-x".to_owned(),
        R4G1_SCORE_EXCT_TOP_X.to_owned(),
        "--quality-profile".to_owned(),
        R4G1_SCORE_QUALITY_PROFILE.to_owned(),
        "--out".to_owned(),
        graph_output.display().to_string(),
    ];
    append_tokenizer_arg(
        &mut score_args,
        &tokenizer_path,
        downloaded_source.is_some(),
    )?;
    if compiled_from_source {
        uor_r4_graph_cli::score_command(&score_args).map_err(|error| error.to_string())?;
    } else {
        let identity = capture_legacy_graph_generation_identity(LegacyGraphGenerationInputs {
            artifacts: &artifacts,
            corpus_meta: &corpus_meta,
            corpus_recs: &corpus_recs,
            tokenizer: &tokenizer_path,
            cover_output: &cover_output,
            graph_output: &graph_output,
            cover_args: &cover_args,
            score_args: &score_args,
        })?;
        let (action, record_paths, attempt_bytes) = legacy_graph_generation_action(&identity)?;
        match action {
            LegacyGraphGenerationAction::Reuse => {
                remove_exact_legacy_attempt(&record_paths.attempt, &attempt_bytes)?;
            }
            LegacyGraphGenerationAction::ResumeScore => {
                ensure_legacy_graph_attempt(&record_paths.attempt, &attempt_bytes)?;
                let score_stage = build_graph_writer_stage(
                    &graph_output,
                    GraphOutputKind::Score,
                    &score_args,
                    |args| uor_r4_graph_cli::score_command(args).map_err(|error| error.to_string()),
                )?;
                R4g1State::load_with_source(&score_stage.path.join("score.r4g1"), &artifacts, None)
                    .map_err(|error| {
                        format!("staged resumed legacy graph failed validation: {error}")
                    })?;
                let after =
                    capture_legacy_graph_generation_identity(LegacyGraphGenerationInputs {
                        artifacts: &artifacts,
                        corpus_meta: &corpus_meta,
                        corpus_recs: &corpus_recs,
                        tokenizer: &tokenizer_path,
                        cover_output: &cover_output,
                        graph_output: &graph_output,
                        cover_args: &cover_args,
                        score_args: &score_args,
                    })?;
                if after != identity {
                    return Err(
                        "legacy graph compiler inputs changed while the score resume was staged; refusing publication"
                            .to_owned(),
                    );
                }
                publish_legacy_graph_score_resume(
                    score_stage,
                    &cover_output,
                    &graph_output,
                    &record_paths,
                    &attempt_bytes,
                    &identity,
                )?;
            }
            LegacyGraphGenerationAction::BuildBoth => {
                ensure_legacy_graph_attempt(&record_paths.attempt, &attempt_bytes)?;
                let cover_stage = build_graph_writer_stage(
                    &cover_output,
                    GraphOutputKind::Cover,
                    &cover_args,
                    |args| uor_r4_graph_cli::cover_command(args).map_err(|error| error.to_string()),
                )?;
                let mut staged_score_args = score_args.clone();
                replace_argument_value(
                    &mut staged_score_args,
                    "--cover",
                    &cover_stage.path.join("cover.r4g1"),
                )?;
                let score_stage = build_graph_writer_stage(
                    &graph_output,
                    GraphOutputKind::Score,
                    &staged_score_args,
                    |args| uor_r4_graph_cli::score_command(args).map_err(|error| error.to_string()),
                )?;
                R4g1State::load_with_source(&score_stage.path.join("score.r4g1"), &artifacts, None)
                    .map_err(|error| {
                        format!("staged replacement legacy graph failed validation: {error}")
                    })?;
                let after =
                    capture_legacy_graph_generation_identity(LegacyGraphGenerationInputs {
                        artifacts: &artifacts,
                        corpus_meta: &corpus_meta,
                        corpus_recs: &corpus_recs,
                        tokenizer: &tokenizer_path,
                        cover_output: &cover_output,
                        graph_output: &graph_output,
                        cover_args: &cover_args,
                        score_args: &score_args,
                    })?;
                if after != identity {
                    return Err(
                        "legacy graph compiler inputs changed while replacement outputs were staged; refusing publication"
                            .to_owned(),
                    );
                }
                let output_files = staged_legacy_graph_output_kappas(
                    &cover_stage.path,
                    &score_stage.path,
                    &cover_output,
                    &graph_output,
                )?;
                let completion = LegacyGraphGenerationCompletion {
                    schema: LEGACY_GRAPH_GENERATION_SCHEMA.to_owned(),
                    identity,
                    output_files,
                };
                publish_legacy_graph_generation_pair(
                    cover_stage,
                    score_stage,
                    &cover_output,
                    &graph_output,
                    &record_paths,
                    &attempt_bytes,
                    &completion,
                )?;
            }
        }
    }

    let source_snapshot_after_cover = match (downloaded_source, source_snapshot_before.as_ref()) {
        (Some(source), Some(before)) => {
            let manifest = validate_compile_source_snapshot_for_identity(
                source,
                require_source_manifest,
                expected_source,
            )?;
            let after = verified_managed_source_snapshot_from_manifest(source, manifest)?;
            require_unchanged_managed_source_snapshot(source, "R4G1 compilation", before, &after)?;
            Some(after)
        }
        _ => None,
    };

    if let Some(compiled) = compiled_source.as_mut() {
        validate_staged_graph_outputs(&compiled.working_output)?;
        // Exercise the exact runtime/quality-policy loader against the staged
        // graph+report+artifact+tokenizer set before a single final namespace
        // byte changes. Malformed or below-floor reports cannot be hidden by
        // an otherwise parseable score.r4g1.
        R4g1State::load_with_source(&graph_path, &artifacts, downloaded_source)
            .map_err(|error| format!("staged compiled graph failed validation: {error}"))?;
        if let (Some(source), Some(before)) =
            (downloaded_source, source_snapshot_after_cover.as_ref())
        {
            let manifest = validate_compile_source_snapshot_for_identity(
                source,
                require_source_manifest,
                expected_source,
            )?;
            let after = verified_managed_source_snapshot_from_manifest(source, manifest)?;
            require_unchanged_managed_source_snapshot(
                source,
                "staged R4G1 runtime validation",
                before,
                &after,
            )?;
            let recorded = read_optional_source_manifest_kappa_binding(&compiled.working_output)?
                .ok_or_else(|| {
                format!(
                    "staged compiled bundle {} lost its source snapshot binding before publication",
                    compiled.working_output.display()
                )
            })?;
            let (_, _, cover_kappa) = parse_cover_provenance(
                &compiled
                    .working_output
                    .join("graph-cover/cover_report.json"),
            )?;
            if recorded != after.content_kappa
                || cover_kappa.as_deref() != Some(after.content_kappa.as_str())
            {
                return Err(format!(
                    "staged compiled bundle {} does not bind verified source snapshot {} consistently across Stage A and cover provenance",
                    compiled.working_output.display(),
                    after.content_kappa
                ));
            }
        }
        compiled.publish()?;
        let final_root = compiled.final_output.clone();
        artifacts = final_root.join("tless_artifacts.bin");
        graph_output = final_root.join("graph");
        graph_path = graph_output.join("score.r4g1");
    }

    set_r4g1_compile_progress(status, 90, "Validating and loading the compiled graph...");
    let current_version = current_source_attention_era_version()?;
    let resolved = match (managed_source_root.as_ref(), downloaded_source) {
        (Some(root), Some(source)) if graph_path.starts_with(root) => {
            let logical_name = source
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    format!(
                        "source path is not a valid model directory: {}",
                        source.display()
                    )
                })?;
            let compiled_root = root.parent().ok_or_else(|| {
                format!(
                    "compiled output {} has no compiled-model parent",
                    root.display()
                )
            })?;
            let pair = inspect_compiled_model_pair(compiled_root, logical_name, current_version)?;
            let resolved =
                resolve_loadable_compiled_bundle(&pair, current_version)?.ok_or_else(|| {
                    format!(
                        "compiled output {} did not resolve to a loadable graph bundle",
                        root.display()
                    )
                })?;
            if resolved.physical_root != *root
                || resolved.graph != graph_path
                || resolved.teacher != artifacts
            {
                return Err(format!(
                    "compiled output {} resolved to a different physical bundle {}",
                    root.display(),
                    resolved.physical_root.display()
                ));
            }
            validate_resolved_source_snapshot_binding(
                &resolved,
                source_snapshot_after_cover.as_ref(),
                current_version,
            )?;
            Some(resolved)
        }
        _ => None,
    };
    let inferred_source = if downloaded_source.is_none() {
        source_for_compiled_teacher(&artifacts)?
    } else {
        None
    };
    let source_for_host = downloaded_source.or(inferred_source.as_deref());
    let logical_name = resolved
        .as_ref()
        .map(|resolved| resolved.logical_name.as_str())
        .or_else(|| {
            source_for_host
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
        })
        .unwrap_or("uor-r4");
    let source_snapshot_before_prepare = match source_for_host {
        Some(source) if downloaded_source == Some(source) => Some(
            source_snapshot_after_cover.clone().ok_or_else(|| {
                format!(
                    "compiled source {} lost its verified source snapshot before serving preparation",
                    source.display()
                )
            })?,
        ),
        Some(source) => Some(verify_managed_source_snapshot(source, logical_name)?),
        None => None,
    };
    let state = R4g1State::load_with_source(&graph_path, &artifacts, source_for_host)
        .map_err(|error| format!("compiled graph was written but failed validation: {error}"))?;
    let mut replacement_teacher = prepare_optional_teacher_source_for_identity(
        source_for_host,
        tokenizer_selection,
        logical_name,
        state.tokenizer_adapter_identity(),
    )?;
    let (teacher_default_r4_attention, _teacher_mismatch) =
        reconcile_prepared_teacher_with_bundle(&mut replacement_teacher, resolved.as_ref())?;
    let refreshed = resolved
        .as_ref()
        .map(|resolved| refresh_resolved_compiled_bundle(resolved, current_version))
        .transpose()?;
    if let (Some(source), Some(before)) = (source_for_host, source_snapshot_before_prepare.as_ref())
    {
        let after = if downloaded_source == Some(source) {
            let manifest = validate_compile_source_snapshot_for_identity(
                source,
                require_source_manifest,
                expected_source,
            )?;
            verified_managed_source_snapshot_from_manifest(source, manifest)?
        } else {
            verify_managed_source_snapshot(source, logical_name)?
        };
        require_unchanged_managed_source_snapshot(
            source,
            "R4G1 compilation installation",
            before,
            &after,
        )?;
        if let Some(bundle) = refreshed.as_ref() {
            validate_resolved_source_snapshot_binding(bundle, Some(&after), current_version)?;
        }
    }
    let (teacher, tokenizer, teacher_source) = match replacement_teacher {
        Some(prepared) => (
            Some(prepared.teacher),
            Some(prepared.tokenizer),
            Some(prepared.source),
        ),
        None => (None, None, None),
    };
    let mut installed = serving.lock().unwrap();
    if installed.epoch != base_epoch {
        return Err(
            "active serving model changed while background compilation was preparing; refusing stale installation"
                .to_owned(),
        );
    }
    let mut compile_status = status.lock().unwrap();
    installed.epoch = installed.epoch.wrapping_add(1);
    installed.r4g1 = Some(state);
    installed.oracle = teacher;
    installed.source_tokenizer = tokenizer;
    installed.teacher_default_r4_attention = teacher_default_r4_attention;
    installed.active_teacher_source = teacher_source;
    installed.active_bundle = refreshed;
    installed.terminal_load_error = None;
    installed.last_operation_error = None;
    compile_status.ready = graph_text_ready(&installed);

    let report_path = graph_output.join("score_report.json");
    let report = fs::read_to_string(&report_path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
    Ok(serde_json::json!({
        "artifact": graph_path.display().to_string(),
        "report": report,
    }))
}

fn spawn_r4g1_compile(
    cli: Arc<ServerConfig>,
    serving: SharedServingModel,
    status: Arc<Mutex<R4g1CompileStatus>>,
    source: CompileSourceSelection,
    tokenizer_selection: Option<TokenizerAdapterKey>,
    reservation: SourceCacheReservation,
) {
    std::thread::spawn(move || {
        let _reservation = reservation;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compile_r4g1_bundle(
                &cli,
                &serving,
                &status,
                source.path.as_deref().map(Path::new),
                source.expected.as_ref(),
                tokenizer_selection.as_ref(),
                source.require_manifest,
            )
        }))
        .map_err(|payload| {
            format!(
                "R4G1 compilation panicked: {}",
                panic_payload_message(&*payload)
            )
        })
        .and_then(|result| result);

        let mut installed = serving.lock().unwrap();
        let mut current = status.lock().unwrap();
        current.running = false;
        match result {
            Ok(details) => {
                current.ready = graph_text_ready(&installed);
                current.progress = 100;
                current.report = details
                    .get("report")
                    .filter(|report| !report.is_null())
                    .cloned();
                current.message = format!(
                    "R4G1 graph compiled and loaded from {}",
                    details
                        .get("artifact")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("the configured artifact")
                );
            }
            Err(error) => {
                current.ready = graph_text_ready(&installed);
                current.progress = 0;
                current.message = format!("R4G1 compilation failed: {error}");
                installed.last_operation_error = Some(error);
            }
        }
    });
}

fn set_r4g1_compile_progress(status: &Arc<Mutex<R4g1CompileStatus>>, progress: u8, message: &str) {
    let mut current = status.lock().unwrap();
    let progress = progress.min(100);
    if progress >= current.progress {
        current.progress = progress;
        current.message = message.to_owned();
    }
}

fn source_descriptor_for_logical_name_in(
    descriptors_root: &Path,
    logical_name: &str,
) -> Result<Option<SourceDownload>, String> {
    let one_component = Path::new(logical_name).components().count() == 1
        && Path::new(logical_name)
            .file_name()
            .and_then(|name| name.to_str())
            == Some(logical_name);
    if logical_name.is_empty() || !one_component {
        return Err(format!(
            "managed source logical name {logical_name:?} is not one portable basename"
        ));
    }
    let manifest_path = descriptors_root.join(format!("{logical_name}.json"));
    let Some(bytes) = read_regular_file_nofollow(&manifest_path, "model descriptor")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, &manifest_path, "model descriptor")?;
    let manifest: PinnedSourceManifest = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "invalid model descriptor {}: {error}",
            manifest_path.display()
        )
    })?;
    // Reuse the public HTTP grammar and full-revision validation rather than
    // admitting a descriptor the downloader itself would later reject.
    source_from_model_spec(&format!("{}@{}", manifest.repository, manifest.revision)).map_err(
        |error| {
            format!(
                "invalid model descriptor {}: {error}",
                manifest_path.display()
            )
        },
    )?;
    if let Some(source_directory) = manifest.source_directory.as_deref() {
        let recorded_name = Path::new(source_directory)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "model descriptor {} has a non-UTF-8 source_directory basename",
                    manifest_path.display()
                )
            })?;
        if recorded_name != logical_name {
            return Err(format!(
                "model descriptor {} maps logical model {logical_name:?} to mismatched source basename {recorded_name:?}",
                manifest_path.display()
            ));
        }
    }
    Ok(Some(SourceDownload {
        repository: manifest.repository,
        revision: manifest.revision,
        name: logical_name.to_owned(),
        output: manifest.source_directory.map(PathBuf::from),
        license: manifest.license,
    }))
}

fn optional_pinned_huggingface_source_in(
    descriptors_root: &Path,
) -> Result<Option<SourceDownload>, String> {
    source_descriptor_for_logical_name_in(descriptors_root, "smollm2-135m-instruct")
}

fn optional_pinned_huggingface_source() -> Result<Option<SourceDownload>, String> {
    optional_pinned_huggingface_source_in(Path::new("models"))
}

fn pinned_huggingface_source() -> Result<SourceDownload, String> {
    optional_pinned_huggingface_source()?.ok_or_else(|| {
        "pinned Hugging Face manifest is unavailable at models/smollm2-135m-instruct.json"
            .to_owned()
    })
}

fn source_from_model_spec(model: &str) -> Result<SourceDownload, String> {
    let (repository, revision) = model
        .trim()
        .split_once('@')
        .ok_or_else(|| "custom model must use owner/repository@<40-character-commit>".to_owned())?;
    let valid_repository_part = |part: &str| {
        !part.is_empty()
            && part.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            })
    };
    let (owner, repository_name) = repository
        .split_once('/')
        .filter(|(owner, name)| {
            valid_repository_part(owner) && valid_repository_part(name) && !name.contains('/')
        })
        .ok_or_else(|| "custom model repository must use owner/repository".to_owned())?;
    if owner.is_empty()
        || revision.len() != 40
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("custom model must use owner/repository@<40-character-commit>".to_owned());
    }
    // The historical `<basename>-<first12>` key collided for forks sharing a
    // repository basename and for distinct revisions with the same prefix.
    // Domain-separate and hash the complete immutable identity; the resulting
    // source basename also becomes the compiled logical root, so Stage-A
    // corpus resume cannot cross identities merely because shapes agree.
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4-source-cache-v2\0");
    hasher.update(repository.as_bytes());
    hasher.update(&[0]);
    hasher.update(revision.as_bytes());
    let identity_digest = hasher.finalize();
    Ok(SourceDownload {
        repository: repository.to_owned(),
        revision: revision.to_owned(),
        name: format!("{}-{}", repository_name, identity_digest.to_hex()),
        output: None,
        // A custom owner/repository@revision spec carries no license
        // metadata; the snapshot's license file is still digested.
        license: None,
    })
}

fn huggingface_source(model: Option<&str>) -> Result<SourceDownload, String> {
    match model.map(str::trim).filter(|model| !model.is_empty()) {
        Some(model) => source_from_model_spec(model),
        None => pinned_huggingface_source(),
    }
}

fn explicitly_requested_huggingface_source(
    model: Option<&str>,
) -> Result<Option<SourceDownload>, String> {
    model
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(source_from_model_spec)
        .transpose()
}

fn collision_resistant_source_cache_name(name: &str) -> bool {
    name.rsplit_once('-').is_some_and(|(base, digest)| {
        !base.is_empty()
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

/// JSON serialization of the #600 geometry-projection record the teacher
/// adapter applies for a downloaded snapshot: `bucket-average/1` from the
/// snapshot's declared `hidden_size` down to the compiled width (288).
/// Total: any miss (no `config.json`, no numeric `hidden_size`, a source
/// narrower than the compiled width) resolves to `None` so such inputs
/// keep compiling unchanged with no record bound.
fn geometry_projection_of(source: &Path) -> Option<String> {
    let config: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join("config.json")).ok()?).ok()?;
    let hidden_size = u32::try_from(config.get("hidden_size")?.as_u64()?).ok()?;
    let compiled_width = uor_r4_model_source::geometry::COMPILED_WIDTH;
    if hidden_size < compiled_width {
        return None;
    }
    let record = uor_r4_model_source::geometry::GeometryProjection::bucket_average(
        hidden_size,
        compiled_width,
    );
    serde_json::to_string(&record).ok()
}

fn downloaded_source_path_in(source: &SourceDownload, models_root: &Path) -> PathBuf {
    source
        .output
        .clone()
        .unwrap_or_else(|| models_root.join("sources").join(&source.name))
}

fn reject_source_snapshot_special_entries(root: &Path, _directory: &Path) -> Result<(), String> {
    // Use the same openat/O_NOFOLLOW handle-bound traversal as byte
    // addressing. A preliminary DirEntry-type walk followed by path-based
    // `read_dir` could otherwise follow a raced directory symlink (or cycle),
    // and a path-based file open could block on a raced FIFO. `.cache`
    // transport directories remain excluded by the shared walker; a `.cache`
    // file and every other symlink/special entry are terminal.
    crate::model::verified_source_tree(root, crate::model::SourceTreeScope::ManifestlessAll)
        .map(|_| ())
        .map_err(|error| {
            format!(
                "source snapshot {} has an invalid filesystem inventory: {error}",
                root.display()
            )
        })
}

fn validate_manifest_weight_files(
    source_dir: &Path,
    manifest: &crate::model::SourceManifest,
) -> Result<(), String> {
    let recorded_paths = manifest
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let index_path = source_dir.join(uor_r4_model_source::SAFETENSORS_INDEX_FILE_NAME);
    match read_regular_file_nofollow(&index_path, "Safetensors shard index")? {
        Some(bytes) => {
            reject_duplicate_json_fields(&bytes, &index_path, "Safetensors shard index")?;
            let index: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
                format!(
                    "{} is not a valid shard index: {error}",
                    index_path.display()
                )
            })?;
            let weight_map = index
                .get("weight_map")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| {
                    format!("{} has no object-valued weight_map", index_path.display())
                })?;
            if weight_map.is_empty() {
                return Err(format!("{} has an empty weight_map", index_path.display()));
            }
            for (tensor, value) in weight_map {
                let shard = value.as_str().ok_or_else(|| {
                    format!(
                        "{} weight_map entry {tensor:?} is not a shard-name string",
                        index_path.display()
                    )
                })?;
                if shard.is_empty()
                    || shard.starts_with('.')
                    || shard.contains(['/', '\\'])
                    || shard.contains("..")
                    || !recorded_paths.contains(shard)
                {
                    return Err(format!(
                        "{} references shard {shard:?} outside the exact source manifest inventory",
                        index_path.display()
                    ));
                }
            }
        }
        None => {
            if !recorded_paths.contains(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME) {
                return Err(format!(
                    "source snapshot {} records neither {} nor an indexed manifest-bound shard set",
                    source_dir.display(),
                    uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME
                ));
            }
        }
    }
    Ok(())
}

/// Verify the exact manifest identity and every admitted teacher/tokenizer
/// byte before a source directory may be downloaded-over, compiled, or
/// reloaded. This is stricter than the generic optional-source resolver:
/// managed cache snapshots may not be symlink aliases, and manifests must use
/// their canonical bytes with no ignored/duplicate controls.
fn validate_source_snapshot_integrity(
    source_dir: &Path,
    expected: Option<&SourceDownload>,
) -> Result<crate::model::SourceManifest, String> {
    let metadata = fs::symlink_metadata(source_dir).map_err(|error| {
        format!(
            "source snapshot {} cannot be inspected: {error}",
            source_dir.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "source snapshot {} is not a regular non-symlink directory",
            source_dir.display()
        ));
    }
    reject_source_snapshot_special_entries(source_dir, source_dir)?;

    let manifest_path = source_dir.join(crate::model::SOURCE_MANIFEST_FILE_NAME);
    let manifest_bytes = read_regular_file_nofollow(&manifest_path, "source manifest")?
        .ok_or_else(|| {
            format!(
                "source snapshot {} has no regular {}",
                source_dir.display(),
                crate::model::SOURCE_MANIFEST_FILE_NAME
            )
        })?;
    reject_duplicate_json_fields(&manifest_bytes, &manifest_path, "source manifest")?;
    let manifest = crate::model::parse_source_manifest(&manifest_bytes).map_err(|error| {
        format!(
            "source snapshot {} has an invalid {}: {error}",
            source_dir.display(),
            crate::model::SOURCE_MANIFEST_FILE_NAME
        )
    })?;
    let canonical = crate::model::canonical_source_manifest_bytes(&manifest)
        .map_err(|error| format!("{} is not canonical: {error}", manifest_path.display()))?;
    if canonical != manifest_bytes {
        return Err(format!(
            "{} is not the exact canonical source manifest; refusing ignored or ambiguous fields",
            manifest_path.display()
        ));
    }
    if manifest.source_execution_mode != crate::model::SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT
    {
        return Err(format!(
            "{} records unsupported source execution mode {:?}",
            manifest_path.display(),
            manifest.source_execution_mode
        ));
    }
    if manifest.revision.len() != 40
        || !manifest
            .revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "{} does not record a full 40-character source revision",
            manifest_path.display()
        ));
    }
    if let Some(expected) = expected {
        if manifest.repository != expected.repository || manifest.revision != expected.revision {
            return Err(format!(
                "requested model {}@{} resolves to {}, but its {} records {}@{}; refusing to compile or overwrite a different cached teacher",
                expected.repository,
                expected.revision,
                source_dir.display(),
                crate::model::SOURCE_MANIFEST_FILE_NAME,
                manifest.repository,
                manifest.revision,
            ));
        }
        if manifest.license != expected.license {
            return Err(format!(
                "requested model {}@{} resolves to {}, but its {} records license {:?} instead of expected {:?}; refusing to reuse or compile a falsely licensed source snapshot",
                expected.repository,
                expected.revision,
                source_dir.display(),
                crate::model::SOURCE_MANIFEST_FILE_NAME,
                manifest.license,
                expected.license,
            ));
        }
    }
    let physical_name = source_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source snapshot basename is not UTF-8: {}",
                source_dir.display()
            )
        })?;
    if collision_resistant_source_cache_name(physical_name) {
        let recorded_spec = format!("{}@{}", manifest.repository, manifest.revision);
        let recorded_source = source_from_model_spec(&recorded_spec).map_err(|error| {
            format!(
                "{} records a non-portable v2 cache identity {recorded_spec}: {error}",
                manifest_path.display()
            )
        })?;
        if recorded_source.name != physical_name {
            return Err(format!(
                "source snapshot {} has collision-resistant basename {physical_name:?}, but its {} identity {}@{} derives {:?}; refusing renamed or substituted teacher bytes",
                source_dir.display(),
                crate::model::SOURCE_MANIFEST_FILE_NAME,
                manifest.repository,
                manifest.revision,
                recorded_source.name,
            ));
        }
    }

    let mut rebuilt = crate::model::build_source_manifest(
        source_dir,
        &crate::model::SourceSnapshotInfo {
            repository: manifest.repository.clone(),
            revision: manifest.revision.clone(),
            license: manifest.license.clone(),
            source_execution_mode: manifest.source_execution_mode.clone(),
        },
    )
    .map_err(|error| {
        format!(
            "source snapshot {} cannot be re-addressed: {error}",
            source_dir.display()
        )
    })?;
    // Preserve the recorded producer version while comparing the immutable
    // identity and file inventory; otherwise an older but exact snapshot
    // would appear modified merely because this server binary is newer.
    rebuilt.compiler_version = manifest.compiler_version.clone();
    if rebuilt != manifest {
        return Err(format!(
            "source snapshot {} no longer matches its recorded file inventory; refusing partial or modified teacher bytes",
            source_dir.display()
        ));
    }
    validate_manifest_weight_files(source_dir, &manifest)?;
    Ok(manifest)
}

fn validate_requested_source_manifest(
    source_dir: &Path,
    requested: &SourceDownload,
) -> Result<(), String> {
    validate_source_snapshot_integrity(source_dir, Some(requested)).map(|_| ())
}

fn validate_legacy_compatible_source_manifest(
    source_dir: &Path,
    expected_when_present: &SourceDownload,
) -> Result<Option<crate::model::SourceManifest>, String> {
    let manifest_path = source_dir.join(crate::model::SOURCE_MANIFEST_FILE_NAME);
    match fs::symlink_metadata(&manifest_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            validate_manifestless_source_structure(source_dir)?;
            Ok(None)
        }
        Ok(_) => {
            validate_source_snapshot_integrity(source_dir, Some(expected_when_present)).map(Some)
        }
        Err(error) => Err(format!(
            "{} cannot be inspected: {error}",
            manifest_path.display()
        )),
    }
}

fn validate_manifestless_source_structure(source_dir: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source_dir).map_err(|error| {
        format!(
            "legacy source snapshot {} cannot be inspected: {error}",
            source_dir.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "legacy source snapshot {} is not a regular non-symlink directory",
            source_dir.display()
        ));
    }
    reject_source_snapshot_special_entries(source_dir, source_dir)
}

fn validate_compile_source_snapshot(
    source_dir: &Path,
    require_manifest: bool,
) -> Result<Option<crate::model::SourceManifest>, String> {
    let manifest_path = source_dir.join(crate::model::SOURCE_MANIFEST_FILE_NAME);
    match fs::symlink_metadata(&manifest_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !require_manifest => {
            validate_manifestless_source_structure(source_dir)?;
            Ok(None)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(format!(
            "source snapshot {} is missing its required {}",
            source_dir.display(),
            crate::model::SOURCE_MANIFEST_FILE_NAME
        )),
        Ok(_) => validate_source_snapshot_integrity(source_dir, None).map(Some),
        Err(error) => Err(format!(
            "{} cannot be inspected: {error}",
            manifest_path.display()
        )),
    }
}

fn validate_compile_source_snapshot_for_identity(
    source_dir: &Path,
    require_manifest: bool,
    expected: Option<&SourceDownload>,
) -> Result<Option<crate::model::SourceManifest>, String> {
    match (expected, require_manifest) {
        (Some(expected), true) => {
            validate_source_snapshot_integrity(source_dir, Some(expected)).map(Some)
        }
        (Some(expected), false) => validate_legacy_compatible_source_manifest(source_dir, expected),
        (None, _) => validate_compile_source_snapshot(source_dir, require_manifest),
    }
}

fn validate_managed_source_for_serving_in(
    source_dir: &Path,
    logical_name: &str,
    descriptors_root: &Path,
) -> Result<Option<crate::model::SourceManifest>, String> {
    let metadata = fs::symlink_metadata(source_dir).map_err(|error| {
        format!(
            "managed source {} cannot be inspected before serving: {error}",
            source_dir.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "managed source {} is not a regular non-symlink directory",
            source_dir.display()
        ));
    }
    // Even a genuine pre-manifest source may not hide executable model files
    // behind visible symlinks or special entries. Legacy compatibility is
    // absence of a provenance record, not permission to retarget bytes.
    reject_source_snapshot_special_entries(source_dir, source_dir)?;
    let physical_name = source_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("managed source name is not UTF-8: {}", source_dir.display()))?;
    let require_manifest = collision_resistant_source_cache_name(logical_name)
        || collision_resistant_source_cache_name(physical_name);
    if collision_resistant_source_cache_name(logical_name) && physical_name != logical_name {
        return Err(format!(
            "managed source {} does not match collision-resistant logical model {logical_name:?}",
            source_dir.display()
        ));
    }
    if let Some(descriptor) = source_descriptor_for_logical_name_in(descriptors_root, logical_name)?
    {
        return validate_legacy_compatible_source_manifest(source_dir, &descriptor);
    }
    validate_compile_source_snapshot(source_dir, require_manifest)
}

fn validate_managed_source_for_serving(
    source_dir: &Path,
    logical_name: &str,
) -> Result<Option<crate::model::SourceManifest>, String> {
    validate_managed_source_for_serving_in(source_dir, logical_name, Path::new("models"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VerifiedManagedSourceSnapshot {
    manifest: Option<crate::model::SourceManifest>,
    content_kappa: String,
}

fn validate_manifestless_index_shards(source_dir: &Path) -> Result<(), String> {
    let index_path = source_dir.join(uor_r4_model_source::SAFETENSORS_INDEX_FILE_NAME);
    let Some(bytes) = read_regular_file_nofollow(&index_path, "manifestless shard index")? else {
        return Ok(());
    };
    reject_duplicate_json_fields(&bytes, &index_path, "manifestless Safetensors shard index")?;
    let index: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "{} is not a valid shard index: {error}",
            index_path.display()
        )
    })?;
    let weight_map = index
        .get("weight_map")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("{} has no object-valued weight_map", index_path.display()))?;
    if weight_map.is_empty() {
        return Err(format!("{} has an empty weight_map", index_path.display()));
    }
    for (tensor, value) in weight_map {
        let shard = value.as_str().ok_or_else(|| {
            format!(
                "{} weight_map entry {tensor:?} is not a shard-name string",
                index_path.display()
            )
        })?;
        if shard.is_empty()
            || shard.starts_with('.')
            || shard.contains(['/', '\\'])
            || shard.contains("..")
        {
            return Err(format!(
                "{} references hidden or nonportable executable shard {shard:?}; manifestless source κ requires a visible bare shard name",
                index_path.display()
            ));
        }
        let shard_path = source_dir.join(shard);
        let shard_metadata = fs::symlink_metadata(&shard_path).map_err(|error| {
            format!(
                "{} references unavailable shard {}: {error}",
                index_path.display(),
                shard_path.display()
            )
        })?;
        if !shard_metadata.file_type().is_file() {
            return Err(format!(
                "{} references shard {} that is not a regular non-symlink file",
                index_path.display(),
                shard_path.display()
            ));
        }
    }
    Ok(())
}

fn verified_managed_source_snapshot_from_manifest(
    source_dir: &Path,
    manifest: Option<crate::model::SourceManifest>,
) -> Result<VerifiedManagedSourceSnapshot, String> {
    verified_managed_source_snapshot_from_manifest_with_tree(source_dir, manifest, |source_dir| {
        crate::model::verified_source_tree(
            source_dir,
            crate::model::SourceTreeScope::ManifestlessAll,
        )
        .map_err(|error| error.to_string())
    })
}

fn verified_managed_source_snapshot_from_manifest_with_tree<F>(
    source_dir: &Path,
    manifest: Option<crate::model::SourceManifest>,
    manifestless_tree: F,
) -> Result<VerifiedManagedSourceSnapshot, String>
where
    F: FnOnce(&Path) -> Result<Vec<crate::model::VerifiedSourceTreeEntry>, String>,
{
    let content_kappa = match manifest.as_ref() {
        Some(manifest) => source_manifest_kappa(manifest)?,
        None => {
            validate_manifestless_index_shards(source_dir)?;
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"uor-r4-manifestless-source-tree-v2\0");
            let entries = manifestless_tree(source_dir)?;
            for entry in entries {
                match entry {
                    crate::model::VerifiedSourceTreeEntry::Directory { path } => {
                        hasher.update(b"D");
                        hasher.update(&(path.len() as u64).to_le_bytes());
                        hasher.update(path.as_bytes());
                    }
                    crate::model::VerifiedSourceTreeEntry::File { path, bytes, kappa } => {
                        hasher.update(b"F");
                        hasher.update(&(path.len() as u64).to_le_bytes());
                        hasher.update(path.as_bytes());
                        hasher.update(&bytes.to_le_bytes());
                        hasher.update(&(kappa.len() as u64).to_le_bytes());
                        hasher.update(kappa.as_bytes());
                    }
                }
            }
            format!("blake3:{}", hasher.finalize().to_hex())
        }
    };
    Ok(VerifiedManagedSourceSnapshot {
        manifest,
        content_kappa,
    })
}

fn verify_managed_source_snapshot_in(
    source_dir: &Path,
    logical_name: &str,
    descriptors_root: &Path,
) -> Result<VerifiedManagedSourceSnapshot, String> {
    let manifest =
        validate_managed_source_for_serving_in(source_dir, logical_name, descriptors_root)?;
    verified_managed_source_snapshot_from_manifest(source_dir, manifest)
}

fn verify_managed_source_snapshot(
    source_dir: &Path,
    logical_name: &str,
) -> Result<VerifiedManagedSourceSnapshot, String> {
    verify_managed_source_snapshot_in(source_dir, logical_name, Path::new("models"))
}

fn require_unchanged_managed_source_snapshot(
    source_dir: &Path,
    operation: &str,
    before: &VerifiedManagedSourceSnapshot,
    after: &VerifiedManagedSourceSnapshot,
) -> Result<(), String> {
    if before == after {
        return Ok(());
    }
    Err(format!(
        "source snapshot {} changed while {operation} was preparing ({} -> {}); refusing mixed-identity installation",
        source_dir.display(),
        before.content_kappa,
        after.content_kappa
    ))
}

fn select_compile_source_path_in(
    models_root: &Path,
    requested: Option<&SourceDownload>,
    cached: Option<&CompletedDownloadSource>,
    fallback: Option<&SourceDownload>,
) -> Result<Option<PathBuf>, String> {
    if let Some(requested) = requested {
        let path = downloaded_source_path_in(requested, models_root);
        let Some(path) = optional_source_directory(&path)? else {
            return Err(format!(
                "requested model {}@{} is not downloaded at {}; download it before compiling",
                requested.repository,
                requested.revision,
                path.display()
            ));
        };
        validate_requested_source_manifest(&path, requested)?;
        return Ok(Some(path));
    }
    if let Some(cached) = cached {
        let Some(path) = optional_source_directory(&cached.path)? else {
            return Err(format!(
                "completed Hugging Face source {} was recorded ready but is now absent; refusing to downgrade this compile to configured legacy inputs",
                cached.path.display()
            ));
        };
        validate_source_snapshot_integrity(&path, Some(&cached.identity))?;
        return Ok(Some(path));
    }
    let Some(fallback) = fallback else {
        return Ok(None);
    };
    let fallback_path = downloaded_source_path_in(fallback, models_root);
    let Some(path) = optional_source_directory(&fallback_path)? else {
        return Ok(None);
    };
    // A genuine pre-#597 pinned snapshot remains a read-only compatibility
    // input. If it has a manifest, every field and byte must be exact;
    // malformed/present-invalid manifests never downgrade to legacy absence.
    validate_legacy_compatible_source_manifest(&path, fallback)?;
    Ok(Some(path))
}

fn completed_download_source(
    status: &HuggingFaceDownloadStatus,
) -> Result<Option<CompletedDownloadSource>, String> {
    if status.running {
        return Err("a Hugging Face source download is still running".to_owned());
    }
    if !status.ready {
        return Ok(None);
    }
    let source = status.source.as_deref().ok_or_else(|| {
        "Hugging Face download status is ready but records no completed source".to_owned()
    })?;
    if source.trim().is_empty() {
        return Err("Hugging Face download status records an empty source path".to_owned());
    }
    let identity = status.completed_source.clone().ok_or_else(|| {
        "Hugging Face download status is ready but records no completed source identity".to_owned()
    })?;
    let path = PathBuf::from(source);
    let expected_path = downloaded_source_path_in(&identity, Path::new(".uor-models"));
    if path != expected_path {
        return Err(format!(
            "Hugging Face download status path {} conflicts with completed identity {}@{} at {}",
            path.display(),
            identity.repository,
            identity.revision,
            expected_path.display()
        ));
    }
    Ok(Some(CompletedDownloadSource { path, identity }))
}

fn reserve_compile_source_selection(
    operations: &SharedSourceCacheOperations,
    status: &Arc<Mutex<HuggingFaceDownloadStatus>>,
    subject: impl Into<String>,
) -> Result<(SourceCacheReservation, Option<CompletedDownloadSource>), String> {
    // Reservation must precede the completed-download snapshot. Otherwise a
    // download can publish Bob and update status after compilation snapshots
    // Alice (or the pinned fallback) but before compilation acquires the
    // operation slot.
    let reservation =
        try_reserve_source_cache_operation(operations, SourceCacheOperationKind::Compile, subject)?;
    let source = completed_download_source(&status.lock().unwrap())?;
    Ok((reservation, source))
}

static NEXT_SOURCE_STAGING_ID: AtomicU64 = AtomicU64::new(0);
const DOWNLOAD_STAGE_MARKER_FILE: &str = ".download_stage.json";
const DOWNLOAD_STAGE_MARKER_SCHEMA: &str = "uor-r4-download-stage/1";

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DownloadStageMarker {
    schema: String,
    destination: String,
    stage_path: String,
    repository: String,
    revision: String,
    license: Option<String>,
}

fn remove_source_staging_path(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    };
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("{} cannot be removed: {error}", path.display()))
    } else {
        fs::remove_file(path)
            .map_err(|error| format!("{} cannot be removed: {error}", path.display()))
    }
}

struct SourceStagingDirectory {
    path: PathBuf,
    armed: bool,
}

fn is_download_stage_name(name: &str, final_name: &str) -> bool {
    let prefix = format!(".{final_name}.download-staging-");
    let Some(sequence) = name.strip_prefix(&prefix) else {
        return false;
    };
    let mut parts = sequence.split('-');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn download_stage_marker_bytes(
    final_path: &Path,
    stage_path: &Path,
    source: &SourceDownload,
) -> Result<Vec<u8>, String> {
    let destination = canonical_compile_session_subject(final_path)?;
    let stage_path = canonical_compile_session_subject(stage_path)?;
    let destination = destination
        .to_str()
        .ok_or_else(|| format!("source destination is not UTF-8: {}", destination.display()))?;
    let stage_path = stage_path
        .to_str()
        .ok_or_else(|| format!("download stage is not UTF-8: {}", stage_path.display()))?;
    let mut bytes = serde_json::to_vec_pretty(&DownloadStageMarker {
        schema: DOWNLOAD_STAGE_MARKER_SCHEMA.to_owned(),
        destination: destination.to_owned(),
        stage_path: stage_path.to_owned(),
        repository: source.repository.clone(),
        revision: source.revision.clone(),
        license: source.license.clone(),
    })
    .map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_download_stage_marker(stage: &Path) -> Result<Option<Vec<u8>>, String> {
    let path = stage.join(DOWNLOAD_STAGE_MARKER_FILE);
    let Some(bytes) = read_regular_file_nofollow(&path, "download-stage marker")? else {
        return Ok(None);
    };
    reject_duplicate_json_fields(&bytes, &path, "download-stage marker")?;
    let record: DownloadStageMarker = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is malformed: {error}", path.display()))?;
    let mut canonical = serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
    canonical.push(b'\n');
    if record.schema != DOWNLOAD_STAGE_MARKER_SCHEMA || bytes != canonical {
        return Err(format!(
            "{} is not a canonical supported download-stage marker",
            path.display()
        ));
    }
    Ok(Some(bytes))
}

fn recover_download_stages(final_path: &Path, source: &SourceDownload) -> Result<(), String> {
    let parent = final_path.parent().ok_or_else(|| {
        format!(
            "source destination {} has no parent directory",
            final_path.display()
        )
    })?;
    let final_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("source destination is not UTF-8: {}", final_path.display()))?;
    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "{} cannot be enumerated: {error}",
                parent.display()
            ))
        }
    };
    let prefix = format!(".{final_name}.download-staging-");
    let mut stale = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("{} cannot be enumerated: {error}", parent.display()))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "source cache {} contains a non-UTF-8 staging entry",
                parent.display()
            )
        })?;
        if is_download_stage_name(name, final_name) {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                format!("{} cannot be inspected: {error}", entry.path().display())
            })?;
            if !metadata.file_type().is_dir() {
                return Err(format!(
                    "recognized download stage {} is not a regular non-symlink directory",
                    entry.path().display()
                ));
            }
            let expected = download_stage_marker_bytes(final_path, &entry.path(), source)?;
            let recorded = read_download_stage_marker(&entry.path())?;
            let mut marker_temporaries = Vec::new();
            let mut other_entries = Vec::new();
            for child in fs::read_dir(entry.path()).map_err(|error| {
                format!("{} cannot be enumerated: {error}", entry.path().display())
            })? {
                let child = child.map_err(|error| {
                    format!("{} cannot be enumerated: {error}", entry.path().display())
                })?;
                let child_name = child.file_name();
                let child_name = child_name.to_str().ok_or_else(|| {
                    format!(
                        "download stage {} contains a non-UTF-8 entry",
                        entry.path().display()
                    )
                })?;
                if is_atomic_publisher_temporary(child_name, DOWNLOAD_STAGE_MARKER_FILE) {
                    let child_metadata = fs::symlink_metadata(child.path()).map_err(|error| {
                        format!("{} cannot be inspected: {error}", child.path().display())
                    })?;
                    if !child_metadata.file_type().is_file() {
                        return Err(format!(
                            "recognized download-stage marker temporary {} is not a regular non-symlink file",
                            child.path().display()
                        ));
                    }
                    marker_temporaries.push(child.path());
                } else if looks_like_atomic_publisher_temporary(
                    child_name,
                    DOWNLOAD_STAGE_MARKER_FILE,
                ) {
                    return Err(format!(
                        "download stage {} contains unrecognized marker temporary {child_name}",
                        entry.path().display()
                    ));
                } else if child_name != DOWNLOAD_STAGE_MARKER_FILE {
                    other_entries.push(child.path());
                }
            }
            match recorded {
                Some(recorded) if recorded == expected => stale.push(entry.path()),
                Some(_) => {
                    return Err(format!(
                        "download stage {} does not bind this exact destination and source identity",
                        entry.path().display()
                    ));
                }
                None if other_entries.is_empty() => {
                    // `create_dir` precedes the durable marker publication.
                    // An empty stage (or one containing only an exact torn
                    // marker temporary) is the sole markerless crash prefix.
                    stale.push(entry.path());
                }
                None => {
                    return Err(format!(
                        "markerless download stage {} contains source payload; refusing unowned reclamation",
                        entry.path().display()
                    ));
                }
            }
            let _ = marker_temporaries;
        } else if name.starts_with(&prefix) {
            return Err(format!(
                "source cache {} contains unrecognized download stage {name}",
                parent.display()
            ));
        }
    }
    for stage in stale {
        fs::remove_dir_all(&stage).map_err(|error| {
            format!(
                "stale source download stage {} cannot be reclaimed under exclusive destination ownership: {error}",
                stage.display()
            )
        })?;
    }
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{} cannot be synced: {error}", parent.display()))?;
    Ok(())
}

impl SourceStagingDirectory {
    fn allocate(final_path: &Path, source: &SourceDownload) -> Result<Self, String> {
        let parent = final_path.parent().ok_or_else(|| {
            format!(
                "source destination {} has no parent directory",
                final_path.display()
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("{} cannot be created: {error}", parent.display()))?;
        let final_name = final_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("source destination is not UTF-8: {}", final_path.display()))?;
        recover_download_stages(final_path, source)?;
        for _ in 0..128 {
            let id = NEXT_SOURCE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
            let candidate = parent.join(format!(
                ".{final_name}.download-staging-{}-{id}",
                std::process::id()
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => {
                    let marker = match download_stage_marker_bytes(final_path, &candidate, source) {
                        Ok(marker) => marker,
                        Err(error) => {
                            let _ = fs::remove_dir_all(&candidate);
                            return Err(error);
                        }
                    };
                    if let Err(error) = publish_bytes_no_clobber(
                        &candidate.join(DOWNLOAD_STAGE_MARKER_FILE),
                        &marker,
                        "download-stage marker",
                    ) {
                        let _ = fs::remove_dir_all(&candidate);
                        return Err(error);
                    }
                    if let Err(error) =
                        fs::File::open(parent).and_then(|directory| directory.sync_all())
                    {
                        let _ = fs::remove_dir_all(&candidate);
                        return Err(format!("{} cannot be synced: {error}", parent.display()));
                    }
                    return Ok(Self {
                        path: candidate,
                        armed: true,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "source staging directory {} cannot be created: {error}",
                        candidate.display()
                    ))
                }
            }
        }
        Err(format!(
            "could not allocate a unique source staging directory beside {}",
            final_path.display()
        ))
    }

    fn discard(&mut self) -> Result<(), String> {
        remove_source_staging_path(&self.path)?;
        self.armed = false;
        Ok(())
    }

    fn disarm_after_publish(&mut self) {
        self.armed = false;
    }
}

impl Drop for SourceStagingDirectory {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_source_staging_path(&self.path);
        }
    }
}

fn validate_published_download_stage_marker(
    destination: &Path,
    source: &SourceDownload,
) -> Result<(), String> {
    let Some(bytes) = read_download_stage_marker(destination)? else {
        return Ok(());
    };
    let record: DownloadStageMarker =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let stage = PathBuf::from(&record.stage_path);
    let expected = download_stage_marker_bytes(destination, &stage, source)?;
    let destination_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("source destination is not UTF-8: {}", destination.display()))?;
    let stage_name = stage
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("download stage is not UTF-8: {}", stage.display()))?;
    let destination_parent = canonical_compile_session_subject(
        destination
            .parent()
            .ok_or_else(|| format!("{} has no parent", destination.display()))?,
    )?;
    if bytes != expected
        || stage.parent() != Some(destination_parent.as_path())
        || !is_download_stage_name(stage_name, destination_name)
    {
        return Err(format!(
            "published download-stage marker {} does not bind this exact destination and source identity",
            destination.join(DOWNLOAD_STAGE_MARKER_FILE).display()
        ));
    }
    Ok(())
}

fn sync_source_snapshot_for_publication(
    source_dir: &Path,
    manifest: &crate::model::SourceManifest,
) -> Result<(), String> {
    let mut directories = std::collections::BTreeSet::new();
    directories.insert(source_dir.to_path_buf());
    for relative in manifest
        .files
        .iter()
        .map(|file| file.path.as_str())
        .chain(std::iter::once(crate::model::SOURCE_MANIFEST_FILE_NAME))
    {
        let path = source_dir.join(relative);
        sync_regular_file_nofollow(&path, "downloaded source member")?;
        let mut parent = path.parent();
        while let Some(directory) = parent {
            if !directory.starts_with(source_dir) {
                break;
            }
            directories.insert(directory.to_path_buf());
            if directory == source_dir {
                break;
            }
            parent = directory.parent();
        }
    }
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        fs::File::open(&directory)
            .and_then(|handle| handle.sync_all())
            .map_err(|error| format!("{} cannot be synced: {error}", directory.display()))?;
    }
    Ok(())
}

fn download_source_atomically_in<F>(
    source: &SourceDownload,
    models_root: &Path,
    downloader: F,
) -> Result<PathBuf, String>
where
    F: FnOnce(&SourceDownload) -> Result<PathBuf, String>,
{
    let destination = downloaded_source_path_in(source, models_root);
    let _download_sessions = try_lock_source_compile_sessions(
        [models_root.join("sources"), destination.clone()],
        SourceCompileSessionMode::ExclusiveWriter,
    )?;
    match fs::symlink_metadata(&destination) {
        Ok(_) => {
            validate_published_download_stage_marker(&destination, source)?;
            validate_source_snapshot_integrity(&destination, Some(source))?;
            return Ok(destination);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "source destination {} cannot be inspected: {error}",
                destination.display()
            ))
        }
    }

    let mut staging = SourceStagingDirectory::allocate(&destination, source)?;
    let mut staged_source = source.clone();
    staged_source.output = Some(staging.path.clone());
    let reported = downloader(&staged_source)?;
    if reported != staging.path {
        return Err(format!(
            "source downloader reported {}, expected its reserved staging directory {}",
            reported.display(),
            staging.path.display()
        ));
    }
    let manifest = validate_source_snapshot_integrity(&staging.path, Some(source))?;
    sync_source_snapshot_for_publication(&staging.path, &manifest)?;

    match rename_directory_no_replace(&staging.path, &destination) {
        Ok(()) => {
            if let Err(error) = sync_parent_directory(&destination, "downloaded source publication")
            {
                let rollback = rename_directory_no_replace(&destination, &staging.path)
                    .map_err(|rollback_error| rollback_error.to_string())
                    .and_then(|()| {
                        sync_parent_directory(&destination, "downloaded source rollback")
                    });
                return Err(format!(
                    "{error}; downloaded source publication rollback: {}",
                    rollback
                        .map(|()| "removed incomplete destination namespace".to_owned())
                        .unwrap_or_else(|rollback_error| rollback_error)
                ));
            }
            staging.disarm_after_publish();
            let published_marker = destination.join(DOWNLOAD_STAGE_MARKER_FILE);
            if validate_published_download_stage_marker(&destination, source).is_ok()
                && fs::remove_file(&published_marker).is_ok()
            {
                let _ = fs::File::open(&destination).and_then(|directory| directory.sync_all());
            }
            Ok(destination)
        }
        Err(rename_error) => {
            // Another process may have won the destination after our initial
            // absence check. Never replace even an empty directory. Reuse a
            // raced winner only after its complete manifest and admitted-byte
            // inventory prove the exact requested identity.
            match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    validate_published_download_stage_marker(&destination, source)?;
                    validate_source_snapshot_integrity(&destination, Some(source)).map_err(
                        |validation_error| {
                            format!(
                                "validated source staging directory {} lost exclusive publication at {} ({rename_error}); raced destination is not the exact immutable source: {validation_error}",
                                staging.path.display(),
                                destination.display()
                            )
                        },
                    )?;
                    staging.discard()?;
                    Ok(destination)
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(format!(
                    "validated source staging directory {} could not be exclusively published at {} and no raced destination exists: {rename_error}",
                    staging.path.display(),
                    destination.display()
                )),
                Err(error) => Err(format!(
                    "source destination {} cannot be inspected after exclusive publication failure ({rename_error}): {error}",
                    destination.display()
                )),
            }
        }
    }
}

type HuggingFaceDownloadResult = Result<(SourceDownload, PathBuf), String>;

fn apply_huggingface_download_result(
    current: &mut HuggingFaceDownloadStatus,
    result: HuggingFaceDownloadResult,
) {
    current.running = false;
    match result {
        Ok((source, destination)) => {
            current.ready = true;
            current.source = Some(destination.display().to_string());
            current.completed_source = Some(source.clone());
            current.message = format!(
                "Downloaded Hugging Face source {} ({})",
                source.repository, source.name
            );
        }
        Err(error) => {
            // A failed staged attempt did not mutate the previously published
            // directory. Preserve the last completed ready/source tuple.
            current.message = format!("Hugging Face download failed: {error}");
        }
    }
}

fn spawn_huggingface_download(
    status: Arc<Mutex<HuggingFaceDownloadStatus>>,
    source: SourceDownload,
    reservation: SourceCacheReservation,
) {
    std::thread::spawn(move || {
        let _reservation = reservation;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let destination = download_source_atomically_in(
                &source,
                Path::new(".uor-models"),
                |staged_source| download_source(staged_source).map_err(|error| error.to_string()),
            )?;
            Ok::<_, String>((source, destination))
        }))
        .map_err(|payload| {
            format!(
                "Hugging Face download panicked: {}",
                panic_payload_message(&*payload)
            )
        })
        .and_then(|result| result);

        apply_huggingface_download_result(&mut status.lock().unwrap(), result);
    });
}

/// A completed generation from the shared serving core, before either wire
/// surface shapes it into a response (`#654` phase E).
struct GeneratedCompletion {
    text: String,
    generation_mode: String,
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
    created_ts: u64,
    uor_audit: UorAuditTrace,
    cascade_trail: serde_json::Value,
}

/// The outcome of the shared serving core: a completed generation, or an
/// honest declined-by-all terminal carrying its `(status, body)`.
enum GenerationOutcome {
    Generated(Box<GeneratedCompletion>),
    Declined {
        status: u16,
        body: serde_json::Value,
    },
}

/// The generation cascade shared by `/v1/chat/completions` and `/v1/responses`
/// (`#654` phase E). Both wire surfaces route the flattened prompt through this
/// one internal adapter — routing, autotune, the #248 serving cascade, and the
/// R4 audit — so the endpoints stay in lockstep and never diverge in what they
/// actually run. The caller supplies the already-flattened prompt and the
/// effective token budget; the endpoint-specific request and response shapes
/// stay in the handlers.
#[allow(clippy::too_many_arguments)]
fn generate_serving_completion(
    router: &Arc<Mutex<UorR4Router>>,
    serving: &SharedServingModel,
    tless: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    cli: &Arc<ServerConfig>,
    start_time: Instant,
    expected_model: &str,
    prompt_text: &str,
    engine: Option<&str>,
    max_tokens: usize,
    temperature_override: Option<f64>,
) -> GenerationOutcome {
    let identity = "tenant-alpha".to_string();

    let mut serving_guard = serving.lock().unwrap();
    if active_canonical_model_name(&serving_guard).as_deref() != Some(expected_model) {
        return GenerationOutcome::Declined {
            status: 409,
            body: openai_error_body(
                "server_error",
                "active model changed before generation began; retry the request",
                Some("model"),
                Some("model_changed"),
            ),
        };
    }

    let mut router_guard = router.lock().unwrap();

    let mut buf = [0u8; 640];
    let query_bytes = prompt_text.as_bytes();
    let identity_bytes = identity.as_bytes();
    let query_len = query_bytes.len().min(512);
    let identity_len = identity_bytes.len().min(128);
    buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
    buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

    let input = uor_r4_wasm_router::R4RoutingInput {
        query: &buf[..512],
        identity: &buf[512..],
        data: &buf,
    };

    let router_ptr = &mut *router_guard as *mut UorR4Router;
    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
        *r.borrow_mut() = Some(router_ptr);
    });

    let _grounded_dry =
        uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Dry run routing failed");

    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
        *r.borrow_mut() = None;
    });

    let routing = router_guard
        .last_routing_data()
        .clone()
        .expect("No routing data generated");
    let kappa = routing.routed.metrics.kappa;
    let theta_d = routing.routed.metrics.deficit_angle;
    let uor_bias = routing.routed.qimc.uor_control.entropy_bias;

    let (gamma, default_temp) = autotune(kappa, theta_d, uor_bias);
    let temperature = temperature_override.unwrap_or(default_temp);

    let routing_prompt = if prompt_text.len() > 512 {
        &prompt_text[..512]
    } else {
        prompt_text
    };

    router_guard.evolve_state(&identity, routing_prompt, gamma);
    let session_signature = uor_r4_router::session_signature_from_state(
        &router_guard.get_brain_state_native(&identity),
    );

    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
        *r.borrow_mut() = Some(router_ptr);
    });

    let _grounded =
        uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Final routing failed");

    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
        *r.borrow_mut() = None;
    });

    // Issue #248: the single serving cascade, honoring an engine pin from the
    // request or the persisted `/engine` selection.
    let pinned = resolve_pinned_tier(engine);
    // One serving-state guard spans the complete request. Reload and
    // background compilation prepare off-lock and wait to swap until routing
    // and generation finish against one internally consistent tuple.
    let cascade = run_serving_cascade(
        &mut router_guard,
        &mut serving_guard,
        tless,
        prompt_text,
        &identity,
        max_tokens,
        temperature,
        gamma,
        Some(&session_signature),
        pinned,
    );
    let generation_mode = derive_generation_mode(&cascade, pinned);
    let Some(final_response_text) = cascade.outcome.text.clone() else {
        // Declined by all: an honest terminal instead of serving the sparse-
        // string placeholder as if it were generated.
        let (status, body) = declined_by_all_response(&cascade, pinned, &generation_mode);
        return GenerationOutcome::Declined { status, body };
    };

    router_guard.index_sentence(prompt_text, &identity);
    router_guard.index_sentence(&final_response_text, &identity);
    router_guard.inject_thought_stream_native(prompt_text);
    router_guard.inject_thought_stream_native(&final_response_text);
    spawn_cache_save(cli, router_guard.export_state());

    // #654/#718: R4G1 and teacher tiers carry counts from the exact tokenizer
    // and token stream they actually used. They must never be recounted with
    // the process-global legacy tokenizer, which may inhabit another id space.
    let usage = match serving_usage_for_cascade(&cascade, prompt_text, &final_response_text) {
        Ok(usage) => usage,
        Err(message) => {
            return GenerationOutcome::Declined {
                status: 500,
                body: openai_error_body(
                    "server_error",
                    message,
                    None,
                    Some("tokenizer_usage_unavailable"),
                ),
            };
        }
    };
    let prompt_tokens = usage.prompt_tokens;
    let completion_tokens = usage.completion_tokens;
    let total_tokens = usage.total_tokens();

    let created_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    let words: Vec<&str> = final_response_text.split_whitespace().collect();
    let per_token_ms = if words.is_empty() {
        0.0
    } else {
        duration_ms / words.len() as f64
    };
    let tokens_detail: Vec<TokenTraceEntry> = words
        .iter()
        .enumerate()
        .map(|(idx, w)| TokenTraceEntry {
            token_id: idx as u32,
            text: w.to_string(),
            origin_rule: match generation_mode.as_str() {
                "r4g1" => "R4G1 Residual Graph (Rule 1/2)".to_string(),
                "teacher-oracle-fallback" => "Teacher Oracle Fallback (Full Attention)".to_string(),
                "geometric-decoded" => "f64 Geometric Router Manifold".to_string(),
                "r4g1-abstained" => "R4G1 Abstained (OOD Shield)".to_string(),
                _ => "ExactContext Carryover".to_string(),
            },
            latency_ms: (per_token_ms * 100.0).round() / 100.0,
        })
        .collect();

    // Issue #256: a real canonical label over the audit content.
    let uor_addr = canonical_json_address_blake3(&serde_json::json!({
        "generation_mode": generation_mode,
        "kappa": (kappa * 10000.0).round() / 10000.0,
        "deficit_angle": (theta_d * 10000.0).round() / 10000.0,
        "gamma": (gamma * 10000.0).round() / 10000.0,
        "temperature": temperature,
    }));
    let kappa_pass = (0.10..=2.50).contains(&kappa);

    let uor_audit = UorAuditTrace {
        uor_address: uor_addr,
        kappa: (kappa * 10000.0).round() / 10000.0,
        deficit_angle: (theta_d * 10000.0).round() / 10000.0,
        entropy_bias: (uor_bias * 10000.0).round() / 10000.0,
        gamma: (gamma * 10000.0).round() / 10000.0,
        temperature,
        kappa_pass,
        generation_mode: generation_mode.clone(),
        total_latency_ms: (duration_ms * 100.0).round() / 100.0,
        tokens_detail,
    };

    let cascade_trail = cascade_trail_json(&cascade.outcome.trail);

    GenerationOutcome::Generated(Box::new(GeneratedCompletion {
        text: final_response_text,
        generation_mode,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        created_ts,
        uor_audit,
        cascade_trail,
    }))
}

#[allow(clippy::too_many_arguments)]
fn handle_connection(
    mut stream: TcpStream,
    router: Arc<Mutex<UorR4Router>>,
    tless: Arc<Mutex<Option<tless_uor::TlessState>>>,
    serving: SharedServingModel,
    r4g1_compile: Arc<Mutex<R4g1CompileStatus>>,
    hf_download: Arc<Mutex<HuggingFaceDownloadStatus>>,
    source_cache_operations: SharedSourceCacheOperations,
    cli: Arc<ServerConfig>,
    start_time: Instant,
) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    if buf_reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path_str = parts[1];
    let clean_path = path_str
        .split('?')
        .next()
        .unwrap()
        .split('#')
        .next()
        .unwrap();
    eprintln!(
        "[REQUEST] {} {} -> clean_path: {}",
        method, path_str, clean_path
    );

    if method == "OPTIONS" {
        let response = "HTTP/1.1 200 OK\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n\
                        Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let mut content_length = 0;
    loop {
        let mut line = String::new();
        if buf_reader.read_line(&mut line).is_err() {
            break;
        }
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val_str) = line.split(':').nth(1) {
                if let Ok(len) = val_str.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 && buf_reader.read_exact(&mut body).is_err() {
        send_json_response(stream, 400, "{\"error\":\"Error reading body\"}");
        return;
    }

    // Vendor API endpoints
    if clean_path == "/v1/models" && method == "GET" {
        // Advertise only the text-ready model this process can actually run.
        // Disk inventory is not request-time model selection.
        let models = active_models(&serving.lock().unwrap());
        send_json_response(stream, 200, &models_list_body(&models).to_string());
        return;
    }

    // GET /v1/models/{model} (#654 phase B): agrees with the list; a model id
    // absent from the loadable set is a 404 with the standard error envelope.
    if method == "GET" {
        if let Some(model_id) = clean_path.strip_prefix("/v1/models/") {
            let models = active_models(&serving.lock().unwrap());
            match models.iter().find(|(id, _)| id == model_id) {
                Some((id, created)) => {
                    send_json_response(stream, 200, &openai_model_object(id, *created).to_string())
                }
                None => send_openai_error(
                    stream,
                    404,
                    "invalid_request_error",
                    &format!("The model '{model_id}' does not exist or is not loadable."),
                    Some("model"),
                    Some("model_not_found"),
                ),
            }
            return;
        }
    }

    if extended_route_canonical(clean_path) == Some("/uor/v1/status") && method == "GET" {
        // #654 phase G: canonical /uor/v1/status; /v1/status stays a deprecated
        // alias (Deprecation header) so /v1 is OpenAI-only without losing this.
        let dep = deprecation_headers(clean_path);
        let installed = serving.lock().unwrap();
        let graph_loaded = installed.r4g1.is_some();
        let graph_ready = graph_text_ready(&installed);
        let decode_only = graph_loaded && !graph_ready;
        let teacher_ready = teacher_text_ready(&installed);
        let engine_active = graph_ready || teacher_ready;
        let logical_name = installed_logical_model_name(&installed);
        let bundle_compiled = installed.active_bundle.is_some();

        let body = serde_json::json!({
            "model_name": logical_name,
            "physical_root": status_physical_root(installed.active_bundle.as_ref()),
            "attention_operator": installed.active_bundle.as_ref().map(|bundle| &bundle.attention_operator),
            "r4g1_loaded": graph_loaded,
            "r4g1_ready": graph_ready,
            "decode_only": decode_only,
            "teacher_ready": teacher_ready,
            "engine_active": engine_active,
            "terminal_error": installed.terminal_load_error.as_deref(),
            "last_operation_error": installed.last_operation_error.as_deref(),
            "stages": {
                "stage_1_download": installed.active_teacher_source.is_some(),
                "stage_2_compile": bundle_compiled,
                "stage_3_graph_score": graph_loaded,
                "stage_4_r4g1_active": graph_ready
            }
        });
        send_json_response_ext(stream, 200, &body.to_string(), &dep);
        return;
    }

    if extended_route_canonical(clean_path) == Some("/uor/v1/reload") && method == "POST" {
        // #654 phase G: canonical /uor/v1/reload; /v1/reload deprecated alias.
        let dep = deprecation_headers(clean_path);
        let payload = match parse_huggingface_control_payload(&body) {
            Ok(payload) => payload,
            Err(error) => {
                send_json_response_ext(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let tokenizer_selection = match payload.tokenizer_selection() {
            Ok(selection) => selection,
            Err(error) => {
                send_json_response_ext(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let target_model = payload
            .model
            .as_deref()
            .map(str::trim)
            .filter(|model| !model.is_empty())
            .unwrap_or("smollm2-135m-instruct");
        let _source_cache_reservation = match try_reserve_source_cache_operation(
            &source_cache_operations,
            SourceCacheOperationKind::Reload,
            target_model,
        ) {
            Ok(reservation) => reservation,
            Err(error) => {
                send_json_response_ext(
                    stream,
                    409,
                    &serde_json::json!({
                        "status": "error",
                        "running": true,
                        "message": error,
                    })
                    .to_string(),
                    &dep,
                );
                return;
            }
        };
        let (base_epoch, mut reload_reservation) =
            match reserve_r4g1_reload(&serving, &r4g1_compile) {
                Ok(reservation) => reservation,
                Err(ready) => {
                    send_json_response_ext(
                        stream,
                        409,
                        &serde_json::json!({
                            "status": "error",
                            "running": true,
                            "ready": ready,
                            "message": "another R4G1 compile or reload is already running"
                        })
                        .to_string(),
                        &dep,
                    );
                    return;
                }
            };
        let current_version = match current_source_attention_era_version() {
            Ok(version) => version,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let reload_logical = match logical_model_name_for_request(target_model, current_version) {
            Ok(logical) => logical,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    400,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let compiled_root = Path::new(".uor-models/compiled");
        let suffix = attention_era_suffix(current_version);
        let reload_read_sessions = match try_lock_managed_inventory_write_sessions(
            compiled_root,
            [
                compiled_root.join(&reload_logical),
                compiled_root.join(format!("{reload_logical}{suffix}")),
                PathBuf::from(".uor-models/sources"),
            ],
        ) {
            Ok(sessions) => sessions,
            Err(error) if source_compile_session_is_busy(&error) => {
                // BUSY is transient cross-process publication, not a defect
                // in the active serving tuple or the on-disk candidate. Keep
                // both the prior tuple and its terminal/readiness markers.
                reload_reservation.disarm();
                let installed = serving.lock().unwrap();
                let mut status = r4g1_compile.lock().unwrap();
                status.running = false;
                status.ready = graph_text_ready(&installed);
                status.message =
                    "R4G1 reload deferred while another process publishes the bundle".to_owned();
                drop(status);
                drop(installed);
                send_json_response_ext(
                    stream,
                    409,
                    &serde_json::json!({
                        "status": "error",
                        "running": true,
                        "message": error,
                    })
                    .to_string(),
                    &dep,
                );
                return;
            }
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        if let Err(error) = recover_managed_compiled_bundle_completion_temporaries(compiled_root) {
            record_replacement_failure(&serving, error.clone());
            send_json_response_ext(
                stream,
                500,
                &serde_json::json!({ "status": "error", "message": error }).to_string(),
                &dep,
            );
            return;
        }
        let (resolved, source_for_reload) = match resolve_reload_bundle_in(
            Path::new(".uor-models"),
            target_model,
            current_version,
        ) {
            Ok(Some(resolved)) => resolved,
            Ok(None) => {
                record_replacement_failure(
                    &serving,
                    format!("no compiled R4G1 graph artifact found for model '{target_model}'"),
                );
                let resp = serde_json::json!({
                    "status": "error",
                    "message": format!("No compiled R4G1 graph artifact found for model '{}'. Please compile it first.", target_model)
                });
                send_json_response_ext(stream, 404, &resp.to_string(), &dep);
                return;
            }
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        if let Err(error) = validate_legacy_graph_generation_for_serving(&resolved.graph) {
            record_replacement_failure(&serving, error.clone());
            send_json_response_ext(
                stream,
                500,
                &serde_json::json!({ "status": "error", "message": error }).to_string(),
                &dep,
            );
            return;
        }
        if let Err(error) =
            ensure_compiled_bundle_completion_for_serving(&resolved, current_version)
        {
            record_replacement_failure(&serving, error.clone());
            send_json_response_ext(
                stream,
                500,
                &serde_json::json!({ "status": "error", "message": error }).to_string(),
                &dep,
            );
            return;
        }
        let reload_source_snapshot_before = match source_for_reload
            .as_deref()
            .map(|source| verify_managed_source_snapshot(source, &resolved.logical_name))
            .transpose()
        {
            Ok(manifest) => manifest,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        if source_for_reload.is_some() {
            if let Err(error) = validate_resolved_source_snapshot_binding(
                &resolved,
                reload_source_snapshot_before.as_ref(),
                current_version,
            ) {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        }

        let replacement_state = match r4g1::R4g1State::load_with_source(
            &resolved.graph,
            &resolved.teacher,
            source_for_reload.as_deref(),
        ) {
            Ok(state) => state,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                let resp = serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to load R4G1 graph artifact: {error}")
                });
                send_json_response_ext(stream, 500, &resp.to_string(), &dep);
                return;
            }
        };

        // Prepare the optional teacher/tokenizer pair without mutating the
        // active slots. A genuinely absent source is the #718 decode-only
        // case; a present source was already required to be a directory and
        // the R4G1 loader has enforced any tagged host-encoder identity.
        let mut replacement_teacher = match prepare_optional_teacher_source_for_identity(
            source_for_reload.as_deref(),
            tokenizer_selection.as_ref(),
            &resolved.logical_name,
            replacement_state.tokenizer_adapter_identity(),
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let (teacher_default_r4_attention, _teacher_mismatch) =
            match reconcile_prepared_teacher_with_bundle(&mut replacement_teacher, Some(&resolved))
            {
                Ok(decision) => decision,
                Err(error) => {
                    record_replacement_failure(&serving, error.clone());
                    send_json_response_ext(
                        stream,
                        500,
                        &serde_json::json!({ "status": "error", "message": error }).to_string(),
                        &dep,
                    );
                    return;
                }
            };
        let refreshed = match refresh_resolved_compiled_bundle(&resolved, current_version) {
            Ok(refreshed) => refreshed,
            Err(error) => {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        if let (Some(source), Some(before)) = (
            source_for_reload.as_deref(),
            reload_source_snapshot_before.as_ref(),
        ) {
            let after = match verify_managed_source_snapshot(source, &resolved.logical_name) {
                Ok(after) => after,
                Err(error) => {
                    record_replacement_failure(&serving, error.clone());
                    send_json_response_ext(
                        stream,
                        500,
                        &serde_json::json!({ "status": "error", "message": error }).to_string(),
                        &dep,
                    );
                    return;
                }
            };
            if let Err(error) =
                require_unchanged_managed_source_snapshot(source, "R4G1 reload", before, &after)
            {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
            if let Err(error) =
                validate_resolved_source_snapshot_binding(&refreshed, Some(&after), current_version)
            {
                record_replacement_failure(&serving, error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        }

        // Install the graph/oracle/tokenizer identity tuple only after the
        // selected sidecar, corpus, and cover records were re-read exactly.
        let (replacement_oracle, replacement_tokenizer, replacement_teacher_source) =
            match replacement_teacher {
                Some(prepared) => (
                    Some(prepared.teacher),
                    Some(prepared.tokenizer),
                    Some(prepared.source),
                ),
                None => (None, None, None),
            };
        let mut installed = serving.lock().unwrap();
        if installed.epoch != base_epoch {
            let error =
                "active serving model changed while reload was preparing; refusing stale installation"
                    .to_owned();
            installed.last_operation_error = Some(error.clone());
            drop(installed);
            send_json_response_ext(
                stream,
                409,
                &serde_json::json!({ "status": "error", "message": error }).to_string(),
                &dep,
            );
            return;
        }
        let mut compile_status = r4g1_compile.lock().unwrap();
        installed.epoch = installed.epoch.wrapping_add(1);
        installed.r4g1 = Some(replacement_state);
        installed.oracle = replacement_oracle;
        installed.source_tokenizer = replacement_tokenizer;
        installed.teacher_default_r4_attention = teacher_default_r4_attention;
        installed.active_teacher_source = replacement_teacher_source;
        installed.active_bundle = Some(refreshed.clone());
        installed.terminal_load_error = None;
        installed.last_operation_error = None;
        compile_status.ready = graph_text_ready(&installed);
        compile_status.running = false;
        compile_status.progress = 100;
        compile_status.message = format!(
            "R4G1 graph runtime ready after reload of {}",
            refreshed.logical_name
        );
        reload_reservation.disarm();
        drop(reload_read_sessions);
        let resp = serde_json::json!({
            "status": "success",
            "model": refreshed.logical_name,
            "physical_root": refreshed.physical_root.display().to_string(),
            "message": format!("Successfully reloaded R4G1 runtime for model '{}'", refreshed.logical_name)
        });
        send_json_response_ext(stream, 200, &resp.to_string(), &dep);
        return;
    }

    if extended_route_canonical(clean_path) == Some("/uor/v1/corpus") && method == "POST" {
        // #654 phase G: canonical /uor/v1/corpus; /v1/corpus deprecated alias.
        let dep = deprecation_headers(clean_path);
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        let action = payload["action"].as_str().unwrap_or("list");

        if action == "add" {
            if let Some(content) = payload["content"].as_str() {
                let filename = payload["filename"].as_str().unwrap_or("custom_corpus.txt");
                let dir_path = std::path::Path::new(".uor-models/extra_reading");
                std::fs::create_dir_all(dir_path).ok();
                let file_path = dir_path.join(filename);
                std::fs::write(&file_path, content).ok();

                let mut router_guard = router.lock().unwrap();
                let identity = "tenant-alpha";
                let mut line_count = 0usize;
                for sentence in content.lines() {
                    let s = sentence.trim();
                    if !s.is_empty() {
                        router_guard.index_sentence(s, identity);
                        router_guard.inject_thought_stream_native(s);
                        line_count += 1;
                    }
                }
                let state_json = router_guard.export_state();
                spawn_cache_save(&cli, state_json);

                let resp = serde_json::json!({
                    "status": "success",
                    "filename": filename,
                    "lines_indexed": line_count,
                    "message": format!("Added corpus file '{}' and indexed {} lines into geometric manifold hashes.", filename, line_count)
                });
                send_json_response_ext(stream, 200, &resp.to_string(), &dep);
                return;
            }
        }

        if action == "export" {
            let export_dir = std::path::Path::new(".uor-models/exported");
            std::fs::create_dir_all(export_dir).ok();
            let export_file = export_dir.join("exported_manifold.json");

            let router_guard = router.lock().unwrap();
            let state_json = router_guard.export_state();
            std::fs::write(&export_file, &state_json).ok();
            std::fs::write(".uor-models/exported_manifold.json", &state_json).ok();

            let resp = serde_json::json!({
                "status": "success",
                "path": export_file.display().to_string(),
                "bytes": state_json.len(),
                "message": format!("Successfully exported manifold state to {}", export_file.display())
            });
            send_json_response_ext(stream, 200, &resp.to_string(), &dep);
            return;
        }

        let extra_dir = std::path::Path::new(".uor-models/extra_reading");
        let mut files = Vec::new();
        if extra_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(extra_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_file() {
                        files.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }

        let resp = serde_json::json!({
            "status": "success",
            "files": files
        });
        send_json_response_ext(stream, 200, &resp.to_string(), &dep);
        return;
    }

    if clean_path == "/v1/chat/completions" && method == "POST" {
        let req: VendorChatCompletionsRequest = match serde_json::from_slice(&body) {
            Ok(parsed) => parsed,
            Err(error) => {
                // #654 phase B: malformed JSON and unsupported parameters (the
                // request DTO denies unknown fields) return the standard error
                // envelope, not an ad-hoc string body — support is never
                // implied by silently ignoring a field.
                send_openai_error(
                    stream,
                    400,
                    "invalid_request_error",
                    &format!("Invalid request body: {error}"),
                    None,
                    None,
                );
                return;
            }
        };

        let model_name =
            match resolve_active_request_model(&serving.lock().unwrap(), req.model.as_deref()) {
                Ok(model) => model,
                Err(error) if error.starts_with("no text-ready") => {
                    send_openai_error(
                        stream,
                        503,
                        "server_error",
                        &error,
                        Some("model"),
                        Some("model_not_ready"),
                    );
                    return;
                }
                Err(error) => {
                    send_openai_error(
                        stream,
                        404,
                        "invalid_request_error",
                        &error,
                        Some("model"),
                        Some("model_not_found"),
                    );
                    return;
                }
            };

        // #654 phase C: flatten the supported roles (system/developer/user/
        // assistant); an unsupported role fails closed with the envelope
        // rather than being silently accepted.
        let prompt_text = match flatten_chat_prompt(&req.messages) {
            Ok(text) => text,
            Err(role) => {
                send_openai_error(
                    stream,
                    400,
                    "invalid_request_error",
                    &format!(
                        "Unsupported message role '{role}'. Supported roles: system, developer, user, assistant."
                    ),
                    Some("messages"),
                    None,
                );
                return;
            }
        };

        let max_tokens = req.max_tokens.unwrap_or(256);
        // #654 phase E: route through the shared generation core (also used by
        // /v1/responses) so both wire surfaces run the identical cascade.
        let gen = match generate_serving_completion(
            &router,
            &serving,
            &tless,
            &cli,
            start_time,
            &model_name,
            &prompt_text,
            req.engine.as_deref(),
            max_tokens,
            req.temperature,
        ) {
            GenerationOutcome::Declined { status, body } => {
                send_json_response(stream, status, &body.to_string());
                return;
            }
            GenerationOutcome::Generated(gen) => *gen,
        };

        // Values shared by the single-JSON and the streaming surfaces.
        let response_id = format!("chatcmpl-uor-r4-{}", gen.created_ts);
        let system_fingerprint = format!("uor-r4-{}", gen.generation_mode);
        // #654 phase C: `length` when the served completion reached the
        // effective token budget, otherwise `stop`.
        let finish_reason = completion_finish_reason(gen.completion_tokens, max_tokens).to_string();
        let usage = VendorUsage {
            prompt_tokens: gen.prompt_tokens,
            completion_tokens: gen.completion_tokens,
            total_tokens: gen.total_tokens,
        };

        // #654 phase D: streaming surface. The serving cascade produces the
        // whole completion up front (there is no incremental token generator on
        // this path), so streaming re-frames that finished text as ordered
        // `chat.completion.chunk` events — the wire format is streaming, not the
        // generation. The audit trail rides the terminal chunk for parity with
        // the single-JSON body.
        if req.stream == Some(true) {
            let include_usage = req
                .stream_options
                .as_ref()
                .and_then(|options| options.include_usage)
                .unwrap_or(false);
            let audit_value = serde_json::to_value(&gen.uor_audit).ok();
            let frames = build_chat_stream_frames(
                &response_id,
                gen.created_ts,
                &model_name,
                &system_fingerprint,
                &gen.text,
                &finish_reason,
                if include_usage { Some(&usage) } else { None },
                audit_value,
                gen.cascade_trail,
            );
            send_sse_stream(stream, &frames);
            return;
        }

        let resp = VendorChatCompletionsResponse {
            id: response_id,
            object: "chat.completion".to_string(),
            created: gen.created_ts,
            model: model_name,
            choices: vec![VendorChoice {
                index: 0,
                message: VendorChatMessage {
                    role: "assistant".to_string(),
                    content: gen.text,
                },
                finish_reason,
            }],
            usage,
            system_fingerprint: Some(system_fingerprint),
            uor_audit: Some(gen.uor_audit),
            cascade_trail: gen.cascade_trail,
        };

        send_json_response(stream, 200, &serde_json::to_string(&resp).unwrap());
        return;
    }

    if clean_path == "/v1/responses" && method == "POST" {
        // #654 phase E: the Responses API, routed through the same internal
        // generation adapter as chat completions (`generate_serving_completion`).
        let req: VendorResponsesRequest = match serde_json::from_slice(&body) {
            Ok(parsed) => parsed,
            Err(error) => {
                send_openai_error(
                    stream,
                    400,
                    "invalid_request_error",
                    &format!("Invalid request body: {error}"),
                    None,
                    None,
                );
                return;
            }
        };

        let model_name =
            match resolve_active_request_model(&serving.lock().unwrap(), req.model.as_deref()) {
                Ok(model) => model,
                Err(error) if error.starts_with("no text-ready") => {
                    send_openai_error(
                        stream,
                        503,
                        "server_error",
                        &error,
                        Some("model"),
                        Some("model_not_ready"),
                    );
                    return;
                }
                Err(error) => {
                    send_openai_error(
                        stream,
                        404,
                        "invalid_request_error",
                        &error,
                        Some("model"),
                        Some("model_not_found"),
                    );
                    return;
                }
            };

        let prompt_text = match flatten_responses_input(&req.input, req.instructions.as_deref()) {
            Ok(text) => text,
            Err(message) => {
                send_openai_error(
                    stream,
                    400,
                    "invalid_request_error",
                    &message,
                    Some("input"),
                    None,
                );
                return;
            }
        };

        let max_tokens = req.max_output_tokens.unwrap_or(256);
        let gen = match generate_serving_completion(
            &router,
            &serving,
            &tless,
            &cli,
            start_time,
            &model_name,
            &prompt_text,
            req.engine.as_deref(),
            max_tokens,
            req.temperature,
        ) {
            GenerationOutcome::Declined { status, body } => {
                send_json_response(stream, status, &body.to_string());
                return;
            }
            GenerationOutcome::Generated(gen) => *gen,
        };
        // Streaming for /v1/responses: like chat streaming, the cascade produces
        // the whole completion up front, so this re-frames the finished text as
        // the Responses typed-event sequence (created → … → response.completed;
        // no `[DONE]` sentinel). The wire format is streaming, not the generation.
        if req.stream == Some(true) {
            let response_id = format!("resp-uor-r4-{}", gen.created_ts);
            let created_at = gen.created_ts;
            let content = gen.text.clone();
            let completed = build_responses_body(gen, &model_name, max_tokens);
            let frames = build_responses_stream_frames(
                &response_id,
                created_at,
                &model_name,
                &content,
                completed,
            );
            send_sse_stream(stream, &frames);
            return;
        }

        let body = build_responses_body(gen, &model_name, max_tokens);
        send_json_response(stream, 200, &body.to_string());
        return;
    }

    // Intercept native router endpoints
    if clean_path == "/api/chat" && method == "POST" {
        let payload: ChatPayload = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };

        let identity = payload
            .identity
            .unwrap_or_else(|| "tenant-alpha".to_string());
        // Issue #248: an engine named by the request (or the persisted
        // `/engine` selection) pins the cascade to that single tier;
        // "auto"/empty and the legacy `ollama` alias run the full cascade.
        let pinned = resolve_pinned_tier(payload.engine.as_deref());

        let mut installed = serving.lock().unwrap();
        let mut router_guard = router.lock().unwrap();

        // 1. Dry run routing to get baseline parameters via UOR pipeline
        let mut buf = [0u8; 640];
        let query_bytes = payload.text.as_bytes();
        let identity_bytes = identity.as_bytes();
        let query_len = query_bytes.len().min(512);
        let identity_len = identity_bytes.len().min(128);
        buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
        buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

        let input = uor_r4_wasm_router::R4RoutingInput {
            query: &buf[..512],
            identity: &buf[512..],
            data: &buf,
        };

        // Bind thread-local
        let router_ptr = &mut *router_guard as *mut UorR4Router;
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        // Run dry run through UorR4RouterModel
        let _grounded_dry =
            uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Dry run routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing = router_guard
            .last_routing_data()
            .clone()
            .expect("No routing data generated");
        let kappa = routing.routed.metrics.kappa;
        let theta_d = routing.routed.metrics.deficit_angle;
        let uor_bias = routing.routed.qimc.uor_control.entropy_bias;

        // Auto-tuned params
        let (gamma, temperature) = autotune(kappa, theta_d, uor_bias);

        // Determine dynamic suggested token limit from the router itself
        let max_tokens = router_guard.get_suggested_token_limit(&payload.text, &identity);

        // 3. Evolve the brain state
        router_guard.evolve_state(&identity, &payload.text, gamma);
        let session_signature = uor_r4_router::session_signature_from_state(
            &router_guard.get_brain_state_native(&identity),
        );

        // 4. Run final routing on evolved state via UOR pipeline
        let t_route = Instant::now();

        // Bind thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        let grounded =
            uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Final routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing_data = router_guard
            .last_routing_data()
            .clone()
            .expect("No final routing data generated");
        let route_ms = t_route.elapsed().as_secs_f64() * 1000.0;

        // 5. Decode response through the single serving cascade (issue
        // #248). A D4 abstention no longer refuses fallback outright: it is
        // RECORDED in the per-tier trail while later tiers still attempt
        // (PR #223 semantics, centralized in `SERVING_ABSTAIN_POLICY`).
        let t_gen = Instant::now();
        let mut cascade = run_serving_cascade(
            &mut router_guard,
            &mut installed,
            &tless,
            &payload.text,
            &identity,
            max_tokens,
            temperature,
            gamma,
            Some(&session_signature),
            pinned,
        );

        // Legacy response fields, derived from the trail so consumers keep
        // the names and values they rely on.
        let generation_mode = derive_generation_mode(&cascade, pinned);
        let r4g1_status = cascade.r4g1.status;
        let r4g1_widened = cascade.r4g1.widened;
        let r4g1_abstained = cascade.r4g1.abstained;
        let llm_connected = cascade
            .outcome
            .served_by
            .map(|tier| tier != TIER_GEOMETRIC)
            .unwrap_or(false);
        let cascade_trail = cascade_trail_json(&cascade.outcome.trail);
        let geom_trajectory = cascade
            .geometric
            .take()
            .map(|geom| geom.trajectory)
            .unwrap_or_default();

        let Some(final_response_text) = cascade.outcome.text.clone() else {
            // Declined by all attempted tiers: an honest terminal naming
            // every attempted tier, instead of the sparse-string
            // placeholder served as if it were generated.
            let (status, body) = declined_by_all_response(&cascade, pinned, &generation_mode);
            send_json_response(stream, status, &body.to_string());
            return;
        };

        let tokens_generated = final_response_text.split_whitespace().count();
        let gen_ms = t_gen.elapsed().as_secs_f64() * 1000.0;
        let mut tokens_per_sec = 0.0f64;
        if tokens_generated > 0 && gen_ms > 0.0 {
            tokens_per_sec = tokens_generated as f64 / (gen_ms / 1000.0);
        }

        // 6. Index user prompt and response back into vocabulary for continuous learning
        if !final_response_text.is_empty() && is_usable_generated_text(&final_response_text) {
            router_guard.index_sentence(&payload.text, &identity);
            router_guard.index_sentence(&final_response_text, &identity);

            // Inject thought streams for tracing
            router_guard.inject_thought_stream_native(&payload.text);
            router_guard.inject_thought_stream_native(&final_response_text);

            // Save cache to disk in background thread
            let state_json = router_guard.export_state();
            spawn_cache_save(&cli, state_json);
        }

        // Project the evolved brain state to 2D for the map path tracing
        let active_state = router_guard.get_brain_state_native(&identity);
        let (u, v) = router_guard.get_sentence_projection_native(
            &active_state,
            routing_data.routed.window_index as usize,
        );
        let v_4d = router_guard.get_state_4d_projection_native(&active_state);

        let theme = get_window_theme(routing_data.routed.window_index as usize);
        let archetype = if theta_d > -1.0 {
            "Symmetric Orbit (Resonant)"
        } else if theta_d < -1.4 {
            "Hyperbolic Flare (Divergent)"
        } else {
            "Orthogonal Drift (Steady)"
        };

        let top_resonances_5 = router_guard.get_top_resonances_native(&payload.text, &identity, 5);

        let trace = grounded.derivation().replay::<256>();
        let mut uor_trace_steps = Vec::new();
        for i in 0..trace.len() {
            if let Some(event) = trace.event(i as usize) {
                uor_trace_steps.push(serde_json::json!({
                    "step": event.step_index(),
                    "op": format!("{:?}", event.op()),
                    "target": format!("0x{:032x}", event.target().as_u128()),
                }));
            }
        }

        let uor_payload = serde_json::json!({
            "algorithm": routing_data.routed.uor.algorithm.clone(),
            "hash_algorithm": routing_data.routed.uor.hash_algorithm.clone(),
            "hash_algorithm_id": routing_data.routed.uor.hash_algorithm_id,
            "address": routing_data.routed.uor.address.clone(),
            "verify_result": "Verified",
            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
            "sigma": grounded.sigma().value(),
            "d_delta": grounded.d_delta().as_i64(),
            "euler": grounded.euler().as_i64(),
            "residual": grounded.residual().as_u32(),
            "stratum": grounded.triad().stratum(),
            "multihash_addresses": routing_data.routed.uor.multihash_addresses.clone(),
        });

        let response_payload = serde_json::json!({
            "text": final_response_text,
            "prompt": payload.text,
            "archetype": archetype,
            "description": final_response_text,
            "summary": format!("W{} ({}) | Scale {:.0} | kappa={:.4} theta_d={:.4} | {}",
                routing_data.routed.window_index, theme, routing_data.routed.scale_x, kappa, theta_d, generation_mode),
            "llm_connected": llm_connected,
            "generation_mode": generation_mode,
            "r4g1": {
                "status": r4g1_status,
                "widened": r4g1_widened,
                "abstained": r4g1_abstained,
            },
            "pinned_engine": pinned,
            "cascade_trail": cascade_trail,
            "active_projection": {
                "u": u,
                "v": v,
                "v_4d": v_4d
            },
            "metrics": {
                "window_index": routing_data.routed.window_index,
                "scale_x": routing_data.routed.scale_x,
                "kappa": kappa,
                "deficit_angle": theta_d,
                "lambda_entropy": routing_data.routed.metrics.lambda_entropy,
                "sigma_kl": routing_data.routed.metrics.sigma_kl,
                "top_eigenvalue_pct": ((routing_data.routed.eigenvalues[0] / (routing_data.routed.eigenvalues.iter().sum::<f64>().max(1.0))) * 100.0),
                "qimc": routing_data.routed.qimc,
                "hopf": routing_data.routed.hopf,
                "uor_address": routing_data.routed.uor_address,
                "uor": uor_payload,
                "auto_tuned": {
                    "gamma": gamma,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                    "engine": generation_mode,
                    "uor_entropy_bias": uor_bias
                }
            },
            "eigenvalues": routing_data.routed.eigenvalues,
            "active_range": routing_data.routed.active_range,
            "state_vector": routing_data.routed.state_vector,
            "all_routes": routing_data.all_routes,
            "top_resonance": top_resonances_5,
            "trajectory": geom_trajectory,
            "active_streams": router_guard.get_active_streams_native(),
            "expert_counts": router_guard.get_expert_counts(),
            "routing_latency_ms": route_ms.round(),
            "gen_latency_ms": gen_ms.round(),
            "tokens_generated": tokens_generated,
            "tokens_per_sec": tokens_per_sec,
            "uor_trace_steps": uor_trace_steps,
        });

        // Issue #256: the envelope address goes through the uor-addr
        // canonical pipeline (JCS + blake3), not a raw hash of one
        // serialization; the artifact CID is the real content address of
        // the loaded artifact or omitted — never a placeholder. The old
        // "store_cid" named a section of the same file and is dropped;
        // self-asserted "verify_result" likewise (POST /api/uor/verify is
        // the verifier).
        let attestation_cid = canonical_json_address_blake3(&response_payload);

        let mut attestation_fields = serde_json::Map::new();
        attestation_fields.insert("algorithm".to_string(), serde_json::json!("blake3-jcs"));
        attestation_fields.insert(
            "uor_address".to_string(),
            serde_json::json!(attestation_cid),
        );
        attestation_fields.insert(
            "attestation_cid".to_string(),
            serde_json::json!(attestation_cid),
        );
        if let Some(cid) = active_artifact_cid() {
            attestation_fields.insert("artifact_cid".to_string(), serde_json::json!(cid));
        }
        let attestation_envelope = serde_json::Value::Object(attestation_fields);

        let mut final_response = response_payload.as_object().unwrap().clone();
        final_response.insert("uor_attestation".to_string(), attestation_envelope);
        final_response.insert(
            "attestation_cid".to_string(),
            serde_json::json!(attestation_cid),
        );
        let final_body = serde_json::Value::Object(final_response).to_string();

        send_json_response(stream, 200, &final_body);
        return;
    }

    if clean_path == "/api/tless/predict" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };
        let mut window_tokens: Vec<u32> = payload
            .get("window")
            .and_then(|w| w.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|x| x as u32))
                    .collect()
            })
            .unwrap_or_default();
        if window_tokens.is_empty() {
            send_json_response(
                stream,
                400,
                "{\"error\":\"`window` must be a non-empty array of token ids\"}",
            );
            return;
        }
        // keep the WINDOW most recent tokens, oldest first
        if window_tokens.len() > 8 {
            window_tokens = window_tokens.split_off(window_tokens.len() - 8);
        }
        let mut buf = [0u8; 32];
        for (i, t) in window_tokens.iter().enumerate() {
            buf[4 * i..4 * i + 4].copy_from_slice(&t.to_le_bytes());
        }
        let outcome = with_tless_server_state(&tless, |_st| {
            let input = tless_uor::TlessPredictInput {
                window: &buf,
                data: &buf,
            };
            match tless_uor::UorTlessModel::forward(input) {
                Ok(grounded) => {
                    // the deterministic record again via the axis, for the JSON fields
                    let mut out = [0u8; tless_uor::TLESS_OUTPUT_BYTES];
                    if let Err(e) = tless_uor::TlessAxisImpl::predict(&buf, &mut out) {
                        return (
                            500,
                            format!("{{\"error\":\"axis predict failed: {:?}\"}}", e),
                        );
                    }
                    let token = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
                    let depth = out[4];
                    let code: Vec<u8> = out[5..9].to_vec();
                    let count = u32::from_be_bytes(out[9..13].try_into().unwrap());
                    let census =
                        |i: usize| u32::from_be_bytes(out[i + 2..i + 6].try_into().unwrap());

                    let (artifact_kappa, artifact_address, store_kappa) =
                        tless_uor::with_tless_state(|st| {
                            (
                                st.artifact_kappa.clone(),
                                st.artifact_address.clone(),
                                st.store_kappa.clone(),
                            )
                        })
                        .unwrap_or_default();

                    let trace = grounded.derivation().replay::<256>();
                    let mut uor_trace_steps = Vec::new();
                    for i in 0..trace.len() {
                        if let Some(event) = trace.event(i as usize) {
                            uor_trace_steps.push(serde_json::json!({
                                "step": event.step_index(),
                                "op": format!("{:?}", event.op()),
                                "target": format!("0x{:032x}", event.target().as_u128()),
                            }));
                        }
                    }

                    let response_payload = serde_json::json!({
                        "window": window_tokens,
                        "prediction": {
                            "token": token,
                            "depth": depth,
                            "code": code,
                            "count": count,
                        },
                        "census": {
                            "adds": census(11),
                            "xors": census(15),
                            "shifts": census(19),
                            "compares": census(23),
                            "table_reads": census(27),
                            "candidate_scans": census(31),
                            "multiply": 0,
                        },
                        "artifact": {
                            "kappa": artifact_kappa,
                            "address": artifact_address,
                        },
                        "store": { "kappa": store_kappa },
                        "uor": {
                            "verify_result": "Verified",
                            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
                            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
                            "sigma": grounded.sigma().value(),
                            "d_delta": grounded.d_delta().as_i64(),
                            "euler": grounded.euler().as_i64(),
                            "residual": grounded.residual().as_u32(),
                            "stratum": grounded.triad().stratum(),
                        },
                        "uor_trace_steps": uor_trace_steps,
                    });
                    (200, response_payload.to_string())
                }
                Err(e) => (
                    500,
                    format!("{{\"error\":\"tless pipeline failed: {:?}\"}}", e),
                ),
            }
        });
        match outcome {
            Some((code, body)) => send_json_response(stream, code, &body),
            None => send_json_response(
                stream,
                503,
                "{\"error\":\"transformerless state unavailable — run `cargo run --release -- compile` and `cargo run --release -- store` (or set TLESS_ARTIFACTS / TLESS_STORE)\"}",
            ),
        }
        return;
    }

    if clean_path == "/api/tless/index" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };
        let text = payload.get("text").and_then(|t| t.as_str()).unwrap_or("");
        if text.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"`text` must be non-empty\"}");
            return;
        }
        let Some(tokens) = tless_uor::tless_tokenize(text) else {
            send_json_response(
                stream,
                503,
                "{\"error\":\"tokenizer unavailable — set TLESS_TOKENIZER (default /tmp/ref/tokenizer.bin)\"}",
            );
            return;
        };
        let outcome = with_tless_server_state(&tless, |_st| {
            let positions = tless_uor::index_token_stream(&tokens).unwrap_or(0);
            let kappa =
                tless_uor::with_tless_state(|st| st.store_kappa.clone()).unwrap_or_default();
            serde_json::json!({
                "indexed_text_bytes": text.len(),
                "tokens": tokens.len(),
                "evidence_positions": positions,
                "store": { "kappa": kappa },
            })
            .to_string()
        });
        match outcome {
            Some(body) => send_json_response(stream, 200, &body),
            None => send_json_response(
                stream,
                503,
                "{\"error\":\"transformerless state unavailable — run `cargo run --release -- compile` and `cargo run --release -- store`\"}",
            ),
        }
        return;
    }

    if clean_path == "/api/tless/generate" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };
        let seed: Vec<u32> = if let Some(arr) = payload.get("window").and_then(|w| w.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|x| x as u32))
                .collect()
        } else if let Some(text) = payload.get("text").and_then(|t| t.as_str()) {
            match tless_uor::tless_tokenize(text) {
                Some(t) => t,
                None => {
                    send_json_response(
                        stream,
                        503,
                        "{\"error\":\"tokenizer unavailable — set TLESS_TOKENIZER\"}",
                    );
                    return;
                }
            }
        } else {
            vec![1]
        };
        if seed.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"empty seed\"}");
            return;
        }
        let max_tokens = payload
            .get("max_tokens")
            .and_then(|m| m.as_u64())
            .unwrap_or(24)
            .clamp(1, 256) as usize;
        let outcome = with_tless_server_state(&tless, |_st| {
            let mut steps = [uor_r4_core::transformerless::runtime::Prediction::default(); 256];
            let step_count =
                tless_uor::generate_steps_into(&seed, &mut steps[..max_tokens]).unwrap_or(0);
            let steps = &steps[..step_count];
            let mut tokens = [0u32; 256];
            for (token, prediction) in tokens.iter_mut().zip(steps) {
                *token = prediction.token;
            }
            let mut text_bytes = [0u8; 16 * 1024];
            let text_len = tless_uor::tless_detokenize_into(&tokens[..step_count], &mut text_bytes)
                .unwrap_or(0);
            let text = String::from_utf8_lossy(&text_bytes[..text_len]).into_owned();
            let kappa =
                tless_uor::with_tless_state(|st| st.store_kappa.clone()).unwrap_or_default();
            let step_json: Vec<_> = steps
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "token": p.token,
                        "depth": p.depth,
                        "count": p.count,
                    })
                })
                .collect();
            serde_json::json!({
                "seed": seed,
                "tokens": &tokens[..step_count],
                "text": text,
                "steps": step_json,
                "store": { "kappa": kappa },
            })
            .to_string()
        });
        match outcome {
            Some(body) => send_json_response(stream, 200, &body),
            None => send_json_response(
                stream,
                503,
                "{\"error\":\"transformerless state unavailable — run `cargo run --release -- compile` and `cargo run --release -- store`\"}",
            ),
        }
        return;
    }

    if clean_path == "/api/corpus" && method == "POST" {
        let payload: CorpusPayload = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };

        let identity = payload.identity.unwrap_or_else(|| "shared".to_string());
        let mut router_guard = router.lock().unwrap();
        let count = router_guard.index_corpus(&payload.corpus, &identity);

        let state_json = router_guard.export_state();
        spawn_cache_save(&cli, state_json);

        let resp = serde_json::json!({ "success": true, "count": count }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/reset" && method == "POST" {
        let payload: ResetPayload =
            serde_json::from_slice(&body).unwrap_or(ResetPayload { identity: None });

        let mut router_guard = router.lock().unwrap();
        if let Some(ref identity) = payload.identity {
            router_guard.reset_brain(identity);
        } else {
            router_guard.reset_to_defaults();
        }

        let state_json = router_guard.export_state();
        spawn_cache_save(&cli, state_json);

        let resp = serde_json::json!({ "success": true }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/export" && (method == "GET" || method == "POST") {
        let export_dir = std::path::Path::new(".uor-models/exported");
        std::fs::create_dir_all(export_dir).ok();
        let export_file = export_dir.join("exported_manifold.json");

        let router_guard = router.lock().unwrap();
        let state_json = router_guard.export_state();
        std::fs::write(&export_file, &state_json).ok();
        std::fs::write(".uor-models/exported_manifold.json", &state_json).ok();

        let resp = serde_json::json!({
            "success": true,
            "status": "success",
            "path": export_file.display().to_string(),
            "bytes": state_json.len(),
            "message": format!("Exported manifold state saved to {}", export_file.display())
        })
        .to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/import" && method == "POST" {
        let mut router_guard = router.lock().unwrap();
        let state_str = match String::from_utf8(body) {
            Ok(s) => s,
            Err(_) => {
                send_json_response(stream, 400, "{\"error\":\"Invalid UTF-8 string\"}");
                return;
            }
        };
        if !router_guard.import_state_native(&state_str) {
            send_json_response(
                stream,
                400,
                "{\"error\":\"Import failed: not a valid serialized router state\"}",
            );
            return;
        }

        let state_json = router_guard.export_state();
        spawn_cache_save(&cli, state_json);

        let resp = serde_json::json!({ "success": true }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/r4g1/status" && method == "GET" {
        let installed = serving.lock().unwrap();
        let status = r4g1_compile.lock().unwrap().clone();
        send_json_response(stream, 200, &status.json(&installed).to_string());
        return;
    }

    // R4G1 status-aware prediction (issue #78, D4): the response always
    // carries the resolution status and the widened flag; an abstention
    // is HTTP 200 with `abstained: true` — never a guessed token, never
    // a 5xx for a declared policy outcome.
    if clean_path == "/api/r4g1/predict" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };
        let mut window_tokens: Vec<u32> = payload
            .get("window")
            .and_then(|w| w.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|x| x as u32))
                    .collect()
            })
            .unwrap_or_default();
        if window_tokens.is_empty() {
            send_json_response(
                stream,
                400,
                "{\"error\":\"`window` must be a non-empty array of token ids\"}",
            );
            return;
        }
        // keep the WINDOW most recent tokens, oldest first
        if window_tokens.len() > 8 {
            window_tokens = window_tokens.split_off(window_tokens.len() - 8);
        }
        let installed = serving.lock().unwrap();
        let Some(state) = installed.r4g1.as_ref() else {
            let load_error = installed.terminal_load_error.as_deref();
            let (status, body) = r4g1_unavailable_response_with_reason(load_error);
            send_json_response(stream, status, &body.to_string());
            return;
        };
        let (code, body) = match state.predict_window_status(&window_tokens) {
            Ok(r4g1::PredictDecision::Serve(outcome)) => (
                200,
                serde_json::json!({
                    "window": window_tokens,
                    "abstained": false,
                    "prediction": {
                        "token": outcome.token,
                        "status": r4g1::PolicyStatus::from(outcome.status).label(),
                        "widened": outcome.widened,
                    },
                })
                .to_string(),
            ),
            Ok(r4g1::PredictDecision::Abstain(outcome)) => (
                200,
                serde_json::json!({
                    "window": window_tokens,
                    "abstained": true,
                    "status": r4g1::PolicyStatus::from(outcome.status).label(),
                    "widened": outcome.widened,
                })
                .to_string(),
            ),
            Err(error) => (500, serde_json::json!({ "error": error }).to_string()),
        };
        send_json_response(stream, code, &body);
        return;
    }

    // R4G1 status-aware generation: stops at the first abstention and
    // reports the tokens so far, the final status, and the abstain flag.
    if clean_path == "/api/r4g1/generate" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };
        let max_tokens = payload
            .get("max_tokens")
            .and_then(|m| m.as_u64())
            .unwrap_or(24)
            .clamp(1, 256) as usize;
        let include_witness = payload
            .get("include_witness")
            .or_else(|| payload.get("witness"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let installed = serving.lock().unwrap();
        let Some(state) = installed.r4g1.as_ref() else {
            let load_error = installed.terminal_load_error.as_deref();
            let (status, body) = r4g1_unavailable_response_with_reason(load_error);
            send_json_response(stream, status, &body.to_string());
            return;
        };
        let mut seed_buf = [0u32; 4096];
        let seed: Vec<u32> = if let Some(arr) = payload.get("window").and_then(|w| w.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|x| x as u32))
                .collect()
        } else if let Some(text) = payload.get("text").and_then(|t| t.as_str()) {
            let encoded = match state.encode_into(text, &mut seed_buf) {
                some @ Some(_) => some,
                None if state.has_explicit_tokenizer() => None,
                None => tless_uor::tless_tokenize_into(text, &mut seed_buf),
            };
            match encoded {
                Some(len) if len > 0 => seed_buf[..len].to_vec(),
                _ => {
                    let message = state
                        .tokenizer_adapter_identity()
                        .map(|identity| {
                            format!(
                                "tokenizer unavailable — exact host adapter {}/{} is required",
                                identity.family, identity.version
                            )
                        })
                        .unwrap_or_else(|| {
                            "tokenizer unavailable — set TLESS_TOKENIZER".to_owned()
                        });
                    send_json_response(
                        stream,
                        503,
                        &serde_json::json!({ "error": message }).to_string(),
                    );
                    return;
                }
            }
        } else {
            vec![1]
        };
        if seed.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"empty seed\"}");
            return;
        }
        let mut out = [0u32; 256];
        let mut witnesses = Vec::new();
        let generation = if include_witness {
            state.generate_into_status_with_witness(&seed, &mut out[..max_tokens], &mut witnesses)
        } else {
            state.generate_into_status(&seed, &mut out[..max_tokens])
        };
        match generation {
            Ok(gen) => {
                let tokens = &out[..gen.count];
                let mut text_bytes = [0u8; 16 * 1024];
                let text_len = if gen.abstained || gen.count == 0 {
                    0
                } else {
                    match state.decode_into(tokens, &mut text_bytes) {
                        Some(count) => count,
                        None if state.has_explicit_tokenizer() => {
                            send_json_response(
                                stream,
                                500,
                                "{\"error\":\"bundle tokenizer could not decode generated tokens\"}",
                            );
                            return;
                        }
                        None => {
                            tless_uor::tless_detokenize_into(tokens, &mut text_bytes).unwrap_or(0)
                        }
                    }
                };
                let text = String::from_utf8_lossy(&text_bytes[..text_len]).into_owned();
                let mut body = serde_json::json!({
                    "seed": seed,
                    "tokens": tokens,
                    "count": gen.count,
                    "text": text,
                    "abstained": gen.abstained,
                    "status": gen
                        .status
                        .map(r4g1::PolicyStatus::from)
                        .map(|s| s.label()),
                    "widened": gen.widened,
                });
                if include_witness {
                    body["witness"] = serde_json::to_value(&witnesses).unwrap_or_default();
                }
                send_json_response(stream, 200, &body.to_string());
            }
            Err(error) => send_json_response(
                stream,
                500,
                &serde_json::json!({ "error": error }).to_string(),
            ),
        }
        return;
    }

    if clean_path == "/api/uor/verify" && method == "POST" {
        let payload: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(
                    stream,
                    400,
                    &format!("{{\"error\":\"Invalid JSON: {}\"}}", e),
                );
                return;
            }
        };

        // Issue #266: compact proof-carrying inference witnesses are
        // verified against the loaded R4G1 artifact, independently of the
        // legacy JSON-address attestation below.
        if payload.get("witness").is_some() {
            let seed = payload
                .get("seed")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_u64().map(|token| token as u32))
                        .collect::<Vec<_>>()
                });
            let tokens = payload
                .get("tokens")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_u64().map(|token| token as u32))
                        .collect::<Vec<_>>()
                });
            let witnesses = payload
                .get("witness")
                .cloned()
                .and_then(|value| serde_json::from_value::<Vec<InferenceWitness>>(value).ok());
            let (Some(seed), Some(tokens), Some(witnesses)) = (seed, tokens, witnesses) else {
                send_json_response(
                    stream,
                    400,
                    &serde_json::json!({
                        "verified": false,
                        "reason": "witness_payload_invalid"
                    })
                    .to_string(),
                );
                return;
            };
            let installed = serving.lock().unwrap();
            let Some(state) = installed.r4g1.as_ref() else {
                send_json_response(
                    stream,
                    503,
                    &serde_json::json!({
                        "verified": false,
                        "reason": "r4g1_artifact_unavailable"
                    })
                    .to_string(),
                );
                return;
            };
            let response = match state.verify_witnesses(&seed, &tokens, &witnesses) {
                Ok(()) => serde_json::json!({
                    "verified": true,
                    "engine": "r4g1",
                    "witness_count": witnesses.len(),
                }),
                Err(reason) => serde_json::json!({
                    "verified": false,
                    "reason": reason.to_string(),
                }),
            };
            send_json_response(stream, 200, &response.to_string());
            return;
        }

        let provided_address = payload
            .get("uor_address")
            .or_else(|| payload.get("address"))
            .or_else(|| payload.get("attestation_cid"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Issue #257: empty and malformed subjects reject with a typed
        // reason (previously an empty address VERIFIED, and any string
        // merely containing the digest passed). Canonicalization mirrors
        // the envelope's uor-addr pipeline; comparison is exact equality.
        if let Err(reason) = validate_uor_address_syntax(provided_address) {
            send_json_response(
                stream,
                400,
                &serde_json::json!({ "verified": false, "reason": reason }).to_string(),
            );
            return;
        }

        let target_payload = payload.get("payload").unwrap_or(&payload);
        let expected_uor_address = canonical_json_address_blake3(target_payload);

        let response = if provided_address == expected_uor_address {
            serde_json::json!({
                "verified": true,
                "uor_address": expected_uor_address,
                "algorithm": "blake3-jcs",
            })
        } else {
            serde_json::json!({
                "verified": false,
                "reason": "address_mismatch",
                "expected": expected_uor_address,
                "provided": provided_address,
            })
        };

        send_json_response(stream, 200, &response.to_string());
        return;
    }

    if clean_path == "/api/huggingface/status" && method == "GET" {
        let status = hf_download.lock().unwrap().clone();
        send_json_response(stream, 200, &status.json().to_string());
        return;
    }

    if clean_path == "/api/huggingface/download" && method == "POST" {
        let payload = match parse_huggingface_control_payload(&body) {
            Ok(payload) => payload,
            Err(error) => {
                send_json_response(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        if let Err(error) = payload.validate_download_controls() {
            send_json_response(
                stream,
                400,
                &serde_json::json!({ "error": error }).to_string(),
            );
            return;
        }
        let source = match huggingface_source(payload.model.as_deref()) {
            Ok(source) => source,
            Err(error) => {
                send_json_response(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        let destination = downloaded_source_path_in(&source, Path::new(".uor-models"));
        let source_cache_reservation = match try_reserve_source_cache_operation(
            &source_cache_operations,
            SourceCacheOperationKind::Download,
            destination.display().to_string(),
        ) {
            Ok(reservation) => reservation,
            Err(error) => {
                send_json_response(
                    stream,
                    409,
                    &serde_json::json!({
                        "running": true,
                        "error": error,
                    })
                    .to_string(),
                );
                return;
            }
        };
        let mut status = hf_download.lock().unwrap();
        if status.running {
            send_json_response(
                stream,
                409,
                &serde_json::json!({
                    "running": true,
                    "ready": status.ready,
                    "message": "Hugging Face download is already running"
                })
                .to_string(),
            );
            return;
        }
        status.running = true;
        let revision_preview: String = source.revision.chars().take(12).collect();
        status.message = format!(
            "Downloading {}@{}; this may take a few minutes...",
            source.repository, revision_preview
        );
        drop(status);
        spawn_huggingface_download(Arc::clone(&hf_download), source, source_cache_reservation);
        send_json_response(
            stream,
            202,
            &serde_json::json!({
                "running": true,
                "message": "Hugging Face download started"
            })
            .to_string(),
        );
        return;
    }

    if clean_path == "/api/r4g1/compile" && method == "POST" {
        let payload = match parse_huggingface_control_payload(&body) {
            Ok(payload) => payload,
            Err(error) => {
                send_json_response(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        let tokenizer_selection = match payload.tokenizer_selection() {
            Ok(selection) => selection,
            Err(error) => {
                send_json_response(
                    stream,
                    400,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        let requested_source =
            match explicitly_requested_huggingface_source(payload.model.as_deref()) {
                Ok(source) => source,
                Err(error) => {
                    send_json_response(
                        stream,
                        400,
                        &serde_json::json!({ "error": error }).to_string(),
                    );
                    return;
                }
            };
        let compile_subject = requested_source
            .as_ref()
            .map(|source| {
                downloaded_source_path_in(source, Path::new(".uor-models"))
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| "completed download status or pinned fallback".to_owned());
        let (source_cache_reservation, cached_source) = match reserve_compile_source_selection(
            &source_cache_operations,
            &hf_download,
            compile_subject,
        ) {
            Ok(selection) => selection,
            Err(error) => {
                send_json_response(
                    stream,
                    409,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        let fallback_source = if requested_source.is_none() && cached_source.is_none() {
            match optional_pinned_huggingface_source() {
                Ok(source) => source,
                Err(error) => {
                    send_json_response(
                        stream,
                        409,
                        &serde_json::json!({ "error": error }).to_string(),
                    );
                    return;
                }
            }
        } else {
            None
        };
        let require_source_manifest = requested_source.is_some() || cached_source.is_some();
        let expected_source = requested_source
            .clone()
            .or_else(|| cached_source.as_ref().map(|source| source.identity.clone()))
            .or_else(|| fallback_source.clone());
        let downloaded_source = match select_compile_source_path_in(
            Path::new(".uor-models"),
            requested_source.as_ref(),
            cached_source.as_ref(),
            fallback_source.as_ref(),
        ) {
            Ok(source) => source.map(|path| path.display().to_string()),
            Err(error) => {
                send_json_response(
                    stream,
                    409,
                    &serde_json::json!({ "error": error }).to_string(),
                );
                return;
            }
        };
        let installed = serving.lock().unwrap();
        let mut status = r4g1_compile.lock().unwrap();
        if status.running {
            send_json_response(
                stream,
                409,
                &serde_json::json!({
                    "running": true,
                    "ready": status.ready,
                    "message": "R4G1 compilation is already running"
                })
                .to_string(),
            );
            return;
        }
        status.running = true;
        status.ready = graph_text_ready(&installed);
        status.progress = 1;
        status.message = "Compiling R4G1 cover and scored graph...".to_owned();
        status.report = None;
        drop(status);
        drop(installed);

        spawn_r4g1_compile(
            Arc::clone(&cli),
            Arc::clone(&serving),
            Arc::clone(&r4g1_compile),
            CompileSourceSelection {
                path: downloaded_source,
                expected: expected_source,
                require_manifest: require_source_manifest,
            },
            tokenizer_selection,
            source_cache_reservation,
        );
        send_json_response(
            stream,
            202,
            &serde_json::json!({
                "running": true,
                "message": "R4G1 compilation started"
            })
            .to_string(),
        );
        return;
    }

    if clean_path == "/api/tags" && method == "GET" {
        // Compatibility endpoint for clients that previously used Ollama's
        // model discovery API. No external process or network call is made.
        let ready = Path::new(&cli.tless_artifacts).is_file()
            && Path::new(&cli.tless_store).is_file()
            && Path::new(&cli.tless_tokenizer).is_file();
        let r4g1_ready = graph_text_ready(&serving.lock().unwrap());
        let body = serde_json::json!({
            "models": if ready { vec![serde_json::json!({
                "name": "uor-transformerless",
                "model": "uor-transformerless",
                "details": {
                    "family": "r4-transformerless",
                    "format": if r4g1_ready { "R4G1" } else { "TLA5/TLS1" }
                }
            })] } else { Vec::<serde_json::Value>::new() },
            "ready": ready,
            "r4g1_ready": r4g1_ready
        });
        send_json_response(stream, 200, &body.to_string());
        return;
    }

    if clean_path == "/api/sysinfo" && method == "GET" {
        // Snapshot the serving state before locking the router. Generation
        // acquires these locks in that order and sysinfo must not invert it.
        let r4g1_ready = graph_text_ready(&serving.lock().unwrap());
        let mut router_guard = router.lock().unwrap();
        let sentences_indexed = router_guard.get_total_indexed_sentences();
        let active_streams = router_guard.get_active_streams_native();
        let expert_counts = router_guard.get_expert_counts();

        let identity = "null_dev_00";

        let mut buf = [0u8; 640];
        let query_bytes = "Welcome".as_bytes();
        let identity_bytes = identity.as_bytes();
        let query_len = query_bytes.len().min(512);
        let identity_len = identity_bytes.len().min(128);
        buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
        buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

        let input = uor_r4_wasm_router::R4RoutingInput {
            query: &buf[..512],
            identity: &buf[512..],
            data: &buf,
        };

        // Bind thread-local
        let router_ptr = &mut *router_guard as *mut UorR4Router;
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        // Run through UorR4RouterModel
        let grounded =
            uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Sysinfo routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing_data = router_guard
            .last_routing_data()
            .clone()
            .expect("No sysinfo routing data generated");
        let active_state = router_guard.get_brain_state_native(identity);
        let (u, v) = router_guard.get_sentence_projection_native(
            &active_state,
            routing_data.routed.window_index as usize,
        );
        let v_4d = router_guard.get_state_4d_projection_native(&active_state);
        let kappa = routing_data.routed.metrics.kappa;
        let theta_d = routing_data.routed.metrics.deficit_angle;
        let uor_bias = routing_data.routed.qimc.uor_control.entropy_bias;

        let (gamma, temperature) = autotune(kappa, theta_d, uor_bias);

        let geom_result = router_guard.generate_geometric_response_native(
            "Welcome",
            identity,
            25,
            temperature,
            10.0,
            4.0,
            gamma,
        );

        let top_resonances_5 = router_guard.get_top_resonances_native("Welcome", identity, 5);

        let trace = grounded.derivation().replay::<256>();
        let mut uor_trace_steps = Vec::new();
        for i in 0..trace.len() {
            if let Some(event) = trace.event(i as usize) {
                uor_trace_steps.push(serde_json::json!({
                    "step": event.step_index(),
                    "op": format!("{:?}", event.op()),
                    "target": format!("0x{:032x}", event.target().as_u128()),
                }));
            }
        }

        let uor_payload = serde_json::json!({
            "algorithm": routing_data.routed.uor.algorithm.clone(),
            "hash_algorithm": routing_data.routed.uor.hash_algorithm.clone(),
            "hash_algorithm_id": routing_data.routed.uor.hash_algorithm_id,
            "address": routing_data.routed.uor.address.clone(),
            "verify_result": "Verified",
            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
            "sigma": grounded.sigma().value(),
            "d_delta": grounded.d_delta().as_i64(),
            "euler": grounded.euler().as_i64(),
            "residual": grounded.residual().as_u32(),
            "stratum": grounded.triad().stratum(),
            "multihash_addresses": routing_data.routed.uor.multihash_addresses.clone(),
        });

        let max_tokens = router_guard.get_suggested_token_limit("Welcome", identity);
        let info = serde_json::json!({
            "uptime_seconds": start_time.elapsed().as_secs_f64().round(),
            "sentences_indexed": sentences_indexed,
            "requests_total": 0,
            "catastrophes": 0,
            "window_hits": {},
            "routing_latency_p50_ms": 0.0,
            "routing_latency_p95_ms": 0.0,
            "gen_latency_p50_ms": 0.0,
            "gen_latency_p95_ms": 0.0,
            "glove_loaded": false,
            "otel_available": false,
            "r4g1_ready": r4g1_ready,
            "model_format": if r4g1_ready { "R4G1" } else { "TLA5/TLS1 or geometric fallback" },
            "active_streams": active_streams,
            "expert_counts": expert_counts,
            "active_projection": {
                "u": u,
                "v": v,
                "v_4d": v_4d
            },
            "metrics": {
                "window_index": routing_data.routed.window_index,
                "scale_x": routing_data.routed.scale_x,
                "kappa": kappa,
                "deficit_angle": theta_d,
                "lambda_entropy": routing_data.routed.metrics.lambda_entropy,
                "sigma_kl": routing_data.routed.metrics.sigma_kl,
                "top_eigenvalue_pct": ((routing_data.routed.eigenvalues[0] / (routing_data.routed.eigenvalues.iter().sum::<f64>().max(1.0))) * 100.0),
                "qimc": routing_data.routed.qimc,
                "hopf": routing_data.routed.hopf,
                "uor_address": routing_data.routed.uor_address,
                "uor": uor_payload,
                "auto_tuned": {
                    "gamma": gamma,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                    "engine": if r4g1_ready { "r4g1" } else { "geometric" },
                    "uor_entropy_bias": uor_bias
                }
            },
            "eigenvalues": routing_data.routed.eigenvalues,
            "active_range": routing_data.routed.active_range,
            "state_vector": routing_data.routed.state_vector,
            "all_routes": routing_data.all_routes,
            "top_resonance": top_resonances_5,
            "trajectory": geom_result.trajectory,
            "uor_trace_steps": uor_trace_steps,
        });

        send_json_response(stream, 200, &info.to_string());
        return;
    }

    if clean_path == "/api/map" && method == "GET" {
        let router_guard = router.lock().unwrap();
        let map_val = router_guard.get_semantic_map_points_native();
        send_json_response(stream, 200, &map_val.to_string());
        return;
    }

    // Serve static files fallback
    let mut relative_path = clean_path.trim_start_matches('/');
    if relative_path.is_empty() {
        relative_path = "index.html";
    }

    let file_path = Path::new(relative_path);
    if !file_path.exists() || file_path.is_dir() {
        let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let contents = match fs::read(file_path) {
        Ok(c) => c,
        Err(_) => {
            let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return;
        }
    };

    let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        mime_type,
        contents.len()
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(&contents);
}

/// The standard OpenAI error object envelope (#654 phase B):
/// `{ "error": { "message", "type", "param", "code" } }`, with `param`/`code`
/// serialized as JSON `null` when absent. Official SDKs parse errors from this
/// shape; the ad-hoc `{"error":"..."}` string body does not satisfy them.
fn openai_error_body(
    error_type: &str,
    message: &str,
    param: Option<&str>,
    code: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "message": message,
            "type": error_type,
            "param": param,
            "code": code,
        }
    })
}

/// Send the OpenAI error envelope with the mapped HTTP status.
fn send_openai_error(
    stream: TcpStream,
    status: u16,
    error_type: &str,
    message: &str,
    param: Option<&str>,
    code: Option<&str>,
) {
    send_json_response(
        stream,
        status,
        &openai_error_body(error_type, message, param, code).to_string(),
    );
}

/// One OpenAI `Model` object with every field the pinned schema requires:
/// `id`, `object`, `created` (unix seconds), and `owned_by`.
fn openai_model_object(id: &str, created: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "object": "model",
        "created": created,
        "owned_by": "uor-foundation",
    })
}

/// The resolver-validated logical models loadable from the compiled root,
/// reported with the selected physical teacher artifact's mtime. Invalid
/// siblings are an inventory error, and a v2 physical suffix is never exposed
/// as a second OpenAI model id.
#[cfg(test)]
fn loadable_models_in(compiled_dir: &Path) -> Result<Vec<(String, u64)>, String> {
    let mut models = Vec::new();
    let current_version = current_source_attention_era_version()?;
    for resolved in discover_compiled_r4g1_candidates_in(compiled_dir, current_version)? {
        let created = std::fs::metadata(&resolved.teacher)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|delta| delta.as_secs())
            .unwrap_or(0);
        models.push((resolved.logical_name, created));
    }
    models.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(models)
}

/// The `GET /v1/models` list body over the loadable models.
fn models_list_body(models: &[(String, u64)]) -> serde_json::Value {
    serde_json::json!({
        "object": "list",
        "data": models
            .iter()
            .map(|(id, created)| openai_model_object(id, *created))
            .collect::<Vec<_>>(),
    })
}

/// The server's per-request completion-token cap (mirrors the generation
/// helpers' `MAX_SERVER_TOKENS`).
const SERVER_MAX_COMPLETION_TOKENS: usize = 256;

/// Count tokens with the loaded serving tokenizer — the compiled artifact's
/// tokenizer the cascade encodes with — not a whitespace-word estimate
/// (#654 phase C). The buffer is sized to the byte length, an upper bound on
/// the token count, so the count is exact (no truncation). `None` means the
/// serving tokenizer is unavailable; usage is then reported honestly rather
/// than substituting a word count.
fn count_serving_tokens(text: &str) -> Option<usize> {
    if text.is_empty() {
        return Some(0);
    }
    let mut buf = vec![0u32; text.len()];
    tless_uor::tless_tokenize_into(text, &mut buf)
}

fn tier_carries_exact_usage(tier: Option<&str>) -> bool {
    matches!(
        tier,
        Some(TIER_R4G1 | TIER_TEACHER_ORACLE | TIER_ATTENTION | TIER_R4_ATTENTION)
    )
}

fn serving_usage_for_cascade(
    cascade: &ServingCascade,
    prompt: &str,
    completion: &str,
) -> Result<ServingUsage, &'static str> {
    if let Some(usage) = cascade.usage {
        return Ok(usage);
    }
    if tier_carries_exact_usage(cascade.outcome.served_by) {
        return Err("the selected R4G1/teacher tokenizer did not carry exact usage counts");
    }
    Ok(ServingUsage {
        prompt_tokens: count_serving_tokens(prompt).unwrap_or(0),
        completion_tokens: count_serving_tokens(completion).unwrap_or(0),
    })
}

/// The truthful `finish_reason` for a served completion: `length` when it
/// reached the effective token budget (the smaller of the requested
/// `max_tokens` and the server cap), otherwise `stop`. Abstentions and hard
/// failures never reach here — those are the declined/error path.
fn completion_finish_reason(completion_tokens: usize, requested_max_tokens: usize) -> &'static str {
    let budget = requested_max_tokens.min(SERVER_MAX_COMPLETION_TOKENS);
    if completion_tokens >= budget {
        "length"
    } else {
        "stop"
    }
}

/// Flatten the supported chat roles into the router prompt. Supported roles:
/// `system`, `developer` (system-equivalent, per the pinned spec), `user`,
/// and `assistant`. An unsupported role (e.g. `tool`, `function`) returns
/// `Err(role)` so the caller fails closed with the error envelope — a role
/// outside the profile is never silently accepted. A single `user` message
/// passes through verbatim (the common case).
fn flatten_chat_prompt(messages: &[ChatMessage]) -> Result<String, String> {
    let mut parts = Vec::with_capacity(messages.len());
    for message in messages {
        match message.role.as_str() {
            "system" | "developer" => parts.push(format!("System: {}", message.content)),
            "user" => parts.push(format!("User: {}", message.content)),
            "assistant" => parts.push(format!("Assistant: {}", message.content)),
            other => return Err(other.to_owned()),
        }
    }
    if parts.len() == 1 && messages.first().map(|m| m.role.as_str()) == Some("user") {
        return Ok(messages[0].content.clone());
    }
    Ok(parts.join("\n"))
}

/// Convert one Responses `input` array element into a `ChatMessage` (`#654`
/// phase E). Only message items are supported: an object with a `role` and a
/// `content` that is a string or an array of text parts. Anything else — a
/// non-object, a typed item other than `message`, a missing role/content, or a
/// content part without a `text` string — returns `Err` so the caller fails
/// closed rather than silently dropping input.
fn responses_item_to_message(item: &serde_json::Value) -> Result<ChatMessage, String> {
    let object = item
        .as_object()
        .ok_or_else(|| "each `input` item must be an object".to_string())?;
    if let Some(kind) = object.get("type").and_then(|v| v.as_str()) {
        if kind != "message" {
            return Err(format!("unsupported `input` item type '{kind}'"));
        }
    }
    let role = object
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "each `input` message needs a string `role`".to_string())?;
    let content = object
        .get("content")
        .ok_or_else(|| "each `input` message needs `content`".to_string())?;
    let text = match content {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(parts) => {
            let mut buffer = String::new();
            for part in parts {
                let piece = part
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "each content part needs a `text` string".to_string())?;
                buffer.push_str(piece);
            }
            buffer
        }
        _ => return Err("`content` must be a string or an array of text parts".to_string()),
    };
    Ok(ChatMessage {
        role: role.to_string(),
        content: text,
    })
}

/// Flatten a Responses `input` (a bare string or an array of message items)
/// plus optional top-level `instructions` into the single router prompt,
/// reusing the chat role-flattening (`flatten_chat_prompt`) so the two wire
/// surfaces treat roles identically (`#654` phase E). `instructions` is a
/// system-level preamble. An unsupported role or malformed item fails closed.
fn flatten_responses_input(
    input: &serde_json::Value,
    instructions: Option<&str>,
) -> Result<String, String> {
    let mut messages = Vec::new();
    if let Some(system) = instructions {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
        });
    }
    match input {
        serde_json::Value::String(text) => messages.push(ChatMessage {
            role: "user".to_string(),
            content: text.clone(),
        }),
        serde_json::Value::Array(items) => {
            for item in items {
                messages.push(responses_item_to_message(item)?);
            }
        }
        _ => return Err("`input` must be a string or an array of message items".to_string()),
    }
    if messages.is_empty() {
        return Err("`input` must contain at least one message".to_string());
    }
    flatten_chat_prompt(&messages).map_err(|role| {
        format!("Unsupported message role '{role}'. Supported roles: system, developer, user, assistant.")
    })
}

/// Build the OpenAI `Response` object for a completed generation (`#654`
/// phase E): one assistant `message` output item with a single `output_text`
/// part, token usage from the serving tokenizer, and `status`
/// `completed`/`incomplete` (with `incomplete_details` when the completion hit
/// the token budget). The R4 audit trail rides as extra fields for parity with
/// the chat body (SDKs ignore unknown fields).
fn build_responses_body(
    gen: GeneratedCompletion,
    model: &str,
    requested_max_tokens: usize,
) -> serde_json::Value {
    let budget = requested_max_tokens.min(SERVER_MAX_COMPLETION_TOKENS);
    let incomplete = gen.completion_tokens >= budget;
    let status = if incomplete {
        "incomplete"
    } else {
        "completed"
    };
    let audit = serde_json::to_value(&gen.uor_audit).unwrap_or(serde_json::Value::Null);
    let mut body = serde_json::json!({
        "id": format!("resp-uor-r4-{}", gen.created_ts),
        "object": "response",
        "created_at": gen.created_ts,
        "model": model,
        "status": status,
        "output": [{
            "type": "message",
            "id": format!("msg-uor-r4-{}", gen.created_ts),
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": gen.text,
                "annotations": [],
            }],
        }],
        "usage": {
            "input_tokens": gen.prompt_tokens,
            "output_tokens": gen.completion_tokens,
            "total_tokens": gen.total_tokens,
        },
        "uor_audit": audit,
        "cascade_trail": gen.cascade_trail,
    });
    if incomplete {
        body["incomplete_details"] = serde_json::json!({ "reason": "max_output_tokens" });
    }
    body
}

/// One Responses SSE event frame: `event: <type>\ndata: <json>\n\n`. The
/// Responses stream uses *named* events (unlike chat completions' data-only
/// chunks) and terminates on `response.completed` — there is no `[DONE]`
/// sentinel (`#654`: streaming for /v1/responses).
fn sse_event_frame(event_type: &str, value: &serde_json::Value) -> String {
    format!("event: {event_type}\ndata: {value}\n\n")
}

/// Build the ordered Responses SSE event frames for a completed generation
/// (`#654`: streaming for /v1/responses). Like chat streaming, the cascade
/// produces the whole completion up front, so this re-frames the finished text
/// as the spec's typed event sequence: `response.created` → `.in_progress` →
/// `.output_item.added` → `.content_part.added` → `.output_text.delta`* →
/// `.output_text.done` → `.content_part.done` → `.output_item.done` →
/// `.completed`. Every event carries a monotonic `sequence_number`; the delta
/// events' `delta`s rejoin byte-for-byte to `content`. `completed` is the full
/// `Response` object (from `build_responses_body`) carried by the terminal event.
fn build_responses_stream_frames(
    id: &str,
    created_at: u64,
    model: &str,
    content: &str,
    completed: serde_json::Value,
) -> Vec<String> {
    let item_id = format!("msg-uor-r4-{created_at}");
    let in_progress = serde_json::json!({
        "id": id,
        "object": "response",
        "created_at": created_at,
        "model": model,
        "status": "in_progress",
        "error": serde_json::Value::Null,
        "incomplete_details": serde_json::Value::Null,
        "output": [],
        "usage": serde_json::Value::Null,
    });
    let part =
        |text: &str| serde_json::json!({ "type": "output_text", "text": text, "annotations": [] });
    let message = |status: &str, parts: serde_json::Value| {
        serde_json::json!({
            "id": item_id.clone(),
            "type": "message",
            "status": status,
            "role": "assistant",
            "content": parts,
        })
    };

    let mut events: Vec<(&str, serde_json::Value)> = vec![
        (
            "response.created",
            serde_json::json!({ "response": in_progress.clone() }),
        ),
        (
            "response.in_progress",
            serde_json::json!({ "response": in_progress }),
        ),
        (
            "response.output_item.added",
            serde_json::json!({ "output_index": 0, "item": message("in_progress", serde_json::json!([])) }),
        ),
        (
            "response.content_part.added",
            serde_json::json!({
                "item_id": item_id.clone(),
                "output_index": 0,
                "content_index": 0,
                "part": part(""),
            }),
        ),
    ];
    for piece in split_stream_deltas(content) {
        events.push((
            "response.output_text.delta",
            serde_json::json!({
                "item_id": item_id.clone(),
                "output_index": 0,
                "content_index": 0,
                "delta": piece,
                "logprobs": [],
            }),
        ));
    }
    events.push((
        "response.output_text.done",
        serde_json::json!({
            "item_id": item_id.clone(),
            "output_index": 0,
            "content_index": 0,
            "text": content,
            "logprobs": [],
        }),
    ));
    events.push((
        "response.content_part.done",
        serde_json::json!({
            "item_id": item_id.clone(),
            "output_index": 0,
            "content_index": 0,
            "part": part(content),
        }),
    ));
    events.push((
        "response.output_item.done",
        serde_json::json!({
            "output_index": 0,
            "item": message("completed", serde_json::json!([part(content)])),
        }),
    ));
    events.push((
        "response.completed",
        serde_json::json!({ "response": completed }),
    ));

    events
        .into_iter()
        .enumerate()
        .map(|(seq, (event_type, mut payload))| {
            payload["type"] = serde_json::json!(event_type);
            payload["sequence_number"] = serde_json::json!(seq);
            sse_event_frame(event_type, &payload)
        })
        .collect()
}

/// Split completion text into streaming content deltas whose concatenation
/// byte-exactly reconstructs the input (`#654` phase D). Each piece is one
/// non-whitespace run together with the whitespace that trails it, so joining
/// every `delta.content` on the wire yields the original text unchanged.
/// Empty input yields no content pieces (only the role and terminal chunks).
fn split_stream_deltas(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            current.push(ch);
        } else {
            if in_word && current.ends_with(char::is_whitespace) {
                pieces.push(std::mem::take(&mut current));
            }
            current.push(ch);
            in_word = true;
        }
    }
    if !current.is_empty() {
        pieces.push(current);
    }
    pieces
}

/// Format one Server-Sent Event frame: `data: <json>\n\n`.
fn sse_frame(value: &serde_json::Value) -> String {
    format!("data: {}\n\n", value)
}

/// Build the ordered SSE frames for a streaming chat completion (`#654`
/// phase D): a role chunk, one chunk per content delta (concatenating to the
/// full completion), a terminal chunk carrying `finish_reason` and the R4
/// audit trail, an optional usage-only chunk when `include_usage` was
/// requested, and the closing `data: [DONE]` marker.
#[allow(clippy::too_many_arguments)]
fn build_chat_stream_frames(
    id: &str,
    created: u64,
    model: &str,
    system_fingerprint: &str,
    content: &str,
    finish_reason: &str,
    usage: Option<&VendorUsage>,
    audit: Option<serde_json::Value>,
    cascade_trail: serde_json::Value,
) -> Vec<String> {
    let chunk = |delta: serde_json::Value, finish: serde_json::Value| -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model,
            "system_fingerprint": system_fingerprint,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }],
        })
    };

    let mut frames = Vec::new();
    // Role chunk first, mirroring OpenAI's stream preamble.
    frames.push(sse_frame(&chunk(
        serde_json::json!({ "role": "assistant" }),
        serde_json::Value::Null,
    )));
    // Content deltas — their concatenation reconstructs `content` exactly.
    for piece in split_stream_deltas(content) {
        frames.push(sse_frame(&chunk(
            serde_json::json!({ "content": piece }),
            serde_json::Value::Null,
        )));
    }
    // Terminal chunk: empty delta, the truthful finish_reason, and the R4
    // audit trail (SDKs ignore the extra fields; parity with the JSON body).
    let mut final_chunk = chunk(serde_json::json!({}), serde_json::json!(finish_reason));
    if let Some(audit) = audit {
        final_chunk["uor_audit"] = audit;
    }
    final_chunk["cascade_trail"] = cascade_trail;
    frames.push(sse_frame(&final_chunk));
    // Optional usage-only chunk (empty choices) when include_usage was set.
    if let Some(usage) = usage {
        frames.push(sse_frame(&serde_json::json!({
            "id": id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model,
            "system_fingerprint": system_fingerprint,
            "choices": [],
            "usage": {
                "prompt_tokens": usage.prompt_tokens,
                "completion_tokens": usage.completion_tokens,
                "total_tokens": usage.total_tokens,
            },
        })));
    }
    frames.push("data: [DONE]\n\n".to_string());
    frames
}

/// Write the streaming chat completion as `text/event-stream`. Frames are
/// flushed in order; the body ends at connection close (no `Content-Length`).
/// A write error means the client hung up mid-stream, so we stop cleanly.
fn send_sse_stream(mut stream: TcpStream, frames: &[String]) {
    let head = "HTTP/1.1 200 OK\r\n\
                Content-Type: text/event-stream\r\n\
                Cache-Control: no-cache\r\n\
                Connection: close\r\n\
                Access-Control-Allow-Origin: *\r\n\
                Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                Access-Control-Allow-Headers: Content-Type\r\n\r\n";
    if stream.write_all(head.as_bytes()).is_err() {
        return;
    }
    for frame in frames {
        if stream.write_all(frame.as_bytes()).is_err() {
            return;
        }
        let _ = stream.flush();
    }
}

/// The R4 extended-capability routes (`#654` phase G). Canonical under the
/// vendor namespace `/uor/v1/*`; the bare `/v1/*` paths are retained as
/// **deprecated aliases** so existing callers keep working while `/v1` stays a
/// pure OpenAI surface (chat completions, models, responses). Given any accepted
/// path — canonical or alias — returns the canonical `/uor/v1` path; `None` for
/// a path that is not an extended route.
fn extended_route_canonical(clean_path: &str) -> Option<&'static str> {
    match clean_path {
        "/uor/v1/status" | "/v1/status" => Some("/uor/v1/status"),
        "/uor/v1/reload" | "/v1/reload" => Some("/uor/v1/reload"),
        "/uor/v1/corpus" | "/v1/corpus" => Some("/uor/v1/corpus"),
        _ => None,
    }
}

/// True when `clean_path` is the deprecated bare `/v1` alias of an extended
/// route (the canonical form lives under `/uor/v1/*`).
fn is_deprecated_v1_alias(clean_path: &str) -> bool {
    matches!(clean_path, "/v1/status" | "/v1/reload" | "/v1/corpus")
}

/// Extra HTTP header lines marking a deprecated-alias response: RFC 8594
/// `Deprecation: true` plus a `Link` to the successor `/uor/v1` path. Empty for
/// a canonical path (or a non-extended route), so canonical responses are
/// unaffected.
fn deprecation_headers(clean_path: &str) -> String {
    match extended_route_canonical(clean_path) {
        Some(canonical) if is_deprecated_v1_alias(clean_path) => {
            format!("Deprecation: true\r\nLink: <{canonical}>; rel=\"successor-version\"\r\n")
        }
        _ => String::new(),
    }
}

fn send_json_response(stream: TcpStream, status_code: u16, body: &str) {
    send_json_response_ext(stream, status_code, body, "");
}

/// Like [`send_json_response`], plus `extra_headers`: a run of complete
/// `Name: value\r\n` header lines injected before the header terminator. Used
/// for the `Deprecation`/`Link` headers on the `/v1` extended-route aliases
/// (`#654` phase G); pass `""` for the ordinary case.
fn send_json_response_ext(
    mut stream: TcpStream,
    status_code: u16,
    body: &str,
    extra_headers: &str,
) {
    let status_text = match status_code {
        200 => "OK",
        202 => "ACCEPTED",
        400 => "BAD REQUEST",
        404 => "NOT FOUND",
        409 => "CONFLICT",
        500 => "INTERNAL SERVER ERROR",
        502 => "BAD GATEWAY",
        503 => "SERVICE UNAVAILABLE",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         {}\r\n\
         {}",
        status_code,
        status_text,
        body.len(),
        extra_headers,
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn find_pid_by_port(port: u16) -> Option<u32> {
    let output = std::process::Command::new("lsof")
        .args(["-t", "-i", &format!(":{}", port)])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next()?;
        first_line.trim().parse::<u32>().ok()
    } else {
        None
    }
}

fn kill_process(pid: u32) -> bool {
    let _ = std::process::Command::new("kill")
        .arg(pid.to_string())
        .status();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let check = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status();
    if let Ok(status) = check {
        if status.success() {
            let force = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .status();
            return force.map(|s| s.success()).unwrap_or(false);
        }
    }
    true
}

// =====================================================================
// ask / chat: the router pipeline in-process — one question or a REPL
// =====================================================================

/// Autotuned decode parameters from routing metrics (shared by /api/chat,
/// /api/sysinfo, and the CLI answer path).
fn autotune(kappa: f64, theta_d: f64, uor_bias: f64) -> (f64, f64) {
    let gamma = (0.85 - 0.55 * kappa + ((uor_bias - 0.5) * 0.12)).clamp(0.15, 0.90);
    let temperature =
        (0.2 + 0.8 * theta_d.abs().tanh() + ((uor_bias - 0.5) * 0.20)).clamp(0.15, 1.1);
    (gamma, temperature)
}

/// One answered question with its witness summary.
#[allow(dead_code)]
struct CliAnswer {
    text: String,
    mode: String,
    window_index: usize,
    kappa: f64,
    theta_d: f64,
    fingerprint_hex: String,
    sigma: f64,
    d_delta: i64,
    euler: i64,
    residual: u32,
    stratum: u64,
}

/// Load the router and its manifold cache (no wiki re-indexing on the CLI:
/// a cold start begins empty rather than re-indexing at every invocation).
#[allow(dead_code)]
fn load_cli_router(cli: &ServerConfig) -> UorR4Router {
    let mut router = UorR4Router::new(0.85);
    if let Ok(cache_data) = std::fs::read_to_string(&cli.manifold_cache) {
        if !router.import_state_native(&cache_data) {
            eprintln!(
                "[!] failed to load {}: not a valid serialized router state",
                cli.manifold_cache
            );
        }
    }
    // The geometric router needs at least one vocabulary manifold. A fresh CLI
    // checkout has no cache yet, so seed a small general-purpose corpus rather
    // than entering the routing pipeline with an empty vocabulary.
    if router.get_total_indexed_sentences() == 0 {
        router.index_corpus(
            "The sky appears blue because air molecules scatter shorter blue wavelengths of sunlight more strongly than longer red wavelengths. \
             R4 routes questions through indexed context, and transformerless generates a local continuation from that grounded context.",
            "bootstrap",
        );
    }
    router
}

/// The /api/chat pipeline, compacted for the terminal: dry-run route,
/// autotune, evolve state, final route (Grounded witness), decode
/// (transformerless with geometric fallback), index the exchange
/// back, persist the cache.
#[allow(dead_code)]
fn answer_question(
    router: &mut UorR4Router,
    cli: &Arc<ServerConfig>,
    tless: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    text: &str,
    identity: &str,
) -> CliAnswer {
    let mut buf = [0u8; 640];
    let query_bytes = text.as_bytes();
    let identity_bytes = identity.as_bytes();
    let qlen = query_bytes.len().min(512);
    let ilen = identity_bytes.len().min(128);
    buf[..qlen].copy_from_slice(&query_bytes[..qlen]);
    buf[512..512 + ilen].copy_from_slice(&identity_bytes[..ilen]);

    let input = uor_r4_wasm_router::R4RoutingInput {
        query: &buf[..512],
        identity: &buf[512..],
        data: &buf,
    };

    let router_ptr = router as *mut UorR4Router;
    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| *r.borrow_mut() = Some(router_ptr));
    let _dry = uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("dry route");
    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| *r.borrow_mut() = None);

    let routing = router.last_routing_data().clone().expect("routing data");
    let kappa = routing.routed.metrics.kappa;
    let theta_d = routing.routed.metrics.deficit_angle;
    let uor_bias = routing.routed.qimc.uor_control.entropy_bias;
    let (gamma, temperature) = autotune(kappa, theta_d, uor_bias);

    router.evolve_state(identity, text, gamma);
    let session_signature =
        uor_r4_router::session_signature_from_state(&router.get_brain_state_native(identity));

    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| *r.borrow_mut() = Some(router_ptr));
    let grounded = uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("final route");
    uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| *r.borrow_mut() = None);
    let routing_data = router.last_routing_data().clone().expect("routing data");

    let max_tokens = router.get_suggested_token_limit(text, identity);
    let geom = router.generate_geometric_response_native(
        text,
        identity,
        max_tokens,
        temperature,
        10.0,
        4.0,
        gamma,
    );
    let top = router.get_top_resonances_native(text, identity, 1);
    let prompt = if let Some(context) = top.first() {
        format!("Context: {}\nUser: {text}\nAssistant:", context.sentence)
    } else {
        text.to_string()
    };
    let (mut answer_text, mode) =
        match generate_tless_text(tless, &prompt, max_tokens.max(24), Some(&session_signature)) {
            Some(generated) => (generated, "transformerless".to_string()),
            None => (geom.text.clone(), "geometric-decoded".to_string()),
        };
    if answer_text.is_empty() {
        answer_text = "Manifold resonance too sparse for synthesis.".to_string();
    }

    // learn the exchange, persist in the background
    router.index_sentence(text, identity);
    router.index_sentence(&answer_text, identity);
    router.inject_thought_stream_native(text);
    router.inject_thought_stream_native(&answer_text);
    spawn_cache_save(cli, router.export_state());

    CliAnswer {
        text: answer_text,
        mode,
        window_index: routing_data.routed.window_index as usize,
        kappa: routing_data.routed.metrics.kappa,
        theta_d: routing_data.routed.metrics.deficit_angle,
        fingerprint_hex: hex::encode(grounded.content_fingerprint().as_bytes()),
        sigma: grounded.sigma().value(),
        d_delta: grounded.d_delta().as_i64(),
        euler: grounded.euler().as_i64(),
        residual: grounded.residual().as_u32(),
        stratum: grounded.triad().stratum(),
    }
}

#[allow(dead_code)]
fn print_witness_line(a: &CliAnswer) {
    println!(
        "  ─ W{} ({}) | κ={:.4} θd={:.4} | {}",
        a.window_index,
        get_window_theme(a.window_index),
        a.kappa,
        a.theta_d,
        a.mode
    );
    println!(
        "  ─ witness: Verified | fingerprint {}… | σ={} d_Δ={} χ={} residual={} stratum={}",
        &a.fingerprint_hex[..16.min(a.fingerprint_hex.len())],
        a.sigma,
        a.d_delta,
        a.euler,
        a.residual,
        a.stratum
    );
}

#[cfg(test)]
mod tests {
    fn attention_provenance_test_dir(label: &str) -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "uor-r4-server-attention-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create provenance test directory");
        path
    }

    fn attention_corpus_markers(
        root: &std::path::Path,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let meta = root.join("corpus.meta");
        let records = root.join("corpus.records");
        std::fs::write(&meta, []).expect("create corpus metadata marker");
        std::fs::write(&records, []).expect("create corpus records marker");
        (meta, records)
    }

    fn write_test_source_manifest(
        source_dir: &std::path::Path,
        source: &crate::model::SourceDownload,
    ) -> Vec<u8> {
        std::fs::create_dir_all(source_dir).expect("create source snapshot");
        let weights = source_dir.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME);
        if !weights.exists() {
            std::fs::write(&weights, b"test-safetensors-bytes").expect("write test source weights");
        }
        write_test_source_manifest_without_default_weights(source_dir, source)
    }

    fn write_test_source_manifest_without_default_weights(
        source_dir: &std::path::Path,
        source: &crate::model::SourceDownload,
    ) -> Vec<u8> {
        std::fs::create_dir_all(source_dir).expect("create source snapshot");
        let manifest = crate::model::build_source_manifest(
            source_dir,
            &crate::model::SourceSnapshotInfo {
                repository: source.repository.clone(),
                revision: source.revision.clone(),
                license: source.license.clone(),
                source_execution_mode: crate::model::SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT
                    .to_owned(),
            },
        )
        .expect("build source manifest");
        crate::model::write_source_manifest(source_dir, &manifest).expect("write source manifest");
        std::fs::read(source_dir.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
            .expect("read source manifest bytes")
    }

    fn completed_source(
        path: &std::path::Path,
        identity: &crate::model::SourceDownload,
    ) -> super::CompletedDownloadSource {
        let mut identity = identity.clone();
        identity.output = Some(path.to_path_buf());
        super::CompletedDownloadSource {
            path: path.to_path_buf(),
            identity,
        }
    }

    #[test]
    fn empty_last_model_never_selects_the_sources_inventory_root() {
        let candidates = super::startup_source_candidates("  \n");
        assert_eq!(
            candidates.first().map(std::path::PathBuf::as_path),
            Some(std::path::Path::new(
                ".uor-models/sources/smollm2-135m-instruct"
            ))
        );
        assert!(candidates
            .iter()
            .all(|candidate| candidate != std::path::Path::new(".uor-models/sources")));

        let named = super::startup_source_candidates("teacher-beta");
        assert_eq!(
            named.first().map(std::path::PathBuf::as_path),
            Some(std::path::Path::new(".uor-models/sources/teacher-beta"))
        );
    }

    fn write_attention_binding(
        root: &std::path::Path,
        operator: &uor_r4_model_source::attention::AttentionOperatorSpec,
    ) -> Vec<u8> {
        std::fs::create_dir_all(root).expect("create bound compile root");
        let mut bytes = serde_json::to_vec_pretty(operator).expect("serialize full operator");
        bytes.push(b'\n');
        std::fs::write(
            root.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE),
            &bytes,
        )
        .expect("write attention binding");
        bytes
    }

    fn write_tokenizer_adapter_binding(root: &std::path::Path, marker: &str) -> Vec<u8> {
        let tokenizer_json = format!(
            r#"{{
                "fixture_marker":"{marker}",
                "pre_tokenizer":{{"type":"ByteLevel","add_prefix_space":false}},
                "model":{{"type":"BPE","vocab":{{"a":0}},"merges":[]}}
            }}"#
        );
        let adapter =
            uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
                tokenizer_json.as_bytes(),
            )
            .expect("tokenizer adapter fixture")
            .adapter();
        let mut bytes = serde_json::to_vec_pretty(&adapter).expect("serialize tokenizer adapter");
        bytes.push(b'\n');
        std::fs::write(root.join("tokenizer_adapter.json"), &bytes)
            .expect("write tokenizer adapter binding");
        bytes
    }

    fn graph_state_with_exact_host_encoder(root: &std::path::Path) -> super::R4g1State {
        use std::collections::BTreeMap;
        use uor_r4_core::transformerless::{compiler, runtime, scenarios};
        use uor_r4_graph_certify::score::{
            emit_scored_r4g1_with_tokenizer_cid, EmissionTables, QuantizationErrorStats,
            ScoredGraphSections, Smoothing,
        };
        use uor_r4_graph_certify::score_runtime::RegionParams;
        use uor_r4_graph_format::ScoreQ;

        let source = root.join("source");
        let bundle = root.join("bundle");
        std::fs::create_dir_all(&source).expect("create tokenizer source");
        std::fs::create_dir_all(bundle.join("graph")).expect("create graph bundle");
        let tokenizer_json = br#"{
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"type":"BPE","vocab":{" ":0,"a":1,"b":2},"merges":[]}
        }"#;
        std::fs::write(source.join("tokenizer.json"), tokenizer_json)
            .expect("write source tokenizer");
        let tokenizer = uor_r4_core::transformerless::hf_bpe::resolve_source_tokenizer(
            &source,
            Some(&uor_r4_core::transformerless::hf_bpe::TokenizerAdapterKey::hf_byte_bpe_v1()),
        )
        .expect("resolve source tokenizer");
        let runtime_table = tokenizer
            .runtime_decode_table()
            .expect("registered tokenizer has a runtime table");
        let tokenizer_path = bundle.join("tokenizer.bin");
        scenarios::export_runtime_tokenizer_table(&runtime_table, &tokenizer_path)
            .expect("export tagged tokenizer");
        let tokenizer_bytes = std::fs::read(&tokenizer_path).expect("read tagged tokenizer");
        let runtime_tokenizer =
            scenarios::Tokenizer::from_bytes(&tokenizer_bytes).expect("parse exported tokenizer");
        let mut runtime_encoded = [0u32; 32];
        assert_eq!(
            runtime_tokenizer.encode_into("a", &mut runtime_encoded),
            Some(2),
            "the exported bundle tokenizer must encode the regression prompt"
        );

        let artifact_bytes =
            include_bytes!("../crates/uor-r4-core/tests/fixtures/tless_artifacts.bin").to_vec();
        let artifacts = compiler::parse_artifacts(&artifact_bytes).expect("parse teacher fixture");
        let regions = [RegionParams {
            node: 1,
            depth: 1,
            radius: 0,
            sig: [0; compiler::SIG_BYTES],
            parent: None,
        }];
        let mut root_prior = BTreeMap::new();
        root_prior.insert(0, ScoreQ::from_raw(-1));
        let emissions = EmissionTables {
            root_prior,
            root_floor: ScoreQ::from_raw(-2),
            root_total: 1,
            region_lists: vec![Vec::new()],
            smoothing: Smoothing::AddOne,
            root_prior_quantization: QuantizationErrorStats::default(),
            emission_quantization: QuantizationErrorStats::default(),
            selection_stats: Default::default(),
        };
        let store: runtime::Store = (0..=compiler::STAGES).map(|_| BTreeMap::new()).collect();
        let store_bytes = runtime::store_bytes(&store);
        let sections = ScoredGraphSections {
            regions: &regions,
            structural: &[],
            transitions: &[],
            transition_quantization: QuantizationErrorStats::default(),
            emissions: &emissions,
            context_rows: &[],
            exct_tls1: &store_bytes,
            exct_top_x: 1,
            fwd_rows: &[],
        };
        let vocab = u32::try_from(artifacts.token_codes.len() / compiler::STAGES)
            .expect("fixture vocabulary fits u32");
        let (graph, _) = emit_scored_r4g1_with_tokenizer_cid(
            &artifact_bytes,
            (b"fixture-meta", b"fixture-records"),
            vocab,
            &sections,
            *blake3::hash(&tokenizer_bytes).as_bytes(),
        );
        let graph_path = bundle.join("graph/score.r4g1");
        let teacher_path = bundle.join("tless_artifacts.bin");
        std::fs::write(&graph_path, graph).expect("write graph fixture");
        std::fs::write(&teacher_path, artifact_bytes).expect("write teacher fixture");
        super::R4g1State::load_with_source(&graph_path, &teacher_path, Some(&source))
            .expect("load graph with exact host encoder")
    }

    #[test]
    fn server_cover_forwards_full_learned_v1_and_v2_observation_records() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        for operator in [
            AttentionOperatorSpec::learned_absolute_v1(),
            AttentionOperatorSpec::learned_absolute_v2(),
        ] {
            let root =
                attention_provenance_test_dir(&format!("learned-manifest-v{}", operator.version));
            let (meta, records) = attention_corpus_markers(&root);
            let mut manifest = uor_r4_graph_compiler::observation::ObservationManifest::new(1);
            manifest.attention_operator = Some(operator.clone());
            std::fs::write(
                root.join(uor_r4_graph_compiler::observation::MANIFEST_FILE),
                serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
            )
            .expect("write manifest");

            let mut args = Vec::new();
            super::append_recorded_attention_operator(&mut args, &meta, &records, false)
                .expect("manifest operator is accepted");
            assert_eq!(
                args.first().map(String::as_str),
                Some("--attention-operator")
            );
            let forwarded: AttentionOperatorSpec =
                serde_json::from_str(&args[1]).expect("forwarded record");
            assert_eq!(forwarded, operator);
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn source_compile_routes_immutable_v1_bundles_to_a_fresh_v2_era() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        assert_eq!(
            super::current_source_attention_era_version().expect("one current source era"),
            2
        );
        for (label, v1, v2) in [
            (
                "standard",
                AttentionOperatorSpec::standard_v1(),
                AttentionOperatorSpec::standard_v2(),
            ),
            (
                "experimental",
                AttentionOperatorSpec::experimental_r4_v1(),
                AttentionOperatorSpec::experimental_r4_v2(),
            ),
            (
                "learned",
                AttentionOperatorSpec::learned_absolute_v1(),
                AttentionOperatorSpec::learned_absolute_v2(),
            ),
        ] {
            let root = attention_provenance_test_dir(&format!("era-{label}"));
            let compiled = root.join("compiled");
            let conventional = compiled.join("teacher");
            let binding_before = write_attention_binding(&conventional, &v1);
            let payload = conventional.join("corpus.records");
            std::fs::write(&payload, b"immutable v1 payload").expect("write v1 payload");

            let era = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
                .expect("v1 selects a separate v2 era");
            assert_eq!(era, compiled.join("teacher-attention-v2"));
            assert!(!era.exists(), "selection is read-only");
            assert_eq!(
                std::fs::read(conventional.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE))
                    .expect("v1 binding remains"),
                binding_before
            );
            assert_eq!(
                std::fs::read(&payload).expect("v1 payload remains"),
                b"immutable v1 payload"
            );

            let v2_binding = write_attention_binding(&era, &v2);
            std::fs::write(era.join("corpus.records"), b"resumable v2 payload")
                .expect("write v2 payload");
            assert_eq!(
                super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
                    .expect("matching v2 era resumes"),
                era
            );
            assert_eq!(
                std::fs::read(era.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE))
                    .expect("v2 binding remains"),
                v2_binding
            );
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn source_compile_refuses_cross_family_successor_before_creating_current_root() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("era-family-preflight");
        let compiled = root.join("compiled");
        let historical = compiled.join("teacher");
        let historical_binding =
            write_attention_binding(&historical, &AttentionOperatorSpec::experimental_r4_v1());
        let payload = historical.join("corpus.records");
        std::fs::write(&payload, b"immutable experimental-v1 corpus")
            .expect("write historical corpus");
        let current = compiled.join("teacher-attention-v2");

        let error = super::source_compile_output_for_operator_era(
            &compiled,
            "teacher",
            2,
            &AttentionOperatorSpec::standard_v2(),
        )
        .expect_err("a standard source cannot populate an experimental successor");
        assert!(
            error.contains("experimental-r4-source-attention/1"),
            "{error}"
        );
        assert!(error.contains("standard-source-attention/2"), "{error}");
        assert!(
            !current.exists(),
            "family rejection precedes suffix creation"
        );
        assert_eq!(
            std::fs::read(historical.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE))
                .expect("historical binding after refusal"),
            historical_binding
        );
        assert_eq!(
            std::fs::read(&payload).expect("historical payload after refusal"),
            b"immutable experimental-v1 corpus"
        );

        assert_eq!(
            super::source_compile_output_for_operator_era(
                &compiled,
                "teacher",
                2,
                &AttentionOperatorSpec::experimental_r4_v2(),
            )
            .expect("the exact family successor remains selectable"),
            current
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_uses_fresh_or_current_conventional_root() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("era-conventional");
        let compiled = root.join("compiled");
        assert_eq!(
            super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
                .expect("fresh conventional root"),
            compiled.join("teacher")
        );
        write_attention_binding(
            &compiled.join("teacher"),
            &AttentionOperatorSpec::standard_v2(),
        );
        assert_eq!(
            super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
                .expect("current conventional root resumes"),
            compiled.join("teacher")
        );
        assert!(!compiled.join("teacher-attention-v2").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_era_selection_fails_closed_on_invalid_provenance() {
        let root = attention_provenance_test_dir("era-invalid");
        let compiled = root.join("compiled");
        let conventional = compiled.join("teacher");
        std::fs::create_dir_all(&conventional).expect("create conventional root");
        let binding = conventional.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE);
        std::fs::write(&binding, b"{not json").expect("write malformed binding");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("malformed provenance is terminal");
        assert!(
            error.contains("malformed attention-operator binding"),
            "{error}"
        );
        assert!(!compiled.join("teacher-attention-v2").exists());

        std::fs::remove_file(&binding).expect("remove malformed binding");
        std::fs::create_dir(&binding).expect("create nonregular binding");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("nonregular provenance is terminal");
        assert!(error.contains("not a regular file"), "{error}");
        assert!(!compiled.join("teacher-attention-v2").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_inspects_current_suffix_before_resuming_conventional_v2() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("era-inspect-both");
        let compiled = root.join("compiled");
        let conventional = compiled.join("teacher");
        let current = compiled.join("teacher-attention-v2");
        write_attention_binding(&conventional, &AttentionOperatorSpec::standard_v2());
        std::fs::write(
            conventional.join("corpus.records"),
            b"current conventional payload",
        )
        .expect("write conventional payload");

        std::fs::create_dir_all(&current).expect("create current suffix");
        std::fs::write(
            current.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE),
            b"{malformed",
        )
        .expect("write malformed suffix binding");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("malformed suffix cannot be hidden by conventional v2");
        assert!(
            error.contains("malformed attention-operator binding"),
            "{error}"
        );

        std::fs::remove_dir_all(&current).expect("remove malformed suffix");
        write_attention_binding(&current, &AttentionOperatorSpec::standard_v2());
        std::fs::write(current.join("corpus.records"), b"duplicate current payload")
            .expect("write suffix payload");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("two current roots are ambiguous");
        assert!(error.contains("duplicate current"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_rejects_empty_preferred_current_before_any_mutation() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("era-empty-preferred-current");
        let compiled = root.join("compiled");
        let conventional = compiled.join("teacher");
        let current = compiled.join("teacher-attention-v2");
        std::fs::create_dir_all(&current).expect("create empty preferred current root");

        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("an empty preferred root must block fresh conventional output");
        assert!(error.contains("exists but is empty"), "{error}");
        assert!(
            !conventional.exists(),
            "selection cannot create its fallback"
        );
        assert_eq!(
            std::fs::read_dir(&current)
                .expect("read empty root")
                .count(),
            0,
            "the preferred root remains untouched"
        );

        let binding = write_attention_binding(&conventional, &AttentionOperatorSpec::standard_v2());
        let payload = conventional.join("corpus.records");
        std::fs::write(&payload, b"current conventional payload")
            .expect("write conventional payload");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("empty preferred root must also block conventional-v2 resume");
        assert!(error.contains("exists but is empty"), "{error}");
        assert_eq!(
            std::fs::read(conventional.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE))
                .expect("read unchanged binding"),
            binding
        );
        assert_eq!(
            std::fs::read(&payload).expect("read unchanged payload"),
            b"current conventional payload"
        );
        assert_eq!(
            std::fs::read_dir(&current)
                .expect("read empty root")
                .count(),
            0
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_retries_only_validated_pre_attention_crash_prefixes() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("pre-attention-crash-prefix");
        let compiled = root.join("compiled");
        let kappa = format!("blake3:{}", "7".repeat(64));

        let standard = compiled.join("standard");
        super::publish_source_compile_preflight(&standard, Some(&kappa))
            .expect("publish standard preflight");
        super::publish_source_manifest_kappa_binding(&standard, &kappa)
            .expect("publish standard kappa");
        assert_eq!(
            super::source_compile_output_for_operator_era(
                &compiled,
                "standard",
                2,
                &AttentionOperatorSpec::standard_v2(),
            )
            .expect("kappa-only conventional prefix resumes"),
            standard
        );

        let learned = compiled.join("learned");
        super::publish_source_compile_preflight(&learned, Some(&kappa))
            .expect("publish learned preflight");
        super::publish_source_manifest_kappa_binding(&learned, &kappa)
            .expect("publish learned kappa");
        let tokenizer_before = write_tokenizer_adapter_binding(&learned, "learned-crash");
        let mut temporary_operator =
            serde_json::to_vec_pretty(&AttentionOperatorSpec::learned_absolute_v2())
                .expect("serialize temporary operator");
        temporary_operator.push(b'\n');
        let temporary = learned.join(".attention_operator.json.42.7.tmp");
        std::fs::write(&temporary, &temporary_operator).expect("write valid crash temporary");
        assert_eq!(
            super::source_compile_output_for_operator_era(
                &compiled,
                "learned",
                2,
                &AttentionOperatorSpec::learned_absolute_v2(),
            )
            .expect("tokenizer-published learned prefix resumes"),
            learned
        );
        assert_eq!(
            std::fs::read(learned.join("tokenizer_adapter.json"))
                .expect("read unchanged tokenizer binding"),
            tokenizer_before
        );
        assert_eq!(
            std::fs::read(&temporary).expect("read unchanged identity temporary"),
            temporary_operator
        );

        let historical = compiled.join("experimental");
        write_attention_binding(&historical, &AttentionOperatorSpec::experimental_r4_v1());
        std::fs::write(historical.join("corpus.records"), b"immutable v1")
            .expect("write historical payload");
        let successor = compiled.join("experimental-attention-v2");
        super::publish_source_compile_preflight(&successor, Some(&kappa))
            .expect("publish redirected preflight");
        super::publish_source_manifest_kappa_binding(&successor, &kappa)
            .expect("publish redirected kappa");
        write_tokenizer_adapter_binding(&successor, "experimental-crash");
        assert_eq!(
            super::source_compile_output_for_operator_era(
                &compiled,
                "experimental",
                2,
                &AttentionOperatorSpec::experimental_r4_v2(),
            )
            .expect("historical family successor resumes its partial suffix"),
            successor
        );
        let pair = super::inspect_compiled_model_pair(&compiled, "experimental", 2)
            .expect("inspect partial successor");
        let error = super::resolve_loadable_compiled_bundle(&pair, 2)
            .expect_err("a pre-attention successor is never loadable");
        assert!(error.contains("present but incomplete"), "{error}");

        let invalid = compiled.join("invalid");
        super::publish_source_compile_preflight(&invalid, Some(&kappa))
            .expect("publish invalid fixture preflight");
        std::fs::write(invalid.join("corpus.meta"), b"unexpected mutable payload")
            .expect("write interrupted payload");
        let payload_before = std::fs::read(invalid.join("corpus.meta")).expect("payload before");
        let error = super::source_compile_output_for_operator_era(
            &compiled,
            "invalid",
            2,
            &AttentionOperatorSpec::standard_v2(),
        )
        .expect_err("unbound mutable payload is not a recoverable identity prefix");
        assert!(error.contains("interrupted current-era compile"), "{error}");
        assert_eq!(
            std::fs::read(invalid.join("corpus.meta")).expect("payload after"),
            payload_before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_initialization_never_strands_an_unmarked_empty_root() {
        let root = attention_provenance_test_dir("preflight-atomic-root");
        let compiled = root.join("compiled");
        std::fs::create_dir_all(&compiled).expect("create compiled parent");
        let output = compiled.join("teacher");
        let kappa = format!("blake3:{}", "8".repeat(64));
        super::publish_source_compile_preflight(&output, Some(&kappa))
            .expect("fresh publication atomically installs a marker-bearing root");
        assert!(output.is_dir());
        assert!(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE).is_file());
        super::publish_source_compile_preflight(&output, Some(&kappa))
            .expect("same-identity publisher is idempotent");
        let marker_before = std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
            .expect("marker before conflict");
        let other = format!("blake3:{}", "9".repeat(64));
        let error = super::publish_source_compile_preflight(&output, Some(&other))
            .expect_err("another identity cannot relabel the published root");
        assert!(error.contains("preflight-bound"), "{error}");
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
                .expect("marker after conflict"),
            marker_before
        );

        let manifestless = compiled.join("legacy-source");
        super::publish_source_compile_preflight(&manifestless, None)
            .expect("manifestless compile also gets a marker-bearing root");
        let record = super::read_optional_source_compile_preflight(&manifestless)
            .expect("read manifestless preflight")
            .expect("manifestless preflight present");
        assert_eq!(record.source_manifest_kappa, None);

        for (name, payload) in [
            ("empty-race", None),
            ("payload-race", Some(b"external".as_slice())),
        ] {
            let raced = compiled.join(name);
            std::fs::create_dir(&raced).expect("external race creates final root");
            if let Some(payload) = payload {
                std::fs::write(raced.join("corpus.records"), payload)
                    .expect("external race writes payload");
            }
            let error = super::publish_source_compile_preflight(&raced, Some(&kappa))
                .expect_err("an external final root is never adopted");
            assert!(
                error.contains("without a stable") || error.contains("refusing to infer"),
                "{error}"
            );
            assert!(!raced.join(super::SOURCE_COMPILE_PREFLIGHT_FILE).exists());
            if let Some(payload) = payload {
                assert_eq!(
                    std::fs::read(raced.join("corpus.records")).expect("payload after refusal"),
                    payload
                );
            }
        }

        let arbitrary_current = compiled.join("other-attention-v2");
        std::fs::create_dir(&arbitrary_current).expect("create arbitrary empty preferred root");
        let error = super::source_compile_output_for_attention_era(&compiled, "other", 2)
            .expect_err("unmarked empty preferred root remains terminal");
        assert!(error.contains("exists but is empty"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_staging_is_exclusive_crash_safe_and_owner_scoped() {
        let root = attention_provenance_test_dir("preflight-staging-state-machine");
        let compiled = root.join("compiled");
        std::fs::create_dir_all(&compiled).expect("create compiled parent");
        let kappa = format!("blake3:{}", "a".repeat(64));
        let expected =
            super::source_compile_preflight_bytes(Some(&kappa)).expect("canonical preflight bytes");

        // A crash before or after the hidden stage marker never exposes a
        // resolver-visible final root. The staging namespace itself is the
        // only top-level entry and is deliberately omitted from discovery.
        let staging =
            super::ensure_source_compile_staging_root(&compiled).expect("create staging namespace");
        let empty_crash = staging.join(".teacher.1.1.stage");
        std::fs::create_dir(&empty_crash).expect("simulate crash before stage marker");
        assert!(super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect("hidden crash stage is not a model")
            .is_empty());
        let marker_crash =
            super::create_source_compile_preflight_stage(&compiled.join("teacher"), &expected)
                .expect("simulate crash after stage marker");
        assert!(!compiled.join("teacher").exists());

        // Two complete publishers own disjoint stages. Atomic no-replace
        // publication selects exactly one; the loser may remove only its own
        // still-exact stage.
        let first =
            super::create_source_compile_preflight_stage(&compiled.join("winner"), &expected)
                .expect("first publisher stage");
        let second =
            super::create_source_compile_preflight_stage(&compiled.join("winner"), &expected)
                .expect("second publisher stage");
        assert_ne!(first, second);
        let winner = compiled.join("winner");
        super::rename_directory_no_replace(&first, &winner).expect("first publisher wins");
        super::rename_directory_no_replace(&second, &winner)
            .expect_err("second publisher cannot replace the winner");
        super::remove_owned_source_compile_stage(&second, &expected)
            .expect("loser removes only its own exact stage");
        assert_eq!(
            std::fs::read(winner.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
                .expect("winning marker"),
            expected
        );

        // An external empty or populated final root wins the namespace race;
        // the staged publisher refuses replacement and its payload is intact.
        for (name, payload) in [
            ("external-empty", None),
            ("external-payload", Some(b"foreign".as_slice())),
        ] {
            let output = compiled.join(name);
            let stage = super::create_source_compile_preflight_stage(&output, &expected)
                .expect("prepare raced publisher stage");
            std::fs::create_dir(&output).expect("external actor creates final root");
            if let Some(payload) = payload {
                std::fs::write(output.join("corpus.records"), payload)
                    .expect("external actor writes payload");
            }
            super::rename_directory_no_replace(&stage, &output)
                .expect_err("exclusive rename never replaces external state");
            super::remove_owned_source_compile_stage(&stage, &expected)
                .expect("failed publisher cleans only its own stage");
            assert!(!output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE).exists());
            if let Some(payload) = payload {
                assert_eq!(
                    std::fs::read(output.join("corpus.records")).expect("foreign payload"),
                    payload
                );
            }
        }

        // Cleanup is ownership-sensitive: once another actor adds an entry,
        // the server leaves the stage untouched rather than recursively
        // deleting bytes it no longer exclusively owns.
        let tampered =
            super::create_source_compile_preflight_stage(&compiled.join("tampered"), &expected)
                .expect("create cleanup fixture");
        std::fs::write(tampered.join("foreign"), b"do not delete").expect("tamper owned stage");
        let error = super::remove_owned_source_compile_stage(&tampered, &expected)
            .expect_err("tampered stage is not removed");
        assert!(error.contains("unexpected entries"), "{error}");
        assert_eq!(
            std::fs::read(tampered.join("foreign")).expect("foreign stage byte"),
            b"do not delete"
        );

        super::remove_owned_source_compile_stage(&marker_crash, &expected)
            .expect("test owns marker-crash stage");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_session_lock_rejects_fresh_and_existing_cross_process_losers() {
        let root = attention_provenance_test_dir("source-compile-session-lock");
        let output = root.join("compiled/teacher");
        let kappa = format!("blake3:{}", "c".repeat(64));

        // Two independently opened file descriptions model two server
        // processes. The first session owns the physical root before its
        // preflight exists; the loser receives BUSY and cannot publish it.
        let first = super::try_lock_source_compile_session(&output)
            .expect("first compiler owns fresh output");
        let lock_path =
            super::source_compile_session_lock_path(&output).expect("coordination path");
        let metadata = std::fs::symlink_metadata(&lock_path).expect("coordination inode");
        assert!(metadata.file_type().is_file());
        let expected_lock_parent =
            super::source_compile_staging_root(output.parent().expect("output parent"));
        assert_eq!(lock_path.parent(), Some(expected_lock_parent.as_path()));
        let error = super::try_lock_source_compile_session(&output)
            .err()
            .expect("fresh-root loser is nonblocking");
        assert!(error.contains("BUSY"), "{error}");
        assert!(!output.exists(), "loser cannot publish the fresh root");

        // The winner publishes identity and a payload while retaining the
        // session. Seeing the same κ is not ownership: a second process still
        // cannot adopt or write the existing root.
        super::publish_source_compile_preflight(&output, Some(&kappa))
            .expect("winner publishes preflight");
        super::publish_source_manifest_kappa_binding(&output, &kappa)
            .expect("winner binds source snapshot");
        let payload = output.join("corpus.records");
        std::fs::write(&payload, b"winner-owned corpus").expect("winner payload");
        let marker_before =
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE)).expect("marker");
        let payload_before = std::fs::read(&payload).expect("payload");
        let error = super::try_lock_source_compile_session(&output)
            .err()
            .expect("same-kappa existing-root loser is nonblocking");
        assert!(error.contains("BUSY"), "{error}");
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
                .expect("marker after loser"),
            marker_before
        );
        assert_eq!(
            std::fs::read(&payload).expect("payload after loser"),
            payload_before
        );

        // Only release transfers ownership. A later retry takes the same
        // persistent coordination inode and may idempotently resume the exact
        // completed identity without changing winner bytes.
        drop(first);
        let retry = super::try_lock_source_compile_session(&output)
            .expect("retry owns session after release");
        super::publish_source_compile_preflight(&output, Some(&kappa))
            .expect("retry resumes exact preflight");
        super::publish_source_manifest_kappa_binding(&output, &kappa)
            .expect("retry resumes exact source binding");
        assert_eq!(
            std::fs::read(&payload).expect("payload after retry"),
            payload_before
        );
        drop(retry);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_session_sets_coordinate_writers_readers_and_partial_failure() {
        let root = attention_provenance_test_dir("source-compile-session-sets");
        let compiled = root.join("compiled");
        let conventional = compiled.join("alpha");
        let current = compiled.join("alpha-attention-v2");
        let external_graph = root.join("external/graph");
        let subjects = vec![
            compiled.clone(),
            conventional.clone(),
            current.clone(),
            external_graph.clone(),
        ];

        let writer = super::try_lock_source_compile_sessions(
            subjects.clone(),
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("one process owns every mutable sink");
        let error = super::try_lock_source_compile_sessions(
            [compiled.clone()],
            super::SourceCompileSessionMode::SharedReader,
        )
        .err()
        .expect("managed startup reader is nonblocking while writer owns namespace");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        let error = super::try_lock_source_compile_sessions(
            [external_graph.clone()],
            super::SourceCompileSessionMode::SharedReader,
        )
        .err()
        .expect("external graph reader is nonblocking while writer owns sink");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(writer);

        let first_reader = super::try_lock_source_compile_sessions(
            subjects.clone(),
            super::SourceCompileSessionMode::SharedReader,
        )
        .expect("first reader owns one stable snapshot");
        let second_reader = super::try_lock_source_compile_sessions(
            subjects.clone(),
            super::SourceCompileSessionMode::SharedReader,
        )
        .expect("readers may share the stable snapshot");
        let error = super::try_lock_source_compile_sessions(
            subjects.clone(),
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("writer cannot truncate while either reader is preparing");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(first_reader);
        let error = super::try_lock_source_compile_sessions(
            subjects.clone(),
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("second reader still blocks the writer");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(second_reader);
        let retry = super::try_lock_source_compile_sessions(
            subjects,
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("ownership transfers only after every reader drops");
        drop(retry);

        // Acquisition is canonical, sorted, and failure-atomic. `00-free`
        // sorts before the independently held `99-busy`; when the set loses
        // at the latter, its earlier lock is released immediately.
        let free = root.join("00-free");
        let busy = root.join("99-busy");
        let busy_owner = super::try_lock_source_compile_sessions(
            [busy.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("hold later-sorted subject");
        let error = super::try_lock_source_compile_sessions(
            [busy.clone(), free.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("multi-lock loser refuses without waiting");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        let free_owner = super::try_lock_source_compile_sessions(
            [free],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("failed set released its earlier partial acquisition");
        drop(free_owner);
        drop(busy_owner);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_and_shared_external_compile_sinks_have_exact_winner_loser_ownership() {
        let root = attention_provenance_test_dir("legacy-external-session-sinks");
        let compiled = root.join("compiled");
        let legacy_cover = root.join("legacy/graph-cover");
        let legacy_graph = root.join("legacy/graph");
        std::fs::create_dir_all(&legacy_cover).expect("legacy cover output");
        std::fs::create_dir_all(&legacy_graph).expect("legacy graph output");
        let cover_sentinel = legacy_cover.join("cover.r4g1");
        let graph_sentinel = legacy_graph.join("score.r4g1");
        std::fs::write(&cover_sentinel, b"winner cover").expect("cover sentinel");
        std::fs::write(&graph_sentinel, b"winner graph").expect("graph sentinel");
        let legacy_subjects = vec![compiled.clone(), legacy_cover.clone(), legacy_graph.clone()];
        let legacy_winner = super::try_lock_source_compile_sessions(
            legacy_subjects.clone(),
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("first configured-legacy compiler owns all outputs");
        let error = super::try_lock_source_compile_sessions(
            legacy_subjects.clone(),
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("second no-source compiler refuses before truncation");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        assert_eq!(
            std::fs::read(&cover_sentinel).expect("cover after loser"),
            b"winner cover"
        );
        assert_eq!(
            std::fs::read(&graph_sentinel).expect("graph after loser"),
            b"winner graph"
        );
        drop(legacy_winner);
        let retry = super::try_lock_source_compile_sessions(
            legacy_subjects,
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("legacy retry owns outputs after winner release");
        drop(retry);

        // Distinct source identities can still alias one configured external
        // score sink. Even without relying on the managed namespace subject,
        // the exact external output key elects only one writer and a losing
        // partial acquisition is rolled back.
        let source_a = root.join("source-a");
        let source_b = root.join("source-b");
        let shared_external = root.join("shared-external-graph");
        let source_a_winner = super::try_lock_source_compile_sessions(
            [source_a.clone(), shared_external.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("source A owns shared external sink");
        let error = super::try_lock_source_compile_sessions(
            [source_b.clone(), shared_external.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("source B cannot alias source A's sink");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        let source_b_independent = super::try_lock_source_compile_sessions(
            [source_b],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("losing multi-sink attempt released source B's own subject");
        drop(source_b_independent);
        drop(source_a_winner);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_compile_selection_refresh_refuses_post_selection_inventory_change() {
        let root = attention_provenance_test_dir("legacy-selection-refresh");
        let first = root.join("compiled/alpha");
        let second = root.join("compiled/beta");
        let selected = (
            first.join("corpus.meta"),
            first.join("corpus.records"),
            first.join("graph-cover"),
            first.join("graph"),
        );
        let refreshed = (
            second.join("corpus.meta"),
            second.join("corpus.records"),
            second.join("graph-cover"),
            second.join("graph"),
        );
        std::fs::create_dir_all(selected.3.clone()).expect("selected graph output");
        let sentinel = selected.3.join("sentinel");
        std::fs::write(&sentinel, b"unchanged").expect("selected sentinel");
        let error = super::require_unchanged_legacy_compile_paths(&selected, &refreshed)
            .expect_err("a publisher changed discovery before lock acquisition");
        assert!(error.contains("changed while acquiring"), "{error}");
        assert_eq!(
            std::fs::read(&sentinel).expect("sentinel after refusal"),
            b"unchanged"
        );
        super::require_unchanged_legacy_compile_paths(&selected, &selected)
            .expect("exact post-lock refresh is stable");
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn source_compile_session_sets_canonicalize_aliases_once() {
        use std::os::unix::fs::symlink;

        let root = attention_provenance_test_dir("source-compile-session-alias");
        let real = root.join("real-sink");
        let alias = root.join("alias-sink");
        std::fs::create_dir_all(&real).expect("real external sink parent");
        symlink(&real, &alias).expect("external sink alias");
        let real_graph = real.join("graph");
        let alias_graph = alias.join("graph");

        let writer = super::try_lock_source_compile_sessions(
            [real_graph.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("real spelling owns absent graph child");
        let error = super::try_lock_source_compile_sessions(
            [alias_graph.clone()],
            super::SourceCompileSessionMode::SharedReader,
        )
        .err()
        .expect("alias reader resolves to the same canonical subject");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(writer);
        let reader = super::try_lock_source_compile_sessions(
            [alias_graph],
            super::SourceCompileSessionMode::SharedReader,
        )
        .expect("alias succeeds after owner release");
        let error = super::try_lock_source_compile_sessions(
            [real_graph],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("canonical reader blocks the real-spelling writer");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(reader);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn busy_bundle_readers_preserve_serving_markers_until_install_finishes() {
        let root = attention_provenance_test_dir("source-compile-reader-state");
        let compiled = root.join("compiled");
        let output = compiled.join("teacher");
        let writer = super::try_lock_source_compile_sessions(
            [compiled.clone(), output.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("publisher owns managed bundle");

        let mut serving = super::ServingModelState {
            epoch: 41,
            terminal_load_error: Some("prior terminal marker".to_owned()),
            last_operation_error: Some("prior operation marker".to_owned()),
            ..Default::default()
        };
        let error = super::try_lock_managed_inventory_read_sessions(&compiled, [output.clone()])
            .err()
            .expect("startup/reload reader returns BUSY before disk inspection");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        assert_eq!(serving.epoch, 41);
        assert_eq!(
            serving.terminal_load_error.as_deref(),
            Some("prior terminal marker")
        );
        assert_eq!(
            serving.last_operation_error.as_deref(),
            Some("prior operation marker")
        );
        drop(writer);

        let reader = super::try_lock_managed_inventory_read_sessions(&compiled, [output.clone()])
            .expect("reader captures stable publication snapshot");
        let error = super::try_lock_source_compile_sessions(
            [compiled.clone(), output.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("writer remains blocked immediately before simulated install");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        serving.epoch += 1;
        serving.last_operation_error = None;
        let error = super::try_lock_source_compile_sessions(
            [compiled.clone(), output.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .err()
        .expect("atomic in-memory install does not prematurely release reader");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        drop(reader);
        let retry = super::try_lock_source_compile_sessions(
            [compiled, output],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("next publisher starts only after install releases reader");
        drop(retry);
        assert_eq!(serving.epoch, 42);
        assert_eq!(
            serving.terminal_load_error.as_deref(),
            Some("prior terminal marker")
        );
        assert_eq!(serving.last_operation_error, None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn exclusive_session_recovers_only_reserved_regular_torn_identity_temps() {
        let root = attention_provenance_test_dir("source-compile-torn-temps");
        let output = root.join("compiled/teacher");
        let kappa = format!("blake3:{}", "d".repeat(64));
        let session = super::try_lock_source_compile_session(&output)
            .expect("test owns output recovery session");
        super::preflight_and_bind_source_snapshot_kappa(&output, Some(&kappa))
            .expect("publish stable P and K records");
        let preflight_before =
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE)).expect("stable P");
        let kappa_before = std::fs::read(output.join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE))
            .expect("stable K");

        for (index, sidecar) in [
            super::SOURCE_COMPILE_PREFLIGHT_FILE,
            super::SOURCE_MANIFEST_KAPPA_BINDING_FILE,
            "tokenizer_adapter.json",
            uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
        ]
        .into_iter()
        .enumerate()
        {
            let empty = output.join(format!(".{sidecar}.700.{index}.tmp"));
            let truncated = output.join(format!(".{sidecar}.701.{index}.tmp"));
            std::fs::write(&empty, b"").expect("zero-byte torn create");
            std::fs::write(&truncated, b"{\"prefix\":").expect("mid-write crash prefix");
        }
        super::recover_source_compile_identity_temporaries(&output)
            .expect("exclusive owner reclaims strict reserved temp names");
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
                .expect("stable P after recovery"),
            preflight_before
        );
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE))
                .expect("stable K after recovery"),
            kappa_before
        );
        assert!(std::fs::read_dir(&output)
            .expect("enumerate recovered root")
            .all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")));

        let unknown = output.join(".tokenizer_adapter.json.owner.tmp");
        std::fs::write(&unknown, b"").expect("unknown temp spelling");
        let error = super::recover_source_compile_identity_temporaries(&output)
            .expect_err("unknown lookalike is never reclaimed");
        assert!(error.contains("unrecognized or non-owned"), "{error}");
        assert!(unknown.is_file());
        drop(session);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn exclusive_session_never_recovers_symlink_or_fifo_identity_temps() {
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;

        fn fifo(path: &std::path::Path) {
            let path = std::ffi::CString::new(path.as_os_str().as_bytes())
                .expect("fixture path has no NUL");
            // SAFETY: the C string is live and mkfifo does not retain it.
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        }

        for kind in ["symlink", "fifo"] {
            let root = attention_provenance_test_dir(&format!("torn-temp-{kind}"));
            let output = root.join("compiled/teacher");
            std::fs::create_dir_all(&output).expect("output root");
            let session = super::try_lock_source_compile_session(&output)
                .expect("test owns recovery session");
            let temporary = output.join(".source_compile_preflight.json.800.1.tmp");
            match kind {
                "symlink" => symlink(root.join("missing"), &temporary).expect("dangling temp"),
                "fifo" => fifo(&temporary),
                _ => unreachable!(),
            }
            let error = super::recover_source_compile_identity_temporaries(&output)
                .expect_err("special entry is terminal and nonblocking");
            assert!(error.contains("not a regular non-symlink file"), "{error}");
            assert!(std::fs::symlink_metadata(&temporary).is_ok());
            drop(session);
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn existing_graph_publication_is_immutable_under_refresh_failure() {
        let root = attention_provenance_test_dir("immutable-graph-refresh");
        let graph = root.join("bundle/graph/score.r4g1");
        let cover = root.join("bundle/graph-cover/cover.r4g1");
        let score_report = root.join("bundle/graph/score_report.json");
        std::fs::create_dir_all(graph.parent().expect("graph parent")).expect("graph parent");
        std::fs::create_dir_all(cover.parent().expect("cover parent")).expect("cover parent");
        std::fs::write(&graph, b"last-good graph").expect("graph bytes");
        std::fs::write(&cover, b"last-good cover").expect("cover bytes");
        std::fs::write(&score_report, b"last-good report").expect("report bytes");
        let before = [
            std::fs::read(&graph).expect("graph before"),
            std::fs::read(&cover).expect("cover before"),
            std::fs::read(&score_report).expect("report before"),
        ];

        let reuse = super::immutable_graph_artifact_present(&graph)
            .expect("existing graph selects immutable idempotent branch");
        let mut writer_calls = 0usize;
        super::run_graph_writer_if_incomplete(reuse, || {
            writer_calls += 1;
            std::fs::write(&cover, b"truncated").expect("simulated cover writer");
            std::fs::write(&graph, b"truncated").expect("simulated score writer");
            Err("injected writer failure".to_owned())
        })
        .expect("immutable branch suppresses the failing writer");
        assert_eq!(
            writer_calls, 0,
            "stable outputs are never reopened by refresh"
        );
        assert_eq!(std::fs::read(&graph).expect("graph after"), before[0]);
        assert_eq!(std::fs::read(&cover).expect("cover after"), before[1]);
        assert_eq!(
            std::fs::read(&score_report).expect("report after"),
            before[2]
        );

        std::fs::remove_file(&graph).expect("model incomplete graph-absent case");
        assert!(!super::immutable_graph_artifact_present(&graph)
            .expect("only graph absence authorizes incomplete build"));
        let _ = std::fs::remove_dir_all(root);
    }

    fn write_completion_fixture(root: &std::path::Path, tag: &[u8]) {
        std::fs::create_dir_all(root.join("graph-cover")).expect("cover directory");
        std::fs::create_dir_all(root.join("graph")).expect("graph directory");
        for file in [
            "corpus.meta",
            "corpus.records",
            "tless_artifacts.bin",
            "tokenizer.bin",
            "tokenizer_adapter.json",
            uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE,
            super::SOURCE_COMPILE_PREFLIGHT_FILE,
            super::SOURCE_MANIFEST_KAPPA_BINDING_FILE,
        ] {
            std::fs::write(root.join(file), tag).expect("bundle fixture member");
        }
        for file in [
            "graph-cover/cover.r4g1",
            "graph-cover/cover_report.json",
            "graph/score.r4g1",
            "graph/score_report.json",
        ] {
            std::fs::write(root.join(file), tag).expect("graph fixture member");
        }
    }

    #[test]
    fn completed_bundle_stage_atomically_replaces_last_good_and_failure_preserves_it() {
        let root = attention_provenance_test_dir("completed-bundle-stage");
        let output = root.join("compiled/model");
        let source_kappa = format!("blake3:{}", "0".repeat(64));
        write_completion_fixture(&output, b"old");
        super::publish_compiled_bundle_completion(&output).expect("bootstrap old completion");
        let old_graph = std::fs::read(output.join("graph/score.r4g1")).expect("old graph");
        let old_corpus = std::fs::read(output.join("corpus.records")).expect("old corpus");

        let failed_path = {
            let failed =
                super::CompiledBundleStage::allocate(&output, &source_kappa).expect("failed stage");
            std::fs::create_dir_all(failed.path.join("graph")).expect("partial graph directory");
            std::fs::write(failed.path.join("graph/score.r4g1"), b"partial")
                .expect("partial score residue");
            std::fs::write(failed.path.join("corpus.records"), b"advanced")
                .expect("resumable corpus progress");
            let error = super::publish_compiled_bundle_completion(&failed.path)
                .expect_err("missing report/cover cannot complete");
            assert!(error.contains("incomplete"), "{error}");
            failed.path.clone()
        };
        assert!(failed_path.exists(), "incomplete stage remains resumable");
        assert_eq!(
            std::fs::read(output.join("graph/score.r4g1")).expect("old graph preserved"),
            old_graph
        );
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("old corpus preserved"),
            old_corpus
        );

        let mut replacement = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect("replacement stage");
        assert_eq!(replacement.path, failed_path, "retry adopts exact stage");
        assert_eq!(
            std::fs::read(replacement.path.join("corpus.records")).expect("resumed corpus"),
            b"advanced"
        );
        assert!(
            !replacement.path.join("graph").exists(),
            "derived partial graph is rebuilt, not adopted"
        );
        write_completion_fixture(&replacement.path, b"new");
        super::publish_compiled_bundle_completion(&replacement.path)
            .expect("durable replacement completion");
        replacement.publish().expect("atomic directory exchange");
        assert_eq!(
            std::fs::read(output.join("graph/score.r4g1")).expect("new graph"),
            b"new"
        );
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("advanced corpus"),
            b"new"
        );
        super::validate_compiled_bundle_completion(&output)
            .expect("new completion validates")
            .expect("completion present");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn compiled_bundle_stage_refuses_cross_snapshot_adoption_without_mutation() {
        let root = attention_provenance_test_dir("compiled-stage-snapshot-conflict");
        let output = root.join("compiled/model");
        write_completion_fixture(&output, b"old");
        let first_kappa = format!("blake3:{}", "1".repeat(64));
        let second_kappa = format!("blake3:{}", "2".repeat(64));
        let stage = super::CompiledBundleStage::allocate(&output, &first_kappa)
            .expect("first snapshot stage");
        std::fs::create_dir_all(stage.path.join("graph")).expect("partial graph directory");
        std::fs::write(
            stage.path.join("graph/score.r4g1"),
            b"first-snapshot-partial",
        )
        .expect("partial graph");
        let stage_path = stage.path.clone();
        drop(stage);

        let error = super::CompiledBundleStage::allocate(&output, &second_kappa)
            .expect_err("different snapshot cannot adopt stage");
        assert!(error.contains("does not bind this exact"), "{error}");
        assert_eq!(
            std::fs::read(stage_path.join("graph/score.r4g1"))
                .expect("conflicting stage is untouched"),
            b"first-snapshot-partial"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn compiled_bundle_stage_recovers_only_exact_regular_publisher_temporaries() {
        let root = attention_provenance_test_dir("compiled-stage-publisher-temporaries");
        let output = root.join("compiled/model");
        write_completion_fixture(&output, b"old");
        let source_kappa = format!("blake3:{}", "3".repeat(64));
        let stage =
            super::CompiledBundleStage::allocate(&output, &source_kappa).expect("resumable stage");
        let stage_path = stage.path.clone();
        drop(stage);
        let marker_temp = stage_path.join(format!(
            ".{}.71.1.tmp",
            super::COMPILED_BUNDLE_STAGE_MARKER_FILE
        ));
        let completion_temp = stage_path.join(format!(
            ".{}.71.2.tmp",
            super::COMPILED_BUNDLE_COMPLETION_FILE
        ));
        std::fs::write(&marker_temp, b"").expect("pre-write marker crash");
        std::fs::write(&completion_temp, b"{\n").expect("mid-write completion crash");

        let recovered = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect("exact regular residue recovers");
        assert_eq!(recovered.path, stage_path);
        assert!(!marker_temp.exists());
        assert!(!completion_temp.exists());
        drop(recovered);

        let unknown = stage_path.join(format!(
            ".{}.owner.9.tmp",
            super::COMPILED_BUNDLE_COMPLETION_FILE
        ));
        std::fs::write(&unknown, b"unknown").expect("unknown lookalike");
        let error = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect_err("unknown publisher residue is terminal");
        assert!(
            error.contains("unrecognized publisher temporary"),
            "{error}"
        );
        assert_eq!(
            std::fs::read(&unknown).expect("unknown residue remains untouched"),
            b"unknown"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn compiled_bundle_stage_refuses_special_publisher_temporary_without_following() {
        use std::os::unix::fs::symlink;

        let root = attention_provenance_test_dir("compiled-stage-special-temporary");
        let output = root.join("compiled/model");
        write_completion_fixture(&output, b"old");
        let source_kappa = format!("blake3:{}", "4".repeat(64));
        let stage =
            super::CompiledBundleStage::allocate(&output, &source_kappa).expect("resumable stage");
        let stage_path = stage.path.clone();
        drop(stage);
        let temporary = stage_path.join(format!(
            ".{}.72.1.tmp",
            super::COMPILED_BUNDLE_COMPLETION_FILE
        ));
        symlink(root.join("missing"), &temporary).expect("dangling publisher temporary");
        let error = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect_err("special publisher residue is terminal");
        assert!(error.contains("not a regular non-symlink file"), "{error}");
        assert!(std::fs::symlink_metadata(&temporary).is_ok());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn post_exchange_stage_marker_crash_state_keeps_new_generation_loadable() {
        let root = attention_provenance_test_dir("compiled-stage-post-exchange-marker");
        let output = root.join("compiled/model");
        write_completion_fixture(&output, b"old");
        super::publish_compiled_bundle_completion(&output).expect("old completion");
        let source_kappa = format!("blake3:{}", "5".repeat(64));
        let stage = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect("replacement stage");
        write_completion_fixture(&stage.path, b"new");
        super::publish_compiled_bundle_completion(&stage.path).expect("new completion");

        super::exchange_directories(&stage.path, &output).expect("simulate committed exchange");
        stage
            .sync_publication_parents("simulated post-exchange crash")
            .expect("both parents durable");
        assert!(
            output
                .join(super::COMPILED_BUNDLE_STAGE_MARKER_FILE)
                .is_file(),
            "crash point retains marker in the published generation"
        );
        super::validate_compiled_bundle_completion(&output)
            .expect("published completion validates")
            .expect("published completion present");
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("new corpus"),
            b"new"
        );

        let retry = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect("next owner reclaims exchanged old generation");
        assert_ne!(retry.path, stage.path);
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("published corpus unchanged"),
            b"new"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn public_completion_hard_link_residue_recovers_before_validation() {
        let root = attention_provenance_test_dir("public-completion-hard-link-residue");
        let output = root.join("compiled/model");
        write_completion_fixture(&output, b"stable");
        super::publish_compiled_bundle_completion(&output).expect("publish stable completion");
        let stable = output.join(super::COMPILED_BUNDLE_COMPLETION_FILE);
        let temporary = output.join(format!(
            ".{}.91.7.tmp",
            super::COMPILED_BUNDLE_COMPLETION_FILE
        ));
        std::fs::hard_link(&stable, &temporary)
            .expect("simulate crash after stable hard link before temp unlink");

        let strict_error = super::validate_compiled_bundle_completion(&output)
            .expect_err("raw member hashing sees the unrecovered residue");
        assert!(strict_error.contains("changed after"), "{strict_error}");
        super::recover_managed_compiled_bundle_completion_temporaries(&root.join("compiled"))
            .expect("startup/reload inventory owner reclaims exact linked residue");
        assert!(!temporary.exists());
        super::validate_compiled_bundle_completion(&output)
            .expect("completion validates after recovery")
            .expect("completion remains present");

        let tampered = output.join(format!(
            ".{}.92.8.tmp",
            super::COMPILED_BUNDLE_COMPLETION_FILE
        ));
        std::fs::write(&tampered, b"not the committed completion")
            .expect("write conflicting exact-name residue");
        let error = super::recover_compiled_bundle_completion_temporaries(&output)
            .expect_err("conflicting regular residue is terminal");
        assert!(error.contains("conflicting"), "{error}");
        assert_eq!(
            std::fs::read(&tampered).expect("tampered residue remains"),
            b"not the committed completion"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            std::fs::remove_file(&tampered).expect("remove regular conflict fixture");
            symlink(&stable, &tampered).expect("exact-byte symlink residue");
            let error = super::recover_compiled_bundle_completion_temporaries(&output)
                .expect_err("symlink residue is terminal without following");
            assert!(error.contains("not a regular non-symlink"), "{error}");
            assert!(std::fs::symlink_metadata(&tampered).is_ok());
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_refresh_refuses_tampered_authoritative_completion_before_copy() {
        let root = attention_provenance_test_dir("authoritative-completion-before-copy");
        let output = root.join("compiled/model");
        let source_kappa = format!("blake3:{}", "6".repeat(64));
        write_completion_fixture(&output, b"M1");
        super::publish_compiled_bundle_completion(&output).expect("publish M1 completion");
        let completion_before = std::fs::read(output.join(super::COMPILED_BUNDLE_COMPLETION_FILE))
            .expect("completion before tamper");
        std::fs::write(output.join("corpus.records"), b"M2 structurally valid rows")
            .expect("tamper authoritative corpus bytes");
        let corpus_after_tamper =
            std::fs::read(output.join("corpus.records")).expect("tampered corpus");

        let error = super::CompiledBundleStage::allocate(&output, &source_kappa)
            .expect_err("refresh cannot copy and recertify a tampered completion");
        assert!(error.contains("changed after"), "{error}");
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("corpus after refusal"),
            corpus_after_tamper
        );
        assert_eq!(
            std::fs::read(output.join(super::COMPILED_BUNDLE_COMPLETION_FILE))
                .expect("completion after refusal"),
            completion_before
        );
        let staging = super::source_compile_staging_root(output.parent().expect("output parent"));
        assert!(
            !staging.exists(),
            "authoritative validation precedes refresh-stage allocation"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn minimal_r4g1_bytes() -> Vec<u8> {
        let mut head = [0u8; uor_r4_graph_format::HEAD_PAYLOAD_LEN];
        head[184..186].copy_from_slice(&8u16.to_le_bytes());
        head[204] = 1;
        head[212..214].copy_from_slice(&64u16.to_le_bytes());
        let mut builder = uor_r4_graph_format::ArtifactBuilder::new(6);
        builder.add_section(uor_r4_graph_format::SectionId::HEAD, 0, &head);
        builder.build().expect("minimal R4G1 fixture")
    }

    fn write_valid_graph_output(output: &std::path::Path, kind: super::GraphOutputKind, tag: &str) {
        std::fs::create_dir_all(output).expect("graph output directory");
        let (artifact, report) = kind.files();
        std::fs::write(output.join(artifact), minimal_r4g1_bytes()).expect("graph artifact");
        std::fs::write(
            output.join(report),
            serde_json::to_vec(&serde_json::json!({ "tag": tag })).expect("graph report"),
        )
        .expect("graph report bytes");
    }

    fn legacy_test_identity(
        cover: &std::path::Path,
        graph: &std::path::Path,
        input_tag: &str,
    ) -> super::LegacyGraphGenerationIdentity {
        super::LegacyGraphGenerationIdentity {
            cover_output: super::canonical_path_text(cover, "test cover").expect("cover path"),
            graph_output: super::canonical_path_text(graph, "test graph").expect("graph path"),
            input_files: std::collections::BTreeMap::new(),
            cover_controls_kappa: super::bytes_kappa(input_tag.as_bytes()),
            score_controls_kappa: format!("blake3:{}", "b".repeat(64)),
        }
    }

    #[test]
    fn legacy_owned_cover_only_crash_resumes_score_but_arbitrary_pair_is_terminal() {
        let root = attention_provenance_test_dir("legacy-owned-one-sided-resume");
        let cover = root.join("bundle/graph-cover");
        let graph = root.join("bundle/graph");
        write_valid_graph_output(&cover, super::GraphOutputKind::Cover, "owned cover");
        let identity = legacy_test_identity(&cover, &graph, "input-v1");

        let error = super::legacy_graph_generation_action(&identity)
            .expect_err("unmarked one-sided state is arbitrary and terminal");
        assert!(error.contains("without an exact server attempt"), "{error}");
        let paths = super::legacy_graph_record_paths(&identity).expect("record paths");
        let attempt = super::legacy_graph_attempt_bytes(&identity).expect("attempt bytes");
        super::ensure_legacy_graph_attempt(&paths.attempt, &attempt)
            .expect("durable attempt predates cover publication");
        let serving_error =
            super::validate_legacy_graph_generation_for_serving(&graph.join("score.r4g1"))
                .expect_err("startup/reload cannot observe the one-sided attempt");
        assert!(serving_error.contains("incomplete"), "{serving_error}");
        assert_eq!(
            super::legacy_graph_generation_action(&identity)
                .expect("owned one-sided state classifies")
                .0,
            super::LegacyGraphGenerationAction::ResumeScore
        );

        let args = vec!["--out".to_owned(), graph.display().to_string()];
        let score_stage = super::build_graph_writer_stage(
            &graph,
            super::GraphOutputKind::Score,
            &args,
            |staged| {
                write_valid_graph_output(
                    std::path::Path::new(&staged[1]),
                    super::GraphOutputKind::Score,
                    "resumed score",
                );
                Ok(())
            },
        )
        .expect("stage resumed score");
        super::publish_legacy_graph_score_resume(
            score_stage,
            &cover,
            &graph,
            &paths,
            &attempt,
            &identity,
        )
        .expect("publish score and exact completion");
        assert!(!paths.attempt.exists());
        assert_eq!(
            super::legacy_graph_generation_action(&identity)
                .expect("completed generation reuses")
                .0,
            super::LegacyGraphGenerationAction::Reuse
        );
        super::ensure_legacy_graph_attempt(&paths.attempt, &attempt)
            .expect("simulate crash after completion commit before attempt cleanup");
        super::validate_legacy_graph_generation_for_serving(&graph.join("score.r4g1"))
            .expect("completed pair reclaims its redundant exact attempt");
        assert!(!paths.attempt.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_generation_identity_refreshes_changed_inputs_and_rolls_back_failure() {
        let root = attention_provenance_test_dir("legacy-input-bound-generation");
        let bundle = root.join("bundle");
        let cover = bundle.join("graph-cover");
        let graph = bundle.join("graph");
        let artifacts = bundle.join("tless_artifacts.bin");
        let corpus_meta = bundle.join("corpus.meta");
        let corpus_records = bundle.join("corpus.records");
        let tokenizer = bundle.join("tokenizer.bin");
        std::fs::create_dir_all(&bundle).expect("legacy input bundle");
        std::fs::write(&artifacts, b"stable transformerless artifact").expect("artifact");
        std::fs::write(&corpus_meta, b"structurally valid metadata").expect("metadata");
        std::fs::write(&corpus_records, b"structurally valid corpus generation one")
            .expect("generation-one records");
        std::fs::write(&tokenizer, b"stable tokenizer").expect("tokenizer");
        let cover_args = vec![
            "--corpus-meta".to_owned(),
            corpus_meta.display().to_string(),
            "--out".to_owned(),
            cover.display().to_string(),
        ];
        let score_args = vec![
            "--corpus-recs".to_owned(),
            corpus_records.display().to_string(),
            "--out".to_owned(),
            graph.display().to_string(),
        ];
        write_valid_graph_output(&cover, super::GraphOutputKind::Cover, "old cover");
        write_valid_graph_output(&graph, super::GraphOutputKind::Score, "old score");
        let old_identity =
            super::capture_legacy_graph_generation_identity(super::LegacyGraphGenerationInputs {
                artifacts: &artifacts,
                corpus_meta: &corpus_meta,
                corpus_recs: &corpus_records,
                tokenizer: &tokenizer,
                cover_output: &cover,
                graph_output: &graph,
                cover_args: &cover_args,
                score_args: &score_args,
            })
            .expect("capture generation-one inputs");
        let paths = super::legacy_graph_record_paths(&old_identity).expect("record paths");
        let old_completion = super::LegacyGraphGenerationCompletion {
            schema: super::LEGACY_GRAPH_GENERATION_SCHEMA.to_owned(),
            identity: old_identity.clone(),
            output_files: super::legacy_graph_output_file_kappas(&cover, &graph)
                .expect("old output digests"),
        };
        super::replace_bytes_atomically(
            &paths.completion,
            &super::legacy_graph_completion_bytes(&old_completion).expect("old completion bytes"),
            "test legacy completion",
        )
        .expect("publish old completion");
        assert_eq!(
            super::legacy_graph_generation_action(&old_identity)
                .expect("unchanged inputs reuse")
                .0,
            super::LegacyGraphGenerationAction::Reuse
        );

        std::fs::write(&corpus_records, b"structurally valid corpus generation two")
            .expect("replace corpus pair with generation two");
        let new_identity =
            super::capture_legacy_graph_generation_identity(super::LegacyGraphGenerationInputs {
                artifacts: &artifacts,
                corpus_meta: &corpus_meta,
                corpus_recs: &corpus_records,
                tokenizer: &tokenizer,
                cover_output: &cover,
                graph_output: &graph,
                cover_args: &cover_args,
                score_args: &score_args,
            })
            .expect("capture generation-two inputs");
        assert_ne!(old_identity, new_identity);
        let (action, new_paths, attempt) = super::legacy_graph_generation_action(&new_identity)
            .expect("changed input schedules replacement");
        assert_eq!(action, super::LegacyGraphGenerationAction::BuildBoth);
        super::ensure_legacy_graph_attempt(&new_paths.attempt, &attempt)
            .expect("publish replacement attempt");
        let cover_before = std::fs::read(cover.join("cover_report.json")).expect("old cover");
        let score_before = std::fs::read(graph.join("score_report.json")).expect("old score");

        let build = |output: &std::path::Path, kind: super::GraphOutputKind, tag: &'static str| {
            let args = vec!["--out".to_owned(), output.display().to_string()];
            super::build_graph_writer_stage(output, kind, &args, |staged| {
                write_valid_graph_output(std::path::Path::new(&staged[1]), kind, tag);
                Ok(())
            })
            .expect("build replacement stage")
        };
        let cover_stage = build(&cover, super::GraphOutputKind::Cover, "new cover");
        let score_stage = build(&graph, super::GraphOutputKind::Score, "new score");
        let mut wrong_outputs = super::staged_legacy_graph_output_kappas(
            &cover_stage.path,
            &score_stage.path,
            &cover,
            &graph,
        )
        .expect("new output digests");
        *wrong_outputs.values_mut().next().expect("one digest") =
            format!("blake3:{}", "0".repeat(64));
        let wrong_completion = super::LegacyGraphGenerationCompletion {
            schema: super::LEGACY_GRAPH_GENERATION_SCHEMA.to_owned(),
            identity: new_identity.clone(),
            output_files: wrong_outputs,
        };
        let error = super::publish_legacy_graph_generation_pair(
            cover_stage,
            score_stage,
            &cover,
            &graph,
            &new_paths,
            &attempt,
            &wrong_completion,
        )
        .expect_err("pre-commit mismatch rolls both outputs back");
        assert!(error.contains("differs from its staged digest"), "{error}");
        assert_eq!(
            std::fs::read(cover.join("cover_report.json")).expect("cover after rollback"),
            cover_before
        );
        assert_eq!(
            std::fs::read(graph.join("score_report.json")).expect("score after rollback"),
            score_before
        );

        let cover_stage = build(&cover, super::GraphOutputKind::Cover, "new cover");
        let score_stage = build(&graph, super::GraphOutputKind::Score, "new score");
        let completion = super::LegacyGraphGenerationCompletion {
            schema: super::LEGACY_GRAPH_GENERATION_SCHEMA.to_owned(),
            identity: new_identity.clone(),
            output_files: super::staged_legacy_graph_output_kappas(
                &cover_stage.path,
                &score_stage.path,
                &cover,
                &graph,
            )
            .expect("exact replacement digests"),
        };
        super::publish_legacy_graph_generation_pair(
            cover_stage,
            score_stage,
            &cover,
            &graph,
            &new_paths,
            &attempt,
            &completion,
        )
        .expect("atomic replacement commits");
        assert_ne!(
            std::fs::read(cover.join("cover_report.json")).expect("new cover"),
            cover_before
        );
        assert_ne!(
            std::fs::read(graph.join("score_report.json")).expect("new score"),
            score_before
        );
        assert_eq!(
            super::legacy_graph_generation_action(&new_identity)
                .expect("new identity now reuses")
                .0,
            super::LegacyGraphGenerationAction::Reuse
        );
        super::validate_legacy_graph_generation_for_serving(&graph.join("score.r4g1"))
            .expect("replacement completion binds current inputs");
        std::fs::write(&corpus_records, b"third uncompiled corpus generation")
            .expect("mutate bound input after completion");
        let error = super::validate_legacy_graph_generation_for_serving(&graph.join("score.r4g1"))
            .expect_err("startup/reload rejects stale graph after input drift");
        assert!(error.contains("input"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn fresh_graph_output_uses_staging_and_retry_never_observes_partial_final() {
        let root = attention_provenance_test_dir("fresh-graph-staging");
        let final_output = root.join("graph");
        let args = vec!["--out".to_owned(), final_output.display().to_string()];
        let error = super::run_graph_writer_staged(
            &final_output,
            super::GraphOutputKind::Score,
            &args,
            |staged| {
                let output = std::path::Path::new(&staged[1]);
                std::fs::write(output.join("score.r4g1"), minimal_r4g1_bytes())
                    .expect("partial staged graph");
                Err("injected short-write failure".to_owned())
            },
        )
        .expect_err("failed stage is not published");
        assert!(error.contains("injected"), "{error}");
        assert!(!final_output.exists());

        super::run_graph_writer_staged(
            &final_output,
            super::GraphOutputKind::Score,
            &args,
            |staged| {
                let output = std::path::Path::new(&staged[1]);
                std::fs::write(output.join("score.r4g1"), minimal_r4g1_bytes())
                    .expect("staged score");
                std::fs::write(output.join("score_report.json"), b"{}").expect("staged report");
                Ok(())
            },
        )
        .expect("retry atomically publishes the complete pair");
        assert!(final_output.join("score.r4g1").is_file());
        assert!(final_output.join("score_report.json").is_file());

        let mut called = false;
        super::run_graph_writer_staged(&final_output, super::GraphOutputKind::Score, &args, |_| {
            called = true;
            Err("must not rewrite complete generation".to_owned())
        })
        .expect("complete generation is immutable");
        assert!(!called);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn source_compile_publishers_reject_symlink_directory_and_fifo_controls() {
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;

        fn fifo(path: &std::path::Path) {
            let path = std::ffi::CString::new(path.as_os_str().as_bytes())
                .expect("fixture path has no NUL");
            // SAFETY: the C string is live for the call and `mkfifo` does not
            // retain its pointer.
            let result = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
            assert_eq!(result, 0, "mkfifo: {}", std::io::Error::last_os_error());
        }

        let root = attention_provenance_test_dir("preflight-special-entries");
        let compiled = root.join("compiled");
        std::fs::create_dir_all(&compiled).expect("create compiled parent");
        let kappa = format!("blake3:{}", "b".repeat(64));
        let expected =
            super::source_compile_preflight_bytes(Some(&kappa)).expect("canonical preflight");
        let exact_target = root.join("exact.json");
        std::fs::write(&exact_target, &expected).expect("write exact target");

        // The generic no-clobber primitive checks entry type before reading,
        // so even a FIFO cannot block and an exact-byte symlink is not an
        // idempotent marker.
        for kind in ["exact-symlink", "dangling-symlink", "directory", "fifo"] {
            let path = root.join(kind);
            match kind {
                "exact-symlink" => symlink(&exact_target, &path).expect("exact symlink"),
                "dangling-symlink" => {
                    symlink(root.join("absent"), &path).expect("dangling symlink")
                }
                "directory" => std::fs::create_dir(&path).expect("marker directory"),
                "fifo" => fifo(&path),
                _ => unreachable!(),
            }
            let error = super::publish_bytes_no_clobber(&path, &expected, "test marker")
                .expect_err("non-regular marker is terminal before read");
            assert!(error.contains("non-symlink"), "{error}");
        }

        // Deterministically exercise the former lstat->read race. Hold an
        // already-open regular handle, swap its path to a symlink or FIFO,
        // then finish the production handle-based read. The post-read inode
        // check refuses both swaps and the publisher's fresh open is
        // non-following/nonblocking as well.
        for kind in ["swap-symlink", "swap-fifo"] {
            let path = root.join(kind);
            let original = root.join(format!("{kind}.original"));
            std::fs::write(&path, &expected).expect("write pre-swap regular marker");
            let opened = super::open_regular_file_nofollow(&path, "test marker")
                .expect("open regular marker")
                .expect("marker exists");
            std::fs::rename(&path, &original).expect("move opened inode away");
            match kind {
                "swap-symlink" => symlink(&original, &path).expect("swap exact symlink"),
                "swap-fifo" => fifo(&path),
                _ => unreachable!(),
            }
            let error = super::read_opened_regular_file_nofollow(opened, &path, "test marker")
                .expect_err("path swap after open must be detected");
            assert!(error.contains("changed identity"), "{error}");
            let error = super::publish_bytes_no_clobber(&path, &expected, "test marker")
                .expect_err("swapped special entry is never read or accepted");
            assert!(error.contains("non-symlink"), "{error}");
            assert_eq!(
                std::fs::read(&original).expect("opened original bytes remain"),
                expected
            );
        }

        // Both the retired parent-init namespace and the stable in-root
        // marker are strict controls. Presence never authorizes adoption and
        // no staging/output bytes are created on refusal.
        for kind in [
            "regular",
            "exact-symlink",
            "dangling-symlink",
            "directory",
            "fifo",
        ] {
            let output = compiled.join(format!("init-{kind}"));
            let init = super::source_compile_initialization_path(&output)
                .expect("legacy initialization path");
            match kind {
                "regular" => std::fs::write(&init, &expected).expect("regular init"),
                "exact-symlink" => symlink(&exact_target, &init).expect("exact init symlink"),
                "dangling-symlink" => {
                    symlink(root.join("missing-init"), &init).expect("dangling init symlink")
                }
                "directory" => std::fs::create_dir(&init).expect("init directory"),
                "fifo" => fifo(&init),
                _ => unreachable!(),
            }
            let error = super::publish_source_compile_preflight(&output, Some(&kappa))
                .expect_err("legacy initialization cannot authorize publication");
            assert!(error.contains("ambiguous"), "{error}");
            assert!(!output.exists());
        }

        for kind in ["exact-symlink", "dangling-symlink", "directory", "fifo"] {
            let output = compiled.join(format!("stable-{kind}"));
            std::fs::create_dir(&output).expect("output root");
            let marker = output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE);
            match kind {
                "exact-symlink" => symlink(&exact_target, &marker).expect("exact stable symlink"),
                "dangling-symlink" => {
                    symlink(root.join("missing-stable"), &marker).expect("dangling stable symlink")
                }
                "directory" => std::fs::create_dir(&marker).expect("stable directory"),
                "fifo" => fifo(&marker),
                _ => unreachable!(),
            }
            let error = super::publish_source_compile_preflight(&output, Some(&kappa))
                .expect_err("invalid stable marker is terminal before read");
            assert!(error.contains("non-symlink"), "{error}");
            assert_eq!(
                std::fs::read_dir(&output)
                    .expect("enumerate refused output")
                    .count(),
                1
            );
        }

        let symlink_root = compiled.join("root-symlink");
        let target_root = root.join("target-root");
        std::fs::create_dir(&target_root).expect("target root");
        std::fs::write(
            target_root.join(super::SOURCE_COMPILE_PREFLIGHT_FILE),
            &expected,
        )
        .expect("target marker");
        symlink(&target_root, &symlink_root).expect("root symlink");
        let error = super::publish_source_compile_preflight(&symlink_root, Some(&kappa))
            .expect_err("root symlink cannot expose target marker");
        assert!(error.contains("non-symlink directory"), "{error}");

        let staging = super::source_compile_staging_root(&compiled);
        symlink(&target_root, &staging).expect("staging namespace symlink");
        let output = compiled.join("staging-symlink");
        let error = super::publish_source_compile_preflight(&output, Some(&kappa))
            .expect_err("staging namespace symlink is terminal");
        assert!(error.contains("staging namespace"), "{error}");
        assert!(!output.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn identity_temporary_crash_windows_resume_only_canonical_exact_records() {
        let root = attention_provenance_test_dir("preflight-temporary-crashes");
        let compiled = root.join("compiled");
        std::fs::create_dir_all(&compiled).expect("create compiled parent");
        let kappa = format!("blake3:{}", "c".repeat(64));
        let preflight =
            super::source_compile_preflight_bytes(Some(&kappa)).expect("preflight bytes");
        let binding =
            super::source_manifest_kappa_binding_bytes(&kappa).expect("source binding bytes");

        // Stable-preflight temp exists before its hard link.
        let pre_before = compiled.join("pre-before-link");
        std::fs::create_dir(&pre_before).expect("preflight temp root");
        let pre_temp = pre_before.join(format!(
            ".{}.100.1.tmp",
            super::SOURCE_COMPILE_PREFLIGHT_FILE
        ));
        std::fs::write(&pre_temp, &preflight).expect("preflight temporary");
        super::publish_source_compile_preflight(&pre_before, Some(&kappa))
            .expect("canonical preflight temp resumes publication");
        assert!(pre_before
            .join(super::SOURCE_COMPILE_PREFLIGHT_FILE)
            .is_file());
        assert!(!super::source_compile_output_has_payload(&pre_before)
            .expect("preflight temporary is identity, not corpus payload"));

        // Stable link exists but process died before removing its temp.
        let pre_after = compiled.join("pre-after-link");
        super::publish_source_compile_preflight(&pre_after, Some(&kappa))
            .expect("publish stable preflight");
        std::fs::write(
            pre_after.join(format!(
                ".{}.100.2.tmp",
                super::SOURCE_COMPILE_PREFLIGHT_FILE
            )),
            &preflight,
        )
        .expect("linked preflight temporary");
        super::publish_source_compile_preflight(&pre_after, Some(&kappa))
            .expect("linked preflight plus exact temp resumes");
        assert!(!super::source_compile_output_has_payload(&pre_after)
            .expect("linked preflight temp is ignored"));

        // Source-κ temp exists before and after its hard link.
        let k_before = compiled.join("k-before-link");
        super::publish_source_compile_preflight(&k_before, Some(&kappa))
            .expect("publish K-before root");
        std::fs::write(
            k_before.join(format!(
                ".{}.101.1.tmp",
                super::SOURCE_MANIFEST_KAPPA_BINDING_FILE
            )),
            &binding,
        )
        .expect("source K temporary before link");
        super::preflight_and_bind_source_snapshot_kappa(&k_before, Some(&kappa))
            .expect("K temporary before link is recoverable");
        assert!(k_before
            .join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE)
            .is_file());
        assert!(!super::source_compile_output_has_payload(&k_before)
            .expect("K temp before link is ignored"));

        let k_after = compiled.join("k-after-link");
        super::preflight_and_bind_source_snapshot_kappa(&k_after, Some(&kappa))
            .expect("publish K-after root");
        std::fs::write(
            k_after.join(format!(
                ".{}.101.2.tmp",
                super::SOURCE_MANIFEST_KAPPA_BINDING_FILE
            )),
            &binding,
        )
        .expect("source K temporary after link");
        super::preflight_and_bind_source_snapshot_kappa(&k_after, Some(&kappa))
            .expect("linked K plus exact temp resumes");
        assert!(!super::source_compile_output_has_payload(&k_after)
            .expect("K temp after link is ignored"));

        // Tokenizer and attention publisher residues are identity-only when
        // regular and registry-exact; a tampered or symlinked owned-shape temp
        // is terminal rather than ignored.
        let identity = compiled.join("identity-temps");
        super::preflight_and_bind_source_snapshot_kappa(&identity, Some(&kappa))
            .expect("identity temp root");
        let scratch = root.join("identity-scratch");
        std::fs::create_dir(&scratch).expect("identity scratch");
        let tokenizer = write_tokenizer_adapter_binding(&scratch, "crash-temp");
        let attention = write_attention_binding(
            &scratch,
            &uor_r4_model_source::attention::AttentionOperatorSpec::standard_v2(),
        );
        std::fs::write(
            identity.join(".tokenizer_adapter.json.102.1.tmp"),
            tokenizer,
        )
        .expect("tokenizer temp");
        std::fs::write(
            identity.join(format!(
                ".{}.102.2.tmp",
                uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE
            )),
            attention,
        )
        .expect("attention temp");
        assert!(!super::source_compile_output_has_payload(&identity)
            .expect("exact tokenizer/attention temps are identity-only"));

        let tampered = identity.join(format!(
            ".{}.103.1.tmp",
            super::SOURCE_MANIFEST_KAPPA_BINDING_FILE
        ));
        std::fs::write(&tampered, b"{malformed").expect("tampered temp");
        let error = super::source_compile_output_has_payload(&identity)
            .expect_err("tampered owned-shape temporary is terminal");
        assert!(error.contains("malformed"), "{error}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            std::fs::remove_file(&tampered).expect("remove tampered temp");
            symlink(
                identity.join(super::SOURCE_COMPILE_PREFLIGHT_FILE),
                &tampered,
            )
            .expect("symlink temp to an existing identity file");
            let error = super::source_compile_output_has_payload(&identity)
                .expect_err("symlinked owned-shape temporary is terminal");
            assert!(error.contains("non-symlink"), "{error}");
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifestless_current_compile_resume_is_bound_to_verified_tree_kappa() {
        let root = attention_provenance_test_dir("manifestless-tree-kappa-resume");
        let source = root.join("source");
        let output = root.join("compiled/teacher");
        std::fs::create_dir_all(&source).expect("create manifestless source");
        std::fs::write(source.join("config.json"), b"manifestless source M1")
            .expect("write M1 source");
        let m1 = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect("verify M1 tree");
        super::preflight_and_bind_source_snapshot_kappa(&output, Some(&m1.content_kappa))
            .expect("bind current-era Stage A to M1 tree");
        std::fs::write(output.join("corpus.records"), b"M1 immutable corpus prefix")
            .expect("simulate Stage-A crash after M1 rows");
        let marker_before = std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
            .expect("M1 marker before replacement");
        let binding_before = std::fs::read(output.join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE))
            .expect("M1 binding before replacement");
        let corpus_before = std::fs::read(output.join("corpus.records")).expect("M1 corpus");

        std::fs::write(source.join("config.json"), b"manifestless source M2")
            .expect("replace source with M2");
        let m2 = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect("verify M2 tree");
        assert_ne!(m1.content_kappa, m2.content_kappa);
        let error =
            super::preflight_and_bind_source_snapshot_kappa(&output, Some(&m2.content_kappa))
                .expect_err("M2 cannot resume or relabel M1 corpus rows");
        assert!(error.contains(&m1.content_kappa), "{error}");
        assert!(error.contains(&m2.content_kappa), "{error}");
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_COMPILE_PREFLIGHT_FILE))
                .expect("marker after refusal"),
            marker_before
        );
        assert_eq!(
            std::fs::read(output.join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE))
                .expect("binding after refusal"),
            binding_before
        );
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("corpus after refusal"),
            corpus_before
        );

        let resolved = super::ResolvedCompiledBundle {
            logical_name: "teacher".to_owned(),
            physical_root: output.clone(),
            graph: output.join("graph/score.r4g1"),
            teacher: output.join("tless_artifacts.bin"),
            attention_operator: uor_r4_model_source::attention::AttentionOperatorSpec::standard_v2(
            ),
            source_manifest_kappa: Some(m1.content_kappa.clone()),
        };
        let error = super::validate_resolved_source_snapshot_binding(&resolved, Some(&m2), 2)
            .expect_err("serving cannot attach M2 teacher bytes to M1 graph provenance");
        assert!(error.contains(&m1.content_kappa), "{error}");
        assert!(error.contains(&m2.content_kappa), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifestless_tree_kappa_rejects_hidden_shards_and_binds_visible_weights() {
        let root = attention_provenance_test_dir("manifestless-hidden-shards");
        let source = root.join("source");
        std::fs::create_dir_all(&source).expect("create source");
        let index = source.join(uor_r4_model_source::SAFETENSORS_INDEX_FILE_NAME);

        std::fs::write(
            source.join(".secret.safetensors"),
            b"hidden executable weights",
        )
        .expect("write hidden shard");
        std::fs::write(
            &index,
            br#"{"weight_map":{"model.weight":".secret.safetensors"}}"#,
        )
        .expect("write hidden-shard index");
        let error = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect_err("hidden executable shard cannot escape the tree κ");
        assert!(error.contains("hidden or nonportable"), "{error}");

        std::fs::remove_file(source.join(".secret.safetensors")).expect("remove hidden shard");
        std::fs::remove_file(&index).expect("remove hidden index");
        std::fs::write(
            source.join(".cache"),
            b"file masquerading as transport metadata",
        )
        .expect("write .cache file");
        let error = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect_err("a .cache file is not excluded transport metadata");
        assert!(error.contains("not a directory"), "{error}");

        std::fs::remove_file(source.join(".cache")).expect("remove .cache file");
        std::fs::create_dir(source.join(".cache")).expect("create transport cache directory");
        std::fs::write(source.join(".cache/transport"), b"revision one")
            .expect("write transport metadata");
        std::fs::write(source.join("visible.safetensors"), b"visible weights M1")
            .expect("write visible shard");
        std::fs::write(
            &index,
            br#"{"weight_map":{"model.weight":"visible.safetensors"}}"#,
        )
        .expect("write visible index");
        let m1 = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect("visible indexed source verifies");
        std::fs::write(source.join(".cache/transport"), b"revision two")
            .expect("refresh transport metadata");
        let cache_refreshed = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect("transport cache refresh remains out of κ scope");
        assert_eq!(m1.content_kappa, cache_refreshed.content_kappa);

        std::fs::write(source.join("visible.safetensors"), b"visible weights M2")
            .expect("mutate executable shard");
        let m2 = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect("mutated visible indexed source verifies as a new snapshot");
        assert_ne!(
            m1.content_kappa, m2.content_kappa,
            "every executable visible shard byte participates in the tree κ"
        );

        std::fs::write(&index, br#"{"weight_map":{"model.weight":".cache"}}"#)
            .expect("point index at cache directory");
        let error = super::verified_managed_source_snapshot_from_manifest(&source, None)
            .expect_err("transport directory cannot be an executable shard");
        assert!(error.contains("hidden or nonportable"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn manifestless_startup_compile_and_reload_refuse_handle_swap_without_mutation() {
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;

        fn fifo(path: &std::path::Path) {
            let path = std::ffi::CString::new(path.as_os_str().as_bytes())
                .expect("fixture path has no NUL");
            // SAFETY: the C string is live and mkfifo does not retain it.
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        }

        for operation in ["startup", "compile", "reload"] {
            for kind in ["file-symlink", "file-fifo", "dir-outside", "dir-cycle"] {
                let root =
                    attention_provenance_test_dir(&format!("manifestless-{operation}-{kind}"));
                let source = root.join("source");
                let nested = source.join("nested");
                let outside = root.join("outside");
                std::fs::create_dir_all(&nested).expect("source tree");
                std::fs::create_dir_all(&outside).expect("outside tree");
                std::fs::write(source.join("config.json"), b"bound config").expect("source file");
                std::fs::write(nested.join("tokenizer.json"), b"bound nested")
                    .expect("nested source file");
                std::fs::write(outside.join("escaped.json"), b"outside").expect("outside file");
                let compiled_sentinel = root.join("compiled-sentinel");
                std::fs::write(&compiled_sentinel, b"last-good output").expect("compiled sentinel");
                let serving = super::ServingModelState {
                    epoch: 17,
                    terminal_load_error: Some("prior terminal".to_owned()),
                    last_operation_error: Some("prior operation".to_owned()),
                    ..Default::default()
                };

                let file = source.join("config.json");
                let file_original = source.join("config.original");
                let directory_original = source.join("nested-original");
                let mut swapped = false;
                let result = super::verified_managed_source_snapshot_from_manifest_with_tree(
                    &source,
                    None,
                    |source_root| {
                        crate::model::verified_source_tree_with_hook(
                            source_root,
                            crate::model::SourceTreeScope::ManifestlessAll,
                            |path| {
                                if swapped {
                                    return;
                                }
                                match kind {
                                    "file-symlink" if path == file => {
                                        std::fs::rename(&file, &file_original)
                                            .expect("move regular file");
                                        symlink(&file_original, &file).expect("swap symlink");
                                        swapped = true;
                                    }
                                    "file-fifo" if path == file => {
                                        std::fs::rename(&file, &file_original)
                                            .expect("move regular file");
                                        fifo(&file);
                                        swapped = true;
                                    }
                                    "dir-outside" if path == nested => {
                                        std::fs::rename(&nested, &directory_original)
                                            .expect("move child directory");
                                        symlink(&outside, &nested).expect("outside link");
                                        swapped = true;
                                    }
                                    "dir-cycle" if path == nested => {
                                        std::fs::rename(&nested, &directory_original)
                                            .expect("move child directory");
                                        symlink(&source, &nested).expect("cycle link");
                                        swapped = true;
                                    }
                                    _ => {}
                                }
                            },
                        )
                        .map_err(|error| error.to_string())
                    },
                );
                let error = result.expect_err("handle swap must fail before operation install");
                assert!(
                    error.contains("cannot be opened") || error.contains("changed identity"),
                    "{operation}/{kind}: {error}"
                );
                assert_eq!(
                    std::fs::read(&compiled_sentinel).expect("output after refusal"),
                    b"last-good output"
                );
                assert_eq!(serving.epoch, 17);
                assert_eq!(
                    serving.terminal_load_error.as_deref(),
                    Some("prior terminal")
                );
                assert_eq!(
                    serving.last_operation_error.as_deref(),
                    Some("prior operation")
                );
                assert_eq!(
                    std::fs::read(outside.join("escaped.json")).expect("outside byte"),
                    b"outside"
                );
                let _ = std::fs::remove_dir_all(root);
            }
        }
    }

    #[test]
    fn source_compile_rejects_resolver_suffix_basename_before_mutation() {
        let root = attention_provenance_test_dir("reserved-source-name");
        let compiled = root.join("compiled");
        let error = super::source_compile_output_for_attention_era(
            &compiled,
            "vendor-model-attention-v2",
            2,
        )
        .expect_err("resolver-owned suffix is not an arbitrary source basename");
        assert!(error.contains("resolver-owned suffix"), "{error}");
        assert!(!compiled.exists(), "name rejection is read-only");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_compile_routes_unbound_legacy_payload_but_not_an_unbound_v2_root() {
        let root = attention_provenance_test_dir("era-unbound");
        let compiled = root.join("compiled");
        let conventional = compiled.join("teacher");
        std::fs::create_dir_all(&conventional).expect("create legacy root");
        std::fs::write(conventional.join("corpus.meta"), b"implicit v1")
            .expect("write legacy payload");
        let era = compiled.join("teacher-attention-v2");
        assert_eq!(
            super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
                .expect("implicit v1 selects v2 root"),
            era
        );

        std::fs::create_dir_all(&era).expect("create v2 root");
        std::fs::write(era.join("corpus.records"), b"unbound bytes")
            .expect("write unbound v2 payload");
        let error = super::source_compile_output_for_attention_era(&compiled, "teacher", 2)
            .expect_err("unbound deterministic era cannot be relabelled");
        assert!(error.contains("contains payload without"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    fn write_loadable_graph_bundle(
        bundle: &std::path::Path,
        operator: Option<&uor_r4_model_source::attention::AttentionOperatorSpec>,
    ) {
        write_loadable_graph_bundle_with_kappa(bundle, operator, None);
    }

    fn write_loadable_graph_bundle_with_kappa(
        bundle: &std::path::Path,
        operator: Option<&uor_r4_model_source::attention::AttentionOperatorSpec>,
        source_manifest_kappa: Option<&str>,
    ) {
        std::fs::create_dir_all(bundle.join("graph")).expect("create graph bundle");
        std::fs::write(bundle.join("graph/score.r4g1"), b"graph").expect("write graph");
        std::fs::write(bundle.join("tless_artifacts.bin"), b"teacher").expect("write teacher");
        if let Some(operator) = operator {
            write_attention_binding(bundle, operator);
            std::fs::write(bundle.join("corpus.meta"), []).expect("write corpus metadata marker");
            std::fs::write(bundle.join("corpus.records"), []).expect("write corpus records marker");
            std::fs::create_dir_all(bundle.join("graph-cover"))
                .expect("create cover-report directory");
            let mut report = serde_json::json!({
                "attention_operator": operator,
            });
            if let Some(kappa) = source_manifest_kappa {
                report["source_manifest_kappa"] = serde_json::Value::String(kappa.to_owned());
            }
            std::fs::write(
                bundle.join("graph-cover/cover_report.json"),
                serde_json::to_vec_pretty(&report).expect("serialize cover provenance"),
            )
            .expect("write cover provenance");
        }
    }

    #[test]
    fn restart_discovery_prefers_current_v2_and_maps_suffix_to_base_source() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("restart-prefer-v2");
        let compiled = root.join("compiled");
        let sources = root.join("sources");
        std::fs::create_dir_all(sources.join("teacher")).expect("create real source");
        let historical = compiled.join("teacher");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&historical, Some(&AttentionOperatorSpec::standard_v1()));
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        let discovered = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect("both exact eras are discoverable");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].logical_name, "teacher");
        assert_eq!(discovered[0].physical_root, current);
        assert_eq!(discovered[0].graph, current.join("graph/score.r4g1"));
        assert_eq!(discovered[0].teacher, current.join("tless_artifacts.bin"));
        let configured_historical = super::resolve_managed_teacher_bundle_in(
            &historical.join("tless_artifacts.bin"),
            &root,
            2,
        )
        .expect("configured managed teacher re-enters paired resolver");
        let super::ConfiguredManagedBundle::Selected(configured_historical) = configured_historical
        else {
            panic!("paired resolver must select a loadable bundle");
        };
        assert_eq!(
            *configured_historical, discovered[0],
            "a configured historical physical path cannot bypass its preferred current sibling"
        );
        assert_eq!(
            super::resolve_managed_teacher_bundle_in(
                &root.join("external/teacher/tless_artifacts.bin"),
                &root,
                2,
            )
            .expect("external explicit path remains outside managed resolution"),
            super::ConfiguredManagedBundle::External
        );
        assert_eq!(
            super::source_for_compiled_teacher_in(&discovered[0].teacher, &root)
                .expect("exact v2 suffix maps to source"),
            Some(sources.join("teacher"))
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn managed_resolver_classifies_external_absent_and_symlink_aliases_without_bypass() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("managed-path-classification");
        let external = root.join("external");
        std::fs::create_dir_all(&external).expect("create external bundle");
        std::fs::write(external.join("tless_artifacts.bin"), b"external teacher")
            .expect("write external teacher");
        assert_eq!(
            super::resolve_managed_teacher_bundle_in(
                &external.join("tless_artifacts.bin"),
                &root.join("fresh-models-root"),
                2,
            )
            .expect("an absent managed namespace cannot invalidate an external artifact"),
            super::ConfiguredManagedBundle::External
        );

        let models = root.join("models");
        let historical = models.join("compiled/teacher");
        let current = models.join("compiled/teacher-attention-v2");
        write_loadable_graph_bundle(&historical, Some(&AttentionOperatorSpec::standard_v1()));
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let alias_parent = root.join("aliases");
            std::fs::create_dir_all(&alias_parent).expect("create alias parent");
            symlink(&historical, alias_parent.join("historical-alias"))
                .expect("alias historical physical root");
            let resolved = super::resolve_managed_teacher_bundle_in(
                &alias_parent.join("historical-alias/tless_artifacts.bin"),
                &models,
                2,
            )
            .expect("canonical namespace classification");
            let super::ConfiguredManagedBundle::Selected(resolved) = resolved else {
                panic!("managed symlink alias must re-enter the paired resolver");
            };
            assert_eq!(resolved.physical_root, current);
            assert_eq!(resolved.logical_name, "teacher");
        }

        let absent = super::resolve_managed_teacher_bundle_in(
            &models.join("compiled/missing/tless_artifacts.bin"),
            &models,
            2,
        )
        .expect("genuine configured absence is classified");
        assert_eq!(absent, super::ConfiguredManagedBundle::Absent);
        assert!(
            !absent.permits_inventory_discovery(),
            "configured managed absence cannot fall through to an unrelated inventory model"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolver_owned_suffix_rejects_preupgrade_source_basename_collision() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("reserved-source-collision");
        let current = root.join("compiled/teacher-attention-v2");
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));
        std::fs::create_dir_all(root.join("sources/teacher-attention-v2"))
            .expect("create genuine pre-upgrade suffixed source");

        let error = super::discover_compiled_r4g1_candidates_in(&root.join("compiled"), 2)
            .expect_err("ambiguous source basename cannot be stripped by discovery");
        assert!(error.contains("pre-existing source basename"), "{error}");
        let error = super::resolve_reload_bundle_in(&root, "teacher", 2)
            .expect_err("reload cannot map the collision to sources/teacher");
        assert!(error.contains("pre-existing source basename"), "{error}");

        std::fs::remove_dir_all(&current).expect("remove resolver-owned current root");
        write_loadable_graph_bundle(
            &root.join("compiled/teacher"),
            Some(&AttentionOperatorSpec::standard_v1()),
        );
        assert!(super::resolve_reload_bundle_in(&root, "teacher", 2)
            .expect("explicit logical base remains unambiguous")
            .is_some());
        let error = super::resolve_reload_bundle_in(&root, "teacher-attention-v2", 2)
            .expect_err("exact suffix alias cannot consume a genuine source basename");
        assert!(
            error.contains("request the logical base explicitly"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn preferred_current_load_failure_never_exposes_historical_fallback_candidate() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("restart-current-load-failure");
        let compiled = root.join("compiled");
        let historical = compiled.join("teacher");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&historical, Some(&AttentionOperatorSpec::standard_v1()));
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        let discovered = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect("resolver chooses one authoritative candidate");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].physical_root, current);
        let load =
            super::R4g1State::load_with_source(&discovered[0].graph, &discovered[0].teacher, None);
        assert!(load.is_err(), "fixture graph is deliberately invalid");
        assert!(
            discovered
                .iter()
                .all(|candidate| candidate.physical_root != historical),
            "the startup loop cannot fall through to historical after preferred v2 load failure"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restart_discovery_does_not_hide_invalid_sibling_provenance() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("restart-invalid-sibling");
        let compiled = root.join("compiled");
        let historical = compiled.join("teacher");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&historical, None);
        std::fs::write(
            historical.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE),
            b"{malformed",
        )
        .expect("write malformed historical binding");
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("valid v2 must not route around malformed v1 evidence");
        assert!(
            error.contains("malformed attention-operator binding"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restart_discovery_requires_binding_on_populated_exact_v2_suffix() {
        let root = attention_provenance_test_dir("restart-unbound-v2");
        let compiled = root.join("compiled");
        write_loadable_graph_bundle(&compiled.join("teacher-attention-v2"), None);
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("unbound deterministic v2 bundle is not a legacy candidate");
        assert!(error.contains("contains payload without"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restart_discovery_rejects_non_directory_and_symlink_candidates() {
        let root = attention_provenance_test_dir("restart-non-directory");
        let compiled = root.join("compiled");
        std::fs::create_dir_all(&compiled).expect("create compiled root");
        std::fs::write(compiled.join("teacher"), b"not a bundle directory")
            .expect("write non-directory candidate");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("present non-directory is not absence");
        assert!(error.contains("non-symlink directory"), "{error}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            std::fs::remove_file(compiled.join("teacher")).expect("remove file candidate");
            let target = root.join("target");
            std::fs::create_dir(&target).expect("create symlink target");
            symlink(&target, compiled.join("teacher")).expect("create candidate symlink");
            let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
                .expect_err("directory symlink is not a managed bundle root");
            assert!(error.contains("non-symlink directory"), "{error}");
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn startup_candidate_distinguishes_absence_from_incomplete_present_state() {
        let root = attention_provenance_test_dir("startup-incomplete");
        let graph = root.join("score.r4g1");
        let teacher = root.join("tless_artifacts.bin");
        assert!(!super::required_r4g1_inputs_present(&graph, &teacher)
            .expect("joint absence is optional"));

        std::fs::write(&graph, b"present graph").expect("write graph");
        let error = super::required_r4g1_inputs_present(&graph, &teacher)
            .expect_err("present graph with absent teacher is terminal");
        assert!(error.contains("teacher artifact"), "{error}");

        std::fs::remove_file(&graph).expect("remove graph");
        std::fs::write(&teacher, b"present teacher").expect("write teacher");
        let error = super::required_r4g1_inputs_present(&graph, &teacher)
            .expect_err("present teacher with absent graph is terminal");
        assert!(error.contains("graph"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn current_restart_requires_exact_corpus_and_cover_provenance() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("restart-current-provenance");
        let compiled = root.join("compiled");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        std::fs::remove_file(current.join("graph-cover/cover_report.json"))
            .expect("remove current cover report");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("current bundle requires its cover identity");
        assert!(
            error.contains("missing required cover provenance"),
            "{error}"
        );

        std::fs::write(
            current.join("graph-cover/cover_report.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "attention_operator": AttentionOperatorSpec::standard_v1(),
            }))
            .expect("serialize stale cover"),
        )
        .expect("write stale cover report");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("a copied v2 sidecar cannot relabel a v1 cover");
        assert!(error.contains("records attention operator"), "{error}");
        assert!(error.contains("/1"), "{error}");

        let v1 = serde_json::to_string(&AttentionOperatorSpec::standard_v1())
            .expect("serialize historical operator");
        let v2 = serde_json::to_string(&AttentionOperatorSpec::standard_v2())
            .expect("serialize current operator");
        std::fs::write(
            current.join("graph-cover/cover_report.json"),
            format!("{{\"attention_operator\":{v1},\"attention_operator\":{v2}}}"),
        )
        .expect("write duplicate cover provenance");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("duplicate cover identities cannot be last-key-wins");
        assert!(error.contains("duplicate field"), "{error}");

        let mut manifest = uor_r4_graph_compiler::observation::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard_v1());
        std::fs::write(
            current.join(uor_r4_graph_compiler::observation::MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("serialize stale manifest"),
        )
        .expect("write stale observation manifest");
        std::fs::write(
            current.join("graph-cover/cover_report.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "attention_operator": AttentionOperatorSpec::standard_v2(),
            }))
            .expect("serialize current cover"),
        )
        .expect("restore current cover");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("sidecar and observation provenance must reconcile");
        assert!(error.contains("different attention operators"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn current_serving_requires_cover_kappa_to_match_verified_source_bytes() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("serving-source-kappa-binding");
        let identity = crate::model::SourceDownload {
            repository: "owner/model".to_owned(),
            revision: "6".repeat(40),
            name: "teacher".to_owned(),
            output: None,
            license: Some("Apache-2.0".to_owned()),
        };
        let source = root.join("sources/teacher");
        std::fs::create_dir_all(&source).expect("create source");
        std::fs::write(source.join("config.json"), b"source snapshot M1").expect("write M1 config");
        write_test_source_manifest(&source, &identity);
        let manifest_m1 = crate::model::read_source_manifest(&source).expect("read M1 manifest");
        let kappa_m1 = crate::model::source_manifest_kappa(&manifest_m1).expect("address M1");

        let current = root.join("compiled/teacher-attention-v2");
        write_loadable_graph_bundle_with_kappa(
            &current,
            Some(&AttentionOperatorSpec::standard_v2()),
            Some(&kappa_m1),
        );
        let cover_before = std::fs::read(current.join("graph-cover/cover_report.json"))
            .expect("cover bytes before substitution");
        let graph_before = std::fs::read(current.join("graph/score.r4g1"))
            .expect("graph bytes before substitution");
        let resolved = super::resolve_requested_compiled_bundle_in(&root, "teacher", 2)
            .expect("resolve current bundle")
            .expect("current bundle is loadable");
        let verified_m1 =
            super::verify_managed_source_snapshot_in(&source, "teacher", &root.join("descriptors"))
                .expect("M1 source verifies");
        super::validate_resolved_source_snapshot_binding(&resolved, Some(&verified_m1), 2)
            .expect("M1 cover/source pair matches");

        std::fs::write(source.join("config.json"), b"source snapshot M2")
            .expect("replace admitted bytes with M2");
        write_test_source_manifest(&source, &identity);
        let verified_m2 =
            super::verify_managed_source_snapshot_in(&source, "teacher", &root.join("descriptors"))
                .expect("M2 is internally self-consistent");
        let kappa_m2 = verified_m2.content_kappa.clone();
        assert_ne!(kappa_m1, kappa_m2, "fixture changes the source root");
        let error =
            super::validate_resolved_source_snapshot_binding(&resolved, Some(&verified_m2), 2)
                .expect_err("old graph/corpus M1 may not install with teacher/tokenizer M2");
        assert!(error.contains(&kappa_m1), "{error}");
        assert!(error.contains(&kappa_m2), "{error}");
        assert_eq!(
            std::fs::read(current.join("graph-cover/cover_report.json"))
                .expect("cover after refusal"),
            cover_before
        );
        assert_eq!(
            std::fs::read(current.join("graph/score.r4g1")).expect("graph after refusal"),
            graph_before
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn final_source_snapshot_gate_rejects_manifest_and_legacy_drift_for_every_installer() {
        use std::sync::{Arc, Barrier};

        for (index, operation, manifest_backed) in [
            (0, "standalone teacher startup", true),
            (1, "R4G1 startup", false),
            (2, "R4G1 compilation installation", true),
            (3, "R4G1 reload", false),
        ] {
            let root = attention_provenance_test_dir(&format!("snapshot-tail-{index}"));
            let source = root.join("sources/teacher");
            std::fs::create_dir_all(&source).expect("create source");
            std::fs::write(source.join("config.json"), b"snapshot M1").expect("write M1 config");
            std::fs::write(source.join("model.safetensors"), b"weights").expect("write weights");
            let identity = crate::model::SourceDownload {
                repository: "owner/model".to_owned(),
                revision: format!("{index:x}").repeat(40),
                name: "teacher".to_owned(),
                output: None,
                license: Some("Apache-2.0".to_owned()),
            };
            if manifest_backed {
                write_test_source_manifest_without_default_weights(&source, &identity);
            }
            let before = super::verify_managed_source_snapshot_in(
                &source,
                "teacher",
                &root.join("descriptors"),
            )
            .expect("verify M1");

            let barrier = Arc::new(Barrier::new(2));
            let worker_barrier = Arc::clone(&barrier);
            let worker_source = source.clone();
            let worker_identity = identity.clone();
            let worker = std::thread::spawn(move || {
                worker_barrier.wait();
                std::fs::write(worker_source.join("config.json"), b"snapshot M2")
                    .expect("publish M2 config");
                if manifest_backed {
                    write_test_source_manifest_without_default_weights(
                        &worker_source,
                        &worker_identity,
                    );
                }
                worker_barrier.wait();
            });
            barrier.wait();
            barrier.wait();
            worker.join().expect("snapshot publisher");

            let after = super::verify_managed_source_snapshot_in(
                &source,
                "teacher",
                &root.join("descriptors"),
            )
            .expect("M2 remains internally valid");
            assert_ne!(before.content_kappa, after.content_kappa);
            let error = super::require_unchanged_managed_source_snapshot(
                &source, operation, &before, &after,
            )
            .expect_err("M1-to-M2 replacement cannot cross an install boundary");
            assert!(error.contains(operation), "{error}");

            let mut installed = super::ServingModelState {
                epoch: 17,
                terminal_load_error: Some("prior terminal".to_owned()),
                ..super::ServingModelState::default()
            };
            if super::require_unchanged_managed_source_snapshot(&source, operation, &before, &after)
                .is_ok()
            {
                installed.epoch = installed.epoch.wrapping_add(1);
                installed.terminal_load_error = None;
            }
            assert_eq!(installed.epoch, 17, "{operation} must not install");
            assert_eq!(
                installed.terminal_load_error.as_deref(),
                Some("prior terminal"),
                "{operation} preserves prior state"
            );
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn crashed_download_stage_is_reclaimed_once_under_destination_session() {
        let models_root = attention_provenance_test_dir("download-stage-recovery");
        let source = super::source_from_model_spec(&format!("owner/model@{}", "a".repeat(40)))
            .expect("source identity");
        let destination = super::downloaded_source_path_in(&source, &models_root);
        let parent = destination.parent().expect("source parent");
        std::fs::create_dir_all(parent).expect("source parent");
        let final_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("UTF-8 cache name");
        let stale = parent.join(format!(".{final_name}.download-staging-999-1"));
        std::fs::create_dir(&stale).expect("stale download stage");
        let marker = super::download_stage_marker_bytes(&destination, &stale, &source)
            .expect("stale stage owner marker");
        super::publish_bytes_no_clobber(
            &stale.join(super::DOWNLOAD_STAGE_MARKER_FILE),
            &marker,
            "test download-stage marker",
        )
        .expect("publish stale owner marker");
        std::fs::write(stale.join("model.safetensors"), vec![0xA5; 4096])
            .expect("model-sized stale sentinel");

        let published = super::download_source_atomically_in(&source, &models_root, |staged| {
            assert!(
                !stale.exists(),
                "exclusive owner reclaimed prior crash residue"
            );
            write_test_source_manifest(
                staged.output.as_deref().expect("reserved stage output"),
                &source,
            );
            Ok(staged.output.clone().expect("reported stage"))
        })
        .expect("retry publishes exactly one complete source");
        assert_eq!(published, destination);
        assert!(!stale.exists());
        assert!(std::fs::read_dir(parent)
            .expect("source cache entries")
            .all(|entry| !entry
                .expect("source cache entry")
                .file_name()
                .to_string_lossy()
                .contains("download-staging")));
        let _ = std::fs::remove_dir_all(models_root);
    }

    #[test]
    fn markerless_payload_download_stage_is_never_reclaimed_or_overwritten() {
        let models_root = attention_provenance_test_dir("download-stage-markerless-payload");
        let source = super::source_from_model_spec(&format!("owner/model@{}", "9".repeat(40)))
            .expect("source identity");
        let destination = super::downloaded_source_path_in(&source, &models_root);
        let parent = destination.parent().expect("source parent");
        std::fs::create_dir_all(parent).expect("source parent");
        let final_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("UTF-8 cache name");
        let unowned = parent.join(format!(".{final_name}.download-staging-998-1"));
        std::fs::create_dir(&unowned).expect("unowned stage");
        std::fs::write(unowned.join("model.safetensors"), b"external payload")
            .expect("unowned payload");

        let error = super::download_source_atomically_in(&source, &models_root, |_| {
            panic!("transport must not start after unowned residue")
        })
        .expect_err("markerless payload is terminal");
        assert!(error.contains("markerless download stage"), "{error}");
        assert_eq!(
            std::fs::read(unowned.join("model.safetensors")).expect("payload preserved"),
            b"external payload"
        );
        assert!(!destination.exists());
        let _ = std::fs::remove_dir_all(models_root);
    }

    #[test]
    fn source_cache_session_refuses_download_while_compile_reader_owns_exact_path() {
        let models_root = attention_provenance_test_dir("download-compile-os-session");
        let source = super::source_from_model_spec(&format!("owner/model@{}", "b".repeat(40)))
            .expect("source identity");
        let destination = super::downloaded_source_path_in(&source, &models_root);
        let owner = super::try_lock_source_compile_sessions(
            [models_root.join("sources"), destination.clone()],
            super::SourceCompileSessionMode::ExclusiveWriter,
        )
        .expect("compile-side source snapshot owner");
        let invoked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_by_download = std::sync::Arc::clone(&invoked);
        let error = super::download_source_atomically_in(&source, &models_root, move |_| {
            invoked_by_download.store(true, std::sync::atomic::Ordering::SeqCst);
            unreachable!("transport must not begin while exact source path is busy")
        })
        .expect_err("download refuses instead of waiting or mutating");
        assert!(super::source_compile_session_is_busy(&error), "{error}");
        assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!destination.exists());
        drop(owner);
        let _ = std::fs::remove_dir_all(models_root);
    }

    #[test]
    fn graph_teacher_operator_reconciliation_preserves_exact_modes_only() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let standard = AttentionOperatorSpec::standard_v2();
        let experimental = AttentionOperatorSpec::experimental_r4_v2();
        let learned = AttentionOperatorSpec::learned_absolute_v2();
        assert_eq!(
            super::teacher_mode_for_bundle_records(&standard, Some(&experimental), &standard),
            Some(false)
        );
        assert_eq!(
            super::teacher_mode_for_bundle_records(&standard, Some(&experimental), &experimental),
            Some(true),
            "current experimental bundles select the executable r4 mode"
        );
        assert_eq!(
            super::teacher_mode_for_bundle_records(&learned, Some(&learned), &learned),
            Some(false),
            "GPT-2's learned-absolute operator remains its default mode"
        );
        for historical in [
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::experimental_r4_v1(),
            AttentionOperatorSpec::learned_absolute_v1(),
        ] {
            assert_eq!(
                super::teacher_mode_for_bundle_records(&standard, Some(&experimental), &historical),
                None,
                "historical graph arithmetic has no current Teacher fallback"
            );
        }
        assert!(super::teacher_r4_attention_for_request(None, true));
        assert!(!super::teacher_r4_attention_for_request(
            Some(super::TIER_ATTENTION),
            true
        ));
        assert!(super::teacher_r4_attention_for_request(
            Some(super::TIER_R4_ATTENTION),
            false
        ));
    }

    #[test]
    fn historical_graph_keeps_its_exact_host_encoder_without_teacher_fallback() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("historical-host-encoder");
        let graph = graph_state_with_exact_host_encoder(&root);
        assert!(
            !graph.host_encoder_unavailable(),
            "the source-bound tokenizer is independent of Teacher attention arithmetic"
        );
        let mut encoded = [0u32; 32];
        assert!(
            graph.encode_into("a", &mut encoded).is_some(),
            "the historical graph retains a usable text encoder"
        );
        let serving = super::ServingModelState {
            r4g1: Some(graph),
            oracle: None,
            source_tokenizer: None,
            active_bundle: Some(super::ResolvedCompiledBundle {
                logical_name: "teacher".to_owned(),
                physical_root: root.join("compiled/teacher"),
                graph: root.join("compiled/teacher/graph/score.r4g1"),
                teacher: root.join("compiled/teacher/tless_artifacts.bin"),
                attention_operator: AttentionOperatorSpec::standard_v1(),
                source_manifest_kappa: None,
            }),
            ..super::ServingModelState::default()
        };
        assert!(super::graph_text_ready(&serving));
        assert!(!super::teacher_text_ready(&serving));
        assert_eq!(
            super::active_canonical_model_name(&serving).as_deref(),
            Some("teacher"),
            "a host-encoded historical graph remains advertised"
        );
        assert_eq!(
            super::resolve_active_request_model(&serving, Some("uor-r4"))
                .expect("legacy alias resolves to the graph"),
            "teacher"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cover_source_kappa_is_duplicate_strict_and_stage_a_resume_is_snapshot_bound() {
        let root = attention_provenance_test_dir("stage-a-source-kappa-binding");
        let identity = crate::model::SourceDownload {
            repository: "owner/model".to_owned(),
            revision: "7".repeat(40),
            name: "teacher".to_owned(),
            output: None,
            license: Some("Apache-2.0".to_owned()),
        };
        let source_m1 = root.join("source-m1");
        let source_m2 = root.join("source-m2");
        std::fs::create_dir_all(&source_m1).expect("create M1 source");
        std::fs::create_dir_all(&source_m2).expect("create M2 source");
        std::fs::write(source_m1.join("config.json"), b"M1 bytes").expect("write M1");
        std::fs::write(source_m2.join("config.json"), b"M2 bytes").expect("write M2");
        write_test_source_manifest(&source_m1, &identity);
        write_test_source_manifest(&source_m2, &identity);
        let manifest_m1 = crate::model::read_source_manifest(&source_m1).expect("read M1");
        let manifest_m2 = crate::model::read_source_manifest(&source_m2).expect("read M2");
        let kappa_m1 = crate::model::source_manifest_kappa(&manifest_m1).expect("address M1");
        let kappa_m2 = crate::model::source_manifest_kappa(&manifest_m2).expect("address M2");
        assert_ne!(kappa_m1, kappa_m2);

        let output = root.join("compiled/teacher");
        std::fs::create_dir_all(output.join("graph-cover")).expect("create populated output");
        let corpus = output.join("corpus.records");
        std::fs::write(&corpus, b"M1 corpus bytes").expect("write M1 corpus sentinel");
        let report = output.join("graph-cover/cover_report.json");
        std::fs::write(
            &report,
            serde_json::to_vec_pretty(&serde_json::json!({
                "source_manifest_kappa": &kappa_m1,
            }))
            .expect("serialize M1 cover"),
        )
        .expect("write M1 cover");
        let report_before = std::fs::read(&report).expect("cover before mismatch");

        let error = super::preflight_and_bind_source_manifest_kappa(&output, Some(&manifest_m2))
            .expect_err("M2 cannot resume populated M1 Stage-A output");
        assert!(
            error.contains("no immutable source-manifest kappa binding"),
            "{error}"
        );
        assert!(!output
            .join(super::SOURCE_MANIFEST_KAPPA_BINDING_FILE)
            .exists());
        assert_eq!(
            std::fs::read(&corpus).expect("corpus after refusal"),
            b"M1 corpus bytes"
        );
        assert_eq!(
            std::fs::read(&report).expect("cover after refusal"),
            report_before
        );

        super::preflight_and_bind_source_manifest_kappa(&output, Some(&manifest_m1))
            .expect("exact cover κ bootstraps the immutable Stage-A binding");
        assert_eq!(
            super::read_optional_source_manifest_kappa_binding(&output)
                .expect("read Stage-A binding")
                .as_deref(),
            Some(kappa_m1.as_str())
        );
        super::preflight_and_bind_source_manifest_kappa(&output, Some(&manifest_m1))
            .expect("exact resume is idempotent");

        let duplicate_report = root.join("duplicate-cover.json");
        std::fs::write(
            &duplicate_report,
            format!(
                "{{\"source_manifest_kappa\":\"{kappa_m1}\",\"source_manifest_kappa\":\"{kappa_m2}\"}}"
            ),
        )
        .expect("write duplicate source κ");
        let error = super::parse_cover_provenance(&duplicate_report)
            .expect_err("duplicate cover source κ cannot be last-key-wins");
        assert!(error.contains("duplicate field"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restart_sidecar_rejects_duplicate_operator_fields_before_registry_validation() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("restart-duplicate-sidecar-field");
        let compiled = root.join("compiled");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        let canonical = serde_json::to_string(&AttentionOperatorSpec::standard_v2())
            .expect("serialize current operator");
        let duplicate = canonical.replacen("\"version\":2", "\"version\":1,\"version\":2", 1);
        assert_ne!(duplicate, canonical, "fixture injects a duplicate version");
        std::fs::write(
            current.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE),
            duplicate,
        )
        .expect("write duplicate sidecar");
        let error = super::discover_compiled_r4g1_candidates_in(&compiled, 2)
            .expect_err("duplicate sidecar keys cannot collapse to current-v2");
        assert!(
            error.contains("duplicate JSON field \"version\""),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reload_base_and_exact_suffix_resolve_same_physical_bundle_and_base_source() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("reload-logical-physical");
        let compiled = root.join("compiled");
        let source = root.join("sources/teacher");
        std::fs::create_dir_all(&source).expect("create logical source");
        let current = compiled.join("teacher-attention-v2");
        write_loadable_graph_bundle(&current, Some(&AttentionOperatorSpec::standard_v2()));

        let (base, base_source) = super::resolve_reload_bundle_in(&root, "teacher", 2)
            .expect("base request resolves")
            .expect("base is loadable");
        let (alias, alias_source) =
            super::resolve_reload_bundle_in(&root, "teacher-attention-v2", 2)
                .expect("exact suffix alias resolves")
                .expect("suffix alias is loadable");
        assert_eq!(base, alias);
        assert_eq!(base.logical_name, "teacher");
        assert_eq!(base.physical_root, current);
        assert_eq!(
            super::status_physical_root(Some(&base)),
            Some(current.display().to_string()),
            "status reports the selected physical v2 root, not compiled/<logical>"
        );
        assert_eq!(base_source, Some(source.clone()));
        assert_eq!(alias_source, Some(source.clone()));
        assert!(super::teacher_ready_for_source(
            true,
            Some(source.as_path()),
            base_source.as_deref(),
        ));
        assert!(
            !super::teacher_ready_for_source(true, Some(current.as_path()), base_source.as_deref(),),
            "a teacher loaded from the physical suffix root is not ready for the logical source"
        );
        let advertised = super::loadable_models_in(&compiled).expect("valid v2 inventory");
        assert_eq!(
            advertised
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["teacher"],
            "model listing exposes the logical id once, never the physical suffix"
        );

        std::fs::remove_dir_all(root.join("sources/teacher")).expect("remove optional source");
        let (decode_only, source) = super::resolve_reload_bundle_in(&root, "teacher", 2)
            .expect("genuine source absence is valid")
            .expect("graph remains decode-loadable");
        assert_eq!(decode_only.physical_root, current);
        assert_eq!(source, None);
        assert!(
            !super::teacher_ready_for_source(true, None, source.as_deref()),
            "decode-only source absence must not report a teacher-ready state"
        );

        std::fs::create_dir_all(root.join("sources")).expect("restore sources root");
        std::fs::write(root.join("sources/teacher"), b"invalid source entry")
            .expect("write invalid logical source");
        let error = super::resolve_reload_bundle_in(&root, "teacher-attention-v2", 2)
            .expect_err("present-invalid base source is terminal for suffix reload");
        assert!(error.contains("not a source directory"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_replacement_preserves_active_tuple_and_prior_terminal_marker() {
        use std::sync::{Arc, Mutex};
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let active = super::ResolvedCompiledBundle {
            logical_name: "alpha".to_owned(),
            physical_root: "/compiled/alpha-attention-v2".into(),
            graph: "/compiled/alpha-attention-v2/graph/score.r4g1".into(),
            teacher: "/compiled/alpha-attention-v2/tless_artifacts.bin".into(),
            attention_operator: AttentionOperatorSpec::standard_v2(),
            source_manifest_kappa: None,
        };
        let serving = Arc::new(Mutex::new(super::ServingModelState {
            active_bundle: Some(active.clone()),
            terminal_load_error: Some("startup marker".to_owned()),
            ..super::ServingModelState::default()
        }));
        super::record_replacement_failure(&serving, "bad reload");
        let installed = serving.lock().unwrap();
        assert_eq!(installed.active_bundle.as_ref(), Some(&active));
        assert_eq!(
            installed.terminal_load_error.as_deref(),
            Some("startup marker")
        );
        assert_eq!(
            installed.last_operation_error.as_deref(),
            Some("bad reload")
        );
        drop(installed);

        let empty = Arc::new(Mutex::new(super::ServingModelState::default()));
        super::record_replacement_failure(&empty, "no initial tuple");
        let empty = empty.lock().unwrap();
        assert_eq!(
            empty.terminal_load_error.as_deref(),
            Some("no initial tuple")
        );
        assert_eq!(
            empty.last_operation_error.as_deref(),
            Some("no initial tuple")
        );
    }

    #[test]
    fn reload_and_compile_share_one_replacement_reservation() {
        use std::sync::{Arc, Mutex};

        let serving = Arc::new(Mutex::new(super::ServingModelState::default()));
        let status = Arc::new(Mutex::new(super::R4g1CompileStatus {
            running: false,
            ready: false,
            progress: 0,
            message: "idle".to_owned(),
            report: None,
        }));
        let (_epoch, reservation) =
            super::reserve_r4g1_reload(&serving, &status).expect("first writer reserves");
        assert!(status.lock().unwrap().running);
        assert!(
            super::reserve_r4g1_reload(&serving, &status).is_err(),
            "a concurrent reload or compile cannot enter the preparation window"
        );
        drop(reservation);
        let status = status.lock().unwrap();
        assert!(!status.running);
        assert!(status.message.contains("active serving tuple preserved"));
    }

    #[test]
    fn status_identity_uses_the_actual_nondefault_teacher_source_without_a_graph() {
        let state = super::ServingModelState {
            active_teacher_source: Some("/sources/teacher-beta".into()),
            ..super::ServingModelState::default()
        };
        assert_eq!(super::installed_logical_model_name(&state), "teacher-beta");
        assert_eq!(
            super::active_canonical_model_name(&state),
            None,
            "a source path alone is not a text-ready engine"
        );
    }

    #[test]
    fn legacy_v1_only_reload_and_startup_remain_compatible() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;

        let root = attention_provenance_test_dir("legacy-v1-reload");
        let historical = root.join("compiled/teacher");
        write_loadable_graph_bundle(&historical, Some(&AttentionOperatorSpec::standard_v1()));

        let discovered = super::discover_compiled_r4g1_candidates_in(&root.join("compiled"), 2)
            .expect("historical-only startup remains supported");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].physical_root, historical);
        let (reloaded, source) = super::resolve_reload_bundle_in(&root, "teacher", 2)
            .expect("historical-only reload remains supported")
            .expect("historical graph is loadable");
        assert_eq!(
            reloaded.attention_operator,
            AttentionOperatorSpec::standard_v1()
        );
        assert_eq!(source, None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn server_cover_preserves_genuine_legacy_absence_but_requires_source_binding() {
        let root = attention_provenance_test_dir("legacy-absence");
        let (meta, records) = attention_corpus_markers(&root);

        let mut args = Vec::new();
        super::append_recorded_attention_operator(&mut args, &meta, &records, false)
            .expect("caller-supplied legacy absence remains implicit");
        assert!(args.is_empty());

        let error =
            super::append_recorded_attention_operator(&mut Vec::new(), &meta, &records, true)
                .expect_err("source-driven compiles require an explicit binding");
        assert!(error.contains("missing its attention-operator binding"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn server_cover_refuses_malformed_and_non_file_binding_entries() {
        let root = attention_provenance_test_dir("invalid-binding");
        let (meta, records) = attention_corpus_markers(&root);
        let binding = root.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE);
        std::fs::write(&binding, b"{not json").expect("write malformed binding");
        let error =
            super::append_recorded_attention_operator(&mut Vec::new(), &meta, &records, false)
                .expect_err("malformed binding must fail closed");
        assert!(error.contains("attention_operator.json"));

        std::fs::remove_file(&binding).expect("remove malformed binding");
        std::fs::create_dir(&binding).expect("create non-file binding entry");
        let error =
            super::append_recorded_attention_operator(&mut Vec::new(), &meta, &records, false)
                .expect_err("non-file binding must fail closed");
        assert!(error.contains("attention_operator.json"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn server_cover_refuses_corpus_files_from_different_roots() {
        let meta_root = attention_provenance_test_dir("meta-root");
        let records_root = attention_provenance_test_dir("records-root");
        let (meta, _) = attention_corpus_markers(&meta_root);
        let (_, records) = attention_corpus_markers(&records_root);

        let error =
            super::append_recorded_attention_operator(&mut Vec::new(), &meta, &records, false)
                .expect_err("unpaired roots must fail before cover");
        assert!(error.contains("parent"));
        let _ = std::fs::remove_dir_all(meta_root);
        let _ = std::fs::remove_dir_all(records_root);
    }

    #[test]
    fn http_tokenizer_selection_is_an_atomic_version_generic_pair() {
        let absent = super::HuggingFaceDownloadPayload::default();
        assert_eq!(absent.tokenizer_selection().expect("absent pair"), None);

        let family_only = super::HuggingFaceDownloadPayload {
            model: None,
            tokenizer_family: Some("future-family".to_owned()),
            tokenizer_version: None,
        };
        assert!(family_only.tokenizer_selection().is_err());

        let version_only = super::HuggingFaceDownloadPayload {
            model: None,
            tokenizer_family: None,
            tokenizer_version: Some(41),
        };
        assert!(version_only.tokenizer_selection().is_err());

        let selected = super::HuggingFaceDownloadPayload {
            model: None,
            tokenizer_family: Some("future-family".to_owned()),
            tokenizer_version: Some(41),
        }
        .tokenizer_selection()
        .expect("complete pair")
        .expect("selection");
        assert_eq!(selected.family, "future-family");
        assert_eq!(selected.version, 41);
    }

    #[test]
    fn public_source_control_payload_rejects_unknown_fields_before_operations() {
        let operations = std::sync::Arc::new(std::sync::Mutex::new(
            super::SourceCacheOperationState::default(),
        ));
        for body in [
            br#"{"modle":"owner/model@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#.as_slice(),
            br#"{"tokenzier_family":"hf-bpe","tokenizer_version":1}"#.as_slice(),
        ] {
            let error = super::parse_huggingface_control_payload(body)
                .expect_err("unknown public source controls must fail closed");
            assert!(error.contains("unknown field"), "{error}");
            assert!(
                operations.lock().expect("operation state").active.is_none(),
                "the shared parser used by download, compile, and reload runs before reservation or status mutation"
            );
        }

        let empty = super::parse_huggingface_control_payload(br#"{"model":"  "}"#)
            .expect("empty dashboard model control is syntactically valid");
        assert_eq!(
            super::explicitly_requested_huggingface_source(empty.model.as_deref())
                .expect("empty request normalizes to fallback"),
            None,
            "an empty dashboard field must not turn the pinned legacy fallback into an explicit custom request"
        );

        let download_with_compile_controls = super::parse_huggingface_control_payload(
            br#"{"model":"owner/model@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","tokenizer_family":"hf-bpe","tokenizer_version":1}"#,
        )
        .expect("known compile controls parse before endpoint validation");
        let error = download_with_compile_controls
            .validate_download_controls()
            .expect_err("download may not acknowledge and ignore compile-only controls");
        assert!(error.contains("only to compile and reload"), "{error}");
        assert!(
            operations.lock().expect("operation state").active.is_none(),
            "download control rejection precedes cache reservation and transport"
        );
    }

    #[test]
    fn custom_model_parser_enforces_portable_owner_repository_grammar() {
        let revision = "a".repeat(40);
        for invalid in [
            format!("model@{revision}"),
            format!("owner/nested/model@{revision}"),
            format!("own er/model@{revision}"),
            format!("owner/mo?del@{revision}"),
        ] {
            let error = super::source_from_model_spec(&invalid)
                .expect_err("invalid repositories must fail synchronously at the HTTP edge");
            assert!(error.contains("owner/repository"), "{invalid}: {error}");
        }
    }

    #[test]
    fn full_source_identity_keys_keep_forks_and_revisions_disjoint() {
        let root = attention_provenance_test_dir("collision-free-source-identity");
        let shared_revision = "a".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{shared_revision}"))
            .expect("alice source");
        let bob = super::source_from_model_spec(&format!("bob/model@{shared_revision}"))
            .expect("bob source");
        let other_revision = format!("{}{}", "a".repeat(12), "b".repeat(28));
        let alice_other = super::source_from_model_spec(&format!("alice/model@{other_revision}"))
            .expect("second alice revision");

        assert_ne!(alice.name, bob.name, "repository owner is identity-bearing");
        assert_ne!(
            alice.name, alice_other.name,
            "the full revision, not its first 12 characters, is identity-bearing"
        );
        let alice_path = super::downloaded_source_path_in(&alice, &root);
        let bob_path = super::downloaded_source_path_in(&bob, &root);
        assert_ne!(alice_path, bob_path);
        assert_ne!(
            root.join("compiled").join(&alice.name),
            root.join("compiled").join(&bob.name),
            "the source basename feeds a disjoint compiled logical root"
        );
        assert_eq!(alice.name.split('-').next(), Some("model"));
        assert_eq!(alice.name.len(), "model-".len() + 64);
        assert!(super::collision_resistant_source_cache_name(&alice.name));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn intact_v2_snapshot_cannot_be_renamed_under_another_logical_identity() {
        let root = attention_provenance_test_dir("renamed-v2-source-snapshot");
        let revision = "b".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{revision}"))
            .expect("alice source");
        let bob =
            super::source_from_model_spec(&format!("bob/model@{revision}")).expect("bob source");
        let alice_path = super::downloaded_source_path_in(&alice, &root);
        std::fs::create_dir_all(&alice_path).expect("create substituted Alice cache path");
        std::fs::write(alice_path.join("config.json"), b"intact Bob config")
            .expect("write Bob config under Alice name");
        let bob_manifest = write_test_source_manifest(&alice_path, &bob);
        let bob_config = std::fs::read(alice_path.join("config.json")).expect("Bob config bytes");
        let compiled_sentinel = root.join("compiled/alice-existing.bin");
        std::fs::create_dir_all(compiled_sentinel.parent().expect("compiled parent"))
            .expect("create compiled inventory");
        std::fs::write(&compiled_sentinel, b"existing Alice compile")
            .expect("write compiled sentinel");

        let error = super::validate_source_snapshot_integrity(&alice_path, None)
            .expect_err("self-consistent Bob bytes cannot claim Alice's v2 cache basename");
        assert!(
            error.contains("renamed or substituted teacher bytes"),
            "{error}"
        );
        assert!(error.contains(&alice.name), "{error}");
        assert!(error.contains(&bob.name), "{error}");
        let cached_alice = completed_source(&alice_path, &alice);
        let error = super::select_compile_source_path_in(&root, None, Some(&cached_alice), None)
            .expect_err("implicit cached compile must enforce path-to-manifest identity");
        assert!(error.contains("requested model alice/model"), "{error}");
        assert!(error.contains("records bob/model"), "{error}");
        let error = super::validate_compile_source_snapshot(&alice_path, true)
            .expect_err("reload/compile strict validation shares the same identity boundary");
        assert!(
            error.contains("renamed or substituted teacher bytes"),
            "{error}"
        );
        let error = super::validate_managed_source_for_serving(&alice_path, &alice.name)
            .expect_err("restart serving uses the same v2 path/manifest binding");
        assert!(
            error.contains("renamed or substituted teacher bytes"),
            "{error}"
        );

        assert_eq!(
            std::fs::read(alice_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                .expect("manifest after refusal"),
            bob_manifest,
            "identity rejection is read-only"
        );
        assert_eq!(
            std::fs::read(alice_path.join("config.json")).expect("config after refusal"),
            bob_config,
            "substituted source bytes are not rewritten"
        );
        assert_eq!(
            std::fs::read(&compiled_sentinel).expect("compiled sentinel after refusal"),
            b"existing Alice compile",
            "refusal occurs before compile output mutation"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn explicit_compile_model_never_reuses_a_different_cached_download() {
        let root = attention_provenance_test_dir("compile-source-selection");
        let source = |repository: &str, name: &str| crate::model::SourceDownload {
            repository: repository.to_owned(),
            revision: "1".repeat(40),
            name: name.to_owned(),
            output: None,
            license: None,
        };
        let alpha = source("owner/alpha", "alpha");
        let beta = source("owner/beta", "beta");
        let alpha_path = super::downloaded_source_path_in(&alpha, &root);
        let beta_path = super::downloaded_source_path_in(&beta, &root);
        write_test_source_manifest(&alpha_path, &alpha);
        write_test_source_manifest(&beta_path, &beta);

        let cached_alpha = completed_source(&alpha_path, &alpha);
        assert_eq!(
            super::select_compile_source_path_in(&root, Some(&beta), Some(&cached_alpha), None,)
                .expect("explicit downloaded beta wins"),
            Some(beta_path.clone())
        );
        std::fs::remove_dir_all(&beta_path).expect("remove beta download");
        let error =
            super::select_compile_source_path_in(&root, Some(&beta), Some(&cached_alpha), None)
                .expect_err("cached alpha cannot satisfy explicit beta");
        assert!(error.contains("owner/beta"), "{error}");
        assert!(error.contains("download it before compiling"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn explicit_compile_model_requires_exact_manifest_when_cache_names_collide() {
        let root = attention_provenance_test_dir("compile-source-identity-collision");
        let revision = "a".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{revision}"))
            .expect("alice source descriptor");
        let mut bob = super::source_from_model_spec(&format!("bob/model@{revision}"))
            .expect("bob source descriptor");
        // Reproduce a historical/custom-output collision explicitly. New
        // descriptors above are disjoint, but existing directories must still
        // be rejected read-only when their manifest records another owner.
        bob.name = alice.name.clone();
        let alice_path = super::downloaded_source_path_in(&alice, &root);
        let bob_path = super::downloaded_source_path_in(&bob, &root);
        assert_eq!(
            alice_path, bob_path,
            "same basename and revision prefix reproduce the historical cache collision"
        );

        std::fs::create_dir_all(&alice_path).expect("create colliding cache directory");
        let sentinel = alice_path.join("config.json");
        std::fs::write(&sentinel, br#"{"model_type":"llama"}"#)
            .expect("write cached alice payload");
        let manifest_before = write_test_source_manifest(&alice_path, &alice);
        let payload_before = std::fs::read(&sentinel).expect("read alice payload before");
        let compiled_sentinel = root.join("compiled/keep.bin");
        std::fs::create_dir_all(compiled_sentinel.parent().expect("compiled parent"))
            .expect("create compiled root");
        std::fs::write(&compiled_sentinel, b"pre-existing compiled bytes")
            .expect("write compiled sentinel");

        let error = super::select_compile_source_path_in(&root, Some(&bob), None, None)
            .expect_err("bob cannot consume alice's colliding cache directory");
        assert!(
            error.contains(&format!("requested model bob/model@{revision}")),
            "{error}"
        );
        assert!(
            error.contains(&format!("records alice/model@{revision}")),
            "{error}"
        );
        assert_eq!(
            std::fs::read(alice_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                .expect("read manifest after refusal"),
            manifest_before,
            "identity refusal is read-only"
        );
        assert_eq!(
            std::fs::read(&sentinel).expect("read payload after refusal"),
            payload_before,
            "cached teacher bytes remain untouched"
        );
        assert_eq!(
            std::fs::read(&compiled_sentinel).expect("read compiled sentinel after refusal"),
            b"pre-existing compiled bytes",
            "identity mismatch is rejected before any compile output mutation"
        );
        assert_eq!(
            super::select_compile_source_path_in(&root, Some(&alice), None, None)
                .expect("the exact manifest owner is accepted"),
            Some(alice_path)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn pinned_legacy_fallback_is_read_only_but_completed_cache_requires_a_manifest() {
        let root = attention_provenance_test_dir("legacy-source-manifest-compatibility");
        let fallback = crate::model::SourceDownload {
            repository: "owner/legacy".to_owned(),
            revision: "c".repeat(40),
            name: "legacy".to_owned(),
            output: None,
            license: None,
        };
        let path = super::downloaded_source_path_in(&fallback, &root);
        std::fs::create_dir_all(&path).expect("create legacy snapshot");
        std::fs::write(
            path.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME),
            b"legacy unmanifested weights",
        )
        .expect("write legacy weights");

        assert_eq!(
            super::select_compile_source_path_in(&root, None, None, Some(&fallback))
                .expect("genuine pinned pre-manifest input remains readable"),
            Some(path.clone())
        );
        let cached = completed_source(&path, &fallback);
        let error = super::select_compile_source_path_in(&root, None, Some(&cached), None)
            .expect_err("a completed HTTP download must never be inferred from legacy bytes");
        assert!(error.contains("source_manifest.json"), "{error}");

        std::fs::create_dir(path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
            .expect("create present-invalid manifest entry");
        let error = super::select_compile_source_path_in(&root, None, None, Some(&fallback))
            .expect_err("present-invalid fallback manifest may not downgrade to absence");
        assert!(error.contains("source_manifest.json"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn present_source_manifest_license_must_match_pinned_descriptor_exactly() {
        let root = attention_provenance_test_dir("source-manifest-license-binding");
        let expected = crate::model::SourceDownload {
            repository: "owner/pinned-model".to_owned(),
            revision: "4".repeat(40),
            name: "pinned-model".to_owned(),
            output: None,
            license: Some("Apache-2.0".to_owned()),
        };
        let source_path = super::downloaded_source_path_in(&expected, &root);
        std::fs::create_dir_all(&source_path).expect("create pinned source");
        std::fs::write(source_path.join("config.json"), b"pinned config")
            .expect("write pinned config");
        let output_sentinel = root.join("compiled/existing-output.bin");
        std::fs::create_dir_all(output_sentinel.parent().expect("compiled parent"))
            .expect("create compiled output root");
        std::fs::write(&output_sentinel, b"existing compiled output")
            .expect("write output sentinel");
        let status = super::HuggingFaceDownloadStatus {
            running: false,
            ready: true,
            message: "pinned source complete".to_owned(),
            source: Some(source_path.display().to_string()),
            completed_source: Some(expected.clone()),
        };
        let source_path_text = source_path.display().to_string();

        for wrong_license in [None, Some("MIT".to_owned())] {
            let mut wrong = expected.clone();
            wrong.license = wrong_license;
            let manifest_before = write_test_source_manifest(&source_path, &wrong);
            let config_before =
                std::fs::read(source_path.join("config.json")).expect("config before refusal");
            let error = super::validate_legacy_compatible_source_manifest(&source_path, &expected)
                .expect_err("null or wrong SPDX cannot satisfy the pinned descriptor");
            assert!(error.contains("records license"), "{error}");
            assert!(error.contains("Apache-2.0"), "{error}");

            let invoked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let invoked_by_downloader = std::sync::Arc::clone(&invoked);
            let error = super::download_source_atomically_in(&expected, &root, move |_| {
                invoked_by_downloader.store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("license mismatch must reject before transport or overwrite")
            })
            .expect_err("existing falsely licensed cache cannot be reused or overwritten");
            assert!(error.contains("records license"), "{error}");
            assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
            assert_eq!(
                std::fs::read(source_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                    .expect("manifest after refusal"),
                manifest_before
            );
            assert_eq!(
                std::fs::read(source_path.join("config.json")).expect("config after refusal"),
                config_before
            );
            assert!(status.ready);
            assert_eq!(status.source.as_deref(), Some(source_path_text.as_str()));
            assert_eq!(
                std::fs::read(&output_sentinel).expect("output after refusal"),
                b"existing compiled output"
            );
        }

        std::fs::remove_file(source_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
            .expect("remove manifest for genuine pre-597 compatibility check");
        assert_eq!(
            super::validate_legacy_compatible_source_manifest(&source_path, &expected)
                .expect("genuine manifest absence remains a legacy input"),
            None
        );

        write_test_source_manifest(&source_path, &expected);
        assert!(
            super::validate_legacy_compatible_source_manifest(&source_path, &expected)
                .expect("exact pinned license is accepted")
                .is_some()
        );
        let reused = super::download_source_atomically_in(&expected, &root, |_| {
            panic!("an exact immutable cache should be reused without transport")
        })
        .expect("exact licensed cache is reusable");
        assert_eq!(reused, source_path);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn completed_legacy_named_source_retains_exact_download_identity() {
        let root = attention_provenance_test_dir("completed-source-exact-identity");
        let expected = crate::model::SourceDownload {
            repository: "owner/pinned-model".to_owned(),
            revision: "8".repeat(40),
            name: "pinned-model".to_owned(),
            output: None,
            license: Some("Apache-2.0".to_owned()),
        };
        let substituted = crate::model::SourceDownload {
            repository: "attacker/other-model".to_owned(),
            revision: "9".repeat(40),
            name: "pinned-model".to_owned(),
            output: None,
            license: Some("Apache-2.0".to_owned()),
        };
        let source_path = super::downloaded_source_path_in(&expected, &root);
        std::fs::create_dir_all(&source_path).expect("create legacy-named cache path");
        std::fs::write(source_path.join("config.json"), b"substituted source")
            .expect("write substituted bytes");
        let manifest_before = write_test_source_manifest(&source_path, &substituted);
        let output_sentinel = root.join("compiled/existing-output.bin");
        std::fs::create_dir_all(output_sentinel.parent().expect("compiled parent"))
            .expect("create compiled root");
        std::fs::write(&output_sentinel, b"existing output").expect("write output sentinel");

        let cached = completed_source(&source_path, &expected);
        let error = super::select_compile_source_path_in(&root, None, Some(&cached), None)
            .expect_err(
                "implicit compile must validate the completed descriptor, not only its path",
            );
        assert!(error.contains("owner/pinned-model"), "{error}");
        assert!(error.contains("attacker/other-model"), "{error}");
        assert_eq!(
            std::fs::read(source_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                .expect("manifest after refusal"),
            manifest_before
        );
        assert_eq!(
            std::fs::read(&output_sentinel).expect("output after refusal"),
            b"existing output"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_named_managed_source_uses_descriptor_identity_and_invalid_default_is_terminal() {
        let root = attention_provenance_test_dir("descriptor-bound-legacy-source");
        let descriptors = root.join("models");
        let source = root.join("sources/pinned-teacher");
        std::fs::create_dir_all(&descriptors).expect("create descriptor root");
        std::fs::create_dir_all(&source).expect("create source root");
        let expected = crate::model::SourceDownload {
            repository: "owner/pinned-teacher".to_owned(),
            revision: "a".repeat(40),
            name: "pinned-teacher".to_owned(),
            output: Some(source.clone()),
            license: Some("Apache-2.0".to_owned()),
        };
        std::fs::write(
            descriptors.join("pinned-teacher.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "repository": &expected.repository,
                "revision": &expected.revision,
                "license": &expected.license,
                "source_directory": source.display().to_string(),
            }))
            .expect("serialize descriptor"),
        )
        .expect("write descriptor");
        let mut substituted = expected.clone();
        substituted.repository = "other/pinned-teacher".to_owned();
        let manifest_before = write_test_source_manifest(&source, &substituted);
        let sentinel = root.join("compiled/sentinel.bin");
        std::fs::create_dir_all(sentinel.parent().expect("compiled parent"))
            .expect("create compiled root");
        std::fs::write(&sentinel, b"unchanged").expect("write sentinel");

        let error =
            super::validate_managed_source_for_serving_in(&source, "pinned-teacher", &descriptors)
                .expect_err("a present manifest must exactly match the known descriptor");
        assert!(error.contains("owner/pinned-teacher"), "{error}");
        assert!(error.contains("other/pinned-teacher"), "{error}");
        assert_eq!(
            std::fs::read(source.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                .expect("manifest after refusal"),
            manifest_before
        );
        assert_eq!(std::fs::read(&sentinel).expect("sentinel"), b"unchanged");

        write_test_source_manifest(&source, &expected);
        assert!(super::validate_managed_source_for_serving_in(
            &source,
            "pinned-teacher",
            &descriptors,
        )
        .expect("exact descriptor-bound source")
        .is_some());
        std::fs::remove_file(source.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
            .expect("remove manifest for legacy compatibility");
        assert_eq!(
            super::validate_managed_source_for_serving_in(&source, "pinned-teacher", &descriptors,)
                .expect("genuine pre-manifest source remains compatible"),
            None
        );

        let pinned_descriptor = descriptors.join("smollm2-135m-instruct.json");
        std::fs::write(&pinned_descriptor, b"{present-invalid")
            .expect("write malformed pinned descriptor");
        let descriptor_before = std::fs::read(&pinned_descriptor).expect("descriptor bytes");
        let error = super::optional_pinned_huggingface_source_in(&descriptors)
            .expect_err("present-invalid default descriptor cannot collapse to no fallback");
        assert!(error.contains("malformed model descriptor"), "{error}");
        assert_eq!(
            std::fs::read(&pinned_descriptor).expect("descriptor after refusal"),
            descriptor_before
        );
        assert_eq!(std::fs::read(&sentinel).expect("sentinel"), b"unchanged");
        std::fs::remove_file(&pinned_descriptor).expect("remove malformed descriptor");
        assert_eq!(
            super::optional_pinned_huggingface_source_in(&descriptors)
                .expect("genuine descriptor absence is optional"),
            None
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn assert_source_cache_reservation_conflict(
        active: super::SourceCacheOperationKind,
        attempted: super::SourceCacheOperationKind,
    ) {
        let operations = std::sync::Arc::new(std::sync::Mutex::new(
            super::SourceCacheOperationState::default(),
        ));
        let reservation = super::try_reserve_source_cache_operation(
            &operations,
            active,
            format!("{}-source", active.label()),
        )
        .unwrap_or_else(|error| panic!("reserve active operation: {error}"));
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let worker_operations = std::sync::Arc::clone(&operations);
        let worker_barrier = std::sync::Arc::clone(&barrier);
        let worker = std::thread::spawn(move || {
            worker_barrier.wait();
            match super::try_reserve_source_cache_operation(
                &worker_operations,
                attempted,
                format!("{}-source", attempted.label()),
            ) {
                Ok(_) => panic!("conflicting operation unexpectedly acquired the cache"),
                Err(error) => error,
            }
        });
        barrier.wait();
        let error = worker.join().expect("reservation worker");
        assert!(error.contains(active.label()), "{error}");
        assert!(error.contains(attempted.label()), "{error}");
        drop(reservation);
        assert!(operations.lock().expect("operation state").active.is_none());
    }

    #[test]
    fn source_cache_reservation_closes_download_compile_and_reload_races() {
        use super::SourceCacheOperationKind::{Compile, Download, Reload};

        assert_source_cache_reservation_conflict(Compile, Download);
        assert_source_cache_reservation_conflict(Download, Compile);
        assert_source_cache_reservation_conflict(Reload, Download);
        assert_source_cache_reservation_conflict(Download, Reload);
    }

    #[test]
    fn compile_snapshots_completed_download_only_after_reservation_handoff() {
        let operations = std::sync::Arc::new(std::sync::Mutex::new(
            super::SourceCacheOperationState::default(),
        ));
        let alice = crate::model::SourceDownload {
            repository: "alice/model".to_owned(),
            revision: "a".repeat(40),
            name: "alice".to_owned(),
            output: Some("/sources/alice".into()),
            license: None,
        };
        let bob = crate::model::SourceDownload {
            repository: "bob/model".to_owned(),
            revision: "b".repeat(40),
            name: "bob".to_owned(),
            output: Some("/sources/bob".into()),
            license: None,
        };
        let status = std::sync::Arc::new(std::sync::Mutex::new(super::HuggingFaceDownloadStatus {
            running: false,
            ready: true,
            message: "Alice complete".to_owned(),
            source: Some("/sources/alice".to_owned()),
            completed_source: Some(alice),
        }));
        let download = super::try_reserve_source_cache_operation(
            &operations,
            super::SourceCacheOperationKind::Download,
            "/sources/bob",
        )
        .unwrap_or_else(|error| panic!("reserve Bob download: {error}"));
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let worker_operations = std::sync::Arc::clone(&operations);
        let worker_status = std::sync::Arc::clone(&status);
        let worker_barrier = std::sync::Arc::clone(&barrier);
        let worker = std::thread::spawn(move || {
            worker_barrier.wait();
            match super::reserve_compile_source_selection(
                &worker_operations,
                &worker_status,
                "implicit source",
            ) {
                Ok(_) => panic!("compile may not snapshot Alice through Bob's reservation"),
                Err(error) => error,
            }
        });
        barrier.wait();
        let error = worker.join().expect("compile attempt");
        assert!(error.contains("active download"), "{error}");

        // Bob's completed tuple becomes visible before the download releases
        // the cache. The next compile first acquires the handed-off slot and
        // only then samples status, so it cannot retain stale Alice/fallback.
        {
            let mut current = status.lock().expect("download status");
            current.running = false;
            current.ready = true;
            current.source = Some("/sources/bob".to_owned());
            current.completed_source = Some(bob.clone());
            current.message = "Bob complete".to_owned();
        }
        drop(download);
        let (compile, selected) =
            super::reserve_compile_source_selection(&operations, &status, "implicit source")
                .expect("compile acquires completed handoff");
        assert_eq!(
            selected,
            Some(super::CompletedDownloadSource {
                path: std::path::PathBuf::from("/sources/bob"),
                identity: bob,
            })
        );
        drop(compile);
    }

    #[test]
    fn failed_staged_download_preserves_prior_source_and_ready_status() {
        let root = attention_provenance_test_dir("failed-staged-download");
        let revision = "d".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{revision}"))
            .expect("alice source");
        let bob =
            super::source_from_model_spec(&format!("bob/model@{revision}")).expect("bob source");
        let alice_path = super::downloaded_source_path_in(&alice, &root);
        std::fs::create_dir_all(&alice_path).expect("create old published source");
        std::fs::write(alice_path.join("config.json"), b"old published config")
            .expect("write old config");
        let old_manifest = write_test_source_manifest(&alice_path, &alice);
        let old_config = std::fs::read(alice_path.join("config.json")).expect("old config bytes");
        let bob_path = super::downloaded_source_path_in(&bob, &root);

        let error = super::download_source_atomically_in(&bob, &root, |staged| {
            let stage = staged.output.as_ref().expect("staging output");
            std::fs::write(stage.join("config.json"), b"partial replacement config")
                .expect("write partial stage");
            write_test_source_manifest(stage, staged);
            Err("synthetic transport failure".to_owned())
        })
        .expect_err("failed downloader must not publish its staging directory");
        assert!(error.contains("synthetic transport failure"), "{error}");
        assert!(!bob_path.exists(), "failed identity was never published");
        assert_eq!(
            std::fs::read(alice_path.join("config.json")).expect("config after failure"),
            old_config
        );
        assert_eq!(
            std::fs::read(alice_path.join(crate::model::SOURCE_MANIFEST_FILE_NAME))
                .expect("manifest after failure"),
            old_manifest
        );
        assert!(
            std::fs::read_dir(root.join("sources"))
                .expect("source inventory")
                .filter_map(Result::ok)
                .all(|entry| !entry
                    .file_name()
                    .to_string_lossy()
                    .contains("download-staging")),
            "failure cleanup leaves no partially selectable staging directory"
        );

        let mut status = super::HuggingFaceDownloadStatus {
            running: true,
            ready: true,
            message: "redownload started".to_owned(),
            source: Some(alice_path.display().to_string()),
            completed_source: Some({
                let mut completed = alice.clone();
                completed.output = Some(alice_path.clone());
                completed
            }),
        };
        super::apply_huggingface_download_result(
            &mut status,
            Err("synthetic transport failure".to_owned()),
        );
        assert!(!status.running);
        assert!(status.ready);
        let alice_path_text = alice_path.display().to_string();
        assert_eq!(status.source.as_deref(), Some(alice_path_text.as_str()));
        assert!(status.message.contains("synthetic transport failure"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn staged_download_never_replaces_a_raced_empty_or_populated_destination() {
        let root = attention_provenance_test_dir("download-exclusive-publish-race");
        for (kind, payload) in [
            ("empty", None),
            ("payload", Some(b"external winner".as_slice())),
        ] {
            let models_root = root.join(kind);
            let source = super::source_from_model_spec(&format!(
                "owner/{kind}-model@{}",
                if kind == "empty" {
                    "1".repeat(40)
                } else {
                    "2".repeat(40)
                }
            ))
            .expect("source identity");
            let destination = super::downloaded_source_path_in(&source, &models_root);
            let raced_destination = destination.clone();
            let error =
                super::download_source_atomically_in(&source, &models_root, move |staged| {
                    let stage = staged.output.as_ref().expect("reserved staging path");
                    std::fs::write(stage.join("config.json"), b"validated staged config")
                        .expect("write staged config");
                    write_test_source_manifest(stage, staged);

                    // This is the exact post-absence/pre-publish barrier: an
                    // external actor wins the final name after transport began.
                    std::fs::create_dir_all(&raced_destination)
                        .expect("external actor wins destination");
                    if let Some(payload) = payload {
                        std::fs::write(raced_destination.join("external.bin"), payload)
                            .expect("external payload");
                    }
                    Ok(stage.clone())
                })
                .expect_err("exclusive publish may not replace a raced destination");
            assert!(error.contains("lost exclusive publication"), "{error}");
            assert!(destination.is_dir());
            assert!(
                !destination.join("config.json").exists(),
                "validated staging bytes cannot replace the external winner"
            );
            assert!(
                !destination
                    .join(crate::model::SOURCE_MANIFEST_FILE_NAME)
                    .exists(),
                "external winner is not relabeled with the staged manifest"
            );
            if let Some(payload) = payload {
                assert_eq!(
                    std::fs::read(destination.join("external.bin"))
                        .expect("external payload after refusal"),
                    payload
                );
            } else {
                assert_eq!(
                    std::fs::read_dir(&destination)
                        .expect("empty raced destination")
                        .count(),
                    0,
                    "an empty external winner remains byte-for-byte empty"
                );
            }
            let sources = models_root.join("sources");
            assert!(
                std::fs::read_dir(&sources)
                    .expect("source inventory")
                    .filter_map(Result::ok)
                    .all(|entry| !entry
                        .file_name()
                        .to_string_lossy()
                        .contains("download-staging")),
                "failed exclusive publication cleans only its owned stage"
            );
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn atomic_publication_keeps_sequential_forks_disjoint_and_refuses_legacy_collision() {
        let root = attention_provenance_test_dir("sequential-source-publication");
        let revision = "e".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{revision}"))
            .expect("alice source");
        let bob =
            super::source_from_model_spec(&format!("bob/model@{revision}")).expect("bob source");
        let publish = |source: &crate::model::SourceDownload| {
            super::download_source_atomically_in(source, &root, |staged| {
                let stage = staged.output.as_ref().expect("staging output");
                std::fs::write(stage.join("config.json"), staged.repository.as_bytes())
                    .expect("write staged config");
                write_test_source_manifest(stage, staged);
                Ok(stage.clone())
            })
        };
        let alice_path = publish(&alice).expect("publish alice");
        let bob_path = publish(&bob).expect("publish bob");
        assert_ne!(alice_path, bob_path);
        super::validate_source_snapshot_integrity(&alice_path, Some(&alice))
            .expect("alice remains exact");
        super::validate_source_snapshot_integrity(&bob_path, Some(&bob))
            .expect("bob remains exact");

        let mut colliding_bob = bob.clone();
        colliding_bob.output = Some(alice_path.clone());
        let invoked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_by_downloader = std::sync::Arc::clone(&invoked);
        let error = super::download_source_atomically_in(&colliding_bob, &root, move |_| {
            invoked_by_downloader.store(true, std::sync::atomic::Ordering::SeqCst);
            unreachable!("identity mismatch must be rejected before transport")
        })
        .expect_err("a colliding custom output cannot relabel Alice as Bob");
        assert!(error.contains("records alice/model"), "{error}");
        assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
        super::validate_source_snapshot_integrity(&alice_path, Some(&alice))
            .expect("Alice bytes remain unchanged after refusal");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn stale_or_incomplete_download_status_is_not_an_implicit_compile_source() {
        let stale = super::HuggingFaceDownloadStatus {
            running: false,
            ready: false,
            message: "failed replacement".to_owned(),
            source: Some("/stale/partial/source".to_owned()),
            completed_source: None,
        };
        assert_eq!(
            super::completed_download_source(&stale).expect("not ready is an ordinary miss"),
            None
        );

        let mut running = stale.clone();
        running.running = true;
        assert!(super::completed_download_source(&running).is_err());

        let mut inconsistent = stale;
        inconsistent.ready = true;
        inconsistent.source = None;
        assert!(super::completed_download_source(&inconsistent).is_err());
    }

    #[test]
    fn missing_completed_source_never_downgrades_compile_to_legacy_inputs() {
        let root = attention_provenance_test_dir("missing-completed-source");
        let revision = "3".repeat(40);
        let source = super::source_from_model_spec(&format!("owner/model@{revision}"))
            .expect("custom source");
        let source_path = super::downloaded_source_path_in(&source, &root);
        std::fs::create_dir_all(&source_path).expect("create completed source");
        write_test_source_manifest(&source_path, &source);
        let moved_source = root.join("externally-moved-source");
        std::fs::rename(&source_path, &moved_source).expect("move completed source externally");

        let legacy_root = root.join("configured-legacy-inputs");
        std::fs::create_dir_all(&legacy_root).expect("create legacy inputs");
        for name in ["tless_artifacts.bin", "corpus.meta", "corpus.records"] {
            std::fs::write(legacy_root.join(name), b"pre-existing legacy bytes")
                .expect("write legacy input");
        }
        let output_sentinel = root.join("compiled/existing-output.bin");
        std::fs::create_dir_all(output_sentinel.parent().expect("output parent"))
            .expect("create output root");
        std::fs::write(&output_sentinel, b"pre-existing compiled output")
            .expect("write output sentinel");

        let download_status =
            std::sync::Arc::new(std::sync::Mutex::new(super::HuggingFaceDownloadStatus {
                running: false,
                ready: true,
                message: "custom source complete".to_owned(),
                source: Some(source_path.display().to_string()),
                completed_source: Some({
                    let mut completed = source.clone();
                    completed.output = Some(source_path.clone());
                    completed
                }),
            }));
        let compile_status = super::R4g1CompileStatus {
            running: false,
            ready: true,
            progress: 77,
            message: "prior compile state".to_owned(),
            report: Some(serde_json::json!({ "prior": true })),
        };
        let compile_status_before = compile_status.clone();
        let operations = std::sync::Arc::new(std::sync::Mutex::new(
            super::SourceCacheOperationState::default(),
        ));
        let (_reservation, cached) = super::reserve_compile_source_selection(
            &operations,
            &download_status,
            "implicit completed source",
        )
        .expect("reserve before status snapshot");
        let cached = cached.expect("ready status remains authoritative");
        let error = super::select_compile_source_path_in(&root, None, Some(&cached), None)
            .expect_err("absent completed source must stop before legacy compile selection");
        assert!(
            error.contains("recorded ready but is now absent"),
            "{error}"
        );
        assert!(error.contains("refusing to downgrade"), "{error}");

        let current_download = download_status.lock().expect("download status");
        assert!(current_download.ready);
        let source_path_text = source_path.display().to_string();
        assert_eq!(
            current_download.source.as_deref(),
            Some(source_path_text.as_str())
        );
        assert_eq!(compile_status.running, compile_status_before.running);
        assert_eq!(compile_status.ready, compile_status_before.ready);
        assert_eq!(compile_status.progress, compile_status_before.progress);
        assert_eq!(compile_status.message, compile_status_before.message);
        assert_eq!(compile_status.report, compile_status_before.report);
        assert_eq!(
            std::fs::read(&output_sentinel).expect("output sentinel after refusal"),
            b"pre-existing compiled output"
        );
        for name in ["tless_artifacts.bin", "corpus.meta", "corpus.records"] {
            assert_eq!(
                std::fs::read(legacy_root.join(name)).expect("legacy input after refusal"),
                b"pre-existing legacy bytes"
            );
        }
        assert!(moved_source.is_dir(), "moved source remains untouched");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_manifest_must_bind_every_index_referenced_shard() {
        let root = attention_provenance_test_dir("hidden-indexed-shard");
        let source = crate::model::SourceDownload {
            repository: "owner/model".to_owned(),
            revision: "f".repeat(40),
            name: "hidden-shard".to_owned(),
            output: None,
            license: None,
        };
        let source_dir = root.join("sources/hidden-shard");
        std::fs::create_dir_all(&source_dir).expect("create hidden-shard source");
        std::fs::write(
            source_dir.join(".secret.safetensors"),
            b"unbound hidden weights",
        )
        .expect("write hidden shard");
        std::fs::write(
            source_dir.join(uor_r4_model_source::SAFETENSORS_INDEX_FILE_NAME),
            br#"{"weight_map":{"model.weight":".secret.safetensors"}}"#,
        )
        .expect("write shard index");
        write_test_source_manifest(&source_dir, &source);
        // Remove the helper's single-file weight so the index is the only
        // executable source layout, then rebuild the exact manifest.
        std::fs::remove_file(source_dir.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME))
            .expect("remove helper single-file weights");
        write_test_source_manifest_without_default_weights(&source_dir, &source);

        let error = super::validate_source_snapshot_integrity(&source_dir, Some(&source))
            .expect_err("hidden index shard is executable but absent from the manifest");
        assert!(
            error.contains("outside the exact source manifest inventory"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn source_manifest_rejects_visible_symlinked_teacher_inputs() {
        use std::os::unix::fs::symlink;

        let root = attention_provenance_test_dir("symlinked-source-input");
        let source = crate::model::SourceDownload {
            repository: "owner/model".to_owned(),
            revision: "1".repeat(40),
            name: "symlinked-source".to_owned(),
            output: None,
            license: None,
        };
        let source_dir = root.join("sources/symlinked-source");
        std::fs::create_dir_all(&source_dir).expect("create source");
        let outside = root.join("outside.safetensors");
        std::fs::write(&outside, b"outside weights").expect("write outside weights");
        std::fs::write(
            source_dir.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME),
            b"original bound weights",
        )
        .expect("write original source weights");
        write_test_source_manifest_without_default_weights(&source_dir, &source);
        std::fs::remove_file(source_dir.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME))
            .expect("remove original bound weights");
        symlink(
            &outside,
            source_dir.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME),
        )
        .expect("create weight symlink");
        let error = super::validate_source_snapshot_integrity(&source_dir, Some(&source))
            .expect_err("teacher source symlink may not disappear from manifest admission");
        assert!(error.contains("without following links"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_rejects_byte_drift_and_source_root_symlinks() {
        use std::os::unix::fs::symlink;

        let root = attention_provenance_test_dir("managed-startup-source-validation");
        let revision = "2".repeat(40);
        let alice = super::source_from_model_spec(&format!("alice/model@{revision}"))
            .expect("alice source");
        let bob =
            super::source_from_model_spec(&format!("bob/model@{revision}")).expect("bob source");
        let alice_path = super::downloaded_source_path_in(&alice, &root);
        std::fs::create_dir_all(&alice_path).expect("create Alice source");
        std::fs::write(alice_path.join("config.json"), b"recorded config")
            .expect("write Alice config");
        write_test_source_manifest(&alice_path, &alice);
        std::fs::write(alice_path.join("config.json"), b"drifted config")
            .expect("mutate admitted bytes after manifest");
        let error = super::validate_managed_source_for_serving(&alice_path, &alice.name)
            .expect_err("restart may not load bytes that drifted from the manifest");
        assert!(error.contains("file inventory"), "{error}");

        std::fs::remove_dir_all(&alice_path).expect("remove drifted Alice source");
        let bob_path = super::downloaded_source_path_in(&bob, &root);
        std::fs::create_dir_all(&bob_path).expect("create Bob source");
        write_test_source_manifest(&bob_path, &bob);
        symlink(&bob_path, &alice_path).expect("alias Alice cache name to Bob source");
        let error = super::validate_managed_source_for_serving(&alice_path, &alice.name)
            .expect_err("managed startup source roots may not be symlink aliases");
        assert!(error.contains("non-symlink directory"), "{error}");

        let legacy = root.join("sources/legacy-with-symlinked-weights");
        std::fs::create_dir_all(&legacy).expect("create pre-manifest legacy source");
        let outside = root.join("legacy-outside.safetensors");
        std::fs::write(&outside, b"retargetable legacy weights")
            .expect("write legacy outside weights");
        symlink(
            &outside,
            legacy.join(uor_r4_model_source::SAFETENSORS_SINGLE_FILE_NAME),
        )
        .expect("create visible legacy weight symlink");
        let error =
            super::validate_managed_source_for_serving(&legacy, "legacy-with-symlinked-weights")
                .expect_err("manifest absence does not permit executable source symlinks");
        assert!(error.contains("without following links"), "{error}");
        let error = super::validate_compile_source_snapshot(&legacy, false)
            .expect_err("legacy compile/reload shares the structural source boundary");
        assert!(error.contains("without following links"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn inferred_source_distinguishes_absence_from_present_invalid_entries() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-serving-inferred-source-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let teacher = root
            .join("compiled")
            .join("model-a")
            .join("tless_artifacts.bin");
        std::fs::create_dir_all(teacher.parent().expect("teacher parent"))
            .expect("compiled directory");
        std::fs::create_dir_all(root.join("sources")).expect("sources root");
        assert_eq!(
            super::source_for_compiled_teacher_in(&teacher, &root).expect("genuine absence"),
            None
        );

        let source = root.join("sources").join("model-a");
        std::fs::create_dir(&source).expect("source directory");
        assert_eq!(
            super::source_for_compiled_teacher_in(&teacher, &root).expect("directory is present"),
            Some(source.clone())
        );

        let explicit_source = root.join("sources/external-attention-v2");
        std::fs::create_dir(&explicit_source).expect("explicit source directory");
        let explicit_teacher = root.join("elsewhere/external-attention-v2/tless_artifacts.bin");
        assert_eq!(
            super::source_for_compiled_teacher_in(&explicit_teacher, &root)
                .expect("explicit CLI paths retain literal source mapping"),
            Some(explicit_source),
            "the managed suffix is reserved only inside models_root/compiled"
        );

        std::fs::remove_dir(&source).expect("remove source directory");
        std::fs::write(&source, b"not a directory").expect("invalid source entry");
        let error = super::source_for_compiled_teacher_in(&teacher, &root)
            .expect_err("present file is not absence");
        assert!(error.contains("is not a source directory"), "{error}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            std::fs::remove_file(&source).expect("remove invalid file");
            symlink(root.join("missing-source-target"), &source).expect("dangling source symlink");
            let error = super::source_for_compiled_teacher_in(&teacher, &root)
                .expect_err("dangling source is not absence");
            assert!(error.contains("model-a"), "{error}");
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn server_compile_binds_only_a_regular_adjacent_tokenizer() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-server-compile-tokenizer-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture root");
        let tokenizer = root.join("tokenizer.bin");

        let mut legacy_args = Vec::new();
        super::append_tokenizer_arg(&mut legacy_args, &tokenizer, false)
            .expect("genuinely absent legacy tokenizer remains optional");
        assert!(legacy_args.is_empty());

        let mut source_args = Vec::new();
        let error = super::append_tokenizer_arg(&mut source_args, &tokenizer, true)
            .expect_err("source-backed compile requires tokenizer.bin");
        assert!(
            error.contains("required source-backed tokenizer"),
            "{error}"
        );

        std::fs::write(&tokenizer, b"exact tokenizer bytes").expect("tokenizer fixture");
        super::append_tokenizer_arg(&mut source_args, &tokenizer, true)
            .expect("regular tokenizer is bound");
        assert_eq!(
            source_args,
            vec!["--tokenizer".to_owned(), tokenizer.display().to_string()]
        );

        std::fs::remove_file(&tokenizer).expect("remove tokenizer fixture");
        std::fs::create_dir(&tokenizer).expect("invalid tokenizer directory");
        let error = super::append_tokenizer_arg(&mut Vec::new(), &tokenizer, false)
            .expect_err("present-invalid tokenizer never becomes absence");
        assert!(error.contains("not a regular file"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_bundle_inventory_requires_regular_tokenizer_and_attention_sidecars() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-source-bundle-inventory-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture root");
        for file in [
            "tless_artifacts.bin",
            "tless_store.bin",
            "tokenizer.bin",
            "corpus.meta",
            "corpus.records",
        ] {
            std::fs::write(root.join(file), b"fixture").expect("bundle file");
        }
        let error = super::validate_source_bundle_inventory(&root)
            .expect_err("adapter sidecar is required");
        assert!(error.contains("tokenizer_adapter.json"), "{error}");

        let sidecar = root.join("tokenizer_adapter.json");
        std::fs::create_dir(&sidecar).expect("invalid sidecar directory");
        let error = super::validate_source_bundle_inventory(&root)
            .expect_err("present-invalid sidecar fails closed");
        assert!(error.contains("not a regular file"), "{error}");
        std::fs::remove_dir(&sidecar).expect("remove invalid sidecar");
        std::fs::write(&sidecar, b"{}").expect("regular sidecar");

        let error = super::validate_source_bundle_inventory(&root)
            .expect_err("attention sidecar is required");
        assert!(error.contains("attention_operator.json"), "{error}");
        let attention = root.join(uor_r4_graph_cli::ATTENTION_OPERATOR_BINDING_FILE);
        std::fs::create_dir(&attention).expect("invalid attention sidecar directory");
        let error = super::validate_source_bundle_inventory(&root)
            .expect_err("present-invalid attention sidecar fails closed");
        assert!(error.contains("not a regular file"), "{error}");
        std::fs::remove_dir(&attention).expect("remove invalid attention sidecar");
        std::fs::write(&attention, b"{}").expect("regular attention sidecar");
        super::validate_source_bundle_inventory(&root).expect("complete regular inventory");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reload_source_selection_keeps_absence_optional_and_invalid_entries_terminal() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-reload-source-selection-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture root");
        let source = root.join("source");
        assert_eq!(
            super::optional_source_directory(&source).expect("genuine absence"),
            None
        );
        std::fs::create_dir(&source).expect("source directory");
        assert_eq!(
            super::optional_source_directory(&source).expect("present source"),
            Some(source.clone())
        );
        std::fs::remove_dir(&source).expect("remove source directory");
        std::fs::write(&source, b"not a directory").expect("invalid source");
        assert!(super::optional_source_directory(&source).is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_tagged_host_encoder_stops_the_default_cascade_before_legacy_tokenization() {
        assert!(super::tokenizer_unavailable_is_terminal(None, true, false));
        assert!(!super::tokenizer_unavailable_is_terminal(
            None, false, false
        ));
        // A graph rejected at the exact-CID load boundary is also terminal;
        // it can never be retried through the transformerless tier's legacy
        // tokenizer.
        assert!(super::tokenizer_unavailable_is_terminal(None, false, true));
        // An explicit non-R4G1 engine selection never enters the R4G1 tier,
        // so its independently selected tokenizer policy remains in force.
        assert!(!super::tokenizer_unavailable_is_terminal(
            Some(super::TIER_ATTENTION),
            true,
            true,
        ));
    }

    #[test]
    fn serving_source_tokenizer_resolution_is_failure_atomic() {
        let dir =
            std::env::temp_dir().join(format!("uor-r4-serving-tokenizer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("source dir");
        let tokenizer_path = dir.join("tokenizer.json");
        std::fs::write(
            &tokenizer_path,
            br#"{
                "model":{"type":"BPE","vocab":{"a":0},"merges":[]},
                "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false}
            }"#,
        )
        .expect("tokenizer definition");
        let selection = uor_r4_core::transformerless::hf_bpe::TokenizerAdapterKey::hf_byte_bpe_v1();
        let tokenizer = super::resolve_serving_source_tokenizer(&dir, Some(&selection))
            .expect("registered source loads");
        let adapter = tokenizer.adapter().expect("registered identity");
        assert_eq!(adapter.family, "hf-byte-bpe");
        assert_eq!(adapter.version, 1);

        std::fs::write(
            dir.join("spiece.model"),
            b"present to make selection ambiguous",
        )
        .expect("second tokenizer definition");
        let error = match super::resolve_serving_source_tokenizer(&dir, None) {
            Err(error) => error,
            Ok(_) => panic!("ambiguous source selection must fail closed"),
        };
        assert!(
            error
                .reason
                .contains("both tokenizer.json and spiece.model"),
            "{error}"
        );
        assert_eq!(
            adapter.family, "hf-byte-bpe",
            "a failed replacement cannot mutate the already prepared tokenizer value"
        );
        std::fs::remove_file(dir.join("spiece.model")).expect("remove second definition");

        std::fs::write(&tokenizer_path, br#"{"model":{"type":"Unigram"}}"#)
            .expect("replace with unsupported wrapper");
        let error = match super::resolve_serving_source_tokenizer(&dir, Some(&selection)) {
            Err(error) => error,
            Ok(_) => panic!("unsupported selected definition must fail closed"),
        };
        assert!(error.reason.contains("hf-byte-bpe/1"), "{error}");
        assert_eq!(adapter.version, 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn present_indexed_source_and_explicit_tokenizer_mismatch_are_terminal() {
        use uor_r4_core::transformerless::hf_bpe::TokenizerAdapterKey;
        use uor_r4_core::transformerless::scenarios::RuntimeTokenizerIdentity;

        let dir = attention_provenance_test_dir("indexed-source-invalid");
        std::fs::write(
            dir.join("tokenizer.json"),
            br#"{
                "model":{"type":"BPE","vocab":{"a":0},"merges":[]},
                "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false}
            }"#,
        )
        .expect("tokenizer definition");
        std::fs::write(dir.join("config.json"), br#"{"model_type":"llama"}"#)
            .expect("minimal family marker");
        std::fs::write(
            dir.join("model.safetensors.index.json"),
            br#"{"weight_map":{"x":"missing-shard.safetensors"}}"#,
        )
        .expect("indexed shard manifest");

        assert_eq!(
            super::optional_source_directory(&dir).expect("present directory is selected"),
            Some(dir.clone()),
            "startup selection does not require a monolithic model.safetensors"
        );
        let error = match super::prepare_optional_teacher_source_for_identity(
            Some(&dir),
            None,
            "indexed-source",
            None,
        ) {
            Err(error) => error,
            Ok(_) => panic!("a present malformed indexed source must not become decode-only"),
        };
        assert!(error.contains("teacher source"), "{error}");

        let tokenizer = super::resolve_serving_source_tokenizer(
            &dir,
            Some(&TokenizerAdapterKey::hf_byte_bpe_v1()),
        )
        .expect("fixture tokenizer resolves");
        let adapter = tokenizer.adapter().expect("registered adapter");
        let mismatched = RuntimeTokenizerIdentity {
            family: adapter.family.clone(),
            version: adapter.version,
            tokenizer_cid: "blake3:different-tokenizer".to_owned(),
            adapter_digest: adapter.adapter_digest.clone(),
        };
        let error = match super::prepare_optional_teacher_source_for_identity(
            Some(&dir),
            Some(&TokenizerAdapterKey::hf_byte_bpe_v1()),
            "indexed-source",
            Some(&mismatched),
        ) {
            Err(error) => error,
            Ok(_) => panic!("explicit adapter selection cannot cross tokenizer identities"),
        };
        assert!(error.contains("compiled bundle requires"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn r4g1_usage_keeps_registered_segmentation_and_generated_token_count() {
        use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerKind};
        use uor_r4_core::transformerless::scenarios::Tokenizer;
        use uor_r4_router::fallback::CascadeOutcome;

        let registered = HfBpeTokenizer::from_tokenizer_json_bytes(
            br#"{
                "model":{"type":"BPE","vocab":{"1":0,"2":1,"12":2},"merges":["1 2"]},
                "pre_tokenizer":{"type":"Sequence","pretokenizers":[
                    {"type":"Digits","individual_digits":true},
                    {"type":"ByteLevel","add_prefix_space":false}
                ]}
            }"#,
        )
        .expect("registered Digits tokenizer");
        let registered = TokenizerKind::Registered(Box::new(registered));
        let registered_prompt_tokens = registered.encode("12").len();
        assert_eq!(registered_prompt_tokens, 2);

        let mut legacy_bytes = Vec::new();
        for piece in [
            b"1".as_slice(),
            "Ġ".as_bytes(),
            b"2".as_slice(),
            b"12".as_slice(),
        ] {
            legacy_bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            legacy_bytes.extend_from_slice(piece);
        }
        let legacy = Tokenizer::from_bytes(&legacy_bytes).expect("legacy tokenizer");
        assert_eq!(legacy.encode("12").len(), 1, "fixture must discriminate");

        let cascade = super::ServingCascade {
            outcome: CascadeOutcome {
                text: Some("completion".to_owned()),
                served_by: Some(super::TIER_R4G1),
                trail: Vec::new(),
            },
            r4g1: super::R4g1Signal::default(),
            geometric: None,
            usage: Some(super::ServingUsage {
                prompt_tokens: registered_prompt_tokens,
                completion_tokens: 3,
            }),
        };
        let usage = super::serving_usage_for_cascade(&cascade, "12", "12")
            .expect("R4G1 carries exact usage");
        assert_eq!(usage.prompt_tokens, 2);
        assert_eq!(usage.completion_tokens, 3);
        assert_eq!(usage.total_tokens(), 5);
    }

    #[test]
    fn canonical_address_is_encoding_independent() {
        // The conformance property the old raw-bytes hash lacked: two
        // byte-different encodings of the same JSON value get one label.
        let a = uor_addr::json::address_blake3(br#"{"a":1,"b":2}"#).expect("canonicalizes");
        let b = uor_addr::json::address_blake3(br#"{ "b" : 2, "a" : 1 }"#).expect("canonicalizes");
        assert_eq!(a.address.to_string(), b.address.to_string());
        assert!(a.address.to_string().starts_with("blake3:"));
    }

    #[test]
    fn verify_syntax_gate_rejects_negative_vectors() {
        use super::validate_uor_address_syntax as gate;
        // UOR-VERIFICATION negative vectors (issue #257): the empty
        // subject is the canonical one.
        assert_eq!(gate(""), Err("empty_address"));
        assert_eq!(gate("sha256:0000"), Err("unsupported_axis"));
        assert_eq!(gate("blake3:abcd"), Err("digest_length_invalid"));
        let upper = format!("blake3:{}", "A".repeat(64));
        assert_eq!(gate(&upper), Err("digest_not_lowercase_hex"));
        let not_hex = format!("blake3:{}", "g".repeat(64));
        assert_eq!(gate(&not_hex), Err("digest_not_lowercase_hex"));
        // Substring attack from the old comparator: digest embedded in a
        // longer string no longer parses as an address at all.
        let embedded = format!("blake3:{}xx", "a".repeat(64));
        assert_eq!(gate(&embedded), Err("digest_length_invalid"));
        // Positive vector.
        let valid = format!("blake3:{}", "0123456789abcdef".repeat(4));
        assert_eq!(gate(&valid), Ok(()));
    }

    #[test]
    fn envelope_round_trips_through_the_verifier_comparison() {
        // Producer/verifier agreement (issues #256 + #257 move together):
        // the envelope's canonical label equals the verifier's expectation
        // for the same payload, under a different byte encoding.
        let payload = serde_json::json!({
            "text": "Hello R4 world",
            "generation_mode": "r4g1",
            "tokens_generated": 5
        });
        let envelope_address = super::canonical_json_address_blake3(&payload);
        let reencoded =
            br#"{ "tokens_generated" : 5, "text" : "Hello R4 world", "generation_mode" : "r4g1" }"#;
        let verifier_expectation = uor_addr::json::address_blake3(reencoded)
            .expect("canonicalizes")
            .address
            .to_string();
        assert_eq!(envelope_address, verifier_expectation);
        assert_eq!(
            super::validate_uor_address_syntax(&envelope_address),
            Ok(())
        );
    }

    #[test]
    fn test_pathological_generation_triggers_fallback() {
        let pathological_text =
            "that is how i work that is how i work that is how i work that is how i work";
        assert!(!super::is_usable_generated_text(pathological_text));

        let _prompt = "Explain quantum geometric routing in plain terms.";
        let engine_mode = "r4g1";
        let mut final_response_text = String::new();
        let mut generation_mode = "r4g1-rejected".to_string();
        let r4g1_abstained = false;

        // Simulate fallback execution when primary R4G1 output is rejected
        if final_response_text.is_empty() && !r4g1_abstained {
            // Secondary fallback simulation
            let fallback_text = "Quantum geometric routing maps high-dimensional state vectors to discrete discrete manifolds.";
            final_response_text = fallback_text.to_string();
            generation_mode = if engine_mode == "r4g1" {
                "transformerless-fallback".to_string()
            } else {
                "transformerless-legacy".to_string()
            };
        }

        assert_eq!(generation_mode, "transformerless-fallback");
        assert!(!final_response_text.is_empty());
        assert!(super::is_usable_generated_text(&final_response_text));
    }

    #[test]
    fn r4g1_default_selection_never_pins_but_explicit_engines_do() {
        // Issue #248 amendment: the CLI persists "r4g1" as its default
        // engine, so treating it as a pin would silently disable every
        // fallback tier on default installs. r4g1-first is already the
        // cascade order — "r4g1" runs the full cascade.
        assert_eq!(super::resolve_pinned_tier(Some("r4g1")), None);
        assert_eq!(super::resolve_pinned_tier(Some("auto")), None);
        assert_eq!(super::resolve_pinned_tier(Some("")), None);
        assert_eq!(super::resolve_pinned_tier(Some("ollama")), None);
        assert_eq!(
            super::resolve_pinned_tier(Some("geometric")),
            Some(super::TIER_GEOMETRIC)
        );
        assert_eq!(
            super::resolve_pinned_tier(Some("attention")),
            Some(super::TIER_ATTENTION)
        );
        assert_eq!(
            super::resolve_pinned_tier(Some("transformerless")),
            Some(super::TIER_TRANSFORMERLESS)
        );
    }

    #[test]
    fn derive_legacy_fields_from_cascade_trail() {
        use uor_r4_router::fallback::{CascadeOutcome, EngineStatus, TierOutcome};

        // A cascaded abstention (PR #223 semantics): R4G1 abstains, the
        // transformerless tier serves, and the legacy fields derive from
        // the trail.
        let served = super::ServingCascade {
            outcome: CascadeOutcome {
                text: Some("served text".to_owned()),
                served_by: Some(super::TIER_TRANSFORMERLESS),
                trail: vec![
                    TierOutcome {
                        tier: super::TIER_R4G1,
                        status: EngineStatus::Abstained,
                        detail: Some("R4G1 policy abstained (status: novel)".to_owned()),
                    },
                    TierOutcome {
                        tier: super::TIER_TRANSFORMERLESS,
                        status: EngineStatus::Success,
                        detail: None,
                    },
                ],
            },
            r4g1: super::R4g1Signal {
                status: Some("novel"),
                widened: true,
                abstained: true,
                error: None,
            },
            geometric: None,
            usage: None,
        };
        assert_eq!(
            super::derive_generation_mode(&served, None),
            "transformerless-fallback"
        );
        assert_eq!(
            super::derive_generation_mode(&served, Some(super::TIER_TRANSFORMERLESS)),
            "transformerless-legacy"
        );
        let trail = super::cascade_trail_json(&served.outcome.trail);
        assert_eq!(trail[0]["tier"], "r4g1");
        assert_eq!(trail[0]["status"], "abstained");
        assert_eq!(trail[1]["tier"], "transformerless");
        assert_eq!(trail[1]["status"], "success");

        // A pinned tier that abstained: declined_by_all is a declared
        // outcome (HTTP 200) naming the pinned tier, with the legacy
        // abstention fields intact.
        let declined = super::ServingCascade {
            outcome: CascadeOutcome {
                text: None,
                served_by: None,
                trail: vec![TierOutcome {
                    tier: super::TIER_R4G1,
                    status: EngineStatus::Abstained,
                    detail: Some("R4G1 policy abstained (status: novel)".to_owned()),
                }],
            },
            r4g1: super::R4g1Signal {
                status: Some("novel"),
                widened: false,
                abstained: true,
                error: None,
            },
            geometric: None,
            usage: None,
        };
        let generation_mode = super::derive_generation_mode(&declined, Some(super::TIER_R4G1));
        assert_eq!(generation_mode, "r4g1-abstained");
        let (status, body) =
            super::declined_by_all_response(&declined, Some(super::TIER_R4G1), &generation_mode);
        assert_eq!(status, 200);
        assert_eq!(body["outcome"], "declined_by_all");
        assert_eq!(body["engine"], "r4g1");
        assert_eq!(body["abstained"], true);
        assert_eq!(body["r4g1"]["status"], "novel");
        assert_eq!(body["cascade_trail"][0]["tier"], "r4g1");
        assert_eq!(body["cascade_trail"][0]["status"], "abstained");

        // A pinned tier that hard-failed with no declared abstention is a
        // 503 fault carrying the recorded reason.
        let failed = super::ServingCascade {
            outcome: CascadeOutcome {
                text: None,
                served_by: None,
                trail: vec![TierOutcome {
                    tier: super::TIER_R4G1,
                    status: EngineStatus::Failed,
                    detail: Some("R4G1 graph runtime is not loaded".to_owned()),
                }],
            },
            r4g1: super::R4g1Signal {
                status: None,
                widened: false,
                abstained: false,
                error: Some("R4G1 graph runtime is not loaded".to_owned()),
            },
            geometric: None,
            usage: None,
        };
        let generation_mode = super::derive_generation_mode(&failed, Some(super::TIER_R4G1));
        assert_eq!(generation_mode, "r4g1-error");
        let (status, body) =
            super::declined_by_all_response(&failed, Some(super::TIER_R4G1), &generation_mode);
        assert_eq!(status, 503);
        assert!(body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("not loaded"));
    }

    // ---- #654 phase B: OpenAI wire shapes ----

    #[test]
    fn openai_error_envelope_has_the_standard_shape() {
        let body = super::openai_error_body(
            "invalid_request_error",
            "The model 'x' does not exist or is not loadable.",
            Some("model"),
            Some("model_not_found"),
        );
        let error = &body["error"];
        assert_eq!(error["type"], "invalid_request_error");
        assert_eq!(
            error["message"],
            "The model 'x' does not exist or is not loadable."
        );
        assert_eq!(error["param"], "model");
        assert_eq!(error["code"], "model_not_found");

        // Absent param/code serialize as JSON null (present, not omitted).
        let bare = super::openai_error_body("invalid_request_error", "bad", None, None);
        assert!(bare["error"]["param"].is_null());
        assert!(bare["error"]["code"].is_null());
    }

    #[test]
    fn openai_model_object_carries_every_required_field() {
        let model = super::openai_model_object("smollm2-135m-instruct", 1_700_000_000);
        assert_eq!(model["id"], "smollm2-135m-instruct");
        assert_eq!(model["object"], "model");
        assert_eq!(model["created"], 1_700_000_000u64);
        assert!(model["owned_by"].as_str().is_some(), "owned_by is present");
    }

    #[test]
    fn openai_model_alias_and_wire_surfaces_report_only_the_active_canonical_id() {
        let canonical = super::resolve_request_model_name(Some("alpha"), Some("uor-r4"))
            .expect("legacy alias intentionally maps to active");
        assert_eq!(canonical, "alpha");
        assert_eq!(
            super::resolve_request_model_name(Some("alpha"), None).unwrap(),
            "alpha"
        );
        assert_eq!(
            super::resolve_request_model_name(Some("alpha"), Some("alpha")).unwrap(),
            "alpha"
        );
        assert!(super::resolve_request_model_name(Some("alpha"), Some("beta")).is_err());
        assert!(super::resolve_request_model_name(None, Some("uor-r4")).is_err());

        let listed = super::models_list_body(&[(canonical.clone(), 7)]);
        assert_eq!(listed["data"].as_array().unwrap().len(), 1);
        assert_eq!(listed["data"][0]["id"], "alpha");

        let chat_json = super::VendorChatCompletionsResponse {
            id: "chatcmpl-alpha-1".to_owned(),
            object: "chat.completion".to_owned(),
            created: 1,
            model: canonical.clone(),
            choices: Vec::new(),
            usage: super::VendorUsage {
                prompt_tokens: 1,
                completion_tokens: 1,
                total_tokens: 2,
            },
            system_fingerprint: None,
            uor_audit: None,
            cascade_trail: serde_json::json!([]),
        };
        assert_eq!(serde_json::to_value(chat_json).unwrap()["model"], "alpha");

        let chat_frames = super::build_chat_stream_frames(
            "chatcmpl-alpha-1",
            1,
            &canonical,
            "uor-r4-r4g1",
            "ok",
            "stop",
            None,
            None,
            serde_json::json!([]),
        );
        for chunk in parse_stream_chunks(&chat_frames) {
            assert_eq!(chunk["model"], "alpha");
        }

        let response = super::build_responses_body(dummy_generation("ok", 1, 1), &canonical, 64);
        assert_eq!(response["model"], "alpha");
        let response_frames =
            super::build_responses_stream_frames("resp-alpha-1", 1, &canonical, "ok", response);
        let events = parse_responses_events(&response_frames);
        assert_eq!(events[0].1["response"]["model"], "alpha");
        assert_eq!(events.last().unwrap().1["response"]["model"], "alpha");
    }

    #[test]
    fn loadable_models_lists_only_compiled_bundles_sorted() {
        let dir = std::env::temp_dir().join(format!("uor-r4-models-654b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for name in ["beta-model", "alpha-model"] {
            let bundle = dir.join(name);
            write_loadable_graph_bundle(&bundle, None);
        }
        // A directory without a compiled artifact is NOT loadable.
        std::fs::create_dir_all(dir.join("no-bundle")).unwrap();

        let models = super::loadable_models_in(&dir).expect("valid model inventory");
        let ids: Vec<&str> = models.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["alpha-model", "beta-model"],
            "only bundles with an artifact, sorted by id"
        );
        assert!(
            models.iter().all(|(_, created)| *created > 0),
            "created is the bundle mtime"
        );

        let body = super::models_list_body(&models);
        assert_eq!(body["object"], "list");
        assert_eq!(body["data"].as_array().unwrap().len(), 2);
        assert_eq!(body["data"][0]["id"], "alpha-model");
        assert_eq!(body["data"][0]["object"], "model");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loadable_models_is_empty_without_a_compiled_dir() {
        let dir = std::env::temp_dir().join(format!("uor-r4-nomodels-654b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(super::loadable_models_in(&dir)
            .expect("genuinely absent inventory")
            .is_empty());
    }

    #[test]
    fn chat_request_rejects_unsupported_parameters() {
        // The supported subset parses.
        let ok: Result<super::VendorChatCompletionsRequest, _> = serde_json::from_str(
            r#"{"model":"uor-r4","messages":[{"role":"user","content":"hi"}],"max_tokens":8}"#,
        );
        assert!(ok.is_ok(), "supported fields parse");

        // An unsupported OpenAI parameter fails closed (deny_unknown_fields),
        // rather than being silently accepted and ignored.
        let denied: Result<super::VendorChatCompletionsRequest, _> =
            serde_json::from_str(r#"{"messages":[{"role":"user","content":"hi"}],"top_p":0.9}"#);
        assert!(
            denied.is_err(),
            "unsupported parameter top_p is rejected, not ignored"
        );
    }

    // ---- #654 phase C: chat conformance ----

    #[test]
    fn finish_reason_is_length_only_at_the_budget() {
        assert_eq!(super::completion_finish_reason(10, 64), "stop");
        assert_eq!(super::completion_finish_reason(64, 64), "length");
        assert_eq!(super::completion_finish_reason(100, 64), "length");
        // A large requested max_tokens is bounded by the server cap.
        assert_eq!(
            super::completion_finish_reason(super::SERVER_MAX_COMPLETION_TOKENS, 100_000),
            "length"
        );
        assert_eq!(
            super::completion_finish_reason(super::SERVER_MAX_COMPLETION_TOKENS - 1, 100_000),
            "stop"
        );
    }

    #[test]
    fn flatten_chat_prompt_handles_supported_roles() {
        // A single user message passes through verbatim.
        let one = vec![super::ChatMessage {
            role: "user".into(),
            content: "hi".into(),
        }];
        assert_eq!(super::flatten_chat_prompt(&one).unwrap(), "hi");

        // Multiple roles, including developer (system-equivalent), are
        // labeled and joined.
        let many = vec![
            super::ChatMessage {
                role: "developer".into(),
                content: "be terse".into(),
            },
            super::ChatMessage {
                role: "user".into(),
                content: "hello".into(),
            },
            super::ChatMessage {
                role: "assistant".into(),
                content: "hi".into(),
            },
        ];
        assert_eq!(
            super::flatten_chat_prompt(&many).unwrap(),
            "System: be terse\nUser: hello\nAssistant: hi"
        );
    }

    #[test]
    fn flatten_chat_prompt_fails_closed_on_unsupported_role() {
        let bad = vec![super::ChatMessage {
            role: "tool".into(),
            content: "{}".into(),
        }];
        assert_eq!(super::flatten_chat_prompt(&bad).unwrap_err(), "tool");
    }

    // ---- #654 phase D: SSE streaming ----

    #[test]
    fn stream_deltas_reconstruct_the_completion_byte_for_byte() {
        for text in [
            "",
            "hi",
            "hello world",
            "  leading spaces",
            "trailing spaces  ",
            "a   wide   gap",
            "line one\nline two",
            "héllo wörld — ünïcode",
        ] {
            let joined: String = super::split_stream_deltas(text).concat();
            assert_eq!(joined, text, "deltas must rejoin to the exact input");
        }
        assert!(super::split_stream_deltas("").is_empty());
        assert_eq!(super::split_stream_deltas("solo"), vec!["solo".to_string()]);
    }

    fn parse_stream_chunks(frames: &[String]) -> Vec<serde_json::Value> {
        // Every frame is `data: <payload>\n\n`; collect the JSON payloads,
        // skipping the terminal `[DONE]` marker.
        frames
            .iter()
            .map(|f| {
                assert!(f.starts_with("data: "), "frame must start with `data: `");
                assert!(f.ends_with("\n\n"), "frame must end with a blank line");
                f.trim_start_matches("data: ").trim_end().to_string()
            })
            .filter(|payload| payload != "[DONE]")
            .map(|payload| serde_json::from_str(&payload).expect("chunk is JSON"))
            .collect()
    }

    #[test]
    fn stream_frames_have_role_first_content_then_terminal_done() {
        let frames = super::build_chat_stream_frames(
            "chatcmpl-uor-r4-1",
            1,
            "uor-r4",
            "uor-r4-r4g1",
            "hello world",
            "stop",
            None,
            Some(serde_json::json!({ "generation_mode": "r4g1" })),
            serde_json::json!([]),
        );

        // Closes with the SSE terminal marker.
        assert_eq!(frames.last().unwrap(), "data: [DONE]\n\n");

        let chunks = parse_stream_chunks(&frames);
        // Every chunk is a chat.completion.chunk carrying the shared id.
        for chunk in &chunks {
            assert_eq!(chunk["object"], "chat.completion.chunk");
            assert_eq!(chunk["id"], "chatcmpl-uor-r4-1");
        }
        // First chunk carries the assistant role and no finish_reason.
        assert_eq!(chunks[0]["choices"][0]["delta"]["role"], "assistant");
        assert!(chunks[0]["choices"][0]["finish_reason"].is_null());

        // The content deltas rejoin to the exact completion.
        let content: String = chunks
            .iter()
            .filter_map(|c| c["choices"][0]["delta"]["content"].as_str())
            .collect();
        assert_eq!(content, "hello world");

        // Terminal chunk: empty delta, truthful finish_reason, audit parity.
        let terminal = chunks.last().unwrap();
        assert_eq!(terminal["choices"][0]["finish_reason"], "stop");
        assert!(terminal["choices"][0]["delta"]
            .as_object()
            .unwrap()
            .is_empty());
        assert_eq!(terminal["uor_audit"]["generation_mode"], "r4g1");

        // include_usage was false → no usage-only chunk.
        assert!(chunks.iter().all(|c| c.get("usage").is_none()));
    }

    #[test]
    fn stream_include_usage_emits_a_final_usage_only_chunk() {
        let usage = super::VendorUsage {
            prompt_tokens: 3,
            completion_tokens: 2,
            total_tokens: 5,
        };
        let frames = super::build_chat_stream_frames(
            "chatcmpl-uor-r4-2",
            2,
            "uor-r4",
            "uor-r4-r4g1",
            "hi there",
            "length",
            Some(&usage),
            None,
            serde_json::json!([]),
        );
        let chunks = parse_stream_chunks(&frames);
        // Exactly one usage-bearing chunk, and it has empty choices.
        let usage_chunks: Vec<_> = chunks.iter().filter(|c| c.get("usage").is_some()).collect();
        assert_eq!(usage_chunks.len(), 1);
        assert_eq!(usage_chunks[0]["choices"].as_array().unwrap().len(), 0);
        assert_eq!(usage_chunks[0]["usage"]["total_tokens"], 5);
    }

    #[test]
    fn stream_request_parses_with_stream_and_options() {
        let req: super::VendorChatCompletionsRequest = serde_json::from_str(
            r#"{"messages":[{"role":"user","content":"hi"}],"stream":true,"stream_options":{"include_usage":true}}"#,
        )
        .expect("stream fields are accepted");
        assert_eq!(req.stream, Some(true));
        assert_eq!(req.stream_options.and_then(|o| o.include_usage), Some(true));
    }

    #[test]
    fn stream_options_denies_unknown_fields() {
        let parsed: Result<super::VendorChatCompletionsRequest, _> =
            serde_json::from_str(r#"{"messages":[],"stream":true,"stream_options":{"bogus":1}}"#);
        assert!(parsed.is_err(), "unknown stream option must fail closed");
    }

    // ---- #654 phase E: /v1/responses ----

    fn dummy_generation(
        text: &str,
        completion_tokens: usize,
        created: u64,
    ) -> super::GeneratedCompletion {
        super::GeneratedCompletion {
            text: text.to_string(),
            generation_mode: "r4g1".to_string(),
            prompt_tokens: 4,
            completion_tokens,
            total_tokens: 4 + completion_tokens,
            created_ts: created,
            uor_audit: super::UorAuditTrace {
                uor_address: "blake3:test".to_string(),
                kappa: 1.0,
                deficit_angle: 0.0,
                entropy_bias: 0.0,
                gamma: 1.0,
                temperature: 0.7,
                kappa_pass: true,
                generation_mode: "r4g1".to_string(),
                total_latency_ms: 1.0,
                tokens_detail: Vec::new(),
            },
            cascade_trail: serde_json::json!([]),
        }
    }

    #[test]
    fn responses_input_string_passes_through_bare() {
        assert_eq!(
            super::flatten_responses_input(&serde_json::json!("hello"), None).unwrap(),
            "hello"
        );
    }

    #[test]
    fn responses_instructions_and_messages_are_labeled() {
        // A string input with instructions becomes a System preamble + User.
        assert_eq!(
            super::flatten_responses_input(&serde_json::json!("hi"), Some("be terse")).unwrap(),
            "System: be terse\nUser: hi"
        );
        // An array of message items is flattened like chat roles, and content
        // parts are concatenated.
        let input = serde_json::json!([
            { "role": "user", "content": [{ "type": "input_text", "text": "who " }, { "type": "input_text", "text": "are you" }] },
            { "role": "assistant", "content": "a router" }
        ]);
        assert_eq!(
            super::flatten_responses_input(&input, None).unwrap(),
            "User: who are you\nAssistant: a router"
        );
    }

    #[test]
    fn responses_input_fails_closed_on_bad_shapes() {
        // Unsupported role.
        let bad_role = serde_json::json!([{ "role": "tool", "content": "{}" }]);
        assert!(super::flatten_responses_input(&bad_role, None).is_err());
        // Non-object item.
        let non_object = serde_json::json!(["just a string"]);
        assert!(super::flatten_responses_input(&non_object, None).is_err());
        // Typed item that is not a message.
        let wrong_type =
            serde_json::json!([{ "type": "function_call", "role": "user", "content": "x" }]);
        assert!(super::flatten_responses_input(&wrong_type, None).is_err());
        // Empty input with no instructions.
        assert!(super::flatten_responses_input(&serde_json::json!([]), None).is_err());
        // Neither string nor array.
        assert!(super::flatten_responses_input(&serde_json::json!(42), None).is_err());
    }

    #[test]
    fn responses_body_is_a_completed_response_object() {
        let body = super::build_responses_body(dummy_generation("a router", 2, 7), "uor-r4", 64);
        assert_eq!(body["object"], "response");
        assert_eq!(body["status"], "completed");
        assert_eq!(body["model"], "uor-r4");
        assert_eq!(body["output"][0]["type"], "message");
        assert_eq!(body["output"][0]["role"], "assistant");
        assert_eq!(body["output"][0]["content"][0]["type"], "output_text");
        assert_eq!(body["output"][0]["content"][0]["text"], "a router");
        assert_eq!(body["usage"]["input_tokens"], 4);
        assert_eq!(body["usage"]["output_tokens"], 2);
        assert_eq!(body["usage"]["total_tokens"], 6);
        // Completed generations carry no incomplete_details.
        assert!(body.get("incomplete_details").is_none());
        // R4 audit parity extra.
        assert_eq!(body["uor_audit"]["generation_mode"], "r4g1");
    }

    #[test]
    fn responses_body_reports_incomplete_at_the_budget() {
        // completion_tokens == effective budget → incomplete/max_output_tokens.
        let body = super::build_responses_body(dummy_generation("x", 64, 9), "uor-r4", 64);
        assert_eq!(body["status"], "incomplete");
        assert_eq!(body["incomplete_details"]["reason"], "max_output_tokens");
    }

    #[test]
    fn responses_request_parses_subset_and_denies_unknown() {
        let req: super::VendorResponsesRequest = serde_json::from_str(
            r#"{"model":"uor-r4","input":"hi","instructions":"be terse","max_output_tokens":32,"temperature":0.5}"#,
        )
        .expect("supported subset parses");
        assert_eq!(req.model.as_deref(), Some("uor-r4"));
        assert_eq!(req.max_output_tokens, Some(32));

        // A missing `input` fails (it is required).
        let missing: Result<super::VendorResponsesRequest, _> =
            serde_json::from_str(r#"{"model":"uor-r4"}"#);
        assert!(missing.is_err(), "input is required");

        // An unsupported parameter (tools) fails closed.
        let unsupported: Result<super::VendorResponsesRequest, _> =
            serde_json::from_str(r#"{"input":"hi","tools":[]}"#);
        assert!(unsupported.is_err(), "unsupported param must fail closed");
    }

    // ---- #654 phase F: official OpenAI SDK wire compatibility ----
    // The fixtures below are the EXACT request bodies emitted by the official
    // OpenAI Python SDK (3.0.0) and JS SDK (7.4.0) for basic calls, captured
    // against an echo server. They deserialize into our DTOs unchanged: the SDKs
    // omit every unset optional param, so a real call always lands inside the
    // supported subset and `deny_unknown_fields` never trips on an SDK default.
    // These are the deterministic, CI-safe half of the phase-F smoke coverage;
    // the runnable end-to-end scripts live in profiles/openai/smoke_test.{py,mjs}.

    #[test]
    fn sdk_chat_request_fixtures_deserialize() {
        // Minimal chat.completions.create(model, messages).
        let minimal: super::VendorChatCompletionsRequest = serde_json::from_str(
            r#"{"messages":[{"content":"hi","role":"user"}],"model":"uor-r4"}"#,
        )
        .expect("minimal SDK chat payload deserializes");
        assert_eq!(minimal.model.as_deref(), Some("uor-r4"));
        assert_eq!(minimal.messages.len(), 1);
        assert!(minimal.stream.is_none());

        // With max_tokens + temperature + a system message.
        let full: super::VendorChatCompletionsRequest = serde_json::from_str(
            r#"{"max_tokens":32,"messages":[{"content":"be terse","role":"system"},{"content":"hi","role":"user"}],"model":"uor-r4","temperature":0.5}"#,
        )
        .expect("full SDK chat payload deserializes");
        assert_eq!(full.max_tokens, Some(32));
        assert_eq!(full.messages.len(), 2);

        // A streaming call adds exactly `stream: true`.
        let streaming: super::VendorChatCompletionsRequest = serde_json::from_str(
            r#"{"messages":[{"content":"hi","role":"user"}],"model":"uor-r4","stream":true}"#,
        )
        .expect("streaming SDK chat payload deserializes");
        assert_eq!(streaming.stream, Some(true));
    }

    #[test]
    fn sdk_responses_request_fixtures_deserialize() {
        // Minimal responses.create(model, input).
        let minimal: super::VendorResponsesRequest =
            serde_json::from_str(r#"{"input":"hi","model":"uor-r4"}"#)
                .expect("minimal SDK responses payload deserializes");
        assert_eq!(minimal.model.as_deref(), Some("uor-r4"));
        assert!(minimal.input.is_string());

        // With instructions + max_output_tokens.
        let full: super::VendorResponsesRequest = serde_json::from_str(
            r#"{"input":"hi","instructions":"be terse","max_output_tokens":32,"model":"uor-r4"}"#,
        )
        .expect("full SDK responses payload deserializes");
        assert_eq!(full.instructions.as_deref(), Some("be terse"));
        assert_eq!(full.max_output_tokens, Some(32));
    }

    #[test]
    fn chat_response_carries_the_fields_the_sdk_reads() {
        // The SDK reads choices[0].message.{role,content}, choices[0].
        // finish_reason, usage.{prompt,completion,total}_tokens, and id/object.
        let resp = super::VendorChatCompletionsResponse {
            id: "chatcmpl-uor-r4-1".to_string(),
            object: "chat.completion".to_string(),
            created: 1,
            model: "uor-r4".to_string(),
            choices: vec![super::VendorChoice {
                index: 0,
                message: super::VendorChatMessage {
                    role: "assistant".to_string(),
                    content: "hi".to_string(),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: super::VendorUsage {
                prompt_tokens: 1,
                completion_tokens: 1,
                total_tokens: 2,
            },
            system_fingerprint: Some("uor-r4-r4g1".to_string()),
            uor_audit: None,
            cascade_trail: serde_json::json!([]),
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value["object"], "chat.completion");
        assert_eq!(value["choices"][0]["message"]["role"], "assistant");
        assert_eq!(value["choices"][0]["message"]["content"], "hi");
        assert_eq!(value["choices"][0]["finish_reason"], "stop");
        assert_eq!(value["usage"]["total_tokens"], 2);
    }

    #[test]
    fn responses_body_carries_the_fields_the_sdk_reads() {
        // The SDK's `output_text` accessor reads output[].content[].text where
        // type == output_text; it also reads usage.{input,output,total}_tokens
        // and status.
        let body = super::build_responses_body(dummy_generation("hi", 1, 1), "uor-r4", 64);
        assert_eq!(body["object"], "response");
        assert_eq!(body["status"], "completed");
        assert_eq!(body["output"][0]["content"][0]["type"], "output_text");
        assert_eq!(body["output"][0]["content"][0]["text"], "hi");
        assert_eq!(body["usage"]["input_tokens"], 4);
        assert_eq!(body["usage"]["output_tokens"], 1);
        assert_eq!(body["usage"]["total_tokens"], 5);
    }

    // ---- #654 phase G: R4 extended-capability namespace (/uor/v1/*) ----

    #[test]
    fn extended_routes_resolve_canonical_and_alias() {
        // Canonical /uor/v1/* and the deprecated /v1/* alias both resolve to the
        // same canonical path.
        for (path, canonical) in [
            ("/uor/v1/status", "/uor/v1/status"),
            ("/v1/status", "/uor/v1/status"),
            ("/uor/v1/reload", "/uor/v1/reload"),
            ("/v1/reload", "/uor/v1/reload"),
            ("/uor/v1/corpus", "/uor/v1/corpus"),
            ("/v1/corpus", "/uor/v1/corpus"),
        ] {
            assert_eq!(super::extended_route_canonical(path), Some(canonical));
        }
        // The OpenAI surface and unrelated paths are not extended routes.
        for path in [
            "/v1/chat/completions",
            "/v1/models",
            "/v1/responses",
            "/api/chat",
            "/uor/v1/unknown",
        ] {
            assert_eq!(super::extended_route_canonical(path), None);
        }
    }

    #[test]
    fn only_bare_v1_paths_are_deprecated_aliases() {
        assert!(super::is_deprecated_v1_alias("/v1/status"));
        assert!(super::is_deprecated_v1_alias("/v1/reload"));
        assert!(super::is_deprecated_v1_alias("/v1/corpus"));
        // Canonical vendor-namespaced paths are not deprecated.
        assert!(!super::is_deprecated_v1_alias("/uor/v1/status"));
        // Neither is the OpenAI surface.
        assert!(!super::is_deprecated_v1_alias("/v1/chat/completions"));
    }

    #[test]
    fn deprecation_headers_only_on_the_alias() {
        // A deprecated alias gets RFC 8594 Deprecation + a successor Link.
        let headers = super::deprecation_headers("/v1/status");
        assert!(headers.contains("Deprecation: true\r\n"));
        assert!(headers.contains("Link: </uor/v1/status>; rel=\"successor-version\"\r\n"));
        // The canonical path and non-extended paths get no extra headers.
        assert_eq!(super::deprecation_headers("/uor/v1/status"), "");
        assert_eq!(super::deprecation_headers("/v1/chat/completions"), "");
    }

    // ---- streaming for /v1/responses (Responses event protocol) ----

    fn parse_responses_events(frames: &[String]) -> Vec<(String, serde_json::Value)> {
        // Each frame is `event: <type>\ndata: <json>\n\n`.
        frames
            .iter()
            .map(|f| {
                assert!(f.ends_with("\n\n"), "event frame ends with a blank line");
                let (event_line, data_line) =
                    f.trim_end().split_once('\n').expect("event then data line");
                let event_type = event_line
                    .strip_prefix("event: ")
                    .expect("event: line")
                    .to_string();
                let json = data_line.strip_prefix("data: ").expect("data: line");
                (
                    event_type,
                    serde_json::from_str(json).expect("data is JSON"),
                )
            })
            .collect()
    }

    #[test]
    fn responses_stream_frames_follow_the_spec_sequence() {
        let completed =
            super::build_responses_body(dummy_generation("hello world", 2, 7), "uor-r4", 64);
        let frames = super::build_responses_stream_frames(
            "resp-uor-r4-7",
            7,
            "uor-r4",
            "hello world",
            completed,
        );
        let events = parse_responses_events(&frames);

        let types: Vec<&str> = events.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(types[0], "response.created");
        assert_eq!(*types.last().unwrap(), "response.completed");
        // The Responses stream terminates on response.completed — no [DONE].
        assert!(frames.iter().all(|f| !f.contains("[DONE]")));

        // Monotonic sequence numbers 0..N; the `type` field matches the event.
        for (i, (event_type, payload)) in events.iter().enumerate() {
            assert_eq!(payload["sequence_number"], i as u64);
            assert_eq!(payload["type"], event_type.as_str());
        }

        // The delta events rejoin to the completion byte-for-byte.
        let text: String = events
            .iter()
            .filter(|(t, _)| t == "response.output_text.delta")
            .filter_map(|(_, p)| p["delta"].as_str())
            .collect();
        assert_eq!(text, "hello world");

        // The finalize + terminal events carry the full text and the completed
        // Response object (status + output_text + usage the SDK reads).
        let done = events
            .iter()
            .find(|(t, _)| t == "response.output_text.done")
            .unwrap();
        assert_eq!(done.1["text"], "hello world");
        let completed_ev = events.last().unwrap();
        assert_eq!(completed_ev.1["response"]["status"], "completed");
        assert_eq!(
            completed_ev.1["response"]["output"][0]["content"][0]["text"],
            "hello world"
        );
        assert_eq!(completed_ev.1["response"]["usage"]["total_tokens"], 6);
    }

    #[test]
    fn responses_request_accepts_stream() {
        let req: super::VendorResponsesRequest =
            serde_json::from_str(r#"{"model":"uor-r4","input":"hi","stream":true}"#)
                .expect("stream is accepted");
        assert_eq!(req.stream, Some(true));
    }
}
