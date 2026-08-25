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

use uor_r4_router::{R4HostBounds, TokenHistorySignature, R4_FP_MAX, R4_INLINE_BYTES};

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

struct OwnedLegacyR4g1Bundle {
    graph: Vec<u8>,
    tokenizer: OwnedR4g1Tokenizer,
}

struct OwnedProductionR4g1Bundle {
    graph: Vec<u8>,
    signature_artifact: Vec<u8>,
    tokenizer_bytes: Vec<u8>,
    tokenizer: uor_r4_core::transformerless::scenarios::Tokenizer,
    score_report: Vec<u8>,
    deployed_quality_report: Vec<u8>,
    verified_envelope: uor_r4_api::VerifiedProductionEnvelope,
}

enum OwnedR4g1Bundle {
    Legacy(OwnedLegacyR4g1Bundle),
    Production(Box<OwnedProductionR4g1Bundle>),
}

static OWNED_R4G1: std::sync::RwLock<Option<OwnedR4g1Bundle>> = std::sync::RwLock::new(None);

#[derive(Debug, Clone, Copy)]
struct ValidatedR4g1Graph {
    tokenizer_cid: [u8; 32],
    skipmix_present: bool,
    psi_bag_present: bool,
}

fn validate_r4g1_graph(bytes: &[u8]) -> Result<ValidatedR4g1Graph, String> {
    let view = uor_r4_graph_format::GraphView::parse(bytes)
        .map_err(|error| format!("invalid R4G1 graph: {}", error.reason))?;
    view.verify_cids()
        .map_err(|error| format!("invalid R4G1 graph: {}", error.as_format()))?;
    let head = view
        .head()
        .ok_or_else(|| "invalid R4G1 graph: missing HEAD section".to_owned())?;
    let runtime = uor_r4_graph_runtime::R4G1Runtime::parse(bytes)
        .map_err(|error| format!("invalid R4G1 runtime graph: {}", error.reason))?;
    let (skipmix_present, psi_bag_present) = runtime.skipmix_tables_present();
    Ok(ValidatedR4g1Graph {
        tokenizer_cid: head.tokenizer_cid().0,
        skipmix_present,
        psi_bag_present,
    })
}

fn require_legacy_graph(graph: ValidatedR4g1Graph) -> Result<(), String> {
    if graph.skipmix_present || graph.psi_bag_present {
        return Err(
            "lane-bearing R4G1 graphs require the schema-2 production-envelope installer; legacy graph/tokenizer installation is refused"
                .to_owned(),
        );
    }
    Ok(())
}

/// Try to install graph-only bytes. This compatibility surface accepts only
/// legacy graphs whose HEAD tokenizer CID is zero; a bound graph must use
/// [`set_r4g1_bundle`] so graph and tokenizer enter the process atomically.
pub fn try_set_r4g1_bytes(bytes: Vec<u8>) -> Result<(), String> {
    let graph = validate_r4g1_graph(&bytes)?;
    require_legacy_graph(graph)?;
    if graph.tokenizer_cid != [0; 32] {
        return Err(format!(
            "R4G1 graph requires tokenizer.bin blake3:{}; graph-only installation is refused",
            blake3::Hash::from(graph.tokenizer_cid).to_hex()
        ));
    }
    let mut guard = OWNED_R4G1
        .write()
        .map_err(|_| "R4G1 bundle lock poisoned".to_owned())?;
    *guard = Some(OwnedR4g1Bundle::Legacy(OwnedLegacyR4g1Bundle {
        graph: bytes,
        tokenizer: OwnedR4g1Tokenizer::LegacyGlobal,
    }));
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
    let validated = validate_r4g1_graph(&graph)?;
    require_legacy_graph(validated)?;
    let expected = validated.tokenizer_cid;
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
    *guard = Some(OwnedR4g1Bundle::Legacy(OwnedLegacyR4g1Bundle {
        graph,
        tokenizer: OwnedR4g1Tokenizer::Exact(tokenizer),
    }));
    Ok(())
}

/// Atomically verify and install one complete schema-2 production generation.
/// Every slice is required and content-bound; an error leaves the previously
/// active generation untouched.
#[allow(clippy::too_many_arguments)]
pub fn set_r4g1_production_bundle(
    graph: Vec<u8>,
    sections_absent_graph: Vec<u8>,
    label_shuffled_graph: Vec<u8>,
    signature_artifact: Vec<u8>,
    tla_comparator_store: Vec<u8>,
    tokenizer_bytes: Vec<u8>,
    score_report: Vec<u8>,
    compile_report: Vec<u8>,
    deployed_quality_report: Vec<u8>,
    cross_surface_parity: Vec<u8>,
    witness_replay: Vec<u8>,
    corpus_meta: Vec<u8>,
    corpus_records: Vec<u8>,
    tokenizer_adapter: Vec<u8>,
    release_manifest: Vec<u8>,
) -> Result<(), String> {
    let validated = validate_r4g1_graph(&graph)?;
    if !validated.skipmix_present || !validated.psi_bag_present {
        return Err(
            "schema-2 production R4G1 admission requires both SKMX and PSIB runtime lanes"
                .to_owned(),
        );
    }
    let actual_tokenizer = blake3::hash(&tokenizer_bytes);
    if validated.tokenizer_cid == [0; 32] || validated.tokenizer_cid != *actual_tokenizer.as_bytes()
    {
        return Err(format!(
            "production R4G1 tokenizer CID mismatch: header expected blake3:{}, loaded blake3:{actual_tokenizer}",
            blake3::Hash::from(validated.tokenizer_cid).to_hex()
        ));
    }
    let tokenizer =
        uor_r4_core::transformerless::scenarios::Tokenizer::from_bytes(&tokenizer_bytes)
            .ok_or_else(|| "invalid production tokenizer.bin bytes".to_owned())?;
    let verified = uor_r4_api::verify_production_envelope(uor_r4_api::ProductionEnvelopeParts {
        graph: &graph,
        sections_absent_graph: &sections_absent_graph,
        label_shuffled_graph: &label_shuffled_graph,
        signature_artifact: &signature_artifact,
        tla_comparator_store: &tla_comparator_store,
        tokenizer: &tokenizer_bytes,
        score_report: &score_report,
        compile_report: &compile_report,
        deployed_quality_report: &deployed_quality_report,
        cross_surface_parity: &cross_surface_parity,
        witness_replay: &witness_replay,
        corpus_meta: &corpus_meta,
        corpus_records: &corpus_records,
        tokenizer_adapter: &tokenizer_adapter,
        release_manifest: &release_manifest,
    })
    .map_err(|error| error.to_string())?;
    let mut guard = OWNED_R4G1
        .write()
        .map_err(|_| "R4G1 bundle lock poisoned".to_owned())?;
    *guard = Some(OwnedR4g1Bundle::Production(Box::new(
        OwnedProductionR4g1Bundle {
            graph,
            signature_artifact,
            tokenizer_bytes,
            tokenizer,
            score_report,
            deployed_quality_report,
            verified_envelope: verified,
        },
    )));
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

/// Typed result from the explicitly non-production legacy compatibility path.
///
/// A lane-absent graph can still be replayed for historical and migration
/// studies, but its text is not a deployed-serving result: it has no schema-2
/// production envelope, deployed-quality report, or replay-bound admission
/// evidence. Callers must opt in to this separate surface and retain the
/// warning with any observation they record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R4g1ResearchResponse {
    pub text: String,
    pub warning: &'static str,
}

pub const LEGACY_R4G1_RESEARCH_WARNING: &str =
    "RESEARCH ONLY: legacy lane-absent R4G1 generation is not production-admitted evidence";

/// Replay an installed lane-absent legacy graph for compatibility research.
///
/// This is intentionally distinct from [`generate_r4g1_response`], whose
/// public/WASM contract is production-only. Lane-bearing artifacts are also
/// refused here because they must enter through the schema-2 production
/// envelope instead of a weaker research path.
pub fn generate_legacy_r4g1_research_response(
    prompt: &str,
    max_tokens: usize,
) -> Result<R4g1ResearchResponse, String> {
    let guard = OWNED_R4G1
        .read()
        .map_err(|_| "R4G1 bundle lock poisoned".to_owned())?;
    let bundle = guard
        .as_ref()
        .ok_or_else(|| "no R4G1 bundle is installed".to_owned())?;
    let OwnedR4g1Bundle::Legacy(bundle) = bundle else {
        return Err(
            "the installed schema-2 bundle is production-only; legacy research replay was refused"
                .to_owned(),
        );
    };
    let text =
        generate_legacy_r4g1_response(bundle, prompt, max_tokens, None).ok_or_else(|| {
            "the legacy lane-absent graph/tokenizer cannot replay this request".to_owned()
        })?;
    Ok(R4g1ResearchResponse {
        text,
        warning: LEGACY_R4G1_RESEARCH_WARNING,
    })
}

enum R4g1Generation {
    Supported {
        text: String,
        coverage: Option<&'static str>,
    },
    Abstained,
    HardIncompatibility(String),
}

/// #839 phase 1 (RF-30): the typed selective-prediction boundary response
/// (spec §5, WASM row) in legacy-coverage mode. Always a typed JSON value
/// with the canonical labels — never a trap, and never a bare `None` that
/// conflates an abstention with a failure:
///
/// - a production served answer → `status: "supported-answer"` with the D4
///   coverage reading;
/// - a D4 abstention → the canonical distributionally-novel abstention;
/// - an unusable surface (no installed bundle, runtime/tokenizer rejection,
///   empty generation) → `status: "hard-incompatibility"` with a reason —
///   the request cannot be validly served by this artifact/surface,
///   fail-closed.
pub fn typed_r4g1_response(prompt: &str, max_tokens: usize) -> String {
    match r4g1_generation(prompt, max_tokens, None) {
        R4g1Generation::Supported { text, coverage } => serde_json::json!({
            "status": crate::selective::STATUS_SUPPORTED_ANSWER,
            "coverage": coverage,
            "cause": serde_json::Value::Null,
            "confidence_permille": serde_json::Value::Null,
            "text": text,
        })
        .to_string(),
        R4g1Generation::Abstained => serde_json::json!({
            "status": crate::selective::STATUS_ABSTENTION,
            "coverage": crate::selective::COVERAGE_DISTRIBUTIONALLY_NOVEL,
            "cause": crate::selective::CAUSE_DISTRIBUTIONALLY_NOVEL,
            "confidence_permille": serde_json::Value::Null,
        })
        .to_string(),
        R4g1Generation::HardIncompatibility(reason) => serde_json::json!({
            "status": crate::selective::STATUS_HARD_INCOMPATIBILITY,
            "reason": reason,
        })
        .to_string(),
    }
}

/// Generate through the graph runtime with an optional server-side session
/// signature. The context signature remains the ROUT input; the session lane
/// is consumed by the existing emission-affinity bonus.
pub fn generate_r4g1_response_with_session_signature(
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> Option<String> {
    match r4g1_generation(prompt, max_tokens, session_signature) {
        R4g1Generation::Supported { text, .. } => Some(text),
        R4g1Generation::Abstained => {
            println!("[-] generate_r4g1_response: D4 abstained");
            None
        }
        R4g1Generation::HardIncompatibility(reason) => {
            println!("[-] generate_r4g1_response: {reason}");
            None
        }
    }
}

fn r4g1_generation(
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> R4g1Generation {
    let guard = match OWNED_R4G1.read() {
        Ok(g) => g,
        Err(_) => return R4g1Generation::HardIncompatibility("R4G1 bundle lock poisoned".into()),
    };
    let bundle = match guard.as_ref() {
        Some(bundle) => bundle,
        None => return R4g1Generation::HardIncompatibility("no R4G1 bundle is installed".into()),
    };

    match bundle {
        OwnedR4g1Bundle::Legacy(_) => R4g1Generation::HardIncompatibility(
            "the installed legacy lane-absent bundle is research-only; production serving requires a schema-2 envelope with replay-bound quality evidence"
                .into(),
        ),
        OwnedR4g1Bundle::Production(bundle) => {
            if session_signature.is_some() {
                return R4g1Generation::HardIncompatibility(
                    "the strict production facade does not accept an out-of-envelope session signature"
                        .into(),
                );
            }
            generate_production_r4g1_response(bundle, prompt, max_tokens)
        }
    }
}

fn generate_production_r4g1_response(
    bundle: &OwnedProductionR4g1Bundle,
    prompt: &str,
    max_tokens: usize,
) -> R4g1Generation {
    let mut seed = vec![0u32; prompt.len().saturating_add(2)];
    let Some(seed_count) = bundle.tokenizer.encode_into(prompt, &mut seed) else {
        return R4g1Generation::HardIncompatibility(
            "production tokenizer cannot encode the request".into(),
        );
    };
    seed.truncate(seed_count);

    let mut engine =
        match uor_r4_api::NormativeServingEngine::load(uor_r4_api::ProductionServingParts {
            engine: uor_r4_api::EngineParts {
                graph: &bundle.graph,
                signature_artifact: &bundle.signature_artifact,
                tokenizer: Some(&bundle.tokenizer_bytes),
                score_report: Some(&bundle.score_report),
            },
            deployed_quality_report: &bundle.deployed_quality_report,
            verified_envelope: &bundle.verified_envelope,
        }) {
            Ok(engine) => engine,
            Err(error) => {
                return R4g1Generation::HardIncompatibility(format!(
                    "installed production envelope no longer admits serving: {error}"
                ))
            }
        };

    let steps = max_tokens.min(128);
    let mut trajectory = ProductionTrajectory::from_seed(&seed);
    let mut generated = Vec::with_capacity(steps);
    let mut last_coverage = None;

    for _ in 0..steps {
        let session_signature = trajectory.session_signature();
        match engine.predict_with_session_signature(trajectory.window(), Some(&session_signature)) {
            Ok(uor_r4_api::NormativeServingDecision::Serve(served)) => {
                last_coverage = Some(match served.status {
                    uor_r4_api::ScoreStatus::Novel => {
                        crate::selective::COVERAGE_DISTRIBUTIONALLY_NOVEL
                    }
                    uor_r4_api::ScoreStatus::ExactContext | uor_r4_api::ScoreStatus::Graph => {
                        crate::selective::COVERAGE_COVERED
                    }
                });
                if served.token == 1 || served.token == 2 {
                    break;
                }
                generated.push(served.token);
                trajectory.commit(served.token);
            }
            Ok(uor_r4_api::NormativeServingDecision::Abstain(_)) => {
                return R4g1Generation::Abstained;
            }
            Ok(uor_r4_api::NormativeServingDecision::Decline(reason)) => {
                return R4g1Generation::HardIncompatibility(format!(
                    "normative production runtime declined: {reason:?}"
                ));
            }
            Err(error) => {
                return R4g1Generation::HardIncompatibility(format!(
                    "normative production prediction rejected the request: {error}"
                ));
            }
        }
    }

    let mut bytes = vec![0u8; 16 * 1024];
    match bundle.tokenizer.decode_into(&generated, &mut bytes) {
        Some(count) => R4g1Generation::Supported {
            text: String::from_utf8_lossy(&bytes[..count]).into_owned(),
            coverage: last_coverage,
        },
        None => R4g1Generation::HardIncompatibility(
            "production tokenizer cannot decode the runtime continuation".into(),
        ),
    }
}

/// Fixed-size production decode state. The scorer retains its existing
/// bounded window while the secondary signature lane carries the complete
/// admitted prompt plus committed generated prefix. EOS, abstention, and
/// decline never call [`Self::commit`], so speculative output cannot enter
/// trajectory memory.
struct ProductionTrajectory {
    window: [u32; WINDOW],
    window_len: usize,
    history_signature: TokenHistorySignature,
}

impl ProductionTrajectory {
    fn from_seed(seed: &[u32]) -> Self {
        let mut window = [0u32; WINDOW];
        let tail = &seed[seed.len().saturating_sub(WINDOW)..];
        window[..tail.len()].copy_from_slice(tail);
        Self {
            window,
            window_len: tail.len(),
            history_signature: TokenHistorySignature::from_tokens(seed),
        }
    }

    fn window(&self) -> &[u32] {
        &self.window[..self.window_len]
    }

    fn session_signature(&self) -> [u8; uor_r4_router::session_signature::SESSION_SIGNATURE_BYTES] {
        self.history_signature.signature()
    }

    fn commit(&mut self, token: u32) {
        self.history_signature.push(token);
        if self.window_len < WINDOW {
            self.window[self.window_len] = token;
            self.window_len += 1;
        } else {
            self.window.copy_within(1.., 0);
            self.window[WINDOW - 1] = token;
        }
    }
}

fn generate_legacy_r4g1_response(
    bundle: &OwnedLegacyR4g1Bundle,
    prompt: &str,
    max_tokens: usize,
    session_signature: Option<&[u8]>,
) -> Option<String> {
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

            let candidates = runtime.predict_served_candidates_with_signature_lanes(
                &beam_tokens,
                sig.as_ref().map(|s| &s[..]),
                session_signature,
                &mut node_scores,
            );

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

    #[test]
    fn production_trajectory_keeps_prefix_memory_beyond_runtime_window() {
        let shared_window: Vec<u32> = (100..100 + WINDOW as u32).collect();
        let mut first_seed = vec![1, 2, 3, 4];
        first_seed.extend_from_slice(&shared_window);
        let mut second_seed = vec![91, 92, 93, 94];
        second_seed.extend_from_slice(&shared_window);

        let mut first = ProductionTrajectory::from_seed(&first_seed);
        let mut second = ProductionTrajectory::from_seed(&second_seed);
        assert_eq!(first.window(), second.window());
        assert_ne!(first.session_signature(), second.session_signature());

        first.commit(777);
        second.commit(777);
        assert_eq!(first.window(), second.window());
        assert_ne!(first.session_signature(), second.session_signature());
    }

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

    fn minimal_graph_with_lanes(tokenizer_cid: [u8; 32]) -> Vec<u8> {
        use uor_r4_graph_format::{
            build_psi_bag_table, build_skipmix_table, ArtifactBuilder, GraphView, SectionId,
            SkipmixRowInput,
        };

        let base = minimal_graph(tokenizer_cid);
        let view = GraphView::parse(&base).expect("minimal graph parses");
        let skip_rows: Vec<SkipmixRowInput> = Vec::new();
        let psi_rows: Vec<(u32, Vec<(u32, i32)>)> = Vec::new();
        let skmx = build_skipmix_table(&skip_rows).expect("empty SKMX is valid");
        let psib = build_psi_bag_table(&psi_rows).expect("empty PSIB is valid");
        let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
        for section in view.sections() {
            builder.add_section(section.id, section.flags, section.payload);
        }
        builder.add_section(SectionId::SKMX, 0, &skmx);
        builder.add_section(SectionId::PSIB, 0, &psib);
        builder.build().expect("lane-bearing graph")
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
            match guard.as_ref().expect("installed bundle") {
                OwnedR4g1Bundle::Legacy(bundle) => blake3::hash(&bundle.graph),
                OwnedR4g1Bundle::Production(_) => panic!("legacy installer changed bundle kind"),
            }
        };
        let error = set_r4g1_bundle(graph, tokenizer_b)
            .expect_err("swapped tokenizer must not replace the active bundle");
        assert!(error.contains("tokenizer CID mismatch"), "{error}");
        let retained_hash = {
            let guard = OWNED_R4G1.read().expect("bundle lock");
            match guard.as_ref().expect("retained bundle") {
                OwnedR4g1Bundle::Legacy(bundle) => blake3::hash(&bundle.graph),
                OwnedR4g1Bundle::Production(_) => panic!("failed install changed bundle kind"),
            }
        };
        assert_eq!(retained_hash, installed_hash);

        try_set_r4g1_bytes(minimal_graph([0; 32]))
            .expect("legacy zero-CID graph-only install remains supported");
        let guard = OWNED_R4G1.read().expect("bundle lock");
        assert!(matches!(
            guard.as_ref(),
            Some(OwnedR4g1Bundle::Legacy(OwnedLegacyR4g1Bundle {
                tokenizer: OwnedR4g1Tokenizer::LegacyGlobal,
                ..
            }))
        ));
    }

    #[test]
    fn absent_lane_legacy_generation_is_explicitly_research_only() {
        let _test_guard = R4G1_TEST_LOCK.lock().expect("R4G1 test lock");
        *OWNED_R4G1.write().expect("bundle lock") = None;
        let tokenizer =
            uor_r4_core::transformerless::scenarios::Tokenizer::from_bytes(&tokenizer_bytes(&[
                b" ", b"a",
            ]))
            .expect("legacy tokenizer parses");
        set_tless_tokenizer(tokenizer);
        try_set_r4g1_bytes(minimal_graph([0; 32])).expect("absent-lane graph installs");
        assert_eq!(generate_r4g1_response("a", 1), None);
        let typed: serde_json::Value = serde_json::from_str(&typed_r4g1_response("a", 1))
            .expect("typed production refusal is JSON");
        assert_eq!(
            typed["status"],
            crate::selective::STATUS_HARD_INCOMPATIBILITY
        );
        assert!(typed["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("research-only")));
        assert_eq!(
            crate::generate_r4g1_response("a", 1),
            None,
            "the exact public/WASM facade body is production-only"
        );
        let root_typed: serde_json::Value =
            serde_json::from_str(&crate::typed_r4g1_response("a", 1))
                .expect("root public/WASM facade returns typed JSON");
        assert_eq!(root_typed, typed);

        let replay = generate_legacy_r4g1_research_response("a", 1)
            .expect("explicit research replay retains legacy compatibility");
        assert_eq!(
            replay.text, "R4G1 zero-multiply prediction complete.",
            "absent-section legacy output bytes remain a research compatibility contract"
        );
        assert_eq!(replay.warning, LEGACY_R4G1_RESEARCH_WARNING);
    }

    #[test]
    fn lane_bearing_graphs_require_a_complete_production_envelope() {
        let _test_guard = R4G1_TEST_LOCK.lock().expect("R4G1 test lock");
        *OWNED_R4G1.write().expect("bundle lock") = None;
        let legacy_graph = minimal_graph([0; 32]);
        try_set_r4g1_bytes(legacy_graph.clone()).expect("legacy baseline installs");
        let baseline_hash = blake3::hash(&legacy_graph);

        let tokenizer = tokenizer_bytes(&[b" ", b"a"]);
        let lane_graph = minimal_graph_with_lanes(*blake3::hash(&tokenizer).as_bytes());
        let graph_only_error = try_set_r4g1_bytes(lane_graph.clone())
            .expect_err("lane-bearing graph-only install must fail closed");
        assert!(graph_only_error.contains("production-envelope"));
        let legacy_bundle_error = set_r4g1_bundle(lane_graph.clone(), tokenizer.clone())
            .expect_err("lane-bearing graph/tokenizer install must fail closed");
        assert!(legacy_bundle_error.contains("production-envelope"));

        let strict_error = set_r4g1_production_bundle(
            lane_graph,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            tokenizer,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect_err("incomplete production envelope must fail closed");
        assert!(
            strict_error.contains("release-bundle.json"),
            "{strict_error}"
        );

        let guard = OWNED_R4G1.read().expect("bundle lock");
        let retained_hash = match guard.as_ref().expect("legacy bundle retained") {
            OwnedR4g1Bundle::Legacy(bundle) => blake3::hash(&bundle.graph),
            OwnedR4g1Bundle::Production(_) => panic!("invalid envelope became active"),
        };
        assert_eq!(retained_hash, baseline_hash);
    }

    #[test]
    fn complete_production_envelope_installs_and_corpus_tamper_is_atomic() {
        use crate::release_bundle_packager::{
            package_release_bundle, tests::write_production_bundle, PackageInputs,
            UOR_MATMUL_REVISION,
        };
        use uor_r4_api::{BundleCapability, UorMatmulProvenance};

        let _test_guard = R4G1_TEST_LOCK.lock().expect("R4G1 test lock");
        *OWNED_R4G1.write().expect("bundle lock") = None;
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-wasm-production-envelope-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("production fixture directory");
        let admission = write_production_bundle(&dir);
        let manifest = package_release_bundle(
            &dir,
            PackageInputs {
                model_id: "r4".to_owned(),
                capability: BundleCapability::InstructionChat,
                uor_matmul: UorMatmulProvenance {
                    rev: UOR_MATMUL_REVISION.to_owned(),
                    operation_profile: "exact-gemm-float".to_owned(),
                    license: "MIT".to_owned(),
                    source_digest: None,
                },
                tokenizer_adapter: admission.tokenizer_adapter,
                selector: admission.bindings.selector.clone(),
                compiler: admission.bindings.compiler.clone(),
                provenance_note: Some("strict wasm installer fixture".to_owned()),
            },
        )
        .expect("fixture manifest");

        let read = |path: &str| std::fs::read(dir.join(path)).expect("fixture component");
        let graph = read("graph/score.r4g1");
        let sections_absent_graph = read("graph/score_sections_absent.r4g1");
        let label_shuffled_graph = read("graph/score_label_shuffled.r4g1");
        let teacher = read("tless_artifacts.bin");
        let tla_comparator_store = read("tless_store.bin");
        let tokenizer = read("tokenizer.bin");
        let score_report = read("graph/score_report.json");
        let compile_report = read("graph-cover/cover_report.json");
        let quality_report = read("graph/deployed_quality_report.json");
        let cross_surface_parity = read("graph/cross_surface_parity.json");
        let witness_replay = read("graph/witness_replay.json");
        let corpus_meta = read("corpus.meta");
        let corpus_records = read("corpus.records");
        let tokenizer_adapter = read("tokenizer_adapter.json");
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("manifest bytes");

        set_r4g1_production_bundle(
            graph.clone(),
            sections_absent_graph.clone(),
            label_shuffled_graph.clone(),
            teacher.clone(),
            tla_comparator_store.clone(),
            tokenizer.clone(),
            score_report.clone(),
            compile_report.clone(),
            quality_report.clone(),
            cross_surface_parity.clone(),
            witness_replay.clone(),
            corpus_meta.clone(),
            corpus_records.clone(),
            tokenizer_adapter.clone(),
            manifest_bytes.clone(),
        )
        .expect("complete production envelope installs");
        let installed_graph_hash = blake3::hash(&graph);
        assert!(matches!(
            OWNED_R4G1.read().expect("bundle lock").as_ref(),
            Some(OwnedR4g1Bundle::Production(_))
        ));
        let served: serde_json::Value = serde_json::from_str(&crate::typed_r4g1_response("a", 1))
            .expect("native-testable public/WASM production facade returns JSON");
        assert_eq!(
            served["status"],
            crate::selective::STATUS_SUPPORTED_ANSWER,
            "a complete schema-2 envelope must execute the normative production selector"
        );

        let mut tampered_records = corpus_records;
        tampered_records[4] ^= 1;
        let error = set_r4g1_production_bundle(
            graph,
            sections_absent_graph,
            label_shuffled_graph,
            teacher,
            tla_comparator_store,
            tokenizer,
            score_report,
            compile_report,
            quality_report,
            cross_surface_parity,
            witness_replay,
            corpus_meta,
            tampered_records,
            tokenizer_adapter,
            manifest_bytes,
        )
        .expect_err("tampered corpus cannot replace active production generation");
        assert!(
            error.contains(
                "graph.HEAD.corpus_construction_cid: does not bind the exact corpus construction positions"
            ),
            "{error}"
        );
        let guard = OWNED_R4G1.read().expect("bundle lock");
        let retained_hash = match guard.as_ref().expect("production bundle retained") {
            OwnedR4g1Bundle::Production(bundle) => blake3::hash(&bundle.graph),
            OwnedR4g1Bundle::Legacy(_) => panic!("production bundle was downgraded"),
        };
        assert_eq!(retained_hash, installed_graph_hash);
        let _ = std::fs::remove_dir_all(dir);
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
