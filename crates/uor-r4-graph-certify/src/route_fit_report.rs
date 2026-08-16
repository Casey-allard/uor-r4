//! `R4RouteAttentionV1` fit evaluation (#605): the pre-registered run
//! contract AS DATA, the progressive replacement-ladder runner
//! (null → one-head → one-layer → layer-range → whole-model, plus the
//! real-teacher/real-corpus stages carried as UNAVAILABLE), three-state-
//! plus-`NOT_RUN` stage records, and the canonical `RouteFitReport`.
//!
//! Three separations this module never blurs:
//!
//! - **Compilation success is not fit success.** That an instance builds
//!   and the packed kernel runs (the runtime checks) says nothing about
//!   the fitted codes' quality; the two are separate record fields under
//!   separate gates.
//! - **Fit success is not model quality.** A stage PASS says the fitted
//!   selection cleared the pre-registered synthetic-arm gates on this
//!   tiny teacher — a cheap-instrument result, not a model claim. The
//!   real-teacher/real-corpus stages stay UNAVAILABLE until their
//!   prerequisites exist, and the dormant lane's activation gate
//!   (`route-fit-dormant`) binds to THOSE, not to the synthetic arm.
//! - **Absence is absence.** `NOT_RUN` (a later stage after the exit
//!   rule fired) is distinct from `UNAVAILABLE` (a stage whose
//!   prerequisite is missing, with the prerequisite named) is distinct
//!   from `FAIL` (a stage that ran and missed a gate).
//!
//! ## Extension decision (#307 Gate C surfaces)
//!
//! Per-stage teacher/replaced rows EMBED the existing Gate C parity
//! metric type [`GateCMetrics`] (`crate::score`) — positions, top-1
//! agreement, bits/token, sampled-size, standard error — rather than
//! inventing a parallel parity struct; the source-parity preflight
//! embeds the #599 three-state [`ConformanceStatus`] and
//! [`ConformanceCheck`] types (`uor-r4-model-source::conformance`)
//! rather than a new check vocabulary. What #605 adds is only what did
//! not exist: the run contract, the overlap/null instrument, and the
//! four-state stage verdict.
//!
//! ## Selection evidence is the deployed kernel's
//!
//! Every fitted/null selection in this ladder comes from the REAL
//! packed path: `build_route_attention_instance` (uor-r4-graph-format)
//! over the fitted codes, stepped by `route_attention_step`
//! (uor-r4-graph-runtime) through caller-owned [`RouteState`] — never a
//! reimplementation of the selection in the harness. The certify-side
//! [`RouteAttentionReference`] runs only as a cross-check arm, and
//! [`replay_route_witness`] independently replays the packed witnesses.
//! The zero-allocation claim for the packed step is asserted by the
//! repository allocation census
//! (`crates/uor-r4-core/tests/allocation_census.rs`); this ladder
//! additionally verifies the caller-owned state's epoch discipline on
//! every step and records that the census owns the allocation claim.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use uor_r4_core::transformerless::compiler::xorshift;
use uor_r4_graph_compiler::route_fit::{
    FitManifest, FittedRouteCodes, HeadCodes, RouteTraceCorpus, StoryRestrictionPlan,
    SyntheticRouteTeacher,
};
use uor_r4_graph_format::route_attention::{
    build_route_attention_instance, RouteAttentionView, RouteOpCensus, ROUTE_CODE_BYTES,
    ROUTE_MAX_CANDIDATES,
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::route_attention::{route_attention_step, RouteState};
use uor_r4_model_source::conformance::{ConformanceCheck, ConformanceStatus};
use uor_r4_model_source::SourceUnavailable;

use crate::route_attention::{
    expected_route_census, replay_route_witness, run_packed, RouteAttentionReference,
};
use crate::score::GateCMetrics;

/// Schema tag of the canonical route-fit report.
pub const ROUTE_FIT_REPORT_SCHEMA: &str = "uor-r4-route-fit-report/1";
/// Format tag of the pre-registered run contract.
pub const ROUTE_FIT_CONTRACT_FORMAT: &str = "uor-r4-route-fit-contract/1";
/// Replacement-semantics id (recorded in the contract): the teacher's
/// own attention weights restricted to the fitted-selected top-M
/// support and renormalized — isolating the selection mechanism.
pub const REPLACEMENT_SEMANTICS_ID: &str = "support-restrict-renormalize/1";
/// N1 null id: seeded-random route codes of the same shapes.
pub const NULL_N1_ID: &str = "seeded-random-route-codes-same-shapes/1";
/// N2 null id: the fitted selections scored against a derangement of
/// the query→support mapping (supports shifted by one query position,
/// cyclically within each sequence).
pub const NULL_N2_ID: &str = "deranged-supports-shift-one-cyclic-within-sequence/1";
/// Integer seed of the N1 random-code stream (`compiler::xorshift`).
pub const NULL_N1_SEED: u64 = 0x6050_1001;

/// Stage names, ladder order.
pub const STAGE_NULL: &str = "null";
pub const STAGE_ONE_HEAD: &str = "one-head";
pub const STAGE_ONE_LAYER: &str = "one-layer";
pub const STAGE_LAYER_RANGE: &str = "layer-range";
pub const STAGE_WHOLE_MODEL: &str = "whole-model";
pub const STAGE_REAL_TEACHER: &str = "real-teacher";
pub const STAGE_REAL_CORPUS: &str = "real-corpus";

/// Stage kinds.
pub const STAGE_KIND_SYNTHETIC: &str = "synthetic";
pub const STAGE_KIND_REAL_TEACHER: &str = "real-teacher";
pub const STAGE_KIND_REAL_CORPUS: &str = "real-corpus";

/// One replaced attention head.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacedHead {
    /// Source layer index.
    #[serde(default)]
    pub layer: u32,
    /// Head index within the layer.
    #[serde(default)]
    pub head: u32,
}

/// One pre-declared ladder stage: name, kind, and the exact replaced
/// scope as data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageScope {
    /// Stage name ([`STAGE_NULL`] .. [`STAGE_REAL_CORPUS`]).
    #[serde(default)]
    pub stage: String,
    /// Stage kind ([`STAGE_KIND_SYNTHETIC`] / real-teacher / real-corpus).
    #[serde(default)]
    pub kind: String,
    /// The replaced heads (empty for the null and real stages).
    #[serde(default)]
    pub replaced: Vec<ReplacedHead>,
    /// Human note pinning the scope choice.
    #[serde(default)]
    pub note: String,
}

/// The pre-registered advance-gate margins, as data. These numbers were
/// posted to issue #605 BEFORE any fit ran and must not be tuned after
/// seeing results; a test-local contract may inject different margins
/// only to prove the exit machinery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GateThresholds {
    /// Fitted support overlap must be at least this factor times the
    /// best (largest) null overlap.
    pub overlap_null_factor: f64,
    /// ... and at least this absolute floor.
    pub overlap_floor: f64,
    /// Teacher-forced top-1 agreement floor.
    pub min_top1_agreement: f64,
    /// Replaced bits/token must not exceed this factor times the
    /// teacher's bits/token.
    pub max_bits_per_token_ratio: f64,
    /// Anti-vacuity: N2 overlap must stay BELOW this fraction of the
    /// fitted overlap at every evaluated scope, else the instrument is
    /// VACUOUS and the whole run is invalid.
    pub n2_vacuity_fraction: f64,
}

impl Default for GateThresholds {
    fn default() -> Self {
        // The pre-registered #605 margins.
        Self {
            overlap_null_factor: 2.0,
            overlap_floor: 0.5,
            min_top1_agreement: 0.90,
            max_bits_per_token_ratio: 1.10,
            n2_vacuity_fraction: 0.5,
        }
    }
}

/// The pre-registered null definitions, as data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NullDefinitions {
    /// [`NULL_N1_ID`].
    pub n1: String,
    /// [`NULL_N2_ID`].
    pub n2: String,
    /// Seed of the N1 code stream.
    pub n1_seed: u64,
}

/// The whole pre-registered run contract, serialized INTO the report so
/// the gates a run was judged against travel with its numbers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RunContract {
    /// [`ROUTE_FIT_CONTRACT_FORMAT`].
    pub format: String,
    /// The pre-registered metric statement.
    pub metric: String,
    /// Top-k depth of the top-k agreement row.
    pub top_k: u32,
    /// Null definitions.
    pub nulls: NullDefinitions,
    /// Advance-gate margins.
    pub gates: GateThresholds,
    /// Replacement-semantics id ([`REPLACEMENT_SEMANTICS_ID`]).
    pub replacement_semantics: String,
    /// The replacement rule spelled out, selection provenance included.
    pub replacement_rule: String,
    /// The declared degenerate-renormalization rule.
    pub degenerate_renormalization: String,
    /// Which steps enter the overlap instrument.
    pub overlap_eligibility: String,
    /// How per-step overlaps aggregate into a stage number.
    pub overlap_aggregation: String,
    /// The declared bits/token probability floor (f64 of
    /// `f32::MIN_POSITIVE`) guarding against non-finite report bytes.
    pub bits_probability_floor: f64,
    /// The pre-registered exit rule.
    pub exit_rule: String,
    /// The pre-registered UNAVAILABLE policy.
    pub unavailable_policy: String,
    /// What the null stage checks (it replaces nothing).
    pub null_stage_rule: String,
    /// The anti-vacuity rule.
    pub anti_vacuity_rule: String,
    /// Predeclared next action on a positive synthetic result.
    pub decision_positive: String,
    /// Predeclared next action on a negative synthetic result.
    pub decision_negative: String,
    /// The pre-declared stages, ladder order.
    pub stages: Vec<StageScope>,
}

/// The pre-registered #605 contract (posted to the issue before any fit
/// ran). The margins and null definitions here are BINDING: they are
/// never tuned after seeing results.
pub fn preregistered_route_fit_contract() -> RunContract {
    let head = |layer: u32, head: u32| ReplacedHead { layer, head };
    RunContract {
        format: ROUTE_FIT_CONTRACT_FORMAT.to_owned(),
        metric: "teacher-forced top-1/top-k agreement and bits/token at each replaced \
                 scope; support overlap (Jaccard over the teacher's top-k support vs the \
                 fitted-selected top-M) as diagnostic"
            .to_owned(),
        top_k: 8,
        nulls: NullDefinitions {
            n1: NULL_N1_ID.to_owned(),
            n2: NULL_N2_ID.to_owned(),
            n1_seed: NULL_N1_SEED,
        },
        gates: GateThresholds::default(),
        replacement_semantics: REPLACEMENT_SEMANTICS_ID.to_owned(),
        replacement_rule: "the teacher's own attention weights at each replaced head are \
                           restricted to the fitted-selected top-M support and renormalized \
                           (isolating the selection mechanism); the SELECTION comes from the \
                           deployed packed kernel — build_route_attention_instance over the \
                           fitted key codes of the causal prefix, stepped by \
                           route_attention_step — never a harness reimplementation; the \
                           certify-side RouteAttentionReference runs only as a cross-check arm"
            .to_owned(),
        degenerate_renormalization: "uniform-over-selected-when-restricted-mass-not-positive"
            .to_owned(),
        overlap_eligibility: "steps-with-more-candidates-than-the-declared-selection-width"
            .to_owned(),
        overlap_aggregation: "mean-jaccard-over-(replaced-head,eligible-step)-pairs-fixed-order"
            .to_owned(),
        bits_probability_floor: f32::MIN_POSITIVE as f64,
        exit_rule: "the ladder stops at the FIRST stage failing any gate; every later \
                    stage carries verdict NOT_RUN (absent-as-absent, distinct from \
                    UNAVAILABLE); the negative is preserved, never rerun into a pass"
            .to_owned(),
        unavailable_policy: "a stage whose prerequisite is absent is UNAVAILABLE with the \
                             missing prerequisite named; it never passes vacuously, is never \
                             silently skipped, and does not mark later stages NOT_RUN"
            .to_owned(),
        null_stage_rule: "the null stage replaces nothing: it validates the instrument \
                          (packed-kernel runtime checks, N1/N2 nulls and anti-vacuity at the \
                          one-head reference scope) and the harness identity (replaced \
                          forward with an empty scope must reproduce the teacher); the \
                          overlap threshold gate binds stages replacing at least one head"
            .to_owned(),
        anti_vacuity_rule: "N2 overlap must be < n2_vacuity_fraction x fitted overlap at \
                            every evaluated scope, else the instrument is VACUOUS \
                            (instrument_valid = false) and the run is invalid regardless of \
                            other numbers"
            .to_owned(),
        decision_positive: "keep the operator dormant; the synthetic cheap instrument \
                            passing does NOT clear route-fit-dormant's activation gate — \
                            the pre-declared next action is to rerun this ladder on the \
                            pinned real teacher with the #531 saturation corpus and judge \
                            ONLY that result against the gate"
            .to_owned(),
        decision_negative: "retain the fitted artifacts, this report, and the operator \
                            (never promote, never delete); do not tune route-fit/1 against \
                            this instrument — a method change arrives as route-fit/2 with a \
                            fresh pre-registered contract"
            .to_owned(),
        stages: vec![
            StageScope {
                stage: STAGE_NULL.to_owned(),
                kind: STAGE_KIND_SYNTHETIC.to_owned(),
                replaced: Vec::new(),
                note: "identity replacement: instrument + harness validation".to_owned(),
            },
            StageScope {
                stage: STAGE_ONE_HEAD.to_owned(),
                kind: STAGE_KIND_SYNTHETIC.to_owned(),
                replaced: vec![head(0, 0)],
                note: "the pre-declared reference head (layer 0, head 0)".to_owned(),
            },
            StageScope {
                stage: STAGE_ONE_LAYER.to_owned(),
                kind: STAGE_KIND_SYNTHETIC.to_owned(),
                replaced: vec![head(0, 0), head(0, 1)],
                note: "every head of layer 0".to_owned(),
            },
            StageScope {
                stage: STAGE_LAYER_RANGE.to_owned(),
                kind: STAGE_KIND_SYNTHETIC.to_owned(),
                replaced: vec![head(1, 0), head(1, 1)],
                note: "the contiguous layer range 1..=1 (the final layer; with a 2-layer \
                       teacher this is the maximal proper range and a distinct scope from \
                       one-layer)"
                    .to_owned(),
            },
            StageScope {
                stage: STAGE_WHOLE_MODEL.to_owned(),
                kind: STAGE_KIND_SYNTHETIC.to_owned(),
                replaced: vec![head(0, 0), head(0, 1), head(1, 0), head(1, 1)],
                note: "every attention head of the model".to_owned(),
            },
            StageScope {
                stage: STAGE_REAL_TEACHER.to_owned(),
                kind: STAGE_KIND_REAL_TEACHER.to_owned(),
                replaced: Vec::new(),
                note: "pinned SmolLM2 teacher arm".to_owned(),
            },
            StageScope {
                stage: STAGE_REAL_CORPUS.to_owned(),
                kind: STAGE_KIND_REAL_CORPUS.to_owned(),
                replaced: Vec::new(),
                note: "#531 saturation-corpus arm".to_owned(),
            },
        ],
    }
}

/// Four-state stage verdict. `NOT_RUN` is absent-as-absent: the stage
/// was never evaluated because the exit rule fired earlier — distinct
/// from `UNAVAILABLE` (evaluated for its prerequisite, which is
/// missing) and from `FAIL` (evaluated and missed a gate).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageVerdict {
    /// Every gate of the stage held.
    #[serde(rename = "PASS")]
    Pass,
    /// The stage ran and missed at least one gate.
    #[serde(rename = "FAIL")]
    Fail,
    /// A prerequisite is missing; named in the reason.
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
    /// Never evaluated: the ladder exited at an earlier stage. The
    /// default verdict is deliberately this one — a defaulted record is
    /// an unevaluated record, never a vacuous PASS.
    #[default]
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

/// Deployed-kernel runtime checks of one stage's scope: witness replay,
/// closed-form census, reference cross-check, and the caller-owned
/// state's epoch discipline. The zero-allocation claim for the packed
/// step is owned by the repository allocation census (named in
/// `allocation_note`), not re-measured here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeChecks {
    /// Packed steps driven for this scope.
    pub steps: u64,
    /// Every step's witness replayed via `replay_route_witness`.
    pub witness_replay_pass: bool,
    /// First replay failure, when any.
    pub witness_replay_detail: String,
    /// Every step's census equals its closed form.
    pub census_closed_form_pass: bool,
    /// The certify-side reference selections agree with the packed
    /// selections bit-for-bit (cross-check arm).
    pub reference_crosscheck_pass: bool,
    /// The caller-owned `RouteState` epoch advanced once per step and
    /// the selection width matched the declared `top_m` on every step.
    pub state_epoch_pass: bool,
    /// Where the zero-allocation claim is asserted.
    pub allocation_note: String,
    /// Conjunction of the checks above.
    pub pass: bool,
}

/// Source-parity preflight of the run: the #603 trace corpus replayed
/// against the teacher executor — argmax/top-8, target logprob, q/k
/// lane bits, and attention supports must reproduce. Embeds the #599
/// three-state check types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PreflightRecord {
    /// Overall preflight verdict.
    pub status: ConformanceStatus,
    /// The named checks (#599 `ConformanceCheck` rows).
    pub checks: Vec<ConformanceCheck>,
}

impl Default for PreflightRecord {
    fn default() -> Self {
        // A defaulted preflight is one that never ran: UNAVAILABLE, not
        // a vacuous PASS.
        Self {
            status: ConformanceStatus::Unavailable,
            checks: Vec::new(),
        }
    }
}

/// The overlap instrument of one stage: fitted vs the two nulls over
/// the eligible steps of the stage's scope.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OverlapRecord {
    /// Mean fitted Jaccard overlap.
    pub fitted: f64,
    /// Mean N1 (seeded-random codes) overlap.
    pub n1: f64,
    /// Mean N2 (deranged supports) overlap.
    pub n2: f64,
    /// max(n1, n2).
    pub best_null: f64,
    /// Eligible (head, step) pairs.
    pub eligible_steps: u64,
    /// Whether this scope is vacuous under the anti-vacuity rule.
    pub vacuous: bool,
}

/// One ladder stage's record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StageRecord {
    /// Stage name.
    pub stage: String,
    /// Stage kind.
    pub kind: String,
    /// The replaced scope (as evaluated).
    pub replaced: Vec<ReplacedHead>,
    /// κ of the fit manifest whose parameters this stage evaluated
    /// (absent for stages that evaluated nothing).
    pub fit_manifest_kappa: Option<String>,
    /// Source-parity preflight (run-level, carried per evaluated stage).
    pub preflight: Option<PreflightRecord>,
    /// Deployed-kernel runtime checks over this stage's scope.
    pub runtime: Option<RuntimeChecks>,
    /// The overlap instrument at this stage's scope (the null stage
    /// carries the one-head reference scope's instrument).
    pub overlap: Option<OverlapRecord>,
    /// Teacher parity row (embedded Gate C metric type).
    pub teacher: Option<GateCMetrics>,
    /// Replaced-model parity row (embedded Gate C metric type).
    pub replaced_metrics: Option<GateCMetrics>,
    /// P(recorded teacher argmax within the replaced model's top-k).
    pub top_k_agreement: Option<f64>,
    /// replaced bits/token divided by teacher bits/token.
    pub bits_per_token_ratio: Option<f64>,
    /// The verdict.
    pub verdict: StageVerdict,
    /// Why (names the failed gate, the missing prerequisite, or the
    /// exit-rule stop).
    pub reason: String,
}

/// The predeclared decision record: both next actions were fixed in the
/// contract before the run; `outcome` states what was measured.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DecisionRecord {
    /// Predeclared positive next action (copied from the contract).
    pub positive_next: String,
    /// Predeclared negative next action (copied from the contract).
    pub negative_next: String,
    /// The measured outcome of THIS run, in measurement language.
    pub outcome: String,
}

/// The canonical #605 route-fit report.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RouteFitReport {
    /// [`ROUTE_FIT_REPORT_SCHEMA`].
    pub schema: String,
    /// The pre-registered contract this run was judged against.
    pub contract: RunContract,
    /// κ of the fit manifest.
    pub fit_manifest_kappa: String,
    /// κ of the fitted parameters.
    pub fitted_params_kappa: String,
    /// Anti-vacuity verdict of the whole run: false when ANY evaluated
    /// scope was vacuous — a vacuous instrument fails the whole run
    /// regardless of other numbers.
    pub instrument_valid: bool,
    /// Per-stage records, ladder order.
    pub stages: Vec<StageRecord>,
    /// The decision record.
    pub decision: DecisionRecord,
}

/// Canonical report bytes: ciborium, struct-declaration field order —
/// the certify crate's existing serde byte format (the
/// `Certificate::to_cbor` / #604 witness convention). Every float in
/// the report is finite by construction (probabilities are floored at
/// the declared `bits_probability_floor`), so serialization cannot
/// fail. Running the same ladder twice produces byte-identical
/// canonical bytes.
pub fn canonical_route_fit_report_bytes(report: &RouteFitReport) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(report, &mut bytes)
        .expect("route-fit report serializes to canonical bytes");
    bytes
}

/// The report κ: `blake3:<hex>` over the canonical report bytes.
pub fn route_fit_report_kappa(report: &RouteFitReport) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_route_fit_report_bytes(report)).to_hex()
    )
}

/// Where the real-arm prerequisites would live, probed (never assumed)
/// by the real stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealArmProbe {
    /// The pinned SmolLM2 snapshot directory.
    pub smollm2_snapshot_dir: PathBuf,
    /// The #531 saturation-corpus artifact, when a path convention for
    /// it exists (none does yet).
    pub saturation_corpus: Option<PathBuf>,
}

impl RealArmProbe {
    /// The repository conventions: `SMOLLM2_SOURCE` or the default
    /// snapshot directory; no #531 corpus path convention exists yet.
    pub fn from_env() -> Self {
        Self {
            smollm2_snapshot_dir: PathBuf::from(
                std::env::var("SMOLLM2_SOURCE")
                    .unwrap_or_else(|_| ".uor-models/sources/smollm2-135m-instruct".to_owned()),
            ),
            saturation_corpus: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Deployed-kernel selection evidence.
// ---------------------------------------------------------------------------

/// Per-head selection evidence: for every `(story, pos)`, the positions
/// the DEPLOYED kernel selected, plus the accumulated runtime-check
/// counters.
struct HeadEvidence {
    /// `[story][pos]` → selected positions, selection order.
    selected: Vec<Vec<Vec<u32>>>,
    steps: u64,
    replay_failures: u64,
    first_replay_detail: String,
    census_mismatches: u64,
    crosscheck_failures: u64,
    epoch_failures: u64,
}

const FULL_MASK: [u8; ROUTE_CODE_BYTES] = [0xff; ROUTE_CODE_BYTES];

/// Drive the deployed packed kernel over one head's fitted codes: per
/// step, the candidate table is the causal prefix of key codes
/// (candidate index == position), the query is the step's query code,
/// and `top_m` is clamped to the candidate count as the instance bound
/// requires. Also replays every step's witness independently, checks
/// the closed-form census, cross-checks the certify reference, and
/// verifies the caller-owned state's epoch discipline.
fn packed_selection_evidence(
    codes: &HeadCodes,
    top_m: u32,
    with_checks: bool,
) -> Result<HeadEvidence, SourceUnavailable> {
    let zeros = vec![ScoreQ::ZERO; ROUTE_MAX_CANDIDATES];
    let mut evidence = HeadEvidence {
        selected: Vec::with_capacity(codes.key_codes.len()),
        steps: 0,
        replay_failures: 0,
        first_replay_detail: String::new(),
        census_mismatches: 0,
        crosscheck_failures: 0,
        epoch_failures: 0,
    };
    // ONE caller-owned state reused across every step of the head — the
    // deployed usage pattern; its epoch must advance once per step.
    let mut state = RouteState::new();
    let mut expected_epoch = 0u64;
    for (story_keys, story_queries) in codes.key_codes.iter().zip(codes.query_codes.iter()) {
        let mut story_selected = Vec::with_capacity(story_keys.len());
        for (pos, query) in story_queries.iter().enumerate() {
            let candidates = &story_keys[..=pos];
            let n = candidates.len();
            let m = top_m.min(n as u32);
            let instance = build_route_attention_instance(&FULL_MASK, candidates, &zeros[..n], m)
                .map_err(|error| {
                SourceUnavailable::new(format!(
                    "route instance construction failed at pos {pos}: {error}"
                ))
            })?;
            let view = RouteAttentionView::parse(&instance).map_err(|error| {
                SourceUnavailable::new(format!("route instance parse failed: {error}"))
            })?;
            let mut census = RouteOpCensus::default();
            route_attention_step(&view, query, &mut state, &mut census).map_err(|error| {
                SourceUnavailable::new(format!("packed route step failed at pos {pos}: {error}"))
            })?;
            expected_epoch += 1;
            if state.epoch() != expected_epoch || state.selected_len() != m as usize {
                evidence.epoch_failures += 1;
            }
            let mut selected = Vec::with_capacity(m as usize);
            let mut slot = 0usize;
            while let Some((candidate, _distance)) = state.selected(slot) {
                selected.push(candidate);
                slot += 1;
            }
            evidence.steps += 1;
            if with_checks {
                // Witness path (same kernel, fresh state inside
                // run_packed) + independent replay + closed-form census.
                let (records, witness) = run_packed(&instance, &[*query]).map_err(|error| {
                    SourceUnavailable::new(format!("packed witness run failed: {error}"))
                })?;
                let witness_selected: Vec<u32> = records[0]
                    .selected
                    .iter()
                    .map(|selection| selection.candidate)
                    .collect();
                if witness_selected != selected {
                    evidence.crosscheck_failures += 1;
                }
                if let Some(error) = replay_route_witness(&instance, &[*query], &witness) {
                    evidence.replay_failures += 1;
                    if evidence.first_replay_detail.is_empty() {
                        evidence.first_replay_detail = error.to_string();
                    }
                }
                if witness.census != expected_route_census(n as u32, m as u16, 1) {
                    evidence.census_mismatches += 1;
                }
                // Cross-check arm: the certify-side reference must agree
                // with the deployed kernel bit-for-bit on the selection.
                let reference =
                    RouteAttentionReference::from_instance_bytes(&instance).map_err(|error| {
                        SourceUnavailable::new(format!("reference construction failed: {error}"))
                    })?;
                let mut reference_census = RouteOpCensus::default();
                let reference_record = reference.reference_step(query, &mut reference_census);
                let reference_selected: Vec<u32> = reference_record
                    .selected
                    .iter()
                    .map(|selection| selection.candidate)
                    .collect();
                if reference_selected != selected {
                    evidence.crosscheck_failures += 1;
                }
            }
            story_selected.push(selected);
        }
        evidence.selected.push(story_selected);
    }
    Ok(evidence)
}

/// N1 null codes: seeded-random route codes of the same shapes,
/// generated in fixed order (head order as fitted; story ascending;
/// position ascending; query code then key code; five xorshift draws
/// per code).
fn null_n1_codes(fitted: &FittedRouteCodes, seed: u64) -> Vec<HeadCodes> {
    let mut stream = seed;
    let random_code = |stream: &mut u64| -> [u8; ROUTE_CODE_BYTES] {
        let mut code = [0u8; ROUTE_CODE_BYTES];
        for chunk in code.chunks_mut(8) {
            let draw = xorshift(stream).to_le_bytes();
            chunk.copy_from_slice(&draw[..chunk.len()]);
        }
        code
    };
    fitted
        .heads
        .iter()
        .map(|head| {
            let mut query_codes = Vec::with_capacity(head.query_codes.len());
            let mut key_codes = Vec::with_capacity(head.key_codes.len());
            for (story_queries, story_keys) in head.query_codes.iter().zip(head.key_codes.iter()) {
                let mut null_queries = Vec::with_capacity(story_queries.len());
                let mut null_keys = Vec::with_capacity(story_keys.len());
                for _ in 0..story_queries.len() {
                    null_queries.push(random_code(&mut stream));
                    null_keys.push(random_code(&mut stream));
                }
                query_codes.push(null_queries);
                key_codes.push(null_keys);
            }
            HeadCodes {
                layer: head.layer,
                head: head.head,
                thresholds: Vec::new(),
                query_codes,
                key_codes,
            }
        })
        .collect()
}

fn jaccard(a: &[u32], b: &[u32]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let mut intersection = 0usize;
    for value in a {
        if b.contains(value) {
            intersection += 1;
        }
    }
    let union = a.len() + b.len() - intersection;
    intersection as f64 / union as f64
}

/// Per-head overlap sums over the eligible steps: fitted, N1, N2.
#[derive(Clone)]
struct HeadOverlaps {
    fitted_sum: f64,
    n1_sum: f64,
    n2_sum: f64,
    eligible: u64,
}

fn head_overlaps(
    corpus: &RouteTraceCorpus,
    lane_index: usize,
    head: u32,
    fitted_selected: &[Vec<Vec<u32>>],
    n1_selected: &[Vec<Vec<u32>>],
    top_m: u32,
) -> HeadOverlaps {
    let mut sums = HeadOverlaps {
        fitted_sum: 0.0,
        n1_sum: 0.0,
        n2_sum: 0.0,
        eligible: 0,
    };
    for (story_index, story) in corpus.stories.iter().enumerate() {
        let story_len = story.steps.len();
        for (pos, step) in story.steps.iter().enumerate() {
            // Eligibility: strictly more candidates than the declared
            // selection width — a forced (all-candidate) selection
            // carries no selection information.
            if (pos + 1) as u32 <= top_m {
                continue;
            }
            let teacher_support: Vec<u32> = step.supports[lane_index][head as usize]
                .iter()
                .map(|&(position, _weight)| position)
                .collect();
            // N2: the query→support mapping deranged — the support of
            // the NEXT query position, cyclically within the sequence.
            let deranged_pos = (pos + 1) % story_len;
            let deranged_support: Vec<u32> = story.steps[deranged_pos].supports[lane_index]
                [head as usize]
                .iter()
                .map(|&(position, _weight)| position)
                .collect();
            let fitted_set = &fitted_selected[story_index][pos];
            let n1_set = &n1_selected[story_index][pos];
            sums.fitted_sum += jaccard(fitted_set, &teacher_support);
            sums.n1_sum += jaccard(n1_set, &teacher_support);
            sums.n2_sum += jaccard(fitted_set, &deranged_support);
            sums.eligible += 1;
        }
    }
    sums
}

// ---------------------------------------------------------------------------
// Teacher-forced evaluation.
// ---------------------------------------------------------------------------

/// Max-subtracted softmax in the corpus generator's arithmetic order:
/// first-maximum scan, sequential exp/sum fold, one divide per element.
fn normalized_probabilities(logits: &[f32]) -> Vec<f32> {
    let mut max = f32::NEG_INFINITY;
    for &logit in logits {
        if logit > max {
            max = logit;
        }
    }
    let mut probabilities = Vec::with_capacity(logits.len());
    let mut sum = 0.0f32;
    for &logit in logits {
        let value = (logit - max).exp();
        sum += value;
        probabilities.push(value);
    }
    for probability in probabilities.iter_mut() {
        *probability /= sum;
    }
    probabilities
}

/// Top-`N` token ids by descending probability with the corpus
/// generator's stable insertion tie rule (an equal-probability later
/// token never displaces an earlier one).
fn top_tokens(probabilities: &[f32], depth: usize) -> Vec<u32> {
    let mut top: Vec<(u32, f32)> = Vec::with_capacity(depth);
    for (token, &probability) in probabilities.iter().enumerate() {
        if top.len() < depth {
            top.push((token as u32, probability));
            let mut index = top.len() - 1;
            while index > 0 && top[index].1 > top[index - 1].1 {
                top.swap(index, index - 1);
                index -= 1;
            }
        } else if probability > top[depth - 1].1 {
            top[depth - 1] = (token as u32, probability);
            let mut index = depth - 1;
            while index > 0 && top[index].1 > top[index - 1].1 {
                top.swap(index, index - 1);
                index -= 1;
            }
        }
    }
    top.into_iter().map(|(token, _)| token).collect()
}

fn binomial_standard_error(p: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    (p * (1.0 - p) / n as f64).sqrt()
}

/// Teacher/replaced parity numbers of one evaluated scope. `pub(crate)`
/// (rather than a new struct) so #643's A/B harness
/// (`crate::msa_ab_harness`) can reuse the SAME metric computation this
/// module's ladder uses, never a reimplementation.
pub(crate) struct ScopeMetrics {
    pub(crate) teacher: GateCMetrics,
    pub(crate) replaced: GateCMetrics,
    #[allow(dead_code)]
    pub(crate) top_k_agreement: f64,
    pub(crate) bits_ratio: f64,
}

/// Teacher-forced evaluation of one replaced scope: run the replaced
/// forward (`support-restrict-renormalize/1`) over every story and
/// measure top-1/top-k agreement against the RECORDED teacher argmax
/// and base-2 cross-entropy against the recorded targets. The teacher
/// row's bits come from the production `.prob` sidecar.
pub(crate) fn scope_metrics(
    teacher: &mut SyntheticRouteTeacher,
    corpus: &RouteTraceCorpus,
    scope: &[ReplacedHead],
    selections: &BTreeMap<(u32, u32), Vec<Vec<Vec<u32>>>>,
    contract: &RunContract,
) -> ScopeMetrics {
    let floor = contract.bits_probability_floor;
    let mut positions = 0usize;
    let mut top1_matches = 0usize;
    let mut topk_matches = 0usize;
    let mut replaced_bits_sum = 0.0f64;
    let mut teacher_bits_sum = 0.0f64;
    for (story_index, story) in corpus.stories.iter().enumerate() {
        let mut plan = StoryRestrictionPlan::new();
        for head in scope {
            let per_story = &selections[&(head.layer, head.head)];
            plan.insert((head.layer, head.head), per_story[story_index].clone());
        }
        let logits_per_position = teacher.teacher_forced_logits(&story.tokens, &plan);
        for (step, logits) in story.steps.iter().zip(logits_per_position.iter()) {
            let probabilities = normalized_probabilities(logits);
            let teacher_argmax = step.top_tokens[0];
            let replaced_top = top_tokens(&probabilities, contract.top_k as usize);
            positions += 1;
            if replaced_top.first() == Some(&teacher_argmax) {
                top1_matches += 1;
            }
            if replaced_top.contains(&teacher_argmax) {
                topk_matches += 1;
            }
            let target_probability = f64::from(probabilities[step.next as usize]).max(floor);
            replaced_bits_sum += -target_probability.ln() / std::f64::consts::LN_2;
            teacher_bits_sum += -f64::from(step.target_logprob_nats) / std::f64::consts::LN_2;
        }
    }
    let denominator = positions.max(1) as f64;
    let top1 = top1_matches as f64 / denominator;
    let topk = topk_matches as f64 / denominator;
    let teacher_bits = teacher_bits_sum / denominator;
    let replaced_bits = replaced_bits_sum / denominator;
    ScopeMetrics {
        teacher: GateCMetrics {
            positions,
            top1_agreement: 1.0,
            bits_per_token: teacher_bits,
            positions_sampled: 0,
            standard_error: 0.0,
        },
        replaced: GateCMetrics {
            positions,
            top1_agreement: top1,
            bits_per_token: replaced_bits,
            positions_sampled: 0,
            standard_error: binomial_standard_error(top1, positions),
        },
        top_k_agreement: topk,
        bits_ratio: if teacher_bits > 0.0 {
            replaced_bits / teacher_bits
        } else {
            f64::MAX
        },
    }
}

// ---------------------------------------------------------------------------
// Source-parity preflight.
// ---------------------------------------------------------------------------

/// Replay the corpus against the teacher executor: recorded top-8
/// tokens, target logprobs, q/k lane bits, and attention supports must
/// reproduce. This guards the harness alignment (story reconstruction,
/// row decoding) before any replaced number is interpreted.
fn source_parity_preflight(
    teacher: &mut SyntheticRouteTeacher,
    corpus: &RouteTraceCorpus,
) -> PreflightRecord {
    use uor_r4_model_source::{
        BehaviorSource, TeacherOracle, TraceCaptureRequest, TraceCaptureSinks,
    };
    let declared: Vec<usize> = corpus
        .declared_layers
        .iter()
        .map(|&layer| layer as usize)
        .collect();
    let mut top8_mismatches = 0u64;
    let mut logprob_delta = 0.0f64;
    let mut qk_delta = 0.0f64;
    let mut support_mismatches = 0u64;
    for story in &corpus.stories {
        teacher.reset();
        let mut logits = vec![0.0f32; TeacherOracle::vocab(teacher)];
        for (pos, step) in story.steps.iter().enumerate() {
            let mut q_rows: Vec<Vec<f32>> = Vec::new();
            let mut k_rows: Vec<Vec<f32>> = Vec::new();
            let mut supports: Vec<Vec<(u32, f32)>> = Vec::new();
            {
                let mut residual_sink = |_layer: usize, _x: &[f32]| {};
                let mut qkv_sink = |_layer: usize, q: &[f32], k: &[f32], _v: &[f32]| {
                    q_rows.push(q.to_vec());
                    k_rows.push(k.to_vec());
                };
                let mut attention_sink = |_layer: usize, _head: usize, att: &[f32]| {
                    // Replicate the #603 sink's bounded top-S rule:
                    // descending weight, ties to the lower position.
                    let mut order: Vec<u32> = (0..att.len() as u32).collect();
                    order.sort_by(|a, b| {
                        att[*b as usize]
                            .total_cmp(&att[*a as usize])
                            .then_with(|| a.cmp(b))
                    });
                    supports.push(
                        order
                            .iter()
                            .take(corpus.support_size as usize)
                            .map(|&position| (position, att[position as usize]))
                            .collect(),
                    );
                };
                teacher.step_with_trace_capture(
                    step.input_token as usize,
                    pos,
                    &mut logits,
                    &TraceCaptureRequest {
                        residual_layers: &[],
                        qkv_layers: &declared,
                        attention_layers: &declared,
                    },
                    &mut TraceCaptureSinks {
                        residual: &mut residual_sink,
                        qkv: &mut qkv_sink,
                        attention: &mut attention_sink,
                    },
                );
            }
            let probabilities = normalized_probabilities(&logits);
            let recomputed_top8 = top_tokens(&probabilities, 8);
            if recomputed_top8 != step.top_tokens.to_vec() {
                top8_mismatches += 1;
            }
            let target_probability = f64::from(probabilities[step.next as usize]);
            let recomputed_nats = if target_probability > 0.0 {
                target_probability.ln()
            } else {
                f64::NEG_INFINITY
            };
            let delta = (recomputed_nats - f64::from(step.target_logprob_nats)).abs();
            if delta > logprob_delta {
                logprob_delta = delta;
            }
            for (lane_index, _) in declared.iter().enumerate() {
                for (recomputed, recorded) in [
                    (&q_rows[lane_index], &step.q_rows[lane_index]),
                    (&k_rows[lane_index], &step.k_rows[lane_index]),
                ] {
                    for (a, b) in recomputed.iter().zip(recorded.iter()) {
                        let delta = (f64::from(*a) - f64::from(*b)).abs();
                        if delta > qk_delta {
                            qk_delta = delta;
                        }
                    }
                }
                let heads = corpus.geometry.heads;
                for head in 0..heads {
                    let recomputed = &supports[lane_index * heads + head];
                    let recorded = &step.supports[lane_index][head];
                    let same = recomputed.len() == recorded.len()
                        && recomputed
                            .iter()
                            .zip(recorded.iter())
                            .all(|(&(pa, wa), &(pb, wb))| pa == pb && wa.to_bits() == wb.to_bits());
                    if !same {
                        support_mismatches += 1;
                    }
                }
            }
        }
    }
    let pass_fail = |ok: bool| {
        if ok {
            ConformanceStatus::Pass
        } else {
            ConformanceStatus::Fail
        }
    };
    let logprob_tolerance = 1e-5f64;
    let checks = vec![
        ConformanceCheck {
            name: "preflight/top8-tokens".to_owned(),
            status: pass_fail(top8_mismatches == 0),
            tolerance: None,
            delta: None,
            detail: Some(format!(
                "{top8_mismatches} of {} positions disagree with the recorded top-8",
                corpus.records
            )),
        },
        ConformanceCheck {
            name: "preflight/target-logprob".to_owned(),
            status: pass_fail(logprob_delta.is_finite() && logprob_delta <= logprob_tolerance),
            tolerance: Some(logprob_tolerance),
            delta: Some(logprob_delta),
            detail: None,
        },
        ConformanceCheck {
            name: "preflight/trace-qk-replay".to_owned(),
            status: pass_fail(qk_delta == 0.0),
            tolerance: Some(0.0),
            delta: Some(qk_delta),
            detail: None,
        },
        ConformanceCheck {
            name: "preflight/attention-support-replay".to_owned(),
            status: pass_fail(support_mismatches == 0),
            tolerance: None,
            delta: None,
            detail: Some(format!(
                "{support_mismatches} (layer, head, step) support rows disagree"
            )),
        },
    ];
    let status = if checks
        .iter()
        .all(|check| check.status == ConformanceStatus::Pass)
    {
        ConformanceStatus::Pass
    } else {
        ConformanceStatus::Fail
    };
    PreflightRecord { status, checks }
}

// ---------------------------------------------------------------------------
// The ladder runner.
// ---------------------------------------------------------------------------

const ALLOCATION_NOTE: &str = "zero-allocation packed steps are asserted by the repository \
     allocation census (crates/uor-r4-core/tests/allocation_census.rs); this run verified \
     the caller-owned RouteState epoch discipline on every step";

fn aggregate_runtime(heads: &[&HeadEvidence]) -> RuntimeChecks {
    let mut checks = RuntimeChecks {
        steps: 0,
        witness_replay_pass: true,
        witness_replay_detail: String::new(),
        census_closed_form_pass: true,
        reference_crosscheck_pass: true,
        state_epoch_pass: true,
        allocation_note: ALLOCATION_NOTE.to_owned(),
        pass: true,
    };
    for evidence in heads {
        checks.steps += evidence.steps;
        if evidence.replay_failures > 0 {
            checks.witness_replay_pass = false;
            if checks.witness_replay_detail.is_empty() {
                checks.witness_replay_detail = evidence.first_replay_detail.clone();
            }
        }
        if evidence.census_mismatches > 0 {
            checks.census_closed_form_pass = false;
        }
        if evidence.crosscheck_failures > 0 {
            checks.reference_crosscheck_pass = false;
        }
        if evidence.epoch_failures > 0 {
            checks.state_epoch_pass = false;
        }
    }
    checks.pass = checks.witness_replay_pass
        && checks.census_closed_form_pass
        && checks.reference_crosscheck_pass
        && checks.state_epoch_pass;
    checks
}

fn overlap_record(heads: &[HeadOverlaps], vacuity_fraction: f64) -> OverlapRecord {
    let mut fitted_sum = 0.0;
    let mut n1_sum = 0.0;
    let mut n2_sum = 0.0;
    let mut eligible = 0u64;
    for head in heads {
        fitted_sum += head.fitted_sum;
        n1_sum += head.n1_sum;
        n2_sum += head.n2_sum;
        eligible += head.eligible;
    }
    let denominator = eligible.max(1) as f64;
    let fitted = fitted_sum / denominator;
    let n1 = n1_sum / denominator;
    let n2 = n2_sum / denominator;
    OverlapRecord {
        fitted,
        n1,
        n2,
        best_null: n1.max(n2),
        eligible_steps: eligible,
        // Anti-vacuity: N2 must sit strictly below the declared fraction
        // of the fitted overlap.
        vacuous: n2 >= vacuity_fraction * fitted,
    }
}

/// Run the pre-registered replacement ladder over one fitted artifact.
/// Deterministic: the same inputs produce a byte-identical
/// [`RouteFitReport`]. The contract is taken as a parameter so tests
/// can prove the exit machinery with test-local margins; the BINDING
/// contract is [`preregistered_route_fit_contract`].
pub fn run_route_fit_ladder(
    teacher: &mut SyntheticRouteTeacher,
    corpus: &RouteTraceCorpus,
    fitted: &FittedRouteCodes,
    manifest: &FitManifest,
    contract: &RunContract,
    probe: &RealArmProbe,
) -> Result<RouteFitReport, SourceUnavailable> {
    // Structural validation: every replaced head of every synthetic
    // stage must be fitted, and the fitted code tables must align with
    // the corpus stories.
    for stage in &contract.stages {
        for head in &stage.replaced {
            if fitted.head(head.layer, head.head).is_none() {
                return Err(SourceUnavailable::new(format!(
                    "stage {} replaces (layer {}, head {}) but the fitted artifact \
                     carries no codes for it",
                    stage.stage, head.layer, head.head
                )));
            }
        }
    }
    for head in &fitted.heads {
        if head.query_codes.len() != corpus.stories.len()
            || head.key_codes.len() != corpus.stories.len()
        {
            return Err(SourceUnavailable::new(
                "fitted code tables do not align with the corpus stories",
            ));
        }
    }
    let reference_scope: Vec<ReplacedHead> = contract
        .stages
        .iter()
        .find(|stage| stage.kind == STAGE_KIND_SYNTHETIC && stage.replaced.len() == 1)
        .map(|stage| stage.replaced.clone())
        .ok_or_else(|| {
            SourceUnavailable::new(
                "the contract declares no one-head reference stage; the null stage's \
                 instrument scope is undefined",
            )
        })?;

    // Run-level evidence, computed once in fixed order.
    let preflight = source_parity_preflight(teacher, corpus);
    let null_codes = null_n1_codes(fitted, contract.nulls.n1_seed);
    let mut selections: BTreeMap<(u32, u32), Vec<Vec<Vec<u32>>>> = BTreeMap::new();
    let mut evidence_by_head: BTreeMap<(u32, u32), HeadEvidence> = BTreeMap::new();
    let mut overlaps_by_head: BTreeMap<(u32, u32), HeadOverlaps> = BTreeMap::new();
    for (head_index, head) in fitted.heads.iter().enumerate() {
        let lane_index = corpus
            .declared_layers
            .iter()
            .position(|&layer| layer == head.layer)
            .ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "fitted layer {} is not a declared trace layer",
                    head.layer
                ))
            })?;
        let evidence = packed_selection_evidence(head, fitted.top_m, true)
            .map_err(|error| SourceUnavailable::new(format!("selection evidence: {error}")))?;
        let n1_evidence = packed_selection_evidence(&null_codes[head_index], fitted.top_m, false)
            .map_err(|error| {
            SourceUnavailable::new(format!("null selection evidence: {error}"))
        })?;
        let overlaps = head_overlaps(
            corpus,
            lane_index,
            head.head,
            &evidence.selected,
            &n1_evidence.selected,
            fitted.top_m,
        );
        selections.insert((head.layer, head.head), evidence.selected.clone());
        evidence_by_head.insert((head.layer, head.head), evidence);
        overlaps_by_head.insert((head.layer, head.head), overlaps);
    }

    let manifest_kappa = manifest.kappa();
    let mut stages_out: Vec<StageRecord> = Vec::with_capacity(contract.stages.len());
    let mut stopped_at: Option<String> = None;
    let mut instrument_valid = true;
    let mut any_fail = false;

    for stage in &contract.stages {
        if let Some(stopped) = &stopped_at {
            stages_out.push(StageRecord {
                stage: stage.stage.clone(),
                kind: stage.kind.clone(),
                replaced: stage.replaced.clone(),
                verdict: StageVerdict::NotRun,
                reason: format!(
                    "not run: the ladder exited at stage {stopped} (pre-registered exit \
                     rule: stop at the first stage failing any gate; the negative is \
                     preserved)"
                ),
                ..StageRecord::default()
            });
            continue;
        }
        match stage.kind.as_str() {
            STAGE_KIND_SYNTHETIC => {
                let scope_heads: Vec<(u32, u32)> = if stage.replaced.is_empty() {
                    reference_scope
                        .iter()
                        .map(|head| (head.layer, head.head))
                        .collect()
                } else {
                    stage
                        .replaced
                        .iter()
                        .map(|head| (head.layer, head.head))
                        .collect()
                };
                let runtime = aggregate_runtime(
                    &scope_heads
                        .iter()
                        .map(|key| &evidence_by_head[key])
                        .collect::<Vec<_>>(),
                );
                let overlap = overlap_record(
                    &scope_heads
                        .iter()
                        .map(|key| overlaps_by_head[key].clone())
                        .collect::<Vec<_>>(),
                    contract.gates.n2_vacuity_fraction,
                );
                let metrics =
                    scope_metrics(teacher, corpus, &stage.replaced, &selections, contract);
                if overlap.vacuous {
                    instrument_valid = false;
                }
                // Gate evaluation, fixed order; the first miss names the
                // stage's failure.
                let mut verdict = StageVerdict::Pass;
                let mut reason = String::new();
                if overlap.vacuous {
                    verdict = StageVerdict::Fail;
                    reason = format!(
                        "instrument VACUOUS at this scope: N2 overlap {:.6} is not below \
                         {:.2} x fitted overlap {:.6}; a vacuous instrument fails the whole \
                         run regardless of other numbers",
                        overlap.n2, contract.gates.n2_vacuity_fraction, overlap.fitted
                    );
                } else if preflight.status != ConformanceStatus::Pass {
                    verdict = StageVerdict::Fail;
                    reason = "source-parity preflight FAILED".to_owned();
                } else if !runtime.pass {
                    verdict = StageVerdict::Fail;
                    reason = format!(
                        "runtime checks FAILED (witness replay {}, census {}, reference \
                         cross-check {}, state epoch {})",
                        runtime.witness_replay_pass,
                        runtime.census_closed_form_pass,
                        runtime.reference_crosscheck_pass,
                        runtime.state_epoch_pass
                    );
                } else if !stage.replaced.is_empty() {
                    let overlap_bar = (contract.gates.overlap_null_factor * overlap.best_null)
                        .max(contract.gates.overlap_floor);
                    if overlap.fitted < overlap_bar {
                        verdict = StageVerdict::Fail;
                        reason = format!(
                            "fitted support overlap {:.6} is below the pre-registered bar \
                             max({:.1} x best null {:.6}, {:.2}) = {:.6}",
                            overlap.fitted,
                            contract.gates.overlap_null_factor,
                            overlap.best_null,
                            contract.gates.overlap_floor,
                            overlap_bar
                        );
                    }
                }
                if verdict == StageVerdict::Pass
                    && metrics.replaced.top1_agreement < contract.gates.min_top1_agreement
                {
                    verdict = StageVerdict::Fail;
                    reason = format!(
                        "teacher-forced top-1 agreement {:.6} is below the pre-registered \
                         floor {:.2}",
                        metrics.replaced.top1_agreement, contract.gates.min_top1_agreement
                    );
                }
                if verdict == StageVerdict::Pass
                    && metrics.bits_ratio > contract.gates.max_bits_per_token_ratio
                {
                    verdict = StageVerdict::Fail;
                    reason = format!(
                        "replaced bits/token ratio {:.6} exceeds the pre-registered cap \
                         {:.2}",
                        metrics.bits_ratio, contract.gates.max_bits_per_token_ratio
                    );
                }
                if verdict == StageVerdict::Pass {
                    reason = format!(
                        "every pre-registered gate held at this scope (overlap {:.6}, \
                         nulls n1 {:.6} / n2 {:.6}, top-1 {:.6}, bits ratio {:.6})",
                        overlap.fitted,
                        overlap.n1,
                        overlap.n2,
                        metrics.replaced.top1_agreement,
                        metrics.bits_ratio
                    );
                }
                if verdict == StageVerdict::Fail {
                    any_fail = true;
                    stopped_at = Some(stage.stage.clone());
                }
                stages_out.push(StageRecord {
                    stage: stage.stage.clone(),
                    kind: stage.kind.clone(),
                    replaced: stage.replaced.clone(),
                    fit_manifest_kappa: Some(manifest_kappa.clone()),
                    preflight: Some(preflight.clone()),
                    runtime: Some(runtime),
                    overlap: Some(overlap),
                    teacher: Some(metrics.teacher),
                    replaced_metrics: Some(metrics.replaced),
                    top_k_agreement: Some(metrics.top_k_agreement),
                    bits_per_token_ratio: Some(metrics.bits_ratio),
                    verdict,
                    reason,
                });
            }
            STAGE_KIND_REAL_TEACHER => {
                let snapshot = &probe.smollm2_snapshot_dir;
                let reason = if snapshot.join("config.json").is_file() {
                    format!(
                        "pinned SmolLM2 snapshot present at {}, but the #531 saturation \
                         corpus is not yet produced (compute-bound); the real-teacher \
                         ladder arm is not attempted in this run",
                        snapshot.display()
                    )
                } else {
                    format!(
                        "pinned SmolLM2 snapshot absent from build env ({}: no \
                         config.json); the real-teacher arm cannot run",
                        snapshot.display()
                    )
                };
                stages_out.push(StageRecord {
                    stage: stage.stage.clone(),
                    kind: stage.kind.clone(),
                    replaced: stage.replaced.clone(),
                    verdict: StageVerdict::Unavailable,
                    reason,
                    ..StageRecord::default()
                });
            }
            STAGE_KIND_REAL_CORPUS => {
                let reason = match &probe.saturation_corpus {
                    Some(path) if path.exists() => format!(
                        "saturation-corpus path {} exists, but the real-corpus arm \
                         requires the pinned real teacher together with it; the arm is \
                         not attempted in this run",
                        path.display()
                    ),
                    _ => "#531 saturation corpus not yet produced — compute-bound; no \
                          corpus artifact exists to evaluate against"
                        .to_owned(),
                };
                stages_out.push(StageRecord {
                    stage: stage.stage.clone(),
                    kind: stage.kind.clone(),
                    replaced: stage.replaced.clone(),
                    verdict: StageVerdict::Unavailable,
                    reason,
                    ..StageRecord::default()
                });
            }
            other => {
                return Err(SourceUnavailable::new(format!(
                    "contract declares an unknown stage kind {other:?}"
                )));
            }
        }
    }

    let synthetic_passes = stages_out
        .iter()
        .filter(|record| {
            record.kind == STAGE_KIND_SYNTHETIC && record.verdict == StageVerdict::Pass
        })
        .count();
    let outcome = if !instrument_valid {
        "the run is INVALID: the instrument was measured VACUOUS (N2 not below the \
         pre-registered fraction of the fitted overlap); no stage number in this report \
         may be interpreted as fit evidence"
            .to_owned()
    } else if any_fail {
        let stopped = stopped_at.clone().unwrap_or_default();
        format!(
            "negative synthetic result: the ladder exited at stage {stopped}; \
             {synthetic_passes} synthetic stage(s) passed before it; the negative is \
             preserved and the pre-declared negative next action applies"
        )
    } else {
        format!(
            "positive synthetic result: all {synthetic_passes} synthetic stages passed \
             the pre-registered gates; the real-teacher and real-corpus stages remain \
             UNAVAILABLE, so no model-quality claim exists and the dormant lane's \
             activation gate remains uncleared"
        )
    };
    Ok(RouteFitReport {
        schema: ROUTE_FIT_REPORT_SCHEMA.to_owned(),
        contract: contract.clone(),
        fit_manifest_kappa: manifest_kappa,
        fitted_params_kappa: fitted.kappa(),
        instrument_valid,
        stages: stages_out,
        decision: DecisionRecord {
            positive_next: contract.decision_positive.clone(),
            negative_next: contract.decision_negative.clone(),
            outcome,
        },
    })
}
