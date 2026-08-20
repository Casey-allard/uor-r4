//! Typed library façade over the uor-r4 transformerless graph stack.
//!
//! Two boundaries, both byte-oriented and filesystem-free at the library
//! edge (compilation writes only inside the caller-supplied work
//! directory; the engine never touches the filesystem):
//!
//! - [`compile`]: orchestrates the three existing compiler stages
//!   (teacher bundle → multiresolution cover → scored graph) behind a
//!   typed [`CompileRequest`] / [`CompileOutcome`] surface with structured
//!   progress and errors. The stage entry points predate this crate and
//!   take CLI flag strings; the translation lives in private shims inside
//!   [`compile`] and is documented there.
//! - [`engine`]: the deployed R4G1 inference adapter ([`R4Engine`]) —
//!   CID-verified scored graph + teacher artifact loaded from byte
//!   slices, the D4 manifest status policy as data, typed abstention,
//!   and an allocation-free steady-state predict/generate step.
//! - [`serving_eval`]: held-out evaluation of the serving surface —
//!   the certify C row (issue #280) — measuring [`engine::R4Engine`]
//!   with the D4 policy on a compiled bundle's own held-out split.
//! - [`release_bundle`]: versioned release-bundle manifest schema
//!   (#655-C0) — the declared identity a packaged serving bundle carries
//!   (ABI/contract version, pinned `uor-matmul` provenance, component
//!   digests, tokenizer identity). Schema and structural validation only;
//!   no discovery/loading/serving wiring yet (that is #655-C1/D).
//! - [`capability_suite`]: the versioned evaluation constitution (#832) —
//!   committed capability-suite manifests, a comparable report schema,
//!   per-token resolution-path attribution bound to the normative scorer
//!   (ADR-0001 / #831), and the leakage/tamper/CID and degenerate-control
//!   checks that keep a report honest. Schema and structural validation
//!   only; it runs no evaluation.
//!
//! Claim language follows `docs/formal_vocabulary.md`; nothing here
//! strengthens or weakens the guarantees of the underlying crates.

pub mod capability_suite;
pub mod compile;
pub mod engine;
pub mod release_bundle;
pub mod serving_eval;

// `capability_suite::Stage` (programme stage S0–S7) is deliberately NOT
// re-exported here — it would collide with `compile::Stage` (the compile
// pipeline stage). Reach it (and `StageEntry`) via the module path.
pub use capability_suite::{
    builtin_constitution, builtin_manifests, compute_cid, detect_document_leakage,
    is_degenerate_control, verify_cid, AttributionHistogram, CapabilityReport, Constitution,
    ControlKind, ControlReport, MetricReport, MetricStatus, ResolutionPath, ScoringMode,
    SplitRules, SuiteIdentities, SuiteManifest, TokenAttribution, Workload,
    CAPABILITY_REPORT_SCHEMA, CAPABILITY_SUITE_SCHEMA, NORMATIVE_SCORER_ID,
};

pub use compile::{
    compile, CompileOptions, CompileOutcome, CompileProvenance, CompileRequest, CompiledModel,
    ComponentDigests, ProgressEvent, QualityProfile, ResumeHint, ScoringOptions,
    SourceExecutionIdentity, Stage, TokenizerAdapter, TokenizerAdapterKey,
};
pub use engine::{
    validate_quality_report, AbiVersion, AbstainOutcome, EngineParts, GenerateStatus,
    InferenceRequest, InferenceResponse, InferenceWitness, PolicyCounters, PolicyStatus,
    PredictDecision, PredictOutcome, PredictOutput, R4Engine, ResolutionStatus, StatusAction,
    StatusPolicy, WitnessVerificationError,
};
pub use release_bundle::{
    BundleAbi, BundleCapability, BundleComponentDigests, ReleaseBundleManifest,
    UorMatmulProvenance, RELEASE_BUNDLE_MANIFEST_SCHEMA,
};

// The bytes-based tokenizer the engine's text helpers use
// (`Tokenizer::from_bytes`) and the full identity carried by tagged runtime
// tokenizers; re-exported so downstream consumers bind the same vocabulary
// and registered host adapter the bundle was compiled with.
pub use uor_r4_core::transformerless::scenarios::{RuntimeTokenizerIdentity, Tokenizer};
pub use uor_r4_graph_certify::ScoreStatus;
// The sanctioned host-source failure type `compile` and `R4Engine::load`
// return (R5): re-exported so consumers name it without depending on
// `uor-r4-model-source` directly.
pub use uor_r4_model_source::SourceUnavailable;
