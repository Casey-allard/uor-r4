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
//!
//! Claim language follows `docs/formal_vocabulary.md`; nothing here
//! strengthens or weakens the guarantees of the underlying crates.

pub mod compile;
pub mod engine;
pub mod release_bundle;
pub mod serving_eval;

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
