//! Native R4G1 graph-runtime adapter used by the HTTP server.
//!
//! The graph scorer is intentionally kept separate from the exploratory f64
//! router. It derives the packed input signature with the transformerless
//! artifact, then selects a token from the validated R4G1 graph.
//!
//! This module is the path-based wrapper over the library engine in
//! `uor-r4-api::engine` (moved there so library consumers load from byte
//! slices). All policy types are re-exports; behavior is unchanged.
//!
//! # ResolutionStatus-driven behavior (issue #78, decision D4)
//!
//! Every prediction carries a resolution status and the deployed behavior is
//! declared as data in [`StatusPolicy`] (the D4 manifest policy):
//!
//! | status | default action |
//! |---|---|
//! | `exact_context` | [`StatusAction::Serve`] |
//! | `graph` | [`StatusAction::Serve`] |
//! | `novel` | [`StatusAction::WidenOnce`] (then abstain) |
//! | `contradictory` | [`StatusAction::Abstain`] (reserved; the scorer does not produce it yet) |
//!
//! `WidenOnce` retries the prediction with the per-depth membership set
//! widened to `WIDENED_TOP_M` exactly once; a signature still Novel after
//! widening is remembered in a bounded FIFO so identical probes abstain
//! without widening again (threat model: fallback denial-of-service).
//! Abstention is a typed outcome — no token is emitted, none is guessed,
//! and the server surfaces the status. An optional override is read from
//! the graph's `score_report.json` (`config.status_policy`, e.g.
//! `{"novel": "abstain"}` with values `serve` / `widen_once` / `abstain`);
//! absent or invalid rows keep the defaults.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use uor_r4_api::engine::{EngineParts, InferenceWitness, R4Engine, WitnessVerificationError};
use uor_r4_core::transformerless::compiler::SIG_BYTES;
use uor_r4_core::transformerless::scenarios::Tokenizer;

// The deployed status-policy types and the status-aware decision path are
// re-exports of the library engine (the integer-only delimited block the
// status-policy suite source-scans lives in
// `crates/uor-r4-api/src/engine.rs`).
pub use uor_r4_api::engine::{
    validate_quality_report, AbstainOutcome, GenerateStatus, PolicyCounters, PolicyStatus,
    PredictDecision, PredictOutcome, StatusAction, StatusPolicy,
};

/// A loaded, CID-verified scored graph and the teacher artifact needed to
/// derive input signatures from token ids. Thin `&self` façade over
/// [`R4Engine`] for the server's shared-state access pattern.
pub struct R4g1State {
    engine: RefCell<R4Engine>,
    /// The bundle tokenizer, loaded with the historical fallback chain
    /// (`Tokenizer::try_load` probes sibling vocab.json candidates).
    tokenizer: Option<Tokenizer>,
}

impl R4g1State {
    /// The manifest policy in force (D4 defaults or the score-report
    /// override).
    pub fn policy(&self) -> StatusPolicy {
        self.engine.borrow().policy()
    }

    /// A snapshot of the status-path counters.
    pub fn policy_counters(&self) -> PolicyCounters {
        self.engine.borrow().policy_counters()
    }

    /// The D4 policy decision for one input signature.
    pub fn predict_signature_status(
        &self,
        sig: &[u8; SIG_BYTES],
    ) -> Result<PredictDecision, String> {
        Ok(self.engine.borrow_mut().predict_signature_status(sig))
    }

    /// Score one token window through the D4 policy.
    pub fn predict_window_status(&self, window: &[u32]) -> Result<PredictDecision, String> {
        self.engine
            .borrow_mut()
            .predict_decision(window)
            .map_err(|error| error.to_string())
    }

    /// Derive the artifact-backed sign signature for a token window. This is
    /// exposed for certifier-side research candidates such as issue #290's
    /// low-rank far-field scorer; serving continues through the normal policy
    /// path above.
    pub fn signature_for_window(&self, window: &[u32]) -> Result<[u8; SIG_BYTES], String> {
        self.engine
            .borrow()
            .signature_for_window(window)
            .map_err(|error| error.to_string())
    }

    /// Generate a greedy continuation with per-step policy decisions:
    /// stops at the first abstention (returning the count so far and
    /// the abstaining status) and never emits a guessed token.
    pub fn generate_into_status(
        &self,
        seed: &[u32],
        out: &mut [u32],
    ) -> Result<GenerateStatus, String> {
        self.engine
            .borrow_mut()
            .generate_into(seed, out)
            .map_err(|error| error.to_string())
    }

    /// Witness-enabled generation for the opt-in proof-carrying response
    /// envelope. The ordinary generation method remains allocation-free.
    pub fn generate_into_status_with_witness(
        &self,
        seed: &[u32],
        out: &mut [u32],
        witnesses: &mut Vec<InferenceWitness>,
    ) -> Result<GenerateStatus, String> {
        self.engine
            .borrow_mut()
            .generate_into_with_witness(seed, out, witnesses)
            .map_err(|error| error.to_string())
    }

    /// Replay a compact response witness against the loaded artifact.
    pub fn verify_witnesses(
        &self,
        seed: &[u32],
        generated: &[u32],
        witnesses: &[InferenceWitness],
    ) -> Result<(), WitnessVerificationError> {
        match self
            .engine
            .borrow_mut()
            .verify_witnesses(seed, generated, witnesses)
        {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Load and validate a scored graph. The teacher artifact supplies the
    /// compressed token rows used to derive input signatures. EXCT is not
    /// enabled because its reference implementation performs probe-time
    /// floating-point quantization.
    pub fn load(graph_path: &Path, teacher_path: &Path) -> Result<Self, String> {
        let graph_bytes = std::fs::read(graph_path)
            .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        let teacher_bytes = std::fs::read(teacher_path)
            .map_err(|error| format!("{}: {error}", teacher_path.display()))?;
        // Historical behavior: a score report that does not parse is
        // ignored (D4 defaults), not an error — pre-validate before
        // handing the bytes to the typed loader.
        let score_report = graph_path
            .parent()
            .and_then(|parent| std::fs::read(parent.join("score_report.json")).ok())
            .filter(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).is_ok());
        let engine = R4Engine::load(EngineParts {
            graph: &graph_bytes,
            signature_artifact: &teacher_bytes,
            // The engine's own text helpers are unused here: this wrapper
            // keeps the historical tokenizer fallback chain below.
            tokenizer: None,
            score_report: score_report.as_deref(),
        })
        .map_err(|error| {
            // The engine loader now returns a single sanctioned
            // SourceUnavailable whose reason names the failing part; attribute
            // teacher-artifact reasons to the teacher path, the rest to the
            // graph path.
            let path = if error.reason.contains("teacher") {
                teacher_path.display()
            } else {
                graph_path.display()
            };
            format!("{path}: {error}")
        })?;
        let tokenizer = teacher_path
            .parent()
            .map(|parent| parent.join("tokenizer.bin"))
            .filter(|path| path.is_file())
            .and_then(|path| Tokenizer::try_load(path).ok());

        Ok(Self {
            engine: RefCell::new(engine),
            tokenizer,
        })
    }

    /// Encode with the bundle-matched tokenizer when one is available.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        self.tokenizer.as_ref()?.encode_into(text, out)
    }

    /// Decode with the bundle-matched tokenizer when one is available.
    pub fn decode_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out)
    }

    /// Score one token window using the validated graph artifact.
    ///
    /// Delegates to [`Self::predict_window_status`] and discards the
    /// status: served predictions return their token; a policy
    /// abstention is an error here — no guessed token is emitted.
    pub fn predict_window(&self, window: &[u32]) -> Result<u32, String> {
        match self.predict_window_status(window)? {
            PredictDecision::Serve(outcome) => Ok(outcome.token),
            PredictDecision::Abstain(outcome) => Err(format!(
                "R4G1 policy abstained (status: {})",
                PolicyStatus::from(outcome.status).label()
            )),
        }
    }

    /// Generate a greedy continuation from a token seed. This mirrors the
    /// legacy runtime's fixed-width window behavior while replacing its
    /// graded-store lookup with R4G1 graph scoring. Delegates to
    /// [`Self::generate_into_status`] and discards the status fields; on
    /// a policy abstention the tokens generated so far are returned.
    pub fn generate_into(&self, seed: &[u32], out: &mut [u32]) -> Result<usize, String> {
        Ok(self.generate_into_status(seed, out)?.count)
    }
}

/// Resolve the graph path from an explicit setting or the conventional
/// compiled-bundle location beside `tless_artifacts.bin`.
pub fn discover_path(explicit: Option<&str>, teacher_path: &Path) -> Option<PathBuf> {
    explicit.map(PathBuf::from).or_else(|| {
        teacher_path
            .parent()
            .map(|parent| parent.join("graph/score.r4g1"))
    })
}
