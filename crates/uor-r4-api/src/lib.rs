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

#[cfg(feature = "learned-reference")]
pub mod learned_reference;

#[cfg(feature = "full")]
pub mod capability_suite;
#[cfg(feature = "full")]
pub mod compile;
pub mod deployed_quality;
pub mod engine;
pub mod production_envelope;
pub mod release_bundle;
pub mod serving;
#[cfg(feature = "full")]
pub mod serving_eval;
pub mod witness_replay;

// `capability_suite::Stage` (programme stage S0–S7) is deliberately NOT
// re-exported here — it would collide with `compile::Stage` (the compile
// pipeline stage). Reach it (and `StageEntry`) via the module path.
#[cfg(feature = "full")]
pub use capability_suite::{
    builtin_constitution, builtin_manifests, compute_cid, detect_document_leakage,
    is_degenerate_control, verify_cid, AttributionHistogram, CapabilityReport, Constitution,
    ControlKind, ControlReport, MetricReport, MetricStatus, ResolutionPath, ScoringMode,
    SplitRules, SuiteIdentities, SuiteManifest, TokenAttribution, Workload,
    CAPABILITY_REPORT_SCHEMA, CAPABILITY_SUITE_SCHEMA, NORMATIVE_SCORER_ID,
};

#[cfg(feature = "full")]
pub use compile::{
    compile, CompileOptions, CompileOutcome, CompileProvenance, CompileRequest, CompiledModel,
    ComponentDigests, ProgressEvent, QualityProfile, ResumeHint, ScoringOptions,
    SourceExecutionIdentity, Stage,
};
pub use deployed_quality::{
    deployed_quality_positions_cid, derive_deployed_quality_bindings, is_blake3_cid,
    parse_deployed_quality_for_research, ActiveSectionIdentity, ActiveSectionSetIdentity,
    ArtifactIdentity, ComparatorIdentity, CompilerIdentity, CorpusIdentity, DecodeIdentity,
    DecodeMode, DeployedQualityBindingMaterial, DeployedQualityBindings, DeployedQualityReport,
    DeployedQualityValidationError, EvaluationEvidence, EvaluationMode, ExactRate, ExactSignedRate,
    NegativeControlEvidence, NegativeControlVerdict, PairedComparison, PairedCounts,
    PairedInterval, PartitionIdentity, PositionSelectionMode, QualityMeasurements,
    QualityProfileIdentity, QualityTokenizerIdentity, QualityVerdict,
    ResearchDeployedQualityReport, SampleDecisionKind, SeedIdentity, SelectorIdentity,
    WitnessReplayEvidence, DEPLOYED_QUALITY_PROFILE_ID, DEPLOYED_QUALITY_PROFILE_VERSION,
    DEPLOYED_QUALITY_REPORT_SCHEMA, LABEL_SHUFFLED_CONTROL_ID, NORMATIVE_EXECUTION_SCOPE,
    NORMATIVE_SELECTOR_ID, NORMATIVE_SELECTOR_SEMANTICS_VERSION, PAIRED_INTERVAL_CONFIDENCE_PPM,
    PAIRED_INTERVAL_METHOD, RF31_MIN_LANE_DELTA_PPM, SECTIONS_ABSENT_COMPARATOR_ID,
    SECTIONS_ABSENT_COMPARATOR_VERSION, TLA_COMPARATOR_ID, TLA_COMPARATOR_VERSION,
};
pub use engine::{
    validate_quality_report, AbiVersion, AbstainOutcome, EngineParts, GenerateStatus,
    InferenceRequest, InferenceResponse, InferenceWitness, PolicyCounters, PolicyDecision,
    PolicyPermit, PolicyStatus, PredictDecision, PredictOutcome, PredictOutput, R4Engine,
    ResolutionStatus, SegmentLaneWitness, StatusAction, StatusPolicy, WitnessVerificationError,
};
pub use production_envelope::{
    validate_production_evidence_links, verify_production_envelope, ProductionEnvelopeParts,
    ProductionEvidenceParts, VerifiedProductionEnvelope,
};
pub use release_bundle::{
    BundleAbi, BundleCapability, BundleComponentDigests, ReleaseAdmissionIdentity,
    ReleaseBundleManifest, UorMatmulProvenance, LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA,
    RELEASE_BUNDLE_MANIFEST_SCHEMA,
};
pub use serving::{
    load_production_policy_engine, select_sampled_runtime_candidate,
    validate_production_serving_parts, CrossSurfaceDecodeMode, CrossSurfaceDisposition,
    CrossSurfaceParityEvidence, CrossSurfaceParityEvidenceBuilder, CrossSurfaceParityObservation,
    CrossSurfaceParityRecord, NormativeDecline, NormativeServe, NormativeServingDecision,
    NormativeServingEngine, NormativeStepAdapter, ProductionPolicyEngine, ProductionServingParts,
    CROSS_SURFACE_CONTEXT_CAPACITY, CROSS_SURFACE_PARITY_BUNDLE_PATH,
    CROSS_SURFACE_PARITY_EVIDENCE_SCHEMA, CROSS_SURFACE_SESSION_SIGNATURE_CAPACITY,
};
pub use witness_replay::{
    parse_and_validate_normative_witness_replay, produce_normative_witness_replay,
    select_normative_witness_positions, NormativeWitnessCandidate, NormativeWitnessCandidateSource,
    NormativeWitnessDisposition, NormativeWitnessLaneAttribution, NormativeWitnessPolicyStatus,
    NormativeWitnessReplayArtifact, NormativeWitnessReplayMaterial, NormativeWitnessReplayRecord,
    NormativeWitnessReplaySpec, NormativeWitnessReplayVerdict, DEFAULT_NORMATIVE_WITNESS_SAMPLE,
    NORMATIVE_WITNESS_REPLAY_BUNDLE_PATH, NORMATIVE_WITNESS_REPLAY_SCHEMA,
};

// The bytes-based tokenizer the engine's text helpers use
// (`Tokenizer::from_bytes`) and the full identity carried by tagged runtime
// tokenizers; re-exported so downstream consumers bind the same vocabulary
// and registered host adapter the bundle was compiled with.
pub use uor_r4_core::transformerless::hf_bpe::{TokenizerAdapter, TokenizerAdapterKey};
pub use uor_r4_core::transformerless::scenarios::{RuntimeTokenizerIdentity, Tokenizer};
pub use uor_r4_graph_certify::ScoreStatus;
// The sanctioned host-source failure type `compile` and `R4Engine::load`
// return (R5): re-exported so consumers name it without depending on
// `uor-r4-model-source` directly.
pub use uor_r4_model_source::SourceUnavailable;
