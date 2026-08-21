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
    /// Temperature for geometric sampling; `0` additionally opts the
    /// R4G1 tier into deterministic greedy decode (#655 decode-default
    /// decision, 2026-08-19 — sampled decode is otherwise the default).
    pub temperature: Option<f64>,
    /// Sampling seed override for the R4G1 tier's default sampled
    /// decode; absent requests use the pinned default seed so default
    /// serving stays reproducible. Ignored under `temperature: 0`.
    #[serde(default)]
    pub seed: Option<u32>,
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
    /// #836 segment-lane attribution: present only when the deployed segment
    /// lane changed the served token. Absent (skipped in JSON) for every
    /// artifact without a PSTATE section, so the witness is unchanged — the
    /// witness-level absent-section identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_lane: Option<SegmentLaneWitness>,
}

/// #836 segment-lane witness attribution: which candidate the deployed segment
/// lane promoted to served, the base-scorer token it displaced, and the raw
/// `ScoreQ` boost that promoted it — enough for a verifier to reproduce the
/// re-rank over the decided candidate list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SegmentLaneWitness {
    /// The candidate the segment lane promoted to the served token.
    pub promoted_token: u32,
    /// The base-scorer token the promotion displaced (what would have served
    /// without the lane).
    pub base_token: u32,
    /// The raw `ScoreQ` boost the lane added to the promoted candidate.
    pub boost: i32,
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
use uor_r4_core::transformerless::scenarios::{RuntimeTokenizerIdentity, Tokenizer};
use uor_r4_graph_certify::{
    GraphScorer, ScoreStatus, StepCandidates, StepState, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B,
    TOP_M, WIDENED_TOP_M,
};
use uor_r4_graph_format::{
    ContractVersion, FormatError, GraphView, ObservedBound, ScoreQ, FORMAT_VERSION_MAJOR,
    FORMAT_VERSION_MINOR, INFERENCE_OPERATION_CONTRACT_VERSION, LANE_SEGMENT,
};
use uor_r4_graph_runtime::runtime_state::{SegmentSession, SEGMENT_STATE_CAPACITY};
use uor_r4_model_source::SourceUnavailable;

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
    /// The optional #835 segment-lane descriptor read from the artifact's
    /// PSTATE section at load, or `None` when the artifact carries no PSTATE
    /// (absent-section identity). Drives [`R4Engine::segment_session`].
    segment_lane: Option<SegmentLaneCfg>,
    /// The optional learned content-token→candidate residual table (#836 4c),
    /// extracted once at load from the PSTATE rows and sorted by content key.
    /// `None` when the PSTATE section carries no rows (a config-only descriptor
    /// → the recurrence scorer) or when there is no PSTATE section at all. When
    /// present, the deployed re-rank sums each prompt content token's learned
    /// contributions — the faithful lowering of the #834 §6.2 segment arm.
    segment_table: Option<SegmentTable>,
}

/// The compiled segment-lane descriptor the engine consumes to build a
/// [`SegmentSession`] (#836). Read once from the PSTATE section at load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SegmentLaneCfg {
    decay_shift: u32,
    base_w: ScoreQ,
    boost: ScoreQ,
}

/// The owned learned content-to-candidate residual table (#836 4c): content
/// keys sorted, each mapping to its candidate->ScoreQ entries sorted by candidate.
type SegmentTableRow = (u32, Vec<(u32, i32)>);
type SegmentTable = Vec<SegmentTableRow>;

/// The token maximizing `base_score + contribution(token)` over the decided
/// top-K list, under the canonical tie-break (score descending, then token id
/// ascending). Returns `base_token` when the list is empty. Pure, allocation-
/// free, P-4 (saturating integer add and comparison only).
fn segment_argmax(
    ranked: &[(u32, ScoreQ)],
    base_token: u32,
    contribution: impl Fn(u32) -> i32,
) -> u32 {
    let Some(&(first_token, first_score)) = ranked.first() else {
        return base_token;
    };
    let mut best_token = first_token;
    let mut best_adjusted = first_score.raw().saturating_add(contribution(first_token));
    for &(token, score) in &ranked[1..] {
        let adjusted = score.raw().saturating_add(contribution(token));
        if adjusted > best_adjusted || (adjusted == best_adjusted && token < best_token) {
            best_adjusted = adjusted;
            best_token = token;
        }
    }
    best_token
}

/// Recurrence (config-only) segment re-rank: each candidate present in the
/// prompt-content ring gains the descriptor `boost`. Used when the artifact
/// ships a descriptor with no residual table.
fn segment_adjusted_token<const CAP: usize>(
    ranked: &[(u32, ScoreQ)],
    session: &SegmentSession<CAP>,
    base_token: u32,
) -> u32 {
    segment_argmax(ranked, base_token, |token| {
        session.contribution(token).raw()
    })
}

/// The learned `ScoreQ` a content token contributes to a candidate, or 0 when
/// the content token or the candidate is absent from the table. Two binary
/// searches; no arithmetic operators.
fn table_score(table: &[SegmentTableRow], content_key: u32, candidate: u32) -> i32 {
    let Ok(row) = table.binary_search_by_key(&content_key, |(key, _)| *key) else {
        return 0;
    };
    match table[row]
        .1
        .binary_search_by_key(&candidate, |(token, _)| *token)
    {
        Ok(entry) => table[row].1[entry].1,
        Err(_) => 0,
    }
}

/// The summed learned contribution to `candidate` from the prompt content
/// tokens still live in the ring. Bounded (ring capacity × per-row binary
/// search); saturating add; P-4.
fn table_contribution<const CAP: usize>(
    table: &[SegmentTableRow],
    session: &SegmentSession<CAP>,
    candidate: u32,
) -> i32 {
    let mut acc: i32 = 0;
    for key in session.content_keys() {
        acc = acc.saturating_add(table_score(table, key, candidate));
    }
    acc
}

/// Learned-table (#836 4c) segment re-rank: each candidate gains the sum, over
/// the live prompt-content tokens, of that content token's learned `ScoreQ`
/// contribution to the candidate (the content→candidate table packed in
/// PSTATE). Faithfully lowers the #834 §6.2 segment arm.
fn segment_adjusted_token_with_table<const CAP: usize>(
    ranked: &[(u32, ScoreQ)],
    session: &SegmentSession<CAP>,
    table: &[SegmentTableRow],
    base_token: u32,
) -> u32 {
    segment_argmax(ranked, base_token, |candidate| {
        table_contribution(table, session, candidate)
    })
}

/// The #836 segment-lane witness attribution over a decided candidate list:
/// `Some` only when the lane is active AND promotes a different token than the
/// base winner; `None` otherwise (inactive lane, or no change → no attribution,
/// preserving witness identity). Pure.
fn segment_lane_attribution<const CAP: usize>(
    ranked: &[(u32, ScoreQ)],
    session: &SegmentSession<CAP>,
    base_token: u32,
) -> Option<SegmentLaneWitness> {
    if !session.is_active() {
        return None;
    }
    let promoted = segment_adjusted_token(ranked, session, base_token);
    if promoted == base_token {
        return None;
    }
    Some(SegmentLaneWitness {
        promoted_token: promoted,
        base_token,
        boost: session.contribution(promoted).raw(),
    })
}

/// Learned-table (#836 4c) counterpart of [`segment_lane_attribution`]: `Some`
/// when the table re-rank promotes a different token, carrying the summed
/// learned boost that promoted it.
fn segment_lane_attribution_with_table<const CAP: usize>(
    ranked: &[(u32, ScoreQ)],
    session: &SegmentSession<CAP>,
    table: &[SegmentTableRow],
    base_token: u32,
) -> Option<SegmentLaneWitness> {
    if !session.is_active() {
        return None;
    }
    let promoted = segment_adjusted_token_with_table(ranked, session, table, base_token);
    if promoted == base_token {
        return None;
    }
    Some(SegmentLaneWitness {
        promoted_token: promoted,
        base_token,
        boost: table_contribution(table, session, promoted),
    })
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
    pub fn signature_for_window(&self, window: &[u32]) -> Result<[u8; SIG_BYTES], ObservedBound> {
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
            // The base reference witness carries no segment attribution; the
            // deployed segment-witness path fills it when the lane promotes.
            segment_lane: None,
        }
    }

    /// Reject a window carrying a token id the teacher artifact cannot
    /// decode. Also enforces the `WINDOW = 8` Dyadic-Recency boundary, emitting
    /// a `tracing::warn!` log if `window.len() > 8` before sliding window truncation.
    fn check_window(&self, window: &[u32]) -> Result<(), ObservedBound> {
        if window.len() > WINDOW {
            tracing::warn!(
                target: "uor_r4_core::runtime",
                window_size = WINDOW,
                input_size = window.len(),
                "Input context exceeds 8-token window; truncating to 8 most recent tokens"
            );
        }
        if let Some(&token) = window.iter().find(|&&t| t >= self.token_rows) {
            return Err(ObservedBound {
                observed: i64::from(token),
                bound: i64::from(self.token_rows),
            });
        }
        Ok(())
    }

    /// Score one signature at the given membership width. Artifacts
    /// with legacy TLS1 exact-context evidence stay on the reference
    /// scorer (the deployed step requires residualized RX1 evidence);
    /// that path ignores the width — widening is unavailable there.
    /// A supplied sink additionally receives the step's bounded top-K
    /// candidate list. Selection and status are identical either way;
    /// on the legacy reference path the list is rebuilt from the
    /// reference outcome's own candidate vector under the same
    /// canonical order.
    fn score_sig_with_candidates(
        &mut self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; compiler::STAGES]>,
        top_m: usize,
        recent_tokens: &[u32],
        candidates: Option<&mut StepCandidates>,
    ) -> ScoredProbe {
        if self.step_supported {
            let outcome = match candidates {
                Some(list) => self
                    .scorer
                    .score_step_candidates_coded_with_recent(
                        sig,
                        input_code,
                        top_m,
                        &mut self.step,
                        recent_tokens,
                        list,
                    )
                    .expect("serving: scorer produced no candidates for a validated sig"),
                None => self
                    .scorer
                    .score_step_coded_with_recent(
                        sig,
                        input_code,
                        top_m,
                        &mut self.step,
                        recent_tokens,
                    )
                    .expect("serving: scorer produced no candidates for a validated sig"),
            };
            ScoredProbe {
                token: outcome.selected,
                status: outcome.status,
                ngram_hit: outcome.exact_context_source
                    == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
            }
        } else {
            let outcome = self
                .scorer
                .score_candidates_coded(sig, input_code, recent_tokens)
                .expect("serving: scorer produced no candidates for a validated sig");
            if let Some(list) = candidates {
                list.len = 0;
                for &(token, score) in &outcome.candidates {
                    list.push_ranked(token, score);
                }
            }
            ScoredProbe {
                token: outcome.selected,
                status: outcome.witness.status,
                ngram_hit: outcome.exact_context_source
                    == Some(uor_r4_graph_certify::ExactContextSource::NgramRow),
            }
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
    ) -> uor_r4_graph_certify::ScoreOutcome {
        // The reference scorer has a fixed membership width today; keeping
        // the parameter at the call site makes the witness contract explicit
        // and avoids silently claiming widening when the artifact cannot
        // support it.
        let _ = top_m;
        self.scorer
            .score_candidates_coded(sig, input_code, recent_tokens)
            .expect("serving: scorer produced no candidates for a validated sig")
    }

    /// The D4 policy decision for one input signature: score at the
    /// manifest membership width, then Serve / WidenOnce / Abstain per
    /// the declared policy. WidenOnce re-probes once at
    /// [`WIDENED_TOP_M`]; a signature still Novel after widening is
    /// remembered so identical probes abstain without widening again.
    pub fn predict_signature_status(&mut self, sig: &[u8; SIG_BYTES]) -> PredictDecision {
        self.predict_signature_status_with_recent(sig, None, &[])
    }

    fn predict_signature_status_with_recent(
        &mut self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; compiler::STAGES]>,
        recent_tokens: &[u32],
    ) -> PredictDecision {
        self.predict_signature_status_with_recent_candidates(sig, input_code, recent_tokens, None)
    }

    /// The same policy path with an optional candidate sink: when the
    /// decision serves, the sink holds the bounded top-K list of the
    /// exact probe that served (the widened probe when widening served).
    /// Policy bookkeeping — counters, widen-once, the novel-seen FIFO —
    /// is this one implementation either way.
    fn predict_signature_status_with_recent_candidates(
        &mut self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; compiler::STAGES]>,
        recent_tokens: &[u32],
        mut candidates: Option<&mut StepCandidates>,
    ) -> PredictDecision {
        self.counters.predicts += 1;
        let first = self.score_sig_with_candidates(
            sig,
            input_code,
            TOP_M,
            recent_tokens,
            candidates.as_deref_mut(),
        );
        match self.policy.action(first.status.into()) {
            StatusAction::Serve => {
                self.counters.serves += 1;
                PredictDecision::Serve(PredictOutcome {
                    token: first.token,
                    status: first.status,
                    widened: false,
                    ngram_hit: first.ngram_hit,
                })
            }
            StatusAction::Abstain => {
                self.counters.abstains += 1;
                PredictDecision::Abstain(AbstainOutcome {
                    status: first.status,
                    widened: false,
                    ngram_hit: first.ngram_hit,
                })
            }
            StatusAction::WidenOnce => {
                if !self.step_supported {
                    // No widening on the legacy reference path: abstain
                    // directly (documented degrade for TLS1 artifacts).
                    self.counters.abstains += 1;
                    return PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                        ngram_hit: first.ngram_hit,
                    });
                }
                if self.novel_seen.contains(sig) {
                    self.counters.widen_skipped_seen += 1;
                    self.counters.abstains += 1;
                    return PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                        ngram_hit: first.ngram_hit,
                    });
                }
                self.counters.widen_attempts += 1;
                let second = self.score_sig_with_candidates(
                    sig,
                    input_code,
                    WIDENED_TOP_M,
                    recent_tokens,
                    candidates,
                );
                if second.status == ScoreStatus::Novel {
                    self.novel_seen.insert(sig);
                }
                if self.policy.action(second.status.into()) == StatusAction::Serve {
                    self.counters.serves += 1;
                    PredictDecision::Serve(PredictOutcome {
                        token: second.token,
                        status: second.status,
                        widened: true,
                        ngram_hit: second.ngram_hit,
                    })
                } else {
                    self.counters.abstains += 1;
                    PredictDecision::Abstain(AbstainOutcome {
                        status: second.status,
                        widened: true,
                        ngram_hit: second.ngram_hit,
                    })
                }
            }
        }
    }

    /// The status-aware decision for one token window: score the packed
    /// signature through the D4 policy. Abstention is a typed outcome —
    /// no guessed token is emitted.
    pub fn predict_decision(&mut self, window: &[u32]) -> Result<PredictDecision, ObservedBound> {
        self.check_window(window)?;
        let (sig, code) = self.derive_sig_code(window);
        Ok(self.predict_signature_status_with_recent(&sig, Some(&code), window))
    }

    /// Predict one window and retain the compact proof claim for the
    /// selected result. This is deliberately separate from
    /// [`Self::predict_decision`]: witness assembly uses the allocating
    /// reference scorer and is never paid by ordinary inference.
    pub fn predict_decision_with_witness(
        &mut self,
        window: &[u32],
    ) -> Result<(PredictDecision, InferenceWitness), ObservedBound> {
        self.check_window(window)?;
        let (sig, code) = self.derive_sig_code(window);
        self.counters.predicts += 1;
        let first = self.score_sig_witness(&sig, Some(&code), TOP_M, window);
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
                let second = self.score_sig_witness(&sig, Some(&code), WIDENED_TOP_M, window);
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
    ) -> Result<(), ObservedBound> {
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
    ) -> Result<GenerateStatus, ObservedBound> {
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

    /// The status-aware decision for one token window, additionally
    /// filling `candidates` with the bounded top-K list of the exact
    /// probe that decided (the widened probe when widening served).
    fn predict_decision_candidates(
        &mut self,
        window: &[u32],
        candidates: &mut StepCandidates,
    ) -> Result<PredictDecision, ObservedBound> {
        self.check_window(window)?;
        let (sig, code) = self.derive_sig_code(window);
        Ok(self.predict_signature_status_with_recent_candidates(
            &sig,
            Some(&code),
            window,
            Some(candidates),
        ))
    }

    /// Build a #835 segment-lane session for the loaded artifact: an **active**
    /// session configured from the PSTATE descriptor, or an **inactive** one
    /// when the artifact carries no PSTATE (every artifact today). The caller
    /// owns the session — primes it once with the full prompt via
    /// [`SegmentSession::fold_prompt`] and decays it per generation step via
    /// [`SegmentSession::step`] — so whole-prompt content reaches the scorer
    /// without changing the bounded-window prediction interface.
    pub fn segment_session(&self) -> SegmentSession<SEGMENT_STATE_CAPACITY> {
        match self.segment_lane {
            Some(cfg) => SegmentSession::active(cfg.decay_shift, cfg.base_w, cfg.boost),
            None => SegmentSession::inactive(),
        }
    }

    /// Status-aware decision for one window, re-selecting the served token
    /// under the #835 segment lane (#836).
    ///
    /// When `session` is inactive (no PSTATE), this is **byte-identical** to
    /// [`Self::predict_decision_candidates`]: same decision, same candidate
    /// list, same served token (absent-section identity). When active, the
    /// served token becomes the segment-adjusted argmax over the decided
    /// candidate list — each candidate gains the lane's decode-independent
    /// `boost` when it is present in the primed content ring, under the same
    /// canonical tie-break (score descending, token id ascending). An
    /// abstention is never overridden into a served token.
    pub fn predict_decision_candidates_with_segment(
        &mut self,
        window: &[u32],
        candidates: &mut StepCandidates,
        session: &SegmentSession<SEGMENT_STATE_CAPACITY>,
    ) -> Result<PredictDecision, ObservedBound> {
        let decision = self.predict_decision_candidates(window, candidates)?;
        if !session.is_active() {
            return Ok(decision);
        }
        match decision {
            PredictDecision::Serve(mut outcome) => {
                outcome.token = match &self.segment_table {
                    Some(table) => segment_adjusted_token_with_table(
                        candidates.ranked(),
                        session,
                        table,
                        outcome.token,
                    ),
                    None => segment_adjusted_token(candidates.ranked(), session, outcome.token),
                };
                Ok(PredictDecision::Serve(outcome))
            }
            PredictDecision::Abstain(_) => Ok(decision),
        }
    }

    /// Like [`Self::predict_decision_candidates_with_segment`], but also returns
    /// the #836 segment-lane **witness attribution**: `Some` when the lane
    /// promoted a different served token (carrying the promoted/base tokens and
    /// the `ScoreQ` boost that promoted it), `None` otherwise — inactive lane,
    /// abstention, or a lane that left the winner unchanged. The returned
    /// decision's served token and the attribution's `promoted_token` agree by
    /// construction. Absent PSTATE → `None` → the served result is byte-identical
    /// to the base decision, and callers attach nothing to the witness.
    pub fn predict_decision_candidates_with_segment_witness(
        &mut self,
        window: &[u32],
        candidates: &mut StepCandidates,
        session: &SegmentSession<SEGMENT_STATE_CAPACITY>,
    ) -> Result<(PredictDecision, Option<SegmentLaneWitness>), ObservedBound> {
        let decision = self.predict_decision_candidates(window, candidates)?;
        match decision {
            PredictDecision::Serve(mut outcome) => {
                let attribution = match &self.segment_table {
                    Some(table) => segment_lane_attribution_with_table(
                        candidates.ranked(),
                        session,
                        table,
                        outcome.token,
                    ),
                    None => segment_lane_attribution(candidates.ranked(), session, outcome.token),
                };
                if let Some(attr) = &attribution {
                    outcome.token = attr.promoted_token;
                }
                Ok((PredictDecision::Serve(outcome), attribution))
            }
            PredictDecision::Abstain(_) => Ok((decision, None)),
        }
    }

    /// One #762-scheme draw over a served step's ranked candidates:
    /// order-preserving shift to positive weights, the soft
    /// ~1000-per-occurrence penalty over tokens already emitted this
    /// generation with floor 1 (a penalized candidate stays reachable,
    /// never excluded), and the shared division-free
    /// [`runtime::SampleRng::draw`]. `served` is returned when the list
    /// is degenerate so the policy's served selection is never
    /// overridden by an unweighable list.
    fn sample_step_candidate(
        candidates: &StepCandidates,
        emitted: &[u32],
        served: u32,
        rng: &mut runtime::SampleRng,
    ) -> u32 {
        let ranked = candidates.ranked();
        if ranked.is_empty() {
            return served;
        }
        let min_raw = ranked
            .iter()
            .map(|&(_, score)| i64::from(score.raw()))
            .min()
            .unwrap_or(0);
        let mut weights = [0u32; uor_r4_graph_certify::STEP_TOP_CANDIDATES];
        let mut total = 0u32;
        for (index, &(token, score)) in ranked.iter().enumerate() {
            let occurrences = emitted.iter().filter(|&&t| t == token).count() as i64;
            let mut weight = i64::from(score.raw()) - min_raw + 1;
            weight -= (occurrences << 10) - (occurrences << 4) - (occurrences << 3);
            weights[index] = weight.clamp(1, i64::from(u32::MAX)) as u32;
            total = total.saturating_add(weights[index]);
        }
        if total == 0 {
            return served;
        }
        let draw = rng.draw(total);
        let mut accumulated = 0u32;
        for (index, &(token, _)) in ranked.iter().enumerate() {
            accumulated = accumulated.saturating_add(weights[index]);
            if draw < accumulated {
                return token;
            }
        }
        served
    }

    /// Seeded weighted sampling over the deployed step scorer's own
    /// top-K candidates, through the same D4 policy path as
    /// [`Self::generate_into`] (#655 decode-default decision 2026-08-19;
    /// #785-C2/#762 scheme parity).
    ///
    /// Per step the policy decision — Serve / WidenOnce / Abstain, the
    /// novel-seen FIFO, every counter — is computed exactly as the
    /// greedy path computes it; sampling only replaces WHICH served
    /// candidate is emitted, weighting the serving probe's own candidate
    /// scores (deployed recent-window repetition penalty included).
    /// Abstention semantics are identical by construction: a step that
    /// abstains under greedy abstains here with the same status, and no
    /// token is guessed. A `(seed tokens, rng seed)` pair is
    /// reproducible end to end.
    pub fn generate_sampled_into(
        &mut self,
        seed: &[u32],
        out: &mut [u32],
        rng: &mut runtime::SampleRng,
    ) -> Result<GenerateStatus, ObservedBound> {
        self.check_window(seed)?;
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        let mut candidates = StepCandidates::default();
        let mut last_status = None;
        let mut widened = false;
        for generated in 0..out.len() {
            match self.predict_decision_candidates(&window[..window_len], &mut candidates)? {
                PredictDecision::Serve(outcome) => {
                    last_status = Some(outcome.status);
                    widened = widened || outcome.widened;
                    let next = Self::sample_step_candidate(
                        &candidates,
                        &out[..generated],
                        outcome.token,
                        rng,
                    );
                    out[generated] = next;
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
    ) -> Result<GenerateStatus, ObservedBound> {
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
    ///
    /// Total verifier: `None` means every witness replayed to the same claim;
    /// `Some(reason)` names the first divergence (R5, #510).
    pub fn verify_witnesses(
        &mut self,
        seed: &[u32],
        generated: &[u32],
        witnesses: &[InferenceWitness],
    ) -> Option<WitnessVerificationError> {
        if generated.len() != witnesses.len() {
            return Some(WitnessVerificationError::LengthMismatch);
        }
        if self.check_window(seed).is_err() {
            return Some(WitnessVerificationError::LengthMismatch);
        }
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
            let outcome = self.score_sig_witness(&sig, Some(&code), top_m, &window[..window_len]);
            let expected = self.compact_witness(&outcome.witness, claimed.widened);
            if claimed.engine != "r4g1" {
                return Some(WitnessVerificationError::EngineMismatch);
            }
            if claimed.token != token || claimed.token != expected.token {
                return Some(WitnessVerificationError::TokenMismatch);
            }
            if claimed.region_kappa != expected.region_kappa
                || claimed.region_id != expected.region_id
            {
                return Some(WitnessVerificationError::RegionMismatch);
            }
            if claimed.depth != expected.depth {
                return Some(WitnessVerificationError::DepthMismatch);
            }
            if claimed.resolution_status != expected.resolution_status {
                return Some(WitnessVerificationError::StatusMismatch);
            }
            if claimed.widened != expected.widened {
                return Some(WitnessVerificationError::WidenedMismatch);
            }
            if window_len < WINDOW {
                window[window_len] = token;
                window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[WINDOW - 1] = token;
            }
        }
        None
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
    pub fn load(parts: EngineParts<'_>) -> Result<Self, SourceUnavailable> {
        Self::load_with_quality_gate(parts, true)
    }

    /// [`Self::load`] minus the serving-admission quality gate: the
    /// bundle's recorded quality verdict (`validate_quality_report`)
    /// does not change what the D4 policy resolves, and a caller that
    /// has already decided — with its own recorded warning — to decode
    /// a below-baseline bundle (the CLI's local-bundle path, #655-C1e)
    /// still needs the deployed policy engine over it for the ask-path
    /// abstention gate (#811). Everything else — CID verification,
    /// tokenizer binding, teacher pairing, scorer rebuild, the
    /// status-policy/report parsing — is identical to [`Self::load`].
    /// Serving admission continues to use [`Self::load`] unchanged.
    pub fn load_accepting_quality(parts: EngineParts<'_>) -> Result<Self, SourceUnavailable> {
        Self::load_with_quality_gate(parts, false)
    }

    fn load_with_quality_gate(
        parts: EngineParts<'_>,
        enforce_quality: bool,
    ) -> Result<Self, SourceUnavailable> {
        // Fail fast with the format crate's focused reason before any scorer
        // state is built (the scorer re-validates internally). Every part is
        // an external byte source; a malformed part is reported as a
        // SourceUnavailable with the specific diagnostic preserved. Failures
        // that operate on the already-parsed-and-CID-verified graph (scorer
        // rebuild, step state, addressability) are self-produced defects and
        // panic (R5, #510).
        let view = GraphView::parse(parts.graph)
            .map_err(|e| SourceUnavailable::new(format!("invalid R4G1 graph: {}", e.reason)))?;
        view.verify_cids().map_err(|k| {
            SourceUnavailable::new(format!("invalid R4G1 graph: {}", k.as_format()))
        })?;
        let expected_tokenizer_cid = view
            .head()
            .ok_or_else(|| {
                SourceUnavailable::new(format!("invalid R4G1 graph: {}", FormatError::MissingHead))
            })?
            .tokenizer_cid();
        if expected_tokenizer_cid.0 != [0u8; 32] {
            let tokenizer_bytes = parts.tokenizer.ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "tokenizer unavailable: R4G1 header requires tokenizer.bin blake3:{}",
                    blake3::Hash::from(expected_tokenizer_cid.0).to_hex()
                ))
            })?;
            let actual = blake3::hash(tokenizer_bytes);
            if expected_tokenizer_cid.0 != *actual.as_bytes() {
                return Err(SourceUnavailable::new(format!(
                    "tokenizer_cid mismatch: header expected blake3:{}, loaded blake3:{actual}",
                    blake3::Hash::from(expected_tokenizer_cid.0).to_hex()
                )));
            }
        } else if parts
            .tokenizer
            .is_some_and(Tokenizer::is_tagged_container_bytes)
        {
            return Err(SourceUnavailable::new(
                "tokenizer binding unavailable: a tagged tokenizer requires a nonzero R4G1 header tokenizer CID",
            ));
        }
        // `parts.graph` and `parts.signature_artifact` are two
        // independently-sourced inputs when loading a persisted bundle
        // from disk (as opposed to freshly self-compiled, same-process
        // bytes) — they can legitimately drift out of pairing (for
        // example, a bundle directory reused across multiple compile runs,
        // so `score.r4g1` and its accompanying teacher file no longer
        // belong to the same generation). Check that explicitly, before
        // spending effort parsing `signature_artifact` as a TLA artifact,
        // and report a typed `SourceUnavailable` decline — rather than
        // let `from_artifact`'s internal self-produced-defect panic fire
        // further down on a condition that is not actually a code defect
        // (#743). `from_artifact` only consults `HEAD.teacher_cid` when
        // the graph carries an EXCT section
        // (uor-r4-graph-certify/src/score_runtime.rs); no EXCT section
        // means no pairing is required here either.
        if view.section(SectionId::EXCT).is_some() {
            let expected_teacher_cid = view
                .head()
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "invalid R4G1 graph: {}",
                        FormatError::MissingHead
                    ))
                })?
                .teacher_cid();
            let actual_teacher_cid = blake3::hash(parts.signature_artifact);
            if expected_teacher_cid.0 != *actual_teacher_cid.as_bytes() {
                return Err(SourceUnavailable::new(format!(
                    "teacher artifact mismatch: R4G1 header requires teacher blake3:{}, loaded teacher hashes to blake3:{actual_teacher_cid}",
                    blake3::Hash::from(expected_teacher_cid.0).to_hex()
                )));
            }
        }
        let artifacts = compiler::parse_artifacts(parts.signature_artifact)
            .ok_or_else(|| SourceUnavailable::new("not a TLA3/TLA4/TLA5 teacher artifact"))?;
        let score_report = parts
            .score_report
            .map(|bytes| {
                serde_json::from_slice::<serde_json::Value>(bytes).map_err(|error| {
                    SourceUnavailable::new(format!("invalid score report: {error}"))
                })
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
        // Pairing was already checked above (#743); anything else that
        // makes `from_artifact` decline here is an unanticipated defect
        // in the already-CID-verified, now teacher-paired graph bytes,
        // and remains the original self-produced-defect panic (R5, #510).
        let scorer = GraphScorer::from_artifact(
            parts.graph,
            Some(parts.signature_artifact),
            root_top_b,
            exct_top_x,
        )
        .expect("engine load: scorer rebuild from the parsed, CID-verified, and teacher-paired graph bytes");
        if enforce_quality {
            if let Some(report) = score_report.as_ref() {
                if let Some(message) = validate_quality_report(report) {
                    return Err(SourceUnavailable::new(message));
                }
            }
        }
        let tokenizer = parts
            .tokenizer
            .map(|bytes| {
                Tokenizer::from_bytes(bytes)
                    .ok_or_else(|| SourceUnavailable::new("invalid tokenizer bytes"))
            })
            .transpose()?;

        let policy = StatusPolicy::from_report(score_report.as_ref());
        let step_supported = !scorer.has_legacy_exct();
        let step = scorer
            .step_state(WIDENED_TOP_M)
            .expect("engine load: scorer built from a validated graph supports a step state");
        let token_rows = u32::try_from(artifacts.token_codes.len() / STAGES)
            .map_err(|_| SourceUnavailable::new("teacher token table too large"))?;
        let artifact_kappa = r4g1::artifact_kappa(parts.graph)
            .expect("engine load: the validated R4G1 artifact is addressable");
        // The graph is known addressable here (artifact_kappa succeeded), so a
        // `None` is a genuinely absent NODE section, kept as such.
        let node_section_kappa = r4g1::section_kappa(parts.graph, SectionId::NODE);

        // #836: read the optional PSTATE segment-lane descriptor at load. The
        // section is optional, so an artifact without it (every artifact today)
        // yields `None` and the deployed scorer is byte-identical to before
        // (absent-section identity). The descriptor is only honored when its
        // ring capacity fits the engine's fixed caller-owned state; a larger
        // capacity fails safe to `None` (the lane stays inert) rather than
        // silently truncating the reference semantics.
        // The learned residual table (4c) is extracted into an owned,
        // key-sorted form when the section carries rows; a config-only
        // descriptor (no rows) leaves `segment_table` None → the recurrence
        // scorer.
        let mut segment_lane = None;
        let mut segment_table = None;
        if let Ok(Some(table)) = view.pstate_table() {
            if table.lane_kind() == LANE_SEGMENT
                && (table.ring_capacity() as usize) <= SEGMENT_STATE_CAPACITY
            {
                segment_lane = Some(SegmentLaneCfg {
                    decay_shift: u32::from(table.decay_shift()),
                    base_w: table.base_w(),
                    boost: table.boost(),
                });
                let rows: SegmentTable = table
                    .rows()
                    .map(|row| {
                        (
                            row.key(),
                            row.entries().map(|e| (e.token, e.score_q.raw())).collect(),
                        )
                    })
                    .collect();
                if !rows.is_empty() {
                    segment_table = Some(rows);
                }
            }
        }

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
            segment_lane,
            segment_table,
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

    /// Encode text with the bundle-matched tokenizer when its deployed
    /// representation supports encoding. A tagged decode-only tokenizer
    /// returns `None` by construction; callers must supply its exact registered
    /// host adapter outside the engine. (The historical BPE path allocates an
    /// intermediate `String`; text helpers sit outside the allocation-free
    /// step path.)
    pub fn encode_text_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        self.tokenizer.as_ref()?.encode_into(text, out)
    }

    /// Exact registered host-adapter identity carried by a tagged,
    /// decode-only runtime tokenizer. Historical untagged tokenizer bytes
    /// predate this record and return `None`.
    pub fn tokenizer_adapter_identity(&self) -> Option<&RuntimeTokenizerIdentity> {
        self.tokenizer.as_ref()?.adapter_identity()
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
pub fn validate_quality_report(report: &serde_json::Value) -> Option<String> {
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
            return Some(format!(
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
        return None;
    }
    if let Some(graph) = graph_agreement {
        if graph < QUALITY_FLOOR_TOP1_AGREEMENT {
            return Some(format!(
                "R4G1 quality gate failed: graph runtime top-1 {:.2}% digresses below the pinned floor {:.2}%",
                graph * 100.0,
                QUALITY_FLOOR_TOP1_AGREEMENT * 100.0
            ));
        }
    }
    if let Some(bits) = graph_bits {
        if bits > QUALITY_FLOOR_BITS_PER_TOKEN {
            return Some(format!(
                "R4G1 quality gate failed: graph runtime {:.4} bits/token digresses above the pinned ceiling {:.4}",
                bits, QUALITY_FLOOR_BITS_PER_TOKEN
            ));
        }
    }
    None
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
            seed: None,
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

    // #836: the segment-adjusted argmax over the decided candidate list.
    fn active_session() -> SegmentSession<SEGMENT_STATE_CAPACITY> {
        SegmentSession::active(0, ScoreQ::from_raw(1 << 12), ScoreQ::from_raw(1 << 20))
    }

    #[test]
    fn segment_adjust_inactive_keeps_base_winner() {
        let session = SegmentSession::<SEGMENT_STATE_CAPACITY>::inactive();
        let ranked = [
            (5u32, ScoreQ::from_raw(100)),
            (9, ScoreQ::from_raw(80)),
            (2, ScoreQ::from_raw(60)),
        ];
        // Inactive → contributions zero → the base winner stands.
        assert_eq!(segment_adjusted_token(&ranked, &session, 5), 5);
        // Empty list → base token unchanged.
        assert_eq!(segment_adjusted_token(&[], &session, 7), 7);
    }

    #[test]
    fn segment_adjust_promotes_boosted_candidate() {
        let mut session = active_session();
        session.fold_prompt(&[9]); // only token 9 is in the content ring
        let ranked = [
            (5u32, ScoreQ::from_raw(100)),
            (9, ScoreQ::from_raw(80)),
            (2, ScoreQ::from_raw(60)),
        ];
        // 9's adjusted (80 + boost) overtakes 5's base 100 → 9 wins.
        assert_eq!(segment_adjusted_token(&ranked, &session, 5), 9);
    }

    #[test]
    fn segment_adjust_tie_breaks_by_lower_id() {
        let mut session = active_session();
        session.fold_prompt(&[9, 4]); // both boosted equally
        let ranked = [(9u32, ScoreQ::from_raw(50)), (4, ScoreQ::from_raw(50))];
        // Equal adjusted score → canonical tie-break keeps the lower id.
        assert_eq!(segment_adjusted_token(&ranked, &session, 9), 4);
    }

    #[test]
    fn segment_lane_attribution_records_only_real_promotions() {
        let ranked = [(5u32, ScoreQ::from_raw(100)), (9, ScoreQ::from_raw(80))];
        // Inactive lane → no attribution.
        assert!(segment_lane_attribution(
            &ranked,
            &SegmentSession::<SEGMENT_STATE_CAPACITY>::inactive(),
            5
        )
        .is_none());
        // Active but the winner is unchanged (base winner folded) → no attribution.
        let mut unchanged = active_session();
        unchanged.fold_prompt(&[5]);
        assert!(segment_lane_attribution(&ranked, &unchanged, 5).is_none());
        // Active promotion → attribution records promoted/base tokens + boost.
        let mut promo = active_session();
        promo.fold_prompt(&[9]);
        let attr = segment_lane_attribution(&ranked, &promo, 5).expect("promotion attributed");
        assert_eq!(attr.promoted_token, 9);
        assert_eq!(attr.base_token, 5);
        assert_eq!(attr.boost, 1 << 20);
    }

    #[test]
    fn inference_witness_segment_lane_serde_is_backward_compatible() {
        let witness = InferenceWitness {
            region_kappa: None,
            region_id: None,
            depth: 0,
            resolution_status: "novel".to_owned(),
            engine: "r4g1".to_owned(),
            token: 9,
            widened: false,
            segment_lane: Some(SegmentLaneWitness {
                promoted_token: 9,
                base_token: 5,
                boost: 1 << 20,
            }),
        };
        let json = serde_json::to_string(&witness).expect("serialize");
        let back: InferenceWitness = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(witness, back);
        // A pre-#836 witness (no segment_lane) still deserializes → None.
        let legacy = r#"{"region_kappa":null,"region_id":null,"depth":0,"resolution_status":"novel","engine":"r4g1","token":7,"widened":false}"#;
        let parsed: InferenceWitness = serde_json::from_str(legacy).expect("legacy deserialize");
        assert!(parsed.segment_lane.is_none());
    }

    #[test]
    fn table_score_is_an_exact_two_level_lookup() {
        let table: SegmentTable = vec![(10, vec![(1, 7), (5, 9)]), (20, vec![(3, 11)])];
        assert_eq!(table_score(&table, 10, 5), 9);
        assert_eq!(table_score(&table, 10, 1), 7);
        assert_eq!(table_score(&table, 20, 3), 11);
        assert_eq!(table_score(&table, 10, 99), 0); // candidate absent in the row
        assert_eq!(table_score(&table, 99, 1), 0); // content key absent
    }

    #[test]
    fn table_rerank_uses_learned_content_to_candidate_contributions() {
        // content token 100 → candidate 9 (+2_000_000); content 200 → candidate 2 (+5).
        let table: SegmentTable = vec![(100, vec![(9, 2_000_000)]), (200, vec![(2, 5)])];
        let ranked = [(5u32, ScoreQ::from_raw(100)), (9, ScoreQ::from_raw(80))];

        // Prompt content token 100 promotes candidate 9 (80 + 2_000_000 > 100).
        let mut session = active_session();
        session.fold_prompt(&[100]);
        assert_eq!(
            segment_adjusted_token_with_table(&ranked, &session, &table, 5),
            9
        );
        let attr = segment_lane_attribution_with_table(&ranked, &session, &table, 5)
            .expect("table promotion attributed");
        assert_eq!(attr.promoted_token, 9);
        assert_eq!(attr.base_token, 5);
        assert_eq!(attr.boost, 2_000_000);

        // A content token that maps to a non-listed candidate changes nothing.
        let mut other = active_session();
        other.fold_prompt(&[200]);
        assert_eq!(
            segment_adjusted_token_with_table(&ranked, &other, &table, 5),
            5
        );
        assert!(segment_lane_attribution_with_table(&ranked, &other, &table, 5).is_none());
    }
}
