//! Content-bound replay evidence for the normative serving authority.
//!
//! The serving report may not accept caller-supplied replay counters. This
//! module emits canonical rows from one fresh [`NormativeServingEngine`] and
//! independently replays them through a second fresh engine. Parsing the
//! durable bytes performs the replay again, binds every input generation, and
//! recomputes every row verdict and aggregate count.

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::compiler::{self, Corpus};
use uor_r4_graph_runtime::ServedCandidateSource;
use uor_r4_model_source::SourceUnavailable;

use crate::deployed_quality::deployed_quality_positions_cid;
use crate::engine::{EngineParts, PolicyStatus};
use crate::serving::{NormativeServingDecision, NormativeServingEngine};

/// Canonical normative witness-replay artifact schema.
pub const NORMATIVE_WITNESS_REPLAY_SCHEMA: &str = "uor-r4-normative-witness-replay/2";

/// Stable schema-2 bundle path for the durable replay evidence.
pub const NORMATIVE_WITNESS_REPLAY_BUNDLE_PATH: &str = "graph/witness_replay.json";

/// Established Gate C witness sample size, applied to the evaluator's exact
/// selected population (or the whole population when it is smaller). This is
/// retained schema-1 behavior; changing it requires a schema/version decision.
pub const DEFAULT_NORMATIVE_WITNESS_SAMPLE: usize = 64;

/// Exact immutable inputs required to reproduce a serving witness.
#[derive(Debug, Clone, Copy)]
pub struct NormativeWitnessReplayMaterial<'a> {
    pub graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tokenizer: &'a [u8],
    pub score_report: Option<&'a [u8]>,
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
}

/// Replay material plus the evaluator's exact ordered position population.
#[derive(Debug, Clone, Copy)]
pub struct NormativeWitnessReplaySpec<'a> {
    pub material: NormativeWitnessReplayMaterial<'a>,
    pub evaluated_positions: &'a [u64],
    pub sample_size: usize,
}

/// Total disposition of one normative serving step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormativeWitnessDisposition {
    Serve,
    Abstain,
    Decline,
}

/// D4 policy status carried by a served or abstained step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormativeWitnessPolicyStatus {
    ExactContext,
    Graph,
    Novel,
    Contradictory,
}

impl From<PolicyStatus> for NormativeWitnessPolicyStatus {
    fn from(status: PolicyStatus) -> Self {
        match status {
            PolicyStatus::ExactContext => Self::ExactContext,
            PolicyStatus::Graph => Self::Graph,
            PolicyStatus::Novel => Self::Novel,
            PolicyStatus::Contradictory => Self::Contradictory,
        }
    }
}

/// Runtime score domain responsible for the authoritative winner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormativeWitnessCandidateSource {
    Base,
    Skipmix,
}

impl From<ServedCandidateSource> for NormativeWitnessCandidateSource {
    fn from(source: ServedCandidateSource) -> Self {
        match source {
            ServedCandidateSource::Base => Self::Base,
            ServedCandidateSource::Skipmix => Self::Skipmix,
        }
    }
}

/// Exact winner selected by [`uor_r4_graph_runtime::R4G1Runtime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeWitnessCandidate {
    pub token: u32,
    pub score_raw: i32,
    pub source: NormativeWitnessCandidateSource,
    pub skmx_contributed: bool,
    pub psib_contributed: bool,
}

/// Exact learned-lane promotion attached to the runtime winner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeWitnessLaneAttribution {
    pub base_token: u32,
    pub promoted_token: u32,
    pub contribution_raw: i32,
    pub skmx_contributed: bool,
    pub psib_contributed: bool,
}

/// Whether independent replay reproduced the durable row claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormativeWitnessReplayVerdict {
    Match,
    Mismatch,
}

/// One canonical position claim and its independently recomputed verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeWitnessReplayRecord {
    pub position: u64,
    pub disposition: NormativeWitnessDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_status: Option<NormativeWitnessPolicyStatus>,
    pub widened: bool,
    pub ngram_hit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate: Option<NormativeWitnessCandidate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane_attribution: Option<NormativeWitnessLaneAttribution>,
    pub replay_verdict: NormativeWitnessReplayVerdict,
}

/// Canonical, self-counting normative replay artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeWitnessReplayArtifact {
    pub schema: String,
    pub graph_cid: String,
    pub signature_artifact_cid: String,
    pub tokenizer_cid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_report_cid: Option<String>,
    pub corpus_meta_cid: String,
    pub corpus_records_cid: String,
    pub evaluated_positions_cid: String,
    pub sample_positions_cid: String,
    pub requested: u64,
    pub replayed: u64,
    pub failures: u64,
    pub records: Vec<NormativeWitnessReplayRecord>,
}

impl NormativeWitnessReplayArtifact {
    /// Deterministic pretty JSON with one trailing newline.
    pub fn deterministic_json_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            SourceUnavailable::new(format!(
                "serialize normative witness replay as deterministic JSON: {error}"
            ))
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReplayClaim {
    disposition: NormativeWitnessDisposition,
    policy_status: Option<NormativeWitnessPolicyStatus>,
    widened: bool,
    ngram_hit: bool,
    candidate: Option<NormativeWitnessCandidate>,
    lane_attribution: Option<NormativeWitnessLaneAttribution>,
}

impl ReplayClaim {
    fn from_decision(decision: NormativeServingDecision) -> Result<Self, SourceUnavailable> {
        match decision {
            NormativeServingDecision::Serve(outcome) => {
                let winner = outcome.candidates.winner().ok_or_else(|| {
                    SourceUnavailable::new("served decision has no authoritative runtime winner")
                })?;
                Ok(Self {
                    disposition: NormativeWitnessDisposition::Serve,
                    policy_status: Some(PolicyStatus::from(outcome.status).into()),
                    widened: outcome.widened,
                    ngram_hit: outcome.ngram_hit,
                    candidate: Some(NormativeWitnessCandidate {
                        token: winner.token,
                        score_raw: winner.score.raw(),
                        source: winner.source.into(),
                        skmx_contributed: winner.skmx_contributed,
                        psib_contributed: winner.psib_contributed,
                    }),
                    lane_attribution: outcome.candidates.attribution().map(|attribution| {
                        NormativeWitnessLaneAttribution {
                            base_token: attribution.base_token,
                            promoted_token: attribution.promoted_token,
                            contribution_raw: attribution.contribution.raw(),
                            skmx_contributed: attribution.skmx_contributed,
                            psib_contributed: attribution.psib_contributed,
                        }
                    }),
                })
            }
            NormativeServingDecision::Abstain(outcome) => Ok(Self {
                disposition: NormativeWitnessDisposition::Abstain,
                policy_status: Some(PolicyStatus::from(outcome.status).into()),
                widened: outcome.widened,
                ngram_hit: outcome.ngram_hit,
                candidate: None,
                lane_attribution: None,
            }),
            NormativeServingDecision::Decline(_) => Ok(Self {
                disposition: NormativeWitnessDisposition::Decline,
                policy_status: None,
                widened: false,
                ngram_hit: false,
                candidate: None,
                lane_attribution: None,
            }),
        }
    }

    fn from_record(record: &NormativeWitnessReplayRecord) -> Self {
        Self {
            disposition: record.disposition,
            policy_status: record.policy_status,
            widened: record.widened,
            ngram_hit: record.ngram_hit,
            candidate: record.candidate,
            lane_attribution: record.lane_attribution,
        }
    }

    fn into_record(
        self,
        position: u64,
        replay_verdict: NormativeWitnessReplayVerdict,
    ) -> NormativeWitnessReplayRecord {
        NormativeWitnessReplayRecord {
            position,
            disposition: self.disposition,
            policy_status: self.policy_status,
            widened: self.widened,
            ngram_hit: self.ngram_hit,
            candidate: self.candidate,
            lane_attribution: self.lane_attribution,
            replay_verdict,
        }
    }
}

/// Select the canonical witness sample from an evaluator's exact positions.
///
/// This retains Gate C's established first-N rule while making ordering and
/// duplicate assumptions explicit. The returned sample is never empty.
pub fn select_normative_witness_positions(
    evaluated_positions: &[u64],
    sample_size: usize,
) -> Result<Vec<u64>, SourceUnavailable> {
    if evaluated_positions.is_empty() {
        return Err(SourceUnavailable::new(
            "normative witness population is empty",
        ));
    }
    if sample_size == 0 {
        return Err(SourceUnavailable::new(
            "normative witness sample size is zero",
        ));
    }
    if evaluated_positions
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err(SourceUnavailable::new(
            "normative witness population must be strictly increasing",
        ));
    }
    Ok(evaluated_positions
        .iter()
        .copied()
        .take(sample_size)
        .collect())
}

/// Emit content-bound observations and independently replay them.
pub fn produce_normative_witness_replay(
    spec: NormativeWitnessReplaySpec<'_>,
) -> Result<NormativeWitnessReplayArtifact, SourceUnavailable> {
    let positions = select_normative_witness_positions(spec.evaluated_positions, spec.sample_size)?;
    let corpus = parse_corpus(spec.material)?;
    validate_positions(&positions, &corpus)?;

    let mut observer = load_engine(spec.material)?;
    let mut observed = Vec::with_capacity(positions.len());
    for &position in &positions {
        observed.push(replay_position(&mut observer, &corpus, position)?);
    }

    // A second fresh engine is deliberate: artifact production does not mark
    // its own observation pass as replay evidence.
    let mut replayer = load_engine(spec.material)?;
    let mut records = Vec::with_capacity(positions.len());
    let mut failures = 0u64;
    for (&position, claim) in positions.iter().zip(observed) {
        let replayed = replay_position(&mut replayer, &corpus, position)?;
        let verdict = if replayed == claim {
            NormativeWitnessReplayVerdict::Match
        } else {
            failures = failures
                .checked_add(1)
                .ok_or_else(|| SourceUnavailable::new("witness failure count overflow"))?;
            NormativeWitnessReplayVerdict::Mismatch
        };
        records.push(claim.into_record(position, verdict));
    }
    let replayed = u64::try_from(records.len())
        .map_err(|_| SourceUnavailable::new("witness replay count exceeds u64"))?;
    Ok(artifact_from_parts(
        spec, positions, records, replayed, failures,
    ))
}

/// Parse canonical bytes, bind every immutable input, replay each row through
/// the normative runtime path, and recompute all verdicts and counts.
pub fn parse_and_validate_normative_witness_replay(
    bytes: &[u8],
    spec: NormativeWitnessReplaySpec<'_>,
) -> Result<NormativeWitnessReplayArtifact, SourceUnavailable> {
    let artifact: NormativeWitnessReplayArtifact = serde_json::from_slice(bytes)
        .map_err(|error| SourceUnavailable::new(format!("invalid witness replay JSON: {error}")))?;
    let canonical = artifact.deterministic_json_bytes()?;
    if canonical != bytes {
        return Err(SourceUnavailable::new(
            "witness replay artifact bytes are not canonical",
        ));
    }
    validate_artifact(&artifact, spec)?;
    Ok(artifact)
}

fn validate_artifact(
    artifact: &NormativeWitnessReplayArtifact,
    spec: NormativeWitnessReplaySpec<'_>,
) -> Result<(), SourceUnavailable> {
    if artifact.schema != NORMATIVE_WITNESS_REPLAY_SCHEMA {
        return Err(SourceUnavailable::new(format!(
            "witness replay schema {} is unsupported",
            artifact.schema
        )));
    }
    let positions = select_normative_witness_positions(spec.evaluated_positions, spec.sample_size)?;
    let expected = artifact_from_parts(spec, positions.clone(), Vec::new(), 0, 0);
    if artifact.graph_cid != expected.graph_cid
        || artifact.signature_artifact_cid != expected.signature_artifact_cid
        || artifact.tokenizer_cid != expected.tokenizer_cid
        || artifact.score_report_cid != expected.score_report_cid
        || artifact.corpus_meta_cid != expected.corpus_meta_cid
        || artifact.corpus_records_cid != expected.corpus_records_cid
        || artifact.evaluated_positions_cid != expected.evaluated_positions_cid
        || artifact.sample_positions_cid != expected.sample_positions_cid
    {
        return Err(SourceUnavailable::new(
            "witness replay artifact identity does not match the loaded generation",
        ));
    }
    if artifact.records.len() != positions.len()
        || artifact
            .records
            .iter()
            .zip(&positions)
            .any(|(record, position)| record.position != *position)
    {
        return Err(SourceUnavailable::new(
            "witness replay rows do not equal the canonical position sample",
        ));
    }

    let corpus = parse_corpus(spec.material)?;
    validate_positions(&positions, &corpus)?;
    let mut replayer = load_engine(spec.material)?;
    let mut failures = 0u64;
    for record in &artifact.records {
        validate_record_shape(record)?;
        let replayed = replay_position(&mut replayer, &corpus, record.position)?;
        let verdict = if replayed == ReplayClaim::from_record(record) {
            NormativeWitnessReplayVerdict::Match
        } else {
            failures = failures
                .checked_add(1)
                .ok_or_else(|| SourceUnavailable::new("witness failure count overflow"))?;
            NormativeWitnessReplayVerdict::Mismatch
        };
        if record.replay_verdict != verdict {
            return Err(SourceUnavailable::new(format!(
                "witness replay verdict at position {} does not reproduce",
                record.position
            )));
        }
    }
    let replayed = u64::try_from(artifact.records.len())
        .map_err(|_| SourceUnavailable::new("witness replay count exceeds u64"))?;
    if artifact.requested != replayed
        || artifact.replayed != replayed
        || artifact.failures != failures
    {
        return Err(SourceUnavailable::new(format!(
            "witness replay counts do not reproduce: artifact has {}/{}/{}, replay has {replayed}/{replayed}/{failures}",
            artifact.requested, artifact.replayed, artifact.failures
        )));
    }
    Ok(())
}

fn validate_record_shape(record: &NormativeWitnessReplayRecord) -> Result<(), SourceUnavailable> {
    let valid = match record.disposition {
        NormativeWitnessDisposition::Serve => {
            record.policy_status.is_some()
                && record.candidate.is_some()
                && record
                    .candidate
                    .is_some_and(|candidate| match candidate.source {
                        NormativeWitnessCandidateSource::Base => {
                            !candidate.skmx_contributed && !candidate.psib_contributed
                        }
                        NormativeWitnessCandidateSource::Skipmix => {
                            candidate.skmx_contributed || candidate.psib_contributed
                        }
                    })
                && record.lane_attribution.is_none_or(|attribution| {
                    record.candidate.is_some_and(|candidate| {
                        candidate.source == NormativeWitnessCandidateSource::Skipmix
                            && attribution.promoted_token == candidate.token
                            && attribution.contribution_raw == candidate.score_raw
                            && attribution.base_token != attribution.promoted_token
                            && (attribution.skmx_contributed || attribution.psib_contributed)
                    })
                })
                && record.candidate.is_none_or(|candidate| {
                    candidate.source == NormativeWitnessCandidateSource::Skipmix
                        || record.lane_attribution.is_none()
                })
        }
        NormativeWitnessDisposition::Abstain => {
            record.policy_status.is_some()
                && record.candidate.is_none()
                && record.lane_attribution.is_none()
        }
        NormativeWitnessDisposition::Decline => {
            record.policy_status.is_none()
                && !record.widened
                && !record.ngram_hit
                && record.candidate.is_none()
                && record.lane_attribution.is_none()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(SourceUnavailable::new(format!(
            "witness replay record at position {} has fields inconsistent with its disposition",
            record.position
        )))
    }
}

fn artifact_from_parts(
    spec: NormativeWitnessReplaySpec<'_>,
    positions: Vec<u64>,
    records: Vec<NormativeWitnessReplayRecord>,
    replayed: u64,
    failures: u64,
) -> NormativeWitnessReplayArtifact {
    NormativeWitnessReplayArtifact {
        schema: NORMATIVE_WITNESS_REPLAY_SCHEMA.to_string(),
        graph_cid: bytes_cid(spec.material.graph),
        signature_artifact_cid: bytes_cid(spec.material.signature_artifact),
        tokenizer_cid: bytes_cid(spec.material.tokenizer),
        score_report_cid: spec.material.score_report.map(bytes_cid),
        corpus_meta_cid: bytes_cid(spec.material.corpus_meta),
        corpus_records_cid: bytes_cid(spec.material.corpus_records),
        evaluated_positions_cid: deployed_quality_positions_cid(spec.evaluated_positions),
        sample_positions_cid: deployed_quality_positions_cid(&positions),
        requested: records.len() as u64,
        replayed,
        failures,
        records,
    }
}

fn parse_corpus(material: NormativeWitnessReplayMaterial<'_>) -> Result<Corpus, SourceUnavailable> {
    compiler::load_corpus_bytes(material.corpus_meta, material.corpus_records, None)
        .ok_or_else(|| SourceUnavailable::new("witness replay corpus bytes are invalid"))
}

fn validate_positions(positions: &[u64], corpus: &Corpus) -> Result<(), SourceUnavailable> {
    if positions.iter().any(|&position| {
        usize::try_from(position)
            .ok()
            .is_none_or(|position| position >= corpus.n)
    }) {
        return Err(SourceUnavailable::new(
            "witness replay position is outside the bound corpus",
        ));
    }
    Ok(())
}

fn load_engine(
    material: NormativeWitnessReplayMaterial<'_>,
) -> Result<NormativeServingEngine<'_>, SourceUnavailable> {
    NormativeServingEngine::load_for_research(EngineParts {
        graph: material.graph,
        signature_artifact: material.signature_artifact,
        tokenizer: Some(material.tokenizer),
        score_report: material.score_report,
    })
}

fn replay_position(
    engine: &mut NormativeServingEngine<'_>,
    corpus: &Corpus,
    position: u64,
) -> Result<ReplayClaim, SourceUnavailable> {
    let position = usize::try_from(position)
        .map_err(|_| SourceUnavailable::new("witness replay position exceeds usize"))?;
    // Serving evaluation treats every teacher-forced position as an isolated
    // decision. Resetting here is part of the replay contract, not cleanup.
    engine.reset_policy_state();
    let window = compiler::context_window(corpus, position);
    let decision = engine
        .predict(&window)
        .map_err(|error| SourceUnavailable::new(format!("witness replay decision: {error}")))?;
    ReplayClaim::from_decision(decision)
}

fn bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}
