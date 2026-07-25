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
//!
//! Claim language follows `docs/formal_vocabulary.md`; nothing here
//! strengthens or weakens the guarantees of the underlying crates.

pub mod compile;
pub mod engine;

pub use compile::{
    compile, CompileError, CompileOptions, CompileOutcome, CompileProvenance, CompileRequest,
    CompiledModel, ComponentDigests, ProgressEvent, QualityProfile, ResumeHint, ScoringOptions,
    Stage,
};
pub use engine::{
    validate_quality_report, AbiVersion, AbstainOutcome, EngineParts, GenerateStatus,
    InferenceError, LoadError, PolicyCounters, PolicyStatus, PredictDecision, PredictOutcome,
    PredictOutput, R4Engine, ResolutionStatus, StatusAction, StatusPolicy,
};

// The bytes-based tokenizer the engine's text helpers use
// (`Tokenizer::from_bytes`); re-exported so downstream consumers encode
// and decode against the same vocabulary the bundle was compiled with.
pub use uor_r4_core::transformerless::scenarios::Tokenizer;
pub use uor_r4_graph_certify::ScoreStatus;
