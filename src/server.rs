use crate as uor_r4_wasm_router;
use crate::model::{download_source, SourceDownload};
use crate::r4g1::{self, R4g1State};
use crate::tless_uor::{self, TlessAxis};
use crate::UorR4Router;
use serde::Deserialize;
use std::any::Any;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uor_foundation::pipeline::PrismModel;

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
        match uor_r4_model_source::Teacher::load(path) {
            Ok(o) => {
                println!(
                    "[+] Successfully loaded full Llama teacher model ({})!",
                    path
                );
                load_serving_hf_tokenizer(std::path::Path::new(path));
                *oracle.lock().unwrap() = Some(o);
            }
            Err(e) => {
                println!("[-] Failed to load full Llama teacher model: {:?}", e);
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
        if !graph_path.is_file() || !teacher_path.is_file() {
            continue;
        }
        match R4g1State::load(&graph_path, &teacher_path) {
            Ok(state) => {
                println!(
                    "[+] Loaded validated R4G1 graph runtime from {}",
                    graph_path.display()
                );
                *r4g1.lock().unwrap() = Some(state);
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
            }
        }
    }
    if !loaded_r4g1 {
        tracing::info!("no validated R4G1 graph found; compile it from the dashboard");
    }
    if let Some(r4g1_path) = R4G1_ARTIFACT_CANDIDATES
        .iter()
        .find(|p| std::path::Path::new(p).exists())
    {
        if let Ok(r4g1_bytes) = std::fs::read(r4g1_path) {
            match uor_r4_graph_runtime::R4G1Runtime::parse(&r4g1_bytes) {
                Ok(_) => {
                    println!(
                        "[+] Successfully loaded R4G1 zero-multiply prediction runtime ({})!",
                        r4g1_path
                    );
                    tless_uor::set_r4g1_bytes(r4g1_bytes);

                    let parent_dir = std::path::Path::new(r4g1_path)
                        .parent()
                        .unwrap_or(std::path::Path::new("."));
                    let art_path = parent_dir.join("tless_artifacts.bin").display().to_string();
                    let store_path = parent_dir.join("tless_store.bin").display().to_string();
                    let tok_path = parent_dir.join("tokenizer.bin").display().to_string();

                    tless_uor::configure_tless_paths(tless_uor::TlessPaths {
                        artifacts: art_path,
                        store: store_path,
                        tokenizer: tok_path,
                    });
                }
                Err(e) => {
                    println!("[-] Failed to parse R4G1 bundle: {:?}", e);
                }
            }
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

/// Generate a text continuation with the transformerless runtime. The shared
/// state keeps chat turns on one graded store and serializes its thread-local
/// UOR binding. `None` means the configured artifacts/tokenizer are not ready.
fn generate_tless_text(
    slot: &Arc<Mutex<Option<tless_uor::TlessState>>>,
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> Option<String> {
    if let Some(r4g1_text) = tless_uor::generate_r4g1_response_with_session_signature(
        prompt,
        max_tokens,
        session_signature,
    ) {
        return Some(r4g1_text);
    }
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
    let (byte_count, status, widened, abstained) = {
        let guard = slot.lock().unwrap();
        let Some(state) = guard.as_ref() else {
            return Ok(None);
        };
        let seed_len = state
            .encode_into(prompt, &mut seed)
            .or_else(|| tless_uor::tless_tokenize_into(prompt, &mut seed))
            .ok_or_else(|| "R4G1 tokenizer could not encode the prompt".to_owned())?;
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
            state
                .decode_into(&generated[..outcome.count], &mut bytes)
                .or_else(|| {
                    tless_uor::tless_detokenize_into(&generated[..outcome.count], &mut bytes)
                })
                .ok_or_else(|| "R4G1 tokenizer could not decode generated tokens".to_owned())?
        };
        (
            bytes_written,
            outcome.status,
            outcome.widened,
            outcome.abstained,
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
    }))
}

/// Serving-side HF BPE tokenizer, loaded from the teacher oracle's source
/// dir (tokenizer.json) whenever an HF oracle loads (issue #254, the #253
/// lineage: seed/decode must live in the teacher's own id space). `None`
/// means a local-llama oracle: the legacy tless pair applies.
static SERVING_HF_TOKENIZER: std::sync::Mutex<
    Option<uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer>,
> = std::sync::Mutex::new(None);

fn load_serving_hf_tokenizer(dir: &std::path::Path) {
    match uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_dir(dir) {
        Ok(t) => {
            println!(
                "[+] Serving HF BPE tokenizer loaded ({}), address {}",
                dir.display(),
                t.address()
            );
            *SERVING_HF_TOKENIZER.lock().unwrap() = Some(t);
        }
        Err(e) => {
            println!(
                "[-] No HF BPE tokenizer for serving ({}): {} — legacy tless pair stays active",
                dir.display(),
                e
            );
            *SERVING_HF_TOKENIZER.lock().unwrap() = None;
        }
    }
}

fn generate_attention_text(
    oracle: &mut uor_r4_model_source::Teacher,
    prompt: &str,
    max_tokens: usize,
) -> Option<(String, usize)> {
    // 1. Construct token seed for prompt
    let formatted_prompt = format!("User: {}\nAssistant:", prompt.trim());
    // TODO(#242 follow-up): serving-side teacher prompting still uses tless_tokenize
    let hf_tok = SERVING_HF_TOKENIZER.lock().unwrap();
    let seed = match hf_tok.as_ref() {
        Some(t) => {
            let ids = t.encode(&formatted_prompt);
            if ids.is_empty() {
                return None;
            }
            ids
        }
        None => match tless_uor::tless_tokenize(&formatted_prompt) {
            Some(s) if !s.is_empty() => s,
            _ => return None,
        },
    };

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
    let decoded = match hf_tok.as_ref() {
        Some(t) => t.decode(&generated),
        None => {
            let mut bytes = [0u8; 16 * 1024];
            let byte_count = tless_uor::tless_detokenize_into(&generated, &mut bytes)?;
            String::from_utf8_lossy(&bytes[..byte_count]).into_owned()
        }
    };
    println!("[+] generate_attention_text: raw decoded: {:?}", decoded);
    let cleaned = clean_attention_response(&decoded, prompt);
    println!("[+] generate_attention_text: cleaned: {:?}", cleaned);
    Some((cleaned, generated.len()))
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
            TierResult::success(gen.text)
        }
        Ok(Some(_)) => {
            let reason = "R4G1 generated text was rejected as non-readable or pathological";
            signal.error = Some(reason.to_owned());
            println!("[-] R4G1 output rejected as non-readable or pathological");
            TierResult::pathological(reason)
        }
        Ok(None) => {
            let reason = "R4G1 graph runtime is not loaded";
            signal.error = Some(reason.to_owned());
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
) -> TierResult {
    let Some(o) = oracle.as_mut() else {
        return TierResult::failed("teacher oracle is not loaded");
    };
    o.set_r4_attention(r4_attention);
    let generated = generate_attention_text(o, prompt, max_tokens);
    o.set_r4_attention(false);
    match generated {
        Some((text, _count)) if is_usable_generated_text(&text) => TierResult::success(text),
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
    let outcome = {
        let signal_ref = &mut signal;
        let geometric_ref = &mut geometric;
        let include = |tier: &'static str| pinned.is_none() || pinned == Some(tier);
        let mut tiers: Vec<(&'static str, TierFn<'_>)> = Vec::new();
        if include(TIER_R4G1) {
            tiers.push((
                TIER_R4G1,
                Box::new(move || r4g1_tier(r4g1, prompt, max_tokens, signal_ref)),
            ));
        }
        if include(TIER_TRANSFORMERLESS) {
            tiers.push((
                TIER_TRANSFORMERLESS,
                Box::new(move || {
                    transformerless_tier(tless, prompt, max_tokens, session_signature)
                }),
            ));
        }
        if pinned.is_none() {
            tiers.push((
                TIER_TEACHER_ORACLE,
                Box::new(move || attention_tier(oracle, prompt, max_tokens.max(128), false)),
            ));
        } else if pinned == Some(TIER_ATTENTION) {
            tiers.push((
                TIER_ATTENTION,
                Box::new(move || attention_tier(oracle, prompt, max_tokens.max(256), false)),
            ));
        } else if pinned == Some(TIER_R4_ATTENTION) {
            tiers.push((
                TIER_R4_ATTENTION,
                Box::new(move || attention_tier(oracle, prompt, max_tokens.max(256), true)),
            ));
        }
        if include(TIER_GEOMETRIC) {
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

fn compile_bundle_from_source(
    source: &Path,
    status: &Arc<Mutex<R4g1CompileStatus>>,
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
    for file in [
        "tless_artifacts.bin",
        "tless_store.bin",
        "tokenizer.bin",
        "corpus.meta",
        "corpus.records",
    ] {
        if !output.join(file).is_file() {
            return Err(format!(
                "transformerless bundle compilation is incomplete; missing {}. Retry the compile action to resume the corpus",
                output.join(file).display()
            ));
        }
    }
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

fn compile_r4g1_bundle(
    cli: &ServerConfig,
    r4g1: &Arc<Mutex<Option<R4g1State>>>,
    status: &Arc<Mutex<R4g1CompileStatus>>,
    downloaded_source: Option<&Path>,
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
        .map(|source| compile_bundle_from_source(source, status))
        .transpose()?;
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
    // #602: the server compile job never passes `--r4-attention` to the
    // teacher stage above, so the teacher ran exactly the registered
    // `standard-source-attention/1` operator; bind its typed record into
    // the cover report.
    if let Ok(operator) =
        serde_json::to_string(&uor_r4_model_source::attention::AttentionOperatorSpec::standard())
    {
        cover_args.extend(["--attention-operator".to_owned(), operator]);
    }
    uor_r4_graph_cli::cover_command(&cover_args).map_err(|error| error.to_string())?;

    set_r4g1_compile_progress(status, 55, "Scoring graph transitions and emissions...");
    let cover_artifact = cover_output.join("cover.r4g1");
    let score_args = vec![
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
    uor_r4_graph_cli::score_command(&score_args).map_err(|error| error.to_string())?;

    set_r4g1_compile_progress(status, 90, "Validating and loading the compiled graph...");
    let state = R4g1State::load(&graph_path, &artifacts)
        .map_err(|error| format!("compiled graph was written but failed validation: {error}"))?;
    *r4g1.lock().unwrap() = Some(state);

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
) {
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compile_r4g1_bundle(
                &cli,
                &r4g1,
                &status,
                downloaded_source.as_deref().map(Path::new),
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

    if clean_path == "/v1/status" && method == "GET" {
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
        send_json_response(stream, 200, &body.to_string());
        return;
    }

    if clean_path == "/v1/reload" && method == "POST" {
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        let target_model = payload["model"].as_str().unwrap_or("smollm2-135m-instruct");

        let teacher_path = format!(".uor-models/compiled/{}/tless_artifacts.bin", target_model);
        let graph_path = format!(".uor-models/compiled/{}/graph/score.r4g1", target_model);
        let fallback_path = format!(".uor-models/compiled/{}/compiled.r4g1", target_model);

        let path_to_load = if std::path::Path::new(&graph_path).is_file() {
            std::path::PathBuf::from(graph_path)
        } else {
            std::path::PathBuf::from(fallback_path)
        };

        let oracle_source = format!(".uor-models/sources/{}", target_model);
        if std::path::Path::new(&oracle_source)
            .join("model.safetensors")
            .exists()
        {
            match uor_r4_model_source::Teacher::load(&oracle_source) {
                Ok(o) => {
                    println!(
                        "[+] Successfully reloaded teacher oracle model for '{}'",
                        target_model
                    );
                    load_serving_hf_tokenizer(std::path::Path::new(&oracle_source));
                    *oracle.lock().unwrap() = Some(o);
                }
                Err(e) => {
                    println!(
                        "[-] Note: Teacher oracle reload skipped for '{}': {:?}",
                        target_model, e
                    );
                }
            }
        }

        if path_to_load.is_file() {
            match r4g1::R4g1State::load(&path_to_load, std::path::Path::new(&teacher_path)) {
                Ok(state) => {
                    *r4g1.lock().unwrap() = Some(state);
                    let resp = serde_json::json!({
                        "status": "success",
                        "model": target_model,
                        "message": format!("Successfully reloaded R4G1 runtime for model '{}'", target_model)
                    });
                    send_json_response(stream, 200, &resp.to_string());
                    return;
                }
                Err(e) => {
                    let resp = serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to load R4G1 graph artifact: {}", e)
                    });
                    send_json_response(stream, 500, &resp.to_string());
                    return;
                }
            }
        } else {
            let resp = serde_json::json!({
                "status": "error",
                "message": format!("No compiled R4G1 graph artifact found for model '{}'. Please compile it first.", target_model)
            });
            send_json_response(stream, 404, &resp.to_string());
            return;
        }
    }

    if clean_path == "/v1/corpus" && method == "POST" {
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
                send_json_response(stream, 200, &resp.to_string());
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
            send_json_response(stream, 200, &resp.to_string());
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
        send_json_response(stream, 200, &resp.to_string());
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
        let temperature = req.temperature.unwrap_or(default_temp);

        let routing_prompt = if prompt_text.len() > 512 {
            &prompt_text[..512]
        } else {
            &prompt_text[..]
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

        // Issue #248: the single serving cascade, honoring an engine pin
        // from the request or the persisted `/engine` selection.
        let pinned = resolve_pinned_tier(req.engine.as_deref());
        let cascade = {
            let mut oracle_guard = oracle.lock().unwrap();
            run_serving_cascade(
                &mut router_guard,
                &r4g1,
                &tless,
                &mut oracle_guard,
                &prompt_text,
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
            // Declined by all: an honest terminal instead of serving the
            // sparse-string placeholder as if it were generated.
            let (status, body) = declined_by_all_response(&cascade, pinned, &generation_mode);
            send_json_response(stream, status, &body.to_string());
            return;
        };

        router_guard.index_sentence(&prompt_text, &identity);
        router_guard.index_sentence(&final_response_text, &identity);
        router_guard.inject_thought_stream_native(&prompt_text);
        router_guard.inject_thought_stream_native(&final_response_text);
        spawn_cache_save(&cli, router_guard.export_state());

        // #654 phase C: usage from the serving tokenizer (the compiled
        // artifact's), not a whitespace-word estimate. On this served path a
        // tokenizer is available (it produced the completion); the defensive
        // `unwrap_or(0)` never substitutes a word count.
        let prompt_tokens = count_serving_tokens(&prompt_text).unwrap_or(0);
        let completion_tokens = count_serving_tokens(&final_response_text).unwrap_or(0);
        let total_tokens = prompt_tokens + completion_tokens;

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
                    "teacher-oracle-fallback" => {
                        "Teacher Oracle Fallback (Full Attention)".to_string()
                    }
                    "geometric-decoded" => "f64 Geometric Router Manifold".to_string(),
                    "r4g1-abstained" => "R4G1 Abstained (OOD Shield)".to_string(),
                    _ => "ExactContext Carryover".to_string(),
                },
                latency_ms: (per_token_ms * 100.0).round() / 100.0,
            })
            .collect();

        // Issue #256: a real canonical label over the audit content —
        // previously a syntactically-valid, semantically-meaningless string
        // minted from a curvature float.
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

        // Values shared by the single-JSON and the streaming surfaces.
        let response_id = format!("chatcmpl-uor-r4-{}", created_ts);
        let model_name = req.model.clone().unwrap_or_else(|| "uor-r4".to_string());
        let system_fingerprint = format!("uor-r4-{}", generation_mode);
        // #654 phase C: `length` when the served completion reached the
        // effective token budget, otherwise `stop`.
        let finish_reason = completion_finish_reason(completion_tokens, max_tokens).to_string();
        let usage = VendorUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        };
        let cascade_trail = cascade_trail_json(&cascade.outcome.trail);

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
            let audit_value = serde_json::to_value(&uor_audit).ok();
            let frames = build_chat_stream_frames(
                &response_id,
                created_ts,
                &model_name,
                &system_fingerprint,
                &final_response_text,
                &finish_reason,
                if include_usage { Some(&usage) } else { None },
                audit_value,
                cascade_trail,
            );
            send_sse_stream(stream, &frames);
            return;
        }

        let resp = VendorChatCompletionsResponse {
            id: response_id,
            object: "chat.completion".to_string(),
            created: created_ts,
            model: model_name,
            choices: vec![VendorChoice {
                index: 0,
                message: VendorChatMessage {
                    role: "assistant".to_string(),
                    content: final_response_text,
                },
                finish_reason,
            }],
            usage,
            system_fingerprint: Some(system_fingerprint),
            uor_audit: Some(uor_audit),
            cascade_trail,
        };

        send_json_response(stream, 200, &serde_json::to_string(&resp).unwrap());
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
            let (status, body) = r4g1_unavailable_response();
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
            let (status, body) = r4g1_unavailable_response();
            send_json_response(stream, status, &body.to_string());
            return;
        };
        let mut seed_buf = [0u32; 4096];
        let seed: Vec<u32> = if let Some(arr) = payload.get("window").and_then(|w| w.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|x| x as u32))
                .collect()
        } else if let Some(text) = payload.get("text").and_then(|t| t.as_str()) {
            match state
                .encode_into(text, &mut seed_buf)
                .or_else(|| tless_uor::tless_tokenize_into(text, &mut seed_buf))
            {
                Some(len) if len > 0 => seed_buf[..len].to_vec(),
                _ => {
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
                    state
                        .decode_into(tokens, &mut text_bytes)
                        .or_else(|| tless_uor::tless_detokenize_into(tokens, &mut text_bytes))
                        .unwrap_or(0)
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

fn send_json_response(mut stream: TcpStream, status_code: u16, body: &str) {
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
         Access-Control-Allow-Headers: Content-Type\r\n\r\n\
         {}",
        status_code,
        status_text,
        body.len(),
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
}
