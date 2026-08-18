//! Target-operator recompilation certificate (#606): ONE bounded,
//! versioned, machine-readable record that COMPOSES the repository's
//! existing certification surfaces for `R4RouteAttentionV1`
//! recompilation (route-fit/1 fitting + packed lowering) — relating
//! source parity, trace/fit provenance, progressive replacement,
//! runtime bounds, witness replay, and model-quality measurement —
//! without creating a second Gate C, a second teacher-parity harness,
//! or restating any proof obligation.
//!
//! Two separations this module never blurs:
//!
//! - **A compiled artifact never reads as a quality success.** The
//!   certificate's overall quality verdict is a pure function of the
//!   record ([`derive_overall_quality`]) that returns a non-passing
//!   state unless every required row is present AND valid; the passing
//!   value carries a [`QualityPass`] token constructible ONLY by that
//!   derivation, so a missing, blocked, or unavailable prerequisite
//!   makes a passing quality claim unrepresentable rather than merely
//!   discouraged.
//! - **Absence is absence.** Every verdict family supports
//!   `NOT_MEASURED` (nothing attempted), `BLOCKED(reason)` (an
//!   upstream state prevented the measurement — e.g. the #605 exit
//!   rule fired earlier), and `UNAVAILABLE(reason)` (a prerequisite
//!   does not exist, named) as states distinct from `PASS`/`FAIL` and
//!   distinct from any zero value; the defaulted verdict is
//!   `NOT_MEASURED`, never a vacuous pass.
//!
//! ## Composition map (reuse, never duplication)
//!
//! - **Gate C parity rows**: the existing [`GateCMetrics`] type
//!   (`crate::score`, the #307 surface) is embedded verbatim from the
//!   #605 report's teacher/replaced rows — no parallel metric struct
//!   and no second parity harness (`evaluate_gate_c` and the
//!   `score_runtime` reference scorer remain the only ones).
//! - **Fit evidence**: #605's [`RouteFitReport`] and the
//!   [`RunContract`](crate::route_fit_report::RunContract) serialized
//!   inside it are referenced by κ ([`route_fit_report_kappa`]) with
//!   selected rows embedded; nothing here re-runs the ladder or
//!   re-derives fit logic.
//! - **Runtime bounds**: the #605 [`RuntimeChecks`] records are
//!   embedded; the candidate/selection bounds are the declared
//!   constants of `uor-r4-graph-format::route_attention`; per-step
//!   bytes-read and the operation census are that crate's
//!   data-independent closed forms, verified step-by-step by the
//!   embedded checks; the zero-allocation claim stays owned by the
//!   repository allocation census (the embedded `allocation_note`
//!   names it — this module never re-measures allocation).
//! - **Empirical/performance certificates**: `Certificate`,
//!   `PerformanceCertificate`, and `RuntimePerformanceCertificate`
//!   (`crate::certificate`, `crate::performance_certificate`) are
//!   referenced by their own self-CID identity scheme
//!   (`certificate_cid`) when instances exist; where no instance
//!   participates the provenance row carries a typed absence — a
//!   digest is never invented for another type.
//! - **Proof obligations**: linked by the existing proof-matrix
//!   theorem ids with their recorded status tokens
//!   ([`route_attention_obligation_links`]); the obligation logic
//!   stays in `uor-r4-proof-model`. That crate depends on this one,
//!   so its types cannot be imported here — the id/status mirror is
//!   pinned by a proof-model-side test
//!   (`crates/uor-r4-proof-model/tests/proof_matrix_audit.rs`).
//!
//! ## Witness-row semantics (design rule)
//!
//! Replacement comparisons in this certificate bind to OUTCOMES UNDER
//! REPLACEMENT — teacher-forced top-1/top-k agreement and bits/token
//! of the replaced forward, plus independently replayed packed-kernel
//! witnesses — never to per-step path agreement between source and
//! target internals. Measured justification: the historical program
//! measurement recorded in #606 planning found equal task outcomes at
//! roughly 0.1–0.2 per-step path agreement, so a path-agreement gate
//! would have declared equal-outcome runs failures; path agreement may
//! appear as a diagnostic, never as a verdict input.
//!
//! ## Schema discipline
//!
//! `uor-r4-target-operator-certificate/1` follows the #600 versioned-
//! record pattern exactly (`GeometryProjection`, `RouteFitMethod`): a
//! typed record, [`TargetOperatorCertificateSpec::canonical_bytes`] in
//! a pinned line format, a blake3 declared-identity digest over the
//! parameter DECLARATION (never source text), and a registry
//! ([`certificate_spec`]) refusing every unknown `(id, version)` by
//! name on the sanctioned [`SourceUnavailable`] surface.

use serde::{Deserialize, Serialize};

use uor_r4_graph_compiler::route_fit::FitManifest;
use uor_r4_graph_format::route_attention::{ROUTE_MAX_CANDIDATES, ROUTE_MAX_TOP_M};
use uor_r4_model_source::conformance::ConformanceStatus;
use uor_r4_model_source::SourceUnavailable;

use crate::route_fit_report::{
    route_fit_report_kappa, ReplacedHead, RouteFitReport, RuntimeChecks, StageRecord, StageVerdict,
    ROUTE_FIT_CONTRACT_FORMAT, ROUTE_FIT_REPORT_SCHEMA, STAGE_LAYER_RANGE, STAGE_NULL,
    STAGE_ONE_HEAD, STAGE_ONE_LAYER, STAGE_REAL_CORPUS, STAGE_REAL_TEACHER, STAGE_WHOLE_MODEL,
};
use crate::score::GateCMetrics;

/// Schema tag of the certificate record.
pub const TARGET_OPERATOR_CERTIFICATE_SCHEMA: &str = "uor-r4-target-operator-certificate/1";
/// Registry id of the certificate schema.
pub const TARGET_OPERATOR_CERTIFICATE_ID: &str = "target-operator-certificate";
/// Registry version. A behavioral change (families, scopes, quality
/// rule) is a new version, never an in-place edit (#600 discipline).
pub const TARGET_OPERATOR_CERTIFICATE_VERSION: u32 = 1;

/// Scope names, declared order.
pub const SCOPE_HEAD: &str = "head";
pub const SCOPE_LAYER: &str = "layer";
pub const SCOPE_LAYER_RANGE: &str = "layer-range";
pub const SCOPE_MODEL: &str = "model";
pub const SCOPE_REAL_TEACHER: &str = "real-teacher";
pub const SCOPE_REAL_CORPUS: &str = "real-corpus";

/// Every declared scope, fixed certificate order.
pub const CERTIFICATE_SCOPES: [&str; 6] = [
    SCOPE_HEAD,
    SCOPE_LAYER,
    SCOPE_LAYER_RANGE,
    SCOPE_MODEL,
    SCOPE_REAL_TEACHER,
    SCOPE_REAL_CORPUS,
];

/// The scopes a model-quality verdict may bind to. The synthetic
/// scopes are a cheap instrument; a measured model-quality verdict on
/// one of them is an inconsistency that refuses the whole claim.
pub const QUALITY_BEARING_SCOPES: [&str; 2] = [SCOPE_REAL_TEACHER, SCOPE_REAL_CORPUS];

/// Stored overall-quality state tokens.
pub const QUALITY_STATE_PASSING: &str = "PASSING";
pub const QUALITY_STATE_NOT_PASSING: &str = "NOT_PASSING";

/// Declared parameters of the certificate schema — stable machine
/// tokens that enter the canonical digest serialization byte-for-byte.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TargetOperatorCertificateParams {
    /// The five separated verdict families of every scope row.
    pub families: String,
    /// The verdict states every family supports.
    pub verdict_states: String,
    /// The declared scope vocabulary.
    pub scopes: String,
    /// Which scopes a model-quality verdict may bind to.
    pub quality_scopes: String,
    /// The overall-quality derivation rule.
    pub quality_rule: String,
    /// The witness-binding design rule (module docs).
    pub witness_binding: String,
    /// The composition rule (reuse, never duplication).
    pub composition: String,
    /// The absence rule.
    pub absence_rule: String,
}

impl TargetOperatorCertificateParams {
    /// The declared parameters of version 1.
    pub fn v1() -> Self {
        Self {
            families: "source-parity,target-fit,runtime-contract,witness-replay,model-quality"
                .to_owned(),
            verdict_states: "PASS,FAIL,NOT_MEASURED,BLOCKED,UNAVAILABLE".to_owned(),
            scopes: "head,layer,layer-range,model,real-teacher,real-corpus".to_owned(),
            quality_scopes: "real-teacher,real-corpus".to_owned(),
            quality_rule: "not-passing-unless-instrument-valid-and-every-required-identity-\
                           present-and-every-scope-row-present-with-required-families-pass-\
                           and-real-scope-model-quality-pass-and-synthetic-model-quality-\
                           absent-and-runtime-rows-checked-and-provenance-kappas-consistent-\
                           and-no-linked-obligation-unverified"
                .to_owned(),
            witness_binding: "outcomes-under-replacement-never-per-step-path-agreement".to_owned(),
            composition: "reference-existing-surfaces-by-kappa-or-typed-identity-embed-\
                          existing-row-types-never-rerun-never-rederive"
                .to_owned(),
            absence_rule: "not-measured-blocked-unavailable-distinct-from-pass-fail-and-\
                           from-any-zero-value"
                .to_owned(),
        }
    }
}

/// The typed, versioned record of the certificate schema (#606),
/// following the #600 `GeometryProjection` pattern exactly: canonical
/// pinned-line bytes, blake3 declared-identity digest over the
/// parameter declaration (not source text), and a registry that
/// refuses unknown pairs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TargetOperatorCertificateSpec {
    /// Registry id ([`TARGET_OPERATOR_CERTIFICATE_ID`]).
    pub id: String,
    /// Registry version.
    pub version: u32,
    /// The declared parameters.
    pub params: TargetOperatorCertificateParams,
    /// `blake3:<hex>` of [`TargetOperatorCertificateSpec::canonical_bytes`].
    pub declared_digest: String,
}

impl TargetOperatorCertificateSpec {
    /// The version-1 record implemented by
    /// [`assemble_target_operator_certificate`].
    pub fn v1() -> Self {
        let mut record = Self {
            id: TARGET_OPERATOR_CERTIFICATE_ID.to_owned(),
            version: TARGET_OPERATOR_CERTIFICATE_VERSION,
            params: TargetOperatorCertificateParams::v1(),
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// Canonical serialization of the record's declared identity: a
    /// fixed line format, byte-stable by construction (field order and
    /// separators are fixed here, not derived from any serializer).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "{TARGET_OPERATOR_CERTIFICATE_SCHEMA}\n\
             id={}\n\
             version={}\n\
             param.families={}\n\
             param.verdict_states={}\n\
             param.scopes={}\n\
             param.quality_scopes={}\n\
             param.quality_rule={}\n\
             param.witness_binding={}\n\
             param.composition={}\n\
             param.absence_rule={}\n",
            self.id,
            self.version,
            self.params.families,
            self.params.verdict_states,
            self.params.scopes,
            self.params.quality_scopes,
            self.params.quality_rule,
            self.params.witness_binding,
            self.params.composition,
            self.params.absence_rule,
        )
        .into_bytes()
    }

    /// The declared-identity digest: `blake3:<hex>` over
    /// [`TargetOperatorCertificateSpec::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// The versioned schema registry (#606): map `(id, version)` to the
/// record. Every pair outside the registry is refused by name on the
/// sanctioned [`SourceUnavailable`] surface — never guessed, never
/// approximated by a "closest" schema.
pub fn certificate_spec(
    id: &str,
    version: u32,
) -> Result<TargetOperatorCertificateSpec, SourceUnavailable> {
    match (id, version) {
        (TARGET_OPERATOR_CERTIFICATE_ID, TARGET_OPERATOR_CERTIFICATE_VERSION) => {
            Ok(TargetOperatorCertificateSpec::v1())
        }
        _ => Err(SourceUnavailable::new(format!(
            "unknown target-operator certificate schema ({id}, {version}); registered: \
             {TARGET_OPERATOR_CERTIFICATE_ID}/{TARGET_OPERATOR_CERTIFICATE_VERSION}"
        ))),
    }
}

/// One verdict family's state. `NOT_MEASURED` (nothing attempted, the
/// default), `BLOCKED` (an upstream state prevented the measurement,
/// reason named), and `UNAVAILABLE` (a prerequisite does not exist,
/// reason named) are distinct from `PASS`/`FAIL` and from any zero
/// value; a defaulted family is an unmeasured family, never a vacuous
/// pass.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyVerdict {
    /// The family's composed evidence measured PASS.
    #[serde(rename = "PASS")]
    Pass,
    /// The family's composed evidence measured FAIL.
    #[serde(rename = "FAIL")]
    Fail,
    /// Nothing was attempted for this family.
    #[default]
    #[serde(rename = "NOT_MEASURED")]
    NotMeasured,
    /// An upstream state prevented the measurement (e.g. the #605
    /// ladder exited at an earlier stage); the reason names it.
    #[serde(rename = "BLOCKED")]
    Blocked(String),
    /// A prerequisite does not exist; the reason names it.
    #[serde(rename = "UNAVAILABLE")]
    Unavailable(String),
}

impl FamilyVerdict {
    /// Whether this family is in one of the three absence states.
    pub fn is_absent(&self) -> bool {
        matches!(
            self,
            FamilyVerdict::NotMeasured | FamilyVerdict::Blocked(_) | FamilyVerdict::Unavailable(_)
        )
    }
}

/// The identity block: every input/output identity of the composed
/// recompilation, with TYPED absence (the [`FitManifest`] discipline —
/// a genuinely absent identity is `None`, never an empty string
/// pretending to be a κ).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateIdentity {
    /// κ of the teacher source snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_snapshot: Option<String>,
    /// Tokenizer identity (`None` on the synthetic arm — no tokenizer
    /// exists there).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// Adapter identity (which executor produced the traces).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    /// κ of the merged #603 trace-sidecar bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
    /// Declared-identity digest of the #600 geometry record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<String>,
    /// Target operator registry id (e.g. `r4-route-attention`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// Target operator registry version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_version: Option<u32>,
    /// Declared-identity digest of the target operator record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    /// κ of the merged observation-record bytes (corpus identity).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus: Option<String>,
    /// The fitting compiler identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    /// κ of the #605 fit manifest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit_manifest: Option<String>,
    /// κ of the #605 fit report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit_report: Option<String>,
    /// κ of the fitted parameters (the compiled artifact identity).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fitted_params: Option<String>,
}

/// One scope row: the five separated verdict families plus the
/// embedded #307 Gate C parity rows carried verbatim from the #605
/// report. Families are separate instances, never merged — a
/// compilation (runtime) verdict is not a fit verdict is not a quality
/// verdict.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeRow {
    /// Scope name ([`CERTIFICATE_SCOPES`]).
    pub scope: String,
    /// The #605 ladder stage this row composes (empty when the
    /// composed report declares no stage for the scope).
    pub stage: String,
    /// The replaced heads, copied from the stage record.
    pub replaced: Vec<ReplacedHead>,
    /// Source-parity family (the #605 preflight — #599 check types).
    pub source_parity: FamilyVerdict,
    /// Target-fit family (the #605 pre-registered stage gates).
    pub target_fit: FamilyVerdict,
    /// Runtime-contract family (census-vs-closed-form, reference
    /// cross-check, state epoch discipline).
    pub runtime_contract: FamilyVerdict,
    /// Witness-replay family (independent packed-witness replay).
    pub witness_replay: FamilyVerdict,
    /// Model-quality family. Binds only to the quality-bearing scopes;
    /// on a synthetic scope a measured value here is an inconsistency.
    pub model_quality: FamilyVerdict,
    /// Embedded #307 Gate C teacher parity row (existing type, reused).
    pub teacher: Option<GateCMetrics>,
    /// Embedded #307 Gate C replaced-model parity row.
    pub replaced_metrics: Option<GateCMetrics>,
    /// P(recorded teacher argmax within the replaced model's top-k),
    /// copied from the #605 stage record.
    pub top_k_agreement: Option<f64>,
    /// Replaced bits/token divided by teacher bits/token, copied.
    pub bits_per_token_ratio: Option<f64>,
    /// The stage's own reason line, verbatim.
    pub note: String,
}

/// One runtime-bounds row: the declared instance bounds plus the
/// embedded #605 runtime checks. Nothing here is re-measured — the
/// checks are the fit run's, the bounds are the format crate's
/// declared constants, and the census/bytes-read closed forms and the
/// zero-allocation claim stay owned where they already live.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeBoundsRow {
    /// Scope this row covers.
    pub scope: String,
    /// Declared candidate bound (`ROUTE_MAX_CANDIDATES`).
    pub max_candidates: u32,
    /// Declared selection bound (`ROUTE_MAX_TOP_M`).
    pub max_top_m: u32,
    /// Where the per-step bytes-read and operation-census closed forms
    /// are owned and how they were verified.
    pub census_note: String,
    /// The embedded #605 runtime checks (steps, witness replay,
    /// census-vs-closed-form, reference cross-check, epoch discipline,
    /// allocation note). Absent when the composed stage carried none.
    pub checks: Option<RuntimeChecks>,
}

/// One provenance row: a reference to an EXISTING certification
/// surface, by κ when the surface has one, by typed report identity
/// when it does not — a digest is never invented for another type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProvenanceRow {
    /// Short surface name (stable machine token).
    pub surface: String,
    /// The typed report identity (Rust path + format tag where one
    /// exists).
    pub type_identity: String,
    /// κ / self-CID of the concrete instance this certificate
    /// composes, when one exists — typed absence otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kappa: Option<String>,
    /// What the reference means, or why the κ is absent.
    pub note: String,
}

/// One proof-obligation link: the existing proof-matrix theorem id and
/// its recorded status token. The obligation logic stays in
/// `uor-r4-proof-model`; this row only names it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObligationLink {
    /// The existing theorem id
    /// (`uor_r4_proof_model::proof_matrix::TheoremEntry::theorem_id`).
    pub theorem_id: String,
    /// The matrix entry name.
    pub name: String,
    /// The recorded `ProofStatus` token (`Verified` /
    /// `ExecutableSpec` / `DifferentialPass` / `Unverified`).
    pub status: String,
    /// Why this obligation is linked to target-operator recompilation.
    pub note: String,
}

/// The proof obligations linked by a version-1 certificate, with the
/// status tokens the proof matrix records for them. `uor-r4-proof-model`
/// depends on this crate, so its `ProofStatus` type cannot be imported
/// here; these tokens mirror `ProofStatusMatrix::default()` and the
/// mirror is pinned by a proof-model-side test
/// (`crates/uor-r4-proof-model/tests/proof_matrix_audit.rs`) — the
/// obligation logic is never restated here.
pub fn route_attention_obligation_links() -> Vec<ObligationLink> {
    let link = |theorem_id: &str, name: &str, status: &str, note: &str| ObligationLink {
        theorem_id: theorem_id.to_owned(),
        name: name.to_owned(),
        status: status.to_owned(),
        note: note.to_owned(),
    };
    vec![
        link(
            "PDF §16",
            "Allocation Freedom",
            "Verified",
            "the packed route step's zero-allocation steady state is asserted by the \
             repository allocation census; the runtime rows' allocation_note names it",
        ),
        link(
            "Plan §6 / PDF §17",
            "Operation-Set Conformance",
            // #787: mirrors the proof matrix's honest Witnessed status —
            // source-scan evidence until the #160 disassembly audit lands.
            "Witnessed",
            "the multiplication-free operation contract the packed lowering runs under \
             (P-4 source scan; XOR/popcount/add/compare/table-read)",
        ),
        link(
            "Theorem 8",
            "Bounded Ranges",
            "Verified",
            "packed range boundaries behind instance validation; the route instance's \
             hard caps refuse out-of-bound shapes before any step runs",
        ),
        link(
            "PDF §23",
            "Deterministic Top-K",
            "Verified",
            "canonical tie-breaking; the route selection's lowest-candidate-index-on-\
             equal-distance rule is the operator's declared analogue",
        ),
    ]
}

/// The passing token: constructible ONLY by [`derive_overall_quality`]
/// (the field is private to this module), so no caller can materialize
/// a passing overall verdict that the derivation did not compute. Not
/// serializable — the stored record carries state/reason strings that
/// verification recomputes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityPass {
    _sealed: (),
}

/// The derived overall quality verdict of a certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverallQuality {
    /// Every required row is present and valid; carries the sealed
    /// [`QualityPass`] token.
    Passing(QualityPass),
    /// At least one required row is missing, absent, blocked,
    /// unavailable, failed, or inconsistent; the reason names the
    /// first one (fixed evaluation order).
    NotPassing {
        /// The first refusal, named.
        reason: String,
    },
}

impl OverallQuality {
    /// The stored-state token of this verdict.
    pub fn state_token(&self) -> &'static str {
        match self {
            OverallQuality::Passing(_) => QUALITY_STATE_PASSING,
            OverallQuality::NotPassing { .. } => QUALITY_STATE_NOT_PASSING,
        }
    }

    /// The stored-reason line of this verdict.
    pub fn reason_line(&self) -> String {
        match self {
            OverallQuality::Passing(_) => "every required identity, scope row, runtime row, \
                 provenance reference, and obligation link is present and valid under the \
                 registered v1 quality rule"
                .to_owned(),
            OverallQuality::NotPassing { reason } => reason.clone(),
        }
    }
}

/// The #606 target-operator recompilation certificate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TargetOperatorCertificate {
    /// [`TARGET_OPERATOR_CERTIFICATE_SCHEMA`].
    pub schema: String,
    /// Declared-identity digest of the registered spec this instance
    /// follows.
    pub spec_digest: String,
    /// The identity block.
    pub identity: CertificateIdentity,
    /// Anti-vacuity verdict copied from the composed #605 report; a
    /// defaulted record is NOT valid.
    pub instrument_valid: bool,
    /// The instrument-validation (null-stage) record, verbatim.
    pub instrument_note: String,
    /// Scope rows, declared order.
    pub scopes: Vec<ScopeRow>,
    /// Runtime-bounds rows, declared order.
    pub runtime_bounds: Vec<RuntimeBoundsRow>,
    /// Provenance rows, fixed order.
    pub provenance: Vec<ProvenanceRow>,
    /// Proof-obligation links, fixed order.
    pub obligations: Vec<ObligationLink>,
    /// Stored overall-quality state token ([`QUALITY_STATE_PASSING`] /
    /// [`QUALITY_STATE_NOT_PASSING`]), derived at assembly and
    /// recomputed by [`verify_target_operator_certificate`].
    pub overall_quality_state: String,
    /// Stored overall-quality reason line.
    pub overall_quality_reason: String,
}

/// Canonical certificate bytes: ciborium, struct-declaration field
/// order — the certify crate's existing serde byte format (the
/// `Certificate::to_cbor` / #605 report convention). Every float in
/// the record is an embedded finite #605 value, so serialization
/// cannot fail. Assembling the same inputs twice produces
/// byte-identical canonical bytes.
pub fn canonical_target_operator_certificate_bytes(
    certificate: &TargetOperatorCertificate,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(certificate, &mut bytes)
        .expect("target-operator certificate serializes to canonical bytes");
    bytes
}

/// The certificate κ: `blake3:<hex>` over the canonical bytes.
pub fn target_operator_certificate_kappa(certificate: &TargetOperatorCertificate) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_target_operator_certificate_bytes(certificate)).to_hex()
    )
}

// ---------------------------------------------------------------------------
// Family mapping from the #605 stage records (data plumbing only).
// ---------------------------------------------------------------------------

/// Provenance-row surface tokens (fixed).
pub const SURFACE_FIT_REPORT: &str = "route-fit-report";
pub const SURFACE_FIT_CONTRACT: &str = "route-fit-contract";
pub const SURFACE_FIT_MANIFEST: &str = "route-fit-manifest";
pub const SURFACE_FITTED_PARAMS: &str = "route-fit-params";
pub const SURFACE_GATE_C_PARITY: &str = "gate-c-parity";
pub const SURFACE_TEACHER_PARITY_HARNESS: &str = "teacher-parity-harness";
pub const SURFACE_EMPIRICAL_CERTIFICATE: &str = "empirical-certificate";
pub const SURFACE_PERFORMANCE_CERTIFICATE: &str = "performance-certificate";
pub const SURFACE_RUNTIME_PERFORMANCE_CERTIFICATE: &str = "runtime-performance-certificate";
pub const SURFACE_PROOF_OBLIGATIONS: &str = "proof-obligations";

const CENSUS_NOTE: &str = "per-step bytes_read and the operation census are the data-\
     independent closed forms owned by uor-r4-graph-format::route_attention (bytes_read = \
     72*N + 4*M per step, instance bytes only; adds/xors/popcounts/compares/table_reads \
     closed in (N, M)); the embedded checks record that every measured step's census \
     equaled its closed form; the zero-allocation claim is owned by the repository \
     allocation census (see the embedded allocation_note)";

/// The absence state a stage's missing evidence maps to: the #605
/// `NOT_RUN` (exit rule fired earlier) becomes `BLOCKED` with the
/// stage's own reason; `UNAVAILABLE` stays `UNAVAILABLE` with the
/// prerequisite named; anything else is `NOT_MEASURED`.
fn absence_from_stage(record: &StageRecord) -> FamilyVerdict {
    match record.verdict {
        StageVerdict::Unavailable => FamilyVerdict::Unavailable(record.reason.clone()),
        StageVerdict::NotRun => FamilyVerdict::Blocked(record.reason.clone()),
        _ => FamilyVerdict::NotMeasured,
    }
}

fn source_parity_family(record: &StageRecord) -> FamilyVerdict {
    match &record.preflight {
        Some(preflight) => match preflight.status {
            ConformanceStatus::Pass => FamilyVerdict::Pass,
            ConformanceStatus::Fail => FamilyVerdict::Fail,
            ConformanceStatus::Unavailable => FamilyVerdict::Unavailable(format!(
                "source-parity preflight UNAVAILABLE in the composed fit report (stage {})",
                record.stage
            )),
        },
        None => absence_from_stage(record),
    }
}

fn target_fit_family(record: &StageRecord) -> FamilyVerdict {
    match record.verdict {
        StageVerdict::Pass => FamilyVerdict::Pass,
        StageVerdict::Fail => FamilyVerdict::Fail,
        StageVerdict::Unavailable => FamilyVerdict::Unavailable(record.reason.clone()),
        StageVerdict::NotRun => FamilyVerdict::Blocked(record.reason.clone()),
    }
}

fn runtime_contract_family(record: &StageRecord) -> FamilyVerdict {
    match &record.runtime {
        Some(runtime) => {
            if runtime.census_closed_form_pass
                && runtime.reference_crosscheck_pass
                && runtime.state_epoch_pass
            {
                FamilyVerdict::Pass
            } else {
                FamilyVerdict::Fail
            }
        }
        None => absence_from_stage(record),
    }
}

fn witness_replay_family(record: &StageRecord) -> FamilyVerdict {
    match &record.runtime {
        Some(runtime) => {
            if runtime.witness_replay_pass {
                FamilyVerdict::Pass
            } else {
                FamilyVerdict::Fail
            }
        }
        None => absence_from_stage(record),
    }
}

/// The #605 ladder stage each scope composes.
fn stage_name_for_scope(scope: &str) -> &'static str {
    match scope {
        SCOPE_HEAD => STAGE_ONE_HEAD,
        SCOPE_LAYER => STAGE_ONE_LAYER,
        SCOPE_LAYER_RANGE => STAGE_LAYER_RANGE,
        SCOPE_MODEL => STAGE_WHOLE_MODEL,
        SCOPE_REAL_TEACHER => STAGE_REAL_TEACHER,
        SCOPE_REAL_CORPUS => STAGE_REAL_CORPUS,
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Assembly (composition only; nothing is re-run or re-derived).
// ---------------------------------------------------------------------------

/// Assemble the #606 certificate from an existing #605 fit manifest
/// and fit report. Pure composition in fixed order: identities come
/// from the manifest (typed absence preserved), rows from the report's
/// stage records, references by the κs the surfaces already compute.
/// Refuses (sanctioned [`SourceUnavailable`]) a report whose schema is
/// not the registered #605 one or whose embedded fit-manifest κ does
/// not match the manifest handed in — a mismatched or tampered input
/// pair is not a product.
pub fn assemble_target_operator_certificate(
    manifest: &FitManifest,
    report: &RouteFitReport,
) -> Result<TargetOperatorCertificate, SourceUnavailable> {
    if report.schema != ROUTE_FIT_REPORT_SCHEMA {
        return Err(SourceUnavailable::new(format!(
            "cannot certify a report of schema {:?}; this certificate composes \
             {ROUTE_FIT_REPORT_SCHEMA}",
            report.schema
        )));
    }
    let manifest_kappa = manifest.kappa();
    if report.fit_manifest_kappa != manifest_kappa {
        return Err(SourceUnavailable::new(format!(
            "fit-manifest κ mismatch: the report embeds {}, the manifest handed in \
             computes {manifest_kappa}; a mismatched or tampered input pair is not \
             certifiable",
            report.fit_manifest_kappa
        )));
    }
    let spec = TargetOperatorCertificateSpec::v1();
    let report_kappa = route_fit_report_kappa(report);

    let stage = |name: &str| report.stages.iter().find(|record| record.stage == name);

    // The reason a model-quality verdict is unavailable on synthetic
    // scopes: the first quality-bearing arm that did not pass, its
    // reason VERBATIM (the same strings the #605 report carries).
    let real_arm_reason: Option<String> = QUALITY_BEARING_SCOPES
        .iter()
        .filter_map(|scope| stage(stage_name_for_scope(scope)))
        .find(|record| record.verdict != StageVerdict::Pass)
        .map(|record| record.reason.clone());

    let mut scopes = Vec::with_capacity(CERTIFICATE_SCOPES.len());
    let mut runtime_bounds = Vec::with_capacity(CERTIFICATE_SCOPES.len());
    for scope in CERTIFICATE_SCOPES {
        let stage_name = stage_name_for_scope(scope);
        let quality_bearing = QUALITY_BEARING_SCOPES.contains(&scope);
        match stage(stage_name) {
            Some(record) => {
                let model_quality = if quality_bearing {
                    target_fit_family(record)
                } else {
                    // A synthetic scope carries no model-quality
                    // measurement: unavailable, with the real arm's
                    // own reason when one is not passing.
                    FamilyVerdict::Unavailable(real_arm_reason.clone().unwrap_or_else(|| {
                        "model-quality binds to the real-teacher and real-corpus rows; the \
                         synthetic cheap-instrument scope carries no model-quality \
                         measurement"
                            .to_owned()
                    }))
                };
                scopes.push(ScopeRow {
                    scope: scope.to_owned(),
                    stage: record.stage.clone(),
                    replaced: record.replaced.clone(),
                    source_parity: source_parity_family(record),
                    target_fit: target_fit_family(record),
                    runtime_contract: runtime_contract_family(record),
                    witness_replay: witness_replay_family(record),
                    model_quality,
                    teacher: record.teacher.clone(),
                    replaced_metrics: record.replaced_metrics.clone(),
                    top_k_agreement: record.top_k_agreement,
                    bits_per_token_ratio: record.bits_per_token_ratio,
                    note: record.reason.clone(),
                });
                runtime_bounds.push(RuntimeBoundsRow {
                    scope: scope.to_owned(),
                    max_candidates: ROUTE_MAX_CANDIDATES as u32,
                    max_top_m: ROUTE_MAX_TOP_M as u32,
                    census_note: CENSUS_NOTE.to_owned(),
                    checks: record.runtime.clone(),
                });
            }
            None => {
                // Explicit absence beats a silent hole: a scope the
                // composed report never declared is a NOT_MEASURED row.
                scopes.push(ScopeRow {
                    scope: scope.to_owned(),
                    note: "the composed fit report declares no stage for this scope".to_owned(),
                    ..ScopeRow::default()
                });
                runtime_bounds.push(RuntimeBoundsRow {
                    scope: scope.to_owned(),
                    max_candidates: ROUTE_MAX_CANDIDATES as u32,
                    max_top_m: ROUTE_MAX_TOP_M as u32,
                    census_note: CENSUS_NOTE.to_owned(),
                    checks: None,
                });
            }
        }
    }

    let instrument_note = match stage(STAGE_NULL) {
        Some(record) => format!(
            "instrument validation (null stage) verdict {:?}: {}",
            record.verdict, record.reason
        ),
        None => "the composed fit report declares no null (instrument-validation) stage".to_owned(),
    };

    let row =
        |surface: &str, type_identity: &str, kappa: Option<String>, note: &str| ProvenanceRow {
            surface: surface.to_owned(),
            type_identity: type_identity.to_owned(),
            kappa,
            note: note.to_owned(),
        };
    let provenance = vec![
        row(
            SURFACE_FIT_REPORT,
            "uor-r4-graph-certify::route_fit_report::RouteFitReport \
             (uor-r4-route-fit-report/1)",
            Some(report_kappa.clone()),
            "the #605 ladder report every scope row composes; its contract, stage \
             records, and decision record travel under this κ",
        ),
        row(
            SURFACE_FIT_CONTRACT,
            &format!(
                "uor-r4-graph-certify::route_fit_report::RunContract \
                 ({ROUTE_FIT_CONTRACT_FORMAT})"
            ),
            None,
            "the pre-registered run contract is serialized INSIDE the fit report and \
             has no standalone κ; it is bound through the route-fit-report κ — no \
             digest is invented for it here",
        ),
        row(
            SURFACE_FIT_MANIFEST,
            "uor-r4-graph-compiler::route_fit::FitManifest (uor-r4-route-fit-manifest/1)",
            Some(manifest_kappa.clone()),
            "the eight-identity fit manifest; the identity block above copies its \
             typed fields verbatim, absence included",
        ),
        row(
            SURFACE_FITTED_PARAMS,
            "uor-r4-graph-compiler::route_fit::FittedRouteCodes (uor-r4-route-fit-params/1)",
            Some(report.fitted_params_kappa.clone()),
            "the fitted-parameter artifact this certificate is about; compiled, \
             dormant, referenced by no serving path",
        ),
        row(
            SURFACE_GATE_C_PARITY,
            "uor-r4-graph-certify::score::GateCMetrics (#307 Gate C parity row type)",
            None,
            "embedded verbatim in the scope rows from the #605 report; the row type \
             has no κ surface, so it is referenced by its typed report identity — \
             no second Gate C exists",
        ),
        row(
            SURFACE_TEACHER_PARITY_HARNESS,
            "uor-r4-graph-certify::score::evaluate_gate_c + score_runtime reference \
             scorer (#307)",
            None,
            "the only teacher-parity harness; this certificate embeds its row type \
             and never reimplements the evaluation — no second harness exists",
        ),
        row(
            SURFACE_EMPIRICAL_CERTIFICATE,
            "uor-r4-graph-certify::certificate::Certificate (self-CID field \
             certificate_cid, kappa:blake3:<hex>)",
            None,
            "no empirical Certificate instance participates in this composition; when \
             one exists it is referenced by its own certificate_cid self-CID — its \
             checks are never reimplemented here",
        ),
        row(
            SURFACE_PERFORMANCE_CERTIFICATE,
            "uor-r4-graph-certify::performance_certificate::PerformanceCertificate \
             (self-CID field certificate_cid)",
            None,
            "no PerformanceCertificate instance participates in this composition; when \
             one exists it is referenced by its own certificate_cid self-CID",
        ),
        row(
            SURFACE_RUNTIME_PERFORMANCE_CERTIFICATE,
            "uor-r4-graph-certify::performance_certificate::RuntimePerformanceCertificate \
             (#161; self-CID field certificate_cid)",
            None,
            "no RuntimePerformanceCertificate instance participates in this \
             composition; when one exists it is referenced by its own self-CID",
        ),
        row(
            SURFACE_PROOF_OBLIGATIONS,
            "uor-r4-proof-model::proof_matrix::ProofStatusMatrix",
            None,
            "obligations are linked by theorem id + recorded status token in the \
             obligations rows; the matrix type has no κ surface and the obligation \
             logic stays in uor-r4-proof-model",
        ),
    ];

    let mut certificate = TargetOperatorCertificate {
        schema: TARGET_OPERATOR_CERTIFICATE_SCHEMA.to_owned(),
        spec_digest: spec.declared_digest.clone(),
        identity: CertificateIdentity {
            source_snapshot: manifest.source_snapshot.clone(),
            tokenizer: manifest.tokenizer.clone(),
            adapter: manifest.adapter.clone(),
            trace: manifest.trace.clone(),
            geometry: manifest.geometry_identity.clone(),
            operator_id: manifest.operator.as_ref().map(|spec| spec.id.clone()),
            operator_version: manifest.operator.as_ref().map(|spec| spec.version),
            operator: manifest.operator_identity.clone(),
            corpus: manifest.corpus.clone(),
            compiler: manifest.compiler.clone(),
            fit_manifest: Some(manifest_kappa),
            fit_report: Some(report_kappa),
            fitted_params: Some(report.fitted_params_kappa.clone()),
        },
        instrument_valid: report.instrument_valid,
        instrument_note,
        scopes,
        runtime_bounds,
        provenance,
        obligations: route_attention_obligation_links(),
        overall_quality_state: String::new(),
        overall_quality_reason: String::new(),
    };
    let quality = derive_overall_quality(&certificate);
    certificate.overall_quality_state = quality.state_token().to_owned();
    certificate.overall_quality_reason = quality.reason_line();
    Ok(certificate)
}

// ---------------------------------------------------------------------------
// The overall-quality derivation (pure) and verification.
// ---------------------------------------------------------------------------

/// Derive the overall quality verdict from the certificate record — a
/// pure function of everything EXCEPT the stored verdict fields, in a
/// fixed evaluation order whose first miss names the refusal. Passing
/// requires: registered schema/spec; a valid instrument; every
/// identity present (κ-shaped where a κ is claimed); exactly the
/// declared scope rows; every non-quality family PASS everywhere;
/// model-quality PASS on the quality-bearing scopes and ABSENT (never
/// a measured verdict) on the synthetic scopes; checked runtime rows;
/// provenance κs consistent with the identity block; and no linked
/// obligation recorded `Unverified`. Anything less is
/// [`OverallQuality::NotPassing`] — a compiled artifact alone never
/// reads as a quality success.
pub fn derive_overall_quality(certificate: &TargetOperatorCertificate) -> OverallQuality {
    let fail = |reason: String| OverallQuality::NotPassing { reason };
    if certificate.schema != TARGET_OPERATOR_CERTIFICATE_SCHEMA {
        return fail(format!(
            "schema {:?} is not the registered {TARGET_OPERATOR_CERTIFICATE_SCHEMA}",
            certificate.schema
        ));
    }
    let spec = TargetOperatorCertificateSpec::v1();
    if certificate.spec_digest != spec.declared_digest {
        return fail(format!(
            "spec digest {:?} is not the registered v1 declared digest",
            certificate.spec_digest
        ));
    }
    if !certificate.instrument_valid {
        return fail(
            "the composed instrument is not valid (anti-vacuity verdict); no number in \
             the composition may be interpreted as fit or quality evidence"
                .to_owned(),
        );
    }
    let identity = &certificate.identity;
    let required_identities: [(&str, Option<&String>); 12] = [
        ("source_snapshot", identity.source_snapshot.as_ref()),
        ("tokenizer", identity.tokenizer.as_ref()),
        ("adapter", identity.adapter.as_ref()),
        ("trace", identity.trace.as_ref()),
        ("geometry", identity.geometry.as_ref()),
        ("operator_id", identity.operator_id.as_ref()),
        ("operator", identity.operator.as_ref()),
        ("corpus", identity.corpus.as_ref()),
        ("compiler", identity.compiler.as_ref()),
        ("fit_manifest", identity.fit_manifest.as_ref()),
        ("fit_report", identity.fit_report.as_ref()),
        ("fitted_params", identity.fitted_params.as_ref()),
    ];
    for (name, value) in required_identities {
        match value {
            None => {
                return fail(format!(
                    "identity {name} is absent; a model-quality claim requires every \
                     input/output identity present (typed absence refuses the claim, it \
                     never passes vacuously)"
                ));
            }
            Some(value) if value.is_empty() => {
                return fail(format!(
                    "identity {name} is empty; an empty string is not an identity"
                ));
            }
            Some(_) => {}
        }
    }
    if identity.operator_version.is_none() {
        return fail(
            "identity operator_version is absent; a model-quality claim requires the \
             target operator's registry version"
                .to_owned(),
        );
    }
    for (name, value) in [
        ("fit_manifest", identity.fit_manifest.as_ref()),
        ("fit_report", identity.fit_report.as_ref()),
        ("fitted_params", identity.fitted_params.as_ref()),
    ] {
        if let Some(value) = value {
            if !value.starts_with("blake3:") {
                return fail(format!(
                    "identity {name} ({value:?}) is not a blake3 κ; a claimed κ must be one"
                ));
            }
        }
    }

    for scope in CERTIFICATE_SCOPES {
        let rows: Vec<&ScopeRow> = certificate
            .scopes
            .iter()
            .filter(|row| row.scope == scope)
            .collect();
        if rows.len() != 1 {
            return fail(format!(
                "scope {scope} has {} rows; exactly one is required",
                rows.len()
            ));
        }
        let row = rows[0];
        let quality_bearing = QUALITY_BEARING_SCOPES.contains(&scope);
        for (family, verdict) in [
            ("source_parity", &row.source_parity),
            ("target_fit", &row.target_fit),
            ("runtime_contract", &row.runtime_contract),
            ("witness_replay", &row.witness_replay),
        ] {
            if *verdict != FamilyVerdict::Pass {
                return fail(format!(
                    "scope {scope} family {family} is {verdict:?}, not PASS; a missing, \
                     blocked, unavailable, or failed prerequisite refuses the quality \
                     claim"
                ));
            }
        }
        if quality_bearing {
            if row.model_quality != FamilyVerdict::Pass {
                return fail(format!(
                    "scope {scope} family model_quality is {:?}, not PASS; the quality \
                     claim binds to the real-teacher and real-corpus rows",
                    row.model_quality
                ));
            }
        } else if !row.model_quality.is_absent() {
            return fail(format!(
                "scope {scope} carries a measured model_quality verdict ({:?}); the \
                 synthetic cheap-instrument scope may never read as a quality result",
                row.model_quality
            ));
        }
    }
    for row in &certificate.scopes {
        if !CERTIFICATE_SCOPES.contains(&row.scope.as_str()) {
            return fail(format!(
                "scope row {:?} is outside the declared scope vocabulary",
                row.scope
            ));
        }
    }

    for scope in CERTIFICATE_SCOPES {
        let rows: Vec<&RuntimeBoundsRow> = certificate
            .runtime_bounds
            .iter()
            .filter(|row| row.scope == scope)
            .collect();
        if rows.len() != 1 {
            return fail(format!(
                "runtime-bounds scope {scope} has {} rows; exactly one is required",
                rows.len()
            ));
        }
        let row = rows[0];
        if row.max_candidates != ROUTE_MAX_CANDIDATES as u32
            || row.max_top_m != ROUTE_MAX_TOP_M as u32
        {
            return fail(format!(
                "runtime-bounds scope {scope} declares bounds ({}, {}) instead of the \
                 format crate's ({ROUTE_MAX_CANDIDATES}, {ROUTE_MAX_TOP_M})",
                row.max_candidates, row.max_top_m
            ));
        }
        match &row.checks {
            None => {
                return fail(format!(
                    "runtime-bounds scope {scope} carries no embedded runtime checks; an \
                     unchecked scope refuses the quality claim"
                ));
            }
            Some(checks) => {
                if !checks.pass || checks.steps == 0 {
                    return fail(format!(
                        "runtime-bounds scope {scope} checks did not pass (steps {}, \
                         pass {})",
                        checks.steps, checks.pass
                    ));
                }
            }
        }
    }

    for (surface, expected) in [
        (SURFACE_FIT_REPORT, identity.fit_report.as_ref()),
        (SURFACE_FIT_MANIFEST, identity.fit_manifest.as_ref()),
        (SURFACE_FITTED_PARAMS, identity.fitted_params.as_ref()),
    ] {
        let row = certificate
            .provenance
            .iter()
            .find(|row| row.surface == surface);
        match row {
            None => {
                return fail(format!("provenance surface {surface} is absent"));
            }
            Some(row) => {
                if row.kappa.as_ref() != expected {
                    return fail(format!(
                        "provenance surface {surface} κ {:?} disagrees with the identity \
                         block's {:?}",
                        row.kappa, expected
                    ));
                }
            }
        }
    }

    if certificate.obligations.is_empty() {
        return fail("no proof obligations are linked".to_owned());
    }
    for obligation in &certificate.obligations {
        if obligation.status == "Unverified" {
            return fail(format!(
                "linked obligation {:?} ({}) is recorded Unverified",
                obligation.name, obligation.theorem_id
            ));
        }
    }

    OverallQuality::Passing(QualityPass { _sealed: () })
}

/// Verify a certificate's internal consistency: the stored overall-
/// quality state and reason must equal what [`derive_overall_quality`]
/// recomputes from the rows. A mismatch (e.g. a record whose stored
/// state was edited to `PASSING`) is refused on the sanctioned
/// surface — the bytes are not a valid certificate.
pub fn verify_target_operator_certificate(
    certificate: &TargetOperatorCertificate,
) -> Result<(), SourceUnavailable> {
    let derived = derive_overall_quality(certificate);
    if certificate.overall_quality_state != derived.state_token()
        || certificate.overall_quality_reason != derived.reason_line()
    {
        return Err(SourceUnavailable::new(format!(
            "stored overall-quality record ({:?}: {:?}) disagrees with the derivation \
             ({:?}: {:?}); the stored verdict is not the derived one",
            certificate.overall_quality_state,
            certificate.overall_quality_reason,
            derived.state_token(),
            derived.reason_line()
        )));
    }
    Ok(())
}

/// Verify a certificate against the source records it claims to
/// compose: the identity block's fit-manifest / fit-report /
/// fitted-params κs must equal the κs those surfaces compute today. A
/// tampered embedded κ reference is refused on the sanctioned surface
/// with both values named. Also checks internal consistency
/// ([`verify_target_operator_certificate`]).
pub fn verify_certificate_sources(
    certificate: &TargetOperatorCertificate,
    manifest: &FitManifest,
    report: &RouteFitReport,
) -> Result<(), SourceUnavailable> {
    let expectations = [
        (
            "fit_manifest",
            &certificate.identity.fit_manifest,
            manifest.kappa(),
        ),
        (
            "fit_report",
            &certificate.identity.fit_report,
            route_fit_report_kappa(report),
        ),
        (
            "fitted_params",
            &certificate.identity.fitted_params,
            report.fitted_params_kappa.clone(),
        ),
    ];
    for (name, recorded, recomputed) in expectations {
        if recorded.as_deref() != Some(recomputed.as_str()) {
            return Err(SourceUnavailable::new(format!(
                "certificate identity {name} records {recorded:?} but the composed \
                 source computes {recomputed}; a tampered or mismatched κ reference is \
                 not a valid certificate"
            )));
        }
    }
    verify_target_operator_certificate(certificate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_canonical_bytes_are_pinned() {
        let spec = TargetOperatorCertificateSpec::v1();
        let pinned = "uor-r4-target-operator-certificate/1\n\
             id=target-operator-certificate\n\
             version=1\n\
             param.families=source-parity,target-fit,runtime-contract,witness-replay,model-quality\n\
             param.verdict_states=PASS,FAIL,NOT_MEASURED,BLOCKED,UNAVAILABLE\n\
             param.scopes=head,layer,layer-range,model,real-teacher,real-corpus\n\
             param.quality_scopes=real-teacher,real-corpus\n\
             param.quality_rule=not-passing-unless-instrument-valid-and-every-required-identity-present-and-every-scope-row-present-with-required-families-pass-and-real-scope-model-quality-pass-and-synthetic-model-quality-absent-and-runtime-rows-checked-and-provenance-kappas-consistent-and-no-linked-obligation-unverified\n\
             param.witness_binding=outcomes-under-replacement-never-per-step-path-agreement\n\
             param.composition=reference-existing-surfaces-by-kappa-or-typed-identity-embed-existing-row-types-never-rerun-never-rederive\n\
             param.absence_rule=not-measured-blocked-unavailable-distinct-from-pass-fail-and-from-any-zero-value\n";
        assert_eq!(spec.canonical_bytes(), pinned.as_bytes());
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(spec.declared_digest, expected);
        assert_eq!(spec.declared_digest(), expected);
    }

    #[test]
    fn registry_refuses_unknown_id_and_version_by_name() {
        let known = certificate_spec(
            TARGET_OPERATOR_CERTIFICATE_ID,
            TARGET_OPERATOR_CERTIFICATE_VERSION,
        )
        .expect("registered schema");
        assert_eq!(known, TargetOperatorCertificateSpec::v1());
        for (id, version) in [
            (TARGET_OPERATOR_CERTIFICATE_ID, 2u32),
            (TARGET_OPERATOR_CERTIFICATE_ID, 0),
            ("mystery-certificate", 1),
        ] {
            let error =
                certificate_spec(id, version).expect_err("unknown (id, version) is not a product");
            assert!(error.reason.contains(id), "reason names the id: {error}");
            assert!(
                error.reason.contains(&version.to_string()),
                "reason names the version: {error}"
            );
        }
    }

    #[test]
    fn defaulted_family_is_not_measured_and_defaulted_certificate_is_not_passing() {
        // A defaulted family is an unmeasured family, never a vacuous
        // pass; a defaulted certificate derives NOT_PASSING.
        assert_eq!(FamilyVerdict::default(), FamilyVerdict::NotMeasured);
        assert!(FamilyVerdict::default().is_absent());
        let quality = derive_overall_quality(&TargetOperatorCertificate::default());
        assert_eq!(quality.state_token(), QUALITY_STATE_NOT_PASSING);
    }
}
