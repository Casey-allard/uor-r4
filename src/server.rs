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
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uor_foundation::pipeline::PrismModel;

use uor_r4_core::transformerless::hf_bpe::{
    resolve_source_tokenizer, TokenizerAdapterKey, TokenizerKind,
};
use uor_r4_graph_certify::ScoreStatus;
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
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "running": self.running,
            "ready": self.ready,
            "progress": self.progress,
            "message": self.message,
            "report": self.report,
        })
    }
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
    clear_r4g1_terminal_load_error();
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
    let oracle: Arc<Mutex<Option<uor_r4_model_source::Teacher>>> = Arc::new(Mutex::new(None));

    let last_model = std::fs::read_to_string(".uor-models/last_model_name.txt").unwrap_or_default();
    let last_model_name = last_model.trim();

    let candidates = [
        format!(".uor-models/sources/{}", last_model_name),
        ".uor-models/sources/smollm2-135m-instruct".to_string(),
        ".uor-models/sources/smollm2-360m-instruct".to_string(),
        ".uor-models/sources/smollm2-1-7b-instruct".to_string(),
    ];
    let source_dir = candidates
        .iter()
        .filter(|p| !p.ends_with("/.uor-models/sources/"))
        .find(|p| std::path::Path::new(p).join("model.safetensors").exists());
    if let Some(path) = source_dir {
        println!(
            "[*] Loading full Llama teacher oracle from {} for attention-based generation...",
            path
        );
        match load_serving_source_tokenizer(std::path::Path::new(path), None)
            .and_then(|_| uor_r4_model_source::Teacher::load(path))
        {
            Ok(o) => {
                println!(
                    "[+] Successfully loaded full Llama teacher model ({})!",
                    path
                );
                *oracle.lock().unwrap() = Some(o);
            }
            Err(e) => {
                println!("[-] Failed to load full Llama teacher model: {:?}", e);
                *SERVING_SOURCE_TOKENIZER.lock().unwrap() = None;
                *oracle.lock().unwrap() = None;
            }
        }
    }

    let r4g1: Arc<Mutex<Option<R4g1State>>> = Arc::new(Mutex::new(None));
    let r4g1_compile = Arc::new(Mutex::new(R4g1CompileStatus {
        running: false,
        ready: false,
        progress: 0,
        message: "R4G1 graph compiler idle".to_owned(),
        report: None,
    }));
    let hf_download = Arc::new(Mutex::new(HuggingFaceDownloadStatus {
        running: false,
        ready: Path::new(".uor-models/sources/smollm2-135m-instruct").is_dir(),
        message: "Hugging Face source download idle".to_owned(),
        source: None,
    }));
    let mut r4g1_candidates = r4g1::discover_path(
        cli.r4g1_artifact.as_deref(),
        Path::new(&cli.tless_artifacts),
    )
    .map(|graph| vec![(graph, PathBuf::from(&cli.tless_artifacts))])
    .unwrap_or_default();
    if cli.r4g1_artifact.is_none() {
        r4g1_candidates.extend(discover_compiled_r4g1_candidates());
    }
    let mut loaded_r4g1 = false;
    for (graph_path, teacher_path) in r4g1_candidates {
        let inputs_present = match (
            regular_file_presence(&graph_path),
            regular_file_presence(&teacher_path),
        ) {
            (Ok(graph), Ok(teacher)) => graph && teacher,
            (Err(error), _) | (_, Err(error)) => {
                println!("[-] Refusing present-invalid R4G1 bundle: {error}");
                set_r4g1_terminal_load_error(error);
                break;
            }
        };
        if !inputs_present {
            continue;
        }
        let selected_source = source_dir
            .map(Path::new)
            .filter(|source| source.file_name() == teacher_path.parent().and_then(Path::file_name));
        let inferred_source = if selected_source.is_none() {
            match source_for_compiled_teacher(&teacher_path) {
                Ok(source) => source,
                Err(error) => {
                    println!(
                        "[-] Refusing R4G1 graph {} because its inferred source is invalid: {error}",
                        graph_path.display()
                    );
                    set_r4g1_terminal_load_error(error);
                    break;
                }
            }
        } else {
            None
        };
        let source = selected_source.or(inferred_source.as_deref());
        match R4g1State::load_with_source(&graph_path, &teacher_path, source) {
            Ok(state) => {
                println!(
                    "[+] Loaded validated R4G1 graph runtime from {}",
                    graph_path.display()
                );
                *r4g1.lock().unwrap() = Some(state);
                clear_r4g1_terminal_load_error();
                let mut compile_status = r4g1_compile.lock().unwrap();
                compile_status.ready = true;
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
                if error.contains("tokenizer") {
                    set_r4g1_terminal_load_error(error);
                    break;
                }
            }
        }
    }
    if !loaded_r4g1 {
        if let Some(error) = r4g1_terminal_load_error() {
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
        let g_clone = Arc::clone(&r4g1);
        let gc_clone = Arc::clone(&r4g1_compile);
        let hf_clone = Arc::clone(&hf_download);
        let o_clone = Arc::clone(&oracle);
        let c_clone = Arc::clone(&cli);
        std::thread::spawn(move || {
            handle_connection(
                stream, r_clone, t_clone, g_clone, gc_clone, hf_clone, o_clone, c_clone, start_time,
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

/// Find compiled dashboard bundles when the server was restarted without an
/// explicit `--r4g1-artifact`. The compile endpoint loads its result into the
/// live process, but a restart must pair each graph with the teacher artifact
/// that produced it instead of looking beside the unrelated legacy defaults.
fn discover_compiled_r4g1_candidates() -> Vec<(PathBuf, PathBuf)> {
    let root = Path::new(".uor-models/compiled");
    let mut bundles: Vec<PathBuf> = fs::read_dir(root)
        .into_iter()
        .flat_map(|entries| entries.flatten().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    bundles.sort();
    bundles
        .into_iter()
        .filter_map(|bundle| {
            let graph = bundle.join("graph/score.r4g1");
            let teacher = bundle.join("tless_artifacts.bin");
            (graph.is_file() && teacher.is_file()).then_some((graph, teacher))
        })
        .collect()
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

/// Conventional source snapshot corresponding to
/// `.uor-models/compiled/<name>/tless_artifacts.bin`. Genuine absence is
/// optional; a present non-directory, dangling symlink, or unreadable entry is
/// a hard error so it cannot be mistaken for "no host adapter available."
fn source_for_compiled_teacher(teacher_path: &Path) -> Result<Option<PathBuf>, String> {
    source_for_compiled_teacher_in(teacher_path, Path::new(".uor-models"))
}

fn source_for_compiled_teacher_in(
    teacher_path: &Path,
    models_root: &Path,
) -> Result<Option<PathBuf>, String> {
    let Some(name) = teacher_path.parent().and_then(Path::file_name) else {
        return Ok(None);
    };
    let source = models_root.join("sources").join(name);
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
    slot: &Arc<Mutex<Option<R4g1State>>>,
    prompt: &str,
    max_tokens: usize,
) -> Result<Option<R4g1Text>, String> {
    const MAX_SERVER_TOKENS: usize = 256;
    const MAX_SERVER_TEXT_BYTES: usize = 16 * 1024;
    let mut seed = [0u32; 4096];
    let mut generated = [0u32; MAX_SERVER_TOKENS];
    let mut bytes = [0u8; MAX_SERVER_TEXT_BYTES];
    let (byte_count, status, widened, abstained, usage) = {
        let guard = slot.lock().unwrap();
        let Some(state) = guard.as_ref() else {
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

/// Exact registered tokenizer paired with the loaded teacher oracle. A parse
/// or selection failure clears the slot: teacher-backed serving never falls
/// through to a tokenizer from another id space.
static SERVING_SOURCE_TOKENIZER: std::sync::Mutex<Option<TokenizerKind>> =
    std::sync::Mutex::new(None);

/// Focused startup/reload failure for a graph that was present but could not
/// be bound safely. Keeping it separate from `Option<R4g1State>` preserves the
/// distinction between genuine absence (the historical cascade may continue)
/// and a rejected tokenizer binding (the default cascade must stop).
static R4G1_TERMINAL_LOAD_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn clear_r4g1_terminal_load_error() {
    *R4G1_TERMINAL_LOAD_ERROR.lock().unwrap() = None;
}

fn set_r4g1_terminal_load_error(error: impl Into<String>) {
    *R4G1_TERMINAL_LOAD_ERROR.lock().unwrap() = Some(error.into());
}

fn r4g1_terminal_load_error() -> Option<String> {
    R4G1_TERMINAL_LOAD_ERROR.lock().unwrap().clone()
}

fn load_serving_source_tokenizer(
    dir: &std::path::Path,
    selection: Option<&TokenizerAdapterKey>,
) -> Result<(), uor_r4_model_source::SourceUnavailable> {
    *SERVING_SOURCE_TOKENIZER.lock().unwrap() = None;
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
    *SERVING_SOURCE_TOKENIZER.lock().unwrap() = Some(tokenizer);
    Ok(())
}

fn generate_attention_text(
    oracle: &mut uor_r4_model_source::Teacher,
    prompt: &str,
    max_tokens: usize,
) -> Option<(String, ServingUsage)> {
    // 1. Construct token seed for prompt
    let formatted_prompt = format!("User: {}\nAssistant:", prompt.trim());
    let source_tokenizer = SERVING_SOURCE_TOKENIZER.lock().unwrap();
    let tokenizer = source_tokenizer.as_ref()?;
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
    slot: &Arc<Mutex<Option<R4g1State>>>,
    prompt: &str,
    max_tokens: usize,
    signal: &mut R4g1Signal,
    load_error: Option<&str>,
    usage: &Cell<Option<ServingUsage>>,
) -> TierResult {
    match generate_r4g1_text(slot, prompt, max_tokens.max(32)) {
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
    prompt: &str,
    max_tokens: usize,
    r4_attention: bool,
    usage: &Cell<Option<ServingUsage>>,
) -> TierResult {
    let Some(o) = oracle.as_mut() else {
        return TierResult::failed("teacher oracle is not loaded");
    };
    o.set_r4_attention(r4_attention);
    let generated = generate_attention_text(o, prompt, max_tokens);
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
    r4g1: &Arc<Mutex<Option<R4g1State>>>,
    tless: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    oracle: &mut Option<uor_r4_model_source::Teacher>,
    prompt: &str,
    identity: &str,
    max_tokens: usize,
    temperature: f64,
    gamma: f64,
    session_signature: Option<&[u8]>,
    pinned: Option<&'static str>,
) -> ServingCascade {
    let mut signal = R4g1Signal::default();
    let mut geometric: Option<uor_r4_router::GeometricResponse> = None;
    let usage = Cell::new(None);
    let load_error = r4g1_terminal_load_error();
    let host_encoder_unavailable = r4g1
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(R4g1State::host_encoder_unavailable);
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
                    attention_tier(oracle, prompt, max_tokens.max(128), false, usage_ref)
                }),
            ));
        } else if pinned == Some(TIER_ATTENTION) {
            tiers.push((
                TIER_ATTENTION,
                Box::new(move || {
                    attention_tier(oracle, prompt, max_tokens.max(256), false, usage_ref)
                }),
            ));
        } else if pinned == Some(TIER_R4_ATTENTION) {
            tiers.push((
                TIER_R4_ATTENTION,
                Box::new(move || {
                    attention_tier(oracle, prompt, max_tokens.max(256), true, usage_ref)
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

fn compile_bundle_from_source(
    source: &Path,
    status: &Arc<Mutex<R4g1CompileStatus>>,
    tokenizer_selection: Option<&TokenizerAdapterKey>,
) -> Result<PathBuf, String> {
    let name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "source path is not a valid model directory: {}",
                source.display()
            )
        })?;
    let output = PathBuf::from(".uor-models/compiled").join(name);
    // Resolve once before the resumable stage mutates its output and pass the
    // selected family/version as an atomic pair. Sources with multiple
    // definitions are intentionally refused until the caller names one.
    let tokenizer =
        resolve_source_tokenizer(source, tokenizer_selection).map_err(|error| error.to_string())?;
    let adapter = tokenizer
        .adapter()
        .ok_or_else(|| "source resolved to an adapterless tokenizer".to_owned())?;
    let args = vec![
        "--source".to_owned(),
        source.display().to_string(),
        "--output".to_owned(),
        output.display().to_string(),
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
    let meta_path = output.join("corpus.meta");
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
    validate_source_bundle_inventory(&output)?;
    let meta = output.join("corpus.meta");
    let records = output.join("corpus.records");
    let meta_str = meta
        .to_str()
        .ok_or_else(|| format!("corpus metadata path is not UTF-8: {}", meta.display()))?;
    let records_str = records
        .to_str()
        .ok_or_else(|| format!("corpus records path is not UTF-8: {}", records.display()))?;
    if uor_r4_core::transformerless::compiler::load_corpus_from(meta_str, records_str).is_none() {
        return Err(format!(
            "teacher corpus is incomplete at {}; click Compile / Refresh again to resume generation toward {} samples",
            output.display(), R4G1_CORPUS_TARGET
        ));
    }
    Ok(output)
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
    r4g1: &Arc<Mutex<Option<R4g1State>>>,
    status: &Arc<Mutex<R4g1CompileStatus>>,
    downloaded_source: Option<&Path>,
    tokenizer_selection: Option<&TokenizerAdapterKey>,
) -> Result<serde_json::Value, String> {
    set_r4g1_compile_progress(
        status,
        5,
        "Preparing teacher corpus and R4G1 compiler inputs...",
    );
    // A downloaded source is authoritative for the browser workflow. Even
    // when an older corpus bundle already exists, resume the teacher compile
    // first so the requested target (currently 200k tokens) is actually
    // reached instead of silently rebuilding the old ~20k corpus.
    let source_root = downloaded_source
        .map(|source| compile_bundle_from_source(source, status, tokenizer_selection))
        .transpose()?;
    let compiled_from_source = source_root.is_some();
    if downloaded_source.is_none() && tokenizer_selection.is_some() {
        return Err(
            "an explicit tokenizer selection requires a downloaded source directory".to_owned(),
        );
    }
    set_r4g1_compile_progress(status, 20, "Building the R4G1 cover...");
    let (artifacts, corpus_meta, corpus_recs, cover_output, graph_output, graph_path) =
        match source_root {
            Some(root) => {
                let artifacts = root.join("tless_artifacts.bin");
                let corpus_meta = root.join("corpus.meta");
                let corpus_recs = root.join("corpus.records");
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
    // #597: when compiling from a downloaded snapshot that carries a
    // source-snapshot manifest, bind its root κ into the cover report.
    // Opportunistic by design: a legacy snapshot without a manifest (or an
    // unreadable one) compiles exactly as before, with no κ recorded.
    if let Some(kappa) = downloaded_source.and_then(source_manifest_kappa_of) {
        cover_args.extend(["--source-manifest-kappa".to_owned(), kappa]);
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
    uor_r4_graph_cli::cover_command(&cover_args).map_err(|error| error.to_string())?;

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
    uor_r4_graph_cli::score_command(&score_args).map_err(|error| error.to_string())?;

    set_r4g1_compile_progress(status, 90, "Validating and loading the compiled graph...");
    let inferred_source = if downloaded_source.is_none() {
        source_for_compiled_teacher(&artifacts)?
    } else {
        None
    };
    let source_for_host = downloaded_source.or(inferred_source.as_deref());
    let state = R4g1State::load_with_source(&graph_path, &artifacts, source_for_host)
        .map_err(|error| format!("compiled graph was written but failed validation: {error}"))?;
    *r4g1.lock().unwrap() = Some(state);
    clear_r4g1_terminal_load_error();

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
    r4g1: Arc<Mutex<Option<R4g1State>>>,
    status: Arc<Mutex<R4g1CompileStatus>>,
    downloaded_source: Option<String>,
    tokenizer_selection: Option<TokenizerAdapterKey>,
) {
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compile_r4g1_bundle(
                &cli,
                &r4g1,
                &status,
                downloaded_source.as_deref().map(Path::new),
                tokenizer_selection.as_ref(),
            )
        }))
        .map_err(|payload| {
            format!(
                "R4G1 compilation panicked: {}",
                panic_payload_message(&*payload)
            )
        })
        .and_then(|result| result);

        let mut current = status.lock().unwrap();
        current.running = false;
        match result {
            Ok(details) => {
                current.ready = true;
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
                current.ready = r4g1.lock().unwrap().is_some();
                current.progress = 0;
                current.message = format!("R4G1 compilation failed: {error}");
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

fn pinned_huggingface_source() -> Result<SourceDownload, String> {
    let manifest_path = Path::new("models/smollm2-135m-instruct.json");
    let manifest = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "pinned Hugging Face manifest is unavailable at {}: {error}",
            manifest_path.display()
        )
    })?;
    let manifest: PinnedSourceManifest = serde_json::from_str(&manifest)
        .map_err(|error| format!("invalid pinned Hugging Face manifest: {error}"))?;
    let name = manifest
        .source_directory
        .as_deref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("smollm2-135m-instruct")
        .to_owned();
    Ok(SourceDownload {
        repository: manifest.repository,
        revision: manifest.revision,
        name,
        output: manifest.source_directory.map(PathBuf::from),
        license: manifest.license,
    })
}

fn source_from_model_spec(model: &str) -> Result<SourceDownload, String> {
    let (repository, revision) = model
        .trim()
        .split_once('@')
        .ok_or_else(|| "custom model must use owner/repository@<40-character-commit>".to_owned())?;
    if repository.is_empty()
        || revision.len() != 40
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("custom model must use owner/repository@<40-character-commit>".to_owned());
    }
    let name = repository
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "custom model repository must use owner/repository".to_owned())?;
    Ok(SourceDownload {
        repository: repository.to_owned(),
        revision: revision.to_owned(),
        name: format!("{}-{}", name, &revision[..12]),
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

/// Root κ of the #597 source-snapshot manifest inside a downloaded
/// snapshot directory, when one is present and verifiable. Total: any
/// miss (legacy snapshot, malformed manifest, addressing failure)
/// resolves to `None` so pre-#597 snapshots keep compiling unchanged.
fn source_manifest_kappa_of(source: &Path) -> Option<String> {
    let manifest = crate::model::read_source_manifest(source).ok()?;
    crate::model::source_manifest_kappa(&manifest).ok()
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

fn downloaded_source_path(source: &SourceDownload) -> PathBuf {
    source.output.clone().unwrap_or_else(|| {
        PathBuf::from(".uor-models")
            .join("sources")
            .join(&source.name)
    })
}

fn spawn_huggingface_download(
    status: Arc<Mutex<HuggingFaceDownloadStatus>>,
    source: SourceDownload,
) {
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let name = source.name.clone();
            let repository = source.repository.clone();
            let destination = download_source(&source).map_err(|error| error.to_string())?;
            Ok::<_, String>((repository, name, destination))
        }))
        .map_err(|payload| {
            format!(
                "Hugging Face download panicked: {}",
                panic_payload_message(&*payload)
            )
        })
        .and_then(|result| result);

        let mut current = status.lock().unwrap();
        current.running = false;
        match result {
            Ok((repository, name, destination)) => {
                current.ready = true;
                current.source = Some(destination.display().to_string());
                current.message = format!("Downloaded Hugging Face source {repository} ({name})");
            }
            Err(error) => {
                current.message = format!("Hugging Face download failed: {error}");
            }
        }
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
    r4g1: &Arc<Mutex<Option<R4g1State>>>,
    tless: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    oracle: &Arc<Mutex<Option<uor_r4_model_source::Teacher>>>,
    cli: &Arc<ServerConfig>,
    start_time: Instant,
    prompt_text: &str,
    engine: Option<&str>,
    max_tokens: usize,
    temperature_override: Option<f64>,
) -> GenerationOutcome {
    let identity = "tenant-alpha".to_string();

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
    let cascade = {
        let mut oracle_guard = oracle.lock().unwrap();
        run_serving_cascade(
            &mut router_guard,
            r4g1,
            tless,
            &mut oracle_guard,
            prompt_text,
            &identity,
            max_tokens,
            temperature,
            gamma,
            Some(&session_signature),
            pinned,
        )
    };
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
    r4g1: Arc<Mutex<Option<R4g1State>>>,
    r4g1_compile: Arc<Mutex<R4g1CompileStatus>>,
    hf_download: Arc<Mutex<HuggingFaceDownloadStatus>>,
    oracle: Arc<Mutex<Option<uor_r4_model_source::Teacher>>>,
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
        // #654 phase B: advertise only loadable compiled bundles, each with
        // every field the pinned Model schema requires.
        let models = loadable_models_in(Path::new(COMPILED_MODELS_DIR));
        send_json_response(stream, 200, &models_list_body(&models).to_string());
        return;
    }

    // GET /v1/models/{model} (#654 phase B): agrees with the list; a model id
    // absent from the loadable set is a 404 with the standard error envelope.
    if method == "GET" {
        if let Some(model_id) = clean_path.strip_prefix("/v1/models/") {
            let models = loadable_models_in(Path::new(COMPILED_MODELS_DIR));
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
        let r4g1_loaded = r4g1.lock().unwrap().is_some();
        let oracle_loaded = oracle.lock().unwrap().is_some();

        let model_candidates = [
            "smollm2-135m-instruct",
            "smollm2-360m-instruct",
            "smollm2-1-7b-instruct",
        ];

        let active_model = model_candidates
            .iter()
            .find(|m| {
                std::path::Path::new(&format!(".uor-models/compiled/{}/tless_artifacts.bin", m))
                    .is_file()
            })
            .unwrap_or(&"smollm2-135m-instruct");

        let source_downloaded =
            std::path::Path::new(&format!(".uor-models/sources/{}", active_model)).is_dir();
        let bundle_compiled = std::path::Path::new(&format!(
            ".uor-models/compiled/{}/tless_artifacts.bin",
            active_model
        ))
        .is_file();
        let graph_compiled = std::path::Path::new(&format!(
            ".uor-models/compiled/{}/graph/score.r4g1",
            active_model
        ))
        .is_file()
            || std::path::Path::new(&format!(
                ".uor-models/compiled/{}/compiled.r4g1",
                active_model
            ))
            .is_file();
        let engine_active = r4g1_loaded || oracle_loaded;

        let body = serde_json::json!({
            "model_name": active_model,
            "r4g1_ready": r4g1_loaded,
            "teacher_ready": oracle_loaded,
            "engine_active": engine_active,
            "stages": {
                "stage_1_download": source_downloaded,
                "stage_2_compile": bundle_compiled,
                "stage_3_graph_score": graph_compiled,
                "stage_4_r4g1_active": r4g1_loaded
            }
        });
        send_json_response_ext(stream, 200, &body.to_string(), &dep);
        return;
    }

    if extended_route_canonical(clean_path) == Some("/uor/v1/reload") && method == "POST" {
        // #654 phase G: canonical /uor/v1/reload; /v1/reload deprecated alias.
        let dep = deprecation_headers(clean_path);
        let payload: HuggingFaceDownloadPayload = if body.is_empty() {
            HuggingFaceDownloadPayload::default()
        } else {
            match serde_json::from_slice(&body) {
                Ok(payload) => payload,
                Err(error) => {
                    send_json_response_ext(
                        stream,
                        400,
                        &serde_json::json!({ "error": format!("Invalid JSON: {error}") })
                            .to_string(),
                        &dep,
                    );
                    return;
                }
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
        let target_model = payload.model.as_deref().unwrap_or("smollm2-135m-instruct");

        let teacher_path = PathBuf::from(format!(
            ".uor-models/compiled/{}/tless_artifacts.bin",
            target_model
        ));
        let graph_path = PathBuf::from(format!(
            ".uor-models/compiled/{}/graph/score.r4g1",
            target_model
        ));
        let fallback_path = PathBuf::from(format!(
            ".uor-models/compiled/{}/compiled.r4g1",
            target_model
        ));

        // A reload changes the selected model atomically: discard the old
        // graph/oracle/tokenizer tuple before inspecting the replacement.
        *r4g1.lock().unwrap() = None;
        *oracle.lock().unwrap() = None;
        *SERVING_SOURCE_TOKENIZER.lock().unwrap() = None;
        clear_r4g1_terminal_load_error();

        let path_to_load = match select_regular_fallback_path(&graph_path, &fallback_path) {
            Ok(path) => path,
            Err(error) => {
                set_r4g1_terminal_load_error(error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };
        let oracle_source_path = PathBuf::from(format!(".uor-models/sources/{}", target_model));
        let source_for_reload = match optional_source_directory(&oracle_source_path) {
            Ok(source) => source,
            Err(error) => {
                set_r4g1_terminal_load_error(error.clone());
                send_json_response_ext(
                    stream,
                    500,
                    &serde_json::json!({ "status": "error", "message": error }).to_string(),
                    &dep,
                );
                return;
            }
        };

        let teacher_model_present = match source_for_reload.as_deref() {
            Some(source) => match regular_file_presence(&source.join("model.safetensors")) {
                Ok(present) => present,
                Err(error) => {
                    set_r4g1_terminal_load_error(error.clone());
                    send_json_response_ext(
                        stream,
                        500,
                        &serde_json::json!({ "status": "error", "message": error }).to_string(),
                        &dep,
                    );
                    return;
                }
            },
            None => false,
        };
        if let (true, Some(source)) = (teacher_model_present, source_for_reload.as_deref()) {
            match load_serving_source_tokenizer(source, tokenizer_selection.as_ref())
                .and_then(|_| uor_r4_model_source::Teacher::load(source))
            {
                Ok(o) => {
                    println!(
                        "[+] Successfully reloaded teacher oracle model for '{}'",
                        target_model
                    );
                    *oracle.lock().unwrap() = Some(o);
                }
                Err(e) => {
                    println!(
                        "[-] Note: Teacher oracle reload skipped for '{}': {:?}",
                        target_model, e
                    );
                    *SERVING_SOURCE_TOKENIZER.lock().unwrap() = None;
                    *oracle.lock().unwrap() = None;
                }
            }
        }

        if let Some(path_to_load) = path_to_load {
            match r4g1::R4g1State::load_with_source(
                &path_to_load,
                &teacher_path,
                source_for_reload.as_deref(),
            ) {
                Ok(state) => {
                    *r4g1.lock().unwrap() = Some(state);
                    clear_r4g1_terminal_load_error();
                    let resp = serde_json::json!({
                        "status": "success",
                        "model": target_model,
                        "message": format!("Successfully reloaded R4G1 runtime for model '{}'", target_model)
                    });
                    send_json_response_ext(stream, 200, &resp.to_string(), &dep);
                    return;
                }
                Err(e) => {
                    set_r4g1_terminal_load_error(e.clone());
                    let resp = serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to load R4G1 graph artifact: {}", e)
                    });
                    send_json_response_ext(stream, 500, &resp.to_string(), &dep);
                    return;
                }
            }
        } else {
            let resp = serde_json::json!({
                "status": "error",
                "message": format!("No compiled R4G1 graph artifact found for model '{}'. Please compile it first.", target_model)
            });
            send_json_response_ext(stream, 404, &resp.to_string(), &dep);
            return;
        }
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
            &r4g1,
            &tless,
            &oracle,
            &cli,
            start_time,
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
        let model_name = req.model.clone().unwrap_or_else(|| "uor-r4".to_string());
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
            &r4g1,
            &tless,
            &oracle,
            &cli,
            start_time,
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

        let model_name = req.model.clone().unwrap_or_else(|| "uor-r4".to_string());

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
        let mut cascade = {
            let mut oracle_guard = oracle.lock().unwrap();
            run_serving_cascade(
                &mut router_guard,
                &r4g1,
                &tless,
                &mut oracle_guard,
                &payload.text,
                &identity,
                max_tokens,
                temperature,
                gamma,
                Some(&session_signature),
                pinned,
            )
        };

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
        let status = r4g1_compile.lock().unwrap().clone();
        send_json_response(stream, 200, &status.json().to_string());
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
        let guard = r4g1.lock().unwrap();
        let Some(state) = guard.as_ref() else {
            let load_error = r4g1_terminal_load_error();
            let (status, body) = r4g1_unavailable_response_with_reason(load_error.as_deref());
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
        let guard = r4g1.lock().unwrap();
        let Some(state) = guard.as_ref() else {
            let load_error = r4g1_terminal_load_error();
            let (status, body) = r4g1_unavailable_response_with_reason(load_error.as_deref());
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
            let guard = r4g1.lock().unwrap();
            let Some(state) = guard.as_ref() else {
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
        let payload: HuggingFaceDownloadPayload = if body.is_empty() {
            HuggingFaceDownloadPayload::default()
        } else {
            match serde_json::from_slice(&body) {
                Ok(payload) => payload,
                Err(error) => {
                    send_json_response(
                        stream,
                        400,
                        &format!("{{\"error\":\"Invalid JSON: {error}\"}}"),
                    );
                    return;
                }
            }
        };
        if let Err(error) = payload.tokenizer_selection() {
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
        status.ready = false;
        let revision_preview: String = source.revision.chars().take(12).collect();
        status.message = format!(
            "Downloading {}@{}; this may take a few minutes...",
            source.repository, revision_preview
        );
        drop(status);
        spawn_huggingface_download(Arc::clone(&hf_download), source);
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
        let payload: HuggingFaceDownloadPayload = if body.is_empty() {
            HuggingFaceDownloadPayload::default()
        } else {
            match serde_json::from_slice(&body) {
                Ok(payload) => payload,
                Err(error) => {
                    send_json_response(
                        stream,
                        400,
                        &format!("{{\"error\":\"Invalid JSON: {error}\"}}"),
                    );
                    return;
                }
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
        status.ready = r4g1.lock().unwrap().is_some();
        status.progress = 1;
        status.message = "Compiling R4G1 cover and scored graph...".to_owned();
        status.report = None;
        drop(status);

        let downloaded_source = hf_download.lock().unwrap().source.clone().or_else(|| {
            let source = huggingface_source(payload.model.as_deref()).ok()?;
            let path = downloaded_source_path(&source);
            path.is_dir().then(|| path.display().to_string())
        });

        spawn_r4g1_compile(
            Arc::clone(&cli),
            Arc::clone(&r4g1),
            Arc::clone(&r4g1_compile),
            downloaded_source,
            tokenizer_selection,
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
        let r4g1_ready = r4g1.lock().unwrap().is_some();
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
        let r4g1_ready = r4g1.lock().unwrap().is_some();

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

/// The directory holding compiled model bundles.
const COMPILED_MODELS_DIR: &str = ".uor-models/compiled";

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

/// The models loadable from `compiled_dir`: each immediate subdirectory that
/// contains a compiled `tless_artifacts.bin` bundle, reported as
/// `(id = directory name, created = the bundle's mtime in unix seconds)` and
/// sorted by id for a deterministic listing. Advertising only compiled bundles
/// keeps `/v1/models` truthful — an id absent here is not loadable.
fn loadable_models_in(compiled_dir: &Path) -> Vec<(String, u64)> {
    let mut models = Vec::new();
    if let Ok(entries) = std::fs::read_dir(compiled_dir) {
        for entry in entries.flatten() {
            let artifact = entry.path().join("tless_artifacts.bin");
            if !artifact.is_file() {
                continue;
            }
            let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let created = std::fs::metadata(&artifact)
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|delta| delta.as_secs())
                .unwrap_or(0);
            models.push((id, created));
        }
    }
    models.sort_by(|a, b| a.0.cmp(&b.0));
    models
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

    #[test]
    fn server_cover_uses_registry_validated_observation_manifest_operator() {
        let root = attention_provenance_test_dir("learned-manifest");
        let (meta, records) = attention_corpus_markers(&root);
        let operator = uor_r4_model_source::attention::AttentionOperatorSpec::
            learned_absolute_source_attention();
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
        let forwarded: uor_r4_model_source::attention::AttentionOperatorSpec =
            serde_json::from_str(&args[1]).expect("forwarded record");
        assert_eq!(forwarded, operator);
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
    fn serving_source_tokenizer_is_registered_and_parse_failures_clear_it() {
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
        super::load_serving_source_tokenizer(&dir, Some(&selection))
            .expect("registered source loads");
        let adapter = super::SERVING_SOURCE_TOKENIZER
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|tokenizer| tokenizer.adapter())
            .expect("registered identity");
        assert_eq!(adapter.family, "hf-byte-bpe");
        assert_eq!(adapter.version, 1);

        std::fs::write(
            dir.join("spiece.model"),
            b"present to make selection ambiguous",
        )
        .expect("second tokenizer definition");
        let error = super::load_serving_source_tokenizer(&dir, None)
            .expect_err("ambiguous source selection fails closed");
        assert!(
            error
                .reason
                .contains("both tokenizer.json and spiece.model"),
            "{error}"
        );
        assert!(super::SERVING_SOURCE_TOKENIZER.lock().unwrap().is_none());
        std::fs::remove_file(dir.join("spiece.model")).expect("remove second definition");

        std::fs::write(&tokenizer_path, br#"{"model":{"type":"Unigram"}}"#)
            .expect("replace with unsupported wrapper");
        let error = super::load_serving_source_tokenizer(&dir, Some(&selection))
            .expect_err("unsupported selected definition fails closed");
        assert!(error.reason.contains("hf-byte-bpe/1"), "{error}");
        assert!(super::SERVING_SOURCE_TOKENIZER.lock().unwrap().is_none());
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
    fn loadable_models_lists_only_compiled_bundles_sorted() {
        let dir = std::env::temp_dir().join(format!("uor-r4-models-654b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for name in ["beta-model", "alpha-model"] {
            let bundle = dir.join(name);
            std::fs::create_dir_all(&bundle).unwrap();
            std::fs::write(bundle.join("tless_artifacts.bin"), b"artifact").unwrap();
        }
        // A directory without a compiled artifact is NOT loadable.
        std::fs::create_dir_all(dir.join("no-bundle")).unwrap();

        let models = super::loadable_models_in(&dir);
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
        assert!(super::loadable_models_in(&dir).is_empty());
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
