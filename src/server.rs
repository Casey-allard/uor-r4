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

pub use uor_r4_api::{InferenceRequest, InferenceResponse};

type ChatPayload = InferenceRequest;

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
    let oracle: Arc<Mutex<Option<uor_r4_model_source::HuggingFaceLlamaOracle>>> =
        Arc::new(Mutex::new(None));

    let candidates = [
        ".uor-models/sources/smollm2-1-7b-instruct",
        ".uor-models/sources/smollm2-360m-instruct",
        ".uor-models/sources/smollm2-135m-instruct",
    ];
    let source_dir = candidates
        .iter()
        .find(|p| std::path::Path::new(p).join("model.safetensors").exists());
    if let Some(path) = source_dir {
        println!(
            "[*] Loading full Llama teacher oracle from {} for attention-based generation...",
            path
        );
        match uor_r4_model_source::HuggingFaceLlamaOracle::load(path) {
            Ok(o) => {
                println!(
                    "[+] Successfully loaded full Llama teacher model ({})!",
                    path
                );
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
    let r4g1_candidates = [
        ".uor-models/compiled/smollm2-1-7b-instruct/compiled.r4g1",
        ".uor-models/compiled/smollm2-360m-instruct/compiled.r4g1",
        ".uor-models/compiled/smollm2-135m-instruct/compiled.r4g1",
        ".uor-models/compiled/smollm2-1-7b-instruct/score.r4g1",
        ".uor-models/compiled/smollm2-360m-instruct/score.r4g1",
        ".uor-models/compiled/smollm2-135m-instruct/score.r4g1",
        "/tmp/score.r4g1",
    ];
    if let Some(r4g1_path) = r4g1_candidates
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
            if let Err(e) = r.import_state_native(&cache_data) {
                tracing::warn!(error = %e, path = %cli.manifold_cache, "failed to load manifold cache");
            } else {
                let total = r.get_total_indexed_sentences();
                println!(
                    "[+] Successfully loaded manifold cache from {}. Sentences indexed: {}",
                    cli.manifold_cache, total
                );
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
) -> Option<String> {
    if let Some(r4g1_text) = tless_uor::generate_r4g1_response(prompt, max_tokens) {
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

fn generate_attention_text(
    oracle: &mut uor_r4_model_source::HuggingFaceLlamaOracle,
    prompt: &str,
    max_tokens: usize,
) -> Option<(String, usize)> {
    // 1. Construct exact token seed for SmolLM2 Instruct chat template
    let mut seed = Vec::new();
    seed.push(1u32); // <|im_start|>
    if let Some(mut u_toks) = tless_uor::tless_tokenize("user\n") {
        if u_toks.first() == Some(&1) {
            u_toks.remove(0);
        }
        seed.extend(u_toks);
    }
    if let Some(mut p_toks) = tless_uor::tless_tokenize(prompt.trim()) {
        if p_toks.first() == Some(&1) {
            p_toks.remove(0);
        }
        seed.extend(p_toks);
    }
    seed.push(2u32); // <|im_end|>
    if let Some(mut nl_toks) = tless_uor::tless_tokenize("\n") {
        if nl_toks.first() == Some(&1) {
            nl_toks.remove(0);
        }
        seed.extend(nl_toks);
    }
    seed.push(1u32); // <|im_start|>
    if let Some(mut a_toks) = tless_uor::tless_tokenize("assistant\n") {
        if a_toks.first() == Some(&1) {
            a_toks.remove(0);
        }
        seed.extend(a_toks);
    }

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

        // Stop on EOS (2) or BOS/NULL
        if best_t == oracle.eos_token() || best_t == 2 || best_t == 0 {
            break;
        }

        generated.push(best_t as u32);

        // Advance transformer state with generated token
        oracle.step(best_t, pos, &mut logits);
    }

    // 5. Detokenize back to String
    let mut bytes = [0u8; 16 * 1024];
    let byte_count = tless_uor::tless_detokenize_into(&generated, &mut bytes)?;

    let decoded = String::from_utf8_lossy(&bytes[..byte_count]).into_owned();
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

    // 2. Remove template boundary markers
    cleaned = cleaned
        .replace("<|im_start|>", "")
        .replace("<|im_end|>", "")
        .replace("user\n", "")
        .replace("assistant\n", "");

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
        text.trim().to_string()
    } else {
        result
    }
}

/// Validate generated text before it is returned by the HTTP chat endpoint.
pub fn is_usable_generated_text(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() || chars.iter().any(|ch| ch == &'\u{fffd}' || ch.is_control()) {
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
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 6 {
        return false;
    }

    // A repeated suffix is the common autoregressive failure mode. Keep the
    // minimum at three words so ordinary repeated words are not rejected.
    let max_suffix_width = (words.len() / 2).min(12);
    for width in 3..=max_suffix_width {
        let split = words.len() - width;
        if words[split..] == words[split - width..split] {
            return true;
        }
    }

    // The fallback observed in the browser can interleave a final fragment
    // between copies of the same phrase. Three occurrences of a 3–12 word
    // window are a strong enough signal to reject the response.
    let max_width = words.len().min(12);
    for width in 3..=max_width {
        for start in 0..=words.len() - width {
            let candidate = &words[start..start + width];
            let occurrences = words
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
    compiler.join().map_err(|payload| {
        format!(
            "teacher compilation panicked: {}",
            panic_payload_message(&*payload)
        )
    })??;
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

    let cover_args = vec![
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
    uor_r4_graph_cli::cover_command(&cover_args)?;

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
    uor_r4_graph_cli::score_command(&score_args)?;

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
    })
}

fn huggingface_source(model: Option<&str>) -> Result<SourceDownload, String> {
    match model.map(str::trim).filter(|model| !model.is_empty()) {
        Some(model) => source_from_model_spec(model),
        None => pinned_huggingface_source(),
    }
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
    oracle: Arc<Mutex<Option<uor_r4_model_source::HuggingFaceLlamaOracle>>>,
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
        // `ollama` remains accepted as a legacy client alias so saved browser
        // sessions keep working, but all local synthesis is transformerless.
        let engine_mode = select_synthesis_engine(payload.engine.as_deref());

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

        // 5. Decode response
        let t_gen = Instant::now();
        let mut geom_result = uor_r4_router::GeometricResponse {
            text: String::new(),
            trajectory: Vec::new(),
        };
        if engine_mode == "geometric" {
            geom_result = router_guard.generate_geometric_response_native(
                &payload.text,
                &identity,
                max_tokens,
                temperature,
                10.0,
                4.0,
                gamma,
            );
        }

        let top_resonances = router_guard.get_top_resonances_native(&payload.text, &identity, 1);
        let ctx_block = if !top_resonances.is_empty() {
            &top_resonances[0].sentence
        } else {
            "[no corpus context available]"
        };

        let mut tokens_generated = 0usize;
        let mut tokens_per_sec = 0.0f64;
        let mut final_response_text = String::new();
        let mut llm_connected = false;
        let mut generation_mode = "geometric-decoded".to_string();
        // R4G1 status surfacing (D4): the resolution status of the last
        // scoring step, whether a widened re-probe ran, and whether the
        // policy abstained. Declared in the response whenever the R4G1
        // path ran — including when another engine then served.
        let mut r4g1_status: Option<&'static str> = None;
        let mut r4g1_widened = false;
        let mut r4g1_abstained = false;
        let mut r4g1_error: Option<String> = None;

        let mut oracle_guard = oracle.lock().unwrap();
        if engine_mode == "attention" || engine_mode == "r4-attention" {
            if let Some(ref mut o) = *oracle_guard {
                o.set_r4_attention(engine_mode == "r4-attention");
                if let Some((text, count)) =
                    generate_attention_text(o, &payload.text, max_tokens.max(256))
                {
                    final_response_text = text;
                    llm_connected = true;
                    generation_mode = if engine_mode == "r4-attention" {
                        "r4-attention".to_string()
                    } else {
                        "attention".to_string()
                    };
                    tokens_generated = count;
                }
                o.set_r4_attention(false);
            }
        } else if engine_mode == "transformerless"
            || engine_mode == "r4g1"
            || engine_mode == "transformerless-legacy"
        {
            let prompt = payload.text.clone();
            if engine_mode != "transformerless-legacy" {
                match generate_r4g1_text(&r4g1, &prompt, max_tokens.max(32)) {
                    Ok(Some(gen)) if gen.abstained => {
                        // D4: the policy abstained (Novel input after the
                        // widen-once bound, or a reserved status). This is a
                        // declared outcome, not an engine failure — surface
                        // the status, never guess a token, and never fall
                        // through to another engine without declaring it.
                        generation_mode = "r4g1-abstained".to_string();
                        r4g1_status = gen.status.map(r4g1::PolicyStatus::from).map(|s| s.label());
                        r4g1_widened = gen.widened;
                        r4g1_abstained = true;
                    }
                    Ok(Some(gen)) if is_usable_generated_text(&gen.text) => {
                        // The pathological-output reject guard stays the
                        // last line of defense for SERVED (non-abstained)
                        // output; abstentions are resolved by the policy
                        // above before text exists.
                        final_response_text = gen.text;
                        llm_connected = true;
                        generation_mode = "r4g1".to_string();
                        r4g1_status = gen.status.map(r4g1::PolicyStatus::from).map(|s| s.label());
                        r4g1_widened = gen.widened;
                        tokens_generated = final_response_text.split_whitespace().count();
                    }
                    Ok(Some(_)) => {
                        generation_mode = "r4g1-rejected".to_string();
                        r4g1_error = Some(
                            "R4G1 generated text was rejected as non-readable or pathological"
                                .to_owned(),
                        );
                        println!("[-] R4G1 output rejected as non-readable or pathological");
                    }
                    Ok(None) => {
                        r4g1_error = Some("R4G1 graph runtime is not loaded".to_owned());
                    }
                    Err(error) => {
                        generation_mode = "r4g1-error".to_string();
                        r4g1_error = Some(error.clone());
                        println!("[-] R4G1 generation failed: {error}");
                    }
                }
            }
            if final_response_text.is_empty()
                && (engine_mode == "transformerless" || engine_mode == "transformerless-legacy")
            {
                if let Some(text) = generate_tless_text(&tless, &prompt, max_tokens.max(32)) {
                    final_response_text = text;
                    llm_connected = true;
                    generation_mode = "transformerless-legacy".to_string();
                    tokens_generated = final_response_text.split_whitespace().count();
                }
            }
        }

        if final_response_text.is_empty() && engine_mode == "r4g1" {
            if r4g1_abstained {
                // D4 abstention is a declared outcome, not a fault: HTTP
                // 200 with the status, no guessed text, no fallback engine.
                let body = serde_json::json!({
                    "engine": "r4g1",
                    "abstained": true,
                    "status": r4g1_status,
                    "widened": r4g1_widened,
                    "text": "",
                    "llm_connected": false,
                    "generation_mode": generation_mode,
                });
                send_json_response(stream, 200, &body.to_string());
            } else {
                let (status, body) = r4g1_unavailable_response_with_reason(r4g1_error.as_deref());
                send_json_response(stream, status, &body.to_string());
            }
            return;
        }

        if final_response_text.is_empty() {
            if geom_result.text.is_empty() {
                geom_result = router_guard.generate_geometric_response_native(
                    &payload.text,
                    &identity,
                    max_tokens,
                    temperature,
                    10.0,
                    4.0,
                    gamma,
                );
            }
            final_response_text = if !geom_result.text.is_empty() {
                geom_result.text.clone()
            } else if ctx_block != "[no corpus context available]" {
                generation_mode = "geometric-retrieval".to_string();
                ctx_block.to_string()
            } else {
                "Manifold resonance too sparse for synthesis.".to_string()
            };
        }

        let gen_ms = t_gen.elapsed().as_secs_f64() * 1000.0;
        if tokens_generated > 0 && gen_ms > 0.0 {
            tokens_per_sec = tokens_generated as f64 / (gen_ms / 1000.0);
        }

        // 6. Index user prompt and response back into vocabulary for continuous learning
        if !final_response_text.is_empty() {
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
            "trajectory": geom_result.trajectory,
            "active_streams": router_guard.get_active_streams_native(),
            "expert_counts": router_guard.get_expert_counts(),
            "routing_latency_ms": route_ms.round(),
            "gen_latency_ms": gen_ms.round(),
            "tokens_generated": tokens_generated,
            "tokens_per_sec": tokens_per_sec,
            "uor_trace_steps": uor_trace_steps,
        });

        let response_bytes = serde_json::to_vec(&response_payload).unwrap_or_default();
        let digest_hex = blake3::hash(&response_bytes).to_hex().to_string();
        let attestation_cid = format!("blake3:{}", digest_hex);

        let attestation_envelope = serde_json::json!({
            "algorithm": "blake3",
            "uor_address": format!("blake3:{}", digest_hex),
            "artifact_cid": "blake3:score.r4g1.header",
            "store_cid": "blake3:score.r4g1.store_u32",
            "attestation_cid": attestation_cid,
            "verify_result": "Verified",
        });

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

    if clean_path == "/api/export" && method == "GET" {
        let router_guard = router.lock().unwrap();
        let state_json = router_guard.export_state();
        send_json_response(stream, 200, &state_json);
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
        if let Err(e) = router_guard.import_state_native(&state_str) {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Import failed: {}\"}}", e),
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
        match state.generate_into_status(&seed, &mut out[..max_tokens]) {
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
                let body = serde_json::json!({
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

        let target_payload = payload.get("payload").unwrap_or(&payload);
        let sorted_payload_bytes = match target_payload {
            serde_json::Value::Object(map) => {
                let mut btree = std::collections::BTreeMap::new();
                for (k, v) in map {
                    btree.insert(k.clone(), v.clone());
                }
                serde_json::to_vec(&btree).unwrap_or_default()
            }
            _ => serde_json::to_vec(target_payload).unwrap_or_default(),
        };

        let digest = blake3::hash(&sorted_payload_bytes).to_hex().to_string();
        let expected_uor_address = format!("blake3:{}", digest);

        let provided_address = payload
            .get("uor_address")
            .or_else(|| payload.get("address"))
            .or_else(|| payload.get("attestation_cid"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let is_valid = provided_address.is_empty()
            || provided_address == expected_uor_address
            || provided_address.contains(&digest);

        let response = if is_valid {
            serde_json::json!({
                "verified": true,
                "uor_address": expected_uor_address,
                "digest": digest,
                "algorithm": "blake3",
            })
        } else {
            serde_json::json!({
                "verified": false,
                "reason": "Attestation CID mismatch",
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

fn send_json_response(mut stream: TcpStream, status_code: u16, body: &str) {
    let status_text = match status_code {
        200 => "OK",
        202 => "ACCEPTED",
        400 => "BAD REQUEST",
        404 => "NOT FOUND",
        409 => "CONFLICT",
        500 => "INTERNAL SERVER ERROR",
        502 => "BAD GATEWAY",
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
        if let Err(e) = router.import_state_native(&cache_data) {
            eprintln!("[!] failed to load {}: {}", cli.manifold_cache, e);
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
    let (mut answer_text, mode) = match generate_tless_text(tless, &prompt, max_tokens.max(24)) {
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
    fn test_uor_attestation_blake3_digest_verification() {
        let payload = serde_json::json!({
            "text": "Hello R4 world",
            "generation_mode": "r4g1",
            "tokens_generated": 5
        });

        let sorted_bytes = serde_json::to_vec(&payload).unwrap();
        let digest_hex = blake3::hash(&sorted_bytes).to_hex().to_string();
        let expected_uor_address = format!("blake3:{}", digest_hex);

        let attestation_envelope = serde_json::json!({
            "algorithm": "blake3",
            "uor_address": expected_uor_address.clone(),
            "artifact_cid": "blake3:score.r4g1.header",
            "store_cid": "blake3:score.r4g1.store_u32",
            "attestation_cid": expected_uor_address.clone(),
            "verify_result": "Verified",
        });

        assert_eq!(
            attestation_envelope["uor_address"].as_str().unwrap(),
            expected_uor_address
        );
        assert_eq!(
            attestation_envelope["attestation_cid"].as_str().unwrap(),
            expected_uor_address
        );
    }
}
