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

use uor_r4_api::engine::{
    EngineParts, InferenceWitness, R4Engine, ServedCandidateWitness, SkipmixLaneWitness,
    WitnessVerificationError,
};
use uor_r4_api::{
    validate_production_serving_parts, NormativeServingDecision, NormativeStepAdapter,
    ProductionServingParts,
};
use uor_r4_core::transformerless::compiler::{SIG_BYTES, WINDOW};
use uor_r4_core::transformerless::hf_bpe::{
    resolve_source_tokenizer, TokenizerAdapterKey, TokenizerKind,
};
use uor_r4_core::transformerless::scenarios::{RuntimeTokenizerIdentity, Tokenizer};
use uor_r4_core::transformerless::score_q::ScoreQ;
use uor_r4_graph_runtime::{R4G1Runtime, ServedCandidates};

use crate::release_bundle_loader::{
    capture_production_admission, production_bundle_root, verify_production_admission,
    CapturedProductionAdmission,
};

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
    validate_quality_report, AbstainOutcome, GenerateStatus, PolicyCounters, PolicyDecision,
    PolicyPermit, PolicyStatus, PredictDecision, PredictOutcome, StatusAction, StatusPolicy,
};

/// A loaded, CID-verified scored graph and the teacher artifact needed to
/// derive input signatures from token ids. Thin `&self` façade over
/// [`R4Engine`] for the server's shared-state access pattern.
pub struct R4g1State {
    engine: RefCell<R4Engine>,
    /// Exact graph bytes retained so every served candidate can be selected by
    /// the normative `R4G1Runtime`, never by the D4 reference scorer.
    graph: Vec<u8>,
    /// Caller-owned runtime scratch. It is allocated once at load and reset
    /// in place for every candidate query.
    node_scores: RefCell<Vec<ScoreQ>>,
    /// Exact `tokenizer.bin` bytes beside the teacher artifact. Tagged
    /// tokenizers are decode-only; historical untagged bytes retain their
    /// existing encode/decode behavior.
    tokenizer: Option<Tokenizer>,
    /// Exact registered host encoder for a tagged decode-only tokenizer.
    /// It is installed only after all four identity fields match.
    host_tokenizer: Option<TokenizerKind>,
}

fn normative_step(
    engine: &mut R4Engine,
    runtime: &R4G1Runtime<'_>,
    node_scores: &mut [ScoreQ],
    window: &[u32],
) -> Result<NormativeServingDecision, String> {
    NormativeStepAdapter::new_with_reference_policy(engine, runtime, node_scores)
        .select(window, None)
        .map_err(|error| error.to_string())
}

fn runtime_skipmix_witness(candidates: &ServedCandidates) -> Option<SkipmixLaneWitness> {
    candidates
        .attribution()
        .map(|attribution| SkipmixLaneWitness {
            promoted_token: attribution.promoted_token,
            base_token: attribution.base_token,
            boost: attribution.contribution.raw(),
            skmx_contributed: attribution.skmx_contributed,
            psib_contributed: attribution.psib_contributed,
        })
}

fn normative_inference_witness(
    engine: &mut R4Engine,
    runtime: &R4G1Runtime<'_>,
    window: &[u32],
    outcome: PredictOutcome,
    candidates: &ServedCandidates,
) -> Result<InferenceWitness, String> {
    let winner = candidates
        .winner()
        .ok_or_else(|| "normative witness has no runtime candidate".to_owned())?;
    if winner.token != outcome.token {
        return Err("normative witness winner differs from the served token".to_owned());
    }
    let status = PolicyStatus::from(outcome.status).label().to_owned();
    let lane_present = runtime.skipmix_tables_present() != (false, false);
    if lane_present {
        return Ok(InferenceWitness {
            // The served-candidate surface does not claim a reference-scorer
            // traversal. Runtime candidate/score/source/SKMX/PSIB provenance
            // and lane attribution are the complete production claim replayed
            // below.
            region_kappa: None,
            region_id: None,
            depth: 0,
            resolution_status: status,
            engine: "r4g1".to_owned(),
            token: winner.token,
            served_candidate: Some(ServedCandidateWitness::from_runtime(winner)),
            widened: outcome.widened,
            segment_lane: None,
            skipmix_lane: runtime_skipmix_witness(candidates),
        });
    }

    // Research-era artifacts with both learned sections absent retain their
    // historical JSON bytes. The reference scorer contributes token-free
    // region/depth metadata only after agreeing with the independently
    // selected runtime token; divergence fails closed.
    let metadata = engine
        .legacy_witness_metadata_for_runtime_token(window, outcome.widened, winner.token)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| {
            "absent-lane legacy witness metadata disagrees with the normative runtime token"
                .to_owned()
        })?;
    if metadata.resolution_status != status {
        return Err(
            "absent-lane legacy witness status disagrees with the D4 policy status".to_owned(),
        );
    }
    Ok(InferenceWitness {
        region_kappa: metadata.region_kappa,
        region_id: metadata.region_id,
        depth: metadata.depth,
        resolution_status: metadata.resolution_status,
        engine: "r4g1".to_owned(),
        token: winner.token,
        served_candidate: None,
        widened: outcome.widened,
        segment_lane: None,
        skipmix_lane: None,
    })
}

/// One exact, already-captured standalone artifact generation. The server
/// uses this for an explicit non-bundle `--r4g1-artifact` so no inferred
/// ancestor lock or second pathname read can change the generation loaded.
pub(crate) struct CapturedR4g1Bundle {
    pub(crate) graph: Vec<u8>,
    pub(crate) signature_artifact: Vec<u8>,
    pub(crate) tokenizer: Option<Vec<u8>>,
    pub(crate) score_report: Option<Vec<u8>>,
    /// Schema-2 release/report/corpus/config bytes captured from the same
    /// immutable bundle generation. `None` is legal only through the
    /// explicitly named research loader.
    pub(crate) production_admission: Option<CapturedProductionAdmission>,
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

    /// Explicit reference/research D4 decision for one already-derived input
    /// signature. This signature-only seam cannot call `R4G1Runtime` and is
    /// therefore not a production-serving method; production callers supply
    /// token windows to [`Self::predict_window_status`] or generation.
    pub fn predict_signature_status_for_research(
        &self,
        sig: &[u8; SIG_BYTES],
    ) -> Result<PredictDecision, String> {
        Ok(self.engine.borrow_mut().predict_signature_status(sig))
    }

    /// Score one token window through the D4 policy.
    pub fn predict_window_status(&self, window: &[u32]) -> Result<PredictDecision, String> {
        let runtime = R4G1Runtime::parse(&self.graph).map_err(|error| {
            format!("normative R4G1 runtime rejected the loaded graph: {error:?}")
        })?;
        let mut engine = self.engine.borrow_mut();
        let mut node_scores = self.node_scores.borrow_mut();
        match normative_step(&mut engine, &runtime, &mut node_scores, window)? {
            NormativeServingDecision::Serve(serve) => {
                Ok(PredictDecision::Serve(PredictOutcome {
                    token: serve.token,
                    status: serve.status,
                    widened: serve.widened,
                    ngram_hit: serve.ngram_hit,
                }))
            }
            NormativeServingDecision::Abstain(outcome) => {
                Ok(PredictDecision::Abstain(outcome))
            }
            NormativeServingDecision::Decline(_) => Err(
                "D4 permitted a position for which the normative R4G1 runtime produced no candidate"
                    .to_owned(),
            ),
        }
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
        self.generate_normative_into(seed, out, None, None)
    }

    /// #655 decode-default decision (2026-08-19): seeded weighted
    /// sampling over the deployed step scorer's own candidates, through
    /// the same D4 policy path as [`Self::generate_into_status`] —
    /// abstention semantics identical by construction (see
    /// `R4Engine::generate_sampled_into`). This is the server tier's
    /// default decode; greedy remains the `temperature: 0` opt-in and
    /// the witness path's decode (witness claims bind the greedy
    /// selection).
    pub fn generate_sampled_into_status(
        &self,
        seed: &[u32],
        out: &mut [u32],
        rng: &mut uor_r4_core::transformerless::runtime::SampleRng,
    ) -> Result<GenerateStatus, String> {
        self.generate_normative_into(seed, out, Some(rng), None)
    }

    /// Evidence-only greedy seam which captures the exact first decision
    /// returned inside the ordinary generation loop. It does not recompute a
    /// shortlist after the public surface has emitted.
    pub(crate) fn generate_into_status_with_first_step(
        &self,
        seed: &[u32],
        out: &mut [u32],
    ) -> Result<(GenerateStatus, Option<NormativeServingDecision>), String> {
        let mut first_step = None;
        let status = self.generate_normative_into(seed, out, None, Some(&mut first_step))?;
        Ok((status, first_step))
    }

    /// Sampled counterpart of [`Self::generate_into_status_with_first_step`].
    pub(crate) fn generate_sampled_into_status_with_first_step(
        &self,
        seed: &[u32],
        out: &mut [u32],
        rng: &mut uor_r4_core::transformerless::runtime::SampleRng,
    ) -> Result<(GenerateStatus, Option<NormativeServingDecision>), String> {
        let mut first_step = None;
        let status = self.generate_normative_into(seed, out, Some(rng), Some(&mut first_step))?;
        Ok((status, first_step))
    }

    fn generate_normative_into(
        &self,
        seed: &[u32],
        out: &mut [u32],
        mut rng: Option<&mut uor_r4_core::transformerless::runtime::SampleRng>,
        mut first_step: Option<&mut Option<NormativeServingDecision>>,
    ) -> Result<GenerateStatus, String> {
        let runtime = R4G1Runtime::parse(&self.graph).map_err(|error| {
            format!("normative R4G1 runtime rejected the loaded graph: {error:?}")
        })?;
        let mut engine = self.engine.borrow_mut();
        let mut node_scores = self.node_scores.borrow_mut();
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        let mut last_status = None;
        let mut widened = false;
        for generated in 0..out.len() {
            let decision = normative_step(
                &mut engine,
                &runtime,
                &mut node_scores,
                &window[..window_len],
            )?;
            if generated == 0 {
                if let Some(slot) = first_step.as_deref_mut() {
                    *slot = Some(decision);
                }
            }
            match decision {
                NormativeServingDecision::Serve(serve) => {
                    last_status = Some(serve.status);
                    widened = widened || serve.widened;
                    let next = match rng.as_deref_mut() {
                        Some(sample_rng) => {
                            serve.select_sampled_token(&out[..generated], sample_rng)
                        }
                        None => serve.token,
                    };
                    if next == 1 || next == 2 {
                        return Ok(GenerateStatus {
                            count: generated,
                            status: last_status,
                            widened,
                            abstained: false,
                        });
                    }
                    out[generated] = next;
                    if window_len < WINDOW {
                        window[window_len] = next;
                        window_len += 1;
                    } else {
                        window.copy_within(1.., 0);
                        window[WINDOW - 1] = next;
                    }
                }
                NormativeServingDecision::Abstain(outcome) => {
                    return Ok(GenerateStatus {
                        count: generated,
                        status: Some(outcome.status),
                        widened: widened || outcome.widened,
                        abstained: true,
                    });
                }
                NormativeServingDecision::Decline(_) => {
                    return Err(
                        "D4 permitted a position for which the normative R4G1 runtime produced no candidate"
                            .to_owned(),
                    );
                }
            }
        }
        Ok(GenerateStatus {
            count: out.len(),
            status: last_status,
            widened,
            abstained: false,
        })
    }

    /// Witness-enabled generation for the opt-in proof-carrying response
    /// envelope. The ordinary generation method remains allocation-free.
    pub fn generate_into_status_with_witness(
        &self,
        seed: &[u32],
        out: &mut [u32],
        witnesses: &mut Vec<InferenceWitness>,
    ) -> Result<GenerateStatus, String> {
        let runtime = R4G1Runtime::parse(&self.graph).map_err(|error| {
            format!("normative R4G1 runtime rejected the loaded graph: {error:?}")
        })?;
        let mut engine = self.engine.borrow_mut();
        // Preserve the old boundary behavior even for a zero-length output:
        // every supplied seed token must belong to the loaded artifact.
        engine
            .signature_for_window(seed)
            .map_err(|error| error.to_string())?;
        let mut node_scores = self.node_scores.borrow_mut();
        witnesses.clear();
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        let mut last_status = None;
        let mut widened = false;
        for (generated, token) in out.iter_mut().enumerate() {
            match normative_step(
                &mut engine,
                &runtime,
                &mut node_scores,
                &window[..window_len],
            )? {
                NormativeServingDecision::Serve(serve) => {
                    let outcome = PredictOutcome {
                        token: serve.token,
                        status: serve.status,
                        widened: serve.widened,
                        ngram_hit: serve.ngram_hit,
                    };
                    let candidates = serve.candidates;
                    last_status = Some(outcome.status);
                    widened = widened || outcome.widened;
                    if outcome.token == 1 || outcome.token == 2 {
                        return Ok(GenerateStatus {
                            count: generated,
                            status: last_status,
                            widened,
                            abstained: false,
                        });
                    }
                    let witness = normative_inference_witness(
                        &mut engine,
                        &runtime,
                        &window[..window_len],
                        outcome,
                        &candidates,
                    )?;
                    *token = outcome.token;
                    witnesses.push(witness);
                    if window_len < WINDOW {
                        window[window_len] = outcome.token;
                        window_len += 1;
                    } else {
                        window.copy_within(1.., 0);
                        window[WINDOW - 1] = outcome.token;
                    }
                }
                NormativeServingDecision::Abstain(outcome) => {
                    return Ok(GenerateStatus {
                        count: generated,
                        status: Some(outcome.status),
                        widened: widened || outcome.widened,
                        abstained: true,
                    });
                }
                NormativeServingDecision::Decline(_) => {
                    return Err(
                        "D4 permitted a position for which the normative R4G1 runtime produced no candidate"
                            .to_owned(),
                    );
                }
            }
        }
        Ok(GenerateStatus {
            count: out.len(),
            status: last_status,
            widened,
            abstained: false,
        })
    }

    /// Replay a compact response witness against the loaded artifact.
    pub fn verify_witnesses(
        &self,
        seed: &[u32],
        generated: &[u32],
        witnesses: &[InferenceWitness],
    ) -> Result<(), WitnessVerificationError> {
        if generated.len() != witnesses.len() {
            return Err(WitnessVerificationError::LengthMismatch);
        }
        let runtime = R4G1Runtime::parse(&self.graph)
            .map_err(|_| WitnessVerificationError::CandidateMismatch)?;
        let lane_present = runtime.skipmix_tables_present() != (false, false);
        let mut engine = self.engine.borrow_mut();
        engine
            .signature_for_window(seed)
            .map_err(|_| WitnessVerificationError::LengthMismatch)?;
        let mut node_scores = self.node_scores.borrow_mut();
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        for (&token, claimed) in generated.iter().zip(witnesses) {
            if claimed.engine != "r4g1" {
                return Err(WitnessVerificationError::EngineMismatch);
            }
            let permit = match engine
                .replay_admission(&window[..window_len])
                .map_err(|_| WitnessVerificationError::LengthMismatch)?
            {
                PolicyDecision::Permit(permit) => permit,
                PolicyDecision::Abstain(_) => return Err(WitnessVerificationError::StatusMismatch),
            };
            if claimed.resolution_status != PolicyStatus::from(permit.status).label() {
                return Err(WitnessVerificationError::StatusMismatch);
            }
            if claimed.widened != permit.widened {
                return Err(WitnessVerificationError::WidenedMismatch);
            }

            let signature = engine
                .signature_for_window(&window[..window_len])
                .map_err(|_| WitnessVerificationError::LengthMismatch)?;
            node_scores.fill(ScoreQ::MIN);
            let candidates = runtime.predict_served_candidates(
                &window[..window_len],
                Some(&signature),
                &mut node_scores,
            );
            let winner = candidates
                .winner()
                .ok_or(WitnessVerificationError::CandidateMismatch)?;
            if token != winner.token || claimed.token != token {
                return Err(WitnessVerificationError::TokenMismatch);
            }

            let expected_candidate =
                lane_present.then(|| ServedCandidateWitness::from_runtime(winner));
            if claimed.served_candidate != expected_candidate {
                return Err(WitnessVerificationError::CandidateMismatch);
            }
            if claimed.segment_lane.is_some() {
                return Err(WitnessVerificationError::SegmentLaneMismatch);
            }
            let expected_skipmix = lane_present
                .then(|| runtime_skipmix_witness(&candidates))
                .flatten();
            if claimed.skipmix_lane != expected_skipmix {
                return Err(WitnessVerificationError::SkipmixLaneMismatch);
            }

            if lane_present {
                if claimed.region_kappa.is_some() || claimed.region_id.is_some() {
                    return Err(WitnessVerificationError::RegionMismatch);
                }
                if claimed.depth != 0 {
                    return Err(WitnessVerificationError::DepthMismatch);
                }
            } else {
                let metadata = engine
                    .legacy_witness_metadata_for_runtime_token(
                        &window[..window_len],
                        permit.widened,
                        winner.token,
                    )
                    .map_err(|_| WitnessVerificationError::LengthMismatch)?
                    .ok_or(WitnessVerificationError::TokenMismatch)?;
                if claimed.region_kappa != metadata.region_kappa
                    || claimed.region_id != metadata.region_id
                {
                    return Err(WitnessVerificationError::RegionMismatch);
                }
                if claimed.depth != metadata.depth {
                    return Err(WitnessVerificationError::DepthMismatch);
                }
                if claimed.resolution_status != metadata.resolution_status {
                    return Err(WitnessVerificationError::StatusMismatch);
                }
            }

            if window_len < WINDOW {
                window[window_len] = token;
                window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[WINDOW - 1] = token;
            }
        }
        Ok(())
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
        let root = production_bundle_root(graph_path, teacher_path)?;
        let production_admission = capture_production_admission(root)?;
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
        let score_report = Some(production_admission.score_report.clone());
        Self::load_captured_with_source(
            graph_path,
            teacher_path,
            &CapturedR4g1Bundle {
                graph: graph_bytes,
                signature_artifact: teacher_bytes,
                tokenizer: tokenizer_bytes,
                score_report,
                production_admission: Some(production_admission),
            },
            source_dir,
        )
    }

    /// Explicit research-only path loader. It validates graph, artifact, and
    /// tokenizer structure but deliberately does not claim schema-2 release
    /// admission or deployed-quality PASS.
    pub fn load_for_research(graph_path: &Path, teacher_path: &Path) -> Result<Self, String> {
        Self::load_for_research_with_source(graph_path, teacher_path, None)
    }

    /// Explicit research-only source-aware loader. Production callers must
    /// use [`Self::load_with_source`].
    pub fn load_for_research_with_source(
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
        let score_report = graph_path
            .parent()
            .and_then(|parent| std::fs::read(parent.join("score_report.json")).ok())
            .filter(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).is_ok());
        Self::load_captured_for_research_with_source(
            graph_path,
            teacher_path,
            &CapturedR4g1Bundle {
                graph: graph_bytes,
                signature_artifact: teacher_bytes,
                tokenizer: tokenizer_bytes,
                score_report,
                production_admission: None,
            },
            source_dir,
        )
    }

    pub(crate) fn load_captured_with_source(
        graph_path: &Path,
        teacher_path: &Path,
        captured: &CapturedR4g1Bundle,
        source_dir: Option<&Path>,
    ) -> Result<Self, String> {
        Self::load_captured_inner(graph_path, teacher_path, captured, source_dir, true)
    }

    /// Explicit research-only captured-generation loader. The loud name is
    /// intentional: synthetic fixtures and pre-report build stages may use
    /// it, but no serving installation may silently downgrade to it.
    pub(crate) fn load_captured_for_research_with_source(
        graph_path: &Path,
        teacher_path: &Path,
        captured: &CapturedR4g1Bundle,
        source_dir: Option<&Path>,
    ) -> Result<Self, String> {
        Self::load_captured_inner(graph_path, teacher_path, captured, source_dir, false)
    }

    fn load_captured_inner(
        graph_path: &Path,
        teacher_path: &Path,
        captured: &CapturedR4g1Bundle,
        source_dir: Option<&Path>,
        require_production_admission: bool,
    ) -> Result<Self, String> {
        let tokenizer_path = teacher_path
            .parent()
            .map(|parent| parent.join("tokenizer.bin"));
        let tokenizer = captured
            .tokenizer
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
        // Production admission is contingent on the normative runtime being
        // able to parse every section it will consume. This catches malformed
        // SKMX/PSIB bytes here instead of allowing the policy-only reference
        // scorer to flatten them into an apparently usable bundle.
        let runtime = R4G1Runtime::parse(&captured.graph).map_err(|error| {
            format!(
                "{}: normative R4G1 runtime rejected the graph: {error:?}",
                graph_path.display()
            )
        })?;
        let node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
        drop(runtime);
        let engine_parts = EngineParts {
            graph: &captured.graph,
            signature_artifact: &captured.signature_artifact,
            // Supplying the exact bytes makes the graph header's independent
            // tokenizer.bin CID binding authoritative at this boundary.
            tokenizer: captured.tokenizer.as_deref(),
            score_report: captured.score_report.as_deref(),
        };
        if require_production_admission {
            let admission = captured.production_admission.as_ref().ok_or_else(|| {
                "production R4G1 load has no captured schema-2 release/deployed-quality admission envelope; use the explicitly named research loader only for non-serving work"
                    .to_owned()
            })?;
            let verified = verify_production_admission(
                &captured.graph,
                &captured.signature_artifact,
                captured.tokenizer.as_deref(),
                admission,
            )?;
            validate_production_serving_parts(&ProductionServingParts {
                engine: engine_parts,
                deployed_quality_report: &verified.deployed_quality_report,
                verified_envelope: &verified.envelope,
            })
            .map_err(|error| error.to_string())?;
        }
        // The content-bound deployed-quality report is the production gate;
        // the older Gate C row remains policy/configuration input only.
        let engine = R4Engine::load_accepting_quality(engine_parts).map_err(|error| {
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
            graph: captured.graph.clone(),
            node_scores: RefCell::new(node_scores),
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
