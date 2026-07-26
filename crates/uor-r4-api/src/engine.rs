//! Typed R4G1 engine: the deployed graph-runtime adapter as library code.
//!
//! Moved from the root package's `src/r4g1.rs` (same repository) so
//! library consumers load the engine from byte slices instead of
//! filesystem paths; the root package keeps a thin path-based wrapper
//! over this module with unchanged behavior.
//!
//! The graph scorer is intentionally kept separate from the exploratory
//! f64 router. It derives the packed input signature with the
//! transformerless artifact, then selects a token from the validated
//! R4G1 graph.
//!
//! # ResolutionStatus-driven behavior (issue #78, decision D4)
//!
//! Every prediction carries a resolution status and the deployed behavior
//! is declared as data in [`StatusPolicy`] (the D4 manifest policy):
//!
//! | status | default action |
//! |---|---|
//! | `exact_context` | [`StatusAction::Serve`] |
//! | `graph` | [`StatusAction::Serve`] |
//! | `novel` | [`StatusAction::WidenOnce`] (then abstain) |
//! | `contradictory` | [`StatusAction::Abstain`] (reserved; the scorer does not produce it yet) |
//!
//! `WidenOnce` retries the prediction with the per-depth membership set
//! widened to [`WIDENED_TOP_M`] exactly once; a signature still Novel
//! after widening is remembered in a bounded FIFO so identical probes
//! abstain without widening again (threat model: fallback
//! denial-of-service). Abstention is a typed outcome — no token is
//! emitted, none is guessed, and the caller surfaces the status. An
//! optional override is read from the graph's `score_report.json`
//! (`config.status_policy`, e.g. `{"novel": "abstain"}` with values
//! `serve` / `widen_once` / `abstain`); absent or invalid rows keep the
//! defaults.

use std::fmt;
use std::io;

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::compiler::{self, Compiled, SIG_BYTES, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime;

/// Unified inference request payload across HTTP REST, WebSocket, and WASM interfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceRequest {
    /// Target prompt or query string.
    pub text: String,
    /// Tenant or session identity tag.
    pub identity: Option<String>,
    /// Requested synthesis engine ("r4g1", "transformerless", "geometric", "attention", "r4-attention").
    pub engine: Option<String>,
    /// Maximum continuation tokens to generate.
    pub max_tokens: Option<usize>,
    /// Temperature for geometric sampling.
    pub temperature: Option<f64>,
}

/// Unified inference response payload across HTTP REST, WebSocket, and WASM interfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceResponse {
    /// Generated continuation or response text.
    pub text: String,
    /// Engine that served or processed the request.
    pub engine: String,
    /// Whether language generation was served.
    pub llm_connected: bool,
    /// Number of tokens generated in continuation.
    pub tokens_generated: usize,
    /// Whether D4 policy abstained.
    pub abstained: bool,
    /// Optional status label if abstained or policy served.
    pub status: Option<String>,
    /// Whether a widened search occurred.
    pub widened: bool,
    /// Detailed generation mode (e.g. "r4g1", "r4g1-abstained", "r4g1-fallback-transformerless").
    pub generation_mode: String,
    /// Optional error details if unfulfilled.
    pub error: Option<String>,
}
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_graph_certify::{
    GraphScorer, ScoreStatus, StepState, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B, TOP_M,
    WIDENED_TOP_M,
};
use uor_r4_graph_format::{
    ContractVersion, FormatError, GraphView, FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR,
    INFERENCE_OPERATION_CONTRACT_VERSION,
};

/// The resolution status of a scored prediction. Alias of the production
/// scorer's [`ScoreStatus`] — no second definition of the status space.
pub type ResolutionStatus = ScoreStatus;

/// Owned inputs for [`R4Engine::load`]: every component as bytes, exactly
/// as the compile pipeline emitted it. `signature_artifact` is the
/// teacher artifact the packed input signatures derive from;
/// `score_report` is the graph's JSON quality/config report (optional —
/// absent means D4 policy defaults and scorer defaults).
#[derive(Debug, Clone, Copy)]
pub struct EngineParts<'a> {
    /// Scored deployable R4G1 graph bytes (`score.r4g1`).
    pub graph: &'a [u8],
    /// Teacher artifact bytes the input signatures derive from.
    pub signature_artifact: &'a [u8],
    /// Optional bundle tokenizer bytes (binary tokenizer.bin format).
    pub tokenizer: Option<&'a [u8]>,
    /// Optional `score_report.json` bytes.
    pub score_report: Option<&'a [u8]>,
}

/// The version surface the loaded engine runs against: the R4G1 format
/// version, the normative inference operation contract version, and this
/// crate's own version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiVersion {
    pub format_major: u8,
    pub format_minor: u8,
    pub contract: ContractVersion,
    pub api_crate_version: &'static str,
}

impl AbiVersion {
    /// The ABI surface of this build.
    pub fn current() -> Self {
        Self {
            format_major: FORMAT_VERSION_MAJOR,
            format_minor: FORMAT_VERSION_MINOR,
            contract: INFERENCE_OPERATION_CONTRACT_VERSION,
            api_crate_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

/// Focused load-time failures of [`R4Engine::load`].
#[derive(Debug)]
pub enum LoadError {
    /// The graph bytes failed the format crate's two-stage structural
    /// validation or CID verification.
    InvalidGraph(FormatError),
    /// The signature artifact is not a TLA3/TLA4/TLA5 teacher container.
    InvalidSignatureArtifact,
    /// The score report bytes are not well-formed JSON.
    InvalidScoreReport(String),
    /// The tokenizer bytes are not a well-formed binary tokenizer.
    InvalidTokenizer(io::Error),
    /// The graph's Rule 1+2 quality digresses from the declared quality
    /// basis (see [`validate_quality_report`]).
    QualityGate(String),
    /// The production scorer rejected the validated parts.
    Scorer(String),
    /// The teacher token table exceeds the u32 token-id space.
    TeacherTooLarge,
    /// The loaded tokenizer CID does not match the R4G1 header's tokenizer_cid.
    TokenizerCidMismatch { expected: String, actual: String },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::InvalidGraph(error) => write!(f, "invalid R4G1 graph: {error}"),
            LoadError::InvalidSignatureArtifact => {
                write!(f, "not a TLA3/TLA4/TLA5 teacher artifact")
            }
            LoadError::InvalidScoreReport(message) => {
                write!(f, "invalid score report: {message}")
            }
            LoadError::InvalidTokenizer(error) => write!(f, "invalid tokenizer bytes: {error}"),
            LoadError::QualityGate(message) => write!(f, "{message}"),
            LoadError::Scorer(message) => write!(f, "{message}"),
            LoadError::TeacherTooLarge => write!(f, "teacher token table too large"),
            LoadError::TokenizerCidMismatch { expected, actual } => {
                write!(
                    f,
                    "tokenizer_cid mismatch: header expected {expected}, loaded {actual}"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::InvalidGraph(error) => Some(error),
            LoadError::InvalidTokenizer(error) => Some(error),
            _ => None,
        }
    }
}

/// Focused predict/generate-time failures of [`R4Engine`].
#[derive(Debug)]
pub enum InferenceError {
    /// A window carried a token id the teacher artifact cannot decode
    /// (boundary check: signature derivation indexes by token id).
    TokenOutOfVocabulary {
        /// Exclusive upper bound of decodable token ids.
        token_rows: u32,
    },
    /// The production scorer failed on a validated input.
    Scorer(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::TokenOutOfVocabulary { token_rows } => {
                write!(f, "token id outside the teacher vocabulary ({token_rows})")
            }
            InferenceError::Scorer(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for InferenceError {}

// BEGIN DEPLOYED STATUS POLICY (INTEGER-ONLY) -------------------------
// The D4 manifest policy and the status-aware prediction path below are
// part of the deployed integer contract: no float, no multiply/divide/
// modulo in value arithmetic, and no per-prediction allocation in
// steady state (one-time buffers are built in `load`). The
// status-policy test suite machine-checks this delimited block by
// source scan (the P-4 pattern) and censuses the prediction calls.

/// The adapter-level status space of the D4 manifest policy: the
/// scorer's three [`ScoreStatus`] outcomes plus the reserved
/// `Contradictory` (glossary "Resolution status"). The scorer does not
/// produce `Contradictory` yet; the policy arm is declared and enforced
/// now so the deployed behavior is total when it lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyStatus {
    /// Exact-context evidence resolved (Rule 2).
    ExactContext,
    /// Graph residuals resolved (Rule 1 with a covered chain).
    Graph,
    /// No calibrated region covers the input.
    Novel,
    /// Active regions materially disagree (reserved).
    Contradictory,
}

impl From<ScoreStatus> for PolicyStatus {
    fn from(status: ScoreStatus) -> Self {
        match status {
            ScoreStatus::ExactContext => PolicyStatus::ExactContext,
            ScoreStatus::Graph => PolicyStatus::Graph,
            ScoreStatus::Novel => PolicyStatus::Novel,
        }
    }
}

impl PolicyStatus {
    /// The wire label used in server JSON responses.
    pub fn label(self) -> &'static str {
        match self {
            PolicyStatus::ExactContext => "exact_context",
            PolicyStatus::Graph => "graph",
            PolicyStatus::Novel => "novel",
            PolicyStatus::Contradictory => "contradictory",
        }
    }
}

/// One row of the manifest policy: what the deployed adapter does with
/// a prediction of this status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusAction {
    /// Emit the selected token.
    Serve,
    /// Retry once with the membership set widened to [`WIDENED_TOP_M`];
    /// if the retry does not resolve to a served status, abstain. A
    /// signature confirmed Novel after widening is remembered (bounded)
    /// so identical probes abstain without widening again.
    WidenOnce,
    /// Emit no token; the caller surfaces the status.
    Abstain,
}

/// The D4 manifest policy as data — one action per status. The defaults
/// implement the plan's D4 recommendation (exact-residual evidence,
/// then abstain): ExactContext to Serve, Graph to Serve, Novel to
/// WidenOnce (then Abstain), Contradictory to Abstain.
///
/// Optional override: the `status_policy` key inside `config` of the
/// graph's `score_report.json`, for example
/// `{"exact_context": "serve", "graph": "serve", "novel": "widen_once",
/// "contradictory": "abstain"}`. Missing keys and unknown values keep
/// the default for that row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusPolicy {
    pub exact_context: StatusAction,
    pub graph: StatusAction,
    pub novel: StatusAction,
    pub contradictory: StatusAction,
}

impl Default for StatusPolicy {
    fn default() -> Self {
        Self {
            exact_context: StatusAction::Serve,
            graph: StatusAction::Serve,
            novel: StatusAction::WidenOnce,
            contradictory: StatusAction::Abstain,
        }
    }
}

impl StatusPolicy {
    /// The action declared for `status`.
    pub fn action(&self, status: PolicyStatus) -> StatusAction {
        match status {
            PolicyStatus::ExactContext => self.exact_context,
            PolicyStatus::Graph => self.graph,
            PolicyStatus::Novel => self.novel,
            PolicyStatus::Contradictory => self.contradictory,
        }
    }

    /// Read the optional `status_policy` override from the score
    /// report's `config` section; absent or invalid rows fall back to
    /// the D4 defaults.
    pub fn from_report(report: Option<&serde_json::Value>) -> Self {
        let defaults = Self::default();
        let overrides = report
            .and_then(|r| r.get("config"))
            .and_then(|c| c.get("status_policy"));
        let parse = |key: &str, default: StatusAction| {
            overrides
                .and_then(|o| o.get(key))
                .and_then(serde_json::Value::as_str)
                .and_then(parse_action)
                .unwrap_or(default)
        };
        Self {
            exact_context: parse("exact_context", defaults.exact_context),
            graph: parse("graph", defaults.graph),
            novel: parse("novel", defaults.novel),
            contradictory: parse("contradictory", defaults.contradictory),
        }
    }
}

/// Parse one override value (`serve`, `widen_once`, `abstain`).
fn parse_action(value: &str) -> Option<StatusAction> {
    match value {
        "serve" => Some(StatusAction::Serve),
        "widen_once" => Some(StatusAction::WidenOnce),
        "abstain" => Some(StatusAction::Abstain),
        _ => None,
    }
}

/// Observable counters of the status-aware path: the widen-once bound
/// and the abstain/serve rates, asserted by the probe suite and
/// reportable by the server.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PolicyCounters {
    /// Status-aware predictions run.
    pub predicts: u64,
    /// Predictions served a token.
    pub serves: u64,
    /// Predictions abstained.
    pub abstains: u64,
    /// Widened re-probes run (at most one per distinct Novel
    /// signature; the bounded memory answers the rest).
    pub widen_attempts: u64,
    /// Novel signatures answered from the widen-once memory instead of
    /// re-widening.
    pub widen_skipped_seen: u64,
}

/// A served prediction: the selected token, the status that resolved,
/// and whether a widened re-probe ran for this prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictOutcome {
    pub token: u32,
    pub status: ScoreStatus,
    pub widened: bool,
}

/// A typed abstention: no token was emitted and none is guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbstainOutcome {
    pub status: ScoreStatus,
    pub widened: bool,
}

/// The status-aware prediction result of the deployed adapter: either
/// the policy serves a token or it abstains with the status recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictDecision {
    Serve(PredictOutcome),
    Abstain(AbstainOutcome),
}

/// The result of a status-aware generation run: tokens written, the
/// final step's status, whether any step widened, and whether the run
/// stopped on an abstention (no guessed token was emitted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateStatus {
    pub count: usize,
    pub status: Option<ScoreStatus>,
    pub widened: bool,
    pub abstained: bool,
}

/// Caller-owned prediction result slot for [`R4Engine::predict_next_into`],
/// mirroring [`PredictDecision`] semantics in flat form: when `abstained`
/// is set, no token was emitted and none is guessed (`token` is written
/// as 0 and carries no meaning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PredictOutput {
    pub token: u32,
    pub status: Option<ResolutionStatus>,
    pub widened: bool,
    pub abstained: bool,
}

/// Bound of the widen-once memory: confirmed-Novel signatures whose
/// widening would deterministically re-resolve Novel (scoring is a pure
/// function of the loaded artifact, so the memory is sound for the
/// lifetime of this state; a recompile replaces the whole state).
const NOVEL_SEEN_CAPACITY: usize = 1024;

/// Fixed-capacity FIFO of confirmed-Novel signatures (threat model:
/// repeated adversarial out-of-distribution probes abstain after one
/// membership scan instead of forcing constant widening).
struct NovelSeen {
    sigs: Vec<[u8; SIG_BYTES]>,
    next: usize,
}

impl NovelSeen {
    fn new(capacity: usize) -> Self {
        Self {
            sigs: Vec::with_capacity(capacity),
            next: 0,
        }
    }
    fn contains(&self, sig: &[u8; SIG_BYTES]) -> bool {
        self.sigs.iter().any(|entry| entry == sig)
    }
    fn insert(&mut self, sig: &[u8; SIG_BYTES]) {
        if self.sigs.capacity() == 0 {
            return;
        }
        if self.sigs.len() < self.sigs.capacity() {
            self.sigs.push(*sig);
        } else {
            self.sigs[self.next] = *sig;
        }
        self.next += 1;
        if self.next == self.sigs.capacity() {
            self.next = 0;
        }
    }
}

/// One scored probe: the selection and its status at the given
/// membership width.
struct ScoredProbe {
    token: u32,
    status: ScoreStatus,
}

/// A loaded, CID-verified scored graph and the teacher artifact needed to
/// derive input signatures from token ids.
pub struct R4Engine {
    artifacts: Compiled,
    scorer: GraphScorer,
    rotations: [usize; WINDOW + 1],
    tokenizer: Option<Tokenizer>,
    /// Number of token rows in the teacher artifact: the exclusive upper
    /// bound of decodable token ids (checked at the prediction boundary
    /// so an out-of-vocabulary window is a typed error, not a panic).
    token_rows: u32,
    /// The D4 manifest policy in force (defaults or report override).
    policy: StatusPolicy,
    /// Fixed-capacity scoring scratch, allocated once in `load`.
    step: StepState,
    /// False only for legacy TLS1 exact-context artifacts, which stay on
    /// the reference scorer (no widening there).
    step_supported: bool,
    counters: PolicyCounters,
    novel_seen: NovelSeen,
}

impl R4Engine {
    /// The manifest policy in force (D4 defaults or the score-report
    /// override).
    pub fn policy(&self) -> StatusPolicy {
        self.policy
    }

    /// A snapshot of the status-path counters.
    pub fn policy_counters(&self) -> PolicyCounters {
        self.counters
    }

    /// Derive the packed input signature of a token window.
    fn derive_sig(&self, window: &[u32]) -> [u8; SIG_BYTES] {
        let bundle = runtime::bundle_window_plain(&self.artifacts, &self.rotations, window);
        runtime::sig_plain(&self.artifacts, &bundle)
    }

    /// Reject a window carrying a token id the teacher artifact cannot
    /// decode. Also enforces the `WINDOW = 8` Dyadic-Recency boundary, emitting
    /// a `tracing::warn!` log if `window.len() > 8` before sliding window truncation.
    fn check_window(&self, window: &[u32]) -> Result<(), InferenceError> {
        if window.len() > WINDOW {
            tracing::warn!(
                target: "uor_r4_core::runtime",
                window_size = WINDOW,
                input_size = window.len(),
                "Input context exceeds 8-token window; truncating to 8 most recent tokens"
            );
        }
        if window.iter().any(|&t| t >= self.token_rows) {
            return Err(InferenceError::TokenOutOfVocabulary {
                token_rows: self.token_rows,
            });
        }
        Ok(())
    }

    /// Score one signature at the given membership width. Artifacts
    /// with legacy TLS1 exact-context evidence stay on the reference
    /// scorer (the deployed step requires residualized RX1 evidence);
    /// that path ignores the width — widening is unavailable there.
    fn score_sig(
        &mut self,
        sig: &[u8; SIG_BYTES],
        top_m: usize,
        recent_tokens: &[u32],
    ) -> Result<ScoredProbe, InferenceError> {
        if self.step_supported {
            let outcome = self
                .scorer
                .score_step_with_recent(sig, top_m, &mut self.step, recent_tokens)
                .map_err(InferenceError::Scorer)?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.status,
            })
        } else {
            let outcome = self
                .scorer
                .score_candidates(sig, recent_tokens)
                .map_err(InferenceError::Scorer)?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.witness.status,
            })
        }
    }

    /// The D4 policy decision for one input signature: score at the
    /// manifest membership width, then Serve / WidenOnce / Abstain per
    /// the declared policy. WidenOnce re-probes once at
    /// [`WIDENED_TOP_M`]; a signature still Novel after widening is
    /// remembered so identical probes abstain without widening again.
    pub fn predict_signature_status(
        &mut self,
        sig: &[u8; SIG_BYTES],
    ) -> Result<PredictDecision, InferenceError> {
        self.predict_signature_status_with_recent(sig, &[])
    }

    fn predict_signature_status_with_recent(
        &mut self,
        sig: &[u8; SIG_BYTES],
        recent_tokens: &[u32],
    ) -> Result<PredictDecision, InferenceError> {
        self.counters.predicts += 1;
        let first = self.score_sig(sig, TOP_M, recent_tokens)?;
        match self.policy.action(first.status.into()) {
            StatusAction::Serve => {
                self.counters.serves += 1;
                Ok(PredictDecision::Serve(PredictOutcome {
                    token: first.token,
                    status: first.status,
                    widened: false,
                }))
            }
            StatusAction::Abstain => {
                self.counters.abstains += 1;
                Ok(PredictDecision::Abstain(AbstainOutcome {
                    status: first.status,
                    widened: false,
                }))
            }
            StatusAction::WidenOnce => {
                if !self.step_supported {
                    // No widening on the legacy reference path: abstain
                    // directly (documented degrade for TLS1 artifacts).
                    self.counters.abstains += 1;
                    return Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                    }));
                }
                if self.novel_seen.contains(sig) {
                    self.counters.widen_skipped_seen += 1;
                    self.counters.abstains += 1;
                    return Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                    }));
                }
                self.counters.widen_attempts += 1;
                let second = self.score_sig(sig, WIDENED_TOP_M, recent_tokens)?;
                if second.status == ScoreStatus::Novel {
                    self.novel_seen.insert(sig);
                }
                if self.policy.action(second.status.into()) == StatusAction::Serve {
                    self.counters.serves += 1;
                    Ok(PredictDecision::Serve(PredictOutcome {
                        token: second.token,
                        status: second.status,
                        widened: true,
                    }))
                } else {
                    self.counters.abstains += 1;
                    Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: second.status,
                        widened: true,
                    }))
                }
            }
        }
    }

    /// The status-aware decision for one token window: score the packed
    /// signature through the D4 policy. Abstention is a typed outcome —
    /// no guessed token is emitted.
    pub fn predict_decision(&mut self, window: &[u32]) -> Result<PredictDecision, InferenceError> {
        self.check_window(window)?;
        self.predict_signature_status_with_recent(&self.derive_sig(window), window)
    }

    /// Score one token window into a caller-owned output slot. Mirrors
    /// [`PredictDecision`] semantics: on a policy abstention
    /// `out.abstained` is set and `out.token` carries no meaning.
    pub fn predict_next_into(
        &mut self,
        window: &[u32],
        out: &mut PredictOutput,
    ) -> Result<(), InferenceError> {
        match self.predict_decision(window)? {
            PredictDecision::Serve(outcome) => {
                out.token = outcome.token;
                out.status = Some(outcome.status);
                out.widened = outcome.widened;
                out.abstained = false;
            }
            PredictDecision::Abstain(outcome) => {
                out.token = 0;
                out.status = Some(outcome.status);
                out.widened = outcome.widened;
                out.abstained = true;
            }
        }
        Ok(())
    }

    /// Generate a greedy continuation with per-step policy decisions:
    /// stops at the first abstention (returning the count so far and
    /// the abstaining status) and never emits a guessed token.
    pub fn generate_into(
        &mut self,
        seed: &[u32],
        out: &mut [u32],
    ) -> Result<GenerateStatus, InferenceError> {
        self.check_window(seed)?;
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        let mut last_status = None;
        let mut widened = false;
        for (generated, token) in out.iter_mut().enumerate() {
            match self.predict_decision(&window[..window_len])? {
                PredictDecision::Serve(outcome) => {
                    let next = outcome.token;
                    last_status = Some(outcome.status);
                    widened = widened || outcome.widened;
                    *token = next;
                    if next == 1 || next == 2 {
                        return Ok(GenerateStatus {
                            count: generated,
                            status: last_status,
                            widened,
                            abstained: false,
                        });
                    }
                    if window_len < WINDOW {
                        window[window_len] = next;
                        window_len += 1;
                    } else {
                        window.copy_within(1.., 0);
                        window[WINDOW - 1] = next;
                    }
                }
                PredictDecision::Abstain(outcome) => {
                    return Ok(GenerateStatus {
                        count: generated,
                        status: Some(outcome.status),
                        widened: widened || outcome.widened,
                        abstained: true,
                    });
                }
            }
        }
        Ok(GenerateStatus {
            count: out.len(),
            status: last_status,
            widened,
            abstained: false,
        })
    }
}

// END DEPLOYED STATUS POLICY (INTEGER-ONLY) ---------------------------

impl R4Engine {
    /// Load and validate an engine from byte-slice parts. The graph is
    /// structurally validated (format crate two-stage parse) and its CIDs
    /// verified before any scorer state is built. The teacher artifact
    /// supplies the compressed token rows used to derive input
    /// signatures. EXCT is not enabled because its reference
    /// implementation performs probe-time floating-point quantization.
    pub fn load(parts: EngineParts<'_>) -> Result<Self, LoadError> {
        // Fail fast with a typed format error before any scorer state is
        // built (the scorer re-validates internally; this surfaces the
        // format crate's focused error at the library boundary).
        let view = GraphView::parse(parts.graph).map_err(LoadError::InvalidGraph)?;
        view.verify_cids().map_err(LoadError::InvalidGraph)?;
        if let Some(tokenizer_bytes) = parts.tokenizer {
            let expected = view
                .head()
                .ok_or(FormatError::MissingHead)
                .map_err(LoadError::InvalidGraph)?
                .tokenizer_cid();
            let actual = blake3::hash(tokenizer_bytes);
            if expected.0 != [0u8; 32] && expected.0 != *actual.as_bytes() {
                return Err(LoadError::TokenizerCidMismatch {
                    expected: format!("blake3:{}", blake3::Hash::from(expected.0).to_hex()),
                    actual: format!("blake3:{actual}"),
                });
            }
        }
        let artifacts = compiler::parse_artifacts(parts.signature_artifact)
            .ok_or(LoadError::InvalidSignatureArtifact)?;
        let score_report = parts
            .score_report
            .map(|bytes| {
                serde_json::from_slice::<serde_json::Value>(bytes)
                    .map_err(|error| LoadError::InvalidScoreReport(error.to_string()))
            })
            .transpose()?;
        let root_top_b = score_report
            .as_ref()
            .and_then(|report| report.pointer("/config/root_top_b"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_ROOT_TOP_B);
        let exct_top_x = score_report
            .as_ref()
            .and_then(|report| report.pointer("/config/exct_top_x"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_EXCT_TOP_X);
        // The compiled RX1 EXCT table contains integer residuals. Supplying
        // the teacher artifact here is only for integer class-code lookup;
        // no probe-time log quantization occurs in the deployed path.
        let scorer = GraphScorer::from_artifact(
            parts.graph,
            Some(parts.signature_artifact),
            root_top_b,
            exct_top_x,
        )
        .map_err(LoadError::Scorer)?;
        if let Some(report) = score_report.as_ref() {
            validate_quality_report(report).map_err(LoadError::QualityGate)?;
        }
        let tokenizer = parts
            .tokenizer
            .map(|bytes| Tokenizer::from_bytes(bytes).map_err(LoadError::InvalidTokenizer))
            .transpose()?;

        let policy = StatusPolicy::from_report(score_report.as_ref());
        let step_supported = !scorer.has_legacy_exct();
        let step = scorer
            .step_state(WIDENED_TOP_M)
            .map_err(LoadError::Scorer)?;
        let token_rows = u32::try_from(artifacts.token_codes.len() / STAGES)
            .map_err(|_| LoadError::TeacherTooLarge)?;

        Ok(Self {
            artifacts,
            scorer,
            rotations: compiler::derive_rotations(),
            tokenizer,
            token_rows,
            policy,
            step,
            step_supported,
            counters: PolicyCounters::default(),
            novel_seen: NovelSeen::new(NOVEL_SEEN_CAPACITY),
        })
    }

    /// Reset the session state: status-path counters, the widen-once
    /// memory, and the scoring scratch. The loaded artifacts and policy
    /// are untouched. Rebuilding the scratch cannot fail for the scorer
    /// it was built from; on that unreachable error the previous scratch
    /// is retained.
    pub fn reset(&mut self) {
        self.counters = PolicyCounters::default();
        self.novel_seen = NovelSeen::new(NOVEL_SEEN_CAPACITY);
        if let Ok(step) = self.scorer.step_state(WIDENED_TOP_M) {
            self.step = step;
        }
    }

    /// The version surface this engine runs against.
    pub fn abi_version(&self) -> AbiVersion {
        AbiVersion::current()
    }

    /// Encode text with the bundle-matched tokenizer when one is
    /// available. (The tokenizer's BPE path allocates an intermediate
    /// `String`; text helpers sit outside the allocation-free step path.)
    pub fn encode_text_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        self.tokenizer.as_ref()?.encode_into(text, out).ok()
    }

    /// Decode tokens with the bundle-matched tokenizer when one is
    /// available.
    pub fn decode_tokens_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out).ok()
    }
}

/// Pinned quality floor (issue #110, era: #65-chain anchors). The deployed
/// graph must not digress from the Rule 1+2 anchors the quality chain
/// measured (31.7086% top-1, 9.8612 bits/token) beyond the margins the CI
/// trend alarm allows. Keep these constants in sync with
/// `scripts/check_gate_c_regression.py`; when a compiler redesign
/// legitimately moves the anchors, update both sites in the same commit with
/// an era note.
const QUALITY_FLOOR_TOP1_AGREEMENT: f64 = 0.317 - 0.02;
const QUALITY_FLOOR_BITS_PER_TOKEN: f64 = 9.86 + 0.10;

/// Validate the graph's Rule 1+2 quality against the declared quality basis.
/// Reports without a profile retain the historical pinned-floor behavior.
/// Dynamic Hugging Face builds use `config.quality_profile = "relative_tla"`
/// because their teacher-generated distributions are not comparable to the
/// legacy fixture corpus that established the absolute floor.
pub fn validate_quality_report(report: &serde_json::Value) -> Result<(), String> {
    let graph_agreement = report
        .pointer("/gate_c/rule12_precedence/top1_agreement")
        .and_then(serde_json::Value::as_f64);
    let graph_bits = report
        .pointer("/gate_c/rule12_precedence/bits_per_token")
        .and_then(serde_json::Value::as_f64);
    let baseline_agreement = report
        .pointer("/gate_c/tla3_baseline/top1_agreement")
        .and_then(serde_json::Value::as_f64);
    if let (Some(graph), Some(baseline)) = (graph_agreement, baseline_agreement) {
        if graph < baseline {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime top-1 {:.2}% is below TLA baseline {:.2}%",
                graph * 100.0,
                baseline * 100.0
            ));
        }
    }
    let quality_profile = report
        .pointer("/config/quality_profile")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("pinned");
    if quality_profile == "relative_tla" {
        return Ok(());
    }
    if let Some(graph) = graph_agreement {
        if graph < QUALITY_FLOOR_TOP1_AGREEMENT {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime top-1 {:.2}% digresses below the pinned floor {:.2}%",
                graph * 100.0,
                QUALITY_FLOOR_TOP1_AGREEMENT * 100.0
            ));
        }
    }
    if let Some(bits) = graph_bits {
        if bits > QUALITY_FLOOR_BITS_PER_TOKEN {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime {:.4} bits/token digresses above the pinned ceiling {:.4}",
                bits, QUALITY_FLOOR_BITS_PER_TOKEN
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request_serde_roundtrip() {
        let req = InferenceRequest {
            text: "Hello, world!".to_string(),
            identity: Some("user_123".to_string()),
            engine: Some("r4g1".to_string()),
            max_tokens: Some(32),
            temperature: Some(0.7),
        };
        let json = serde_json::to_string(&req).expect("serialize InferenceRequest");
        let decoded: InferenceRequest =
            serde_json::from_str(&json).expect("deserialize InferenceRequest");
        assert_eq!(req, decoded);
    }

    #[test]
    fn test_inference_response_serde_roundtrip() {
        let res = InferenceResponse {
            text: "Response generated cleanly.".to_string(),
            engine: "r4g1".to_string(),
            llm_connected: true,
            tokens_generated: 16,
            abstained: false,
            status: Some("exact_context".to_string()),
            widened: false,
            generation_mode: "r4g1-zero-multiply".to_string(),
            error: None,
        };
        let json = serde_json::to_string(&res).expect("serialize InferenceResponse");
        let decoded: InferenceResponse =
            serde_json::from_str(&json).expect("deserialize InferenceResponse");
        assert_eq!(res, decoded);
    }

    #[test]
    fn test_load_error_tokenizer_cid_mismatch_display() {
        let err = LoadError::TokenizerCidMismatch {
            expected: "blake3:1111".to_string(),
            actual: "blake3:2222".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "tokenizer_cid mismatch: header expected blake3:1111, loaded blake3:2222"
        );
    }
}
