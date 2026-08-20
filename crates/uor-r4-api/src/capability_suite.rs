//! Versioned capability-suite constitution and per-token resolution
//! attribution schema (#832, item D of S0 tracker #821).
//!
//! Schema and structural validation ONLY — this module mirrors the
//! schema-first shape of [`crate::release_bundle`] (#655-C0): it defines
//! the versioned records the evaluation programme commits and the pure
//! checks that keep a report honest, but it does **not** run an
//! evaluation, load a corpus, or serve a model. It is filesystem-free at
//! the library edge; the committed manifests are embedded with
//! `include_str!`, so [`builtin_manifests`] and [`builtin_constitution`]
//! parse the committed constitution with zero I/O.
//!
//! Why one constitution (the #832 problem). Gate C, teacher parity,
//! canaries, OOD probes, and corpus replay answer different questions on
//! different slices; some suites skip when a fixture is absent; and an
//! aggregate score hides which serving path produced each token. Without
//! document-disjoint partitions, pinned identities, powered samples,
//! declared controls, and per-token path attribution, a later
//! "improvement" can be leakage, ExactContext dominance, a decoder
//! effect, a vacuous control, or benchmark drift. The types here make
//! each of those failure modes a schema-level, testable object.
//!
//! Execution scope. Offline evaluation/certification plus measured
//! reachability to the normative deployed [`crate::engine::R4Engine`]
//! path. A production token is attributed to a [`ResolutionPath`] and the
//! normative scorer identity ([`NORMATIVE_SCORER_ID`], designated by
//! ADR-0001 / #831); evidence outside this scope is not credited as a
//! deployed-serving result. Claim language follows
//! `docs/formal_vocabulary.md`; nothing here strengthens or weakens the
//! guarantees of the underlying crates.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::PolicyStatus;

/// Current capability-suite *manifest* schema version this crate writes
/// and accepts. A non-additive field change bumps this and documents the
/// migration here (mirrors [`crate::release_bundle::RELEASE_BUNDLE_MANIFEST_SCHEMA`]).
pub const CAPABILITY_SUITE_SCHEMA: u32 = 1;

/// Current capability-*report* schema version. Reports version
/// independently of manifests: an older report stays readable but is
/// marked incomplete for any newly required identity, scope, or
/// attribution field (the #832 compatibility contract).
pub const CAPABILITY_REPORT_SCHEMA: u32 = 1;

/// The normative scorer identity every reported *production* token binds
/// to. Designated by ADR-0001 (`docs/adr/0001-normative-r4g1-scorer.md`,
/// #831): the deployed R4G1 serving path scored under
/// `uor-r4-graph-format::scoring_semantics` v1.0.0. A production
/// [`TokenAttribution`] whose `scorer_id` is not this string does not
/// validate — an evaluation may not credit a served token to an
/// unnamed or alternate scorer. The version tail is pinned against
/// `ScoringSemanticsVersion::V1_0_0` by the crate's tests, so it cannot
/// silently drift from the specification it names.
pub const NORMATIVE_SCORER_ID: &str = "uor-r4-graph-format::scoring_semantics@1.0.0";

// --- programme axes: stage, workload, scoring mode ---------------------------

/// A programme stage of #820 (S0–S7). The constitution names, for every
/// stage, a frozen primary suite — so a capability claim cannot quietly
/// change which slice or metric it is judged on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
}

impl Stage {
    /// Every stage, in programme order.
    pub const ALL: [Stage; 8] = [
        Stage::S0,
        Stage::S1,
        Stage::S2,
        Stage::S3,
        Stage::S4,
        Stage::S5,
        Stage::S6,
        Stage::S7,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stage::S0 => "s0",
            Stage::S1 => "s1",
            Stage::S2 => "s2",
            Stage::S3 => "s3",
            Stage::S4 => "s4",
            Stage::S5 => "s5",
            Stage::S6 => "s6",
            Stage::S7 => "s7",
        }
    }
}

/// The nine capability workloads the constitution commits a manifest for.
/// Distinct workloads are never merged into one score (see
/// [`CapabilityReport::comparable_to`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Workload {
    BroadText,
    ContinuityText,
    AssistantCanaries,
    CausalPromptPairs,
    AnswerabilityOod,
    FreeRunning,
    CompositionalReasoning,
    InstructionRetention,
    Scale,
}

impl Workload {
    pub const ALL: [Workload; 9] = [
        Workload::BroadText,
        Workload::ContinuityText,
        Workload::AssistantCanaries,
        Workload::CausalPromptPairs,
        Workload::AnswerabilityOod,
        Workload::FreeRunning,
        Workload::CompositionalReasoning,
        Workload::InstructionRetention,
        Workload::Scale,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Workload::BroadText => "broad-text",
            Workload::ContinuityText => "continuity-text",
            Workload::AssistantCanaries => "assistant-canaries",
            Workload::CausalPromptPairs => "causal-prompt-pairs",
            Workload::AnswerabilityOod => "answerability-ood",
            Workload::FreeRunning => "free-running",
            Workload::CompositionalReasoning => "compositional-reasoning",
            Workload::InstructionRetention => "instruction-retention",
            Workload::Scale => "scale",
        }
    }
}

/// The comparability class of a score. Teacher-forced and free-running
/// numbers measure different things and are **never** merged (a #832
/// non-goal); [`CapabilityReport::comparable_to`] refuses to compare
/// across modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScoringMode {
    TeacherForced,
    FreeRunning,
}

impl ScoringMode {
    pub fn label(self) -> &'static str {
        match self {
            ScoringMode::TeacherForced => "teacher-forced",
            ScoringMode::FreeRunning => "free-running",
        }
    }
}

// --- per-token resolution-path attribution -----------------------------------

/// Which normative mechanism produced one token on the deployed serving
/// path. This is the *path* (mechanism), a separate axis from the D4
/// *status* (`ExactContext`/`Graph`/`Novel`/`Contradictory`): a token has
/// exactly one path.
///
/// The deployed [`crate::engine::R4Engine`] surfaces the served subset
/// directly — [`from_served`] maps its `(PolicyStatus, ngram_hit)` and a
/// decline maps to [`ResolutionPath::Decline`]. The remaining categories
/// carry their own explicit signals a report producer supplies:
/// [`ResolutionPath::RootPrior`] (root base-prior fallback),
/// [`ResolutionPath::PatchDelta`] (a token supplied by an active patch/
/// delta chain), and [`ResolutionPath::SampledSelection`] (the decode
/// policy sampled from the distribution rather than taking the argmax).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolutionPath {
    ExactContext,
    Ngram,
    Graph,
    RootPrior,
    PatchDelta,
    SampledSelection,
    Decline,
}

impl ResolutionPath {
    /// Every path, in a fixed order (the attribution-histogram column
    /// order and the canonical enumeration for coverage checks).
    pub const ALL: [ResolutionPath; 7] = [
        ResolutionPath::ExactContext,
        ResolutionPath::Ngram,
        ResolutionPath::Graph,
        ResolutionPath::RootPrior,
        ResolutionPath::PatchDelta,
        ResolutionPath::SampledSelection,
        ResolutionPath::Decline,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ResolutionPath::ExactContext => "exact-context",
            ResolutionPath::Ngram => "ngram",
            ResolutionPath::Graph => "graph",
            ResolutionPath::RootPrior => "root-prior",
            ResolutionPath::PatchDelta => "patch-delta",
            ResolutionPath::SampledSelection => "sampled-selection",
            ResolutionPath::Decline => "decline",
        }
    }

    /// Whether this path emits a token (everything but a decline).
    pub fn is_served(self) -> bool {
        self != ResolutionPath::Decline
    }

    /// Map a served deployed decision to its path from the observable
    /// `(status, ngram_hit)` signals. The D4 `ExactContext` status splits
    /// on `ngram_hit` (#362): an explicit NGRAM context row is
    /// [`ResolutionPath::Ngram`], the EXCT probe is
    /// [`ResolutionPath::ExactContext`]. A served `Graph`/`Novel`/
    /// `Contradictory` status came from graph-tier selection and maps to
    /// [`ResolutionPath::Graph`] — the D4 status is not the mechanism.
    /// Callers with a finer signal (root-prior fallback, an active patch
    /// chain, or a sampled decode) set [`ResolutionPath::RootPrior`],
    /// [`ResolutionPath::PatchDelta`], or [`ResolutionPath::SampledSelection`]
    /// explicitly instead of calling this.
    pub fn from_served(status: PolicyStatus, ngram_hit: bool) -> ResolutionPath {
        match status {
            PolicyStatus::ExactContext if ngram_hit => ResolutionPath::Ngram,
            PolicyStatus::ExactContext => ResolutionPath::ExactContext,
            PolicyStatus::Graph | PolicyStatus::Novel | PolicyStatus::Contradictory => {
                ResolutionPath::Graph
            }
        }
    }
}

/// A fixed-column histogram of served tokens by resolution path, plus a
/// decline count. Aggregates the per-token attribution so a report always
/// carries the path distribution even when it does not embed every token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributionHistogram {
    pub exact_context: u64,
    pub ngram: u64,
    pub graph: u64,
    pub root_prior: u64,
    pub patch_delta: u64,
    pub sampled_selection: u64,
    pub decline: u64,
}

impl AttributionHistogram {
    /// Record one token's path.
    pub fn record(&mut self, path: ResolutionPath) {
        match path {
            ResolutionPath::ExactContext => self.exact_context += 1,
            ResolutionPath::Ngram => self.ngram += 1,
            ResolutionPath::Graph => self.graph += 1,
            ResolutionPath::RootPrior => self.root_prior += 1,
            ResolutionPath::PatchDelta => self.patch_delta += 1,
            ResolutionPath::SampledSelection => self.sampled_selection += 1,
            ResolutionPath::Decline => self.decline += 1,
        }
    }

    /// Count for one path.
    pub fn count(&self, path: ResolutionPath) -> u64 {
        match path {
            ResolutionPath::ExactContext => self.exact_context,
            ResolutionPath::Ngram => self.ngram,
            ResolutionPath::Graph => self.graph,
            ResolutionPath::RootPrior => self.root_prior,
            ResolutionPath::PatchDelta => self.patch_delta,
            ResolutionPath::SampledSelection => self.sampled_selection,
            ResolutionPath::Decline => self.decline,
        }
    }

    /// Total tokens recorded (served + declined).
    pub fn total(&self) -> u64 {
        ResolutionPath::ALL.iter().map(|&p| self.count(p)).sum()
    }

    /// Tokens that emitted (everything but declines).
    pub fn served(&self) -> u64 {
        self.total() - self.decline
    }
}

// --- controls ----------------------------------------------------------------

/// A negative control / falsifier a suite runs alongside its primary
/// metric. A control that does not separate from the primary (a
/// *degenerate* control) invalidates the reading — see
/// [`is_degenerate_control`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlKind {
    /// EXCT disabled: forces the graph/back-off path so an ExactContext
    /// dominance cannot masquerade as generalization.
    ExctDisabled,
    /// Prompt swapped for an unrelated prompt: a causal-influence null.
    PromptSwap,
    /// Suffix-only context: an n-gram/suffix memorization null.
    SuffixOnly,
    /// Emission table shuffled: destroys the token→score binding.
    ShuffledEmission,
    /// Semantic state shuffled: destroys typed-state carryover.
    ShuffledState,
    /// Trivial (unigram/base) prior: the no-context floor.
    TrivialPrior,
    /// Always serve: the coverage ceiling of an abstaining policy.
    AlwaysServe,
    /// Always decline: the risk floor of an abstaining policy.
    AlwaysDecline,
}

impl ControlKind {
    pub const ALL: [ControlKind; 8] = [
        ControlKind::ExctDisabled,
        ControlKind::PromptSwap,
        ControlKind::SuffixOnly,
        ControlKind::ShuffledEmission,
        ControlKind::ShuffledState,
        ControlKind::TrivialPrior,
        ControlKind::AlwaysServe,
        ControlKind::AlwaysDecline,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ControlKind::ExctDisabled => "exct-disabled",
            ControlKind::PromptSwap => "prompt-swap",
            ControlKind::SuffixOnly => "suffix-only",
            ControlKind::ShuffledEmission => "shuffled-emission",
            ControlKind::ShuffledState => "shuffled-state",
            ControlKind::TrivialPrior => "trivial-prior",
            ControlKind::AlwaysServe => "always-serve",
            ControlKind::AlwaysDecline => "always-decline",
        }
    }
}

// --- metric status: value, unavailable, or not-run ---------------------------

/// A metric outcome. A rate is an exact integer fraction
/// `numerator/denominator` — never a float — so a report serializes
/// deterministically and byte-reproducibly (the #609–#613 serde float
/// lesson). Absence is first-class: a missing fixture is
/// [`MetricStatus::Unavailable`], never a vacuous `Measured` zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum MetricStatus {
    /// A measured rate `numerator/denominator` (both integers). The
    /// denominator is the powered sample count.
    Measured { numerator: u64, denominator: u64 },
    /// The metric could not be measured; `reason` names the missing
    /// fixture/identity. This is the UNAVAILABLE encoding — never PASS.
    Unavailable { reason: String },
    /// The metric is declared by the manifest but was not run in this
    /// report (distinct from Unavailable: nothing was missing, it simply
    /// was not executed).
    NotRun,
}

impl MetricStatus {
    /// Whether a value is present (a `Measured` outcome).
    pub fn is_measured(&self) -> bool {
        matches!(self, MetricStatus::Measured { .. })
    }

    /// The rate in parts-per-thousand (integer, floor), or `None` when
    /// not measured or the denominator is zero.
    pub fn rate_permille(&self) -> Option<u32> {
        match self {
            MetricStatus::Measured {
                numerator,
                denominator,
            } if *denominator > 0 => Some(((numerator * 1000) / denominator) as u32),
            _ => None,
        }
    }
}

// --- metric and control report rows ------------------------------------------

/// One reported metric: its name, comparability mode, outcome, powered
/// sample count, an optional confidence interval (parts-per-thousand),
/// and whether it is the suite's primary (promotion) statistic or a
/// secondary metric.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricReport {
    pub name: String,
    pub mode: ScoringMode,
    pub status: MetricStatus,
    pub sample_n: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_low_permille: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_high_permille: Option<u32>,
    pub primary: bool,
}

/// One reported control: its kind, outcome, and an optional note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlReport {
    pub kind: ControlKind,
    pub status: MetricStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// --- per-token attribution record --------------------------------------------

/// One token's attribution: its position, the emitted token id, the
/// resolution [`ResolutionPath`], the scorer identity it binds to, and
/// whether a widened re-probe ran. A served token must bind
/// [`NORMATIVE_SCORER_ID`]; a decline may bind it or leave it empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenAttribution {
    pub position: u32,
    pub token: u32,
    pub path: ResolutionPath,
    pub scorer_id: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub widened: bool,
}

impl TokenAttribution {
    /// Attribute a real deployed [`PredictDecision`] at `position`, bound
    /// to the normative scorer identity. A served decision maps its
    /// `(status, ngram_hit)` to a path via [`ResolutionPath::from_served`];
    /// an abstention is a [`ResolutionPath::Decline`] and carries token 0.
    pub fn from_decision(position: u32, decision: &crate::engine::PredictDecision) -> Self {
        match decision {
            crate::engine::PredictDecision::Serve(outcome) => TokenAttribution {
                position,
                token: outcome.token,
                path: ResolutionPath::from_served(
                    PolicyStatus::from(outcome.status),
                    outcome.ngram_hit,
                ),
                scorer_id: NORMATIVE_SCORER_ID.to_string(),
                widened: outcome.widened,
            },
            crate::engine::PredictDecision::Abstain(outcome) => TokenAttribution {
                position,
                token: 0,
                path: ResolutionPath::Decline,
                scorer_id: NORMATIVE_SCORER_ID.to_string(),
                widened: outcome.widened,
            },
        }
    }

    /// Structural validation. A served token must bind exactly
    /// [`NORMATIVE_SCORER_ID`] (an evaluation may not credit a served
    /// token to an unnamed or alternate scorer); a decline may bind the
    /// normative id or leave it empty. Returns the first violation, or
    /// `None`.
    pub fn validate(&self) -> Option<String> {
        if self.path.is_served() {
            if self.scorer_id != NORMATIVE_SCORER_ID {
                return Some(format!(
                    "served token at position {} is a {} path but binds scorer_id {:?}, not the normative {:?}",
                    self.position,
                    self.path.label(),
                    self.scorer_id,
                    NORMATIVE_SCORER_ID
                ));
            }
        } else if !self.scorer_id.is_empty() && self.scorer_id != NORMATIVE_SCORER_ID {
            return Some(format!(
                "declined token at position {} binds a non-normative scorer_id {:?}",
                self.position, self.scorer_id
            ));
        }
        None
    }
}

// --- content identity, leakage, and degenerate-control checks ----------------

/// The content identity of `bytes`: a blake3 digest, `blake3:<64 hex>` —
/// the workspace CID convention (`is_blake3_cid` in `uor-r4-graph-cli`,
/// `is_blake3_digest` in `crate::release_bundle`).
pub fn compute_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// Whether `value` is a well-formed `blake3:<64 hex>` identity.
pub fn is_cid(value: &str) -> bool {
    value
        .strip_prefix("blake3:")
        .is_some_and(|hex| hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Verify a committed identity against bytes (tamper detection). Returns
/// `Some(reason)` on a malformed CID or a content mismatch — a single
/// flipped byte fails this check — and `None` when the bytes match.
/// Returns `Option<String>` rather than a `Result` so this shipped crate
/// names no custom error type (R5), mirroring the `validate` methods here.
pub fn verify_cid(expected: &str, bytes: &[u8]) -> Option<String> {
    if !is_cid(expected) {
        return Some(format!(
            "expected identity {expected:?} is not a blake3 CID"
        ));
    }
    let actual = compute_cid(bytes);
    if actual != expected {
        return Some(format!(
            "content identity mismatch: expected {expected}, computed {actual}"
        ));
    }
    None
}

/// Detect document leakage across a split: any id present in both the
/// train and the eval partition. Returns the first overlapping id (as a
/// reason), or `None` when the partitions are document-disjoint.
pub fn detect_document_leakage(train_ids: &[&str], eval_ids: &[&str]) -> Option<String> {
    let train: std::collections::BTreeSet<&str> = train_ids.iter().copied().collect();
    for id in eval_ids {
        if train.contains(id) {
            return Some(format!(
                "document {id:?} appears in both the train and eval partitions"
            ));
        }
    }
    None
}

/// Whether a control failed to separate from the primary — i.e. the
/// control reproduces the primary rate (within `tol_permille`), which
/// means the primary reading is not attributable to the capability under
/// test. Only defined when both are measured; an unmeasured control is
/// not degenerate, it is simply absent.
pub fn is_degenerate_control(
    primary: &MetricStatus,
    control: &MetricStatus,
    tol_permille: u32,
) -> bool {
    match (primary.rate_permille(), control.rate_permille()) {
        (Some(p), Some(c)) => p.abs_diff(c) <= tol_permille,
        _ => false,
    }
}

// --- split rules -------------------------------------------------------------

/// The document/domain/template/entity/topology split axes a suite
/// declares, plus whether it runs leakage and tamper checks. At least one
/// axis and the leakage check are required for a valid suite (a split
/// with no disjointness axis and no leakage check cannot rule out
/// memorization).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitRules {
    #[serde(default)]
    pub by_document: bool,
    #[serde(default)]
    pub by_domain: bool,
    #[serde(default)]
    pub by_template: bool,
    #[serde(default)]
    pub by_entity: bool,
    #[serde(default)]
    pub by_topology: bool,
    pub leakage_check: bool,
    pub tamper_check: bool,
}

impl SplitRules {
    /// The number of disjointness axes this split declares.
    pub fn axis_count(&self) -> usize {
        [
            self.by_document,
            self.by_domain,
            self.by_template,
            self.by_entity,
            self.by_topology,
        ]
        .into_iter()
        .filter(|&b| b)
        .count()
    }
}

/// Known identity slots a suite may require. A manifest's
/// `required_identities` entries must each be one of these labels — a
/// typo would otherwise silently weaken the identity contract.
pub const IDENTITY_SLOTS: [&str; 10] = [
    "teacher",
    "tokenizer",
    "corpus",
    "compiler",
    "artifact",
    "decoder",
    "seed",
    "judge",
    "hardware",
    "report",
];

// --- suite manifest ----------------------------------------------------------

/// One committed, versioned capability-suite manifest: the frozen
/// declaration of what a stage's suite measures and how it is kept honest
/// — its primary metric, promotion statistic, split rules, control set,
/// target report schema, and the identity slots every report must pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteManifest {
    pub schema: u32,
    pub id: String,
    pub stage: Stage,
    pub workload: Workload,
    pub mode: ScoringMode,
    pub primary_metric: String,
    pub promotion_statistic: String,
    pub split: SplitRules,
    pub controls: Vec<ControlKind>,
    pub report_schema: u32,
    pub required_identities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl SuiteManifest {
    /// Structural validation. Returns the first violation, or `None` when
    /// the manifest is structurally valid. Does not touch bytes on disk.
    pub fn validate(&self) -> Option<String> {
        if self.schema != CAPABILITY_SUITE_SCHEMA {
            return Some(format!(
                "suite {:?}: unsupported manifest schema {} (this build reads {CAPABILITY_SUITE_SCHEMA})",
                self.id, self.schema
            ));
        }
        if self.id.trim().is_empty() {
            return Some("a suite has an empty id".to_string());
        }
        if self.report_schema != CAPABILITY_REPORT_SCHEMA {
            return Some(format!(
                "suite {:?}: targets report schema {} (this build reads {CAPABILITY_REPORT_SCHEMA})",
                self.id, self.report_schema
            ));
        }
        if self.primary_metric.trim().is_empty() {
            return Some(format!("suite {:?}: primary_metric is empty", self.id));
        }
        if self.promotion_statistic.trim().is_empty() {
            return Some(format!("suite {:?}: promotion_statistic is empty", self.id));
        }
        if self.workload == Workload::FreeRunning && self.mode != ScoringMode::FreeRunning {
            return Some(format!(
                "suite {:?}: the free-running workload must use the free-running scoring mode",
                self.id
            ));
        }
        if self.controls.is_empty() {
            return Some(format!("suite {:?}: declares no controls", self.id));
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.controls {
            if !seen.insert(*c) {
                return Some(format!(
                    "suite {:?}: control {} is declared twice",
                    self.id,
                    c.label()
                ));
            }
        }
        if !self.split.leakage_check {
            return Some(format!(
                "suite {:?}: split declares no leakage check",
                self.id
            ));
        }
        if self.split.axis_count() == 0 {
            return Some(format!(
                "suite {:?}: split declares no disjointness axis",
                self.id
            ));
        }
        if self.required_identities.is_empty() {
            return Some(format!("suite {:?}: pins no identity slots", self.id));
        }
        for slot in &self.required_identities {
            if !IDENTITY_SLOTS.contains(&slot.as_str()) {
                return Some(format!(
                    "suite {:?}: required identity {slot:?} is not a known identity slot",
                    self.id
                ));
            }
        }
        None
    }
}

// --- constitution: stage -> frozen primary suite -----------------------------

/// One stage's frozen entry in the constitution: the id of its primary
/// (promotion) suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEntry {
    pub primary_suite: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The evaluation constitution: for every programme stage, the id of the
/// frozen primary suite that gates its promotion. Binding stage → suite
/// here (not in prose) is what lets a stage *name* one frozen primary
/// suite, split, control set, report schema, and promotion statistic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constitution {
    pub schema: u32,
    pub stages: BTreeMap<Stage, StageEntry>,
}

impl Constitution {
    /// Structural validation against the committed manifests: schema
    /// support, all eight stages present, and every stage's primary suite
    /// present among `manifests` with a matching stage. Returns the first
    /// violation, or `None`.
    pub fn validate(&self, manifests: &[SuiteManifest]) -> Option<String> {
        if self.schema != CAPABILITY_SUITE_SCHEMA {
            return Some(format!(
                "constitution: unsupported schema {} (this build reads {CAPABILITY_SUITE_SCHEMA})",
                self.schema
            ));
        }
        for stage in Stage::ALL {
            let Some(entry) = self.stages.get(&stage) else {
                return Some(format!(
                    "constitution: stage {} names no primary suite",
                    stage.label()
                ));
            };
            let Some(m) = manifests.iter().find(|m| m.id == entry.primary_suite) else {
                return Some(format!(
                    "constitution: stage {} names primary suite {:?}, which has no committed manifest",
                    stage.label(),
                    entry.primary_suite
                ));
            };
            if m.stage != stage {
                return Some(format!(
                    "constitution: stage {} names primary suite {:?}, whose manifest is stage {}",
                    stage.label(),
                    entry.primary_suite,
                    m.stage.label()
                ));
            }
        }
        None
    }
}

// --- pinned identities of one report -----------------------------------------

/// The content identities one report pins. Each slot is a CID (or a
/// pinned identity string) when the fixture is present, and `None` when
/// it is absent — an absent required slot forces the metrics bound to it
/// to [`MetricStatus::Unavailable`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteIdentities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teacher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<String>,
}

impl SuiteIdentities {
    /// The value pinned for one identity slot label, if present.
    pub fn get(&self, slot: &str) -> Option<&str> {
        match slot {
            "teacher" => self.teacher.as_deref(),
            "tokenizer" => self.tokenizer.as_deref(),
            "corpus" => self.corpus.as_deref(),
            "compiler" => self.compiler.as_deref(),
            "artifact" => self.artifact.as_deref(),
            "decoder" => self.decoder.as_deref(),
            "seed" => self.seed.as_deref(),
            "judge" => self.judge.as_deref(),
            "hardware" => self.hardware.as_deref(),
            "report" => self.report.as_deref(),
            _ => None,
        }
    }

    /// Which of the `required` slots are absent. Metrics bound to any of
    /// these must be reported [`MetricStatus::Unavailable`], never a
    /// value.
    pub fn missing(&self, required: &[String]) -> Vec<String> {
        required
            .iter()
            .filter(|slot| self.get(slot).is_none())
            .cloned()
            .collect()
    }
}

// --- capability report --------------------------------------------------------

/// A versioned capability-suite report: the record one evaluation run
/// emits. It binds every score to the suite identity, execution scope,
/// pinned identities, controls, and — for the deployed serving path — a
/// per-token resolution-path attribution. Serializes deterministically
/// (integer metrics, fixed field order) so identical inputs produce
/// identical report bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityReport {
    pub schema: u32,
    pub suite_id: String,
    pub stage: Stage,
    pub workload: Workload,
    pub mode: ScoringMode,
    pub execution_scope: String,
    /// The corpus partition/slice identity this report was measured on.
    /// Two reports compare only when this matches (see
    /// [`CapabilityReport::comparable_to`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slice_partition_cid: Option<String>,
    pub identities: SuiteIdentities,
    pub metrics: Vec<MetricReport>,
    pub controls: Vec<ControlReport>,
    pub attribution: AttributionHistogram,
    /// The per-token attribution, when the report embeds it (bounded).
    /// The histogram is authoritative even when this is omitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub per_token: Vec<TokenAttribution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl CapabilityReport {
    /// The single primary (promotion) metric, if exactly one is declared.
    pub fn primary_metric(&self) -> Option<&MetricReport> {
        let mut primaries = self.metrics.iter().filter(|m| m.primary);
        let first = primaries.next()?;
        match primaries.next() {
            None => Some(first),
            Some(_) => None,
        }
    }

    /// Canonical, deterministic JSON serialization of this report.
    pub fn to_canonical_json(&self) -> String {
        // serde_json emits struct fields in declaration order and map
        // keys sorted (BTreeMap); with integer-only metrics there is no
        // float ambiguity, so this is byte-stable.
        serde_json::to_string(self).expect("capability report serializes")
    }

    /// Structural validation independent of any manifest. Returns the
    /// first violation, or `None`.
    pub fn validate(&self) -> Option<String> {
        if self.schema != CAPABILITY_REPORT_SCHEMA {
            return Some(format!(
                "report {:?}: unsupported schema {} (this build reads {CAPABILITY_REPORT_SCHEMA})",
                self.suite_id, self.schema
            ));
        }
        if self.suite_id.trim().is_empty() {
            return Some("report has an empty suite_id".to_string());
        }
        if self.execution_scope.trim().is_empty() {
            return Some(format!(
                "report {:?}: execution_scope is empty",
                self.suite_id
            ));
        }
        if let Some(cid) = &self.slice_partition_cid {
            if !is_cid(cid) {
                return Some(format!(
                    "report {:?}: slice_partition_cid {cid:?} is not a blake3 CID",
                    self.suite_id
                ));
            }
        }
        if self.primary_metric().is_none() {
            return Some(format!(
                "report {:?}: must declare exactly one primary metric",
                self.suite_id
            ));
        }
        for m in &self.metrics {
            if let MetricStatus::Measured {
                numerator,
                denominator,
            } = &m.status
            {
                if *denominator == 0 {
                    return Some(format!(
                        "report {:?}: metric {:?} is Measured with a zero denominator",
                        self.suite_id, m.name
                    ));
                }
                if numerator > denominator {
                    return Some(format!(
                        "report {:?}: metric {:?} numerator exceeds denominator",
                        self.suite_id, m.name
                    ));
                }
            }
        }
        for t in &self.per_token {
            if let Some(reason) = t.validate() {
                return Some(format!("report {:?}: {reason}", self.suite_id));
            }
        }
        if !self.per_token.is_empty() {
            let mut tally = AttributionHistogram::default();
            for t in &self.per_token {
                tally.record(t.path);
            }
            if tally != self.attribution {
                return Some(format!(
                    "report {:?}: attribution histogram disagrees with the embedded per-token tally",
                    self.suite_id
                ));
            }
        }
        None
    }

    /// Validate this report against the suite manifest it claims to
    /// implement: matching identity axes, the manifest's primary metric
    /// present and primary, the manifest's controls all reported, and —
    /// the fixture-absence contract — a primary metric that is `Measured`
    /// only when every required identity the manifest pins is present.
    pub fn validate_against(&self, manifest: &SuiteManifest) -> Option<String> {
        if let Some(reason) = self.validate() {
            return Some(reason);
        }
        if self.suite_id != manifest.id {
            return Some(format!(
                "report suite_id {:?} does not match manifest id {:?}",
                self.suite_id, manifest.id
            ));
        }
        if self.stage != manifest.stage
            || self.workload != manifest.workload
            || self.mode != manifest.mode
        {
            return Some(format!(
                "report {:?}: stage/workload/mode disagree with the manifest",
                self.suite_id
            ));
        }
        let primary = self.primary_metric().expect("validated above");
        if primary.name != manifest.primary_metric {
            return Some(format!(
                "report {:?}: primary metric {:?} is not the manifest's {:?}",
                self.suite_id, primary.name, manifest.primary_metric
            ));
        }
        let missing = self.identities.missing(&manifest.required_identities);
        if !missing.is_empty() && primary.status.is_measured() {
            return Some(format!(
                "report {:?}: primary metric is Measured but required identities are absent ({}); absent fixtures must be Unavailable",
                self.suite_id,
                missing.join(", ")
            ));
        }
        for c in &manifest.controls {
            if !self.controls.iter().any(|r| r.kind == *c) {
                return Some(format!(
                    "report {:?}: manifest control {} is not reported",
                    self.suite_id,
                    c.label()
                ));
            }
        }
        None
    }

    /// Whether two reports can be compared without changing corpus
    /// partition or metric semantics: identical workload, identical
    /// scoring mode (teacher-forced and free-running never mix), and the
    /// same pinned slice partition. A missing slice identity on either
    /// side makes them incomparable — sameness cannot be assumed.
    pub fn comparable_to(&self, other: &CapabilityReport) -> bool {
        self.workload == other.workload
            && self.mode == other.mode
            && match (&self.slice_partition_cid, &other.slice_partition_cid) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            }
    }
}

// --- committed constitution (embedded, zero-I/O) -----------------------------

/// The committed suite manifests, embedded so the library validates its
/// own constitution with no filesystem access. Order matches
/// `Workload::ALL` for the nine workload suites, with the secondary
/// continuity suite last.
const MANIFEST_JSON: &[(&str, &str)] = &[
    (
        "broad-text",
        include_str!("../capability_suites/broad_text.json"),
    ),
    (
        "causal-prompt-pairs",
        include_str!("../capability_suites/causal_prompt_pairs.json"),
    ),
    (
        "assistant-canaries",
        include_str!("../capability_suites/assistant_canaries.json"),
    ),
    (
        "answerability-ood",
        include_str!("../capability_suites/answerability_ood.json"),
    ),
    (
        "free-running",
        include_str!("../capability_suites/free_running.json"),
    ),
    (
        "compositional-reasoning",
        include_str!("../capability_suites/compositional_reasoning.json"),
    ),
    (
        "instruction-retention",
        include_str!("../capability_suites/instruction_retention.json"),
    ),
    ("scale", include_str!("../capability_suites/scale.json")),
    (
        "continuity-text",
        include_str!("../capability_suites/continuity_text.json"),
    ),
];

const CONSTITUTION_JSON: &str = include_str!("../capability_suites/constitution.json");

/// Parse the committed suite manifests. Panics only if a committed
/// manifest is malformed — a build-time invariant, like [`env!`], not a
/// recoverable runtime condition; the crate's tests parse and validate
/// every one so a malformed manifest never reaches a release.
pub fn builtin_manifests() -> Vec<SuiteManifest> {
    MANIFEST_JSON
        .iter()
        .map(|(name, json)| {
            serde_json::from_str(json).unwrap_or_else(|e| {
                panic!("committed capability-suite manifest {name:?} is malformed: {e}")
            })
        })
        .collect()
}

/// Parse the committed constitution index (stage → frozen primary suite).
pub fn builtin_constitution() -> Constitution {
    serde_json::from_str(CONSTITUTION_JSON)
        .unwrap_or_else(|e| panic!("committed constitution is malformed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_graph_format::scoring_semantics::ScoringSemanticsVersion;

    const CID_A: &str = "blake3:0000000000000000000000000000000000000000000000000000000000000000";

    // --- the normative scorer identity cannot drift from the spec ------------

    #[test]
    fn normative_scorer_id_pins_the_scoring_semantics_version() {
        let v = ScoringSemanticsVersion::V1_0_0;
        let expected = format!("uor-r4-graph-format::scoring_semantics@{v}");
        assert_eq!(
            NORMATIVE_SCORER_ID, expected,
            "the normative scorer id must name the deployed scoring-semantics version (ADR-0001)"
        );
    }

    // --- the committed constitution parses, validates, and is complete -------

    #[test]
    fn builtin_manifests_all_validate() {
        for m in builtin_manifests() {
            assert_eq!(m.validate(), None, "manifest {:?} must validate", m.id);
        }
    }

    #[test]
    fn builtin_manifests_cover_all_nine_workloads() {
        let manifests = builtin_manifests();
        for w in Workload::ALL {
            assert!(
                manifests.iter().any(|m| m.workload == w),
                "no committed manifest covers workload {}",
                w.label()
            );
        }
        assert_eq!(manifests.len(), 9, "one manifest per workload");
    }

    #[test]
    fn builtin_constitution_validates_and_covers_every_stage() {
        let manifests = builtin_manifests();
        let constitution = builtin_constitution();
        assert_eq!(constitution.validate(&manifests), None);
        for stage in Stage::ALL {
            let entry = constitution
                .stages
                .get(&stage)
                .unwrap_or_else(|| panic!("stage {} has a constitution entry", stage.label()));
            let m = manifests
                .iter()
                .find(|m| m.id == entry.primary_suite)
                .expect("primary suite exists");
            assert_eq!(m.stage, stage, "primary suite stage matches");
            // acceptance criterion 1: a stage names a frozen primary
            // suite, split, control set, report schema, promotion statistic.
            assert!(!m.primary_metric.is_empty());
            assert!(!m.promotion_statistic.is_empty());
            assert!(!m.controls.is_empty());
            assert_eq!(m.report_schema, CAPABILITY_REPORT_SCHEMA);
            assert!(m.split.axis_count() > 0 && m.split.leakage_check);
        }
    }

    // --- CID: identity, tamper, malformed ------------------------------------

    #[test]
    fn cid_round_trips_and_verifies() {
        let bytes = b"a small committed fixture";
        let cid = compute_cid(bytes);
        assert!(is_cid(&cid));
        assert_eq!(verify_cid(&cid, bytes), None);
    }

    #[test]
    fn verify_cid_detects_a_single_flipped_byte() {
        let mut bytes = b"a small committed fixture".to_vec();
        let cid = compute_cid(&bytes);
        bytes[0] ^= 0x01;
        assert!(
            verify_cid(&cid, &bytes).is_some(),
            "tamper must be rejected"
        );
    }

    #[test]
    fn verify_cid_rejects_a_malformed_identity() {
        assert!(verify_cid("not-a-cid", b"x").is_some());
    }

    // --- leakage and degenerate controls -------------------------------------

    #[test]
    fn document_leakage_is_detected_and_disjoint_passes() {
        assert!(detect_document_leakage(&["a", "b"], &["c", "d"]).is_none());
        assert!(detect_document_leakage(&["a", "b"], &["b", "c"]).is_some());
    }

    #[test]
    fn degenerate_control_flags_a_control_that_matches_the_primary() {
        let primary = MetricStatus::Measured {
            numerator: 500,
            denominator: 1000,
        };
        let same = MetricStatus::Measured {
            numerator: 501,
            denominator: 1000,
        };
        let separated = MetricStatus::Measured {
            numerator: 100,
            denominator: 1000,
        };
        assert!(is_degenerate_control(&primary, &same, 5));
        assert!(!is_degenerate_control(&primary, &separated, 5));
        // an unmeasured control is absent, not degenerate.
        assert!(!is_degenerate_control(&primary, &MetricStatus::NotRun, 5));
    }

    // --- attribution mapping and histogram -----------------------------------

    #[test]
    fn resolution_path_maps_deployed_signals() {
        assert_eq!(
            ResolutionPath::from_served(PolicyStatus::ExactContext, false),
            ResolutionPath::ExactContext
        );
        assert_eq!(
            ResolutionPath::from_served(PolicyStatus::ExactContext, true),
            ResolutionPath::Ngram
        );
        assert_eq!(
            ResolutionPath::from_served(PolicyStatus::Graph, false),
            ResolutionPath::Graph
        );
        assert_eq!(
            ResolutionPath::from_served(PolicyStatus::Novel, false),
            ResolutionPath::Graph
        );
    }

    #[test]
    fn attribution_histogram_records_and_totals() {
        let mut h = AttributionHistogram::default();
        h.record(ResolutionPath::ExactContext);
        h.record(ResolutionPath::Ngram);
        h.record(ResolutionPath::Graph);
        h.record(ResolutionPath::Decline);
        assert_eq!(h.count(ResolutionPath::ExactContext), 1);
        assert_eq!(h.total(), 4);
        assert_eq!(h.served(), 3);
    }

    #[test]
    fn metric_status_unavailable_has_no_rate() {
        let u = MetricStatus::Unavailable {
            reason: "corpus fixture absent".to_string(),
        };
        assert!(!u.is_measured());
        assert_eq!(u.rate_permille(), None);
    }

    // --- token attribution: served must bind the normative scorer ------------

    #[test]
    fn served_token_requires_the_normative_scorer_id() {
        let good = TokenAttribution {
            position: 0,
            token: 7,
            path: ResolutionPath::Graph,
            scorer_id: NORMATIVE_SCORER_ID.to_string(),
            widened: false,
        };
        assert_eq!(good.validate(), None);

        let bad = TokenAttribution {
            scorer_id: "some-other-scorer".to_string(),
            ..good.clone()
        };
        assert!(
            bad.validate().is_some(),
            "a served token bound to a non-normative scorer must be rejected"
        );
    }

    #[test]
    fn declined_token_may_leave_scorer_empty() {
        let decline = TokenAttribution {
            position: 3,
            token: 0,
            path: ResolutionPath::Decline,
            scorer_id: String::new(),
            widened: false,
        };
        assert_eq!(decline.validate(), None);
    }

    // --- report: round-trip, determinism, comparability, fixture absence -----

    fn valid_report() -> CapabilityReport {
        let mut attribution = AttributionHistogram::default();
        attribution.record(ResolutionPath::Graph);
        attribution.record(ResolutionPath::ExactContext);
        CapabilityReport {
            schema: CAPABILITY_REPORT_SCHEMA,
            suite_id: "s0-broad-text".to_string(),
            stage: Stage::S0,
            workload: Workload::BroadText,
            mode: ScoringMode::TeacherForced,
            execution_scope: "offline-eval + measured R4Engine reachability".to_string(),
            slice_partition_cid: Some(CID_A.to_string()),
            identities: SuiteIdentities {
                teacher: Some("teacher-smollm2-135m".to_string()),
                tokenizer: Some("tok-hf-byte-bpe".to_string()),
                corpus: Some(CID_A.to_string()),
                compiler: Some("compiler-v1".to_string()),
                artifact: Some(CID_A.to_string()),
                decoder: Some("argmax".to_string()),
                ..Default::default()
            },
            metrics: vec![MetricReport {
                name: "held-out-top1".to_string(),
                mode: ScoringMode::TeacherForced,
                status: MetricStatus::Measured {
                    numerator: 181,
                    denominator: 1000,
                },
                sample_n: 1000,
                ci_low_permille: Some(160),
                ci_high_permille: Some(202),
                primary: true,
            }],
            controls: vec![
                ControlReport {
                    kind: ControlKind::ExctDisabled,
                    status: MetricStatus::Measured {
                        numerator: 40,
                        denominator: 1000,
                    },
                    note: None,
                },
                ControlReport {
                    kind: ControlKind::SuffixOnly,
                    status: MetricStatus::Measured {
                        numerator: 55,
                        denominator: 1000,
                    },
                    note: None,
                },
                ControlReport {
                    kind: ControlKind::TrivialPrior,
                    status: MetricStatus::Measured {
                        numerator: 30,
                        denominator: 1000,
                    },
                    note: None,
                },
            ],
            attribution,
            per_token: vec![
                TokenAttribution {
                    position: 0,
                    token: 42,
                    path: ResolutionPath::Graph,
                    scorer_id: NORMATIVE_SCORER_ID.to_string(),
                    widened: false,
                },
                TokenAttribution {
                    position: 1,
                    token: 7,
                    path: ResolutionPath::ExactContext,
                    scorer_id: NORMATIVE_SCORER_ID.to_string(),
                    widened: false,
                },
            ],
            notes: None,
        }
    }

    #[test]
    fn report_validates_round_trips_and_is_deterministic() {
        let report = valid_report();
        assert_eq!(report.validate(), None);
        let json = report.to_canonical_json();
        let parsed: CapabilityReport = serde_json::from_str(&json).expect("round trip");
        assert_eq!(report, parsed);
        assert_eq!(json, parsed.to_canonical_json(), "serialization is stable");
    }

    #[test]
    fn report_rejects_an_unknown_field() {
        let mut value = serde_json::to_value(valid_report()).expect("to_value");
        value
            .as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::json!(1));
        let parsed: Result<CapabilityReport, _> = serde_json::from_value(value);
        assert!(parsed.is_err(), "deny_unknown_fields rejects extra keys");
    }

    #[test]
    fn report_histogram_must_match_embedded_tokens() {
        let mut report = valid_report();
        report.per_token[0].path = ResolutionPath::Ngram; // no longer matches the histogram
        assert!(
            report.validate().is_some(),
            "a histogram that disagrees with the per-token tally is rejected"
        );
    }

    #[test]
    fn comparability_requires_same_slice_and_mode() {
        let a = valid_report();
        let mut same = valid_report();
        assert!(a.comparable_to(&same));

        same.slice_partition_cid = Some(
            "blake3:1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        );
        assert!(
            !a.comparable_to(&same),
            "different partitions are incomparable"
        );

        let mut free = valid_report();
        free.mode = ScoringMode::FreeRunning;
        assert!(
            !a.comparable_to(&free),
            "teacher-forced and free-running never compare"
        );

        let mut no_slice = valid_report();
        no_slice.slice_partition_cid = None;
        assert!(
            !a.comparable_to(&no_slice),
            "a missing slice identity is incomparable, not assumed-equal"
        );
    }

    #[test]
    fn report_against_manifest_enforces_fixture_absence() {
        let manifest = builtin_manifests()
            .into_iter()
            .find(|m| m.id == "s0-broad-text")
            .expect("s0 manifest");
        let good = valid_report();
        assert_eq!(good.validate_against(&manifest), None);

        // Drop a required identity but keep the primary Measured -> rejected.
        let mut absent = valid_report();
        absent.identities.corpus = None;
        assert!(
            absent.validate_against(&manifest).is_some(),
            "a Measured primary with an absent required identity must be rejected"
        );

        // The honest form: mark the primary Unavailable when a fixture is absent.
        let mut honest = valid_report();
        honest.identities.corpus = None;
        honest.metrics[0].status = MetricStatus::Unavailable {
            reason: "corpus fixture absent".to_string(),
        };
        assert_eq!(honest.validate_against(&manifest), None);
    }
}
