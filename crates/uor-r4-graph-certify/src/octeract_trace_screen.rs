//! Deterministic Octeract route-trace collision/reachability screen (#661/#722).
//!
//! This host-side certifier consumes the existing `full/1` trace and
//! `route-fit/1` products. It neither reads a fixture implicitly nor defines a
//! new fit, packed operator, artifact section, or serving path. Synthetic
//! inputs exercise instrument conformance only; empirical candidate verdicts
//! require an explicitly supplied, completely identified real trace.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use uor_r4_core::transformerless::compiler::xorshift;
use uor_r4_core::transformerless::hf_bpe::adapter_constructor;
use uor_r4_graph_compiler::observation::ObservationManifest;
use uor_r4_graph_compiler::route_fit::{
    fit_method_spec, fit_route_codes, route_fit_v1_parameter_labels, FitManifest, FittedRouteCodes,
    RouteFitMethod, RouteTraceCorpus, FIT_MANIFEST_FORMAT,
};
use uor_r4_graph_compiler::trace_profile::{
    profile_spec, TraceCaptureBounds, FULL_PROFILE, PROFILE_VERSION,
};
use uor_r4_graph_format::route_attention::{
    build_route_attention_instance, RouteOpCensus, ROUTE_CODE_BITS, ROUTE_CODE_BYTES,
    ROUTE_MAX_CANDIDATES,
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_model_source::attention::{operator_spec, AttentionOperatorSpec};
use uor_r4_model_source::geometry::{projection_implementation, GeometryProjection};

use crate::frame_consistency::{frame_control_default, FrameControl};
use crate::octeract::{
    distance_from_oriented, folded_class, masked_byte_distance, masked_weight_lower_bound,
    oriented_class, BlockDistance, OCTERACT_CYPHER_SOURCE, OCTERACT_VALIDATION_SOURCE,
};
use crate::route_attention::{run_packed, RouteAttentionReference, RouteSelection};
use crate::route_fit_report::StageVerdict;

/// Canonical pre-inspection contract tag.
pub const OCTERACT_TRACE_CONTRACT_FORMAT: &str = "uor-r4-octeract-trace-screen-contract/1";
/// Canonical report-envelope tag.
pub const OCTERACT_TRACE_REPORT_FORMAT: &str = "uor-r4-octeract-trace-screen/1";
/// Occupancy-matched fold-null seed locked by #722.
pub const OCCUPANCY_MATCHED_FOLD_SEED: u64 = 0x661B_0001;
/// Shuffled-key-block null seed locked by #722.
pub const SHUFFLED_BLOCK_SEED: u64 = 0x661B_0002;
/// Layer selected before trace inspection.
pub const SCREEN_LAYER: u32 = 0;
/// Head selected before trace inspection.
pub const SCREEN_HEAD: u32 = 0;
/// Full-byte relation mask.
pub const SCREEN_MASK: [u8; ROUTE_CODE_BYTES] = [0xff; ROUTE_CODE_BYTES];

const ARM_V1: &str = "v1-baseline";
const ARM_WEIGHT9: &str = "weight9-control";
const ARM_FOLD5: &str = "octeract-fold5";
const ARM_ORIENTED: &str = "octeract-oriented-control";
const ARM_PREFILTER: &str = "octeract-prefilter";
const ARM_LOWER_BOUND: &str = "safe-lower-bound-control";
const NULL_OCCUPANCY: &str = "occupancy-matched-fold-null";
const NULL_SHUFFLED_BLOCK: &str = "shuffled-block-null";
const NULL_DERANGED_SUPPORT: &str = "deranged-support-null";
/// Initial registry id for structural-only evidence.
pub const INSTRUMENT_CONFORMANCE_EVIDENCE_ID: &str = "instrument-conformance";
/// Initial registry version for structural-only evidence.
pub const INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION: u32 = 1;
/// Number of bijections of the five folded-shell anchor identities.
pub const ANCHOR_RELABELINGS: u32 = 120;

/// Whether supplied evidence may carry empirical candidate verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceKind {
    /// A completely identified, pinned real `full/1` trace.
    #[serde(rename = "pinned-real")]
    PinnedReal,
    /// A synthetic/manual fixture usable only for structural conformance.
    #[serde(rename = "instrument-conformance")]
    InstrumentConformance,
}

/// Opaque registry record authorizing one evidence class. A caller cannot
/// manufacture or relabel this value: records are obtained only through
/// [`registered_trace_evidence`]. Future empirical records must pin every
/// decoded observation/fit identity represented by the private fields below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceEvidenceRecord {
    id: &'static str,
    version: u32,
    kind: TraceKind,
    observation_identity_bundle_digest: Option<&'static str>,
    records_kappa: Option<&'static str>,
    trace_kappa: Option<&'static str>,
    fit_manifest_kappa: Option<&'static str>,
    fitted_params_kappa: Option<&'static str>,
}

impl TraceEvidenceRecord {
    /// Stable registry id.
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// Registry version.
    pub const fn version(self) -> u32 {
        self.version
    }

    /// Evidence class authorized by this registry record.
    pub const fn kind(self) -> TraceKind {
        self.kind
    }

    /// Canonical digest of the complete registry record, including explicit
    /// absence markers for identity pins.
    pub fn declared_digest(self) -> String {
        fn field(value: Option<&str>) -> String {
            value.map_or_else(|| "absent".to_owned(), |value| format!("present:{value}"))
        }
        let kind = match self.kind {
            TraceKind::PinnedReal => "pinned-real",
            TraceKind::InstrumentConformance => "instrument-conformance",
        };
        let bytes = format!(
            "uor-r4-octeract-trace-evidence/1\nid={}\nversion={}\nkind={}\nobservation={}\nrecords={}\ntrace={}\nfit-manifest={}\nfitted-params={}\n",
            self.id,
            self.version,
            kind,
            field(self.observation_identity_bundle_digest),
            field(self.records_kappa),
            field(self.trace_kappa),
            field(self.fit_manifest_kappa),
            field(self.fitted_params_kappa),
        );
        format!("blake3:{}", blake3::hash(bytes.as_bytes()).to_hex())
    }
}

/// Resolve an evidence record through the closed #722 registry. The initial
/// registry intentionally contains no empirical record: a real trace becomes
/// eligible only when its exact observation and fit identities are pinned in
/// a reviewed follow-up, never by caller-selected [`TraceKind`] or adapter
/// spelling.
const fn instrument_trace_evidence() -> TraceEvidenceRecord {
    TraceEvidenceRecord {
        id: INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
        version: INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
        kind: TraceKind::InstrumentConformance,
        observation_identity_bundle_digest: None,
        records_kappa: None,
        trace_kappa: None,
        fit_manifest_kappa: None,
        fitted_params_kappa: None,
    }
}

pub fn registered_trace_evidence(id: &str, version: u32) -> Option<TraceEvidenceRecord> {
    match (id, version) {
        (INSTRUMENT_CONFORMANCE_EVIDENCE_ID, INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION) => {
            Some(instrument_trace_evidence())
        }
        _ => None,
    }
}

/// Explicit input boundary. Source/operator provenance is accepted only via
/// the authoritative observation manifest whose identity-bundle digest the
/// decoded corpus carries. It is never inferred from
/// `FitManifest::operator_identity` (which identifies the target dormant
/// operator) or supplied as a free-floating digest/spec.
#[derive(Debug, Clone, Copy)]
pub struct OcteractTraceInput<'a> {
    pub corpus: &'a RouteTraceCorpus,
    pub fitted: &'a FittedRouteCodes,
    pub fit_manifest: &'a FitManifest,
    pub observation_manifest: Option<&'a ObservationManifest>,
    /// Closed registry evidence record. In the initial #722 release this can
    /// authorize structural instrument conformance only.
    pub evidence: TraceEvidenceRecord,
}

/// Final branch selected by the locked #722 decision rules.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenDisposition {
    /// At least one promotable Octeract arm cleared every locked gate.
    #[serde(rename = "advance")]
    Advance,
    /// A valid real screen ran and no promotable arm cleared.
    #[serde(rename = "stop-negative")]
    StopNegative,
    /// Real evidence or a valid frame/instrument was absent.
    #[default]
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

/// One arm/null declaration serialized in the contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmContract {
    /// Stable arm id.
    pub id: String,
    /// Whether this arm can advance #661.
    pub can_advance: bool,
    /// Locked score and tie semantics.
    pub rule: String,
}

/// One closed evidence-registry declaration serialized in the preinspection
/// contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvidenceContractRecord {
    pub id: String,
    pub version: u32,
    pub kind: TraceKind,
    pub declared_digest: String,
}

/// Locked numerical thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenThresholds {
    /// Direct fold oracle and realized Jaccard margin above V1.
    pub direct_jaccard_margin: f64,
    /// Minimum prefilter recall of the full V1 selection.
    pub prefilter_v1_recall: f64,
    /// Maximum exact-refinement fraction on work-eligible steps.
    pub prefilter_refinement_fraction: f64,
}

/// Complete #722 screen contract, constructed without reading support labels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcteractTraceContract {
    /// [`OCTERACT_TRACE_CONTRACT_FORMAT`].
    pub format: String,
    /// Fixed layer.
    pub layer: u32,
    /// Fixed head.
    pub head: u32,
    /// Fixed route-code width.
    pub code_bits: u32,
    /// Natural byte-block count.
    pub blocks: u32,
    /// Fixed full-byte mask.
    pub mask: Vec<u8>,
    /// Stable candidate/tie rule.
    pub candidate_rule: String,
    /// `M=min(8, trace support cap)`.
    pub selection_width_rule: String,
    /// Direct-arm eligibility.
    pub eligible_step_rule: String,
    /// Fixed-order aggregation.
    pub aggregation: String,
    /// Arms in canonical order.
    pub arms: Vec<ArmContract>,
    /// Null arms in canonical order.
    pub nulls: Vec<ArmContract>,
    /// Historical weighted-Hamming comparison owner; cited, never rerun or
    /// relabeled as an Octeract arm.
    pub prior_weighted_hamming_row: String,
    /// Locked null seeds.
    pub occupancy_seed: u64,
    /// Locked shuffled-block seed.
    pub shuffled_block_seed: u64,
    /// Derived fixed nonidentity key-block permutation, serialized so a future
    /// RNG implementation change cannot silently move the null.
    pub shuffled_block_permutation: Vec<u8>,
    /// Locked thresholds.
    pub thresholds: ScreenThresholds,
    /// Frame policy.
    pub frame_rule: String,
    /// Five anchors are labels only; every bijective relabeling must preserve
    /// class-only results.
    pub anchor_relabel_rule: String,
    /// Closed evidence-registry policy.
    pub evidence_registry_rule: String,
    /// Initial closed registry, in stable order.
    pub evidence_registry: Vec<TraceEvidenceContractRecord>,
    /// Evidence/absence policy.
    pub unavailable_rule: String,
    /// Positive branch.
    pub decision_positive: String,
    /// Negative branch.
    pub decision_negative: String,
    /// Unavailable branch.
    pub decision_unavailable: String,
}

/// Identity bundle bound by a report. `None` is typed absence, never an empty
/// digest pretending to be evidence.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenIdentities {
    /// #720 primary-source SHA-256.
    pub octeract_source_sha256: String,
    /// #720 validation-roadmap SHA-256.
    pub validation_source_sha256: String,
    /// #603 observation identity-bundle digest.
    pub observation_identity_bundle_digest: Option<String>,
    /// Exact observation input CID (#603 identity bundle component).
    pub observation_input_cid: Option<String>,
    /// #597 source-manifest file binding from the observation manifest. This
    /// is intentionally distinct from `source_snapshot`.
    pub source_manifest_kappa: Option<String>,
    /// Merged observation-record κ.
    pub records_kappa: Option<String>,
    /// Merged trace-sidecar κ.
    pub trace_kappa: Option<String>,
    /// Declared trace-profile digest.
    pub trace_profile_digest: Option<String>,
    /// `route-fit/1` declared digest.
    pub route_fit_digest: Option<String>,
    /// Fit-manifest κ.
    pub fit_manifest_kappa: Option<String>,
    /// Fitted-parameter κ.
    pub fitted_params_kappa: Option<String>,
    /// Source snapshot identity.
    pub source_snapshot: Option<String>,
    /// Declared observation source-to-compiled geometry digest.
    pub source_geometry_digest: Option<String>,
    /// Raw tokenizer-definition CID from the typed observation adapter.
    pub tokenizer_cid: Option<String>,
    /// Raw declared adapter digest from the typed observation adapter.
    pub tokenizer_adapter_digest: Option<String>,
    /// Tokenizer identity.
    pub tokenizer: Option<String>,
    /// Adapter identity.
    pub adapter: Option<String>,
    /// Target dormant `r4-route-attention/1` digest from the fit manifest.
    pub target_operator_digest: Option<String>,
    /// Teacher/source attention-operator digest, supplied explicitly.
    pub source_attention_operator_digest: Option<String>,
    /// Compiler identity.
    pub compiler: Option<String>,
    /// Closed evidence-registry id, absent only when no input was supplied.
    pub trace_evidence_id: Option<String>,
    /// Closed evidence-registry version.
    pub trace_evidence_version: Option<u32>,
    /// Evidence class encoded by the registry record.
    pub trace_evidence_kind: Option<TraceKind>,
    /// Digest of the complete registry record.
    pub trace_evidence_digest: Option<String>,
}

/// Fixed finite counts for an arm/null.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenCounts {
    pub eligible_stories: u64,
    pub eligible_steps: u64,
    pub candidates: u64,
    pub teacher_support_entries: u64,
}

/// Exact and folded occupancy, pooled and by natural byte block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OccupancyMetrics {
    pub exact_weight: Vec<u64>,
    pub folded_shell: Vec<u64>,
    pub exact_weight_per_block: Vec<Vec<u64>>,
    pub folded_shell_per_block: Vec<Vec<u64>>,
    pub exact_entropy_bits: f64,
    pub folded_entropy_bits: f64,
}

/// Fold-only occupancy for a null that transforms categorical shell labels
/// but has no exact-distance representation of its own.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FoldOccupancyMetrics {
    pub folded_shell: Vec<u64>,
    pub folded_shell_per_block: Vec<Vec<u64>>,
    pub folded_entropy_bits: f64,
}

/// Selection-quality measurements under a locked stable tie rule.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectionMetrics {
    pub mean_teacher_recall: f64,
    pub mean_teacher_jaccard: f64,
    pub score_support_mi_bits: f64,
    pub pooled_block_class_support_mi_bits: f64,
    pub per_block_class_support_mi_bits: Vec<f64>,
}

/// Complete-signature collision measurements.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CollisionMetrics {
    pub complete_fold_signature_groups: u64,
    pub exact_collision_pairs: u64,
    pub complement_collision_pairs: u64,
    pub lossy_signature_groups: u64,
    pub group_size_distribution: Vec<(u32, u64)>,
    pub within_group_membership_disagreements: u64,
}

/// Fold shortlist classification counts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortlistMetrics {
    pub false_positives: u64,
    pub false_negatives: u64,
}

/// V1-refinement work and fidelity for the prefilter arm.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PrefilterMetrics {
    pub work_eligible_steps: u64,
    pub mean_v1_recall: f64,
    pub mean_v1_jaccard: f64,
    pub exact_refinement_candidates: u64,
    pub total_candidates: u64,
    pub exact_refinement_fraction: f64,
    pub exact_refinement_candidates_avoided: u64,
    pub occupancy_null_teacher_jaccard: f64,
    pub shuffled_block_null_teacher_jaccard: f64,
    pub deranged_support_null_teacher_jaccard: f64,
}

/// Safe lower-bound tightness and exact-evaluation work.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LowerBoundMetrics {
    pub candidates: u64,
    pub tight_candidates: u64,
    pub tight_fraction: f64,
    pub exact_evaluations: u64,
    pub exact_evaluations_avoided: u64,
}

/// Optimistic signature-only reachability ceiling.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OracleMetrics {
    pub mean_max_recall: f64,
    pub mean_max_jaccard: f64,
}

/// Measurements shared by the bounded evaluation domain, rather than copied
/// into unrelated arm/null rows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScreenSharedMetrics {
    /// Matched-support base domain used by the direct/control rows. Rows with
    /// a narrowed or transformed domain carry their own counts separately.
    pub counts: ScreenCounts,
    pub occupancy: OccupancyMetrics,
}

/// Measurements carried by one arm/null. `None` is explicit non-applicability,
/// so prefilter, lower-bound, collision, and oracle measurements cannot be
/// mistaken for measurements of unrelated rows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArmMetrics {
    /// Exact row domain. `None` means the row did not run.
    pub counts: Option<ScreenCounts>,
    pub occupancy: Option<OccupancyMetrics>,
    pub fold_occupancy: Option<FoldOccupancyMetrics>,
    pub selection: Option<SelectionMetrics>,
    pub collisions: Option<CollisionMetrics>,
    pub shortlist: Option<ShortlistMetrics>,
    pub prefilter: Option<PrefilterMetrics>,
    pub lower_bound: Option<LowerBoundMetrics>,
    pub oracle: Option<OracleMetrics>,
}

/// One evaluated arm or null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmReport {
    pub id: String,
    pub can_advance: bool,
    pub verdict: StageVerdict,
    pub reason: String,
    pub metrics: ArmMetrics,
}

/// Frame/instrument checks before any empirical negative is accepted.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScreenControls {
    pub v1_scalar_operator_exact: bool,
    pub weight9_exact: bool,
    pub oriented_exact: bool,
    pub lower_bound_exact: bool,
    pub anchor_relabel_invariant: bool,
    pub anchor_relabels_checked: u32,
    pub matched_pair_mean_jaccard: f64,
    pub frame: String,
    pub deranged_support_transformation_distinct: bool,
    pub deranged_support_observably_distinct: bool,
    pub occupancy_null_transformation_distinct: bool,
    pub occupancy_null_result_distinct: bool,
    pub shuffled_block_null_transformation_distinct: bool,
    pub shuffled_block_null_result_distinct: bool,
    pub instrument_valid: bool,
}

/// Canonical #722 report envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcteractTraceReport {
    /// [`OCTERACT_TRACE_REPORT_FORMAT`].
    pub format: String,
    /// Locked contract embedded verbatim.
    pub contract: OcteractTraceContract,
    /// Evidence class requested by the caller.
    pub trace_kind: TraceKind,
    /// Bound identities, with typed absence.
    pub identities: ScreenIdentities,
    /// Structural/frame controls.
    pub controls: ScreenControls,
    /// Domain counts/base occupancy shared by all rows.
    pub shared_metrics: ScreenSharedMetrics,
    /// Candidate/control arms, fixed contract order.
    pub arms: Vec<ArmReport>,
    /// Null rows, fixed contract order.
    pub nulls: Vec<ArmReport>,
    /// Locked branch outcome.
    pub disposition: ScreenDisposition,
    /// Exact reason for the branch.
    pub disposition_reason: String,
    /// Hash over the canonical payload with this field empty. The final
    /// envelope bytes include this non-self-referential identity.
    pub payload_kappa: String,
}

/// Candidate id and scalar score under one locked relation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScoredCandidate {
    pub candidate: u32,
    pub score: u32,
}

/// Prefilter result for one step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefilterStep {
    pub shortlist: Vec<u32>,
    pub selected: Vec<ScoredCandidate>,
    pub exact_refinement_candidates: u32,
    pub exact_refinement_candidates_avoided: u32,
}

/// Safe-lower-bound result for one step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LowerBoundStep {
    pub lower_bounds: Vec<u32>,
    pub selected: Vec<ScoredCandidate>,
    pub exact_evaluations: u32,
    pub exact_evaluations_avoided: u32,
    pub tight_candidates: u32,
}

/// Every exact/control score and selection needed to test one bounded step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepScores {
    pub exact_scores: Vec<u32>,
    pub weight9_scores: Vec<u32>,
    pub folded_scores: Vec<u32>,
    pub oriented_scores: Vec<u32>,
    pub v1_selected: Vec<ScoredCandidate>,
    pub weight9_selected: Vec<ScoredCandidate>,
    pub folded_selected: Vec<ScoredCandidate>,
    pub oriented_selected: Vec<ScoredCandidate>,
    pub prefilter: PrefilterStep,
    pub lower_bound: LowerBoundStep,
    pub fold_classes: Vec<[u8; ROUTE_CODE_BYTES]>,
    pub weight9_blocks: Vec<[u8; ROUTE_CODE_BYTES]>,
    pub oriented_weights: Vec<[u8; ROUTE_CODE_BYTES]>,
}

/// One complete fold-signature group in ascending candidate order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollisionGroup {
    pub candidate_ids: Vec<u32>,
    pub teacher_membership: Vec<bool>,
}

/// Exact bounded oracle result for one step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OracleStep {
    pub max_teacher_hits: u32,
    pub selected_slots: u32,
}

/// Inputs to the public pure verdict classifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandidateGate {
    pub controls_pass: bool,
    pub frame_score: f64,
    pub deranged_distinct: bool,
    pub gate_pass: bool,
}

/// Pure candidate verdict classifier. Synthetic evidence never becomes an
/// empirical PASS/FAIL; a prior structural stop produces NOT_RUN.
pub fn classify_candidate_arm(
    kind: TraceKind,
    gate: CandidateGate,
    prior_stop: bool,
) -> StageVerdict {
    if prior_stop {
        return StageVerdict::NotRun;
    }
    if !gate.controls_pass
        || !gate.deranged_distinct
        || frame_control_default(gate.frame_score).is_frame_mismatch()
    {
        return StageVerdict::Unavailable;
    }
    if kind == TraceKind::InstrumentConformance {
        return StageVerdict::NotRun;
    }
    if gate.gate_pass {
        StageVerdict::Pass
    } else {
        StageVerdict::Fail
    }
}

/// The complete locked contract. This function consumes no trace/support
/// input, so its bytes necessarily predate label inspection.
pub fn preregistered_octeract_trace_contract() -> OcteractTraceContract {
    let arm = |id: &str, can_advance: bool, rule: &str| ArmContract {
        id: id.to_owned(),
        can_advance,
        rule: rule.to_owned(),
    };
    let instrument_evidence = instrument_trace_evidence();
    OcteractTraceContract {
        format: OCTERACT_TRACE_CONTRACT_FORMAT.to_owned(),
        layer: SCREEN_LAYER,
        head: SCREEN_HEAD,
        code_bits: ROUTE_CODE_BITS as u32,
        blocks: ROUTE_CODE_BYTES as u32,
        mask: SCREEN_MASK.to_vec(),
        candidate_rule: "causal-prefix-position-ascending; stable ascending-(score,candidate-index)"
            .to_owned(),
        selection_width_rule: "M=min(8,trace-support-cap)".to_owned(),
        eligible_step_rule: "candidate-count>M".to_owned(),
        aggregation: "fixed-story-position-candidate-block-order; arithmetic-mean-per-step; row-counts-own-consumed-domain; shared-counts-are-matched-base-domain"
            .to_owned(),
        arms: vec![
            arm(
                ARM_V1,
                false,
                "sum-36-full-mask-xor-popcounts; packed/reference/scalar exact identity",
            ),
            arm(
                ARM_WEIGHT9,
                false,
                "preserve-each-byte-distance-0..8-and-sum; exact-V1-control",
            ),
            arm(
                ARM_FOLD5,
                true,
                "sum-per-byte-min(distance,8-distance); no-anchor-integer-scoring",
            ),
            arm(
                ARM_ORIENTED,
                false,
                "preserve-folded-shell-and-high-side; reconstruct-and-sum; exact-V1-control",
            ),
            arm(
                ARM_PREFILTER,
                true,
                "P=floor(3N/4); first-P-by-fold5; exact-V1-refine; only-P>=M-enters-work",
            ),
            arm(
                ARM_LOWER_BOUND,
                false,
                "ascending-index; skip-later-exact-only-when-L>=current-worst-selected-distance",
            ),
        ],
        nulls: vec![
            arm(
                NULL_OCCUPANCY,
                false,
                "within-(story,step,block)-permutation-of-observed-fold5-labels; step-seed=0x661B0001-XOR-(story<<32)-XOR-step",
            ),
            arm(
                NULL_SHUFFLED_BLOCK,
                false,
                "one-seeded-fixed-nonidentity-permutation-of-36-key-byte-indices",
            ),
            arm(
                NULL_DERANGED_SUPPORT,
                false,
                "cyclic-next-position-support-within-sequence",
            ),
        ],
        prior_weighted_hamming_row:
            "issue-310-prior-weighted-Hamming-row-cite-when-budgets-comparable-do-not-rerun"
                .to_owned(),
        occupancy_seed: OCCUPANCY_MATCHED_FOLD_SEED,
        shuffled_block_seed: SHUFFLED_BLOCK_SEED,
        shuffled_block_permutation: shuffled_block_permutation(SHUFFLED_BLOCK_SEED).to_vec(),
        thresholds: ScreenThresholds {
            direct_jaccard_margin: 0.03,
            prefilter_v1_recall: 0.95,
            prefilter_refinement_fraction: 0.75,
        },
        frame_rule: "V1-vs-teacher-support-mean-jaccard-through-frame_control_default; deranged-support-must-be-observably-distinct"
            .to_owned(),
        anchor_relabel_rule:
            "all-five-shell-label-bijections-preserve-class-only-scores-selections-and-groups"
                .to_owned(),
        evidence_registry_rule: "closed-registry-record-kind-must-match-request; empirical-record-must-pin-observation-records-trace-fit-manifest-and-fitted-params; initial-registry-has-instrument-conformance-only; registry-expansion-requires-new-contract-and-report-format-version"
            .to_owned(),
        evidence_registry: vec![TraceEvidenceContractRecord {
            id: instrument_evidence.id().to_owned(),
            version: instrument_evidence.version(),
            kind: instrument_evidence.kind(),
            declared_digest: instrument_evidence.declared_digest(),
        }],
        unavailable_rule: "missing-or-incomplete-real-trace/lane/identity/domain-or-frame-is-UNAVAILABLE-never-zero-FAIL-or-vacuous-PASS"
            .to_owned(),
        decision_positive:
            "create-661-C-with-only-clearing-Octeract-arms-at-most-two-remain-dormant"
                .to_owned(),
        decision_negative:
            "preserve-negative-report-no-one-head-child-trigger-bounded-classification-cache-fallback"
                .to_owned(),
        decision_unavailable:
            "preserve-typed-unavailable-create-neither-one-head-nor-negative-only-fallback"
                .to_owned(),
    }
}

fn bounded_block(distance: u8) -> BlockDistance {
    match BlockDistance::new(distance, 8) {
        Some(block) => block,
        None => unreachable!("a full-byte Hamming distance is always in 0..=8"),
    }
}

fn selected_from_scores(scores: &[u32], m: usize) -> Vec<ScoredCandidate> {
    let mut ranked: Vec<ScoredCandidate> = scores
        .iter()
        .enumerate()
        .map(|(candidate, &score)| ScoredCandidate {
            candidate: candidate as u32,
            score,
        })
        .collect();
    ranked.sort_by_key(|entry| (entry.score, entry.candidate));
    ranked.truncate(m);
    ranked
}

fn exact_score(query: &[u8; ROUTE_CODE_BYTES], key: &[u8; ROUTE_CODE_BYTES]) -> u32 {
    query
        .iter()
        .zip(key)
        .map(|(&query_byte, &key_byte)| {
            u32::from(masked_byte_distance(query_byte, key_byte, u8::MAX))
        })
        .sum()
}

/// Independent weight-9 representation: one exact distance `d_b in 0..=8`
/// per byte block, computed directly from XOR popcount rather than through the
/// V1 scalar helper or the oriented representation.
fn weight9_blocks(
    query: &[u8; ROUTE_CODE_BYTES],
    key: &[u8; ROUTE_CODE_BYTES],
) -> [u8; ROUTE_CODE_BYTES] {
    let mut blocks = [0u8; ROUTE_CODE_BYTES];
    for (slot, (&query_byte, &key_byte)) in query.iter().zip(key).enumerate() {
        blocks[slot] = (query_byte ^ key_byte).count_ones() as u8;
    }
    blocks
}

fn fold_classes(
    query: &[u8; ROUTE_CODE_BYTES],
    key: &[u8; ROUTE_CODE_BYTES],
) -> [u8; ROUTE_CODE_BYTES] {
    let mut classes = [0u8; ROUTE_CODE_BYTES];
    for (slot, (&query_byte, &key_byte)) in query.iter().zip(key).enumerate() {
        let distance = masked_byte_distance(query_byte, key_byte, u8::MAX);
        classes[slot] = folded_class(bounded_block(distance));
    }
    classes
}

fn oriented_weights(
    query: &[u8; ROUTE_CODE_BYTES],
    key: &[u8; ROUTE_CODE_BYTES],
) -> [u8; ROUTE_CODE_BYTES] {
    let mut weights = [0u8; ROUTE_CODE_BYTES];
    for (slot, (&query_byte, &key_byte)) in query.iter().zip(key).enumerate() {
        let distance = masked_byte_distance(query_byte, key_byte, u8::MAX);
        weights[slot] = distance_from_oriented(oriented_class(bounded_block(distance)));
    }
    weights
}

fn prefilter_step(exact_scores: &[u32], folded_scores: &[u32], m: usize) -> PrefilterStep {
    let n = exact_scores.len();
    let p = (3 * n) / 4;
    if p < m {
        return PrefilterStep::default();
    }
    let mut fold_rank: Vec<(u32, u32)> = folded_scores
        .iter()
        .enumerate()
        .map(|(candidate, &score)| (score, candidate as u32))
        .collect();
    fold_rank.sort_unstable();
    let shortlist: Vec<u32> = fold_rank
        .into_iter()
        .take(p)
        .map(|(_, candidate)| candidate)
        .collect();
    let mut selected: Vec<ScoredCandidate> = shortlist
        .iter()
        .map(|&candidate| ScoredCandidate {
            candidate,
            score: exact_scores[candidate as usize],
        })
        .collect();
    selected.sort_by_key(|entry| (entry.score, entry.candidate));
    selected.truncate(m);
    PrefilterStep {
        shortlist,
        selected,
        exact_refinement_candidates: p as u32,
        exact_refinement_candidates_avoided: (n - p) as u32,
    }
}

fn lower_bound_step(
    query: &[u8; ROUTE_CODE_BYTES],
    keys: &[[u8; ROUTE_CODE_BYTES]],
    exact_scores: &[u32],
    m: usize,
) -> LowerBoundStep {
    let mut lower_bounds = Vec::with_capacity(keys.len());
    let mut selected = Vec::<ScoredCandidate>::with_capacity(m);
    let mut exact_evaluations = 0u32;
    let mut avoided = 0u32;
    let mut tight = 0u32;
    for (candidate, key) in keys.iter().enumerate() {
        let lower: u32 = query
            .iter()
            .zip(key)
            .map(|(&query_byte, &key_byte)| {
                u32::from(masked_weight_lower_bound(query_byte, key_byte, u8::MAX))
            })
            .sum();
        lower_bounds.push(lower);
        let exact = exact_scores[candidate];
        if lower == exact {
            tight += 1;
        }
        if selected.len() == m {
            let worst = selected[m - 1].score;
            // Candidate indices arrive strictly ascending. On equality this
            // later candidate loses the stable V1 tie and can be skipped.
            if lower >= worst {
                avoided += 1;
                continue;
            }
        }
        exact_evaluations += 1;
        selected.push(ScoredCandidate {
            candidate: candidate as u32,
            score: exact,
        });
        selected.sort_by_key(|entry| (entry.score, entry.candidate));
        selected.truncate(m);
    }
    LowerBoundStep {
        lower_bounds,
        selected,
        exact_evaluations,
        exact_evaluations_avoided: avoided,
        tight_candidates: tight,
    }
}

/// Score one bounded step under every exact/folded/control relation. Invalid
/// instance shapes do not construct a result (R5).
pub fn score_octeract_step(
    query: &[u8; ROUTE_CODE_BYTES],
    keys: &[[u8; ROUTE_CODE_BYTES]],
    m: usize,
) -> Option<StepScores> {
    if keys.is_empty() || keys.len() > ROUTE_MAX_CANDIDATES || m == 0 || m > keys.len().min(8) {
        return None;
    }
    let mut exact_scores = Vec::with_capacity(keys.len());
    let mut weight9_scores = Vec::with_capacity(keys.len());
    let mut folded_scores = Vec::with_capacity(keys.len());
    let mut oriented_scores = Vec::with_capacity(keys.len());
    let mut all_fold_classes = Vec::with_capacity(keys.len());
    let mut all_weight9_blocks = Vec::with_capacity(keys.len());
    let mut all_oriented_weights = Vec::with_capacity(keys.len());
    for key in keys {
        let exact = exact_score(query, key);
        let weight9 = weight9_blocks(query, key);
        let classes = fold_classes(query, key);
        let oriented = oriented_weights(query, key);
        exact_scores.push(exact);
        weight9_scores.push(weight9.iter().map(|&value| u32::from(value)).sum());
        folded_scores.push(classes.iter().map(|&value| u32::from(value)).sum());
        oriented_scores.push(oriented.iter().map(|&value| u32::from(value)).sum());
        all_fold_classes.push(classes);
        all_weight9_blocks.push(weight9);
        all_oriented_weights.push(oriented);
    }
    let v1_selected = selected_from_scores(&exact_scores, m);
    let weight9_selected = selected_from_scores(&weight9_scores, m);
    let folded_selected = selected_from_scores(&folded_scores, m);
    let oriented_selected = selected_from_scores(&oriented_scores, m);
    let prefilter = prefilter_step(&exact_scores, &folded_scores, m);
    let lower_bound = lower_bound_step(query, keys, &exact_scores, m);
    Some(StepScores {
        exact_scores,
        weight9_scores,
        folded_scores,
        oriented_scores,
        v1_selected,
        weight9_selected,
        folded_selected,
        oriented_selected,
        prefilter,
        lower_bound,
        fold_classes: all_fold_classes,
        weight9_blocks: all_weight9_blocks,
        oriented_weights: all_oriented_weights,
    })
}

fn signature_groups(classes: &[[u8; ROUTE_CODE_BYTES]]) -> Vec<Vec<u32>> {
    let mut groups = BTreeMap::<[u8; ROUTE_CODE_BYTES], Vec<u32>>::new();
    for (candidate, &signature) in classes.iter().enumerate() {
        groups.entry(signature).or_default().push(candidate as u32);
    }
    let mut memberships: Vec<Vec<u32>> = groups.into_values().collect();
    memberships.sort();
    memberships
}

fn next_permutation(values: &mut [u8; 5]) -> bool {
    let Some(pivot) = (0..values.len() - 1)
        .rev()
        .find(|&index| values[index] < values[index + 1])
    else {
        return false;
    };
    let successor = (pivot + 1..values.len())
        .rev()
        .find(|&index| values[pivot] < values[index])
        .unwrap_or(pivot + 1);
    values.swap(pivot, successor);
    values[pivot + 1..].reverse();
    true
}

/// Exhaustively verify the #720 anchor-label invariant for all `5! = 120`
/// bijections. Relabeled identities are resolved through the relabeled anchor
/// table (not interpreted as numeric weights), then scores, stable selections,
/// and complete-signature collision groups are compared to the canonical row.
fn anchor_relabel_control_result(scores: &StepScores, m: usize) -> (bool, u32) {
    if scores.fold_classes.is_empty()
        || m == 0
        || m > scores.fold_classes.len().min(8)
        || scores
            .fold_classes
            .iter()
            .flatten()
            .any(|&class| class >= 5)
    {
        return (false, 0);
    }
    let canonical_scores: Vec<u32> = scores
        .fold_classes
        .iter()
        .map(|classes| classes.iter().map(|&value| u32::from(value)).sum())
        .collect();
    if canonical_scores != scores.folded_scores
        || selected_from_scores(&canonical_scores, m) != scores.folded_selected
    {
        return (false, 0);
    }
    let canonical_groups = signature_groups(&scores.fold_classes);
    let mut permutation = [0u8, 1, 2, 3, 4];
    let mut checked = 0u32;
    loop {
        let relabeled: Vec<[u8; ROUTE_CODE_BYTES]> = scores
            .fold_classes
            .iter()
            .map(|classes| {
                let mut row = [0u8; ROUTE_CODE_BYTES];
                for (block, &class) in classes.iter().enumerate() {
                    row[block] = permutation[usize::from(class)];
                }
                row
            })
            .collect();
        if signature_groups(&relabeled) != canonical_groups {
            return (false, checked);
        }
        let mut inverse = [0u8; 5];
        for (class, &label) in permutation.iter().enumerate() {
            inverse[usize::from(label)] = class as u8;
        }
        let decoded_scores: Vec<u32> = relabeled
            .iter()
            .map(|classes| {
                classes
                    .iter()
                    .map(|&label| u32::from(inverse[usize::from(label)]))
                    .sum()
            })
            .collect();
        if decoded_scores != canonical_scores
            || selected_from_scores(&decoded_scores, m) != scores.folded_selected
        {
            return (false, checked);
        }
        checked += 1;
        if !next_permutation(&mut permutation) {
            break;
        }
    }
    (checked == ANCHOR_RELABELINGS, checked)
}

/// Public boolean form of the exhaustive anchor control.
pub fn exhaustive_anchor_relabel_control(scores: &StepScores, m: usize) -> bool {
    anchor_relabel_control_result(scores, m).0
}

/// Deterministically permute each block's observed fold labels across the
/// causal candidate indices, preserving five-class occupancy exactly.
pub fn occupancy_matched_fold_null(
    classes: &[[u8; ROUTE_CODE_BYTES]],
    seed: u64,
) -> Vec<[u8; ROUTE_CODE_BYTES]> {
    let mut output = classes.to_vec();
    let mut stream = seed;
    for block in 0..ROUTE_CODE_BYTES {
        let mut labels: Vec<u8> = classes.iter().map(|row| row[block]).collect();
        for index in (1..labels.len()).rev() {
            let draw = xorshift(&mut stream) as usize;
            labels.swap(index, draw % (index + 1));
        }
        for (row, label) in output.iter_mut().zip(labels) {
            row[block] = label;
        }
    }
    output
}

/// Derive the fixed non-identity 36-key-byte permutation used by the shuffled
/// block null.
pub fn shuffled_block_permutation(seed: u64) -> [u8; ROUTE_CODE_BYTES] {
    let mut permutation = [0u8; ROUTE_CODE_BYTES];
    for (index, slot) in permutation.iter_mut().enumerate() {
        *slot = index as u8;
    }
    let mut stream = seed;
    for index in (1..permutation.len()).rev() {
        let draw = xorshift(&mut stream) as usize;
        permutation.swap(index, draw % (index + 1));
    }
    if permutation
        .iter()
        .enumerate()
        .all(|(index, &value)| usize::from(value) == index)
    {
        permutation.rotate_left(1);
    }
    permutation
}

/// #605 cyclic one-position support derangement within one sequence.
pub fn deranged_supports(supports: &[Vec<u32>]) -> Vec<Vec<u32>> {
    if supports.is_empty() {
        return Vec::new();
    }
    (0..supports.len())
        .map(|position| supports[(position + 1) % supports.len()].clone())
        .collect()
}

/// Exact bounded collision-oracle optimization. Distinct complete signatures
/// may be ordered arbitrarily; candidates within a signature retain ascending
/// index, and at most one final signature may be partial.
pub fn collision_oracle(groups: &[CollisionGroup], m: usize) -> Option<OracleStep> {
    let total: usize = groups.iter().map(|group| group.candidate_ids.len()).sum();
    if m == 0 || m > 8 || total < m || total > ROUTE_MAX_CANDIDATES {
        return None;
    }
    let mut all_candidates = Vec::with_capacity(total);
    for group in groups {
        if group.candidate_ids.is_empty()
            || group.candidate_ids.len() != group.teacher_membership.len()
            || !group.candidate_ids.windows(2).all(|pair| pair[0] < pair[1])
        {
            return None;
        }
        all_candidates.extend_from_slice(&group.candidate_ids);
    }
    all_candidates.sort_unstable();
    if all_candidates.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }

    fn whole_group_dp(groups: &[CollisionGroup], omitted: Option<usize>, m: usize) -> Vec<i32> {
        let mut dp = vec![-1i32; m + 1];
        dp[0] = 0;
        for (index, group) in groups.iter().enumerate() {
            if Some(index) == omitted {
                continue;
            }
            let width = group.candidate_ids.len();
            if width > m {
                continue;
            }
            let hits = group
                .teacher_membership
                .iter()
                .filter(|&&member| member)
                .count() as i32;
            let previous = dp.clone();
            for used in 0..=m.saturating_sub(width) {
                if previous[used] >= 0 {
                    dp[used + width] = dp[used + width].max(previous[used] + hits);
                }
            }
        }
        dp
    }

    let mut best = whole_group_dp(groups, None, m)[m];
    for (partial_index, partial) in groups.iter().enumerate() {
        let dp = whole_group_dp(groups, Some(partial_index), m);
        let mut prefix_hits = vec![0i32; partial.candidate_ids.len() + 1];
        for (index, &member) in partial.teacher_membership.iter().enumerate() {
            prefix_hits[index + 1] = prefix_hits[index] + i32::from(member);
        }
        for (used, &hits) in dp.iter().enumerate() {
            if hits < 0 || used >= m {
                continue;
            }
            let take = (m - used).min(partial.candidate_ids.len());
            if used + take == m {
                best = best.max(hits + prefix_hits[take]);
            }
        }
    }
    if best < 0 {
        None
    } else {
        Some(OracleStep {
            max_teacher_hits: best as u32,
            selected_slots: m as u32,
        })
    }
}

fn serialize_report(report: &OcteractTraceReport) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(report, &mut bytes)
        .expect("bounded Octeract trace report serializes to canonical bytes");
    bytes
}

fn canonical_payload_bytes(report: &OcteractTraceReport) -> Vec<u8> {
    let mut payload = report.clone();
    payload.payload_kappa.clear();
    serialize_report(&payload)
}

/// Non-self-referential payload identity: blake3 over canonical report bytes
/// with the `payload_kappa` field empty.
pub fn octeract_trace_payload_kappa(report: &OcteractTraceReport) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_payload_bytes(report)).to_hex()
    )
}

/// Canonical final envelope bytes. The payload identity is recomputed before
/// serialization so caller mutation of that derived field cannot fork bytes.
pub fn canonical_octeract_trace_report_bytes(report: &OcteractTraceReport) -> Vec<u8> {
    let mut envelope = report.clone();
    envelope.payload_kappa = octeract_trace_payload_kappa(&envelope);
    serialize_report(&envelope)
}

/// Final envelope κ over bytes which include the separately defined payload
/// identity.
pub fn octeract_trace_report_kappa(report: &OcteractTraceReport) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_octeract_trace_report_bytes(report)).to_hex()
    )
}

#[derive(Clone)]
struct MetricAccumulator {
    steps: u64,
    recall_sum: f64,
    jaccard_sum: f64,
    score_joint: BTreeMap<(u32, bool), u64>,
    class_joint: Vec<BTreeMap<(u32, bool), u64>>,
}

struct OccupancyAccumulator {
    exact_weight: Vec<u64>,
    folded_shell: Vec<u64>,
    exact_weight_per_block: Vec<Vec<u64>>,
    folded_shell_per_block: Vec<Vec<u64>>,
}

impl OccupancyAccumulator {
    fn new() -> Self {
        Self {
            exact_weight: vec![0; 9],
            folded_shell: vec![0; 5],
            exact_weight_per_block: vec![vec![0; 9]; ROUTE_CODE_BYTES],
            folded_shell_per_block: vec![vec![0; 5]; ROUTE_CODE_BYTES],
        }
    }

    fn record_fold(&mut self, classes: &[[u8; ROUTE_CODE_BYTES]]) {
        for row in classes {
            for (block, &class) in row.iter().enumerate() {
                self.folded_shell[usize::from(class)] += 1;
                self.folded_shell_per_block[block][usize::from(class)] += 1;
            }
        }
    }

    fn record_exact_and_fold(
        &mut self,
        exact: &[[u8; ROUTE_CODE_BYTES]],
        classes: &[[u8; ROUTE_CODE_BYTES]],
    ) {
        if exact.len() != classes.len() {
            return;
        }
        self.record_fold(classes);
        for row in exact {
            for (block, &distance) in row.iter().enumerate() {
                self.exact_weight[usize::from(distance)] += 1;
                self.exact_weight_per_block[block][usize::from(distance)] += 1;
            }
        }
    }

    fn finish(self) -> OccupancyMetrics {
        OccupancyMetrics {
            exact_entropy_bits: entropy(&self.exact_weight),
            folded_entropy_bits: entropy(&self.folded_shell),
            exact_weight: self.exact_weight,
            folded_shell: self.folded_shell,
            exact_weight_per_block: self.exact_weight_per_block,
            folded_shell_per_block: self.folded_shell_per_block,
        }
    }

    fn finish_fold(self) -> FoldOccupancyMetrics {
        FoldOccupancyMetrics {
            folded_entropy_bits: entropy(&self.folded_shell),
            folded_shell: self.folded_shell,
            folded_shell_per_block: self.folded_shell_per_block,
        }
    }
}

impl MetricAccumulator {
    fn new() -> Self {
        Self {
            steps: 0,
            recall_sum: 0.0,
            jaccard_sum: 0.0,
            score_joint: BTreeMap::new(),
            class_joint: vec![BTreeMap::new(); ROUTE_CODE_BYTES],
        }
    }

    fn record(
        &mut self,
        scores: &[u32],
        classes: &[[u8; ROUTE_CODE_BYTES]],
        selected: &[ScoredCandidate],
        teacher: &[u32],
    ) {
        let selected_ids: Vec<u32> = selected.iter().map(|entry| entry.candidate).collect();
        self.recall_sum += recall(&selected_ids, teacher);
        self.jaccard_sum += jaccard(&selected_ids, teacher);
        self.steps += 1;
        for (candidate, (&score, block_classes)) in scores.iter().zip(classes).enumerate() {
            let member = teacher.contains(&(candidate as u32));
            *self.score_joint.entry((score, member)).or_default() += 1;
            for (block, &class) in block_classes.iter().enumerate() {
                *self.class_joint[block]
                    .entry((u32::from(class), member))
                    .or_default() += 1;
            }
        }
    }

    fn finish(&self) -> SelectionMetrics {
        let denominator = self.steps as f64;
        let per_block: Vec<f64> = self.class_joint.iter().map(mutual_information).collect();
        let mut pooled = BTreeMap::new();
        for block in &self.class_joint {
            for (&key, &count) in block {
                *pooled.entry(key).or_default() += count;
            }
        }
        SelectionMetrics {
            mean_teacher_recall: if self.steps == 0 {
                0.0
            } else {
                self.recall_sum / denominator
            },
            mean_teacher_jaccard: if self.steps == 0 {
                0.0
            } else {
                self.jaccard_sum / denominator
            },
            score_support_mi_bits: mutual_information(&self.score_joint),
            pooled_block_class_support_mi_bits: mutual_information(&pooled),
            per_block_class_support_mi_bits: per_block,
        }
    }
}

fn intersection_count(left: &[u32], right: &[u32]) -> usize {
    left.iter().filter(|value| right.contains(value)).count()
}

fn recall(selected: &[u32], teacher: &[u32]) -> f64 {
    if teacher.is_empty() {
        0.0
    } else {
        intersection_count(selected, teacher) as f64 / teacher.len() as f64
    }
}

fn jaccard(left: &[u32], right: &[u32]) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 0.0;
    }
    let intersection = intersection_count(left, right);
    let union = left.len() + right.len() - intersection;
    intersection as f64 / union as f64
}

fn mutual_information(joint: &BTreeMap<(u32, bool), u64>) -> f64 {
    let total: u64 = joint.values().sum();
    if total == 0 {
        return 0.0;
    }
    let mut by_score = BTreeMap::<u32, u64>::new();
    let mut by_membership = [0u64; 2];
    for (&(score, member), &count) in joint {
        *by_score.entry(score).or_default() += count;
        by_membership[usize::from(member)] += count;
    }
    let total_f = total as f64;
    let mut information = 0.0;
    for (&(score, member), &count) in joint {
        if count == 0 {
            continue;
        }
        let p_xy = count as f64 / total_f;
        let p_x = by_score[&score] as f64 / total_f;
        let p_y = by_membership[usize::from(member)] as f64 / total_f;
        information += p_xy * libm::log2(p_xy / (p_x * p_y));
    }
    information
}

fn entropy(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let total_f = total as f64;
    counts
        .iter()
        .filter(|&&count| count != 0)
        .map(|&count| {
            let probability = count as f64 / total_f;
            -probability * libm::log2(probability)
        })
        .sum()
}

fn choose_two(value: usize) -> u64 {
    if value < 2 {
        0
    } else {
        (value as u64 * (value as u64 - 1)) / 2
    }
}

fn selection_ids(selected: &[ScoredCandidate]) -> Vec<u32> {
    selected.iter().map(|entry| entry.candidate).collect()
}

fn route_seam_matches(
    query: &[u8; ROUTE_CODE_BYTES],
    keys: &[[u8; ROUTE_CODE_BYTES]],
    m: usize,
    scalar: &[ScoredCandidate],
) -> bool {
    let contributions = vec![ScoreQ::ZERO; keys.len()];
    let instance =
        match build_route_attention_instance(&SCREEN_MASK, keys, &contributions, m as u32) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
    let (packed, _) = match run_packed(&instance, &[*query]) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let reference = match RouteAttentionReference::from_instance_bytes(&instance) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let mut census = RouteOpCensus::default();
    let reference_step = reference.reference_step(query, &mut census);
    let expected: Vec<RouteSelection> = scalar
        .iter()
        .map(|entry| RouteSelection {
            candidate: entry.candidate,
            distance: entry.score,
        })
        .collect();
    packed.len() == 1
        && packed[0].selected == expected
        && reference_step.selected == expected
        && packed[0].selected == reference_step.selected
}

enum InputCheck<'a> {
    Ready {
        input: OcteractTraceInput<'a>,
        lane_index: usize,
        source_operator_digest: Option<String>,
    },
    Unavailable(String),
}

fn nonempty(value: &Option<String>) -> bool {
    value.as_ref().is_some_and(|value| !value.is_empty())
}

fn canonical_blake3(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_input<'a>(input: OcteractTraceInput<'a>, kind: TraceKind) -> InputCheck<'a> {
    let corpus = input.corpus;
    let fitted = input.fitted;
    let manifest = input.fit_manifest;

    let registered_evidence =
        registered_trace_evidence(input.evidence.id(), input.evidence.version());

    let profile = &corpus.trace_profile;
    let registered_profile = match profile_spec(
        &profile.id,
        profile.version,
        &TraceCaptureBounds {
            layer_indices: corpus.declared_layers.clone(),
            support_size: corpus.support_size,
        },
    ) {
        Ok(record) => record,
        Err(_) => {
            return InputCheck::Unavailable(
                "trace profile is not a registered bounded record".to_owned(),
            );
        }
    };
    let qkv_layers = profile
        .qkv_lane
        .as_ref()
        .map(|lane| lane.layer_indices.as_slice());
    let support_lane = profile.attention_support_lane.as_ref();
    if &registered_profile != profile
        || profile.id != FULL_PROFILE
        || profile.version != PROFILE_VERSION
        || qkv_layers != Some(corpus.declared_layers.as_slice())
        || support_lane.map(|lane| lane.layer_indices.as_slice())
            != Some(corpus.declared_layers.as_slice())
        || support_lane.map(|lane| lane.support_size) != Some(corpus.support_size)
    {
        return InputCheck::Unavailable(
            "required pinned full/1 q/k and attention-support lanes are absent or misaligned"
                .to_owned(),
        );
    }
    let Some(lane_index) = corpus
        .declared_layers
        .iter()
        .position(|&layer| layer == SCREEN_LAYER)
    else {
        return InputCheck::Unavailable("declared trace lacks layer 0".to_owned());
    };
    if corpus.geometry.layers == 0
        || corpus.geometry.heads <= SCREEN_HEAD as usize
        || corpus.geometry.kv_heads == 0
        || corpus.geometry.residual_width == 0
        || !corpus
            .geometry
            .heads
            .is_multiple_of(corpus.geometry.kv_heads)
        || !corpus
            .geometry
            .residual_width
            .is_multiple_of(corpus.geometry.heads)
        || corpus.declared_layers.is_empty()
        || corpus.declared_layers.iter().any(|&layer| {
            usize::try_from(layer).map_or(true, |layer| layer >= corpus.geometry.layers)
        })
    {
        return InputCheck::Unavailable(
            "trace geometry has a zero/out-of-range dimension or non-integral head/kv dimensions"
                .to_owned(),
        );
    }
    if fitted
        .heads
        .windows(2)
        .any(|pair| (pair[0].layer, pair[0].head) >= (pair[1].layer, pair[1].head))
        || fitted
            .heads
            .iter()
            .filter(|head| head.layer == SCREEN_LAYER && head.head == SCREEN_HEAD)
            .count()
            != 1
    {
        return InputCheck::Unavailable(
            "route-fit head table is not uniquely ordered by (layer, head)".to_owned(),
        );
    }
    let Some(head) = fitted.head(SCREEN_LAYER, SCREEN_HEAD) else {
        return InputCheck::Unavailable("route-fit/1 lacks layer 0 head 0".to_owned());
    };
    if head.thresholds.len() != ROUTE_CODE_BITS
        || head.thresholds.iter().any(|value| !value.is_finite())
    {
        return InputCheck::Unavailable(
            "route-fit/1 head thresholds are incomplete or non-finite".to_owned(),
        );
    }
    let expected_m = corpus.support_size.min(8);
    if expected_m == 0 || fitted.top_m != expected_m {
        return InputCheck::Unavailable(
            "route-fit selection width disagrees with min(8, trace support cap)".to_owned(),
        );
    }
    if head.query_codes.len() != corpus.stories.len()
        || head.key_codes.len() != corpus.stories.len()
    {
        return InputCheck::Unavailable(
            "route-fit story tables are not aligned to RouteTraceCorpus".to_owned(),
        );
    }
    if corpus
        .stories
        .windows(2)
        .any(|pair| pair[0].story >= pair[1].story)
    {
        return InputCheck::Unavailable(
            "trace stories are not in unique ascending story-id order".to_owned(),
        );
    }
    let Some(kv_product) = corpus
        .geometry
        .residual_width
        .checked_mul(corpus.geometry.kv_heads)
    else {
        return InputCheck::Unavailable("trace kv-width arithmetic overflowed".to_owned());
    };
    if !kv_product.is_multiple_of(corpus.geometry.heads) {
        return InputCheck::Unavailable("trace kv-width is not integral".to_owned());
    }
    let kv_width = kv_product / corpus.geometry.heads;
    let decoded_records: usize = corpus.stories.iter().map(|story| story.steps.len()).sum();
    if corpus.records != decoded_records {
        return InputCheck::Unavailable(
            "RouteTraceCorpus record count disagrees with decoded stories".to_owned(),
        );
    }
    for (story_index, story) in corpus.stories.iter().enumerate() {
        if story.steps.is_empty()
            || story.steps.len() > ROUTE_MAX_CANDIDATES
            || story.tokens.len() != story.steps.len()
            || head.query_codes[story_index].len() != story.steps.len()
            || head.key_codes[story_index].len() != story.steps.len()
        {
            return InputCheck::Unavailable(
                "route-fit/trace story tables are empty, out of bounds, or misaligned".to_owned(),
            );
        }
        for (position, step) in story.steps.iter().enumerate() {
            if step.pos as usize != position
                || step.input_token != story.tokens[position]
                || step.q_rows.len() != corpus.declared_layers.len()
                || step.k_rows.len() != corpus.declared_layers.len()
                || step
                    .q_rows
                    .iter()
                    .any(|row| row.len() != corpus.geometry.residual_width)
                || step.k_rows.iter().any(|row| row.len() != kv_width)
                || step
                    .q_rows
                    .iter()
                    .chain(&step.k_rows)
                    .flatten()
                    .any(|value| !value.is_finite())
                || step.supports.len() <= lane_index
                || step.supports[lane_index].len() <= SCREEN_HEAD as usize
            {
                return InputCheck::Unavailable(
                    "trace position, q/k lanes, or attention-support alignment is incomplete"
                        .to_owned(),
                );
            }
            let support = &step.supports[lane_index][SCREEN_HEAD as usize];
            let mut ids: Vec<u32> = support.iter().map(|&(candidate, _)| candidate).collect();
            ids.sort_unstable();
            if ids.windows(2).any(|pair| pair[0] == pair[1])
                || ids.iter().any(|&candidate| candidate as usize > position)
                || support
                    .iter()
                    .any(|&(_, weight)| !weight.is_finite() || weight < 0.0)
                || support.windows(2).any(|pair| pair[0].1 < pair[1].1)
                || u32::try_from(position + 1).map_or(true, |candidate_count| {
                    candidate_count > expected_m && ids.len() != expected_m as usize
                })
            {
                return InputCheck::Unavailable(
                    "teacher top-M support is duplicate, non-causal, non-finite, out of order, or incomplete"
                        .to_owned(),
                );
            }
        }
    }
    if !corpus
        .stories
        .iter()
        .any(|story| story.steps.len() as u32 > expected_m)
    {
        return InputCheck::Unavailable(
            "trace has no eligible candidate_count > M step".to_owned(),
        );
    }

    let registered_fit = match fit_method_spec(&fitted.method.id, fitted.method.version) {
        Ok(record) => record,
        Err(_) => {
            return InputCheck::Unavailable(
                "fitted route method is not a registered route-fit record".to_owned(),
            );
        }
    };
    let profile_digest = profile.declared_digest();
    if manifest.format != FIT_MANIFEST_FORMAT
        || manifest.parameters != route_fit_v1_parameter_labels()
        || manifest.trace.as_deref() != Some(corpus.trace_kappa.as_str())
        || manifest.corpus.as_deref() != Some(corpus.records_kappa.as_str())
        || manifest
            .trace_profile
            .as_ref()
            .map(|record| record.declared_digest())
            != Some(profile_digest)
        || registered_fit != RouteFitMethod::route_fit_v1()
        || registered_fit != fitted.method
        || manifest.method != registered_fit
        || manifest.method.declared_digest() != fitted.method.declared_digest()
    {
        return InputCheck::Unavailable(
            "trace/record/profile/route-fit identities do not agree".to_owned(),
        );
    }
    let Some(fit_geometry) = manifest.geometry.as_ref() else {
        return InputCheck::Unavailable("route-fit geometry identity is absent".to_owned());
    };
    let Ok(source_width) = u32::try_from(corpus.geometry.residual_width) else {
        return InputCheck::Unavailable(
            "trace source width does not fit the registered geometry record".to_owned(),
        );
    };
    let Ok(route_width) = u32::try_from(ROUTE_CODE_BITS) else {
        return InputCheck::Unavailable("route width does not fit u32".to_owned());
    };
    let expected_fit_geometry =
        GeometryProjection::bucket_average(source_width.max(route_width), route_width);
    if projection_implementation(&fit_geometry.id, fit_geometry.version).is_err()
        || fit_geometry != &expected_fit_geometry
        || manifest.geometry_identity.as_deref()
            != Some(expected_fit_geometry.declared_digest().as_str())
    {
        return InputCheck::Unavailable(
            "route-fit geometry record/digest is unregistered or inconsistent".to_owned(),
        );
    }
    let Some(target_operator) = manifest.operator.as_ref() else {
        return InputCheck::Unavailable(
            "target r4-route-attention/1 identity is absent".to_owned(),
        );
    };
    let target_registered = match operator_spec(&target_operator.id, target_operator.version) {
        Ok(record) => record,
        Err(_) => {
            return InputCheck::Unavailable(
                "target attention operator is not a registered record".to_owned(),
            );
        }
    };
    if &target_registered != target_operator
        || target_operator.id != AttentionOperatorSpec::R4_ROUTE_ID
        || target_operator.version != AttentionOperatorSpec::R4_ROUTE_VERSION
        || manifest.operator_identity.as_deref() != Some(target_operator.declared_digest().as_str())
    {
        return InputCheck::Unavailable(
            "target attention operator record/digest is tampered or not r4-route-attention/1"
                .to_owned(),
        );
    }

    let source_operator_digest = match input.observation_manifest {
        None => None,
        Some(observation) => {
            if observation.identity_bundle_digest() != corpus.identity_bundle_digest
                || observation.trace_profile.as_ref() != Some(profile)
                || observation.total_records != corpus.records as u64
            {
                return InputCheck::Unavailable(
                    "observation manifest identity bundle/profile/record count does not bind the decoded trace"
                        .to_owned(),
                );
            }
            if let Some(geometry) = observation.geometry.as_ref() {
                let registered_geometry = GeometryProjection::bucket_average(
                    geometry.source_width,
                    geometry.compiled_width,
                );
                if projection_implementation(&geometry.id, geometry.version).is_err()
                    || geometry != &registered_geometry
                    || geometry.source_width != source_width
                    || geometry.compiled_width != route_width
                {
                    return InputCheck::Unavailable(
                        "observation geometry is not the registered corpus-source-to-288 record"
                            .to_owned(),
                    );
                }
            } else if kind == TraceKind::PinnedReal {
                return InputCheck::Unavailable(
                    "authoritative observation manifest has no geometry record".to_owned(),
                );
            }
            if let Some(adapter) = observation.tokenizer_adapter.as_ref() {
                if adapter_constructor(&adapter.family, adapter.version).is_err()
                    || !canonical_blake3(&adapter.tokenizer_cid)
                    || !canonical_blake3(&adapter.adapter_digest)
                    || adapter.adapter_digest != adapter.declared_digest()
                {
                    return InputCheck::Unavailable(
                        "observation tokenizer adapter record is unknown, noncanonical, or tampered"
                            .to_owned(),
                    );
                }
            }
            for value in [
                observation.input_cid.as_deref(),
                observation.source_manifest_kappa.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                if !canonical_blake3(value) {
                    return InputCheck::Unavailable(
                        "observation manifest carries a noncanonical content identity".to_owned(),
                    );
                }
            }
            match observation.attention_operator.as_ref() {
                None => None,
                Some(source_operator) => {
                    let registered =
                        match operator_spec(&source_operator.id, source_operator.version) {
                            Ok(record) => record,
                            Err(_) => {
                                return InputCheck::Unavailable(
                                    "source attention operator is not a registered record"
                                        .to_owned(),
                                );
                            }
                        };
                    if &registered != source_operator
                        || source_operator.id == AttentionOperatorSpec::R4_ROUTE_ID
                    {
                        return InputCheck::Unavailable(
                            "source attention operator record is tampered or names the target operator"
                                .to_owned(),
                        );
                    }
                    Some(source_operator.declared_digest())
                }
            }
        }
    };

    if kind == TraceKind::PinnedReal {
        let Some(observation) = input.observation_manifest else {
            return InputCheck::Unavailable(
                "pinned real trace lacks its authoritative observation manifest".to_owned(),
            );
        };
        if source_operator_digest.is_none()
            || !nonempty(&manifest.source_snapshot)
            || !nonempty(&manifest.tokenizer)
            || !nonempty(&manifest.adapter)
            || !nonempty(&manifest.compiler)
            || corpus.identity_bundle_digest.is_empty()
            || corpus.records_kappa.is_empty()
            || corpus.trace_kappa.is_empty()
            || !nonempty(&observation.source_manifest_kappa)
            || !nonempty(&observation.input_cid)
            || observation.geometry.is_none()
            || observation.tokenizer_adapter.is_none()
        {
            return InputCheck::Unavailable(
                "pinned real trace lacks a complete source/tokenizer/adapter/source-operator/corpus/profile/record/trace identity"
                    .to_owned(),
                );
        }
        let tokenizer = match observation.tokenizer_adapter.as_ref() {
            Some(adapter) => adapter,
            None => {
                return InputCheck::Unavailable(
                    "pinned real trace lost its required tokenizer adapter".to_owned(),
                )
            }
        };
        let tokenizer_adapter_digest = tokenizer.declared_digest();
        if manifest.tokenizer.as_deref() != Some(tokenizer_adapter_digest.as_str()) {
            return InputCheck::Unavailable(
                "route-fit tokenizer identity does not match the observation tokenizer adapter"
                    .to_owned(),
            );
        }
        for value in [
            manifest.source_snapshot.as_deref(),
            manifest.tokenizer.as_deref(),
            observation.input_cid.as_deref(),
            observation.source_manifest_kappa.as_deref(),
            Some(corpus.identity_bundle_digest.as_str()),
            Some(corpus.records_kappa.as_str()),
            Some(corpus.trace_kappa.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            if !canonical_blake3(value) {
                return InputCheck::Unavailable(
                    "pinned real trace carries a noncanonical content identity".to_owned(),
                );
            }
        }
        let Some(registered_evidence) = registered_evidence else {
            return InputCheck::Unavailable(
                "trace evidence is not a registered #722 evidence-kind record".to_owned(),
            );
        };
        if registered_evidence != input.evidence
            || registered_evidence.kind() != TraceKind::PinnedReal
        {
            return InputCheck::Unavailable(
                "evidence-kind registry mismatch: caller-selected TraceKind cannot relabel evidence"
                    .to_owned(),
            );
        }
        let fit_manifest_kappa = manifest.kappa();
        let fitted_params_kappa = fitted.kappa();
        if registered_evidence.observation_identity_bundle_digest
            != Some(corpus.identity_bundle_digest.as_str())
            || registered_evidence.records_kappa != Some(corpus.records_kappa.as_str())
            || registered_evidence.trace_kappa != Some(corpus.trace_kappa.as_str())
            || registered_evidence.fit_manifest_kappa != Some(fit_manifest_kappa.as_str())
            || registered_evidence.fitted_params_kappa != Some(fitted_params_kappa.as_str())
        {
            return InputCheck::Unavailable(
                "registered empirical evidence identities do not exactly bind observation, records, trace, fit manifest, and fitted parameters"
                    .to_owned(),
            );
        }
        // `FittedRouteCodes` is a typed artifact but its fields are public for
        // fixture construction. Recompute route-fit/1 at the empirical
        // boundary so a fabricated code table cannot borrow a pinned κ.
        match fit_route_codes(corpus) {
            Ok(recomputed) if &recomputed == fitted => {}
            Ok(_) | Err(_) => {
                return InputCheck::Unavailable(
                    "fitted route codes do not recompute exactly under registered route-fit/1"
                        .to_owned(),
                );
            }
        }
    } else if registered_evidence != Some(input.evidence)
        || input.evidence.kind() != TraceKind::InstrumentConformance
    {
        return InputCheck::Unavailable(
            "instrument trace evidence is not the full registered evidence-kind record".to_owned(),
        );
    }
    InputCheck::Ready {
        input,
        lane_index,
        source_operator_digest,
    }
}

fn report_identities(
    input: Option<OcteractTraceInput<'_>>,
    source_operator_digest: Option<String>,
) -> ScreenIdentities {
    let mut identities = ScreenIdentities {
        octeract_source_sha256: OCTERACT_CYPHER_SOURCE.sha256.to_owned(),
        validation_source_sha256: OCTERACT_VALIDATION_SOURCE.sha256.to_owned(),
        ..ScreenIdentities::default()
    };
    let Some(input) = input else {
        return identities;
    };
    identities.observation_identity_bundle_digest =
        Some(input.corpus.identity_bundle_digest.clone());
    identities.observation_input_cid = input
        .observation_manifest
        .and_then(|manifest| manifest.input_cid.clone());
    identities.source_manifest_kappa = input
        .observation_manifest
        .and_then(|manifest| manifest.source_manifest_kappa.clone());
    identities.records_kappa = Some(input.corpus.records_kappa.clone());
    identities.trace_kappa = Some(input.corpus.trace_kappa.clone());
    identities.trace_profile_digest = Some(input.corpus.trace_profile.declared_digest());
    identities.route_fit_digest = Some(input.fitted.method.declared_digest());
    identities.fit_manifest_kappa = Some(input.fit_manifest.kappa());
    identities.fitted_params_kappa = Some(input.fitted.kappa());
    identities.source_snapshot = input.fit_manifest.source_snapshot.clone();
    identities.source_geometry_digest = input
        .observation_manifest
        .and_then(|manifest| manifest.geometry.as_ref())
        .map(GeometryProjection::declared_digest);
    identities.tokenizer_cid = input
        .observation_manifest
        .and_then(|manifest| manifest.tokenizer_adapter.as_ref())
        .map(|adapter| adapter.tokenizer_cid.clone());
    identities.tokenizer_adapter_digest = input
        .observation_manifest
        .and_then(|manifest| manifest.tokenizer_adapter.as_ref())
        .map(|adapter| adapter.adapter_digest.clone());
    identities.tokenizer = input.fit_manifest.tokenizer.clone();
    identities.adapter = input.fit_manifest.adapter.clone();
    identities.target_operator_digest = input.fit_manifest.operator_identity.clone();
    identities.source_attention_operator_digest = source_operator_digest;
    identities.compiler = input.fit_manifest.compiler.clone();
    identities.trace_evidence_id = Some(input.evidence.id().to_owned());
    identities.trace_evidence_version = Some(input.evidence.version());
    identities.trace_evidence_kind = Some(input.evidence.kind());
    identities.trace_evidence_digest = Some(input.evidence.declared_digest());
    identities
}

fn unavailable_report(
    contract: OcteractTraceContract,
    kind: TraceKind,
    identities: ScreenIdentities,
    reason: String,
) -> OcteractTraceReport {
    let mut arms = Vec::with_capacity(contract.arms.len());
    for (index, declaration) in contract.arms.iter().enumerate() {
        arms.push(ArmReport {
            id: declaration.id.clone(),
            can_advance: declaration.can_advance,
            verdict: if index == 0 {
                StageVerdict::Unavailable
            } else {
                StageVerdict::NotRun
            },
            reason: if index == 0 {
                reason.clone()
            } else {
                "NOT_RUN after unavailable prerequisite/instrument".to_owned()
            },
            metrics: ArmMetrics::default(),
        });
    }
    let nulls = contract
        .nulls
        .iter()
        .map(|declaration| ArmReport {
            id: declaration.id.clone(),
            can_advance: false,
            verdict: StageVerdict::NotRun,
            reason: "NOT_RUN after unavailable prerequisite/instrument".to_owned(),
            metrics: ArmMetrics::default(),
        })
        .collect();
    let mut report = OcteractTraceReport {
        format: OCTERACT_TRACE_REPORT_FORMAT.to_owned(),
        contract,
        trace_kind: kind,
        identities,
        controls: ScreenControls {
            frame: "UNAVAILABLE".to_owned(),
            ..ScreenControls::default()
        },
        shared_metrics: ScreenSharedMetrics::default(),
        arms,
        nulls,
        disposition: ScreenDisposition::Unavailable,
        disposition_reason: reason,
        payload_kappa: String::new(),
    };
    report.payload_kappa = octeract_trace_payload_kappa(&report);
    report
}

fn arm_report(
    id: &str,
    can_advance: bool,
    verdict: StageVerdict,
    reason: &str,
    metrics: ArmMetrics,
) -> ArmReport {
    ArmReport {
        id: id.to_owned(),
        can_advance,
        verdict,
        reason: reason.to_owned(),
        metrics,
    }
}

/// Execute the locked screen over an explicitly supplied trace/fit/manifest.
/// `None` emits a deterministic typed-UNAVAILABLE report and never searches
/// the filesystem or silently selects a fixture.
pub fn run_octeract_trace_screen(
    input: Option<OcteractTraceInput<'_>>,
    kind: TraceKind,
) -> OcteractTraceReport {
    let contract = preregistered_octeract_trace_contract();
    let Some(supplied) = input else {
        return unavailable_report(
            contract,
            kind,
            report_identities(None, None),
            "required explicitly supplied pinned full/1 trace input is absent".to_owned(),
        );
    };
    let checked = validate_input(supplied, kind);
    let (input, lane_index, source_operator_digest) = match checked {
        InputCheck::Ready {
            input,
            lane_index,
            source_operator_digest,
        } => (input, lane_index, source_operator_digest),
        InputCheck::Unavailable(reason) => {
            return unavailable_report(
                contract,
                kind,
                report_identities(Some(supplied), None),
                reason,
            );
        }
    };
    let identities = report_identities(Some(input), source_operator_digest);
    let Some(head) = input.fitted.head(SCREEN_LAYER, SCREEN_HEAD) else {
        return unavailable_report(
            contract,
            kind,
            identities,
            "validated route-fit input lost layer 0 head 0".to_owned(),
        );
    };
    let m = input.fitted.top_m as usize;

    let mut exact_weight = vec![0u64; 9];
    let mut folded_shell = vec![0u64; 5];
    let mut exact_per_block = vec![vec![0u64; 9]; ROUTE_CODE_BYTES];
    let mut fold_per_block = vec![vec![0u64; 5]; ROUTE_CODE_BYTES];
    let mut eligible_story = BTreeMap::<u32, ()>::new();
    let mut counts = ScreenCounts::default();
    let mut prefilter_eligible_story = BTreeMap::<u32, ()>::new();
    let mut prefilter_counts = ScreenCounts::default();
    let mut deranged_counts = ScreenCounts::default();
    let mut collision = CollisionMetrics::default();
    let mut group_sizes = BTreeMap::<u32, u64>::new();
    let mut oracle_recall_sum = 0.0;
    let mut oracle_jaccard_sum = 0.0;
    let mut oracle_steps = 0u64;
    let mut shortlist = ShortlistMetrics::default();
    let mut prefilter = PrefilterMetrics::default();
    let mut lower = LowerBoundMetrics::default();
    let mut occupancy_null_occupancy = OccupancyAccumulator::new();
    let mut shuffled_null_occupancy = OccupancyAccumulator::new();

    let mut v1 = MetricAccumulator::new();
    let mut weight9 = MetricAccumulator::new();
    let mut fold5 = MetricAccumulator::new();
    let mut oriented = MetricAccumulator::new();
    let mut prefilter_selection = MetricAccumulator::new();
    let mut lower_selection = MetricAccumulator::new();
    let mut occupancy_null = MetricAccumulator::new();
    let mut shuffled_null = MetricAccumulator::new();
    let mut deranged_null = MetricAccumulator::new();
    let mut frame_deranged = MetricAccumulator::new();
    let mut prefilter_occupancy_null = MetricAccumulator::new();
    let mut prefilter_shuffled_null = MetricAccumulator::new();
    let mut prefilter_deranged_null = MetricAccumulator::new();

    let permutation = shuffled_block_permutation(SHUFFLED_BLOCK_SEED);
    let mut scalar_operator_exact = true;
    let mut weight9_exact = true;
    let mut oriented_exact = true;
    let mut lower_bound_exact = true;
    let mut anchor_relabel_invariant = true;
    let mut anchor_relabels_checked = 0u32;
    let mut occupancy_null_transformation_distinct = false;
    let mut occupancy_null_result_distinct = false;
    let mut shuffled_block_null_transformation_distinct = false;
    let mut shuffled_block_null_result_distinct = false;
    let mut deranged_support_transformation_distinct = false;

    for (story_index, story) in input.corpus.stories.iter().enumerate() {
        let story_supports: Vec<Vec<u32>> = story
            .steps
            .iter()
            .map(|step| {
                step.supports[lane_index][SCREEN_HEAD as usize]
                    .iter()
                    .map(|&(candidate, _)| candidate)
                    .collect()
            })
            .collect();
        let deranged = deranged_supports(&story_supports);
        for position in 0..story.steps.len() {
            let query = &head.query_codes[story_index][position];
            let keys = &head.key_codes[story_index][..=position];
            let step_m = m.min(keys.len());
            let Some(scored) = score_octeract_step(query, keys, step_m) else {
                scalar_operator_exact = false;
                continue;
            };
            scalar_operator_exact &= route_seam_matches(query, keys, step_m, &scored.v1_selected);
            weight9_exact &= scored.weight9_blocks == scored.oriented_weights
                && scored.weight9_scores == scored.exact_scores
                && scored.weight9_selected == scored.v1_selected;
            oriented_exact &= scored.oriented_scores == scored.exact_scores
                && scored.oriented_selected == scored.v1_selected;
            lower_bound_exact &= scored.lower_bound.selected == scored.v1_selected
                && scored
                    .lower_bound
                    .lower_bounds
                    .iter()
                    .zip(&scored.exact_scores)
                    .all(|(&bound, &exact)| bound <= exact);

            if keys.len() <= m {
                continue;
            }
            let (step_anchor_invariant, step_relabels_checked) =
                anchor_relabel_control_result(&scored, step_m);
            anchor_relabel_invariant &= step_anchor_invariant;
            anchor_relabels_checked = anchor_relabels_checked.max(step_relabels_checked);
            eligible_story.insert(story.story, ());
            counts.eligible_steps += 1;
            counts.candidates += keys.len() as u64;
            let teacher = &story_supports[position];
            counts.teacher_support_entries += teacher.len() as u64;
            deranged_counts.eligible_steps += 1;
            deranged_counts.candidates += keys.len() as u64;
            deranged_counts.teacher_support_entries += deranged[position].len() as u64;

            for (candidate, classes) in scored.fold_classes.iter().enumerate() {
                for (block, &class) in classes.iter().enumerate() {
                    let distance = usize::from(scored.weight9_blocks[candidate][block]);
                    exact_weight[distance] += 1;
                    folded_shell[class as usize] += 1;
                    exact_per_block[block][distance] += 1;
                    fold_per_block[block][class as usize] += 1;
                }
            }

            v1.record(
                &scored.exact_scores,
                &scored.oriented_weights,
                &scored.v1_selected,
                teacher,
            );
            weight9.record(
                &scored.weight9_scores,
                &scored.weight9_blocks,
                &scored.weight9_selected,
                teacher,
            );
            fold5.record(
                &scored.folded_scores,
                &scored.fold_classes,
                &scored.folded_selected,
                teacher,
            );
            oriented.record(
                &scored.oriented_scores,
                &scored.oriented_weights,
                &scored.oriented_selected,
                teacher,
            );
            let lower_classes: Vec<[u8; ROUTE_CODE_BYTES]> = keys
                .iter()
                .map(|key| {
                    let mut classes = [0u8; ROUTE_CODE_BYTES];
                    for (block, (&query_byte, &key_byte)) in query.iter().zip(key).enumerate() {
                        classes[block] = masked_weight_lower_bound(query_byte, key_byte, u8::MAX);
                    }
                    classes
                })
                .collect();
            lower_selection.record(
                &scored.lower_bound.lower_bounds,
                &lower_classes,
                &scored.lower_bound.selected,
                teacher,
            );

            let v1_ids = selection_ids(&scored.v1_selected);

            let occupancy_classes = occupancy_matched_fold_null(
                &scored.fold_classes,
                OCCUPANCY_MATCHED_FOLD_SEED ^ (u64::from(story.story) << 32) ^ position as u64,
            );
            let occupancy_scores: Vec<u32> = occupancy_classes
                .iter()
                .map(|classes| classes.iter().map(|&value| u32::from(value)).sum())
                .collect();
            let occupancy_selected = selected_from_scores(&occupancy_scores, m);
            occupancy_null_transformation_distinct |= occupancy_classes != scored.fold_classes;
            occupancy_null_result_distinct |= occupancy_scores != scored.folded_scores
                || occupancy_selected != scored.folded_selected;
            occupancy_null_occupancy.record_fold(&occupancy_classes);
            occupancy_null.record(
                &occupancy_scores,
                &occupancy_classes,
                &occupancy_selected,
                teacher,
            );

            let mut shuffled_classes = Vec::with_capacity(keys.len());
            let mut shuffled_blocks = Vec::with_capacity(keys.len());
            for key in keys {
                let mut classes = [0u8; ROUTE_CODE_BYTES];
                let mut blocks = [0u8; ROUTE_CODE_BYTES];
                for block in 0..ROUTE_CODE_BYTES {
                    let distance = masked_byte_distance(
                        query[block],
                        key[usize::from(permutation[block])],
                        u8::MAX,
                    );
                    blocks[block] = distance;
                    classes[block] = folded_class(bounded_block(distance));
                }
                shuffled_blocks.push(blocks);
                shuffled_classes.push(classes);
            }
            let shuffled_scores: Vec<u32> = shuffled_classes
                .iter()
                .map(|classes| classes.iter().map(|&value| u32::from(value)).sum())
                .collect();
            let shuffled_selected = selected_from_scores(&shuffled_scores, m);
            shuffled_block_null_transformation_distinct |= permutation
                .iter()
                .enumerate()
                .any(|(index, &value)| index != usize::from(value))
                && (shuffled_blocks != scored.weight9_blocks
                    || shuffled_classes != scored.fold_classes);
            shuffled_block_null_result_distinct |= shuffled_scores != scored.folded_scores
                || shuffled_selected != scored.folded_selected;
            shuffled_null_occupancy.record_exact_and_fold(&shuffled_blocks, &shuffled_classes);
            shuffled_null.record(
                &shuffled_scores,
                &shuffled_classes,
                &shuffled_selected,
                teacher,
            );
            deranged_null.record(
                &scored.folded_scores,
                &scored.fold_classes,
                &scored.folded_selected,
                &deranged[position],
            );
            deranged_support_transformation_distinct |=
                deranged[position].as_slice() != teacher.as_slice();
            frame_deranged.record(
                &scored.exact_scores,
                &scored.oriented_weights,
                &scored.v1_selected,
                &deranged[position],
            );

            if !scored.prefilter.shortlist.is_empty() {
                prefilter_eligible_story.insert(story.story, ());
                prefilter_counts.eligible_steps += 1;
                prefilter_counts.candidates += keys.len() as u64;
                prefilter_counts.teacher_support_entries += teacher.len() as u64;
                prefilter.work_eligible_steps += 1;
                prefilter.total_candidates += keys.len() as u64;
                prefilter.exact_refinement_candidates +=
                    u64::from(scored.prefilter.exact_refinement_candidates);
                prefilter.exact_refinement_candidates_avoided +=
                    u64::from(scored.prefilter.exact_refinement_candidates_avoided);
                let selected = selection_ids(&scored.prefilter.selected);
                prefilter.mean_v1_recall += recall(&selected, &v1_ids);
                prefilter.mean_v1_jaccard += jaccard(&selected, &v1_ids);
                prefilter_selection.record(
                    &scored.folded_scores,
                    &scored.fold_classes,
                    &scored.prefilter.selected,
                    teacher,
                );
                let occupancy_prefilter =
                    prefilter_step(&scored.exact_scores, &occupancy_scores, m);
                prefilter_occupancy_null.record(
                    &occupancy_scores,
                    &occupancy_classes,
                    &occupancy_prefilter.selected,
                    teacher,
                );
                let shuffled_prefilter = prefilter_step(&scored.exact_scores, &shuffled_scores, m);
                prefilter_shuffled_null.record(
                    &shuffled_scores,
                    &shuffled_classes,
                    &shuffled_prefilter.selected,
                    teacher,
                );
                prefilter_deranged_null.record(
                    &scored.folded_scores,
                    &scored.fold_classes,
                    &scored.prefilter.selected,
                    &deranged[position],
                );
                let shortlist_ids = &scored.prefilter.shortlist;
                for candidate in 0..keys.len() as u32 {
                    let in_v1 = v1_ids.contains(&candidate);
                    let in_shortlist = shortlist_ids.contains(&candidate);
                    shortlist.false_positives += u64::from(in_shortlist && !in_v1);
                    shortlist.false_negatives += u64::from(in_v1 && !in_shortlist);
                }
            }
            lower.candidates += keys.len() as u64;
            lower.tight_candidates += u64::from(scored.lower_bound.tight_candidates);
            lower.exact_evaluations += u64::from(scored.lower_bound.exact_evaluations);
            lower.exact_evaluations_avoided +=
                u64::from(scored.lower_bound.exact_evaluations_avoided);

            let mut fold_groups = BTreeMap::<[u8; ROUTE_CODE_BYTES], Vec<usize>>::new();
            let mut exact_groups = BTreeMap::<[u8; ROUTE_CODE_BYTES], Vec<usize>>::new();
            for candidate in 0..keys.len() {
                fold_groups
                    .entry(scored.fold_classes[candidate])
                    .or_default()
                    .push(candidate);
                exact_groups
                    .entry(scored.oriented_weights[candidate])
                    .or_default()
                    .push(candidate);
            }
            collision.exact_collision_pairs += exact_groups
                .values()
                .map(|members| choose_two(members.len()))
                .sum::<u64>();
            let mut oracle_groups = Vec::with_capacity(fold_groups.len());
            for members in fold_groups.values() {
                if members.len() > 1 {
                    collision.complete_fold_signature_groups += 1;
                    *group_sizes.entry(members.len() as u32).or_default() += 1;
                }
                let mut lossy = false;
                for left in 0..members.len() {
                    for right in left + 1..members.len() {
                        if scored.oriented_weights[members[left]]
                            != scored.oriented_weights[members[right]]
                        {
                            collision.complement_collision_pairs += 1;
                            lossy = true;
                        }
                        let left_member = teacher.contains(&(members[left] as u32));
                        let right_member = teacher.contains(&(members[right] as u32));
                        collision.within_group_membership_disagreements +=
                            u64::from(left_member != right_member);
                    }
                }
                collision.lossy_signature_groups += u64::from(lossy);
                oracle_groups.push(CollisionGroup {
                    candidate_ids: members.iter().map(|&value| value as u32).collect(),
                    teacher_membership: members
                        .iter()
                        .map(|&value| teacher.contains(&(value as u32)))
                        .collect(),
                });
            }
            if let Some(oracle) = collision_oracle(&oracle_groups, m) {
                let hits = oracle.max_teacher_hits as usize;
                oracle_recall_sum += if teacher.is_empty() {
                    0.0
                } else {
                    hits as f64 / teacher.len() as f64
                };
                oracle_jaccard_sum += hits as f64 / (m + teacher.len() - hits) as f64;
                oracle_steps += 1;
            }
        }
    }
    counts.eligible_stories = eligible_story.len() as u64;
    deranged_counts.eligible_stories = counts.eligible_stories;
    prefilter_counts.eligible_stories = prefilter_eligible_story.len() as u64;

    let occupancy = OccupancyMetrics {
        exact_entropy_bits: entropy(&exact_weight),
        folded_entropy_bits: entropy(&folded_shell),
        exact_weight,
        folded_shell,
        exact_weight_per_block: exact_per_block,
        folded_shell_per_block: fold_per_block,
    };
    let occupancy_null_occupancy = occupancy_null_occupancy.finish_fold();
    let shuffled_null_occupancy = shuffled_null_occupancy.finish();
    collision.group_size_distribution = group_sizes.into_iter().collect();
    if prefilter.work_eligible_steps != 0 {
        let denominator = prefilter.work_eligible_steps as f64;
        prefilter.mean_v1_recall /= denominator;
        prefilter.mean_v1_jaccard /= denominator;
    }
    if prefilter.total_candidates != 0 {
        prefilter.exact_refinement_fraction =
            prefilter.exact_refinement_candidates as f64 / prefilter.total_candidates as f64;
    }
    if lower.candidates != 0 {
        lower.tight_fraction = lower.tight_candidates as f64 / lower.candidates as f64;
    }
    let prefilter_occupancy_null_selection = prefilter_occupancy_null.finish();
    let prefilter_shuffled_null_selection = prefilter_shuffled_null.finish();
    let prefilter_deranged_null_selection = prefilter_deranged_null.finish();
    prefilter.occupancy_null_teacher_jaccard =
        prefilter_occupancy_null_selection.mean_teacher_jaccard;
    prefilter.shuffled_block_null_teacher_jaccard =
        prefilter_shuffled_null_selection.mean_teacher_jaccard;
    prefilter.deranged_support_null_teacher_jaccard =
        prefilter_deranged_null_selection.mean_teacher_jaccard;
    let oracle = OracleMetrics {
        mean_max_recall: if oracle_steps == 0 {
            0.0
        } else {
            oracle_recall_sum / oracle_steps as f64
        },
        mean_max_jaccard: if oracle_steps == 0 {
            0.0
        } else {
            oracle_jaccard_sum / oracle_steps as f64
        },
    };
    let shared_metrics = ScreenSharedMetrics {
        counts: counts.clone(),
        occupancy,
    };

    let v1_selection = v1.finish();
    let weight_selection = weight9.finish();
    let fold_selection = fold5.finish();
    let oriented_selection = oriented.finish();
    let prefilter_arm_selection = prefilter_selection.finish();
    let lower_arm_selection = lower_selection.finish();
    let occupancy_null_selection = occupancy_null.finish();
    let shuffled_null_selection = shuffled_null.finish();
    let deranged_null_selection = deranged_null.finish();
    let frame_deranged_selection = frame_deranged.finish();

    let frame = frame_control_default(v1_selection.mean_teacher_jaccard);
    let deranged_distinct =
        (v1_selection.mean_teacher_jaccard - frame_deranged_selection.mean_teacher_jaccard).abs()
            > crate::frame_consistency::FRAME_CONTROL_EPSILON;
    anchor_relabel_invariant &= anchor_relabels_checked == ANCHOR_RELABELINGS;
    let structural_controls = scalar_operator_exact
        && weight9_exact
        && oriented_exact
        && lower_bound_exact
        && anchor_relabel_invariant
        && v1.steps != 0;
    let all_nulls_nonvacuous = deranged_support_transformation_distinct
        && deranged_distinct
        && occupancy_null_transformation_distinct
        && occupancy_null_result_distinct
        && shuffled_block_null_transformation_distinct
        && shuffled_block_null_result_distinct;
    let instrument_valid =
        structural_controls && frame == FrameControl::Framed && all_nulls_nonvacuous;
    let controls = ScreenControls {
        v1_scalar_operator_exact: scalar_operator_exact,
        weight9_exact,
        oriented_exact,
        lower_bound_exact,
        anchor_relabel_invariant,
        anchor_relabels_checked,
        matched_pair_mean_jaccard: v1_selection.mean_teacher_jaccard,
        frame: match frame {
            FrameControl::Framed => "FRAMED".to_owned(),
            FrameControl::MismatchSuspected => {
                "UNAVAILABLE(frame/instrument mismatch suspected)".to_owned()
            }
        },
        deranged_support_transformation_distinct,
        deranged_support_observably_distinct: deranged_distinct,
        occupancy_null_transformation_distinct,
        occupancy_null_result_distinct,
        shuffled_block_null_transformation_distinct,
        shuffled_block_null_result_distinct,
        instrument_valid,
    };
    if !instrument_valid {
        let reason = "UNAVAILABLE(frame/instrument mismatch suspected): V1/weight9/oriented/lower/anchor identity, matched-pair frame, or a null anti-vacuity control failed";
        let mut report = unavailable_report(contract, kind, identities, reason.to_owned());
        report.controls = controls;
        report.shared_metrics = shared_metrics.clone();
        report.arms = vec![
            arm_report(
                ARM_V1,
                false,
                StageVerdict::Unavailable,
                reason,
                ArmMetrics {
                    counts: Some(counts.clone()),
                    selection: Some(v1_selection),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                ARM_WEIGHT9,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    selection: Some(weight_selection),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                ARM_FOLD5,
                true,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    selection: Some(fold_selection),
                    collisions: Some(collision),
                    oracle: Some(oracle),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                ARM_ORIENTED,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    selection: Some(oriented_selection),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                ARM_PREFILTER,
                true,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(prefilter_counts.clone()),
                    selection: Some(prefilter_arm_selection),
                    shortlist: Some(shortlist),
                    prefilter: Some(prefilter),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                ARM_LOWER_BOUND,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    selection: Some(lower_arm_selection),
                    lower_bound: Some(lower),
                    ..ArmMetrics::default()
                },
            ),
        ];
        report.nulls = vec![
            arm_report(
                NULL_OCCUPANCY,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    fold_occupancy: Some(occupancy_null_occupancy),
                    selection: Some(occupancy_null_selection),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                NULL_SHUFFLED_BLOCK,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(counts.clone()),
                    occupancy: Some(shuffled_null_occupancy),
                    selection: Some(shuffled_null_selection),
                    ..ArmMetrics::default()
                },
            ),
            arm_report(
                NULL_DERANGED_SUPPORT,
                false,
                StageVerdict::NotRun,
                "NOT_RUN after unavailable frame/instrument",
                ArmMetrics {
                    counts: Some(deranged_counts.clone()),
                    selection: Some(deranged_null_selection),
                    ..ArmMetrics::default()
                },
            ),
        ];
        report.payload_kappa = octeract_trace_payload_kappa(&report);
        return report;
    }

    let baseline_jaccard = v1_selection.mean_teacher_jaccard;
    let null_max = occupancy_null_selection
        .mean_teacher_jaccard
        .max(shuffled_null_selection.mean_teacher_jaccard)
        .max(deranged_null_selection.mean_teacher_jaccard);
    let prefilter_null_max = prefilter_occupancy_null_selection
        .mean_teacher_jaccard
        .max(prefilter_shuffled_null_selection.mean_teacher_jaccard)
        .max(prefilter_deranged_null_selection.mean_teacher_jaccard);
    let direct_gate = oracle.mean_max_jaccard
        >= baseline_jaccard + contract.thresholds.direct_jaccard_margin
        && fold_selection.mean_teacher_jaccard
            >= baseline_jaccard + contract.thresholds.direct_jaccard_margin
        && fold_selection.mean_teacher_jaccard > null_max;
    let prefilter_gate = prefilter.work_eligible_steps != 0
        && prefilter.mean_v1_recall >= contract.thresholds.prefilter_v1_recall
        && prefilter.exact_refinement_fraction <= contract.thresholds.prefilter_refinement_fraction
        && prefilter_arm_selection.mean_teacher_jaccard > prefilter_null_max;
    let classifier_gate = |gate_pass| CandidateGate {
        controls_pass: structural_controls,
        frame_score: baseline_jaccard,
        deranged_distinct,
        gate_pass,
    };
    let direct_verdict = classify_candidate_arm(kind, classifier_gate(direct_gate), false);
    let prefilter_verdict = classify_candidate_arm(kind, classifier_gate(prefilter_gate), false);
    let control_verdict = if kind == TraceKind::InstrumentConformance {
        StageVerdict::NotRun
    } else {
        StageVerdict::Pass
    };
    let control_reason = if kind == TraceKind::InstrumentConformance {
        "instrument-conformance metric recorded; empirical verdict is NOT_RUN"
    } else {
        "structural conformance control passed"
    };
    let candidate_reason = |verdict: StageVerdict, passed: bool| match verdict {
        StageVerdict::Pass => "valid pinned-real arm cleared every locked gate",
        StageVerdict::Fail if passed => "valid pinned-real arm reached a contradictory gate state",
        StageVerdict::Fail => {
            "valid pinned-real arm missed at least one locked reachability/null/fidelity/work gate"
        }
        StageVerdict::Unavailable => "UNAVAILABLE(frame/instrument mismatch suspected)",
        StageVerdict::NotRun => "instrument-conformance only; empirical PASS/FAIL is NOT_RUN",
    };
    let arms = vec![
        arm_report(
            ARM_V1,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts.clone()),
                selection: Some(v1_selection),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            ARM_WEIGHT9,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts.clone()),
                selection: Some(weight_selection),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            ARM_FOLD5,
            true,
            direct_verdict,
            candidate_reason(direct_verdict, direct_gate),
            ArmMetrics {
                counts: Some(counts.clone()),
                selection: Some(fold_selection),
                collisions: Some(collision),
                oracle: Some(oracle),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            ARM_ORIENTED,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts.clone()),
                selection: Some(oriented_selection),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            ARM_PREFILTER,
            true,
            prefilter_verdict,
            candidate_reason(prefilter_verdict, prefilter_gate),
            ArmMetrics {
                counts: Some(prefilter_counts),
                selection: Some(prefilter_arm_selection),
                shortlist: Some(shortlist),
                prefilter: Some(prefilter),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            ARM_LOWER_BOUND,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts.clone()),
                selection: Some(lower_arm_selection),
                lower_bound: Some(lower),
                ..ArmMetrics::default()
            },
        ),
    ];
    let nulls = vec![
        arm_report(
            NULL_OCCUPANCY,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts.clone()),
                fold_occupancy: Some(occupancy_null_occupancy),
                selection: Some(occupancy_null_selection),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            NULL_SHUFFLED_BLOCK,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(counts),
                occupancy: Some(shuffled_null_occupancy),
                selection: Some(shuffled_null_selection),
                ..ArmMetrics::default()
            },
        ),
        arm_report(
            NULL_DERANGED_SUPPORT,
            false,
            control_verdict,
            control_reason,
            ArmMetrics {
                counts: Some(deranged_counts),
                selection: Some(deranged_null_selection),
                ..ArmMetrics::default()
            },
        ),
    ];
    let (disposition, disposition_reason) = if kind == TraceKind::InstrumentConformance {
        (
            ScreenDisposition::Unavailable,
            "instrument-conformance completed; required pinned real trace row remains UNAVAILABLE"
                .to_owned(),
        )
    } else if direct_verdict == StageVerdict::Pass || prefilter_verdict == StageVerdict::Pass {
        (
            ScreenDisposition::Advance,
            "at least one promotable Octeract arm cleared every locked gate".to_owned(),
        )
    } else {
        (
            ScreenDisposition::StopNegative,
            "valid pinned-real screen ran and no promotable Octeract arm cleared".to_owned(),
        )
    };
    let mut report = OcteractTraceReport {
        format: OCTERACT_TRACE_REPORT_FORMAT.to_owned(),
        contract,
        trace_kind: kind,
        identities,
        controls,
        shared_metrics,
        arms,
        nulls,
        disposition,
        disposition_reason,
        payload_kappa: String::new(),
    };
    report.payload_kappa = octeract_trace_payload_kappa(&report);
    report
}
