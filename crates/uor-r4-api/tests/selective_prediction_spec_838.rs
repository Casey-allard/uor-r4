//! Executable reference model and machine-checked evidence for the typed
//! selective-prediction contract and answerability benchmark constitution
//! (#838, item A of S2 tracker #823).
//!
//! Companion document: `docs/selective_prediction_spec_838.md`. This is a
//! **reference-only / off-serving-path** model in the #830 sense: an owned,
//! integer realization of the specified status space, decision table, surface
//! encodings, fail-closed calibration semantics, benchmark categories,
//! baseline confusion signatures, and power/operating-point constitution. It
//! deploys no predictor and fits no threshold (that is #837), and it changes
//! no serving surface (that is #839). It binds the frozen S2 evaluation
//! vocabulary of #832 (`s2-answerability-ood`): `ControlKind`, the
//! integer-fraction `MetricStatus`, the degeneracy check, the CID helpers, and
//! the leakage check all come from `uor_r4_api::capability_suite`.
//!
//! The completion this file evidences explicitly records: **current semantic
//! abstention is NOT ESTABLISHED** — the deployed D4 policy is a coverage
//! policy, and a coverage-only report cannot render a semantic verdict
//! (`coverage_result_cannot_render_semantic_pass`).

use std::collections::BTreeMap;

use uor_r4_api::capability_suite::{
    compute_cid, detect_document_leakage, is_degenerate_control, verify_cid, ControlKind,
    MetricStatus,
};

// --- §2: the typed status space ---------------------------------------------

/// The eight selective-prediction statuses (spec §2). Each is a separate
/// typed concept with exactly one meaning and one canonical label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SelectiveStatus {
    Covered,
    DistributionallyNovel,
    InsufficientEvidence,
    ConflictingEvidence,
    SupportedAnswer,
    LowConfidence,
    Abstention,
    HardIncompatibility,
}

impl SelectiveStatus {
    const ALL: [SelectiveStatus; 8] = [
        SelectiveStatus::Covered,
        SelectiveStatus::DistributionallyNovel,
        SelectiveStatus::InsufficientEvidence,
        SelectiveStatus::ConflictingEvidence,
        SelectiveStatus::SupportedAnswer,
        SelectiveStatus::LowConfidence,
        SelectiveStatus::Abstention,
        SelectiveStatus::HardIncompatibility,
    ];

    /// Canonical kebab-case wire label (spec §2), used verbatim on every
    /// surface except OpenAI-compatible `error.code`, which applies the
    /// deterministic rewrite `-` → `_` (spec §5).
    fn label(self) -> &'static str {
        match self {
            SelectiveStatus::Covered => "covered",
            SelectiveStatus::DistributionallyNovel => "distributionally-novel",
            SelectiveStatus::InsufficientEvidence => "insufficient-evidence",
            SelectiveStatus::ConflictingEvidence => "conflicting-evidence",
            SelectiveStatus::SupportedAnswer => "supported-answer",
            SelectiveStatus::LowConfidence => "low-confidence",
            SelectiveStatus::Abstention => "abstention",
            SelectiveStatus::HardIncompatibility => "hard-incompatibility",
        }
    }

    fn parse(label: &str) -> Option<SelectiveStatus> {
        SelectiveStatus::ALL
            .into_iter()
            .find(|s| s.label() == label)
    }
}

/// Coverage axis (spec §2): the structural D4 reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Coverage {
    Covered,
    DistributionallyNovel,
}

impl Coverage {
    fn status(self) -> SelectiveStatus {
        match self {
            Coverage::Covered => SelectiveStatus::Covered,
            Coverage::DistributionallyNovel => SelectiveStatus::DistributionallyNovel,
        }
    }
}

/// Evidence axis (spec §2): the evidential reading (#837 fits its deployed
/// classifier; here it is a reference input).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Evidence {
    Supported,
    Insufficient,
    Conflicting,
}

/// Compatibility gate (spec §2): fail-closed, evaluated first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Compat {
    Compatible,
    HardIncompatibility,
}

/// Calibration-data state (spec §6): absent is legacy mode; corrupt is a
/// hard incompatibility and never silently degrades to legacy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Calibration {
    Absent,
    Valid,
    Corrupt,
}

/// Served outcome (spec §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    SupportedAnswer,
    Abstention,
    HardIncompatibility,
}

impl Outcome {
    fn status(self) -> SelectiveStatus {
        match self {
            Outcome::SupportedAnswer => SelectiveStatus::SupportedAnswer,
            Outcome::Abstention => SelectiveStatus::Abstention,
            Outcome::HardIncompatibility => SelectiveStatus::HardIncompatibility,
        }
    }
}

/// Typed abstention cause (spec §2 precedence order; `DistributionallyNovel`
/// is legal only in legacy mode, spec §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cause {
    ConflictingEvidence,
    InsufficientEvidence,
    LowConfidence,
    DistributionallyNovel,
}

impl Cause {
    fn status(self) -> SelectiveStatus {
        match self {
            Cause::ConflictingEvidence => SelectiveStatus::ConflictingEvidence,
            Cause::InsufficientEvidence => SelectiveStatus::InsufficientEvidence,
            Cause::LowConfidence => SelectiveStatus::LowConfidence,
            Cause::DistributionallyNovel => SelectiveStatus::DistributionallyNovel,
        }
    }
}

// --- §4: evidence block and typed response -----------------------------------

/// Bounded evidence counts carried by a served answer (spec §4). Integer-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvidenceBlock {
    supporting: u32,
    conflicting: u32,
}

/// The typed production response (spec §4): outcome plus the reported axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Response {
    outcome: Outcome,
    coverage: Coverage,
    cause: Option<Cause>,
    confidence_permille: Option<u32>,
    evidence: Option<EvidenceBlock>,
}

/// The witness record carries the same typed fields as the response
/// (spec §4, `witness_carries_the_same_typed_fields`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Witness {
    outcome: Outcome,
    coverage: Coverage,
    cause: Option<Cause>,
    confidence_permille: Option<u32>,
    evidence: Option<EvidenceBlock>,
}

fn witness_of(r: &Response) -> Witness {
    Witness {
        outcome: r.outcome,
        coverage: r.coverage,
        cause: r.cause,
        confidence_permille: r.confidence_permille,
        evidence: r.evidence,
    }
}

/// Deterministic reference evidence block for an evidence-axis value.
fn evidence_block(e: Evidence) -> EvidenceBlock {
    match e {
        Evidence::Supported => EvidenceBlock {
            supporting: 3,
            conflicting: 0,
        },
        Evidence::Insufficient => EvidenceBlock {
            supporting: 0,
            conflicting: 0,
        },
        Evidence::Conflicting => EvidenceBlock {
            supporting: 2,
            conflicting: 2,
        },
    }
}

// --- §2/§6: the total deterministic decision table ---------------------------

/// The spec §2 outcome function, total over its input space. `evidence` and
/// `q` are consulted only when `calibration` is `Valid`; a corrupt
/// calibration or an incompatible request fails closed (spec §6).
fn decide(
    calibration: Calibration,
    compat: Compat,
    coverage: Coverage,
    evidence: Evidence,
    q: u32,
    theta: u32,
) -> Response {
    if compat == Compat::HardIncompatibility || calibration == Calibration::Corrupt {
        return Response {
            outcome: Outcome::HardIncompatibility,
            coverage,
            cause: None,
            confidence_permille: None,
            evidence: None,
        };
    }
    match calibration {
        Calibration::Absent => match coverage {
            // Legacy-coverage mode (spec §6): today's D4 policy through the
            // typed schema; no confidence, no evidence axis, ever.
            Coverage::Covered => Response {
                outcome: Outcome::SupportedAnswer,
                coverage,
                cause: None,
                confidence_permille: None,
                evidence: None,
            },
            Coverage::DistributionallyNovel => Response {
                outcome: Outcome::Abstention,
                coverage,
                cause: Some(Cause::DistributionallyNovel),
                confidence_permille: None,
                evidence: None,
            },
        },
        Calibration::Corrupt => unreachable!("handled above"),
        Calibration::Valid => match evidence {
            Evidence::Conflicting => Response {
                outcome: Outcome::Abstention,
                coverage,
                cause: Some(Cause::ConflictingEvidence),
                confidence_permille: Some(q),
                evidence: Some(evidence_block(evidence)),
            },
            Evidence::Insufficient => Response {
                outcome: Outcome::Abstention,
                coverage,
                cause: Some(Cause::InsufficientEvidence),
                confidence_permille: Some(q),
                evidence: Some(evidence_block(evidence)),
            },
            Evidence::Supported => {
                if q < theta {
                    Response {
                        outcome: Outcome::Abstention,
                        coverage,
                        cause: Some(Cause::LowConfidence),
                        confidence_permille: Some(q),
                        evidence: Some(evidence_block(evidence)),
                    }
                } else {
                    Response {
                        outcome: Outcome::SupportedAnswer,
                        coverage,
                        cause: None,
                        confidence_permille: Some(q),
                        evidence: Some(evidence_block(evidence)),
                    }
                }
            }
        },
    }
}

// --- §5: deterministic surface encoders --------------------------------------

fn opt_label(c: Option<Cause>) -> &'static str {
    c.map_or("-", |c| c.status().label())
}

fn opt_permille(q: Option<u32>) -> String {
    q.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

fn json_opt_permille(q: Option<u32>) -> String {
    q.map_or_else(|| "null".to_owned(), |v| v.to_string())
}

/// CLI surface (spec §5): typed record extending the #811 `ChatAbstention`
/// shape. An abstention is a successful, honest outcome (exit 0); a hard
/// incompatibility is a typed error (exit 2).
fn encode_cli(r: &Response) -> String {
    let exit = match r.outcome {
        Outcome::SupportedAnswer | Outcome::Abstention => 0,
        Outcome::HardIncompatibility => 2,
    };
    format!(
        "exit={exit} status={} coverage={} cause={} confidence={}",
        r.outcome.status().label(),
        r.coverage.status().label(),
        opt_label(r.cause),
        opt_permille(r.confidence_permille),
    )
}

/// Native HTTP surface (spec §5): status code + deterministic JSON with a
/// fixed field order.
fn encode_http(r: &Response) -> (u16, String) {
    let code = match r.outcome {
        Outcome::SupportedAnswer | Outcome::Abstention => 200,
        Outcome::HardIncompatibility => 409,
    };
    let evidence = r.evidence.map_or_else(
        || "null".to_owned(),
        |e| {
            format!(
                "{{\"supporting\":{},\"conflicting\":{}}}",
                e.supporting, e.conflicting
            )
        },
    );
    let cause = r.cause.map_or_else(
        || "null".to_owned(),
        |c| format!("{:?}", c.status().label()),
    );
    let confidence = r
        .confidence_permille
        .map_or_else(|| "null".to_owned(), |v| v.to_string());
    let body = format!(
        "{{\"status\":{:?},\"coverage\":{:?},\"cause\":{cause},\"confidence_permille\":{confidence},\"evidence\":{evidence}}}",
        r.outcome.status().label(),
        r.coverage.status().label(),
    );
    (code, body)
}

/// The deterministic `-` → `_` rewrite for OpenAI-compatible `error.code`
/// values (spec §5).
fn snake(label: &str) -> String {
    label.replace('-', "_")
}

/// OpenAI-compatible non-streaming surface (spec §5). An abstention is a
/// structured error body with a typed code — never an empty-`choices`
/// success, never a generic server error.
fn encode_openai_nonstream(r: &Response) -> (u16, String) {
    match r.outcome {
        Outcome::SupportedAnswer => (
            200,
            format!(
                "{{\"choices\":[{{\"message\":\"<answer>\"}}],\"uor\":{{\"status\":{:?},\"coverage\":{:?},\"confidence_permille\":{}}}}}",
                r.outcome.status().label(),
                r.coverage.status().label(),
                json_opt_permille(r.confidence_permille),
            ),
        ),
        Outcome::Abstention => {
            let cause = r.cause.expect("abstention carries a typed cause");
            (
                422,
                format!(
                    "{{\"error\":{{\"type\":\"uor_selective_prediction\",\"code\":\"uor_abstention_{}\",\"coverage\":{:?}}}}}",
                    snake(cause.status().label()),
                    r.coverage.status().label(),
                ),
            )
        }
        Outcome::HardIncompatibility => (
            409,
            "{\"error\":{\"type\":\"uor_selective_prediction\",\"code\":\"uor_incompatible_artifact\"}}"
                .to_owned(),
        ),
    }
}

/// OpenAI-compatible streaming surface (spec §5): the ordered event list. An
/// abstention emits **no** content chunk — one terminal typed error event,
/// then `[DONE]`; never a silent stream end.
fn encode_openai_stream(r: &Response) -> Vec<String> {
    match r.outcome {
        Outcome::SupportedAnswer => vec![
            format!(
                "data: {{\"choices\":[{{\"delta\":\"<answer>\"}}],\"uor\":{{\"coverage\":{:?},\"confidence_permille\":{}}}}}",
                r.coverage.status().label(),
                json_opt_permille(r.confidence_permille),
            ),
            "data: [DONE]".to_owned(),
        ],
        Outcome::Abstention => {
            let cause = r.cause.expect("abstention carries a typed cause");
            vec![
                format!(
                    "event: error\ndata: {{\"code\":\"uor_abstention_{}\"}}",
                    snake(cause.status().label())
                ),
                "data: [DONE]".to_owned(),
            ]
        }
        Outcome::HardIncompatibility => vec![
            "event: error\ndata: {\"code\":\"uor_incompatible_artifact\"}".to_owned(),
            "data: [DONE]".to_owned(),
        ],
    }
}

/// WASM host boundary (spec §5): a typed tagged value with the canonical
/// labels — a hard incompatibility is a typed `Err`, never a trap.
fn encode_wasm(r: &Response) -> (u8, String) {
    match r.outcome {
        Outcome::SupportedAnswer => (
            0,
            format!(
                "served:{}:{}",
                r.coverage.status().label(),
                opt_permille(r.confidence_permille)
            ),
        ),
        Outcome::Abstention => {
            let cause = r.cause.expect("abstention carries a typed cause");
            (
                1,
                format!(
                    "abstained:{}:{}",
                    cause.status().label(),
                    r.coverage.status().label()
                ),
            )
        }
        Outcome::HardIncompatibility => (2, "incompatible".to_owned()),
    }
}

/// Decode the outcome + cause back out of the CLI encoding (round-trip half).
fn decode_cli(s: &str) -> (SelectiveStatus, Option<SelectiveStatus>) {
    let mut status = None;
    let mut cause = None;
    for part in s.split(' ') {
        if let Some(v) = part.strip_prefix("status=") {
            status = SelectiveStatus::parse(v);
        }
        if let Some(v) = part.strip_prefix("cause=") {
            cause = SelectiveStatus::parse(v);
        }
    }
    (status.expect("cli status"), cause)
}

// --- §6: calibration-data state from bytes -----------------------------------

/// Classify calibration data (spec §6): absent → legacy; CID-mismatched
/// (tampered/truncated) → corrupt, which fails closed.
fn classify_calibration(data: Option<(&str, &[u8])>) -> Calibration {
    match data {
        None => Calibration::Absent,
        Some((expected_cid, bytes)) => {
            if verify_cid(expected_cid, bytes).is_some() {
                Calibration::Corrupt
            } else {
                Calibration::Valid
            }
        }
    }
}

// --- §7: benchmark categories and items --------------------------------------

/// The eight benchmark categories (spec §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Category {
    InDomainAnswerable,
    ParaphrasedAnswerable,
    NovelButSupported,
    MissingEvidence,
    PrivateInformation,
    FalsePremise,
    ContradictoryEvidence,
    UnrelatedOod,
}

impl Category {
    const ALL: [Category; 8] = [
        Category::InDomainAnswerable,
        Category::ParaphrasedAnswerable,
        Category::NovelButSupported,
        Category::MissingEvidence,
        Category::PrivateInformation,
        Category::FalsePremise,
        Category::ContradictoryEvidence,
        Category::UnrelatedOod,
    ];

    fn label(self) -> &'static str {
        match self {
            Category::InDomainAnswerable => "in-domain-answerable",
            Category::ParaphrasedAnswerable => "paraphrased-answerable",
            Category::NovelButSupported => "novel-but-supported",
            Category::MissingEvidence => "missing-evidence",
            Category::PrivateInformation => "private-information",
            Category::FalsePremise => "false-premise",
            Category::ContradictoryEvidence => "contradictory-evidence",
            Category::UnrelatedOod => "unrelated-ood",
        }
    }

    fn answerable(self) -> bool {
        matches!(
            self,
            Category::InDomainAnswerable
                | Category::ParaphrasedAnswerable
                | Category::NovelButSupported
        )
    }

    /// Gold outcome and the set of acceptable typed causes (spec §7). The
    /// legacy cause is permitted only for `unrelated-ood`.
    fn gold(self) -> (Outcome, &'static [Cause]) {
        match self {
            Category::InDomainAnswerable
            | Category::ParaphrasedAnswerable
            | Category::NovelButSupported => (Outcome::SupportedAnswer, &[]),
            Category::MissingEvidence | Category::PrivateInformation | Category::FalsePremise => {
                (Outcome::Abstention, &[Cause::InsufficientEvidence])
            }
            Category::ContradictoryEvidence => (Outcome::Abstention, &[Cause::ConflictingEvidence]),
            Category::UnrelatedOod => (
                Outcome::Abstention,
                &[Cause::InsufficientEvidence, Cause::DistributionallyNovel],
            ),
        }
    }

    /// The deterministic reference feature vector a working system would
    /// read for this category (spec §7/§8): coverage, evidence, calibrated
    /// confidence (‰), Hamming distance to the nearest calibrated region,
    /// support count, conflict count, and whether a served answer would be
    /// correct.
    fn features(self) -> ItemFeatures {
        match self {
            Category::InDomainAnswerable => {
                ItemFeatures::new(Coverage::Covered, Evidence::Supported, 800, 10, 4, 0, true)
            }
            // Paraphrase keeps the item covered and supported but pushes the
            // surface Hamming distance past the distance-only threshold —
            // the paraphrase-brittleness signature of that baseline (§8).
            Category::ParaphrasedAnswerable => {
                ItemFeatures::new(Coverage::Covered, Evidence::Supported, 750, 55, 3, 0, true)
            }
            Category::NovelButSupported => ItemFeatures::new(
                Coverage::DistributionallyNovel,
                Evidence::Supported,
                700,
                80,
                3,
                0,
                true,
            ),
            Category::MissingEvidence => ItemFeatures::new(
                Coverage::Covered,
                Evidence::Insufficient,
                200,
                15,
                0,
                0,
                false,
            ),
            Category::PrivateInformation => ItemFeatures::new(
                Coverage::Covered,
                Evidence::Insufficient,
                150,
                25,
                0,
                0,
                false,
            ),
            Category::FalsePremise => ItemFeatures::new(
                Coverage::Covered,
                Evidence::Insufficient,
                250,
                12,
                0,
                0,
                false,
            ),
            Category::ContradictoryEvidence => ItemFeatures::new(
                Coverage::Covered,
                Evidence::Conflicting,
                300,
                18,
                3,
                3,
                false,
            ),
            Category::UnrelatedOod => ItemFeatures::new(
                Coverage::DistributionallyNovel,
                Evidence::Insufficient,
                100,
                90,
                0,
                0,
                false,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ItemFeatures {
    coverage: Coverage,
    evidence: Evidence,
    confidence_permille: u32,
    hamming_distance: u32,
    support_count: u32,
    conflict_count: u32,
    correct_if_served: bool,
}

impl ItemFeatures {
    fn new(
        coverage: Coverage,
        evidence: Evidence,
        confidence_permille: u32,
        hamming_distance: u32,
        support_count: u32,
        conflict_count: u32,
        correct_if_served: bool,
    ) -> Self {
        Self {
            coverage,
            evidence,
            confidence_permille,
            hamming_distance,
            support_count,
            conflict_count,
            correct_if_served,
        }
    }
}

/// A benchmark item with its gold annotation and the four disjointness keys
/// (spec §7).
#[derive(Debug, Clone)]
struct Item {
    id: String,
    category: Category,
    document: String,
    domain: String,
    entity: String,
    template: String,
}

fn make_item(category: Category, idx: usize, partition: &str) -> Item {
    let c = category.label();
    Item {
        id: format!("{partition}-{c}-{idx}"),
        category,
        document: format!("doc-{partition}-{c}-{idx}"),
        domain: format!("domain-{partition}-{}", idx % 2),
        entity: format!("entity-{partition}-{c}-{idx}"),
        template: format!("template-{partition}-{c}"),
    }
}

// --- §8: baseline policies and confusion signatures --------------------------

/// The six frozen baselines (spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BaselinePolicy {
    AlwaysServe,
    AlwaysDecline,
    CurrentD4,
    DistanceOnly,
    EvidenceCountOnly,
    TrivialPrior,
}

impl BaselinePolicy {
    const ALL: [BaselinePolicy; 6] = [
        BaselinePolicy::AlwaysServe,
        BaselinePolicy::AlwaysDecline,
        BaselinePolicy::CurrentD4,
        BaselinePolicy::DistanceOnly,
        BaselinePolicy::EvidenceCountOnly,
        BaselinePolicy::TrivialPrior,
    ];

    /// Whether this policy serves an item with the given features. Pure and
    /// deterministic; the distance threshold (50) and support threshold (2)
    /// are fixed reference constants.
    fn serves(self, f: &ItemFeatures) -> bool {
        match self {
            BaselinePolicy::AlwaysServe | BaselinePolicy::TrivialPrior => true,
            BaselinePolicy::AlwaysDecline => false,
            BaselinePolicy::CurrentD4 => f.coverage == Coverage::Covered,
            BaselinePolicy::DistanceOnly => f.hamming_distance <= 50,
            BaselinePolicy::EvidenceCountOnly => f.support_count >= 2,
        }
    }

    /// Whether a served answer from this policy is correct on this item.
    /// `trivial-prior` serves the no-context marginal, wrong even on
    /// answerable items — the floor that separates it from `always-serve`.
    fn served_correct(self, f: &ItemFeatures) -> bool {
        match self {
            BaselinePolicy::TrivialPrior => false,
            _ => f.correct_if_served,
        }
    }
}

/// Per-category confusion cell: (served, false-answer, false-abstain).
type ConfusionCell = (bool, bool, bool);

/// The confusion signature of a policy over the eight categories, keyed by
/// category label (BTreeMap: a deterministic ordered reduction).
fn confusion_signature(policy: BaselinePolicy) -> BTreeMap<&'static str, ConfusionCell> {
    let mut sig = BTreeMap::new();
    for cat in Category::ALL {
        let f = cat.features();
        let served = policy.serves(&f);
        let false_answer = served && (!cat.answerable() || !policy.served_correct(&f));
        let false_abstain = !served && cat.answerable();
        sig.insert(cat.label(), (served, false_answer, false_abstain));
    }
    sig
}

// --- §9: power constants and UCB arithmetic (fixed before any fitting) -------

const N_PER_CATEGORY: u64 = 600;
const N_TOTAL: u64 = 4_800;
/// Frozen one-sided 95% upper-confidence-bound targets (‰) on the
/// false-answer rate, per operating point (spec §9).
const RELEASE_FALSE_ANSWER_UCB_PERMILLE: u32 = 10;
const RESEARCH_FALSE_ANSWER_UCB_PERMILLE: u32 = 50;
/// Frozen false-abstain ceilings (‰) on answerable items (spec §9).
const RELEASE_FALSE_ABSTAIN_PERMILLE: u32 = 200;
const RESEARCH_FALSE_ABSTAIN_PERMILLE: u32 = 300;

/// The pre-registered conservative UCB95 reference arithmetic (‰):
/// `(1000·k + 3000) / n`, the point estimate plus the rule-of-three margin,
/// integer ceiling. The real report additionally states the exact
/// Clopper–Pearson bound; this frozen form is what the reference selection
/// rule uses, so the rule is deterministic and integer-only.
fn ucb95_permille(failures: u64, n: u64) -> u32 {
    assert!(
        n > 0,
        "UCB over an empty sample is UNAVAILABLE, never a value"
    );
    (1000_u64
        .saturating_mul(failures)
        .saturating_add(3_000)
        .div_ceil(n)) as u32
}

/// One point on a calibration curve: threshold θ (‰), observed false
/// answers among served unanswerable items, the unanswerable sample count,
/// and the achieved coverage (‰) on answerable items.
#[derive(Debug, Clone, Copy)]
struct CurvePoint {
    theta: u32,
    false_answers: u64,
    n_unanswerable: u64,
    coverage_permille: u32,
}

/// The frozen operating-point selection rule (spec §9): maximize coverage
/// subject to `ucb95 ≤ target`; ties break to the smaller θ. Returns `None`
/// when no point satisfies the bound (a legitimate negative — the caller
/// records it, never relaxes the target).
fn select_operating_point(curve: &[CurvePoint], ucb_target_permille: u32) -> Option<u32> {
    let mut best: Option<(u32, u32)> = None; // (coverage, theta)
    for p in curve {
        if ucb95_permille(p.false_answers, p.n_unanswerable) > ucb_target_permille {
            continue;
        }
        let better = match best {
            None => true,
            Some((cov, th)) => {
                p.coverage_permille > cov || (p.coverage_permille == cov && p.theta < th)
            }
        };
        if better {
            best = Some((p.coverage_permille, p.theta));
        }
    }
    best.map(|(_, theta)| theta)
}

// --- §12: the claim gate ------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaimVerdict {
    NotEstablished,
    Establishable,
}

/// The minimal report summary the claim gate consumes (spec §12).
struct ReportSummary {
    suite_id: &'static str,
    calibrated: bool,
    evidence_axis_measured: bool,
    at_frozen_operating_point: bool,
    false_answer_ucb_permille: Option<u32>,
    false_abstain_permille: Option<u32>,
}

/// A "calibrated semantic abstention" claim is `Establishable` only from a
/// calibrated `s2-answerability-ood` report with measured evidence axes that
/// meets the frozen release targets at a frozen operating point. A
/// coverage-only (D4) report — however green — is `NotEstablished`.
fn semantic_abstention_claim(r: &ReportSummary) -> ClaimVerdict {
    let bound_ok = matches!(
        (r.false_answer_ucb_permille, r.false_abstain_permille),
        (Some(ucb), Some(fa))
            if ucb <= RELEASE_FALSE_ANSWER_UCB_PERMILLE && fa <= RELEASE_FALSE_ABSTAIN_PERMILLE
    );
    if r.suite_id == "s2-answerability-ood"
        && r.calibrated
        && r.evidence_axis_measured
        && r.at_frozen_operating_point
        && bound_ok
    {
        ClaimVerdict::Establishable
    } else {
        ClaimVerdict::NotEstablished
    }
}

// --- helpers for exhaustive grids --------------------------------------------

const THETA: u32 = 500;

fn full_input_grid() -> Vec<(Calibration, Compat, Coverage, Evidence, u32)> {
    let mut grid = Vec::new();
    for calibration in [
        Calibration::Absent,
        Calibration::Valid,
        Calibration::Corrupt,
    ] {
        for compat in [Compat::Compatible, Compat::HardIncompatibility] {
            for coverage in [Coverage::Covered, Coverage::DistributionallyNovel] {
                for evidence in [
                    Evidence::Supported,
                    Evidence::Insufficient,
                    Evidence::Conflicting,
                ] {
                    for q in [0, THETA - 1, THETA, 1000] {
                        grid.push((calibration, compat, coverage, evidence, q));
                    }
                }
            }
        }
    }
    grid
}

/// The seven canonical responses whose encodings must be pairwise distinct on
/// every surface (they exercise all eight §2 statuses across their fields).
fn canonical_responses() -> Vec<Response> {
    vec![
        // legacy served (covered)
        decide(
            Calibration::Absent,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Supported,
            0,
            THETA,
        ),
        // legacy abstention (distributionally-novel cause)
        decide(
            Calibration::Absent,
            Compat::Compatible,
            Coverage::DistributionallyNovel,
            Evidence::Supported,
            0,
            THETA,
        ),
        // calibrated abstention: insufficient evidence
        decide(
            Calibration::Valid,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Insufficient,
            400,
            THETA,
        ),
        // calibrated abstention: conflicting evidence
        decide(
            Calibration::Valid,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Conflicting,
            400,
            THETA,
        ),
        // calibrated abstention: low confidence
        decide(
            Calibration::Valid,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Supported,
            THETA - 1,
            THETA,
        ),
        // calibrated served (supported answer)
        decide(
            Calibration::Valid,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Supported,
            800,
            THETA,
        ),
        // hard incompatibility (corrupt calibration)
        decide(
            Calibration::Corrupt,
            Compat::Compatible,
            Coverage::Covered,
            Evidence::Supported,
            800,
            THETA,
        ),
    ]
}

// --- tests -------------------------------------------------------------------

#[test]
fn status_space_is_total_and_labels_round_trip() {
    // Eight distinct statuses, eight distinct labels, total parse.
    let labels: Vec<&str> = SelectiveStatus::ALL.iter().map(|s| s.label()).collect();
    let unique: std::collections::BTreeSet<&str> = labels.iter().copied().collect();
    assert_eq!(unique.len(), 8, "labels are pairwise distinct");
    for s in SelectiveStatus::ALL {
        assert_eq!(
            SelectiveStatus::parse(s.label()),
            Some(s),
            "label round-trips"
        );
    }
    assert_eq!(SelectiveStatus::parse("something-else"), None);
}

#[test]
fn decision_table_is_total_and_deterministic() {
    for (calibration, compat, coverage, evidence, q) in full_input_grid() {
        let a = decide(calibration, compat, coverage, evidence, q, THETA);
        let b = decide(calibration, compat, coverage, evidence, q, THETA);
        assert_eq!(a, b, "identical inputs, identical outcome");
        // Structural invariants of the table (spec §2):
        match a.outcome {
            Outcome::Abstention => {
                assert!(a.cause.is_some(), "every abstention carries a typed cause");
            }
            Outcome::SupportedAnswer => assert!(a.cause.is_none()),
            Outcome::HardIncompatibility => {
                assert!(a.cause.is_none() && a.confidence_permille.is_none());
            }
        }
        // The legacy cause appears only in legacy mode (spec §6).
        if a.cause == Some(Cause::DistributionallyNovel) {
            assert_eq!(calibration, Calibration::Absent);
        }
        // Confidence is never fabricated outside a valid calibration.
        if calibration != Calibration::Valid {
            assert_eq!(a.confidence_permille, None);
        }
    }
}

#[test]
fn answerable_novelty_is_separable_from_unanswerability() {
    // V3 (spec §2): a distributionally-novel input with supported evidence
    // and passing confidence is SERVED — novelty alone is not
    // unanswerability…
    let novel_supported = decide(
        Calibration::Valid,
        Compat::Compatible,
        Coverage::DistributionallyNovel,
        Evidence::Supported,
        700,
        THETA,
    );
    assert_eq!(novel_supported.outcome, Outcome::SupportedAnswer);
    assert_eq!(novel_supported.coverage, Coverage::DistributionallyNovel);

    // …while a fully covered input with no supporting evidence abstains:
    // coverage alone is not answerability (the #811 conflation, separated).
    let covered_unsupported = decide(
        Calibration::Valid,
        Compat::Compatible,
        Coverage::Covered,
        Evidence::Insufficient,
        700,
        THETA,
    );
    assert_eq!(covered_unsupported.outcome, Outcome::Abstention);
    assert_eq!(covered_unsupported.cause, Some(Cause::InsufficientEvidence));
}

#[test]
fn cross_surface_encodings_round_trip_and_are_injective() {
    let family = canonical_responses();
    // Injectivity per surface: pairwise distinct wire forms.
    let cli: Vec<String> = family.iter().map(encode_cli).collect();
    let http: Vec<(u16, String)> = family.iter().map(encode_http).collect();
    let oai: Vec<(u16, String)> = family.iter().map(encode_openai_nonstream).collect();
    let stream: Vec<Vec<String>> = family.iter().map(encode_openai_stream).collect();
    let wasm: Vec<(u8, String)> = family.iter().map(encode_wasm).collect();
    for i in 0..family.len() {
        for j in (i + 1)..family.len() {
            assert_ne!(cli[i], cli[j], "CLI injective over {i}/{j}");
            assert_ne!(http[i], http[j], "HTTP injective over {i}/{j}");
            assert_ne!(oai[i], oai[j], "OpenAI injective over {i}/{j}");
            assert_ne!(stream[i], stream[j], "stream injective over {i}/{j}");
            assert_ne!(wasm[i], wasm[j], "WASM injective over {i}/{j}");
        }
    }
    // Round-trip: the CLI encoding carries the typed status and cause back.
    for r in &family {
        let (status, cause) = decode_cli(&encode_cli(r));
        assert_eq!(status, r.outcome.status());
        assert_eq!(cause, r.cause.map(Cause::status));
    }
    // Canonical labels appear verbatim in the HTTP body; the OpenAI error
    // code uses the deterministic snake rewrite.
    for r in &family {
        let (_, body) = encode_http(r);
        assert!(body.contains(r.outcome.status().label()));
        if let Some(cause) = r.cause {
            let (_, oai_body) = encode_openai_nonstream(r);
            assert!(oai_body.contains(&snake(cause.status().label())));
        }
    }
}

#[test]
fn abstention_is_never_an_empty_success() {
    for r in canonical_responses() {
        let (code, body) = encode_openai_nonstream(&r);
        let events = encode_openai_stream(&r);
        match r.outcome {
            Outcome::Abstention => {
                assert_ne!(code, 200, "abstention is not an HTTP 200 success");
                assert!(body.contains("\"error\""), "structured error body");
                assert!(
                    body.contains("uor_abstention_"),
                    "typed abstention code, not a generic error"
                );
                assert!(
                    !body.contains("\"choices\""),
                    "no empty-choices success masquerade"
                );
                assert!(
                    events.iter().all(|e| !e.contains("delta")),
                    "no content chunk on an abstained stream"
                );
                assert!(
                    events
                        .first()
                        .is_some_and(|e| e.starts_with("event: error")),
                    "typed terminal stream event"
                );
                assert_eq!(events.last().map(String::as_str), Some("data: [DONE]"));
            }
            Outcome::SupportedAnswer => {
                assert_eq!(code, 200);
                assert!(body.contains("\"choices\""));
            }
            Outcome::HardIncompatibility => {
                assert_eq!(code, 409);
                assert!(body.contains("uor_incompatible_artifact"));
            }
        }
    }
}

#[test]
fn legacy_artifact_without_calibration_stays_coverage_only() {
    // Spec §6: legacy mode never mints an evidence-axis status or a
    // confidence value, whatever the evidence input claims.
    for coverage in [Coverage::Covered, Coverage::DistributionallyNovel] {
        for evidence in [
            Evidence::Supported,
            Evidence::Insufficient,
            Evidence::Conflicting,
        ] {
            for q in [0, 1000] {
                let r = decide(
                    Calibration::Absent,
                    Compat::Compatible,
                    coverage,
                    evidence,
                    q,
                    THETA,
                );
                assert_eq!(r.confidence_permille, None, "no fabricated confidence");
                assert_eq!(r.evidence, None, "no fabricated evidence block");
                assert!(
                    !matches!(
                        r.cause,
                        Some(Cause::InsufficientEvidence)
                            | Some(Cause::ConflictingEvidence)
                            | Some(Cause::LowConfidence)
                    ),
                    "no calibrated cause in legacy mode"
                );
            }
        }
    }
    // And a legacy report cannot establish the calibrated claim (§12).
    let legacy_report = ReportSummary {
        suite_id: "s2-answerability-ood",
        calibrated: false,
        evidence_axis_measured: false,
        at_frozen_operating_point: false,
        false_answer_ucb_permille: None,
        false_abstain_permille: None,
    };
    assert_eq!(
        semantic_abstention_claim(&legacy_report),
        ClaimVerdict::NotEstablished
    );
}

#[test]
fn corrupt_calibration_fails_closed() {
    let good = b"calibration-table-v1: theta 500".to_vec();
    let cid = compute_cid(&good);
    assert_eq!(
        classify_calibration(Some((&cid, &good))),
        Calibration::Valid
    );

    // A single flipped byte is corrupt — hard incompatibility, not legacy.
    let mut tampered = good.clone();
    tampered[0] ^= 0x01;
    assert_eq!(
        classify_calibration(Some((&cid, &tampered))),
        Calibration::Corrupt
    );
    let r = decide(
        Calibration::Corrupt,
        Compat::Compatible,
        Coverage::Covered,
        Evidence::Supported,
        900,
        THETA,
    );
    assert_eq!(r.outcome, Outcome::HardIncompatibility);

    // Absent is legacy (served under D4), NOT hard incompatibility — the
    // corrupt/absent distinction of spec §6, in both directions.
    assert_eq!(classify_calibration(None), Calibration::Absent);
    let legacy = decide(
        Calibration::Absent,
        Compat::Compatible,
        Coverage::Covered,
        Evidence::Supported,
        900,
        THETA,
    );
    assert_eq!(legacy.outcome, Outcome::SupportedAnswer);
    assert_ne!(
        legacy.outcome, r.outcome,
        "corrupt never degrades to legacy"
    );
}

#[test]
fn partitions_reject_leakage_and_tamper() {
    // Three partitions, disjoint on all four axes by construction.
    let cal: Vec<Item> = Category::ALL
        .iter()
        .map(|&c| make_item(c, 0, "cal"))
        .collect();
    let eval: Vec<Item> = Category::ALL
        .iter()
        .map(|&c| make_item(c, 1, "eval"))
        .collect();

    for axis in ["document", "domain", "entity", "template"] {
        let key = |i: &Item| -> String {
            match axis {
                "document" => i.document.clone(),
                "domain" => i.domain.clone(),
                "entity" => i.entity.clone(),
                _ => i.template.clone(),
            }
        };
        let cal_keys: Vec<String> = cal.iter().map(key).collect();
        let eval_keys: Vec<String> = eval.iter().map(key).collect();
        let cal_refs: Vec<&str> = cal_keys.iter().map(String::as_str).collect();
        let eval_refs: Vec<&str> = eval_keys.iter().map(String::as_str).collect();
        assert!(
            detect_document_leakage(&cal_refs, &eval_refs).is_none(),
            "{axis}-disjoint split accepted"
        );
    }

    // A planted leak on the entity axis is detected (positive control for
    // the negative check above).
    let leaked = [cal[0].entity.as_str()];
    let eval_with_leak = [cal[0].entity.as_str(), "entity-eval-other"];
    assert!(
        detect_document_leakage(&leaked, &eval_with_leak).is_some(),
        "planted entity leak detected"
    );

    // Gold annotations are CID-bound; a flipped byte fails verification.
    let annotations: String = eval
        .iter()
        .map(|i| {
            format!(
                "{}|{}|answerable={}\n",
                i.id,
                i.category.label(),
                i.category.answerable()
            )
        })
        .collect();
    let cid = compute_cid(annotations.as_bytes());
    assert!(verify_cid(&cid, annotations.as_bytes()).is_none());
    let mut tampered = annotations.into_bytes();
    tampered[0] ^= 0x01;
    assert!(verify_cid(&cid, &tampered).is_some(), "tamper detected");
}

#[test]
fn planted_category_fixtures_classify_per_gold() {
    // V2: each planted category fixture, read through its reference feature
    // vector, classifies to its gold outcome and an acceptable typed cause.
    for cat in Category::ALL {
        let f = cat.features();
        let r = decide(
            Calibration::Valid,
            Compat::Compatible,
            f.coverage,
            f.evidence,
            f.confidence_permille,
            THETA,
        );
        let (gold_outcome, gold_causes) = cat.gold();
        assert_eq!(r.outcome, gold_outcome, "{} outcome", cat.label());
        match r.cause {
            None => assert!(gold_causes.is_empty(), "{} served", cat.label()),
            Some(c) => assert!(
                gold_causes.contains(&c),
                "{} cause {c:?} is in the gold set",
                cat.label()
            ),
        }
        // Paraphrase stability: the paraphrased twin of an answerable item
        // classifies identically to its in-domain original.
        if cat == Category::ParaphrasedAnswerable {
            let original = Category::InDomainAnswerable.features();
            let ro = decide(
                Calibration::Valid,
                Compat::Compatible,
                original.coverage,
                original.evidence,
                original.confidence_permille,
                THETA,
            );
            assert_eq!(ro.outcome, r.outcome, "paraphrase-stable outcome");
        }
    }
}

#[test]
fn baselines_produce_distinct_confusion_profiles() {
    // V4: the six baselines are pairwise distinguishable by confusion
    // signature over the eight categories.
    let signatures: Vec<(BaselinePolicy, BTreeMap<&str, ConfusionCell>)> = BaselinePolicy::ALL
        .iter()
        .map(|&p| (p, confusion_signature(p)))
        .collect();
    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            assert_ne!(
                signatures[i].1, signatures[j].1,
                "{:?} vs {:?} signatures distinct",
                signatures[i].0, signatures[j].0
            );
        }
    }

    // The #811 finding as a fixture: current-D4 (a coverage policy) falsely
    // serves the covered-but-unanswerable categories…
    let d4 = confusion_signature(BaselinePolicy::CurrentD4);
    for cat in [
        "false-premise",
        "contradictory-evidence",
        "missing-evidence",
    ] {
        let (_, false_answer, _) = d4[cat];
        assert!(false_answer, "current-D4 falsely serves {cat}");
    }
    // …and falsely abstains on answerable novelty.
    assert_eq!(d4["novel-but-supported"], (false, false, true));

    // distance-only cannot separate answerable from unanswerable novelty,
    // and is paraphrase-brittle (its two signatures):
    let dist = confusion_signature(BaselinePolicy::DistanceOnly);
    let (s1, _, _) = dist["novel-but-supported"];
    let (s2, _, _) = dist["unrelated-ood"];
    assert_eq!(s1, s2, "distance-only conflates the two novelty categories");
    assert!(!s1, "distance-only abstains on both");
    assert_eq!(
        dist["paraphrased-answerable"],
        (false, false, true),
        "distance-only falsely abstains on a paraphrase"
    );

    // evidence-count-only serves conflicting evidence (its signature): the
    // conflict structure is present in the features and ignored by the
    // policy, which reads support counts alone.
    let contradictory = Category::ContradictoryEvidence.features();
    assert!(
        contradictory.conflict_count > 0,
        "conflict structure is present to be ignored"
    );
    let ec = confusion_signature(BaselinePolicy::EvidenceCountOnly);
    let (served, false_answer, _) = ec["contradictory-evidence"];
    assert!(
        served && false_answer,
        "count-only ignores conflict structure"
    );

    // always-serve/always-decline are the coverage ceiling and risk floor:
    let always_serve = confusion_signature(BaselinePolicy::AlwaysServe);
    let always_decline = confusion_signature(BaselinePolicy::AlwaysDecline);
    for cat in Category::ALL {
        let (s, _, fa) = always_serve[cat.label()];
        assert!(s && !fa);
        let (s, f, _) = always_decline[cat.label()];
        assert!(!s && !f);
    }

    // #832 binding: the reference primary separates from the always-serve
    // control on false-answer rate (non-degenerate), and the ControlKind
    // vocabulary names the bound controls.
    let primary_false_answer = MetricStatus::Measured {
        numerator: 0,
        denominator: 5,
    };
    let always_serve_false_answer = MetricStatus::Measured {
        numerator: 5,
        denominator: 5,
    };
    assert!(!is_degenerate_control(
        &primary_false_answer,
        &always_serve_false_answer,
        50
    ));
    assert!(is_degenerate_control(
        &primary_false_answer,
        &primary_false_answer,
        50
    ));
    for kind in [
        ControlKind::AlwaysServe,
        ControlKind::AlwaysDecline,
        ControlKind::TrivialPrior,
    ] {
        assert!(ControlKind::ALL.contains(&kind));
    }
}

#[test]
fn power_and_ucb_targets_fixed_before_fitting() {
    // The powered sample sizes and bound targets are compile-time constants
    // of this spec, fixed before any calibrator fit (#837).
    assert_eq!(N_TOTAL, N_PER_CATEGORY * Category::ALL.len() as u64);
    let n_unanswerable =
        N_PER_CATEGORY * Category::ALL.iter().filter(|c| !c.answerable()).count() as u64;
    assert_eq!(n_unanswerable, 3_000);

    // Rule-of-three arithmetic (spec §9): at zero failures the design
    // resolves the release target with headroom…
    assert_eq!(ucb95_permille(0, n_unanswerable), 1);
    assert!(ucb95_permille(0, n_unanswerable) <= RELEASE_FALSE_ANSWER_UCB_PERMILLE);
    // …the minimum zero-failure sample for the release bound is n = 300…
    assert_eq!(ucb95_permille(0, 300), 10);
    assert!(ucb95_permille(0, 299) > RELEASE_FALSE_ANSWER_UCB_PERMILLE);
    // …and the research bound is the declared looser target. The frozen
    // looser-than relation is checked through a function so the assertion
    // reads the same values the selection rule consumes.
    assert_eq!(ucb95_permille(0, 60), RESEARCH_FALSE_ANSWER_UCB_PERMILLE);
    fn strictly_looser(research: u32, release: u32) -> bool {
        research > release
    }
    assert!(strictly_looser(
        RESEARCH_FALSE_ANSWER_UCB_PERMILLE,
        RELEASE_FALSE_ANSWER_UCB_PERMILLE
    ));
    assert!(strictly_looser(
        RESEARCH_FALSE_ABSTAIN_PERMILLE,
        RELEASE_FALSE_ABSTAIN_PERMILLE
    ));

    // Category-level resolution at n = 600: a 100‰ rate carries a 95%
    // half-width of ±24‰, so ≥50‰ baseline-profile differences resolve.
    let n = N_PER_CATEGORY as f64;
    let half_width_permille = 1.96 * (0.1_f64 * 0.9 / n).sqrt() * 1000.0;
    assert!(half_width_permille < 25.0, "±{half_width_permille:.1}");

    // Metric encoding is the #832 integer-fraction vocabulary: a missing
    // fixture is Unavailable, never a vacuous Measured zero.
    let unavailable = MetricStatus::Unavailable {
        reason: "calibration partition absent".to_owned(),
    };
    assert!(!unavailable.is_measured());
    assert_eq!(unavailable.rate_permille(), None);
}

#[test]
fn operating_points_frozen_and_selection_rule_deterministic() {
    // A synthetic calibration curve: higher θ serves less, errs less.
    let curve = [
        CurvePoint {
            theta: 200,
            false_answers: 60,
            n_unanswerable: 3_000,
            coverage_permille: 950,
        },
        CurvePoint {
            theta: 400,
            false_answers: 120,
            n_unanswerable: 3_000,
            coverage_permille: 900,
        },
        CurvePoint {
            theta: 600,
            false_answers: 20,
            n_unanswerable: 3_000,
            coverage_permille: 700,
        },
        CurvePoint {
            theta: 800,
            false_answers: 3,
            n_unanswerable: 3_000,
            coverage_permille: 450,
        },
        CurvePoint {
            theta: 900,
            false_answers: 0,
            n_unanswerable: 3_000,
            coverage_permille: 250,
        },
    ];
    // Release (UCB ≤ 10‰): θ=600 (ucb (20·1000+3000)/3000 = 8‰), θ=800
    // (2‰), and θ=900 (1‰) qualify; θ=200 (21‰) and θ=400 (41‰) do not.
    // The rule maximizes coverage among qualifiers → θ=600 (700‰).
    let release = select_operating_point(&curve, RELEASE_FALSE_ANSWER_UCB_PERMILLE);
    assert_eq!(release, Some(600));
    // Research (UCB ≤ 50‰): every point qualifies (21/41/8/2/1‰); max
    // coverage → θ=200 (950‰).
    let research = select_operating_point(&curve, RESEARCH_FALSE_ANSWER_UCB_PERMILLE);
    assert_eq!(research, Some(200));
    // Research serves strictly more than release (the two points differ and
    // are both frozen before fitting).
    assert!(research.unwrap() < release.unwrap());
    // Deterministic under repetition and input reversal.
    let mut reversed = curve;
    reversed.reverse();
    assert_eq!(
        select_operating_point(&reversed, RELEASE_FALSE_ANSWER_UCB_PERMILLE),
        release
    );
    // An unmeetable bound is a typed None — the target is never relaxed.
    assert_eq!(select_operating_point(&curve, 0), None);
}

#[test]
fn coverage_result_cannot_render_semantic_pass() {
    // A D4/coverage-only report — however green — is NOT ESTABLISHED.
    let coverage_only = ReportSummary {
        suite_id: "s2-answerability-ood",
        calibrated: false,
        evidence_axis_measured: false,
        at_frozen_operating_point: true,
        false_answer_ucb_permille: Some(0),
        false_abstain_permille: Some(0),
    };
    assert_eq!(
        semantic_abstention_claim(&coverage_only),
        ClaimVerdict::NotEstablished
    );
    // A different suite cannot establish it either.
    let wrong_suite = ReportSummary {
        suite_id: "s0-broad-text",
        calibrated: true,
        evidence_axis_measured: true,
        at_frozen_operating_point: true,
        false_answer_ucb_permille: Some(1),
        false_abstain_permille: Some(100),
    };
    assert_eq!(
        semantic_abstention_claim(&wrong_suite),
        ClaimVerdict::NotEstablished
    );
    // Missing either frozen target is NOT ESTABLISHED (positive control
    // included so the negative reading is non-vacuous).
    let over_bound = ReportSummary {
        suite_id: "s2-answerability-ood",
        calibrated: true,
        evidence_axis_measured: true,
        at_frozen_operating_point: true,
        false_answer_ucb_permille: Some(RELEASE_FALSE_ANSWER_UCB_PERMILLE + 1),
        false_abstain_permille: Some(100),
    };
    assert_eq!(
        semantic_abstention_claim(&over_bound),
        ClaimVerdict::NotEstablished
    );
    let establishable = ReportSummary {
        suite_id: "s2-answerability-ood",
        calibrated: true,
        evidence_axis_measured: true,
        at_frozen_operating_point: true,
        false_answer_ucb_permille: Some(2),
        false_abstain_permille: Some(150),
    };
    assert_eq!(
        semantic_abstention_claim(&establishable),
        ClaimVerdict::Establishable
    );
}

#[test]
fn witness_carries_the_same_typed_fields() {
    for r in canonical_responses() {
        let w = witness_of(&r);
        assert_eq!(w.outcome, r.outcome);
        assert_eq!(w.coverage, r.coverage);
        assert_eq!(w.cause, r.cause);
        assert_eq!(w.confidence_permille, r.confidence_permille);
        assert_eq!(w.evidence, r.evidence);
    }
}

#[test]
fn double_run_and_reordered_input_determinism() {
    // The full grid, run twice, encodes identically on every surface.
    let grid = full_input_grid();
    let run = |g: &[(Calibration, Compat, Coverage, Evidence, u32)]| -> Vec<String> {
        g.iter()
            .map(|&(calibration, compat, coverage, evidence, q)| {
                let r = decide(calibration, compat, coverage, evidence, q, THETA);
                let (code, body) = encode_http(&r);
                format!("{code} {body} | {}", encode_cli(&r))
            })
            .collect()
    };
    assert_eq!(run(&grid), run(&grid), "double run identical");

    // Aggregation over reversed input order reduces identically (ordered
    // BTreeMap reduction, the #832 shard rule in miniature).
    let aggregate = |g: &[(Calibration, Compat, Coverage, Evidence, u32)]| {
        let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
        for &(calibration, compat, coverage, evidence, q) in g {
            let r = decide(calibration, compat, coverage, evidence, q, THETA);
            *counts.entry(r.outcome.status().label()).or_insert(0) += 1;
        }
        counts
    };
    let mut reversed = grid.clone();
    reversed.reverse();
    assert_eq!(aggregate(&grid), aggregate(&reversed), "order-invariant");
}
