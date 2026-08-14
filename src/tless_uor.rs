//! UOR bindings for R4's integrated transformerless inference module.
//!
//! Three bindings, mirroring the R4Axis pattern in this crate:
//!
//! - **Addressing**: the TLA artifact container (TLA7 era since the #327 re-pin, 2026-08-01) and individual store
//!   entries become uor-addr content (CBOR realization, blake3 axis). The
//!   proof pins stay raw-blake3 of the container bytes; the uor-addr κ-label
//!   addresses the CBOR canonical form — two labels, one artifact, both
//!   blake3-pinned.
//! - **Witness axis**: `TlessAxis` exposes the mul-free runtime's prediction
//!   path (window → bundle → signature → graded code → deepest-populated-
//!   class argmax) as an axis kernel. The output record carries the
//!   resolution witness (token, depth, code, evidence count) and the op
//!   census. There is no multiply field — its absence is the claim, exactly
//!   as in `OpKernel`.
//! - **Model**: `UorTlessModel` implements `PrismModel`, so each prediction
//!   mints a `Grounded` certificate with derivation replay — the
//!   transformerless prediction witness realized on this repository's UOR
//!   substrate.

use std::cell::RefCell;
use std::collections::BTreeMap;

use uor_foundation::enforcement::{GroundedShape, Hasher, ShapeViolation};
use uor_foundation::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, TermValue,
};
use uor_r4_core::transformerless::compiler::{self, Compiled, WINDOW};
use uor_r4_core::transformerless::runtime::{self, Store};

use uor_r4_router::{R4HostBounds, R4_FP_MAX, R4_INLINE_BYTES};

// =====================================================================
// State: compiled artifact + graded store, per-thread
// =====================================================================

pub struct TlessState {
    pub art: Compiled,
    pub store: Store,
    /// raw blake3 κ of the TLA container (the PROOF.md pin)
    pub artifact_kappa: String,
    /// uor-addr κ-label of the container (CBOR realization, blake3 axis)
    pub artifact_address: String,
    /// κ of the TLS1 store container
    pub store_kappa: String,
}

thread_local! {
    pub static ACTIVE_TLESS: RefCell<Option<*mut TlessState>> = const { RefCell::new(None) };
    static OWNED_TLESS: RefCell<Option<TlessState>> = const { RefCell::new(None) };
}

enum OwnedR4g1Tokenizer {
    /// Compatibility for pre-binding graphs whose HEAD tokenizer CID is zero.
    LegacyGlobal,
    /// Exact tokenizer bytes parsed and installed atomically with the graph.
    Exact(uor_r4_core::transformerless::scenarios::Tokenizer),
}

struct OwnedR4g1Bundle {
    graph: Vec<u8>,
    tokenizer: OwnedR4g1Tokenizer,
}

static OWNED_R4G1: std::sync::RwLock<Option<OwnedR4g1Bundle>> = std::sync::RwLock::new(None);

fn validated_graph_tokenizer_cid(bytes: &[u8]) -> Result<[u8; 32], String> {
    let view = uor_r4_graph_format::GraphView::parse(bytes)
        .map_err(|error| format!("invalid R4G1 graph: {}", error.reason))?;
    view.verify_cids()
        .map_err(|error| format!("invalid R4G1 graph: {}", error.as_format()))?;
    let head = view
        .head()
        .ok_or_else(|| "invalid R4G1 graph: missing HEAD section".to_owned())?;
    uor_r4_graph_runtime::R4G1Runtime::parse(bytes)
        .map_err(|error| format!("invalid R4G1 runtime graph: {}", error.reason))?;
    Ok(head.tokenizer_cid().0)
}

/// Try to install graph-only bytes. This compatibility surface accepts only
/// legacy graphs whose HEAD tokenizer CID is zero; a bound graph must use
/// [`set_r4g1_bundle`] so graph and tokenizer enter the process atomically.
pub fn try_set_r4g1_bytes(bytes: Vec<u8>) -> Result<(), String> {
    let tokenizer_cid = validated_graph_tokenizer_cid(&bytes)?;
    if tokenizer_cid != [0; 32] {
        return Err(format!(
            "R4G1 graph requires tokenizer.bin blake3:{}; graph-only installation is refused",
            blake3::Hash::from(tokenizer_cid).to_hex()
        ));
    }
    let mut guard = OWNED_R4G1
        .write()
        .map_err(|_| "R4G1 bundle lock poisoned".to_owned())?;
    *guard = Some(OwnedR4g1Bundle {
        graph: bytes,
        tokenizer: OwnedR4g1Tokenizer::LegacyGlobal,
    });
    Ok(())
}

/// Set active R4G1 binary container bytes for zero-multiply prediction.
pub fn set_r4g1_bytes(bytes: Vec<u8>) {
    if let Err(error) = try_set_r4g1_bytes(bytes) {
        println!("[-] set_r4g1_bytes: {error}");
    }
}

/// Atomically install a graph and its exact deployed tokenizer. A nonzero
/// HEAD CID must equal BLAKE3 of `tokenizer_bytes`; malformed or swapped input
/// returns an error without replacing the previously active bundle.
pub fn set_r4g1_bundle(graph: Vec<u8>, tokenizer_bytes: Vec<u8>) -> Result<(), String> {
    let expected = validated_graph_tokenizer_cid(&graph)?;
    let actual = blake3::hash(&tokenizer_bytes);
    if expected != [0; 32] && expected != *actual.as_bytes() {
        return Err(format!(
            "R4G1 tokenizer CID mismatch: header expected blake3:{}, loaded blake3:{actual}",
            blake3::Hash::from(expected).to_hex()
        ));
    }
    if expected == [0; 32]
        && uor_r4_core::transformerless::scenarios::Tokenizer::is_tagged_container_bytes(
            &tokenizer_bytes,
        )
    {
        return Err("a tagged tokenizer requires a nonzero R4G1 header tokenizer CID".to_owned());
    }
    let tokenizer =
        uor_r4_core::transformerless::scenarios::Tokenizer::from_bytes(&tokenizer_bytes)
            .ok_or_else(|| "invalid tokenizer.bin bytes".to_owned())?;
    let mut guard = OWNED_R4G1
        .write()
        .map_err(|_| "R4G1 bundle lock poisoned".to_owned())?;
    *guard = Some(OwnedR4g1Bundle {
        graph,
        tokenizer: OwnedR4g1Tokenizer::Exact(tokenizer),
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn ensure_owned_tless() {
    OWNED_TLESS.with(|state| {
        if state.borrow().is_none() {
            let candidates = [
                ".uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin",
                ".uor-models/compiled/smollm2-360m-instruct/tless_artifacts.bin",
                "/tmp/tless_artifacts.bin",
            ];
            let store_candidates = [
                ".uor-models/compiled/smollm2-135m-instruct/tless_store.bin",
                ".uor-models/compiled/smollm2-360m-instruct/tless_store.bin",
                "/tmp/tless_store.bin",
            ];
            let tok_candidates = [
                ".uor-models/compiled/smollm2-135m-instruct/tokenizer.bin",
                ".uor-models/compiled/smollm2-360m-instruct/tokenizer.bin",
                "/tmp/ref/tokenizer.bin",
            ];
            for ((art_path, store_path), tok_path) in candidates
                .iter()
                .zip(store_candidates.iter())
                .zip(tok_candidates.iter())
            {
                if std::path::Path::new(art_path).exists()
                    && std::path::Path::new(store_path).exists()
                {
                    if let (Ok(art_bytes), Ok(store_bytes)) =
                        (std::fs::read(art_path), std::fs::read(store_path))
                    {
                        #[allow(deprecated)]
                        if let (Some(art), Some(store)) = (
                            uor_r4_core::transformerless::compiler::parse_artifacts(&art_bytes),
                            uor_r4_core::transformerless::runtime::parse_store(&store_bytes)
                                .or_else(|| {
                                    uor_r4_core::transformerless::runtime::parse_store_legacy_u16(
                                        &store_bytes,
                                    )
                                }),
                        ) {
                            *state.borrow_mut() = Some(make_tless_state(art, store));
                            if std::path::Path::new(tok_path).exists() {
                                TLESS_TOKENIZER.with(|tk| {
                                    *tk.borrow_mut() =
                                        uor_r4_core::transformerless::scenarios::Tokenizer::try_load(
                                            tok_path,
                                        )
                                        .ok();
                                });
                            }
                            break;
                        }
                    }
                }
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn ensure_owned_tless() {}

pub fn generate_r4g1_response(prompt: &str, max_tokens: usize) -> Option<String> {
    generate_r4g1_response_with_session_signature(prompt, max_tokens, None)
}

/// Generate through the graph runtime with an optional server-side session
/// signature. The context signature remains the ROUT input; the session lane
/// is consumed by the existing emission-affinity bonus.
pub fn generate_r4g1_response_with_session_signature(
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> Option<String> {
    let guard = match OWNED_R4G1.read() {
        Ok(g) => g,
        Err(_) => {
            println!("[-] generate_r4g1_response: lock poisoned");
            return None;
        }
    };
    let bundle = match guard.as_ref() {
        Some(bundle) => bundle,
        None => {
            println!("[-] generate_r4g1_response: OWNED_R4G1 is None");
            return None;
        }
    };
    let runtime = match uor_r4_graph_runtime::R4G1Runtime::parse(&bundle.graph) {
        Ok(r) => r,
        Err(e) => {
            println!("[-] generate_r4g1_response: runtime parse failed: {:?}", e);
            return None;
        }
    };

    let tokens = match &bundle.tokenizer {
        OwnedR4g1Tokenizer::LegacyGlobal => tless_tokenize(prompt),
        OwnedR4g1Tokenizer::Exact(tokenizer) => {
            let mut tokens = vec![0u32; prompt.len().saturating_add(2)];
            tokenizer.encode_into(prompt, &mut tokens).map(|count| {
                tokens.truncate(count);
                tokens
            })
        }
    };
    let tokens = match tokens {
        Some(t) => t,
        None => {
            println!("[-] generate_r4g1_response: tokenize failed");
            return None;
        }
    };

    let num_nodes = runtime.node_count() as usize;
    let mut node_scores = vec![uor_r4_core::transformerless::score_q::ScoreQ::MIN; num_nodes];

    ensure_owned_tless();

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

    let steps = max_tokens.min(128);
    for _ in 0..steps {
        let mut all_candidates = Vec::new();
        let mut any_active = false;

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

            let mut beam_tokens = tokens.clone();
            beam_tokens.extend_from_slice(&beam.tokens);

            let sig = OWNED_TLESS.with(|ts| {
                if let Some(ts) = &*ts.borrow() {
                    let rot = uor_r4_core::transformerless::compiler::derive_rotations();
                    let mut window = Vec::new();
                    let len = core::cmp::min(
                        beam_tokens.len(),
                        uor_r4_core::transformerless::compiler::WINDOW,
                    );
                    window.extend_from_slice(&beam_tokens[beam_tokens.len() - len..]);
                    let bundle = uor_r4_core::transformerless::runtime::bundle_window_plain(
                        &ts.art, &rot, &window,
                    );
                    Some(uor_r4_core::transformerless::runtime::sig_plain(
                        &ts.art, &bundle,
                    ))
                } else {
                    None
                }
            });

            let mut cands = [(0u32, uor_r4_core::transformerless::score_q::ScoreQ::ZERO); 8];
            let num_cands = runtime.predict_candidates_with_signature_lanes(
                &beam_tokens,
                sig.as_ref().map(|s| &s[..]),
                session_signature,
                &mut node_scores,
                &mut cands,
            );

            for &(cand_tok, cand_score) in &cands[..num_cands] {
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

        if !any_active || all_candidates.is_empty() {
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

    if best_beam.tokens.is_empty() {
        Some("R4G1 zero-multiply prediction complete.".to_string())
    } else {
        match &bundle.tokenizer {
            OwnedR4g1Tokenizer::LegacyGlobal => tless_detokenize(&best_beam.tokens),
            OwnedR4g1Tokenizer::Exact(tokenizer) => {
                let mut bytes = vec![0u8; 16 * 1024];
                tokenizer
                    .decode_into(&best_beam.tokens, &mut bytes)
                    .map(|count| String::from_utf8_lossy(&bytes[..count]).into_owned())
            }
        }
    }
}

/// Build a state from a loaded artifact + store (κs and address computed).
pub fn make_tless_state(art: Compiled, store: Store) -> TlessState {
    let bytes = compiler::artifact_bytes(&art);
    let artifact_kappa = compiler::artifact_kappa(&art);
    let artifact_address = address_container(&bytes).unwrap_or_default();
    let store_kappa = runtime::store_kappa(&store);
    TlessState {
        art,
        store,
        artifact_kappa,
        artifact_address,
        store_kappa,
    }
}

/// Bind a live state pointer on this thread (mirrors the ACTIVE_ROUTER
/// pattern). Contract: the pointee must stay live and unaliased for the
/// duration of the binding — the server binds from a held MutexGuard and
/// unbinds before releasing it; tests bind a leaked box.
pub fn bind_tless_state(ptr: *mut TlessState) {
    ACTIVE_TLESS.with(|s| *s.borrow_mut() = Some(ptr));
}

pub fn unbind_tless_state() {
    ACTIVE_TLESS.with(|s| *s.borrow_mut() = None);
}

/// Install an owned state on this thread without leaking a heap allocation.
pub fn set_tless_state(art: Compiled, store: Store) {
    OWNED_TLESS.with(|state| *state.borrow_mut() = Some(make_tless_state(art, store)));
    TLESS_TOKENIZER.with(|tk| *tk.borrow_mut() = None);
}

/// Explicit path configuration (the server CLI); each accessor falls back
/// to the environment variable, then the default.
pub struct TlessPaths {
    pub artifacts: String,
    pub store: String,
    pub tokenizer: String,
}

static TLESS_PATHS: std::sync::OnceLock<TlessPaths> = std::sync::OnceLock::new();

/// Pin explicit paths (first call wins; tests use `set_tless_state`).
pub fn configure_tless_paths(paths: TlessPaths) {
    let _ = TLESS_PATHS.set(paths);
    TLESS_TOKENIZER.with(|tk| *tk.borrow_mut() = None);
}

fn artifacts_path() -> String {
    TLESS_PATHS
        .get()
        .map(|p| p.artifacts.clone())
        .or_else(|| std::env::var("TLESS_ARTIFACTS").ok())
        .unwrap_or_else(|| "/tmp/tless_artifacts.bin".to_string())
}

fn store_path() -> String {
    TLESS_PATHS
        .get()
        .map(|p| p.store.clone())
        .or_else(|| std::env::var("TLESS_STORE").ok())
        .unwrap_or_else(|| "/tmp/tless_store.bin".to_string())
}

fn tokenizer_path() -> String {
    TLESS_PATHS
        .get()
        .map(|p| p.tokenizer.clone())
        .or_else(|| std::env::var("TLESS_TOKENIZER").ok())
        .unwrap_or_else(|| {
            if std::path::Path::new(".uor-models/compiled/smollm2-135m-instruct/tokenizer.bin")
                .exists()
            {
                ".uor-models/compiled/smollm2-135m-instruct/tokenizer.bin".to_string()
            } else if std::path::Path::new(
                ".uor-models/compiled/smollm2-360m-instruct/tokenizer.bin",
            )
            .exists()
            {
                ".uor-models/compiled/smollm2-360m-instruct/tokenizer.bin".to_string()
            } else {
                "/tmp/ref/tokenizer.bin".to_string()
            }
        })
}

/// Load state bytes from the configured paths (explicit config, then env
/// TLESS_ARTIFACTS / TLESS_STORE, then the /tmp defaults).
pub fn load_tless_state() -> Option<TlessState> {
    let art_path = artifacts_path();
    let store_path = store_path();
    println!(
        "[*] Loading tless state from art={} and store={}",
        art_path, store_path
    );
    let art_bytes = match std::fs::read(&art_path) {
        Ok(b) => b,
        Err(e) => {
            println!("[-] Failed to read artifacts file at {}: {:?}", art_path, e);
            return None;
        }
    };
    let art = match compiler::parse_artifacts(&art_bytes) {
        Some(a) => a,
        None => {
            println!("[-] Failed to parse artifacts from {}", art_path);
            return None;
        }
    };
    let store_bytes = match std::fs::read(&store_path) {
        Ok(b) => b,
        Err(e) => {
            println!("[-] Failed to read store file at {}: {:?}", store_path, e);
            return None;
        }
    };
    let store = match runtime::parse_store(&store_bytes) {
        Some(s) => s,
        None => {
            println!("[-] Failed to parse store from {}", store_path);
            return None;
        }
    };
    println!("[+] Successfully loaded tless state (online)!");
    Some(make_tless_state(art, store))
}

/// Load-and-bind from the default paths if unbound (single-thread tools;
/// the server binds explicitly around its shared Mutex).
#[cfg(not(target_arch = "wasm32"))]
pub fn ensure_tless_state() -> bool {
    if ACTIVE_TLESS.with(|state| state.borrow().is_some())
        || OWNED_TLESS.with(|state| state.borrow().is_some())
    {
        return true;
    }
    match load_tless_state() {
        Some(st) => {
            OWNED_TLESS.with(|state| *state.borrow_mut() = Some(st));
            true
        }
        None => false,
    }
}

/// On WASM there is no filesystem; state must be injected with
/// `set_tless_state`.
#[cfg(target_arch = "wasm32")]
pub fn ensure_tless_state() -> bool {
    ACTIVE_TLESS.with(|state| state.borrow().is_some())
        || OWNED_TLESS.with(|state| state.borrow().is_some())
}

/// Read-only access to the bound state.
pub fn with_tless_state<R>(f: impl FnOnce(&TlessState) -> R) -> Option<R> {
    // SAFETY: the binding contract above — the pointee is live and
    // unaliased for the binding's duration (server: MutexGuard held;
    // tests: leaked box).
    let active = ACTIVE_TLESS.with(|state| *state.borrow());
    match active {
        Some(pointer) => Some(f(unsafe { &*pointer })),
        None => OWNED_TLESS.with(|state| state.borrow().as_ref().map(f)),
    }
}

/// Mutable access to the bound state (same contract as
/// `with_tless_state`; the server holds the state Mutex across the call).
fn with_tless_state_mut<R>(f: impl FnOnce(&mut TlessState) -> R) -> Option<R> {
    let active = ACTIVE_TLESS.with(|state| *state.borrow());
    match active {
        Some(pointer) => Some(f(unsafe { &mut *pointer })),
        None => OWNED_TLESS.with(|state| state.borrow_mut().as_mut().map(f)),
    }
}

// =====================================================================
// Content addressing (uor-addr, CBOR realization, blake3 axis)
// =====================================================================

fn cbor_header(out: &mut Vec<u8>, major: u8, n: u64) {
    if n < 24 {
        out.push((major << 5) | n as u8);
    } else if n < 256 {
        out.push((major << 5) | 24);
        out.push(n as u8);
    } else if n < 65536 {
        out.push((major << 5) | 25);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else {
        out.push((major << 5) | 26);
        out.extend_from_slice(&(n as u32).to_be_bytes());
    }
}

fn cbor_byte_string(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 9);
    cbor_header(&mut out, 2, bytes.len() as u64);
    out.extend_from_slice(bytes);
    out
}

/// Content-address a TLA artifact container: the container as a CBOR byte
/// string, addressed on the blake3 axis → "blake3:<hex>" κ-label.
pub fn address_container(tla5: &[u8]) -> Result<String, String> {
    uor_addr::cbor::address_blake3(&cbor_byte_string(tla5))
        .map(|out| out.address.as_str().to_string())
        .map_err(|e| format!("{e:?}"))
}

/// Content-address one graded store entry — key prefix at `depth` mapping
/// to (token → count) evidence — as canonical CBOR {"d","k","v"} on the
/// blake3 axis. Per-entry addresses are what attribution and deletion point
/// at: to remove a contribution is to remove its κ.
pub fn address_store_entry(
    depth: usize,
    key: &[u8],
    dist: &BTreeMap<u32, u32>,
) -> Result<String, String> {
    let mut item = Vec::with_capacity(16 + key.len() + dist.len() * 7);
    cbor_header(&mut item, 5, 3); // map(3)
    item.push(0x61);
    item.push(b'd');
    cbor_header(&mut item, 0, depth as u64);
    item.push(0x61);
    item.push(b'k');
    cbor_header(&mut item, 2, key.len() as u64);
    item.extend_from_slice(key);
    item.push(0x61);
    item.push(b'v');
    cbor_header(&mut item, 5, dist.len() as u64);
    for (&t, &c) in dist {
        cbor_header(&mut item, 0, t as u64);
        cbor_header(&mut item, 0, c as u64);
    }
    uor_addr::cbor::address_blake3(&item)
        .map(|out| out.address.as_str().to_string())
        .map_err(|e| format!("{e:?}"))
}

/// Address-attested deletion: remove the graded store entry at
/// (depth, key), returning its pre-removal κ-address and evidence. The
/// returned address is the deletion attestation — the κ that was removed.
pub fn delete_store_entry(depth: usize, key: &[u8]) -> Option<(String, BTreeMap<u32, u32>)> {
    with_tless_state_mut(|st| {
        let dist = st.store.get(depth)?.get(key)?.clone();
        let addr = address_store_entry(depth, key, &dist).ok()?;
        runtime::remove_entry(&mut st.store, depth, key);
        st.store_kappa = runtime::store_kappa(&st.store);
        Some((addr, dist))
    })?
}

// =====================================================================
// Online indexing & generation: the graded store as knowledge substrate
// =====================================================================

thread_local! {
    static TLESS_TOKENIZER: RefCell<Option<uor_r4_core::transformerless::scenarios::Tokenizer>> = const { RefCell::new(None) };
}

/// Inject a tokenizer (WASM, tests).
pub fn set_tless_tokenizer(t: uor_r4_core::transformerless::scenarios::Tokenizer) {
    TLESS_TOKENIZER.with(|tk| *tk.borrow_mut() = Some(t));
}

// `tokenizer.bin` is the deployed decoder. Historical untagged artifacts keep
// their existing encoder; tagged registered artifacts refuse encoding and
// require an exact host adapter at the server/R4G1 boundary.
#[cfg(not(target_arch = "wasm32"))]
fn with_tokenizer<R>(
    f: impl FnOnce(&uor_r4_core::transformerless::scenarios::Tokenizer) -> R,
) -> Option<R> {
    TLESS_TOKENIZER.with(|t| {
        let mut g = t.borrow_mut();
        if g.is_none() {
            let path = tokenizer_path();
            if std::fs::metadata(&path).is_ok() {
                *g = uor_r4_core::transformerless::scenarios::Tokenizer::try_load(&path).ok();
            }
        }
        g.as_ref().map(f)
    })
}

#[cfg(target_arch = "wasm32")]
fn with_tokenizer<R>(
    f: impl FnOnce(&uor_r4_core::transformerless::scenarios::Tokenizer) -> R,
) -> Option<R> {
    TLESS_TOKENIZER.with(|t| t.borrow().as_ref().map(f))
}

/// Tokenize text (BOS-prefixed) with the bound tokenizer.
pub fn tless_tokenize(text: &str) -> Option<Vec<u32>> {
    with_tokenizer(|tokenizer| {
        if tokenizer.is_decode_only() {
            None
        } else {
            Some(tokenizer.encode(text))
        }
    })
    .flatten()
}

/// Tokenize into caller-owned storage without allocating.
pub fn tless_tokenize_into(text: &str, out: &mut [u32]) -> Option<usize> {
    with_tokenizer(|tokenizer| tokenizer.encode_into(text, out)).flatten()
}

/// Detokenize token ids with the bound tokenizer.
pub fn tless_detokenize(tokens: &[u32]) -> Option<String> {
    with_tokenizer(|t| t.decode(tokens))
}

/// Detokenize into caller-owned byte storage without allocating.
pub fn tless_detokenize_into(tokens: &[u32], out: &mut [u8]) -> Option<usize> {
    with_tokenizer(|tokenizer| tokenizer.decode_into(tokens, out)).flatten()
}

/// Index a token stream into the bound graded store as additional evidence
/// (document-isolated: context never crosses the stream start). Returns the
/// number of (code, next) evidence positions written. The store κ changes —
/// that change is the attestation trail of what was learned.
pub fn index_token_stream(tokens: &[u32]) -> Option<usize> {
    with_tless_state_mut(|st| {
        let rot = runtime::derive_rotations();
        let mut n = 0usize;
        for i in 0..tokens.len().saturating_sub(1) {
            let window = &tokens[i.saturating_sub(WINDOW - 1)..=i];
            let b = runtime::bundle_window_plain(&st.art, &rot, window);
            // Metric-respecting assignment (#243 Phase C): bundle-holding
            // callers go through assign_for_bundle so TLA6 artifacts index
            // under the same shift-add-dot codes the generation reader
            // queries. The old sig_plain→assign_plain route wrote evidence
            // under sign-metric codes that a dot-path reader never hits —
            // the writer/reader inconsistency the TLA5 fixtures masked.
            let code = runtime::assign_for_bundle(&st.art, &b);
            runtime::add_evidence(&mut st.store, &code, tokens[i + 1], 1);
            n += 1;
        }
        st.store_kappa = runtime::store_kappa(&st.store);
        n
    })
}

/// Greedy generation from a seed window against the bound store: per-step
/// witnesses (token, depth, evidence count), attributable by construction.
pub fn generate_steps(seed: &[u32], len: usize) -> Option<Vec<runtime::Prediction>> {
    with_tless_state(|st| {
        let mut rt = runtime::Runtime::new(&st.art);
        let mut predictions = vec![runtime::Prediction::default(); len];
        rt.generate_greedy_into(&st.store, seed, &mut predictions);
        predictions
    })
}

/// Allocation-free generation into caller-owned prediction storage.
pub fn generate_steps_into(seed: &[u32], out: &mut [runtime::Prediction]) -> Option<usize> {
    with_tless_state(|st| {
        let mut rt = runtime::Runtime::new(&st.art);
        rt.generate_greedy_into(&st.store, seed, out)
    })
}

// =====================================================================
pub const TLESS_INPUT_BYTES: usize = WINDOW * 4; // 32
pub const TLESS_OUTPUT_BYTES: usize = 37;

uor_foundation_sdk::axis! {
    /// Mul-free table-native prediction axis (transformerless runtime).
    pub trait TlessAxis: AxisExtension {
        const AXIS_ADDRESS: &'static str = "https://uor.foundation/axis/TlessAxis";
        const MAX_OUTPUT_BYTES: usize = 40;
        fn predict(input: &[u8], out: &mut [u8]) -> Result<usize, ShapeViolation>;
    }
}

pub struct TlessAxisImpl;

fn tless_violation(constraint: &'static str, min: usize, max: usize) -> ShapeViolation {
    ShapeViolation {
        shape_iri: <TlessAxisImpl as TlessAxis>::AXIS_ADDRESS,
        constraint_iri: constraint,
        property_iri: "https://uor.foundation/axis/inputBytes",
        expected_range: "https://uor.foundation/axis/Bytes32",
        min_count: min as u32,
        max_count: max as u32,
        kind: uor_foundation::ViolationKind::ValueCheck,
    }
}

impl TlessAxis for TlessAxisImpl {
    const AXIS_ADDRESS: &'static str = "https://uor.foundation/axis/TlessAxis/Impl";
    const MAX_OUTPUT_BYTES: usize = 40;

    /// input: WINDOW u32 token ids, little-endian, oldest first.
    /// output (37 bytes, big-endian fields): token u32 | depth u8 |
    /// code [u8; 4] | count u32 | adds | xors | shifts | compares |
    /// table_reads | candidate_scans (u32 each). No multiply field exists, by design.
    fn predict(input: &[u8], out: &mut [u8]) -> Result<usize, ShapeViolation> {
        if input.len() < TLESS_INPUT_BYTES {
            return Err(tless_violation(
                "https://uor.foundation/axis/TlessAxis/inputSize",
                TLESS_INPUT_BYTES,
                TLESS_INPUT_BYTES,
            ));
        }
        if out.len() < TLESS_OUTPUT_BYTES {
            return Err(tless_violation(
                "https://uor.foundation/axis/TlessAxis/outputSize",
                TLESS_OUTPUT_BYTES,
                TLESS_OUTPUT_BYTES,
            ));
        }
        let mut window = [0u32; WINDOW];
        for (i, w) in window.iter_mut().enumerate() {
            *w = u32::from_le_bytes([
                input[4 * i],
                input[4 * i + 1],
                input[4 * i + 2],
                input[4 * i + 3],
            ]);
        }
        with_tless_state(|st| {
            let mut rt = runtime::Runtime::new(&st.art);
            let (code, by_depth) = rt.assign_window_memberships(&window);

            let mut priors = std::collections::HashMap::new();
            let mut query_text = String::new();

            uor_r4_router::ACTIVE_ROUTER.with(|r| {
                if let Some(ptr) = *r.borrow() {
                    let router = unsafe { &mut *ptr };
                    for &tok_id in &window {
                        if (tok_id as usize) < router.vocabulary.len() {
                            query_text.push_str(&router.vocabulary[tok_id as usize]);
                            query_text.push(' ');
                        }
                    }
                    if !query_text.is_empty() {
                        let resonances = router.get_top_resonances_native(&query_text, "shared", 5);
                        let mut word_to_tok = std::collections::HashMap::new();
                        for (idx, word) in router.vocabulary.iter().enumerate() {
                            word_to_tok.insert(word.to_lowercase(), idx as u32);
                        }
                        for res in resonances {
                            for word in res.sentence.split_whitespace() {
                                let cleaned: String = word
                                    .to_lowercase()
                                    .chars()
                                    .filter(|c| c.is_alphanumeric())
                                    .collect();
                                if let Some(&tok_id) = word_to_tok.get(&cleaned) {
                                    *priors.entry(tok_id).or_insert(0) += 5;
                                }
                            }
                        }
                    }
                }
            });

            let p = if priors.is_empty() {
                rt.predict_witness_beam(&st.store, &by_depth)
            } else {
                rt.predict_witness_with_priors_beam(&st.store, &by_depth, &priors)
            };

            let k = &rt.kernel;
            out[0..4].copy_from_slice(&p.token.to_be_bytes());
            out[4] = p.depth;
            out[5..9].copy_from_slice(&code);
            out[9..13].copy_from_slice(&p.count.to_be_bytes());
            out[13..17].copy_from_slice(&(k.adds as u32).to_be_bytes());
            out[17..21].copy_from_slice(&(k.xors as u32).to_be_bytes());
            out[21..25].copy_from_slice(&(k.shifts as u32).to_be_bytes());
            out[25..29].copy_from_slice(&(k.compares as u32).to_be_bytes());
            out[29..33].copy_from_slice(&(k.table_reads as u32).to_be_bytes());
            out[33..37].copy_from_slice(&(k.candidate_scans as u32).to_be_bytes());
            TLESS_OUTPUT_BYTES
        })
        .ok_or(ShapeViolation {
            shape_iri: <TlessAxisImpl as TlessAxis>::AXIS_ADDRESS,
            constraint_iri: "https://uor.foundation/axis/TlessAxis/stateBound",
            property_iri: "https://uor.foundation/axis/tlessState",
            expected_range: "https://uor.foundation/axis/TlessStateBound",
            min_count: 1,
            max_count: 1,
            kind: uor_foundation::ViolationKind::ValueCheck,
        })
    }
}

axis_extension_impl_for_tless_axis!(TlessAxisImpl);

// =====================================================================
// Shapes and PrismModel binding (mirrors R4RoutingInput/Output, R4Axis)
// =====================================================================

#[derive(Clone, Copy)]
pub struct TlessPredictInput<'a> {
    pub window: &'a [u8],
    pub data: &'a [u8], // packed WINDOW×u32 LE, 32 bytes
}

impl ConstrainedTypeShape for TlessPredictInput<'_> {
    const IRI: &'static str = "urn:uor:product:TlessPredictInput";
    const SITE_COUNT: usize = TLESS_INPUT_BYTES;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl uor_foundation::pipeline::__sdk_seal::Sealed for TlessPredictInput<'_> {}

impl<'a> IntoBindingValue<'a> for TlessPredictInput<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.data)
    }
}

impl PartitionProductFields for TlessPredictInput<'_> {
    const FIELDS: &'static [(u32, u32)] = &[(0, 32)];
    const FIELD_NAMES: &'static [&'static str] = &["window"];
}

#[derive(Debug, Clone, Copy)]
pub struct TlessPredictOutput;

impl ConstrainedTypeShape for TlessPredictOutput {
    const IRI: &'static str = "urn:uor:product:TlessPredictOutput";
    const SITE_COUNT: usize = TLESS_OUTPUT_BYTES;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl uor_foundation::pipeline::__sdk_seal::Sealed for TlessPredictOutput {}
impl GroundedShape for TlessPredictOutput {}

impl<'a> IntoBindingValue<'a> for TlessPredictOutput {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::empty()
    }
}

impl PartitionProductFields for TlessPredictOutput {
    const FIELDS: &'static [(u32, u32)] = &[
        (0, 4),
        (4, 1),
        (5, 4),
        (9, 4),
        (13, 4),
        (17, 4),
        (21, 4),
        (25, 4),
        (29, 4),
        (33, 4),
    ];
    const FIELD_NAMES: &'static [&'static str] = &[
        "token",
        "depth",
        "code",
        "count",
        "adds",
        "xors",
        "shifts",
        "compares",
        "table_reads",
        "candidate_scans",
    ];
}

/// Hasher + axis bundle (same construction as `R4HasherAndAxis`): finalize
/// runs the prediction kernel when a full window is buffered, else SHA-256.
#[derive(Clone)]
pub struct TlessHasherAndAxis {
    buffer: Vec<u8>,
}

impl Hasher<R4_FP_MAX> for TlessHasherAndAxis {
    const OUTPUT_BYTES: usize = R4_FP_MAX;

    fn initial() -> Self {
        Self { buffer: Vec::new() }
    }

    fn fold_byte(mut self, b: u8) -> Self {
        self.buffer.push(b);
        self
    }

    fn fold_bytes(mut self, bytes: &[u8]) -> Self {
        self.buffer.extend_from_slice(bytes);
        self
    }

    fn finalize(self) -> [u8; R4_FP_MAX] {
        let mut out = [0u8; R4_FP_MAX];
        if self.buffer.len() >= TLESS_INPUT_BYTES {
            if let Ok(_len) = TlessAxisImpl::predict(&self.buffer, &mut out) {
                // axis output is the fingerprint
            } else {
                let sha = uor_r4_core::sha256_bytes(&self.buffer);
                out.copy_from_slice(&sha);
            }
        } else {
            let sha = uor_r4_core::sha256_bytes(&self.buffer);
            out.copy_from_slice(&sha);
        }
        out
    }
}

pub struct UorTlessModel;
pub struct UorTlessRoute;

impl uor_foundation::pipeline::__sdk_seal::Sealed for UorTlessModel {}
impl uor_foundation::pipeline::__sdk_seal::Sealed for UorTlessRoute {}

impl uor_foundation::pipeline::FoundationClosed<R4_INLINE_BYTES> for UorTlessRoute {
    fn arena_slice() -> &'static [uor_foundation::enforcement::Term<'static, R4_INLINE_BYTES>] {
        &[
            uor_foundation::enforcement::Term::Variable { name_index: 0 },
            uor_foundation::enforcement::Term::AxisInvocation {
                axis_index: 0,
                kernel_id: 0,
                input_index: 0,
            },
        ]
    }
}

impl<'a>
    uor_foundation::pipeline::PrismModel<
        'a,
        uor_foundation::DefaultHostTypes,
        R4HostBounds,
        TlessHasherAndAxis,
        R4_INLINE_BYTES,
        R4_FP_MAX,
    > for UorTlessModel
{
    type Input = TlessPredictInput<'a>;
    type Output = TlessPredictOutput;
    type Route = UorTlessRoute;

    fn forward(
        input: Self::Input,
    ) -> Result<
        uor_foundation::enforcement::Grounded<'a, Self::Output, R4_INLINE_BYTES, R4_FP_MAX>,
        uor_foundation::PipelineFailure,
    > {
        uor_foundation::pipeline::run_route::<
            uor_foundation::DefaultHostTypes,
            R4HostBounds,
            TlessHasherAndAxis,
            Self,
            uor_foundation::pipeline::NullResolverTuple,
            uor_foundation::pipeline::EmptyCommitment,
            R4_INLINE_BYTES,
            R4_FP_MAX,
        >(
            input,
            &uor_foundation::pipeline::NullResolverTuple,
            &uor_foundation::pipeline::EmptyCommitment,
        )
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uor_foundation::pipeline::PrismModel;
    use uor_r4_core::transformerless::compiler::STAGES;
    use uor_r4_core::transformerless::scenarios::{
        export_runtime_tokenizer_table, RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable,
        RuntimeTokenizerEncodePolicy, RuntimeTokenizerIdentity, Tokenizer,
    };

    static R4G1_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn fixture_state() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/crates/uor-r4-core/tests/fixtures"
        );
        let bytes = std::fs::read(format!("{dir}/tless_artifacts.bin")).unwrap();
        let art = compiler::parse_artifacts(&bytes).expect("fixture container parses");
        let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
        store[0].entry(vec![]).or_default().insert(1, 10);
        set_tless_state(art, store);
    }

    fn tokenizer_bytes(tokens: &[&[u8]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for token in tokens {
            bytes.extend_from_slice(&(token.len() as i32).to_le_bytes());
            bytes.extend_from_slice(token);
        }
        bytes
    }

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
    fn owned_r4g1_installation_is_atomic_and_cid_bound() {
        let _test_guard = R4G1_TEST_LOCK.lock().expect("R4G1 test lock");
        *OWNED_R4G1.write().expect("bundle lock") = None;
        let tokenizer_a = tokenizer_bytes(&[b" ", b"a"]);
        let tokenizer_b = tokenizer_bytes(&[b" ", b"b"]);
        let graph = minimal_graph(*blake3::hash(&tokenizer_a).as_bytes());

        let error = try_set_r4g1_bytes(graph.clone())
            .expect_err("nonzero-CID graph-only install must be refused");
        assert!(
            error.contains("graph-only installation is refused"),
            "{error}"
        );
        assert!(OWNED_R4G1.read().expect("bundle lock").is_none());

        set_r4g1_bundle(graph.clone(), tokenizer_a).expect("exact bundle installs");
        let installed_hash = {
            let guard = OWNED_R4G1.read().expect("bundle lock");
            blake3::hash(&guard.as_ref().expect("installed bundle").graph)
        };
        let error = set_r4g1_bundle(graph, tokenizer_b)
            .expect_err("swapped tokenizer must not replace the active bundle");
        assert!(error.contains("tokenizer CID mismatch"), "{error}");
        let retained_hash = {
            let guard = OWNED_R4G1.read().expect("bundle lock");
            blake3::hash(&guard.as_ref().expect("retained bundle").graph)
        };
        assert_eq!(retained_hash, installed_hash);

        try_set_r4g1_bytes(minimal_graph([0; 32]))
            .expect("legacy zero-CID graph-only install remains supported");
        let guard = OWNED_R4G1.read().expect("bundle lock");
        assert!(matches!(
            guard.as_ref().map(|bundle| &bundle.tokenizer),
            Some(OwnedR4g1Tokenizer::LegacyGlobal)
        ));
    }

    #[test]
    fn tagged_runtime_tokenizer_decodes_but_never_panics_or_encodes() {
        let _test_guard = R4G1_TEST_LOCK.lock().expect("R4G1 test lock");
        let path = std::env::temp_dir().join(format!(
            "uor-r4-tless-decode-only-{}.bin",
            std::process::id()
        ));
        let table = RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: "future-sentencepiece-family".to_owned(),
                version: 41,
                tokenizer_cid: format!("blake3:{}", "5".repeat(64)),
                adapter_digest: format!("blake3:{}", "6".repeat(64)),
            },
            pieces: vec![Vec::new(), "▁hello".as_bytes().to_vec()],
            encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        export_runtime_tokenizer_table(&table, &path).expect("tagged export");
        let bytes = std::fs::read(&path).expect("read tagged");
        let tokenizer = Tokenizer::from_bytes(&bytes).expect("parse tagged");
        set_tless_tokenizer(tokenizer);
        assert_eq!(tless_tokenize("hello"), None);
        assert_eq!(tless_detokenize(&[1]).as_deref(), Some("hello"));
        let error = set_r4g1_bundle(minimal_graph([0; 32]), bytes)
            .expect_err("zero-CID graph cannot bind a tagged tokenizer");
        assert!(error.contains("nonzero"), "{error}");

        let legacy_bytes = tokenizer_bytes(&[b" ", b"a"]);
        let legacy = Tokenizer::from_bytes(&legacy_bytes).expect("usable legacy tokenizer");
        set_tless_tokenizer(legacy);
        assert!(
            tless_tokenize("a").is_some(),
            "the global legacy tokenizer must be demonstrably usable"
        );
        let tagged_bytes = std::fs::read(&path).expect("read tagged again");
        set_r4g1_bundle(
            minimal_graph(*blake3::hash(&tagged_bytes).as_bytes()),
            tagged_bytes,
        )
        .expect("matching nonzero-CID tagged bundle installs");
        assert_eq!(
            generate_r4g1_response("a", 1),
            None,
            "decode-only tagged bundle must not borrow the global legacy encoder"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn axis_predict_carries_census_witness() {
        fixture_state();
        let mut input = [0u8; TLESS_INPUT_BYTES];
        for (i, w) in [1u32, 2, 3, 4, 5, 6, 7, 8].iter().enumerate() {
            input[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
        }
        let mut out = [0u8; 40];
        let n = TlessAxisImpl::predict(&input, &mut out).expect("predict");
        assert_eq!(n, TLESS_OUTPUT_BYTES);
        let token = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
        let depth = out[4];
        let count = u32::from_be_bytes(out[9..13].try_into().unwrap());
        let adds = u32::from_be_bytes(out[13..17].try_into().unwrap());
        let table_reads = u32::from_be_bytes(out[29..33].try_into().unwrap());
        let candidate_scans = u32::from_be_bytes(out[33..37].try_into().unwrap());
        assert_eq!(token, 1, "only level-0 entry populated");
        assert_eq!(depth, 0, "synthetic store answers at level 0");
        assert_eq!(count, 10);
        assert!(
            adds > 0 && table_reads > 0 && candidate_scans > 0,
            "census recorded the path"
        );
        // No multiply field exists in the record: bytes 13..37 are exactly
        // the six census counters, and OpKernel has no multiply to count.
    }

    #[test]
    fn grounded_mints_and_replays() {
        fixture_state();
        let mut buf = [0u8; TLESS_INPUT_BYTES];
        for (i, w) in [1u32, 2, 3, 4, 5, 6, 7, 8].iter().enumerate() {
            buf[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
        }
        let input = TlessPredictInput {
            window: &buf,
            data: &buf,
        };
        let grounded = UorTlessModel::forward(input).expect("forward");
        let trace = grounded.derivation().replay::<256>();
        let certified = uor_foundation_verify::verify_trace(&trace).expect("replay verifies");
        assert_eq!(
            certified.certificate().content_fingerprint().as_bytes(),
            grounded.content_fingerprint().as_bytes(),
            "replayed derivation re-certifies bit-identically"
        );
    }

    #[test]
    fn indexing_and_generation_update_store() {
        fixture_state();
        let kappa_before = with_tless_state(|st| st.store_kappa.clone()).unwrap();
        let n = index_token_stream(&[1, 5, 6, 7]).expect("state bound");
        assert_eq!(n, 3, "three (code, next) evidence positions");
        let kappa_after = with_tless_state(|st| st.store_kappa.clone()).unwrap();
        assert_ne!(kappa_before, kappa_after, "store κ moved with the evidence");

        // the store replays the indexed stream at full depth; how the
        // UNSEEN continuation resolves is a fixture-era property: graded
        // backoff to depth 1 (b142c93-era / Linux-bot TLA5), depth 3 /
        // token 5 (macOS TLA5 re-pin, 2026-07-21), full-depth code-space
        // collision to the key's argmax 7 (1-term TLA6 fixture, #243
        // Phase C re-pin, 2026-07-30), and on the TLA7 residual-wired
        // 500k-corpus fixture (#327 re-pin, 2026-08-01) graded backoff to
        // depth 1 answering token 5 — the residual-wired assignment no
        // longer maps the novel window onto an existing full-depth path.
        let steps = generate_steps(&[1], 4).expect("generate");
        let tokens: Vec<u32> = steps.iter().map(|p| p.token).collect();
        let depths: Vec<u8> = steps.iter().map(|p| p.depth).collect();
        assert_eq!(tokens, vec![5, 6, 7, 5]);
        assert_eq!(
            depths,
            vec![4, 4, 4, 1],
            "indexed stream replays at full depth; the novel window backs off to depth 1 on this fixture"
        );
    }

    #[test]
    fn deletion_is_address_attested() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/crates/uor-r4-core/tests/fixtures"
        );
        let bytes = std::fs::read(format!("{dir}/tless_artifacts.bin")).unwrap();
        let art = compiler::parse_artifacts(&bytes).expect("fixture container parses");
        let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
        store[0].entry(vec![]).or_default().insert(1, 10);
        store[1].entry(vec![9]).or_default().insert(2, 5);
        let pre_addr = address_store_entry(1, &[9], store[1].get(&vec![9u8]).unwrap()).unwrap();
        let pre_store_kappa = runtime::store_kappa(&store);
        set_tless_state(art, store);

        let (addr, dist) = delete_store_entry(1, &[9]).expect("entry exists");
        assert_eq!(addr, pre_addr, "the attestation is the removed entry's κ");
        assert_eq!(dist.get(&2), Some(&5), "evidence returned");
        with_tless_state(|st| {
            assert!(!st.store[1].contains_key(&vec![9u8]), "entry removed");
            assert_ne!(st.store_kappa, pre_store_kappa, "store κ updated");
            let p = runtime::predict_witness_plain(&st.store, &[9, 0, 0, 0]);
            assert_eq!((p.token, p.depth), (1, 0), "resolution backs off");
        });
        assert!(delete_store_entry(1, &[9]).is_none(), "already gone");
    }

    #[test]
    fn addressing_is_stable_and_distinct() {
        let a1 = address_container(b"TLA3-test").expect("address");
        let a2 = address_container(b"TLA3-test").expect("address");
        assert_eq!(a1, a2, "content addressing is deterministic");
        assert!(a1.starts_with("blake3:"));
        let mut d1 = BTreeMap::new();
        d1.insert(1u32, 10u32);
        let e1 = address_store_entry(0, &[], &d1).expect("entry");
        let e2 = address_store_entry(1, &[9], &d1).expect("entry");
        assert!(e1.starts_with("blake3:") && e2.starts_with("blake3:"));
        assert_ne!(e1, e2, "distinct entries have distinct κ");
    }
}
