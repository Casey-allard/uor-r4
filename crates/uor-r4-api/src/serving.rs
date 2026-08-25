//! Normative R4G1 serving composition.
//!
//! ADR-0001 assigns two deliberately different responsibilities: [`R4Engine`]
//! resolves the D4 permit/abstain policy, while
//! [`uor_r4_graph_runtime::R4G1Runtime`] is the only candidate and token
//! selector. This module is the public library facade that composes them
//! without allowing the reference scorer's independently selected token to
//! cross the production boundary.

use serde::{Deserialize, Serialize};
use uor_r4_graph_format::{ObservedBound, ScoreQ};
use uor_r4_graph_runtime::{
    R4G1Runtime, ServedCandidateSource, ServedCandidates, SERVED_CANDIDATE_CAPACITY,
};
use uor_r4_model_source::SourceUnavailable;

use uor_r4_core::transformerless::compiler::WINDOW;
use uor_r4_core::transformerless::runtime::SampleRng;

use crate::engine::{
    AbstainOutcome, EngineParts, PolicyCounters, PolicyDecision, PolicyPermit, R4Engine,
};
use crate::production_envelope::VerifiedProductionEnvelope;
use crate::DeployedQualityReport;

/// Inputs required for production admission.
///
/// `verified_envelope` is an unforgeable result of checking the complete
/// schema-2 bundle. The exact engine/report bytes are re-bound to that
/// capability here, so a verified envelope from one generation cannot
/// authorize bytes from another.
#[derive(Debug, Clone, Copy)]
pub struct ProductionServingParts<'a> {
    pub engine: EngineParts<'a>,
    pub deployed_quality_report: &'a [u8],
    pub verified_envelope: &'a VerifiedProductionEnvelope,
}

/// Validate the content-bound production-admission envelope without taking
/// ownership of the serving engine. Host adapters that retain their own
/// runtime scratch use this seam before constructing it; the report bytes,
/// manifest-declared CID, and identities derived from the loaded generation
/// are still checked by the same fail-closed rule as [`NormativeServingEngine::load`].
pub fn validate_production_serving_parts(
    parts: &ProductionServingParts<'_>,
) -> Result<DeployedQualityReport, SourceUnavailable> {
    let manifest = parts.verified_envelope.manifest();
    require_verified_component(
        "graph/score.r4g1",
        parts.engine.graph,
        &manifest.components.graph,
    )?;
    require_verified_component(
        "tless_artifacts.bin",
        parts.engine.signature_artifact,
        &manifest.components.signature_artifact,
    )?;
    let tokenizer = parts.engine.tokenizer.ok_or_else(|| {
        SourceUnavailable::new(
            "production serving requires the tokenizer bytes verified by the envelope",
        )
    })?;
    let tokenizer_cid = manifest.components.tokenizer.as_deref().ok_or_else(|| {
        SourceUnavailable::new("verified production envelope has no tokenizer component")
    })?;
    require_verified_component("tokenizer.bin", tokenizer, tokenizer_cid)?;
    let score_report = parts.engine.score_report.ok_or_else(|| {
        SourceUnavailable::new(
            "production serving requires the score report verified by the envelope",
        )
    })?;
    require_verified_component(
        "graph/score_report.json",
        score_report,
        &manifest.components.score_report,
    )?;
    let report_cid = manifest
        .components
        .deployed_quality_report
        .as_deref()
        .ok_or_else(|| {
            SourceUnavailable::new(
                "verified production envelope has no deployed-quality report component",
            )
        })?;
    require_verified_component(
        "graph/deployed_quality_report.json",
        parts.deployed_quality_report,
        report_cid,
    )?;
    let report: DeployedQualityReport = serde_json::from_slice(parts.deployed_quality_report)
        .map_err(|error| {
            SourceUnavailable::new(format!("invalid deployed-quality report: {error}"))
        })?;
    if let Some(error) = report.validate_for_production(parts.verified_envelope.loaded_bindings()) {
        return Err(SourceUnavailable::new(error.to_string()));
    }
    Ok(report)
}

fn require_verified_component(
    label: &str,
    bytes: &[u8],
    expected_cid: &str,
) -> Result<(), SourceUnavailable> {
    let actual_cid = format!("blake3:{}", blake3::hash(bytes).to_hex());
    if actual_cid == expected_cid {
        Ok(())
    } else {
        Err(SourceUnavailable::new(format!(
            "production component {label} does not match the verified envelope: expected {expected_cid}, loaded bytes hash to {actual_cid}"
        )))
    }
}

/// Opaque, token-free D4 authority admitted by a verified production
/// envelope. It deliberately exposes neither [`R4Engine`] nor any of its
/// independently token-selecting reference methods.
pub struct ProductionPolicyEngine {
    policy: R4Engine,
}

impl ProductionPolicyEngine {
    fn load_for_research(parts: EngineParts<'_>) -> Result<Self, SourceUnavailable> {
        Ok(Self {
            policy: R4Engine::load_accepting_quality(parts)?,
        })
    }

    /// Reset only the bounded token-free D4 counters and widen-once memory.
    pub fn reset_policy_state(&mut self) {
        self.policy.reset();
    }

    /// Snapshot of the token-free D4 policy counters.
    pub fn policy_counters(&self) -> PolicyCounters {
        self.policy.policy_counters()
    }
}

/// Load the token-free D4 policy only after the content-bound production
/// report has admitted these exact bytes. Hosts which keep their own
/// [`R4G1Runtime`] scratch can use this seam without falling back to the
/// research-only quality bypass.
///
/// The return type is intentionally not the token-selecting reference engine:
///
/// ```compile_fail
/// use uor_r4_api::{
///     load_production_policy_engine, ProductionServingParts, R4Engine,
///     SourceUnavailable,
/// };
/// let _: for<'a> fn(ProductionServingParts<'a>)
///     -> Result<R4Engine, SourceUnavailable> = load_production_policy_engine;
/// ```
pub fn load_production_policy_engine(
    parts: ProductionServingParts<'_>,
) -> Result<ProductionPolicyEngine, SourceUnavailable> {
    validate_production_serving_parts(&parts)?;
    // The deployed-quality report is the current production gate. The older
    // Gate C row remains scorer/policy configuration and is not a second
    // admission authority.
    ProductionPolicyEngine::load_for_research(parts.engine)
}

/// A permitted normative selection, including the exact fixed-capacity
/// candidate list from which the token was selected.
#[derive(Debug, Clone, Copy)]
pub struct NormativeServe {
    pub token: u32,
    /// Winner before SKMX/PSIB contribution. Equal to `token` when the lane
    /// is absent or does not promote another candidate.
    pub base_token: u32,
    /// At least one ranked candidate came from the SKMX/PSIB contribution
    /// domain on this position.
    pub lane_reachable: bool,
    pub status: uor_r4_graph_certify::ScoreStatus,
    pub widened: bool,
    pub ngram_hit: bool,
    pub candidates: ServedCandidates,
}

impl NormativeServe {
    /// Select from the exact runtime-owned ranked candidates with the deployed
    /// deterministic sampler. Reference/base scorer candidates never enter
    /// this method, so sampled decode cannot bypass the normative shortlist.
    pub fn select_sampled_token(&self, emitted: &[u32], rng: &mut SampleRng) -> u32 {
        select_sampled_runtime_candidate(&self.candidates, emitted, self.token, rng)
    }
}

/// Shared deterministic sampler over an already runtime-owned shortlist.
/// Explicit research fallbacks that have no D4 policy object use this helper
/// without creating a synthetic permit or importing a reference candidate.
pub fn select_sampled_runtime_candidate(
    candidates: &ServedCandidates,
    emitted: &[u32],
    fallback: u32,
    rng: &mut SampleRng,
) -> u32 {
    let ranked = candidates.ranked();
    if ranked.is_empty() {
        return fallback;
    }
    let min_raw = ranked
        .iter()
        .map(|candidate| i64::from(candidate.score.raw()))
        .min()
        .unwrap_or(0);
    let mut weights = [0u32; SERVED_CANDIDATE_CAPACITY];
    let mut total = 0u32;
    for (index, candidate) in ranked.iter().enumerate() {
        let occurrences = emitted
            .iter()
            .filter(|&&token| token == candidate.token)
            .count() as i64;
        let mut weight = i64::from(candidate.score.raw()) - min_raw + 1;
        weight -= (occurrences << 10) - (occurrences << 4) - (occurrences << 3);
        weights[index] = weight.clamp(1, i64::from(u32::MAX)) as u32;
        total = total.saturating_add(weights[index]);
    }
    if total == 0 {
        return fallback;
    }
    let draw = rng.draw(total);
    let mut accumulated = 0u32;
    for (index, candidate) in ranked.iter().enumerate() {
        accumulated = accumulated.saturating_add(weights[index]);
        if draw < accumulated {
            return candidate.token;
        }
    }
    fallback
}

/// Fail-closed reason after policy resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormativeDecline {
    /// D4 permitted serving, but the normative runtime produced no candidate.
    NoRuntimeCandidate,
}

/// Total production decision: serve the normative runtime token, preserve a
/// D4 abstention, or decline because the two authorities cannot be composed.
#[derive(Debug, Clone, Copy)]
pub enum NormativeServingDecision {
    Serve(NormativeServe),
    Abstain(AbstainOutcome),
    Decline(NormativeDecline),
}

/// The sole token-authoritative step adapter. It derives the D4 signature and
/// permit/abstain decision from the newest normative window, then asks
/// `R4G1Runtime` for the ranked candidates over the complete supplied context.
/// Optional session-signature lanes are threaded into that same candidate
/// query. D4 never supplies or substitutes a token.
pub struct NormativeStepAdapter<'state, 'graph> {
    policy: &'state mut R4Engine,
    runtime: &'state R4G1Runtime<'graph>,
    node_scores: &'state mut [ScoreQ],
}

impl<'state, 'graph> NormativeStepAdapter<'state, 'graph> {
    /// Compose a verified, token-free production policy with the sole
    /// token-authoritative runtime.
    pub fn new(
        policy: &'state mut ProductionPolicyEngine,
        runtime: &'state R4G1Runtime<'graph>,
        node_scores: &'state mut [ScoreQ],
    ) -> Self {
        Self {
            policy: &mut policy.policy,
            runtime,
            node_scores,
        }
    }

    /// Lower-level composition with the reference-policy object. Possession
    /// of this object is not production admission; named production loaders
    /// return only [`ProductionPolicyEngine`].
    pub fn new_with_reference_policy(
        policy: &'state mut R4Engine,
        runtime: &'state R4G1Runtime<'graph>,
        node_scores: &'state mut [ScoreQ],
    ) -> Self {
        Self {
            policy,
            runtime,
            node_scores,
        }
    }

    pub fn select(
        &mut self,
        context_tokens: &[u32],
        session_signature: Option<&[u8]>,
    ) -> Result<NormativeServingDecision, ObservedBound> {
        self.select_with_admission(context_tokens, session_signature, false)
    }

    /// Replay the exact composed production step without retaining D4
    /// counters or widen-once memory.
    ///
    /// This is the bounded speculative-query seam for beam search: every
    /// hypothesis is still classified by D4 and every candidate still comes
    /// from `R4G1Runtime`, while only the ultimately served path is allowed to
    /// mutate policy state through [`Self::select`]. Runtime scratch remains
    /// caller-owned and is overwritten on the next query.
    pub fn replay_select(
        &mut self,
        context_tokens: &[u32],
        session_signature: Option<&[u8]>,
    ) -> Result<NormativeServingDecision, ObservedBound> {
        self.select_with_admission(context_tokens, session_signature, true)
    }

    fn select_with_admission(
        &mut self,
        context_tokens: &[u32],
        session_signature: Option<&[u8]>,
        replay: bool,
    ) -> Result<NormativeServingDecision, ObservedBound> {
        let policy_window = &context_tokens[context_tokens.len().saturating_sub(WINDOW)..];
        let signature = self.policy.signature_for_window(policy_window)?;
        let policy_decision = if replay {
            self.policy.replay_admission(policy_window)?
        } else {
            self.policy.admit_window(policy_window)?
        };
        match policy_decision {
            PolicyDecision::Abstain(outcome) => Ok(NormativeServingDecision::Abstain(outcome)),
            PolicyDecision::Permit(PolicyPermit {
                status,
                widened,
                ngram_hit,
            }) => {
                self.node_scores.fill(ScoreQ::MIN);
                let candidates = self.runtime.predict_served_candidates_with_signature_lanes(
                    context_tokens,
                    Some(&signature),
                    session_signature,
                    self.node_scores,
                );
                let Some(winner) = candidates.winner() else {
                    return Ok(NormativeServingDecision::Decline(
                        NormativeDecline::NoRuntimeCandidate,
                    ));
                };
                let base_token = candidates
                    .attribution()
                    .map_or(winner.token, |attribution| attribution.base_token);
                let lane_reachable = candidates
                    .ranked()
                    .iter()
                    .any(|candidate| candidate.source == ServedCandidateSource::Skipmix);
                Ok(NormativeServingDecision::Serve(NormativeServe {
                    token: winner.token,
                    base_token,
                    lane_reachable,
                    status,
                    widened,
                    ngram_hit,
                    candidates,
                }))
            }
        }
    }
}

/// Deterministic, replayable cross-surface selector evidence schema.
pub const CROSS_SURFACE_PARITY_EVIDENCE_SCHEMA: &str = "uor-r4-normative-selector-cross-surface/4";

/// Stable schema-2 bundle path for cross-surface evidence.
pub const CROSS_SURFACE_PARITY_BUNDLE_PATH: &str = "graph/cross_surface_parity.json";

/// Cross-surface rows retain only the complete normative compiler window.
/// A larger opaque prompt would make evidence bytes unbounded without
/// changing the step selected by any admitted surface.
pub const CROSS_SURFACE_CONTEXT_CAPACITY: usize = WINDOW;

/// Session signatures are currently 36 bytes, but the graph format owns the
/// lane and deliberately accepts borrowed byte slices. This conservative
/// ceiling keeps the evidence format bounded while leaving room for a
/// versioned signature expansion.
pub const CROSS_SURFACE_SESSION_SIGNATURE_CAPACITY: usize = 1_024;

/// A surface's externally observed step disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrossSurfaceDisposition {
    Serve,
    Abstain,
    Decline,
}

/// Decode rule independently replayed from a runtime-owned shortlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrossSurfaceDecodeMode {
    /// The normative runtime winner is emitted directly.
    Greedy,
    /// One weighted draw from the normative runtime shortlist.
    Sampled,
    /// The first beam step is the same runtime winner as greedy decode.
    BeamFirstStep,
}

/// One canonical comparison against the shared adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossSurfaceParityRecord {
    pub surface: String,
    pub decode_policy: String,
    pub decode_mode: CrossSurfaceDecodeMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode_seed: Option<u32>,
    /// Exact bounded tokens supplied to the shared normative step adapter.
    pub context_tokens: Vec<u32>,
    pub context_cid: String,
    /// Exact optional session-lane input. Its CID remains beside it so a
    /// reader can reject a row before replay if either representation was
    /// altered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_signature: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_signature_cid: Option<String>,
    pub authoritative_disposition: CrossSurfaceDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authoritative_token: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_status: Option<String>,
    pub widened: bool,
    pub ngram_hit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authoritative_ranked_candidates_cid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_token: Option<u32>,
    pub lane_reachable: bool,
    pub observed_disposition: CrossSurfaceDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_token: Option<u32>,
    /// CID derived from the actual bounded shortlist captured by this
    /// surface. It is never copied from the authoritative replay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_ranked_candidates_cid: Option<String>,
    pub matched: bool,
}

/// Raw, self-counting parity evidence. Wall-clock/resource telemetry is kept
/// out of these deterministic bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossSurfaceParityEvidence {
    pub schema: String,
    pub graph_cid: String,
    pub signature_artifact_cid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer_cid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_report_cid: Option<String>,
    pub checks: u64,
    pub mismatches: u64,
    pub records: Vec<CrossSurfaceParityRecord>,
}

impl CrossSurfaceParityEvidence {
    pub fn deterministic_json_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            SourceUnavailable::new(format!(
                "serialize cross-surface parity evidence as deterministic JSON: {error}"
            ))
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Parse canonical evidence bytes and bind them to an artifact-only
    /// generation (graph plus signature artifact, no tokenizer or score
    /// report). Counts and row verdicts are recomputed; a self-consistent
    /// artifact for any other generation is rejected.
    pub fn parse_and_validate_for_artifacts(
        bytes: &[u8],
        graph: &[u8],
        signature_artifact: &[u8],
    ) -> Result<Self, SourceUnavailable> {
        Self::parse_and_validate_for_bundle(bytes, graph, signature_artifact, None, None)
    }

    /// Parse canonical evidence and replay every authoritative row against
    /// the complete immutable D4 configuration. Production callers pass the
    /// exact tokenizer and `score_report.json`; omitting either from a bound
    /// generation fails closed.
    pub fn parse_and_validate_for_bundle(
        bytes: &[u8],
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: Option<&[u8]>,
        score_report: Option<&[u8]>,
    ) -> Result<Self, SourceUnavailable> {
        let evidence: Self = serde_json::from_slice(bytes).map_err(|error| {
            SourceUnavailable::new(format!("invalid cross-surface parity evidence: {error}"))
        })?;
        evidence.validate_for_bundle(graph, signature_artifact, tokenizer, score_report)?;
        let canonical = evidence.deterministic_json_bytes()?;
        if canonical != bytes {
            return Err(SourceUnavailable::new(
                "cross-surface parity evidence bytes are not canonical",
            ));
        }
        Ok(evidence)
    }

    /// Parse production evidence and require the complete canonical adapter
    /// inventory in addition to byte binding and authoritative replay.
    ///
    /// The generic parser above remains useful for planted unit observations
    /// and research diagnostics. Production admission must not accept one
    /// conveniently matching row as proof that greedy, sampled, beam, API,
    /// native-host, and CLI-chat adapters share the normative selector.
    pub fn parse_and_validate_for_production_bundle(
        bytes: &[u8],
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: &[u8],
        score_report: &[u8],
    ) -> Result<Self, SourceUnavailable> {
        let evidence = Self::parse_and_validate_for_bundle(
            bytes,
            graph,
            signature_artifact,
            Some(tokenizer),
            Some(score_report),
        )?;
        evidence.validate_canonical_production_inventory()?;
        Ok(evidence)
    }

    /// Require the eight non-vacuous rows emitted by the canonical #933
    /// producer. HTTP and WASM wrapper reachability is tested separately; the
    /// evidence names only adapters which it mechanically executes.
    pub fn validate_canonical_production_inventory(&self) -> Result<(), SourceUnavailable> {
        const REQUIRED: [(&str, CrossSurfaceDecodeMode); 8] = [
            ("direct-api", CrossSurfaceDecodeMode::Greedy),
            ("direct-api", CrossSurfaceDecodeMode::Sampled),
            (
                "r4g1-state-native-host-adapter",
                CrossSurfaceDecodeMode::Greedy,
            ),
            (
                "r4g1-state-native-host-adapter",
                CrossSurfaceDecodeMode::Sampled,
            ),
            (
                "direct-api-session-bound",
                CrossSurfaceDecodeMode::BeamFirstStep,
            ),
            ("direct-api-session-bound", CrossSurfaceDecodeMode::Sampled),
            (
                "cli-chat-shared-production-step",
                CrossSurfaceDecodeMode::BeamFirstStep,
            ),
            (
                "cli-chat-shared-production-step",
                CrossSurfaceDecodeMode::Sampled,
            ),
        ];
        if self.tokenizer_cid.is_none() || self.score_report_cid.is_none() {
            return Err(SourceUnavailable::new(
                "production cross-surface evidence must bind tokenizer and score-report bytes",
            ));
        }
        if self.records.len() != REQUIRED.len()
            || self.checks != REQUIRED.len() as u64
            || self.mismatches != 0
        {
            return Err(SourceUnavailable::new(format!(
                "production cross-surface inventory requires exactly {} checks and zero mismatches; evidence has {} records, {} checks, {} mismatches",
                REQUIRED.len(),
                self.records.len(),
                self.checks,
                self.mismatches
            )));
        }
        let context_cid = self
            .records
            .first()
            .map(|record| record.context_cid.as_str())
            .ok_or_else(|| SourceUnavailable::new("production cross-surface inventory is empty"))?;
        let mut sampled_seed = None;
        for record in &self.records {
            if record.context_cid != context_cid
                || record.authoritative_disposition != CrossSurfaceDisposition::Serve
                || record.observed_disposition != CrossSurfaceDisposition::Serve
                || !record.matched
                || !record.lane_reachable
                || record.authoritative_ranked_candidates_cid.is_none()
                || record.authoritative_ranked_candidates_cid
                    != record.observed_ranked_candidates_cid
            {
                return Err(SourceUnavailable::new(
                    "production cross-surface rows must share one context and be matched, served, lane-reachable observations",
                ));
            }
            if record.decode_mode == CrossSurfaceDecodeMode::Sampled {
                let seed = record.decode_seed.ok_or_else(|| {
                    SourceUnavailable::new("production sampled parity row has no seed")
                })?;
                match sampled_seed {
                    Some(expected) if expected != seed => {
                        return Err(SourceUnavailable::new(
                            "production sampled parity rows do not share one pinned seed",
                        ));
                    }
                    None => sampled_seed = Some(seed),
                    Some(_) => {}
                }
            }
            let session_bound = matches!(
                record.surface.as_str(),
                "direct-api-session-bound" | "cli-chat-shared-production-step"
            );
            if session_bound != record.session_signature.is_some() {
                return Err(SourceUnavailable::new(
                    "production cross-surface row has the wrong session-input shape for its canonical cohort",
                ));
            }
        }
        for (surface, decode_mode) in REQUIRED {
            let count = self
                .records
                .iter()
                .filter(|record| record.surface == surface && record.decode_mode == decode_mode)
                .count();
            if count != 1 {
                return Err(SourceUnavailable::new(format!(
                    "production cross-surface inventory requires exactly one {surface}/{decode_mode:?} row; found {count}"
                )));
            }
        }
        const COHORTS: [(&str, &str, CrossSurfaceDecodeMode); 4] = [
            (
                "direct-api",
                "r4g1-state-native-host-adapter",
                CrossSurfaceDecodeMode::Greedy,
            ),
            (
                "direct-api",
                "r4g1-state-native-host-adapter",
                CrossSurfaceDecodeMode::Sampled,
            ),
            (
                "direct-api-session-bound",
                "cli-chat-shared-production-step",
                CrossSurfaceDecodeMode::BeamFirstStep,
            ),
            (
                "direct-api-session-bound",
                "cli-chat-shared-production-step",
                CrossSurfaceDecodeMode::Sampled,
            ),
        ];
        for (left_surface, right_surface, decode_mode) in COHORTS {
            let row = |surface: &str| {
                self.records
                    .iter()
                    .find(|record| record.surface == surface && record.decode_mode == decode_mode)
            };
            let left = row(left_surface).ok_or_else(|| {
                SourceUnavailable::new("cross-surface same-input cohort is incomplete")
            })?;
            let right = row(right_surface).ok_or_else(|| {
                SourceUnavailable::new("cross-surface same-input cohort is incomplete")
            })?;
            if left.context_tokens != right.context_tokens
                || left.context_cid != right.context_cid
                || left.session_signature != right.session_signature
                || left.session_signature_cid != right.session_signature_cid
                || left.decode_seed != right.decode_seed
                || left.authoritative_token != right.authoritative_token
                || left.observed_token != right.observed_token
                || left.authoritative_ranked_candidates_cid
                    != right.authoritative_ranked_candidates_cid
                || left.observed_ranked_candidates_cid != right.observed_ranked_candidates_cid
            {
                return Err(SourceUnavailable::new(format!(
                    "cross-surface {decode_mode:?} cohort {left_surface}/{right_surface} does not use identical selector inputs and outputs"
                )));
            }
        }
        Ok(())
    }

    /// Validate an already parsed artifact against the exact serving bytes.
    /// Call [`Self::parse_and_validate_for_artifacts`] when the raw artifact
    /// bytes are available so canonical serialization is checked as well.
    pub fn validate_for_artifacts(
        &self,
        graph: &[u8],
        signature_artifact: &[u8],
    ) -> Result<(), SourceUnavailable> {
        self.validate_for_bundle(graph, signature_artifact, None, None)
    }

    /// Validate identities, independently recompute every authoritative
    /// decision and seeded draw, and then reproduce row/aggregate verdicts.
    pub fn validate_for_bundle(
        &self,
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: Option<&[u8]>,
        score_report: Option<&[u8]>,
    ) -> Result<(), SourceUnavailable> {
        if self.schema != CROSS_SURFACE_PARITY_EVIDENCE_SCHEMA {
            return Err(SourceUnavailable::new(format!(
                "cross-surface parity schema {} is unsupported",
                self.schema
            )));
        }
        let graph_cid = parity_bytes_cid(graph);
        if self.graph_cid != graph_cid {
            return Err(SourceUnavailable::new(format!(
                "cross-surface graph CID mismatch: evidence has {}, loaded bytes hash to {graph_cid}",
                self.graph_cid
            )));
        }
        let signature_artifact_cid = parity_bytes_cid(signature_artifact);
        if self.signature_artifact_cid != signature_artifact_cid {
            return Err(SourceUnavailable::new(format!(
                "cross-surface signature artifact CID mismatch: evidence has {}, loaded bytes hash to {signature_artifact_cid}",
                self.signature_artifact_cid
            )));
        }
        let tokenizer_cid = tokenizer.map(parity_bytes_cid);
        if self.tokenizer_cid != tokenizer_cid {
            return Err(SourceUnavailable::new(format!(
                "cross-surface tokenizer CID mismatch: evidence has {:?}, loaded bytes hash to {tokenizer_cid:?}",
                self.tokenizer_cid
            )));
        }
        let score_report_cid = score_report.map(parity_bytes_cid);
        if self.score_report_cid != score_report_cid {
            return Err(SourceUnavailable::new(format!(
                "cross-surface score-report CID mismatch: evidence has {:?}, loaded bytes hash to {score_report_cid:?}",
                self.score_report_cid
            )));
        }
        if self.records.is_empty() {
            return Err(SourceUnavailable::new(
                "cross-surface parity evidence has no checks",
            ));
        }
        if !is_canonical_parity_cid(&self.graph_cid)
            || !is_canonical_parity_cid(&self.signature_artifact_cid)
            || self
                .tokenizer_cid
                .as_deref()
                .is_some_and(|cid| !is_canonical_parity_cid(cid))
            || self
                .score_report_cid
                .as_deref()
                .is_some_and(|cid| !is_canonical_parity_cid(cid))
        {
            return Err(SourceUnavailable::new(
                "cross-surface artifact identities are not canonical BLAKE3 CIDs",
            ));
        }
        if self
            .records
            .windows(2)
            .any(|pair| parity_record_key(&pair[0]) >= parity_record_key(&pair[1]))
        {
            return Err(SourceUnavailable::new(
                "cross-surface records are not in strict canonical order or contain a duplicate key",
            ));
        }

        let mut replayer = NormativeServingEngine::load_for_research(EngineParts {
            graph,
            signature_artifact,
            tokenizer,
            score_report,
        })?;
        for (index, record) in self.records.iter().enumerate() {
            validate_parity_record(record, index)?;
            replayer.reset_policy_state();
            let decision = replayer
                .predict_with_session_signature(
                    &record.context_tokens,
                    record.session_signature.as_deref(),
                )
                .map_err(|error| {
                    SourceUnavailable::new(format!(
                        "cross-surface authoritative replay at record {index}: {error}"
                    ))
                })?;
            let replayed =
                parity_authoritative_claim(decision, record.decode_mode, record.decode_seed)?;
            if !replayed.equals_record(record) {
                return Err(SourceUnavailable::new(format!(
                    "cross-surface authoritative claim at record {index} does not replay from the loaded bundle"
                )));
            }
            let matched = replayed.disposition == record.observed_disposition
                && replayed.token == record.observed_token
                && replayed.ranked_candidates_cid == record.observed_ranked_candidates_cid;
            if record.matched != matched {
                return Err(SourceUnavailable::new(format!(
                    "cross-surface replayed match verdict at record {index} does not reproduce"
                )));
            }
        }
        let checks = u64::try_from(self.records.len())
            .map_err(|_| SourceUnavailable::new("cross-surface record count exceeds u64"))?;
        let mismatches =
            u64::try_from(self.records.iter().filter(|record| !record.matched).count())
                .map_err(|_| SourceUnavailable::new("cross-surface mismatch count exceeds u64"))?;
        if self.checks != checks || self.mismatches != mismatches {
            return Err(SourceUnavailable::new(format!(
                "cross-surface counts do not reproduce: evidence has {}/{} checks/mismatches, rows reproduce {checks}/{mismatches}",
                self.checks, self.mismatches
            )));
        }
        Ok(())
    }
}

fn parity_record_key(record: &CrossSurfaceParityRecord) -> (&str, &str, &str, Option<&str>) {
    (
        record.surface.as_str(),
        record.decode_policy.as_str(),
        record.context_cid.as_str(),
        record.session_signature_cid.as_deref(),
    )
}

fn validate_parity_record(
    record: &CrossSurfaceParityRecord,
    index: usize,
) -> Result<(), SourceUnavailable> {
    if record.surface.is_empty()
        || record.surface.trim() != record.surface
        || record.decode_policy.is_empty()
        || record.decode_policy.trim() != record.decode_policy
    {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} has a noncanonical surface or decode policy"
        )));
    }
    if record.context_tokens.is_empty()
        || record.context_tokens.len() > CROSS_SURFACE_CONTEXT_CAPACITY
        || record.context_cid != parity_tokens_cid(&record.context_tokens)
    {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} has an invalid, oversized, or CID-mismatched context"
        )));
    }
    if record
        .session_signature
        .as_deref()
        .is_some_and(|signature| {
            signature.is_empty() || signature.len() > CROSS_SURFACE_SESSION_SIGNATURE_CAPACITY
        })
        || record.session_signature_cid != record.session_signature.as_deref().map(parity_bytes_cid)
    {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} has an invalid, oversized, or CID-mismatched session signature"
        )));
    }
    let (decode_mode, decode_seed) = parity_decode_contract(&record.decode_policy)?;
    if record.decode_mode != decode_mode || record.decode_seed != decode_seed {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} decode policy, mode, and seed disagree"
        )));
    }
    if !is_canonical_parity_cid(&record.context_cid)
        || record
            .session_signature_cid
            .as_deref()
            .is_some_and(|cid| !is_canonical_parity_cid(cid))
        || record
            .authoritative_ranked_candidates_cid
            .as_deref()
            .is_some_and(|cid| !is_canonical_parity_cid(cid))
        || record
            .observed_ranked_candidates_cid
            .as_deref()
            .is_some_and(|cid| !is_canonical_parity_cid(cid))
    {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} contains a noncanonical BLAKE3 CID"
        )));
    }
    let policy_status_is_valid = record.policy_status.as_deref().is_some_and(|status| {
        matches!(
            status,
            "exact_context" | "graph" | "novel" | "contradictory"
        )
    });
    let authoritative_shape_ok = match record.authoritative_disposition {
        CrossSurfaceDisposition::Serve => {
            record.authoritative_token.is_some()
                && policy_status_is_valid
                && record.authoritative_ranked_candidates_cid.is_some()
                && record.base_token.is_some()
        }
        CrossSurfaceDisposition::Abstain => {
            record.authoritative_token.is_none()
                && policy_status_is_valid
                && record.authoritative_ranked_candidates_cid.is_none()
                && record.base_token.is_none()
                && !record.lane_reachable
        }
        CrossSurfaceDisposition::Decline => {
            record.authoritative_token.is_none()
                && record.policy_status.is_none()
                && record.authoritative_ranked_candidates_cid.is_none()
                && record.base_token.is_none()
                && !record.widened
                && !record.ngram_hit
                && !record.lane_reachable
        }
    };
    if !authoritative_shape_ok {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} authoritative fields do not match its disposition"
        )));
    }
    let observed_shape_ok = matches!(
        (
            record.observed_disposition,
            record.observed_token,
            record.observed_ranked_candidates_cid.as_ref()
        ),
        (CrossSurfaceDisposition::Serve, Some(_), Some(_))
            | (
                CrossSurfaceDisposition::Abstain | CrossSurfaceDisposition::Decline,
                None,
                None
            )
    );
    if !observed_shape_ok {
        return Err(SourceUnavailable::new(format!(
            "cross-surface record {index} observed token does not match its disposition"
        )));
    }
    Ok(())
}

/// Decode-policy labels are part of the stable human-readable inventory, but
/// the typed mode and seed are what replay executes. Keeping one strict
/// grammar prevents a label from claiming a different policy than the bytes
/// actually reproduced.
fn parity_decode_contract(
    decode_policy: &str,
) -> Result<(CrossSurfaceDecodeMode, Option<u32>), SourceUnavailable> {
    match decode_policy {
        "greedy" => Ok((CrossSurfaceDecodeMode::Greedy, None)),
        "beam-first-step" => Ok((CrossSurfaceDecodeMode::BeamFirstStep, None)),
        _ => {
            let Some(seed) = decode_policy.strip_prefix("default-sampled-seed-") else {
                return Err(SourceUnavailable::new(format!(
                    "unsupported cross-surface decode policy {decode_policy:?}"
                )));
            };
            if seed.is_empty() || (seed.len() > 1 && seed.starts_with('0')) {
                return Err(SourceUnavailable::new(format!(
                    "noncanonical cross-surface sampled seed {seed:?}"
                )));
            }
            let seed = seed.parse::<u32>().map_err(|_| {
                SourceUnavailable::new(format!("invalid cross-surface sampled seed {seed:?}"))
            })?;
            Ok((CrossSurfaceDecodeMode::Sampled, Some(seed)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParityAuthoritativeClaim {
    disposition: CrossSurfaceDisposition,
    token: Option<u32>,
    policy_status: Option<String>,
    widened: bool,
    ngram_hit: bool,
    ranked_candidates_cid: Option<String>,
    base_token: Option<u32>,
    lane_reachable: bool,
}

impl ParityAuthoritativeClaim {
    fn equals_record(&self, record: &CrossSurfaceParityRecord) -> bool {
        self.disposition == record.authoritative_disposition
            && self.token == record.authoritative_token
            && self.policy_status == record.policy_status
            && self.widened == record.widened
            && self.ngram_hit == record.ngram_hit
            && self.ranked_candidates_cid == record.authoritative_ranked_candidates_cid
            && self.base_token == record.base_token
            && self.lane_reachable == record.lane_reachable
    }
}

fn parity_authoritative_claim(
    decision: NormativeServingDecision,
    decode_mode: CrossSurfaceDecodeMode,
    decode_seed: Option<u32>,
) -> Result<ParityAuthoritativeClaim, SourceUnavailable> {
    let expected_seed_shape = matches!(decode_mode, CrossSurfaceDecodeMode::Sampled);
    if expected_seed_shape != decode_seed.is_some() {
        return Err(SourceUnavailable::new(
            "cross-surface decode mode and seed shape disagree",
        ));
    }
    match decision {
        NormativeServingDecision::Serve(serve) => {
            let token = match decode_mode {
                CrossSurfaceDecodeMode::Greedy | CrossSurfaceDecodeMode::BeamFirstStep => {
                    serve.token
                }
                CrossSurfaceDecodeMode::Sampled => {
                    let mut rng = SampleRng::new(decode_seed.ok_or_else(|| {
                        SourceUnavailable::new("sampled cross-surface replay omitted its seed")
                    })?);
                    serve.select_sampled_token(&[], &mut rng)
                }
            };
            Ok(ParityAuthoritativeClaim {
                disposition: CrossSurfaceDisposition::Serve,
                token: Some(token),
                policy_status: Some(
                    crate::engine::PolicyStatus::from(serve.status)
                        .label()
                        .to_string(),
                ),
                widened: serve.widened,
                ngram_hit: serve.ngram_hit,
                ranked_candidates_cid: Some(parity_candidates_cid(&serve.candidates)),
                base_token: Some(serve.base_token),
                lane_reachable: serve.lane_reachable,
            })
        }
        NormativeServingDecision::Abstain(outcome) => Ok(ParityAuthoritativeClaim {
            disposition: CrossSurfaceDisposition::Abstain,
            token: None,
            policy_status: Some(
                crate::engine::PolicyStatus::from(outcome.status)
                    .label()
                    .to_string(),
            ),
            widened: outcome.widened,
            ngram_hit: outcome.ngram_hit,
            ranked_candidates_cid: None,
            base_token: None,
            lane_reachable: false,
        }),
        NormativeServingDecision::Decline(_) => Ok(ParityAuthoritativeClaim {
            disposition: CrossSurfaceDisposition::Decline,
            token: None,
            policy_status: None,
            widened: false,
            ngram_hit: false,
            ranked_candidates_cid: None,
            base_token: None,
            lane_reachable: false,
        }),
    }
}

fn is_canonical_parity_cid(cid: &str) -> bool {
    let Some(hex) = cid.strip_prefix("blake3:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

/// Borrowed inputs for one surface comparison. The authoritative token is
/// policy-specific: greedy/beam use `serve.token`; sampled decode supplies the
/// deterministic token selected from `serve.candidates` with its pinned seed.
pub struct CrossSurfaceParityObservation<'a> {
    pub surface: &'a str,
    pub decode_policy: &'a str,
    pub context_tokens: &'a [u32],
    pub session_signature: Option<&'a [u8]>,
    pub authoritative: NormativeServingDecision,
    pub authoritative_token: Option<u32>,
    pub observed_disposition: CrossSurfaceDisposition,
    pub observed_token: Option<u32>,
    /// Exact shortlist returned by the mechanically executed surface.
    /// Served observations require one; abstain/decline observations forbid
    /// one. The builder, not the caller, derives its CID.
    pub observed_candidates: Option<ServedCandidates>,
}

/// Builder which derives byte and context identities itself, canonicalizes row
/// order, rejects duplicate surface/policy/context keys, and computes exact
/// check/mismatch counts from the observations.
pub struct CrossSurfaceParityEvidenceBuilder {
    graph_cid: String,
    signature_artifact_cid: String,
    tokenizer_cid: Option<String>,
    score_report_cid: Option<String>,
    records: Vec<CrossSurfaceParityRecord>,
}

impl CrossSurfaceParityEvidenceBuilder {
    pub fn new(graph: &[u8], signature_artifact: &[u8]) -> Self {
        Self::new_for_bundle(graph, signature_artifact, None, None)
    }

    /// Construct evidence for a complete D4 bundle configuration.
    pub fn new_for_bundle(
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: Option<&[u8]>,
        score_report: Option<&[u8]>,
    ) -> Self {
        Self {
            graph_cid: parity_bytes_cid(graph),
            signature_artifact_cid: parity_bytes_cid(signature_artifact),
            tokenizer_cid: tokenizer.map(parity_bytes_cid),
            score_report_cid: score_report.map(parity_bytes_cid),
            records: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        observation: CrossSurfaceParityObservation<'_>,
    ) -> Result<(), SourceUnavailable> {
        if observation.surface.is_empty()
            || observation.surface.trim() != observation.surface
            || observation.decode_policy.is_empty()
            || observation.decode_policy.trim() != observation.decode_policy
        {
            return Err(SourceUnavailable::new(
                "cross-surface observation requires canonical nonempty surface and decode policy",
            ));
        }
        let observed_token_shape_ok = matches!(
            (
                observation.observed_disposition,
                observation.observed_token,
                observation.observed_candidates
            ),
            (CrossSurfaceDisposition::Serve, Some(_), Some(candidates)) if !candidates.is_empty()
        ) || matches!(
            (
                observation.observed_disposition,
                observation.observed_token,
                observation.observed_candidates
            ),
            (
                CrossSurfaceDisposition::Abstain | CrossSurfaceDisposition::Decline,
                None,
                None
            )
        );
        if !observed_token_shape_ok {
            return Err(SourceUnavailable::new(
                "cross-surface observed token does not match its disposition",
            ));
        }
        if observation.context_tokens.is_empty()
            || observation.context_tokens.len() > CROSS_SURFACE_CONTEXT_CAPACITY
        {
            return Err(SourceUnavailable::new(format!(
                "cross-surface context must contain 1..={CROSS_SURFACE_CONTEXT_CAPACITY} tokens"
            )));
        }
        if observation.session_signature.is_some_and(|signature| {
            signature.is_empty() || signature.len() > CROSS_SURFACE_SESSION_SIGNATURE_CAPACITY
        }) {
            return Err(SourceUnavailable::new(format!(
                "cross-surface session signature must contain 1..={CROSS_SURFACE_SESSION_SIGNATURE_CAPACITY} bytes"
            )));
        }
        let (decode_mode, decode_seed) = parity_decode_contract(observation.decode_policy)?;
        let authoritative =
            parity_authoritative_claim(observation.authoritative, decode_mode, decode_seed)?;
        if observation.authoritative_token != authoritative.token {
            return Err(SourceUnavailable::new(
                "caller-supplied authoritative token does not equal the pinned decode replay",
            ));
        }
        let observed_ranked_candidates_cid = observation
            .observed_candidates
            .as_ref()
            .map(parity_candidates_cid);
        let matched = authoritative.disposition == observation.observed_disposition
            && authoritative.token == observation.observed_token
            && authoritative.ranked_candidates_cid == observed_ranked_candidates_cid;
        self.records.push(CrossSurfaceParityRecord {
            surface: observation.surface.to_string(),
            decode_policy: observation.decode_policy.to_string(),
            decode_mode,
            decode_seed,
            context_tokens: observation.context_tokens.to_vec(),
            context_cid: parity_tokens_cid(observation.context_tokens),
            session_signature: observation.session_signature.map(ToOwned::to_owned),
            session_signature_cid: observation.session_signature.map(parity_bytes_cid),
            authoritative_disposition: authoritative.disposition,
            authoritative_token: authoritative.token,
            policy_status: authoritative.policy_status,
            widened: authoritative.widened,
            ngram_hit: authoritative.ngram_hit,
            authoritative_ranked_candidates_cid: authoritative.ranked_candidates_cid,
            base_token: authoritative.base_token,
            lane_reachable: authoritative.lane_reachable,
            observed_disposition: observation.observed_disposition,
            observed_token: observation.observed_token,
            observed_ranked_candidates_cid,
            matched,
        });
        Ok(())
    }

    pub fn finish(mut self) -> Result<CrossSurfaceParityEvidence, SourceUnavailable> {
        if self.records.is_empty() {
            return Err(SourceUnavailable::new(
                "cross-surface parity evidence requires at least one observation",
            ));
        }
        self.records
            .sort_by(|left, right| parity_record_key(left).cmp(&parity_record_key(right)));
        if self.records.windows(2).any(|pair| {
            pair[0].surface == pair[1].surface
                && pair[0].decode_policy == pair[1].decode_policy
                && pair[0].context_cid == pair[1].context_cid
                && pair[0].session_signature_cid == pair[1].session_signature_cid
        }) {
            return Err(SourceUnavailable::new(
                "duplicate cross-surface observation key",
            ));
        }
        let checks = self.records.len() as u64;
        let mismatches = self.records.iter().filter(|record| !record.matched).count() as u64;
        Ok(CrossSurfaceParityEvidence {
            schema: CROSS_SURFACE_PARITY_EVIDENCE_SCHEMA.to_string(),
            graph_cid: self.graph_cid,
            signature_artifact_cid: self.signature_artifact_cid,
            tokenizer_cid: self.tokenizer_cid,
            score_report_cid: self.score_report_cid,
            checks,
            mismatches,
            records: self.records,
        })
    }
}

fn parity_bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn parity_tokens_cid(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(8 + tokens.len() * 4);
    bytes.extend_from_slice(&(tokens.len() as u64).to_le_bytes());
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    parity_tagged_cid(b"r4-cross-surface-context/1", &[&bytes])
}

fn parity_candidates_cid(candidates: &ServedCandidates) -> String {
    let mut bytes = Vec::with_capacity(8 + candidates.len() * 11);
    bytes.extend_from_slice(&(candidates.len() as u64).to_le_bytes());
    for candidate in candidates.ranked() {
        bytes.extend_from_slice(&candidate.token.to_le_bytes());
        bytes.extend_from_slice(&candidate.score.raw().to_le_bytes());
        bytes.push(match candidate.source {
            ServedCandidateSource::Base => 0,
            ServedCandidateSource::Skipmix => 1,
        });
        bytes.push(u8::from(candidate.skmx_contributed));
        bytes.push(u8::from(candidate.psib_contributed));
    }
    parity_tagged_cid(b"r4-cross-surface-ranked-candidates/3", &[&bytes])
}

fn parity_tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Loaded production composition over one immutable graph generation.
pub struct NormativeServingEngine<'a> {
    runtime: R4G1Runtime<'a>,
    policy: ProductionPolicyEngine,
    node_scores: Vec<ScoreQ>,
}

impl<'a> NormativeServingEngine<'a> {
    /// Load through the production quality gate and validate that the same
    /// graph is consumable by the normative runtime, including optional lane
    /// sections.
    pub fn load(parts: ProductionServingParts<'a>) -> Result<Self, SourceUnavailable> {
        let policy = load_production_policy_engine(parts)?;
        // The CID-bound deployed report supersedes the old Gate C row for
        // admission. `score_report` remains D4/scorer configuration only.
        Self::load_inner(parts.engine, policy)
    }

    /// Explicit research loader. It preserves all structural and identity
    /// validation but does not claim production admission.
    pub fn load_for_research(parts: EngineParts<'a>) -> Result<Self, SourceUnavailable> {
        let policy = ProductionPolicyEngine::load_for_research(parts)?;
        Self::load_inner(parts, policy)
    }

    fn load_inner(
        parts: EngineParts<'a>,
        policy: ProductionPolicyEngine,
    ) -> Result<Self, SourceUnavailable> {
        let runtime = R4G1Runtime::parse(parts.graph).map_err(|error| {
            SourceUnavailable::new(format!("normative R4G1 runtime rejected graph: {error:?}"))
        })?;
        let node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
        Ok(Self {
            runtime,
            policy,
            node_scores,
        })
    }

    /// Select one production step. Every candidate and served token comes
    /// from `R4G1Runtime`; `R4Engine` contributes policy metadata only.
    pub fn predict(&mut self, window: &[u32]) -> Result<NormativeServingDecision, ObservedBound> {
        self.predict_with_session_signature(window, None)
    }

    /// Session-lane counterpart of [`Self::predict`]. The same adapter owns
    /// policy composition and candidate authority on both surfaces.
    pub fn predict_with_session_signature(
        &mut self,
        context_tokens: &[u32],
        session_signature: Option<&[u8]>,
    ) -> Result<NormativeServingDecision, ObservedBound> {
        NormativeStepAdapter::new(&mut self.policy, &self.runtime, &mut self.node_scores)
            .select(context_tokens, session_signature)
    }

    /// Reset bounded policy state between independent evidence positions.
    /// No reference-scoring or token-selecting API crosses this facade.
    pub fn reset_policy_state(&mut self) {
        self.policy.reset_policy_state();
    }

    /// Snapshot the policy counters without exposing the reference engine.
    pub fn policy_counters(&self) -> PolicyCounters {
        self.policy.policy_counters()
    }
}
