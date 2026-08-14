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
use uor_r4_core::transformerless::hf_bpe::{
    resolve_source_tokenizer, TokenizerAdapterKey, TokenizerKind,
};
use uor_r4_core::transformerless::scenarios::{RuntimeTokenizerIdentity, Tokenizer};

fn read_optional_tokenizer(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let is_regular = if metadata.file_type().is_symlink() {
        std::fs::metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(format!(
            "{}: tokenizer path is not a regular file",
            path.display()
        ));
    }
    std::fs::read(path)
        .map(Some)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn read_required_bundle_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let is_regular = if metadata.file_type().is_symlink() {
        std::fs::metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(format!(
            "{}: bundle path is not a regular file",
            path.display()
        ));
    }
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

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
    /// Exact `tokenizer.bin` bytes beside the teacher artifact. Tagged
    /// tokenizers are decode-only; historical untagged bytes retain their
    /// existing encode/decode behavior.
    tokenizer: Option<Tokenizer>,
    /// Exact registered host encoder for a tagged decode-only tokenizer.
    /// It is installed only after all four identity fields match.
    host_tokenizer: Option<TokenizerKind>,
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
        Self::load_with_source(graph_path, teacher_path, None)
    }

    /// Load a scored graph and, when `source_dir` is supplied for a tagged
    /// decode-only tokenizer, bind the exact registered host encoder. A
    /// present source that cannot parse or whose identity differs is a hard
    /// load error; an absent source leaves decoding available and prompt
    /// encoding explicitly unavailable.
    pub fn load_with_source(
        graph_path: &Path,
        teacher_path: &Path,
        source_dir: Option<&Path>,
    ) -> Result<Self, String> {
        let graph_bytes = read_required_bundle_file(graph_path)?;
        let teacher_bytes = read_required_bundle_file(teacher_path)?;
        let tokenizer_path = teacher_path
            .parent()
            .map(|parent| parent.join("tokenizer.bin"));
        let tokenizer_bytes = tokenizer_path
            .as_deref()
            .map(read_optional_tokenizer)
            .transpose()?
            .flatten();
        let tokenizer = tokenizer_bytes
            .as_deref()
            .map(|bytes| {
                Tokenizer::from_bytes(bytes).ok_or_else(|| {
                    format!(
                        "{}: invalid tokenizer.bin bytes",
                        tokenizer_path
                            .as_deref()
                            .unwrap_or_else(|| Path::new("tokenizer.bin"))
                            .display()
                    )
                })
            })
            .transpose()?;
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
            // Supplying the exact bytes makes the graph header's independent
            // tokenizer.bin CID binding authoritative at this boundary.
            tokenizer: tokenizer_bytes.as_deref(),
            score_report: score_report.as_deref(),
        })
        .map_err(|error| {
            // The engine loader now returns a single sanctioned
            // SourceUnavailable whose reason names the failing part; attribute
            // teacher-artifact and tokenizer-binding reasons to their exact
            // bundle paths, and graph-format reasons to the graph path.
            let path = if error.reason.contains("teacher") {
                teacher_path.display()
            } else if error.reason.contains("tokenizer") {
                tokenizer_path
                    .as_deref()
                    .unwrap_or_else(|| Path::new("tokenizer.bin"))
                    .display()
            } else {
                graph_path.display()
            };
            format!("{path}: {error}")
        })?;
        let host_tokenizer = match (tokenizer.as_ref(), source_dir) {
            (Some(runtime), Some(source)) if runtime.is_decode_only() => {
                let identity = runtime.adapter_identity().ok_or_else(|| {
                    "tagged runtime tokenizer omitted its adapter identity".to_owned()
                })?;
                let key = TokenizerAdapterKey::new(identity.family.clone(), identity.version);
                let host = resolve_source_tokenizer(source, Some(&key))
                    .map_err(|error| format!("{}: {error}", source.display()))?;
                let adapter = host.adapter().ok_or_else(|| {
                    format!(
                        "{} resolved to an adapterless tokenizer for {}/{}",
                        source.display(),
                        identity.family,
                        identity.version
                    )
                })?;
                if adapter.family != identity.family
                    || adapter.version != identity.version
                    || adapter.tokenizer_cid != identity.tokenizer_cid
                    || adapter.adapter_digest != identity.adapter_digest
                {
                    return Err(format!(
                        "{} tokenizer identity mismatch: runtime requires {}/{} CID {} digest {}; host resolved {}/{} CID {} digest {}",
                        source.display(),
                        identity.family,
                        identity.version,
                        identity.tokenizer_cid,
                        identity.adapter_digest,
                        adapter.family,
                        adapter.version,
                        adapter.tokenizer_cid,
                        adapter.adapter_digest,
                    ));
                }
                Some(host)
            }
            _ => None,
        };

        Ok(Self {
            engine: RefCell::new(engine),
            tokenizer,
            host_tokenizer,
        })
    }

    /// Encode with the bundle-matched tokenizer when one is available.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        let runtime = self.tokenizer.as_ref()?;
        if !runtime.is_decode_only() {
            return runtime.encode_into(text, out);
        }
        let encoded = self.host_tokenizer.as_ref()?.encode(text);
        if encoded.len() > out.len() {
            return None;
        }
        out[..encoded.len()].copy_from_slice(&encoded);
        Some(encoded.len())
    }

    /// Decode with the bundle-matched tokenizer when one is available.
    pub fn decode_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out)
    }

    /// Whether this bundle supplied explicit tokenizer bytes. Serving uses
    /// this to distinguish a missing legacy tokenizer (where the historical
    /// configured tokenizer may still be consulted) from a present tokenizer
    /// whose failure must never trigger a cross-id-space fallback.
    pub fn has_explicit_tokenizer(&self) -> bool {
        self.tokenizer.is_some()
    }

    /// Whether a tagged decode-only tokenizer is loaded without its exact
    /// registered host encoder. The serving cascade treats this condition as
    /// terminal so it cannot invoke a legacy greedy tokenizer in another id
    /// space.
    pub fn host_encoder_unavailable(&self) -> bool {
        self.tokenizer
            .as_ref()
            .is_some_and(Tokenizer::is_decode_only)
            && self.host_tokenizer.is_none()
    }

    /// Full tagged runtime identity, when the bundle uses a registered
    /// decode-only tokenizer.
    pub fn tokenizer_adapter_identity(&self) -> Option<&RuntimeTokenizerIdentity> {
        self.tokenizer.as_ref()?.adapter_identity()
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

#[cfg(test)]
mod tests {
    use super::{read_optional_tokenizer, read_required_bundle_file};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-r4g1-tokenizer-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp directory");
        dir
    }

    #[test]
    fn optional_tokenizer_distinguishes_absence_from_a_directory() {
        let dir = temp_dir("directory");
        let missing = dir.join("missing-tokenizer.bin");
        assert_eq!(
            read_optional_tokenizer(&missing).expect("genuine absence"),
            None
        );

        let invalid = dir.join("tokenizer.bin");
        std::fs::create_dir(&invalid).expect("directory at tokenizer path");
        let error = read_optional_tokenizer(&invalid)
            .expect_err("a present non-file tokenizer is not absence");
        assert!(error.contains("tokenizer.bin"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn required_bundle_file_rejects_a_directory_without_reading_it() {
        let dir = temp_dir("required-directory");
        let invalid = dir.join("score.r4g1");
        std::fs::create_dir(&invalid).expect("directory at graph path");
        let error = read_required_bundle_file(&invalid)
            .expect_err("present non-file bundle part must fail closed");
        assert!(error.contains("not a regular file"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn optional_tokenizer_rejects_a_dangling_symlink_instead_of_falling_back() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("dangling");
        let invalid = dir.join("tokenizer.bin");
        symlink(dir.join("missing-target.bin"), &invalid).expect("dangling symlink");
        let error = read_optional_tokenizer(&invalid)
            .expect_err("a present dangling tokenizer is not absence");
        assert!(error.contains("tokenizer.bin"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
