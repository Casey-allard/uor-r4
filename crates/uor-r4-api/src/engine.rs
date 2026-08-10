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

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::compiler::{self, Compiled, SIG_BYTES, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_format::{r4g1, SectionId};

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
    /// Ask a serving endpoint to include a replayable proof summary.
    /// Witness assembly is opt-in and remains outside the default hot path.
    #[serde(default)]
    pub include_witness: bool,
}

/// Compact, per-token proof summary for opt-in serving responses.
///
/// The full scorer witness remains an internal verification artifact. This
/// envelope carries the claims clients need to audit a response while the
/// verifier recomputes the selected token and traversal from the held graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceWitness {
    /// Content address of the region identity that supplied the answer.
    /// `None` means exact-context evidence or a novel/abstaining probe did
    /// not select a covered graph region.
    pub region_kappa: Option<String>,
    /// Zero-based region id within the NODE section, when a graph region
    /// answered the probe.
    pub region_id: Option<u32>,
    /// Number of covered graph regions in the selected traversal.
    pub depth: u8,
    /// `exact_context`, `graph`, or `novel`.
    pub resolution_status: String,
    /// Serving engine that produced the witness.
    pub engine: String,
    /// Token bound to this witness claim.
    pub token: u32,
    /// Whether the policy had to use its widened membership pass.
    pub widened: bool,
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
    /// Optional per-token proof claims, populated only when requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<Vec<InferenceWitness>>,
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
    InvalidTokenizer,
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
            LoadError::InvalidTokenizer => write!(f, "invalid tokenizer bytes"),
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

/// Typed rejection reasons for an opt-in response witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessVerificationError {
    /// The response and witness arrays do not describe the same span.
    LengthMismatch,
    /// The claimed token differs from the artifact replay.
    TokenMismatch,
    /// The claimed region identity differs from the artifact replay.
    RegionMismatch,
    /// The claimed traversal depth differs from the artifact replay.
    DepthMismatch,
    /// The claimed resolution status differs from the artifact replay.
    StatusMismatch,
    /// The claimed serving engine is not the engine being verified.
    EngineMismatch,
    /// The claimed widening flag differs from the replay.
    WidenedMismatch,
}

impl fmt::Display for WitnessVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::LengthMismatch => "witness_length_mismatch",
            Self::TokenMismatch => "witness_token_mismatch",
            Self::RegionMismatch => "witness_region_mismatch",
            Self::DepthMismatch => "witness_depth_mismatch",
            Self::StatusMismatch => "witness_status_mismatch",
            Self::EngineMismatch => "witness_engine_mismatch",
            Self::WidenedMismatch => "witness_widened_mismatch",
        };
        f.write_str(reason)
    }
}

impl std::error::Error for WitnessVerificationError {}

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
    /// The served token came from an explicit NGRAM context row (#362
    /// attribution; only possible when `status` is `ExactContext`).
    pub ngram_hit: bool,
}

/// A typed abstention: no token was emitted and none is guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbstainOutcome {
    pub status: ScoreStatus,
    pub widened: bool,
    /// The abstained status resolved via an explicit NGRAM context row.
    pub ngram_hit: bool,
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
    /// The selection came from an explicit NGRAM context row (#362
    /// attribution; only possible when `status` is `ExactContext`).
    ngram_hit: bool,
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
    /// Representation-level artifact address used to derive stable region
    /// identities for opt-in witnesses. Computed once at load time.
    artifact_kappa: String,
    /// Address of the NODE section, the canonical region-object namespace
    /// until standalone region manifests land.
    node_section_kappa: Option<String>,
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

    /// Derive the packed input signature AND the graded code of a token
    /// window (#243 Phase C option A): the code is assigned under the
    /// artifact's declared metric (`assign_for_bundle` — shift-add dot
    /// on TLA6, sign-Hamming otherwise) and attested into the witness;
    /// the sig keys the cover memberships either way.
    fn derive_sig_code(&self, window: &[u32]) -> ([u8; SIG_BYTES], [u8; compiler::STAGES]) {
        let bundle = runtime::bundle_window_plain(&self.artifacts, &self.rotations, window);
        let sig = runtime::sig_plain(&self.artifacts, &bundle);
        // Allocation-free variant: steady-state serving is censused
        // (tests/status_policy_census.rs) — the membership-beam
        // materializing assign_for_bundle allocates and must not be
        // called per prediction.
        let code = runtime::assign_code_for_bundle(&self.artifacts, &bundle);
        (sig, code)
    }

    /// Derive the sign signature for a token window for certifier-side
    /// experiments. This exposes no deployed scoring semantics; callers still
    /// receive the same artifact-derived signature used by the graph path.
    pub fn signature_for_window(&self, window: &[u32]) -> Result<[u8; SIG_BYTES], InferenceError> {
        self.check_window(window)?;
        Ok(self.derive_sig_code(window).0)
    }

    /// Derive the stable κ-label for a region identity. The identity is
    /// anchored to the canonical NODE section address and node id, so a
    /// changed graph section or node changes the witness claim. This keeps
    /// witness verification honest while the standalone region-object
    /// manifest/resolver work in #263 is completed.
    fn region_kappa(&self, node_id: u32) -> Option<String> {
        let section = self.node_section_kappa.as_deref()?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4-region-witness-v1\0");
        hasher.update(self.artifact_kappa.as_bytes());
        hasher.update(section.as_bytes());
        hasher.update(&node_id.to_le_bytes());
        Some(format!("kappa:blake3:{}", hasher.finalize().to_hex()))
    }

    fn compact_witness(
        &self,
        witness: &uor_r4_graph_certify::ScoreWitness,
        widened: bool,
    ) -> InferenceWitness {
        let node_id = witness.chain.last().copied();
        InferenceWitness {
            region_kappa: node_id.and_then(|node| self.region_kappa(node)),
            region_id: node_id.map(|node| node.saturating_sub(1)),
            depth: witness.chain.len().min(u8::MAX as usize) as u8,
            resolution_status: PolicyStatus::from(witness.status).label().to_owned(),
            engine: "r4g1".to_owned(),
            token: witness.selected,
            widened,
        }
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
        input_code: Option<&[u8; compiler::STAGES]>,
        top_m: usize,
        recent_tokens: &[u32],
    ) -> Result<ScoredProbe, InferenceError> {
        if self.step_supported {
            let outcome = self
                .scorer
                .score_step_coded_with_recent(sig, input_code, top_m, &mut self.step, recent_tokens)
                .ok_or_else(|| {
                    InferenceError::Scorer("deployed step produced no outcome".to_owned())
                })?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.status,
                ngram_hit: outcome.exact_context_source
                    == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
            })
        } else {
            let outcome = self
                .scorer
                .score_candidates_coded(sig, input_code, recent_tokens)
                .ok_or_else(|| {
                    InferenceError::Scorer(
                        "scorer produced no candidates for the probe signature".to_owned(),
                    )
                })?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.witness.status,
                ngram_hit: outcome.exact_context_source
                    == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
            })
        }
    }

    /// Reference scorer used only by the opt-in witness path. The normal
    /// serving path continues to use the allocation-free step scorer.
    fn score_sig_witness(
        &mut self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; compiler::STAGES]>,
        top_m: usize,
        recent_tokens: &[u32],
    ) -> Result<uor_r4_graph_certify::ScoreOutcome, InferenceError> {
        // The reference scorer has a fixed membership width today; keeping
        // the parameter at the call site makes the witness contract explicit
        // and avoids silently claiming widening when the artifact cannot
        // support it.
        let _ = top_m;
        self.scorer
            .score_candidates_coded(sig, input_code, recent_tokens)
            .ok_or_else(|| {
                InferenceError::Scorer(
                    "scorer produced no candidates for the probe signature".to_owned(),
                )
            })
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
        self.predict_signature_status_with_recent(sig, None, &[])
    }

    fn predict_signature_status_with_recent(
        &mut self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; compiler::STAGES]>,
        recent_tokens: &[u32],
    ) -> Result<PredictDecision, InferenceError> {
        self.counters.predicts += 1;
        let first = self.score_sig(sig, input_code, TOP_M, recent_tokens)?;
        match self.policy.action(first.status.into()) {
            StatusAction::Serve => {
                self.counters.serves += 1;
                Ok(PredictDecision::Serve(PredictOutcome {
                    token: first.token,
                    status: first.status,
                    widened: false,
                    ngram_hit: first.ngram_hit,
                }))
            }
            StatusAction::Abstain => {
                self.counters.abstains += 1;
                Ok(PredictDecision::Abstain(AbstainOutcome {
                    status: first.status,
                    widened: false,
                    ngram_hit: first.ngram_hit,
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
                        ngram_hit: first.ngram_hit,
                    }));
                }
                if self.novel_seen.contains(sig) {
                    self.counters.widen_skipped_seen += 1;
                    self.counters.abstains += 1;
                    return Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                        ngram_hit: first.ngram_hit,
                    }));
                }
                self.counters.widen_attempts += 1;
                let second = self.score_sig(sig, input_code, WIDENED_TOP_M, recent_tokens)?;
                if second.status == ScoreStatus::Novel {
                    self.novel_seen.insert(sig);
                }
                if self.policy.action(second.status.into()) == StatusAction::Serve {
                    self.counters.serves += 1;
                    Ok(PredictDecision::Serve(PredictOutcome {
                        token: second.token,
                        status: second.status,
                        widened: true,
                        ngram_hit: second.ngram_hit,
                    }))
                } else {
                    self.counters.abstains += 1;
                    Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: second.status,
                        widened: true,
                        ngram_hit: second.ngram_hit,
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
        let (sig, code) = self.derive_sig_code(window);
        self.predict_signature_status_with_recent(&sig, Some(&code), window)
    }

    /// Predict one window and retain the compact proof claim for the
    /// selected result. This is deliberately separate from
    /// [`Self::predict_decision`]: witness assembly uses the allocating
    /// reference scorer and is never paid by ordinary inference.
    pub fn predict_decision_with_witness(
        &mut self,
        window: &[u32],
    ) -> Result<(PredictDecision, InferenceWitness), InferenceError> {
        self.check_window(window)?;
        let (sig, code) = self.derive_sig_code(window);
        self.counters.predicts += 1;
        let first = self.score_sig_witness(&sig, Some(&code), TOP_M, window)?;
        let first_witness = self.compact_witness(&first.witness, false);
        match self.policy.action(first.witness.status.into()) {
            StatusAction::Serve => {
                self.counters.serves += 1;
                Ok((
                    PredictDecision::Serve(PredictOutcome {
                        token: first.selected,
                        status: first.witness.status,
                        widened: false,
                        ngram_hit: first.exact_context_source
                            == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
                    }),
                    first_witness,
                ))
            }
            StatusAction::Abstain => {
                self.counters.abstains += 1;
                Ok((
                    PredictDecision::Abstain(AbstainOutcome {
                        status: first.witness.status,
                        widened: false,
                        ngram_hit: first.exact_context_source
                            == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
                    }),
                    first_witness,
                ))
            }
            StatusAction::WidenOnce => {
                if !self.step_supported || self.novel_seen.contains(&sig) {
                    self.counters.abstains += 1;
                    return Ok((
                        PredictDecision::Abstain(AbstainOutcome {
                            status: first.witness.status,
                            widened: false,
                            ngram_hit: first.exact_context_source
                                == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
                        }),
                        first_witness,
                    ));
                }
                self.counters.widen_attempts += 1;
                let second = self.score_sig_witness(&sig, Some(&code), WIDENED_TOP_M, window)?;
                let second_witness = self.compact_witness(&second.witness, true);
                if second.witness.status == ScoreStatus::Novel {
                    self.novel_seen.insert(&sig);
                }
                if self.policy.action(second.witness.status.into()) == StatusAction::Serve {
                    self.counters.serves += 1;
                    Ok((
                        PredictDecision::Serve(PredictOutcome {
                            token: second.selected,
                            status: second.witness.status,
                            widened: true,
                            ngram_hit: second.exact_context_source
                                == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
                        }),
                        second_witness,
                    ))
                } else {
                    self.counters.abstains += 1;
                    Ok((
                        PredictDecision::Abstain(AbstainOutcome {
                            status: second.witness.status,
                            widened: true,
                            ngram_hit: second.exact_context_source
                                == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
                        }),
                        second_witness,
                    ))
                }
            }
        }
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

    /// Witness-enabled generation. The caller owns the witness vector so
    /// response assembly can choose its allocation and serialization policy.
    pub fn generate_into_with_witness(
        &mut self,
        seed: &[u32],
        out: &mut [u32],
        witnesses: &mut Vec<InferenceWitness>,
    ) -> Result<GenerateStatus, InferenceError> {
        self.check_window(seed)?;
        witnesses.clear();
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);
        let mut last_status = None;
        let mut widened = false;
        for (generated, token) in out.iter_mut().enumerate() {
            let (decision, witness) = self.predict_decision_with_witness(&window[..window_len])?;
            match decision {
                PredictDecision::Serve(outcome) => {
                    let next = outcome.token;
                    last_status = Some(outcome.status);
                    widened = widened || outcome.widened;
                    if next == 1 || next == 2 {
                        return Ok(GenerateStatus {
                            count: generated,
                            status: last_status,
                            widened,
                            abstained: false,
                        });
                    }
                    *token = next;
                    witnesses.push(witness);
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

    /// Independently replay compact witnesses against this loaded artifact.
    /// This is server-side verification only; it does not mutate the normal
    /// allocation-free step scratch or status counters.
    pub fn verify_witnesses(
        &mut self,
        seed: &[u32],
        generated: &[u32],
        witnesses: &[InferenceWitness],
    ) -> Result<(), WitnessVerificationError> {
        if generated.len() != witnesses.len() {
            return Err(WitnessVerificationError::LengthMismatch);
        }
        self.check_window(seed)
            .map_err(|_| WitnessVerificationError::LengthMismatch)?;
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);
        for (&token, claimed) in generated.iter().zip(witnesses) {
            let (sig, code) = self.derive_sig_code(&window[..window_len]);
            let top_m = if claimed.widened {
                WIDENED_TOP_M
            } else {
                TOP_M
            };
            let outcome = self
                .score_sig_witness(&sig, Some(&code), top_m, &window[..window_len])
                .map_err(|_| WitnessVerificationError::StatusMismatch)?;
            let expected = self.compact_witness(&outcome.witness, claimed.widened);
            if claimed.engine != "r4g1" {
                return Err(WitnessVerificationError::EngineMismatch);
            }
            if claimed.token != token || claimed.token != expected.token {
                return Err(WitnessVerificationError::TokenMismatch);
            }
            if claimed.region_kappa != expected.region_kappa
                || claimed.region_id != expected.region_id
            {
                return Err(WitnessVerificationError::RegionMismatch);
            }
            if claimed.depth != expected.depth {
                return Err(WitnessVerificationError::DepthMismatch);
            }
            if claimed.resolution_status != expected.resolution_status {
                return Err(WitnessVerificationError::StatusMismatch);
            }
            if claimed.widened != expected.widened {
                return Err(WitnessVerificationError::WidenedMismatch);
            }
            if window_len < WINDOW {
                window[window_len] = token;
                window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[WINDOW - 1] = token;
            }
        }
        Ok(())
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
        let view = GraphView::parse(parts.graph).map_err(|e| LoadError::InvalidGraph(e.reason))?;
        view.verify_cids()
            .map_err(|k| LoadError::InvalidGraph(k.as_format()))?;
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
        .ok_or_else(|| {
            LoadError::Scorer(
                "signature artifact is not a scorer (parse, CID, or teacher_cid)".to_owned(),
            )
        })?;
        if let Some(report) = score_report.as_ref() {
            validate_quality_report(report).map_err(LoadError::QualityGate)?;
        }
        let tokenizer = parts
            .tokenizer
            .map(|bytes| Tokenizer::from_bytes(bytes).ok_or(LoadError::InvalidTokenizer))
            .transpose()?;

        let policy = StatusPolicy::from_report(score_report.as_ref());
        let step_supported = !scorer.has_legacy_exct();
        let step = scorer.step_state(WIDENED_TOP_M).ok_or_else(|| {
            LoadError::Scorer("scorer does not support a deployed step state".to_owned())
        })?;
        let token_rows = u32::try_from(artifacts.token_codes.len() / STAGES)
            .map_err(|_| LoadError::TeacherTooLarge)?;
        let artifact_kappa = r4g1::artifact_kappa(parts.graph)
            .ok_or_else(|| LoadError::Scorer("R4G1 artifact is not addressable".to_string()))?;
        // The graph is known addressable here (artifact_kappa succeeded), so a
        // `None` is a genuinely absent NODE section, kept as such.
        let node_section_kappa = r4g1::section_kappa(parts.graph, SectionId::NODE);

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
            artifact_kappa,
            node_section_kappa,
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
        if let Some(step) = self.scorer.step_state(WIDENED_TOP_M) {
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
        self.tokenizer.as_ref()?.encode_into(text, out)
    }

    /// Decode tokens with the bundle-matched tokenizer when one is
    /// available.
    pub fn decode_tokens_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out)
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
            include_witness: false,
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
            witness: None,
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
