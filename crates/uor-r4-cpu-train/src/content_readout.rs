//! Exact Hamming nearest-example content readout on generated prefixes.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use uor_r4_integer::stack::{
    reply_stop, ContentReadout as ServingContentReadout, IntegerStackModel, ReplyEnd, POINT_KEY_BOS,
};

use crate::reply_eval::normalized_reference;
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CONTENT_SIGNATURE_BITS: usize = 256;
pub const CONTENT_CODE_SIGNATURE_BITS: usize = CONTENT_SIGNATURE_BITS * 8;
pub const CONTENT_TRAJECTORY_SIGNATURE_BITS: usize = CONTENT_CODE_SIGNATURE_BITS * 2;
pub const CONTENT_MIN_TRAINING_ROWS: usize = 640;
pub const CONTENT_MIN_TARGET_VALUES: usize = 16;
pub type ContentSignature = Vec<u64>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSignatureMode {
    #[default]
    Sign,
    Code,
    Trajectory,
}

impl ContentSignatureMode {
    pub fn bits(self) -> usize {
        match self {
            Self::Sign => CONTENT_SIGNATURE_BITS,
            Self::Code => CONTENT_CODE_SIGNATURE_BITS,
            Self::Trajectory => CONTENT_TRAJECTORY_SIGNATURE_BITS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExample {
    pub signature: ContentSignature,
    pub target: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadout {
    pub mode: ContentSignatureMode,
    pub signature_bits: usize,
    #[serde(default)]
    pub includes_reference_end: bool,
    pub rows: Vec<ContentExample>,
    pub max_distance: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadoutTraining {
    pub readout: ContentReadout,
    pub rows: u64,
    pub target_values: u64,
    pub leave_one_out_top1: u64,
    pub leave_one_out_ambiguous: u64,
    pub covered_rows: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadoutReading {
    pub prompts: u64,
    pub decisions: u64,
    pub generated_decisions: u64,
    pub readout_decisions: u64,
    pub writer_fallbacks: u64,
    pub top1: u64,
    pub matched_prefix_depth_sum: u64,
    pub opening_correct: u64,
    pub exact_reference: u64,
    pub bos_stopped: u64,
    pub cap_stopped: u64,
    pub context_stopped: u64,
    pub generated_tokens: u64,
    pub distinct_non_empty: u64,
    pub repeated_4gram_replies: u64,
    pub nearest_distance_sum: u64,
    pub ambiguous_readout_decisions: u64,
    pub gated_decisions: u64,
    pub gate_fires: u64,
    pub ungated_token_mismatches: u64,
    pub first_divergence: BTreeMap<usize, u64>,
}

/// Decision-for-decision parity between this CPU readout path and the integer
/// stack engine carrying the same artifact-bound sidecar.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentServingParity {
    pub prompts: u64,
    pub decisions: u64,
    pub readout_fires: u64,
    pub native_fallbacks: u64,
    pub token_mismatches: u64,
    pub decision_mismatches: u64,
    pub first_mismatch: Option<String>,
    pub cpu_decision_sha256: String,
    pub serving_decision_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentPromptReading {
    pub sample: usize,
    pub matched_prefix_depth: usize,
    pub top1: u64,
    pub observed_decisions: u64,
    pub exact_reference: bool,
    pub stop: String,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub generated_ids: Vec<u32>,
    pub decision_sha256: String,
}

pub fn pack_content_signature(
    query: &[i32],
    mode: ContentSignatureMode,
) -> Result<ContentSignature> {
    if query.len() != CONTENT_SIGNATURE_BITS {
        return Err(invalid("content signature query width differs"));
    }
    let mut signature = vec![0u64; mode.bits() / 64];
    match mode {
        ContentSignatureMode::Sign => {
            for (index, &value) in query.iter().enumerate() {
                if value >= 0 {
                    signature[index / 64] |= 1u64 << (index % 64);
                }
            }
        }
        ContentSignatureMode::Code => {
            for (index, &value) in query.iter().enumerate() {
                let bit = index * 8;
                signature[bit / 64] |= u64::from(value as u8) << (bit % 64);
            }
        }
        ContentSignatureMode::Trajectory => {
            return Err(invalid(
                "trajectory signature needs the previous generated state",
            ));
        }
    }
    Ok(signature)
}

pub fn pack_content_trajectory_signature(
    query: &[i32],
    previous: Option<&[u64]>,
) -> Result<ContentSignature> {
    let current = pack_content_signature(query, ContentSignatureMode::Code)?;
    pack_content_trajectory_from_code(&current, previous)
}

fn pack_content_trajectory_from_code(
    current: &[u64],
    previous: Option<&[u64]>,
) -> Result<ContentSignature> {
    if previous.is_some_and(|previous| previous.len() != current.len()) {
        return Err(invalid("content trajectory previous width differs"));
    }
    let mut signature = Vec::with_capacity(CONTENT_TRAJECTORY_SIGNATURE_BITS / 64);
    signature.extend_from_slice(current);
    signature.extend(
        current
            .iter()
            .enumerate()
            .map(|(index, word)| previous.map_or(0, |previous| word ^ previous[index])),
    );
    Ok(signature)
}

pub fn content_hamming(left: &ContentSignature, right: &ContentSignature) -> u32 {
    left.iter()
        .zip(right)
        .map(|(&left, &right)| (left ^ right).count_ones())
        .sum()
}

pub fn derive_content_readout(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    mode: ContentSignatureMode,
) -> Result<ContentReadoutTraining> {
    derive_content_readout_with_endpoint(artifact, samples, cap, memory_enabled, mode, false)
}

pub fn derive_content_stop_readout(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
) -> Result<ContentReadoutTraining> {
    derive_content_readout_with_endpoint(
        artifact,
        samples,
        cap,
        memory_enabled,
        ContentSignatureMode::Code,
        true,
    )
}

fn derive_content_readout_with_endpoint(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    mode: ContentSignatureMode,
    include_reference_end: bool,
) -> Result<ContentReadoutTraining> {
    if samples.is_empty() {
        return Err(invalid("content readout needs training samples"));
    }
    let rows = samples
        .par_iter()
        .map(|sample| {
            generate_with_readout(
                artifact,
                sample,
                cap,
                memory_enabled,
                mode,
                include_reference_end,
                None,
            )
            .map(|(decisions, _)| {
                decisions
                    .into_iter()
                    .filter_map(|decision| {
                        decision.reference_target.map(|target| ContentExample {
                            signature: decision.signature,
                            target,
                        })
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if rows.len() < CONTENT_MIN_TRAINING_ROWS {
        return Err(invalid("content readout has fewer than 640 rows"));
    }
    let target_values = rows
        .iter()
        .map(|row| row.target)
        .collect::<BTreeSet<_>>()
        .len();
    if target_values < CONTENT_MIN_TARGET_VALUES {
        return Err(invalid("content readout target diversity is too small"));
    }
    fit_content_readout(rows, target_values, mode, include_reference_end)
}

fn fit_content_readout(
    rows: Vec<ContentExample>,
    target_values: usize,
    mode: ContentSignatureMode,
    includes_reference_end: bool,
) -> Result<ContentReadoutTraining> {
    if rows.is_empty() {
        return Err(invalid("content readout fitting needs rows"));
    }
    let signature_words = mode.bits() / 64;
    if rows
        .iter()
        .any(|row| row.signature.len() != signature_words)
    {
        return Err(invalid("content readout signature width differs"));
    }
    let leave_one_out = rows
        .par_iter()
        .enumerate()
        .map(|(index, row)| {
            nearest_target(&rows, index, &row.signature)?
                .ok_or_else(|| invalid("content readout has no leave-one-out row"))
                .map(|nearest| {
                    (
                        u64::from(nearest.target == row.target),
                        u64::from(nearest.ambiguous),
                        nearest.distance,
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    let leave_one_out_top1 = leave_one_out.iter().map(|(correct, _, _)| correct).sum();
    let leave_one_out_ambiguous = leave_one_out
        .iter()
        .map(|(_, ambiguous, _)| ambiguous)
        .sum();
    let mut distances = leave_one_out
        .into_iter()
        .map(|(_, _, distance)| distance)
        .collect::<Vec<_>>();
    distances.sort_unstable();
    let required_rows = rows
        .len()
        .checked_mul(9)
        .ok_or_else(|| invalid("content readout coverage overflows"))?
        .div_ceil(10);
    let max_distance = distances[required_rows - 1];
    let covered_rows = distances
        .iter()
        .filter(|&&distance| distance <= max_distance)
        .count() as u64;
    if covered_rows
        .checked_mul(10)
        .is_none_or(|covered| covered < (rows.len() as u64) * 9)
    {
        return Err(invalid("content readout coverage is below 90 percent"));
    }
    Ok(ContentReadoutTraining {
        readout: ContentReadout {
            mode,
            signature_bits: mode.bits(),
            includes_reference_end,
            rows,
            max_distance,
        },
        rows: distances.len() as u64,
        target_values: target_values as u64,
        leave_one_out_top1,
        leave_one_out_ambiguous,
        covered_rows,
    })
}

pub fn shuffle_content_targets(readout: &ContentReadout, seed: u64) -> Result<ContentReadout> {
    if readout.rows.len() < 2 {
        return Err(invalid("content target shuffle needs at least two rows"));
    }
    let original_mapping = content_target_mapping(readout)?;
    let mut targets = readout
        .rows
        .iter()
        .map(|row| row.target)
        .collect::<Vec<_>>();
    let shift = 1 + seed as usize % (targets.len() - 1);
    targets.rotate_right(shift);
    if targets
        .iter()
        .zip(&readout.rows)
        .all(|(&target, row)| target == row.target)
    {
        return Err(invalid("content target shuffle is vacuous"));
    }
    let shuffled = ContentReadout {
        mode: readout.mode,
        signature_bits: readout.signature_bits,
        includes_reference_end: readout.includes_reference_end,
        rows: readout
            .rows
            .iter()
            .cloned()
            .zip(targets)
            .map(|(row, target)| ContentExample {
                signature: row.signature,
                target,
            })
            .collect(),
        max_distance: readout.max_distance,
    };
    if content_target_mapping(&shuffled)? == original_mapping {
        return Err(invalid(
            "content target shuffle leaves the effective mapping unchanged",
        ));
    }
    Ok(shuffled)
}

pub fn content_target_mapping(readout: &ContentReadout) -> Result<BTreeMap<ContentSignature, u32>> {
    validate_content_readout(readout)?;
    let signatures = readout
        .rows
        .iter()
        .map(|row| row.signature.clone())
        .collect::<BTreeSet<_>>();
    signatures
        .into_iter()
        .map(|signature| {
            let target = nearest_target(&readout.rows, readout.rows.len(), &signature)?
                .ok_or_else(|| invalid("content readout has no effective target"))?
                .target;
            Ok((signature, target))
        })
        .collect()
}

pub fn content_mapping_sha256(readout: &ContentReadout) -> Result<String> {
    let mapping = content_target_mapping(readout)?;
    let mut bytes = Vec::with_capacity(mapping.len() * (readout.signature_bits / 8 + 4));
    bytes.extend_from_slice(&(mapping.len() as u64).to_le_bytes());
    for (signature, target) in mapping {
        for word in signature {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&target.to_le_bytes());
    }
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

pub fn evaluate_content_readout(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    mode: ContentSignatureMode,
    readout: Option<&ContentReadout>,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    if let Some(readout) = readout {
        validate_content_readout(readout)?;
        if readout.mode != mode {
            return Err(invalid("content readout evaluation mode differs"));
        }
    }
    evaluate_generated(samples, cap, |sample| {
        generate_with_readout(artifact, sample, cap, memory_enabled, mode, false, readout)
    })
}

fn evaluate_generated(
    samples: &[MemoryWeightSample],
    cap: usize,
    generate: impl Fn(&MemoryWeightSample) -> Result<(Vec<Decision>, Generated)> + Sync,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    let mut reading = ContentReadoutReading {
        prompts: samples.len() as u64,
        ..ContentReadoutReading::default()
    };
    let evaluations = samples
        .par_iter()
        .enumerate()
        .map(|(sample_index, sample)| {
            generate(sample).map(|(decisions, reply)| (sample_index, sample, decisions, reply))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut prompts = Vec::with_capacity(evaluations.len());
    let mut distinct = BTreeSet::new();
    for (sample_index, sample, decisions, reply) in evaluations {
        let reference = normalized_reference(&sample.reply);
        let horizon = reference.len().min(cap);
        let mut matched = 0usize;
        let mut top1 = 0u64;
        let mut observed = 0u64;
        for (offset, &target) in reference.iter().enumerate().take(horizon) {
            if let Some(decision) = decisions.iter().find(|decision| decision.offset == offset) {
                observed += 1;
                reading.decisions += 1;
                let correct = u64::from(decision.chosen == target);
                top1 += correct;
                reading.top1 += correct;
                if correct == 1 && matched == offset {
                    matched += 1;
                }
            }
        }
        reading.matched_prefix_depth_sum += matched as u64;
        *reading.first_divergence.entry(matched).or_default() += 1;
        reading.opening_correct += u64::from(reply.tokens.first() == reference.first());
        let exact = reply.tokens == reference && reply.stop == "bos";
        reading.exact_reference += u64::from(exact);
        match reply.stop {
            "bos" => reading.bos_stopped += 1,
            "cap" => reading.cap_stopped += 1,
            "context" => reading.context_stopped += 1,
            _ => return Err(invalid("content readout stop differs")),
        }
        reading.generated_tokens += reply.tokens.len() as u64;
        reading.generated_decisions += decisions.len() as u64;
        if !reply.tokens.is_empty() {
            distinct.insert(reply.tokens.clone());
        }
        reading.repeated_4gram_replies += u64::from(has_repeated_4gram(&reply.tokens));
        for decision in &decisions {
            reading.gated_decisions += u64::from(decision.gate_fired);
            reading.gate_fires += u64::from(decision.gate_fired);
            reading.ungated_token_mismatches +=
                u64::from(!decision.gate_fired && decision.chosen != decision.content_chosen);
            if let Some(distance) = decision.nearest_distance {
                reading.nearest_distance_sum += u64::from(distance);
                reading.readout_decisions += u64::from(decision.used_readout);
                reading.writer_fallbacks += u64::from(!decision.used_readout);
                reading.ambiguous_readout_decisions +=
                    u64::from(decision.used_readout && decision.ambiguous_nearest);
            }
        }
        prompts.push(ContentPromptReading {
            sample: sample_index,
            matched_prefix_depth: matched,
            top1,
            observed_decisions: observed,
            exact_reference: exact,
            stop: reply.stop.to_owned(),
            generated_tokens: reply.tokens.len() as u64,
            reference_tokens: reference.len() as u64,
            generated_ids: reply.tokens.clone(),
            decision_sha256: uor_r4_discrete::corpus::sha256(
                &serde_json::to_vec(&(&decisions, &reply.tokens, reply.stop))
                    .map_err(|error| invalid(format!("content decision trace: {error}")))?,
            ),
        });
    }
    reading.distinct_non_empty = distinct.len() as u64;
    Ok((reading, prompts))
}

#[derive(serde::Serialize)]
struct Decision {
    offset: usize,
    chosen: u32,
    content_chosen: u32,
    writer_chosen: u32,
    gate_fired: bool,
    signature: ContentSignature,
    reference_target: Option<u32>,
    nearest_distance: Option<u32>,
    ambiguous_nearest: bool,
    used_readout: bool,
}

struct Generated {
    tokens: Vec<u32>,
    stop: &'static str,
}

fn generate_with_readout(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    memory_enabled: bool,
    mode: ContentSignatureMode,
    include_reference_end: bool,
    readout: Option<&ContentReadout>,
) -> Result<(Vec<Decision>, Generated)> {
    if cap == 0 || cap > 64 || sample.prompt.is_empty() {
        return Err(invalid("content readout sample, prompt or cap differs"));
    }
    let reference = normalized_reference(&sample.reply);
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("content readout forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    let mut current = None;
    for (index, &token) in sample.prompt.iter().enumerate() {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content readout prompt step: {error}")))?;
        if index + 1 == sample.prompt.len() {
            current = Some(step);
        }
    }
    let mut current = current.ok_or_else(|| invalid("content readout prompt is empty"))?;
    let mut decisions = Vec::new();
    let mut tokens = Vec::new();
    let mut previous_signature = None;
    let stop = loop {
        let offset = decisions.len();
        let code_signature =
            pack_content_signature(&current.query_q12, ContentSignatureMode::Code)?;
        let signature = match mode {
            ContentSignatureMode::Trajectory => {
                pack_content_trajectory_from_code(&code_signature, previous_signature.as_deref())?
            }
            ContentSignatureMode::Code => code_signature.clone(),
            ContentSignatureMode::Sign => pack_content_signature(&current.query_q12, mode)?,
        };
        previous_signature = Some(code_signature);
        let (chosen, nearest_distance, ambiguous_nearest, used_readout) = match readout {
            Some(readout) => select_content_target(readout, &signature, current.token)?,
            None => (current.token, None, false, false),
        };
        decisions.push(Decision {
            offset,
            chosen,
            content_chosen: chosen,
            writer_chosen: current.token,
            gate_fired: false,
            signature,
            reference_target: reference
                .get(offset)
                .copied()
                .or_else(|| (include_reference_end && offset == reference.len()).then_some(0)),
            nearest_distance,
            ambiguous_nearest,
            used_readout,
        });
        let end = reply_stop(chosen, offset, sample.prompt.len(), cap, context);
        if end != Some(ReplyEnd::Bos) {
            tokens.push(chosen);
        }
        if let Some(end) = end {
            break match end {
                ReplyEnd::Bos => "bos",
                ReplyEnd::Length => "cap",
                ReplyEnd::Context => "context",
            };
        }
        current = forward
            .step(chosen, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content readout generated step: {error}")))?;
    };
    Ok((decisions, Generated { tokens, stop }))
}

fn validate_content_readout(readout: &ContentReadout) -> Result<()> {
    if readout.rows.is_empty() {
        return Err(invalid("content readout has no rows"));
    }
    if readout.signature_bits != readout.mode.bits() {
        return Err(invalid("content readout signature bits differ"));
    }
    if readout
        .rows
        .iter()
        .any(|row| row.signature.len() != readout.signature_bits / 64)
    {
        return Err(invalid("content readout signature row width differs"));
    }
    if readout.max_distance as usize > readout.signature_bits {
        return Err(invalid("content readout threshold exceeds the signature"));
    }
    if readout.includes_reference_end && !readout.rows.iter().any(|row| row.target == 0) {
        return Err(invalid("content stop readout has no BOS row"));
    }
    Ok(())
}

fn select_content_target(
    readout: &ContentReadout,
    signature: &ContentSignature,
    writer_token: u32,
) -> Result<(u32, Option<u32>, bool, bool)> {
    let nearest = nearest_target(&readout.rows, readout.rows.len(), signature)?
        .ok_or_else(|| invalid("content readout has no rows"))?;
    if nearest.distance <= readout.max_distance {
        Ok((
            nearest.target,
            Some(nearest.distance),
            nearest.ambiguous,
            true,
        ))
    } else {
        Ok((
            writer_token,
            Some(nearest.distance),
            nearest.ambiguous,
            false,
        ))
    }
}

struct NearestTarget {
    target: u32,
    distance: u32,
    ambiguous: bool,
}

fn nearest_target(
    rows: &[ContentExample],
    skip: usize,
    signature: &ContentSignature,
) -> Result<Option<NearestTarget>> {
    let mut best = u32::MAX;
    let mut counts = BTreeMap::<u32, usize>::new();
    for (index, row) in rows.iter().enumerate() {
        if index == skip {
            continue;
        }
        let distance = content_hamming(signature, &row.signature);
        if distance < best {
            best = distance;
            counts.clear();
            counts.insert(row.target, 1);
        } else if distance == best {
            *counts.entry(row.target).or_default() += 1;
        }
    }
    let candidate_targets = counts.len();
    let target = counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(target, _)| target);
    Ok(target.map(|target| NearestTarget {
        target,
        distance: best,
        ambiguous: candidate_targets > 1,
    }))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopRow {
    pub signature: ContentSignature,
    pub terminal: bool,
    pub writer_token: u32,
    pub offset: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopGate {
    pub signature_bits: usize,
    pub terminal_prototypes: Vec<ContentSignature>,
    pub nonterminal_prototypes: Vec<ContentSignature>,
    pub margin: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopTraining {
    pub gate: ContentStopGate,
    pub terminal_rows: u64,
    pub nonterminal_rows: u64,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub nonterminal_stop_limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopDensityPoint {
    pub signature: ContentSignature,
    pub offset: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopDensityGate {
    pub signature_bits: usize,
    pub terminal_rows: Vec<ContentStopDensityPoint>,
    pub nonterminal_rows: Vec<ContentStopDensityPoint>,
    pub neighbors: usize,
    pub offset_radius: u16,
    pub margin: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentStopDensityTraining {
    pub gate: ContentStopDensityGate,
    pub terminal_rows: u64,
    pub nonterminal_rows: u64,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub nonterminal_stop_limit: u64,
    pub leave_one_out_neighbors: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContinuationRow {
    pub signature: ContentSignature,
    pub remaining: i16,
    pub writer_token: u32,
    pub offset: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContinuationPoint {
    pub signature: ContentSignature,
    pub remaining: i16,
    pub offset: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContinuationGate {
    pub signature_bits: usize,
    pub rows: Vec<ContentContinuationPoint>,
    pub neighbors: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContinuationCandidate {
    pub neighbors: usize,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub horizon_abs_error: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContinuationTraining {
    pub gate: ContentContinuationGate,
    pub rows: u64,
    pub terminal_rows: u64,
    pub nonterminal_rows: u64,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub nonterminal_stop_limit: u64,
    pub horizon_abs_error: u64,
    pub candidates: Vec<ContentContinuationCandidate>,
}

pub fn derive_content_stop_rows(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
) -> Result<Vec<ContentStopRow>> {
    validate_content_readout(readout)?;
    if readout.mode != ContentSignatureMode::Code || readout.includes_reference_end {
        return Err(invalid("content stop rows need a plain code readout"));
    }
    let rows = samples
        .par_iter()
        .map(|sample| {
            let reference_len = normalized_reference(&sample.reply).len();
            generate_with_readout(
                artifact,
                sample,
                cap,
                memory_enabled,
                ContentSignatureMode::Code,
                false,
                Some(readout),
            )
            .map(|(decisions, _)| {
                decisions
                    .into_iter()
                    .map(|decision| ContentStopRow {
                        signature: decision.signature,
                        terminal: decision.offset == reference_len,
                        writer_token: decision.content_chosen,
                        offset: decision.offset as u16,
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if !rows.iter().any(|row| row.terminal) || !rows.iter().any(|row| !row.terminal) {
        return Err(invalid("content stop rows need both classes"));
    }
    Ok(rows)
}

pub fn derive_content_continuation_rows(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
) -> Result<Vec<ContentContinuationRow>> {
    validate_content_readout(readout)?;
    if readout.mode != ContentSignatureMode::Code || readout.includes_reference_end {
        return Err(invalid(
            "content continuation rows need a plain code readout",
        ));
    }
    if samples.is_empty() {
        return Err(invalid("content continuation rows need samples"));
    }
    let rows = samples
        .par_iter()
        .map(|sample| {
            let reference_len = normalized_reference(&sample.reply).len();
            generate_with_readout(
                artifact,
                sample,
                cap,
                memory_enabled,
                ContentSignatureMode::Code,
                false,
                Some(readout),
            )
            .and_then(|(decisions, _)| {
                decisions
                    .into_iter()
                    .map(|decision| {
                        let remaining = i32::try_from(reference_len)
                            .ok()
                            .and_then(|length| {
                                i32::try_from(decision.offset)
                                    .ok()
                                    .map(|offset| length - offset)
                            })
                            .and_then(|remaining| i16::try_from(remaining).ok())
                            .ok_or_else(|| {
                                invalid("content continuation horizon is out of range")
                            })?;
                        Ok(ContentContinuationRow {
                            signature: decision.signature,
                            remaining,
                            writer_token: decision.content_chosen,
                            offset: decision.offset as u16,
                        })
                    })
                    .collect::<Result<Vec<_>>>()
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if !rows.iter().any(|row| row.remaining == 0) || !rows.iter().any(|row| row.remaining != 0) {
        return Err(invalid(
            "content continuation rows need both horizon classes",
        ));
    }
    Ok(rows)
}

pub fn shuffle_content_continuation_rows(
    rows: &[ContentContinuationRow],
    seed: u64,
) -> Result<Vec<ContentContinuationRow>> {
    if rows.is_empty() {
        return Err(invalid("content continuation shuffle needs rows"));
    }
    let mut groups = BTreeMap::<u16, Vec<usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        groups.entry(row.offset).or_default().push(index);
    }
    let mut shuffled = rows.to_vec();
    for indices in groups.values_mut() {
        if indices.len() < 2 {
            continue;
        }
        indices.sort_by(|left, right| {
            rows[*left]
                .signature
                .cmp(&rows[*right].signature)
                .then_with(|| rows[*left].remaining.cmp(&rows[*right].remaining))
        });
        let values = indices
            .iter()
            .map(|index| rows[*index].remaining)
            .collect::<Vec<_>>();
        let shift = 1 + seed as usize % (values.len() - 1);
        let rotated = values
            .iter()
            .cycle()
            .skip(values.len() - shift % values.len())
            .take(values.len())
            .copied()
            .collect::<Vec<_>>();
        for (index, remaining) in indices.iter().zip(rotated) {
            shuffled[*index].remaining = remaining;
        }
    }
    if shuffled
        .iter()
        .zip(rows)
        .all(|(left, right)| left.remaining == right.remaining)
    {
        return Err(invalid("content continuation shuffle is vacuous"));
    }
    Ok(shuffled)
}

pub fn fit_content_continuation_gate(
    rows: &[ContentContinuationRow],
    max_neighbors: usize,
) -> Result<ContentContinuationTraining> {
    if max_neighbors == 0 || max_neighbors > 16 {
        return Err(invalid("content continuation neighbors are out of bounds"));
    }
    if rows.is_empty()
        || !rows.iter().any(|row| row.remaining == 0)
        || !rows.iter().any(|row| row.remaining != 0)
    {
        return Err(invalid(
            "content continuation fitting needs both horizon classes",
        ));
    }
    if rows
        .iter()
        .any(|row| row.signature.len() != CONTENT_CODE_SIGNATURE_BITS / 64)
    {
        return Err(invalid("content continuation signature width differs"));
    }
    let nonterminal_rows = rows.iter().filter(|row| row.remaining != 0).count() as u64;
    let nonterminal_stop_limit = nonterminal_rows / 100;
    let mut candidates = Vec::new();
    for neighbors in [1usize, 2, 4, 8, 16] {
        if neighbors > max_neighbors {
            continue;
        }
        let predictions = rows
            .par_iter()
            .enumerate()
            .map(|(index, row)| {
                continuation_prediction(rows, Some(index), &row.signature, row.offset, neighbors)
            })
            .collect::<Result<Vec<_>>>()?;
        let terminal_stops = rows
            .iter()
            .zip(&predictions)
            .filter(|(row, prediction)| row.remaining == 0 && **prediction == Some(0))
            .count() as u64;
        let nonterminal_stops = rows
            .iter()
            .zip(&predictions)
            .filter(|(row, prediction)| row.remaining != 0 && **prediction == Some(0))
            .count() as u64;
        let horizon_abs_error =
            rows.iter()
                .zip(&predictions)
                .try_fold(0u64, |sum, (row, prediction)| {
                    let error = prediction
                        .map(|prediction| i32::from(prediction) - i32::from(row.remaining))
                        .unwrap_or(i32::from(row.remaining))
                        .unsigned_abs();
                    sum.checked_add(u64::from(error))
                        .ok_or_else(|| invalid("content continuation error sum overflows"))
                })?;
        candidates.push(ContentContinuationCandidate {
            neighbors,
            terminal_stops,
            nonterminal_stops,
            horizon_abs_error,
        });
    }
    let best = candidates
        .iter()
        .max_by(|left, right| {
            left.terminal_stops
                .cmp(&right.terminal_stops)
                .then_with(|| right.nonterminal_stops.cmp(&left.nonterminal_stops))
                .then_with(|| right.horizon_abs_error.cmp(&left.horizon_abs_error))
                .then_with(|| right.neighbors.cmp(&left.neighbors))
        })
        .cloned()
        .ok_or_else(|| invalid("content continuation has no candidate"))?;
    let mut points = rows
        .iter()
        .map(|row| ContentContinuationPoint {
            signature: row.signature.clone(),
            remaining: row.remaining,
            offset: row.offset,
        })
        .collect::<Vec<_>>();
    points.sort_by(|left, right| {
        left.offset
            .cmp(&right.offset)
            .then_with(|| left.signature.cmp(&right.signature))
            .then_with(|| left.remaining.cmp(&right.remaining))
    });
    Ok(ContentContinuationTraining {
        gate: ContentContinuationGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            rows: points,
            neighbors: best.neighbors,
        },
        rows: rows.len() as u64,
        terminal_rows: rows.iter().filter(|row| row.remaining == 0).count() as u64,
        nonterminal_rows,
        terminal_stops: best.terminal_stops,
        nonterminal_stops: best.nonterminal_stops,
        nonterminal_stop_limit,
        horizon_abs_error: best.horizon_abs_error,
        candidates,
    })
}

fn continuation_prediction(
    rows: &[ContentContinuationRow],
    skip: Option<usize>,
    signature: &ContentSignature,
    offset: u16,
    neighbors: usize,
) -> Result<Option<i16>> {
    let mut candidates = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| Some(*index) != skip && row.offset == offset)
        .map(|(_, row)| (content_hamming(signature, &row.signature), row))
        .collect::<Vec<_>>();
    if candidates.len() < neighbors {
        return Ok(None);
    }
    candidates.sort_by(|(left_distance, left), (right_distance, right)| {
        left_distance
            .cmp(right_distance)
            .then_with(|| left.signature.cmp(&right.signature))
            .then_with(|| left.remaining.cmp(&right.remaining))
    });
    let mut horizons = candidates
        .into_iter()
        .take(neighbors)
        .map(|(_, row)| row.remaining)
        .collect::<Vec<_>>();
    horizons.sort_unstable();
    Ok(horizons.get((neighbors - 1) / 2).copied())
}

pub fn content_continuation_loo_prediction(
    rows: &[ContentContinuationRow],
    index: usize,
    neighbors: usize,
) -> Result<Option<i16>> {
    let row = rows
        .get(index)
        .ok_or_else(|| invalid("content continuation query row is out of bounds"))?;
    if neighbors == 0 || neighbors > 16 {
        return Err(invalid("content continuation neighbors are out of bounds"));
    }
    if rows
        .iter()
        .any(|row| row.signature.len() != CONTENT_CODE_SIGNATURE_BITS / 64)
    {
        return Err(invalid("content continuation signature width differs"));
    }
    continuation_prediction(rows, Some(index), &row.signature, row.offset, neighbors)
}

pub fn content_continuation_gate_prediction(
    gate: &ContentContinuationGate,
    signature: &ContentSignature,
    offset: u16,
) -> Result<Option<i16>> {
    validate_content_continuation_gate(gate)?;
    if signature.len() != gate.signature_bits / 64 {
        return Err(invalid("content continuation query width differs"));
    }
    let mut candidates = gate
        .rows
        .iter()
        .filter(|row| row.offset == offset)
        .map(|row| (content_hamming(signature, &row.signature), row))
        .collect::<Vec<_>>();
    if candidates.len() < gate.neighbors {
        return Ok(None);
    }
    candidates.sort_by(|(left_distance, left), (right_distance, right)| {
        left_distance
            .cmp(right_distance)
            .then_with(|| left.signature.cmp(&right.signature))
            .then_with(|| left.remaining.cmp(&right.remaining))
    });
    let mut horizons = candidates
        .into_iter()
        .take(gate.neighbors)
        .map(|(_, row)| row.remaining)
        .collect::<Vec<_>>();
    horizons.sort_unstable();
    Ok(Some(horizons[(gate.neighbors - 1) / 2]))
}

pub fn content_continuation_gate_fires(
    gate: &ContentContinuationGate,
    signature: &ContentSignature,
    offset: u16,
) -> Result<bool> {
    Ok(content_continuation_gate_prediction(gate, signature, offset)? == Some(0))
}

fn validate_content_continuation_gate(gate: &ContentContinuationGate) -> Result<()> {
    if gate.signature_bits != CONTENT_CODE_SIGNATURE_BITS
        || gate.rows.is_empty()
        || gate.neighbors == 0
        || gate.neighbors > 16
    {
        return Err(invalid("content continuation gate shape differs"));
    }
    if gate
        .rows
        .iter()
        .any(|row| row.signature.len() != gate.signature_bits / 64)
    {
        return Err(invalid("content continuation point width differs"));
    }
    Ok(())
}

pub fn shuffle_content_stop_rows(
    rows: &[ContentStopRow],
    seed: u64,
) -> Result<Vec<ContentStopRow>> {
    if rows.is_empty() {
        return Err(invalid("content stop shuffle needs rows"));
    }
    let terminal = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.terminal.then_some(index))
        .collect::<Vec<_>>();
    if terminal.is_empty() || terminal.len() == rows.len() {
        return Err(invalid("content stop shuffle needs both classes"));
    }
    let shift = 1 + seed as usize % (rows.len() - 1);
    let mut shuffled = rows.to_vec();
    for row in &mut shuffled {
        row.terminal = false;
    }
    for index in terminal {
        shuffled[(index + shift) % rows.len()].terminal = true;
    }
    if shuffled
        .iter()
        .zip(rows)
        .all(|(left, right)| left.terminal == right.terminal)
    {
        return Err(invalid("content stop label shuffle is vacuous"));
    }
    Ok(shuffled)
}

pub fn fit_content_stop_gate(
    rows: &[ContentStopRow],
    prototypes_per_class: usize,
) -> Result<ContentStopTraining> {
    if prototypes_per_class == 0 {
        return Err(invalid("content stop prototypes must be positive"));
    }
    let terminal = rows.iter().filter(|row| row.terminal).collect::<Vec<_>>();
    let nonterminal = rows.iter().filter(|row| !row.terminal).collect::<Vec<_>>();
    if terminal.is_empty() || nonterminal.is_empty() {
        return Err(invalid("content stop fitting needs both classes"));
    }
    let terminal_prototypes = choose_content_prototypes(
        terminal.iter().map(|row| row.signature.clone()),
        prototypes_per_class,
    )?;
    let nonterminal_prototypes = choose_content_prototypes(
        nonterminal.iter().map(|row| row.signature.clone()),
        prototypes_per_class,
    )?;
    let deltas = rows
        .iter()
        .map(|row| {
            content_stop_score(
                &row.signature,
                &terminal_prototypes,
                &nonterminal_prototypes,
            )
        })
        .collect::<Vec<_>>();
    let nonterminal_stop_limit = nonterminal.len() as u64 / 100;
    let mut thresholds = deltas.clone();
    thresholds.sort_unstable();
    thresholds.dedup();
    let mut best = None;
    for margin in thresholds {
        let terminal_stops = rows
            .iter()
            .zip(&deltas)
            .filter(|(row, delta)| row.terminal && **delta >= margin)
            .count() as u64;
        let nonterminal_stops = rows
            .iter()
            .zip(&deltas)
            .filter(|(row, delta)| !row.terminal && **delta >= margin)
            .count() as u64;
        if terminal_stops == 0 || nonterminal_stops > nonterminal_stop_limit {
            continue;
        }
        if best.is_none_or(|(best_terminal, best_nonterminal, best_margin)| {
            terminal_stops > best_terminal
                || (terminal_stops == best_terminal
                    && (nonterminal_stops < best_nonterminal
                        || (nonterminal_stops == best_nonterminal && margin > best_margin)))
        }) {
            best = Some((terminal_stops, nonterminal_stops, margin));
        }
    }
    let (terminal_stops, nonterminal_stops, margin) =
        best.ok_or_else(|| invalid("content stop found no margin under the false-stop limit"))?;
    Ok(ContentStopTraining {
        gate: ContentStopGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            terminal_prototypes,
            nonterminal_prototypes,
            margin,
        },
        terminal_rows: terminal.len() as u64,
        nonterminal_rows: nonterminal.len() as u64,
        terminal_stops,
        nonterminal_stops,
        nonterminal_stop_limit,
    })
}

pub fn fit_content_stop_density_gate(
    rows: &[ContentStopRow],
    max_neighbors: usize,
    offset_radius: u16,
) -> Result<ContentStopDensityTraining> {
    if max_neighbors == 0 || max_neighbors > 16 {
        return Err(invalid("content stop density neighbors are out of bounds"));
    }
    let terminal = rows.iter().filter(|row| row.terminal).count();
    let nonterminal = rows.len() - terminal;
    if terminal == 0 || nonterminal == 0 {
        return Err(invalid("content stop density fitting needs both classes"));
    }
    let nonterminal_stop_limit = nonterminal as u64 / 100;
    let mut best: Option<(u64, u64, i64, usize)> = None;
    for neighbors in [1usize, 2, 4, 8, 16] {
        if neighbors > max_neighbors || neighbors > terminal || neighbors > nonterminal {
            continue;
        }
        let deltas = rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                content_stop_density_score(
                    rows,
                    Some(index),
                    row.offset,
                    &row.signature,
                    neighbors,
                    offset_radius,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let mut thresholds = deltas.clone();
        thresholds.sort_unstable();
        thresholds.dedup();
        for margin in thresholds {
            let terminal_stops = rows
                .iter()
                .zip(&deltas)
                .filter(|(row, delta)| row.terminal && **delta >= margin)
                .count() as u64;
            let nonterminal_stops = rows
                .iter()
                .zip(&deltas)
                .filter(|(row, delta)| !row.terminal && **delta >= margin)
                .count() as u64;
            if terminal_stops == 0 || nonterminal_stops > nonterminal_stop_limit {
                continue;
            }
            if best.is_none_or(
                |(best_terminal, best_nonterminal, best_margin, best_neighbors)| {
                    terminal_stops > best_terminal
                        || (terminal_stops == best_terminal
                            && (nonterminal_stops < best_nonterminal
                                || (nonterminal_stops == best_nonterminal
                                    && (margin > best_margin
                                        || (margin == best_margin && neighbors < best_neighbors)))))
                },
            ) {
                best = Some((terminal_stops, nonterminal_stops, margin, neighbors));
            }
        }
    }
    let (terminal_stops, nonterminal_stops, margin, neighbors) = best.ok_or_else(|| {
        invalid("content stop density found no neighbor/margin under the false-stop limit")
    })?;
    let points = |terminal_class| {
        let mut points = rows
            .iter()
            .filter(|row| row.terminal == terminal_class)
            .map(|row| ContentStopDensityPoint {
                signature: row.signature.clone(),
                offset: row.offset,
            })
            .collect::<Vec<_>>();
        points.sort_by(|left, right| {
            left.offset
                .cmp(&right.offset)
                .then_with(|| left.signature.cmp(&right.signature))
        });
        points
    };
    Ok(ContentStopDensityTraining {
        gate: ContentStopDensityGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            terminal_rows: points(true),
            nonterminal_rows: points(false),
            neighbors,
            offset_radius,
            margin,
        },
        terminal_rows: terminal as u64,
        nonterminal_rows: nonterminal as u64,
        terminal_stops,
        nonterminal_stops,
        nonterminal_stop_limit,
        leave_one_out_neighbors: neighbors,
    })
}

pub fn evaluate_content_stop_density_gate(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentStopDensityGate,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    validate_content_readout(readout)?;
    validate_content_stop_density_gate(gate)?;
    evaluate_generated(samples, cap, |sample| {
        generate_with_content_density_gate(artifact, sample, cap, memory_enabled, readout, gate)
    })
}

pub fn evaluate_content_continuation_gate(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentContinuationGate,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    validate_content_readout(readout)?;
    validate_content_continuation_gate(gate)?;
    evaluate_generated(samples, cap, |sample| {
        generate_with_content_continuation_gate(
            artifact,
            sample,
            cap,
            memory_enabled,
            readout,
            gate,
        )
    })
}

pub fn evaluate_content_stop_gate(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentStopGate,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    validate_content_readout(readout)?;
    validate_content_stop_gate(gate)?;
    evaluate_generated(samples, cap, |sample| {
        generate_with_content_gate(artifact, sample, cap, memory_enabled, readout, gate)
    })
}

fn validate_content_stop_gate(gate: &ContentStopGate) -> Result<()> {
    if gate.signature_bits != CONTENT_CODE_SIGNATURE_BITS
        || gate.terminal_prototypes.is_empty()
        || gate.nonterminal_prototypes.is_empty()
    {
        return Err(invalid("content stop gate shape differs"));
    }
    if gate
        .terminal_prototypes
        .iter()
        .chain(&gate.nonterminal_prototypes)
        .any(|signature| signature.len() != gate.signature_bits / 64)
    {
        return Err(invalid("content stop prototype width differs"));
    }
    Ok(())
}

fn choose_content_prototypes(
    signatures: impl Iterator<Item = ContentSignature>,
    count: usize,
) -> Result<Vec<ContentSignature>> {
    let mut signatures = signatures.collect::<Vec<_>>();
    signatures.sort_unstable();
    signatures.dedup();
    if signatures.is_empty() {
        return Err(invalid("content stop prototype class is empty"));
    }
    let mut selected = vec![content_stop_medoid(&signatures)?];
    while selected.len() < count.min(signatures.len()) {
        let mut best: Option<(u32, ContentSignature)> = None;
        for candidate in &signatures {
            if selected.contains(candidate) {
                continue;
            }
            let nearest = selected
                .iter()
                .map(|point| content_hamming(candidate, point))
                .min()
                .ok_or_else(|| invalid("content stop prototype selection lost its class"))?;
            if best.as_ref().is_none_or(|(distance, point)| {
                nearest > *distance || (nearest == *distance && candidate < point)
            }) {
                best = Some((nearest, candidate.clone()));
            }
        }
        let (_, point) =
            best.ok_or_else(|| invalid("content stop prototype residual group is empty"))?;
        selected.push(point);
    }
    Ok(selected)
}

fn content_stop_medoid(signatures: &[ContentSignature]) -> Result<ContentSignature> {
    let mut best: Option<(u64, ContentSignature)> = None;
    for candidate in signatures {
        let total = signatures.iter().try_fold(0u64, |sum, signature| {
            sum.checked_add(u64::from(content_hamming(candidate, signature)))
                .ok_or_else(|| invalid("content stop medoid distance sum overflows"))
        })?;
        if best.as_ref().is_none_or(|(distance, point)| {
            total < *distance || (total == *distance && candidate < point)
        }) {
            best = Some((total, candidate.clone()));
        }
    }
    Ok(best
        .ok_or_else(|| invalid("content stop medoid class is empty"))?
        .1)
}

fn content_stop_score(
    signature: &ContentSignature,
    terminal: &[ContentSignature],
    nonterminal: &[ContentSignature],
) -> i64 {
    let nearest = |points: &[ContentSignature]| {
        points
            .iter()
            .map(|point| content_hamming(signature, point))
            .min()
            .unwrap_or(u32::MAX)
    };
    i64::from(nearest(nonterminal)) - i64::from(nearest(terminal))
}

fn content_stop_fires(gate: &ContentStopGate, signature: &ContentSignature) -> bool {
    content_stop_score(
        signature,
        &gate.terminal_prototypes,
        &gate.nonterminal_prototypes,
    ) >= gate.margin
}

pub fn content_stop_gate_fires(
    gate: &ContentStopGate,
    signature: &ContentSignature,
) -> Result<bool> {
    validate_content_stop_gate(gate)?;
    if signature.len() != gate.signature_bits / 64 {
        return Err(invalid("content stop query width differs"));
    }
    Ok(content_stop_fires(gate, signature))
}

fn validate_content_stop_density_gate(gate: &ContentStopDensityGate) -> Result<()> {
    if gate.signature_bits != CONTENT_CODE_SIGNATURE_BITS
        || gate.terminal_rows.is_empty()
        || gate.nonterminal_rows.is_empty()
        || gate.neighbors == 0
        || gate.neighbors > 16
    {
        return Err(invalid("content stop density gate shape differs"));
    }
    if gate
        .terminal_rows
        .iter()
        .chain(&gate.nonterminal_rows)
        .any(|point| point.signature.len() != gate.signature_bits / 64)
    {
        return Err(invalid("content stop density point width differs"));
    }
    Ok(())
}

fn content_stop_density_distance_sum(
    rows: &[ContentStopRow],
    skip: Option<usize>,
    terminal: bool,
    signature: &ContentSignature,
    offset: u16,
    neighbors: usize,
    offset_radius: u16,
) -> u64 {
    let eligible = |index: usize, row: &ContentStopRow| {
        Some(index) != skip
            && row.terminal == terminal
            && row.offset.abs_diff(offset) <= offset_radius
    };
    let mut distances = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| eligible(*index, row))
        .map(|(_, row)| content_hamming(signature, &row.signature))
        .collect::<Vec<_>>();
    let available = distances.len();
    distances.sort_unstable();
    let observed = distances
        .into_iter()
        .take(neighbors)
        .map(u64::from)
        .sum::<u64>();
    let missing = neighbors.saturating_sub(available) as u64;
    observed + missing * CONTENT_CODE_SIGNATURE_BITS as u64
}

fn content_stop_density_score(
    rows: &[ContentStopRow],
    skip: Option<usize>,
    offset: u16,
    signature: &ContentSignature,
    neighbors: usize,
    offset_radius: u16,
) -> Result<i64> {
    let terminal = content_stop_density_distance_sum(
        rows,
        skip,
        true,
        signature,
        offset,
        neighbors,
        offset_radius,
    );
    let nonterminal = content_stop_density_distance_sum(
        rows,
        skip,
        false,
        signature,
        offset,
        neighbors,
        offset_radius,
    );
    i64::try_from(nonterminal)
        .ok()
        .and_then(|nonterminal| {
            i64::try_from(terminal)
                .ok()
                .map(|terminal| nonterminal - terminal)
        })
        .ok_or_else(|| invalid("content stop density sum is out of range"))
}

fn content_stop_density_fires(
    gate: &ContentStopDensityGate,
    signature: &ContentSignature,
    offset: u16,
) -> Result<bool> {
    let terminal = gate
        .terminal_rows
        .iter()
        .filter(|point| point.offset.abs_diff(offset) <= gate.offset_radius)
        .map(|point| content_hamming(signature, &point.signature))
        .collect::<Vec<_>>();
    let nonterminal = gate
        .nonterminal_rows
        .iter()
        .filter(|point| point.offset.abs_diff(offset) <= gate.offset_radius)
        .map(|point| content_hamming(signature, &point.signature))
        .collect::<Vec<_>>();
    let nearest_sum = |mut distances: Vec<u32>| {
        let missing = gate.neighbors.saturating_sub(distances.len()) as u64;
        distances.sort_unstable();
        let observed = distances
            .into_iter()
            .take(gate.neighbors)
            .map(u64::from)
            .sum::<u64>();
        observed + missing * gate.signature_bits as u64
    };
    let terminal = nearest_sum(terminal);
    let nonterminal = nearest_sum(nonterminal);
    Ok(
        i64::try_from(nonterminal).unwrap_or(i64::MAX)
            - i64::try_from(terminal).unwrap_or(i64::MAX)
            >= gate.margin,
    )
}

pub fn content_stop_density_gate_fires(
    gate: &ContentStopDensityGate,
    signature: &ContentSignature,
    offset: u16,
) -> Result<bool> {
    validate_content_stop_density_gate(gate)?;
    if signature.len() != gate.signature_bits / 64 {
        return Err(invalid("content stop density query width differs"));
    }
    content_stop_density_fires(gate, signature, offset)
}

fn generate_with_content_gate(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentStopGate,
) -> Result<(Vec<Decision>, Generated)> {
    if cap == 0 || cap > 64 || sample.prompt.is_empty() {
        return Err(invalid("content stop sample, prompt or cap differs"));
    }
    let reference = normalized_reference(&sample.reply);
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("content stop forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    let mut current = None;
    for (index, &token) in sample.prompt.iter().enumerate() {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content stop prompt step: {error}")))?;
        if index + 1 == sample.prompt.len() {
            current = Some(step);
        }
    }
    let mut current = current.ok_or_else(|| invalid("content stop prompt is empty"))?;
    let mut decisions = Vec::new();
    let mut tokens = Vec::new();
    let stop = loop {
        let offset = decisions.len();
        let signature = pack_content_signature(&current.query_q12, ContentSignatureMode::Code)?;
        let (content_chosen, nearest_distance, ambiguous_nearest, used_readout) =
            select_content_target(readout, &signature, current.token)?;
        let gate_fired = content_stop_fires(gate, &signature);
        let chosen = if gate_fired {
            POINT_KEY_BOS
        } else {
            content_chosen
        };
        decisions.push(Decision {
            offset,
            chosen,
            content_chosen,
            writer_chosen: current.token,
            gate_fired,
            signature,
            reference_target: reference.get(offset).copied(),
            nearest_distance,
            ambiguous_nearest,
            used_readout,
        });
        let end = reply_stop(chosen, offset, sample.prompt.len(), cap, context);
        if end != Some(ReplyEnd::Bos) {
            tokens.push(chosen);
        }
        if let Some(end) = end {
            break match end {
                ReplyEnd::Bos => "bos",
                ReplyEnd::Length => "cap",
                ReplyEnd::Context => "context",
            };
        }
        current = forward
            .step(chosen, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content stop generated step: {error}")))?;
    };
    Ok((decisions, Generated { tokens, stop }))
}

fn generate_with_content_density_gate(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentStopDensityGate,
) -> Result<(Vec<Decision>, Generated)> {
    if cap == 0 || cap > 64 || sample.prompt.is_empty() {
        return Err(invalid(
            "content stop density sample, prompt or cap differs",
        ));
    }
    let reference = normalized_reference(&sample.reply);
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("content stop density forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    let mut current = None;
    for (index, &token) in sample.prompt.iter().enumerate() {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content stop density prompt step: {error}")))?;
        if index + 1 == sample.prompt.len() {
            current = Some(step);
        }
    }
    let mut current = current.ok_or_else(|| invalid("content stop density prompt is empty"))?;
    let mut decisions = Vec::new();
    let mut tokens = Vec::new();
    let stop = loop {
        let offset = decisions.len();
        let signature = pack_content_signature(&current.query_q12, ContentSignatureMode::Code)?;
        let (content_chosen, nearest_distance, ambiguous_nearest, used_readout) =
            select_content_target(readout, &signature, current.token)?;
        let gate_fired = content_stop_density_fires(
            gate,
            &signature,
            offset.try_into().map_err(|_| {
                invalid("content stop density offset exceeds the row representation")
            })?,
        )?;
        let chosen = if gate_fired {
            POINT_KEY_BOS
        } else {
            content_chosen
        };
        decisions.push(Decision {
            offset,
            chosen,
            content_chosen,
            writer_chosen: current.token,
            gate_fired,
            signature,
            reference_target: reference.get(offset).copied(),
            nearest_distance,
            ambiguous_nearest,
            used_readout,
        });
        let end = reply_stop(chosen, offset, sample.prompt.len(), cap, context);
        if end != Some(ReplyEnd::Bos) {
            tokens.push(chosen);
        }
        if let Some(end) = end {
            break match end {
                ReplyEnd::Bos => "bos",
                ReplyEnd::Length => "cap",
                ReplyEnd::Context => "context",
            };
        }
        current = forward
            .step(chosen, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content stop density generated step: {error}")))?;
    };
    Ok((decisions, Generated { tokens, stop }))
}

fn generate_with_content_continuation_gate(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    memory_enabled: bool,
    readout: &ContentReadout,
    gate: &ContentContinuationGate,
) -> Result<(Vec<Decision>, Generated)> {
    if cap == 0 || cap > 64 || sample.prompt.is_empty() {
        return Err(invalid(
            "content continuation sample, prompt or cap differs",
        ));
    }
    let reference = normalized_reference(&sample.reply);
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("content continuation forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    let mut current = None;
    for (index, &token) in sample.prompt.iter().enumerate() {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content continuation prompt step: {error}")))?;
        if index + 1 == sample.prompt.len() {
            current = Some(step);
        }
    }
    let mut current = current.ok_or_else(|| invalid("content continuation prompt is empty"))?;
    let mut decisions = Vec::new();
    let mut tokens = Vec::new();
    let stop = loop {
        let offset = decisions.len();
        let signature = pack_content_signature(&current.query_q12, ContentSignatureMode::Code)?;
        let (content_chosen, nearest_distance, ambiguous_nearest, used_readout) =
            select_content_target(readout, &signature, current.token)?;
        let gate_fired = content_continuation_gate_fires(
            gate,
            &signature,
            offset.try_into().map_err(|_| {
                invalid("content continuation offset exceeds the row representation")
            })?,
        )?;
        let chosen = if gate_fired {
            POINT_KEY_BOS
        } else {
            content_chosen
        };
        decisions.push(Decision {
            offset,
            chosen,
            content_chosen,
            writer_chosen: current.token,
            gate_fired,
            signature,
            reference_target: reference.get(offset).copied(),
            nearest_distance,
            ambiguous_nearest,
            used_readout,
        });
        let end = reply_stop(chosen, offset, sample.prompt.len(), cap, context);
        if end != Some(ReplyEnd::Bos) {
            tokens.push(chosen);
        }
        if let Some(end) = end {
            break match end {
                ReplyEnd::Bos => "bos",
                ReplyEnd::Length => "cap",
                ReplyEnd::Context => "context",
            };
        }
        current = forward
            .step(chosen, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("content continuation generated step: {error}")))?;
    };
    Ok((decisions, Generated { tokens, stop }))
}

pub fn verify_content_readout_serving(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    readout: &ContentReadout,
    sidecar: &[u8],
) -> Result<ContentServingParity> {
    validate_content_readout(readout)?;
    if readout.mode != ContentSignatureMode::Code || readout.includes_reference_end {
        return Err(invalid("serving parity needs the plain code readout"));
    }
    if samples.is_empty() {
        return Err(invalid("serving parity needs samples"));
    }
    let mut model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("serving parity model: {error}")))?;
    let serving_readout =
        ServingContentReadout::from_sidecar(sidecar, model.artifact_sha256(), model.shape().vocab)
            .map_err(|error| invalid(format!("serving parity sidecar: {error}")))?;
    model
        .set_content_readout(serving_readout.clone())
        .map_err(|error| invalid(format!("serving parity attach: {error}")))?;

    #[derive(serde::Serialize)]
    struct ParityRow {
        sample: usize,
        offset: usize,
        chosen: u32,
        content_chosen: u32,
        signature: ContentSignature,
        nearest_distance: Option<u32>,
        used_readout: bool,
        stop: String,
        tokens: Vec<u32>,
    }

    let mut outcome = ContentServingParity {
        prompts: samples.len() as u64,
        ..ContentServingParity::default()
    };
    let mut cpu_rows = Vec::new();
    let mut serving_rows = Vec::new();
    for (sample_index, sample) in samples.iter().enumerate() {
        let (cpu_decisions, cpu_generated) = generate_with_readout(
            artifact,
            sample,
            cap,
            true,
            ContentSignatureMode::Code,
            false,
            Some(readout),
        )?;
        let mut session = model.session();
        let mut current = None;
        for (index, &token) in sample.prompt.iter().enumerate() {
            if index + 1 == sample.prompt.len() {
                session.mark_reply_start();
            }
            current = Some(
                session
                    .serve_step(token)
                    .map_err(|error| invalid(format!("serving parity prompt: {error}")))?
                    .0,
            );
        }
        let mut current = current.ok_or_else(|| invalid("serving parity prompt is empty"))?;
        let mut serving_decisions = Vec::new();
        let mut serving_tokens = Vec::new();
        let stop = loop {
            let offset = serving_decisions.len();
            let query = session
                .last_query_q12()
                .ok_or_else(|| invalid("serving parity lost the FBS query"))?;
            let fallback = session
                .last_native_token()
                .ok_or_else(|| invalid("serving parity lost the native token"))?;
            let selected = serving_readout
                .select(query, fallback)
                .map_err(|error| invalid(format!("serving parity select: {error}")))?;
            if selected.token != current {
                return Err(invalid("serving readout and serve_step disagree"));
            }
            let chosen = current;
            serving_decisions.push(Decision {
                offset,
                chosen,
                content_chosen: fallback,
                writer_chosen: fallback,
                gate_fired: false,
                signature: pack_content_signature(query, ContentSignatureMode::Code)?,
                reference_target: None,
                nearest_distance: Some(selected.nearest_distance),
                ambiguous_nearest: false,
                used_readout: selected.readout_fired,
            });
            let end = reply_stop(
                chosen,
                offset,
                sample.prompt.len(),
                cap,
                model.shape().context,
            );
            if end != Some(ReplyEnd::Bos) {
                serving_tokens.push(chosen);
            }
            if let Some(end) = end {
                break match end {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                }
                .to_owned();
            }
            current = session
                .serve_step(chosen)
                .map_err(|error| invalid(format!("serving parity generated step: {error}")))?
                .0;
        };

        if cpu_generated.tokens != serving_tokens {
            outcome.token_mismatches += 1;
            outcome.first_mismatch.get_or_insert_with(|| {
                format!("sample {sample_index}: complete reply tokens differ")
            });
        }
        if cpu_generated.stop != stop {
            outcome.decision_mismatches += 1;
            outcome.first_mismatch.get_or_insert_with(|| {
                format!(
                    "sample {sample_index}: stop {} differs from {stop}",
                    cpu_generated.stop
                )
            });
        }
        let width = cpu_decisions.len().max(serving_decisions.len());
        for index in 0..width {
            let (Some(cpu), Some(served)) =
                (cpu_decisions.get(index), serving_decisions.get(index))
            else {
                outcome.decision_mismatches += 1;
                outcome.first_mismatch.get_or_insert_with(|| {
                    format!("sample {sample_index}: decision count differs")
                });
                break;
            };
            outcome.decisions += 1;
            outcome.readout_fires += u64::from(served.used_readout);
            outcome.native_fallbacks += u64::from(!served.used_readout);
            if cpu.chosen != served.chosen
                || cpu.writer_chosen != served.content_chosen
                || cpu.signature != served.signature
                || cpu.nearest_distance != served.nearest_distance
                || cpu.used_readout != served.used_readout
            {
                outcome.decision_mismatches += 1;
                outcome.first_mismatch.get_or_insert_with(|| {
                    format!(
                        "sample {sample_index}: decision {index} differs \
                         (cpu chosen/fallback/distance/used {}/{}/{:?}/{}, \
                         serving {}/{}/{:?}/{})",
                        cpu.chosen,
                        cpu.writer_chosen,
                        cpu.nearest_distance,
                        cpu.used_readout,
                        served.chosen,
                        served.content_chosen,
                        served.nearest_distance,
                        served.used_readout
                    )
                });
            }
            cpu_rows.push(ParityRow {
                sample: sample_index,
                offset: cpu.offset,
                chosen: cpu.chosen,
                content_chosen: cpu.writer_chosen,
                signature: cpu.signature.clone(),
                nearest_distance: cpu.nearest_distance,
                used_readout: cpu.used_readout,
                stop: cpu_generated.stop.to_owned(),
                tokens: cpu_generated.tokens.clone(),
            });
            serving_rows.push(ParityRow {
                sample: sample_index,
                offset: served.offset,
                chosen: served.chosen,
                content_chosen: served.content_chosen,
                signature: served.signature.clone(),
                nearest_distance: served.nearest_distance,
                used_readout: served.used_readout,
                stop: stop.clone(),
                tokens: serving_tokens.clone(),
            });
        }
    }
    outcome.cpu_decision_sha256 = uor_r4_discrete::corpus::sha256(
        &serde_json::to_vec(&cpu_rows)
            .map_err(|error| invalid(format!("CPU serving-parity rows: {error}")))?,
    );
    outcome.serving_decision_sha256 = uor_r4_discrete::corpus::sha256(
        &serde_json::to_vec(&serving_rows)
            .map_err(|error| invalid(format!("serving-parity rows: {error}")))?,
    );
    if outcome.cpu_decision_sha256 != outcome.serving_decision_sha256 {
        outcome.decision_mismatches = outcome.decision_mismatches.max(1);
        outcome
            .first_mismatch
            .get_or_insert_with(|| "decision digest differs".to_owned());
    }
    Ok(outcome)
}

fn has_repeated_4gram(tokens: &[u32]) -> bool {
    let mut seen = BTreeSet::new();
    tokens
        .windows(4)
        .any(|window| !seen.insert(window.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn signature(bit: usize) -> ContentSignature {
        let mut signature = vec![0u64; CONTENT_SIGNATURE_BITS / 64];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn code_signature(bit: usize) -> ContentSignature {
        let mut signature = vec![0u64; CONTENT_CODE_SIGNATURE_BITS / 64];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn code_signature_bits(bits: impl IntoIterator<Item = usize>) -> ContentSignature {
        let mut signature = vec![0u64; CONTENT_CODE_SIGNATURE_BITS / 64];
        for bit in bits {
            signature[bit / 64] |= 1u64 << (bit % 64);
        }
        signature
    }

    fn row(bit: usize, target: u32) -> ContentExample {
        ContentExample {
            signature: if bit == CONTENT_SIGNATURE_BITS {
                vec![0; CONTENT_SIGNATURE_BITS / 64]
            } else {
                signature(bit)
            },
            target,
        }
    }

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            1361,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: CONTENT_SIGNATURE_BITS,
                mlp: 64,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        snapshot.memory = None;
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn sample() -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 1],
        }
    }

    #[test]
    fn packing_and_xor_distance_use_coordinate_order() {
        let query = (0..CONTENT_SIGNATURE_BITS)
            .map(|index| if index % 3 == 0 { -1 } else { 0 })
            .collect::<Vec<_>>();
        let packed = pack_content_signature(&query, ContentSignatureMode::Sign).unwrap();
        assert_eq!(
            content_hamming(&packed, &vec![0; CONTENT_SIGNATURE_BITS / 64]),
            (0..CONTENT_SIGNATURE_BITS)
                .filter(|index| index % 3 != 0)
                .count() as u32
        );
        let code = pack_content_signature(&query, ContentSignatureMode::Code).unwrap();
        assert_eq!(code.len(), CONTENT_CODE_SIGNATURE_BITS / 64);
        assert_eq!(
            content_hamming(&code, &vec![0; CONTENT_CODE_SIGNATURE_BITS / 64]),
            (0..CONTENT_SIGNATURE_BITS)
                .filter(|index| index % 3 == 0)
                .count() as u32
                * 8
        );
        assert!(pack_content_signature(&query[..255], ContentSignatureMode::Sign).is_err());
    }

    #[test]
    fn trajectory_packing_places_current_code_then_transition() {
        let query = (0..CONTENT_SIGNATURE_BITS)
            .map(|index| index as i32 - 128)
            .collect::<Vec<_>>();
        let previous_query = query
            .iter()
            .map(|value| value.wrapping_mul(3).wrapping_add(7))
            .collect::<Vec<_>>();
        let current = pack_content_signature(&query, ContentSignatureMode::Code).unwrap();
        let previous = pack_content_signature(&previous_query, ContentSignatureMode::Code).unwrap();
        let trajectory = pack_content_trajectory_signature(&query, Some(&previous)).unwrap();
        assert_eq!(trajectory.len(), CONTENT_TRAJECTORY_SIGNATURE_BITS / 64);
        assert_eq!(&trajectory[..current.len()], current.as_slice());
        assert_eq!(
            &trajectory[current.len()..],
            current
                .iter()
                .zip(&previous)
                .map(|(current, previous)| current ^ previous)
                .collect::<Vec<_>>()
                .as_slice()
        );
        let first = pack_content_trajectory_signature(&query, None).unwrap();
        assert_eq!(&first[..current.len()], current.as_slice());
        assert!(first[current.len()..].iter().all(|word| *word == 0));
        assert!(pack_content_trajectory_signature(&query, Some(&previous[1..])).is_err());
        assert!(pack_content_signature(&query, ContentSignatureMode::Trajectory).is_err());
    }

    #[test]
    fn nearest_target_counts_only_minimum_distance_and_breaks_ties_by_token() {
        let rows = vec![row(0, 9), row(0, 7), row(1, 3), row(1, 3)];
        let nearest = nearest_target(&rows, rows.len(), &signature(0))
            .unwrap()
            .unwrap();
        assert_eq!((nearest.target, nearest.distance), (7, 0));
        assert!(nearest.ambiguous);
        let nearest = nearest_target(&rows[..2], rows[..2].len(), &signature(1))
            .unwrap()
            .unwrap();
        assert_eq!((nearest.target, nearest.distance), (7, 2));
        assert!(nearest.ambiguous);
    }

    #[test]
    fn fitted_threshold_covers_at_least_ninety_percent_of_leave_one_out_rows() {
        let mut rows = (0..90)
            .map(|index| row(0, index as u32))
            .collect::<Vec<_>>();
        rows.extend((100..110).map(|index| row(index, index as u32)));
        let training = fit_content_readout(rows, 2, ContentSignatureMode::Sign, false).unwrap();
        assert_eq!(training.covered_rows, 90);
        assert_eq!(training.rows, 100);
        assert_eq!(training.readout.max_distance, 0);
        assert!(training.covered_rows * 10 >= training.rows * 9);
        assert!(fit_content_readout(Vec::new(), 0, ContentSignatureMode::Sign, false).is_err());
        let mut code_rows = (0..90)
            .map(|index| ContentExample {
                signature: vec![0; CONTENT_CODE_SIGNATURE_BITS / 64],
                target: index as u32,
            })
            .collect::<Vec<_>>();
        code_rows.extend((0..10).map(|_| ContentExample {
            signature: vec![u64::MAX; CONTENT_CODE_SIGNATURE_BITS / 64],
            target: 100,
        }));
        let code = fit_content_readout(code_rows, 91, ContentSignatureMode::Code, false).unwrap();
        assert_eq!(code.readout.mode, ContentSignatureMode::Code);
        assert_eq!(code.readout.signature_bits, CONTENT_CODE_SIGNATURE_BITS);
        assert!(
            fit_content_readout(vec![row(0, 1)], 1, ContentSignatureMode::Code, false).is_err()
        );
    }

    #[test]
    fn shuffle_is_deterministic_changes_assignments_and_refuses_empty_rows() {
        let readout = ContentReadout {
            mode: ContentSignatureMode::Sign,
            signature_bits: CONTENT_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![row(0, 1), row(1, 2), row(2, 3)],
            max_distance: 0,
        };
        let first = shuffle_content_targets(&readout, 7).unwrap();
        let second = shuffle_content_targets(&readout, 7).unwrap();
        assert_eq!(first, second);
        assert_ne!(
            first.rows.iter().map(|row| row.target).collect::<Vec<_>>(),
            readout
                .rows
                .iter()
                .map(|row| row.target)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first
                .rows
                .iter()
                .map(|row| row.signature.clone())
                .collect::<Vec<_>>(),
            readout
                .rows
                .iter()
                .map(|row| row.signature.clone())
                .collect::<Vec<_>>()
        );
        assert!(shuffle_content_targets(
            &ContentReadout {
                mode: ContentSignatureMode::Sign,
                signature_bits: CONTENT_SIGNATURE_BITS,
                includes_reference_end: false,
                rows: vec![row(0, 1)],
                max_distance: 0,
            },
            1
        )
        .is_err());
        let repeated = ContentReadout {
            mode: ContentSignatureMode::Sign,
            signature_bits: CONTENT_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![row(0, 9), row(0, 7), row(0, 9), row(0, 7)],
            max_distance: 0,
        };
        assert!(shuffle_content_targets(&repeated, 2).is_err());
    }

    #[test]
    fn readout_uses_a_matching_row_and_falls_back_beyond_the_threshold() {
        let readout = ContentReadout {
            mode: ContentSignatureMode::Sign,
            signature_bits: CONTENT_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![row(0, 7)],
            max_distance: 0,
        };
        assert_eq!(
            select_content_target(&readout, &signature(0), 99).unwrap(),
            (7, Some(0), false, true)
        );
        assert_eq!(
            select_content_target(&readout, &signature(1), 99).unwrap(),
            (99, Some(2), false, false)
        );
        assert!(select_content_target(
            &ContentReadout {
                mode: ContentSignatureMode::Sign,
                signature_bits: CONTENT_SIGNATURE_BITS,
                includes_reference_end: false,
                rows: Vec::new(),
                max_distance: 0,
            },
            &signature(0),
            99
        )
        .is_err());
    }

    #[test]
    fn generated_rows_align_to_offsets_and_free_replies_are_deterministic() {
        let artifact = artifact();
        let samples = vec![sample()];
        for mode in [
            ContentSignatureMode::Sign,
            ContentSignatureMode::Code,
            ContentSignatureMode::Trajectory,
        ] {
            let (writer, _) =
                generate_with_readout(&artifact, &samples[0], 4, false, mode, false, None).unwrap();
            let readout = ContentReadout {
                mode,
                signature_bits: mode.bits(),
                includes_reference_end: false,
                rows: vec![ContentExample {
                    signature: writer[0].signature.clone(),
                    target: writer[0]
                        .reference_target
                        .expect("the first generated row has a reference target"),
                }],
                max_distance: 0,
            };
            let (first_decisions, first_reply) = generate_with_readout(
                &artifact,
                &samples[0],
                4,
                false,
                mode,
                false,
                Some(&readout),
            )
            .unwrap();
            let (second_decisions, second_reply) = generate_with_readout(
                &artifact,
                &samples[0],
                4,
                false,
                mode,
                false,
                Some(&readout),
            )
            .unwrap();
            assert_eq!(first_decisions[0].offset, 0);
            assert_eq!(
                first_decisions[0].chosen,
                writer[0].reference_target.unwrap()
            );
            assert!(first_decisions[0].used_readout);
            assert_eq!(first_reply.tokens, second_reply.tokens);
            assert_eq!(first_reply.stop, second_reply.stop);
            assert_eq!(
                first_decisions
                    .iter()
                    .map(|decision| (decision.offset, decision.chosen, decision.used_readout))
                    .collect::<Vec<_>>(),
                second_decisions
                    .iter()
                    .map(|decision| (decision.offset, decision.chosen, decision.used_readout))
                    .collect::<Vec<_>>()
            );
            let evaluate = |threads| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    evaluate_content_readout(&artifact, &samples, 4, false, mode, Some(&readout))
                        .unwrap()
                })
            };
            assert_eq!(evaluate(1), evaluate(32), "{mode:?}");
        }
    }

    #[test]
    fn capped_generation_counts_decisions_past_the_reference_horizon() {
        let sample = sample();
        let (reading, prompts) = evaluate_generated(std::slice::from_ref(&sample), 4, |_| {
            let decisions = (0..4)
                .map(|offset| Decision {
                    offset,
                    chosen: 7 + offset as u32,
                    content_chosen: 7 + offset as u32,
                    writer_chosen: 7 + offset as u32,
                    gate_fired: false,
                    signature: signature(offset),
                    reference_target: (offset < 2).then_some(42),
                    nearest_distance: Some(1),
                    ambiguous_nearest: false,
                    used_readout: true,
                })
                .collect();
            Ok((
                decisions,
                Generated {
                    tokens: vec![7, 8, 9, 10],
                    stop: "cap",
                },
            ))
        })
        .unwrap();
        assert_eq!(prompts[0].stop, "cap");
        assert_eq!(reading.decisions, 2);
        assert_eq!(reading.generated_decisions, 4);
        assert_eq!(reading.readout_decisions, 4);
        assert_eq!(reading.writer_fallbacks, 0);
    }

    #[test]
    fn reference_end_rows_add_bos_and_stop_the_reply() {
        let artifact = artifact();
        let sample = sample();
        let (writer, _) = generate_with_readout(
            &artifact,
            &sample,
            4,
            false,
            ContentSignatureMode::Code,
            true,
            None,
        )
        .unwrap();
        assert_eq!(
            writer
                .iter()
                .map(|decision| decision.reference_target)
                .collect::<Vec<_>>(),
            vec![Some(42), Some(77), Some(0), None]
        );
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: true,
            rows: vec![ContentExample {
                signature: writer[2].signature.clone(),
                target: 0,
            }],
            max_distance: 0,
        };
        let (decisions, reply) = generate_with_readout(
            &artifact,
            &sample,
            4,
            false,
            ContentSignatureMode::Code,
            false,
            Some(&readout),
        )
        .unwrap();
        assert_eq!(reply.stop, "bos");
        assert_eq!(decisions.last().unwrap().chosen, 0);
        assert_eq!(decisions.len(), 3);
    }

    #[test]
    fn continuation_gate_stops_at_zero_horizon_without_changing_ungated_content() {
        let artifact = artifact();
        let sample = sample();
        let (writer, _) = generate_with_readout(
            &artifact,
            &sample,
            4,
            false,
            ContentSignatureMode::Code,
            false,
            None,
        )
        .unwrap();
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![ContentExample {
                signature: writer[0].signature.clone(),
                target: writer[0].reference_target.unwrap(),
            }],
            max_distance: 0,
        };
        let gate = ContentContinuationGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            rows: vec![ContentContinuationPoint {
                signature: writer[1].signature.clone(),
                remaining: 0,
                offset: 1,
            }],
            neighbors: 1,
        };
        let (decisions, reply) =
            generate_with_content_continuation_gate(&artifact, &sample, 4, false, &readout, &gate)
                .unwrap();
        assert_eq!(decisions.len(), 2);
        assert!(!decisions[0].gate_fired);
        assert_eq!(decisions[0].chosen, decisions[0].content_chosen);
        assert!(decisions[1].gate_fired);
        assert_eq!(decisions[1].chosen, POINT_KEY_BOS);
        assert_eq!(reply.stop, "bos");
        assert_eq!(
            decisions
                .iter()
                .filter(|decision| !decision.gate_fired)
                .map(|decision| (decision.chosen, decision.content_chosen))
                .collect::<Vec<_>>(),
            vec![(decisions[0].content_chosen, decisions[0].content_chosen)]
        );
    }

    #[test]
    fn continuation_horizons_are_reference_length_minus_generated_offset() {
        let artifact = artifact();
        let sample = sample();
        let (writer, _) = generate_with_readout(
            &artifact,
            &sample,
            4,
            false,
            ContentSignatureMode::Code,
            false,
            None,
        )
        .unwrap();
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![ContentExample {
                signature: writer[0].signature.clone(),
                target: writer[0].reference_target.unwrap(),
            }],
            max_distance: 0,
        };
        let rows =
            derive_content_continuation_rows(&artifact, &[sample], 4, false, &readout).unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| (row.offset, row.remaining))
                .collect::<Vec<_>>(),
            vec![(0, 2), (1, 1), (2, 0), (3, -1)]
        );
    }

    #[test]
    fn content_stop_prototypes_fit_and_shuffle_labels() {
        let mut rows = (0..20)
            .map(|index| ContentStopRow {
                signature: code_signature(index),
                terminal: true,
                writer_token: 7,
                offset: index as u16,
            })
            .collect::<Vec<_>>();
        rows.extend((100..120).map(|index| ContentStopRow {
            signature: code_signature(index),
            terminal: false,
            writer_token: 8,
            offset: index as u16,
        }));
        let training = fit_content_stop_gate(&rows, 4).unwrap();
        assert_eq!(
            (training.terminal_rows, training.nonterminal_rows),
            (20, 20)
        );
        assert!(training.terminal_stops > 0);
        assert_eq!(training.nonterminal_stops, 0);
        let shuffled = shuffle_content_stop_rows(&rows, 3).unwrap();
        assert_eq!(
            shuffled.iter().filter(|row| row.terminal).count(),
            training.terminal_rows as usize
        );
        assert!(shuffled
            .iter()
            .zip(&rows)
            .any(|(left, right)| left.terminal != right.terminal));
    }

    #[test]
    fn content_stop_prototype_selection_is_deterministic_on_lexicographic_ties() {
        let signatures = vec![
            code_signature(2),
            code_signature(0),
            code_signature(1),
            code_signature(0),
        ];
        let expected = vec![code_signature(0), code_signature(1)];
        assert_eq!(
            choose_content_prototypes(signatures.clone().into_iter(), 2).unwrap(),
            expected
        );
        assert_eq!(
            choose_content_prototypes(signatures.into_iter(), 2).unwrap(),
            expected
        );
    }

    #[test]
    fn content_stop_density_uses_all_rows_and_respects_generated_offset() {
        let mut rows = (0..20)
            .map(|index| ContentStopRow {
                signature: code_signature(index % 2),
                terminal: true,
                writer_token: 7,
                offset: 5,
            })
            .collect::<Vec<_>>();
        rows.extend((100..120).map(|index| ContentStopRow {
            signature: code_signature(100 + index % 2),
            terminal: false,
            writer_token: 8,
            offset: 5,
        }));
        let training = fit_content_stop_density_gate(&rows, 8, 0).unwrap();
        assert_eq!(training.terminal_rows, 20);
        assert_eq!(training.nonterminal_rows, 20);
        assert!(training.terminal_stops > 0);
        assert_eq!(training.nonterminal_stops, 0);
        assert_eq!(training.gate.terminal_rows.len(), 20);
        assert_eq!(training.gate.nonterminal_rows.len(), 20);
        assert!(content_stop_density_gate_fires(&training.gate, &code_signature(0), 5).unwrap());
        assert!(!content_stop_density_gate_fires(&training.gate, &code_signature(103), 5).unwrap());
        assert!(
            !content_stop_density_gate_fires(&training.gate, &code_signature(0), 2)
                .unwrap_or(false)
        );
        let mut reordered = rows.clone();
        reordered.reverse();
        assert_eq!(
            fit_content_stop_density_gate(&reordered, 8, 0).unwrap(),
            training
        );
        let shuffled = shuffle_content_stop_rows(&rows, 3).unwrap();
        assert_eq!(
            shuffled.iter().map(|row| row.offset).collect::<Vec<_>>(),
            rows.iter().map(|row| row.offset).collect::<Vec<_>>()
        );
        assert_eq!(
            shuffled.iter().filter(|row| row.terminal).count(),
            rows.iter().filter(|row| row.terminal).count()
        );
    }

    fn continuation_row(signature: usize, remaining: i16, offset: u16) -> ContentContinuationRow {
        ContentContinuationRow {
            signature: code_signature(signature),
            remaining,
            writer_token: 7,
            offset,
        }
    }

    #[test]
    fn continuation_fit_uses_exact_offsets_and_lower_median_horizon() {
        let mut rows = (0..4)
            .map(|index| continuation_row(index, if index == 0 { 0 } else { 2 }, 4))
            .collect::<Vec<_>>();
        rows.extend((0..4).map(|index| continuation_row(index, 1, 5)));
        let training = fit_content_continuation_gate(&rows, 1).unwrap();
        assert_eq!(training.gate.neighbors, 1);
        assert_eq!(training.rows, 8);
        assert_eq!(training.terminal_rows, 1);
        assert!(content_continuation_gate_fires(&training.gate, &code_signature(0), 4).unwrap());
        assert!(!content_continuation_gate_fires(&training.gate, &code_signature(0), 5).unwrap());

        let tied = continuation_row(0, 3, 4);
        let mut rows = vec![tied];
        for remaining in [0, 1, 2] {
            rows.push(continuation_row(0, remaining, 4));
        }
        let training = fit_content_continuation_gate(&rows, 4).unwrap();
        let mut reordered = rows.clone();
        reordered.reverse();
        assert_eq!(
            fit_content_continuation_gate(&reordered, 4).unwrap(),
            training
        );
        let tie_gate = ContentContinuationGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            rows: [0, 0, 1, 2]
                .into_iter()
                .map(|remaining| ContentContinuationPoint {
                    signature: code_signature(0),
                    remaining,
                    offset: 4,
                })
                .collect(),
            neighbors: 4,
        };
        assert!(content_continuation_gate_fires(&tie_gate, &code_signature(0), 4).unwrap());
    }

    #[test]
    fn continuation_shuffle_preserves_each_offset_horizon_multiset() {
        let rows = vec![
            continuation_row(0, 0, 4),
            continuation_row(1, 1, 4),
            continuation_row(2, 2, 4),
            continuation_row(3, -1, 5),
            continuation_row(4, 0, 5),
            continuation_row(5, 1, 5),
        ];
        let shuffled = shuffle_content_continuation_rows(&rows, 3).unwrap();
        assert_ne!(
            shuffled.iter().map(|row| row.remaining).collect::<Vec<_>>(),
            rows.iter().map(|row| row.remaining).collect::<Vec<_>>()
        );
        for offset in [4, 5] {
            let mut before = rows
                .iter()
                .filter(|row| row.offset == offset)
                .map(|row| row.remaining)
                .collect::<Vec<_>>();
            let mut after = shuffled
                .iter()
                .filter(|row| row.offset == offset)
                .map(|row| row.remaining)
                .collect::<Vec<_>>();
            before.sort_unstable();
            after.sort_unstable();
            assert_eq!(before, after);
        }
        assert_eq!(
            shuffle_content_continuation_rows(&rows, 3).unwrap(),
            shuffled
        );
    }

    #[test]
    fn continuation_gate_round_trips_and_fires_only_on_zero() {
        let rows = vec![
            continuation_row(0, 0, 4),
            continuation_row(1, 1, 4),
            continuation_row(2, 2, 4),
            continuation_row(3, 3, 4),
        ];
        let training = fit_content_continuation_gate(&rows, 1).unwrap();
        let encoded = serde_json::to_vec(&training.gate).unwrap();
        let decoded = serde_json::from_slice::<ContentContinuationGate>(&encoded).unwrap();
        assert_eq!(decoded, training.gate);
        assert!(content_continuation_gate_fires(&decoded, &code_signature(0), 4).unwrap());
        assert!(!content_continuation_gate_fires(&decoded, &code_signature(1), 4).unwrap());
        assert!(!content_continuation_gate_fires(&decoded, &code_signature(0), 5).unwrap());
        let mut malformed_gate = training.gate.clone();
        malformed_gate.rows[0].signature.pop();
        assert!(
            content_continuation_gate_prediction(&malformed_gate, &code_signature(0), 4).is_err()
        );
        let mut malformed_rows = rows;
        malformed_rows[0].signature.pop();
        assert!(fit_content_continuation_gate(&malformed_rows, 1).is_err());
    }

    #[test]
    fn content_stop_density_round_trips_and_validates_width() {
        let rows = (0..8)
            .map(|index| ContentStopRow {
                signature: code_signature(index),
                terminal: index < 4,
                writer_token: 7,
                offset: index as u16,
            })
            .collect::<Vec<_>>();
        let training = fit_content_stop_density_gate(&rows, 4, 2).unwrap();
        let encoded = serde_json::to_vec(&training).unwrap();
        assert_eq!(
            serde_json::from_slice::<ContentStopDensityTraining>(&encoded).unwrap(),
            training
        );
        let mut malformed = training.gate;
        malformed.terminal_rows[0].signature.pop();
        assert!(content_stop_density_gate_fires(&malformed, &code_signature(0), 0).is_err());
    }

    #[test]
    fn content_stop_margin_uses_the_full_one_percent_false_stop_boundary() {
        let terminal_signature = vec![0u64; CONTENT_CODE_SIGNATURE_BITS / 64];
        let nonterminal_majority = code_signature_bits([100, 101]);
        let boundary_outlier = code_signature_bits(0..10);
        let mut rows = (0..100)
            .map(|_| ContentStopRow {
                signature: terminal_signature.clone(),
                terminal: true,
                writer_token: 7,
                offset: 4,
            })
            .collect::<Vec<_>>();
        rows.extend((0..99).map(|_| ContentStopRow {
            signature: nonterminal_majority.clone(),
            terminal: false,
            writer_token: 8,
            offset: 0,
        }));
        rows.push(ContentStopRow {
            signature: boundary_outlier,
            terminal: false,
            writer_token: 8,
            offset: 1,
        });

        let training = fit_content_stop_gate(&rows, 1).unwrap();
        assert_eq!(training.nonterminal_stop_limit, 1);
        assert_eq!(training.terminal_stops, 100);
        assert_eq!(training.nonterminal_stops, 1);
        assert_eq!(training.gate.margin, 2);
    }

    #[test]
    fn content_stop_margin_refuses_when_every_threshold_exceeds_false_stop_limit() {
        let shared = vec![0u64; CONTENT_CODE_SIGNATURE_BITS / 64];
        let mut rows = (0..20)
            .map(|_| ContentStopRow {
                signature: shared.clone(),
                terminal: true,
                writer_token: 7,
                offset: 4,
            })
            .collect::<Vec<_>>();
        rows.extend((0..100).map(|_| ContentStopRow {
            signature: shared.clone(),
            terminal: false,
            writer_token: 8,
            offset: 0,
        }));

        assert!(fit_content_stop_gate(&rows, 8)
            .unwrap_err()
            .to_string()
            .contains("no margin under the false-stop limit"));
    }

    #[test]
    fn generated_stop_gate_preserves_content_until_it_fires() {
        let artifact = artifact();
        let samples = vec![sample()];
        let (writer, _) = generate_with_readout(
            &artifact,
            &samples[0],
            4,
            false,
            ContentSignatureMode::Code,
            false,
            None,
        )
        .unwrap();
        let content = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: writer
                .iter()
                .map(|decision| ContentExample {
                    signature: decision.signature.clone(),
                    target: decision.content_chosen,
                })
                .collect(),
            max_distance: 0,
        };
        let gate = ContentStopGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            terminal_prototypes: vec![writer[2].signature.clone()],
            nonterminal_prototypes: vec![writer[0].signature.clone()],
            margin: 1,
        };
        let evaluate = |threads| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                evaluate_content_stop_gate(&artifact, &samples, 4, false, &content, &gate).unwrap()
            })
        };
        let (reading, prompts) = evaluate(32);
        assert_eq!(reading.gate_fires, 1);
        assert_eq!(reading.gated_decisions, 1);
        assert_eq!(reading.ungated_token_mismatches, 0);
        assert_eq!(reading.bos_stopped, 1);
        assert_eq!(prompts[0].stop, "bos");
        assert_eq!(evaluate(1), evaluate(32));
    }

    #[test]
    fn generated_density_gate_preserves_content_until_it_fires() {
        let artifact = artifact();
        let samples = vec![sample()];
        let (writer, _) = generate_with_readout(
            &artifact,
            &samples[0],
            4,
            false,
            ContentSignatureMode::Code,
            false,
            None,
        )
        .unwrap();
        let content = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: writer
                .iter()
                .map(|decision| ContentExample {
                    signature: decision.signature.clone(),
                    target: decision.content_chosen,
                })
                .collect(),
            max_distance: 0,
        };
        let gate = ContentStopDensityGate {
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            terminal_rows: vec![ContentStopDensityPoint {
                signature: writer[2].signature.clone(),
                offset: 2,
            }],
            nonterminal_rows: vec![
                ContentStopDensityPoint {
                    signature: writer[0].signature.clone(),
                    offset: 0,
                },
                ContentStopDensityPoint {
                    signature: writer[1].signature.clone(),
                    offset: 1,
                },
            ],
            neighbors: 1,
            offset_radius: 0,
            margin: 1,
        };
        let evaluate = |threads| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                evaluate_content_stop_density_gate(&artifact, &samples, 4, false, &content, &gate)
                    .unwrap()
            })
        };
        let (reading, prompts) = evaluate(1);
        assert_eq!(evaluate(32), evaluate(1));
        assert_eq!(reading.gate_fires, 1);
        assert_eq!(reading.gated_decisions, 1);
        assert_eq!(reading.ungated_token_mismatches, 0);
        assert_eq!(reading.bos_stopped, 1);
        assert_eq!(prompts[0].stop, "bos");
    }

    #[test]
    fn serving_sidecar_reproduces_the_cpu_content_decisions() {
        let artifact = artifact();
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![ContentExample {
                signature: vec![0; CONTENT_CODE_SIGNATURE_BITS / 64],
                target: 5,
            }],
            max_distance: CONTENT_CODE_SIGNATURE_BITS as u32,
        };
        let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
        let sidecar = serde_json::to_vec(&serde_json::json!({
            "schema": "uor-r4.stack-content-readout/1",
            "artifact_sha256": artifact_sha256,
            "readout": &readout,
        }))
        .unwrap();

        let parity =
            verify_content_readout_serving(&artifact, &[sample()], 4, &readout, &sidecar).unwrap();
        assert_eq!(parity.prompts, 1);
        assert!(parity.decisions > 0);
        assert_eq!(parity.readout_fires, parity.decisions);
        assert_eq!(parity.token_mismatches, 0);
        assert_eq!(parity.decision_mismatches, 0, "{parity:?}");
        assert_eq!(parity.first_mismatch, None);
        assert_eq!(parity.cpu_decision_sha256, parity.serving_decision_sha256);
    }
}
