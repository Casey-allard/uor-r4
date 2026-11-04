//! Exact Hamming nearest-example content readout on generated prefixes.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::reply_eval::normalized_reference;
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CONTENT_SIGNATURE_BITS: usize = 256;
pub const CONTENT_SIGNATURE_WORDS: usize = CONTENT_SIGNATURE_BITS / 64;
pub const CONTENT_MIN_TRAINING_ROWS: usize = 640;
pub const CONTENT_MIN_TARGET_VALUES: usize = 16;
pub type ContentSignature = [u64; CONTENT_SIGNATURE_WORDS];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExample {
    pub signature: ContentSignature,
    pub target: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadout {
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
    pub first_divergence: BTreeMap<usize, u64>,
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
}

pub fn pack_content_signature(query: &[i32]) -> Result<ContentSignature> {
    if query.len() != CONTENT_SIGNATURE_BITS {
        return Err(invalid("content signature query width differs"));
    }
    let mut signature = [0u64; CONTENT_SIGNATURE_WORDS];
    for (index, &value) in query.iter().enumerate() {
        if value >= 0 {
            signature[index / 64] |= 1u64 << (index % 64);
        }
    }
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
) -> Result<ContentReadoutTraining> {
    if samples.is_empty() {
        return Err(invalid("content readout needs training samples"));
    }
    let rows = samples
        .par_iter()
        .map(|sample| {
            generate_with_readout(artifact, sample, cap, memory_enabled, None).map(
                |(decisions, _)| {
                    decisions
                        .into_iter()
                        .filter_map(|decision| {
                            decision.reference_target.map(|target| ContentExample {
                                signature: decision.signature,
                                target,
                            })
                        })
                        .collect::<Vec<_>>()
                },
            )
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
    fit_content_readout(rows, target_values)
}

fn fit_content_readout(
    rows: Vec<ContentExample>,
    target_values: usize,
) -> Result<ContentReadoutTraining> {
    if rows.is_empty() {
        return Err(invalid("content readout fitting needs rows"));
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
        readout: ContentReadout { rows, max_distance },
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
        .map(|row| row.signature)
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
    let mut bytes = Vec::with_capacity(mapping.len() * (CONTENT_SIGNATURE_BITS + 4));
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
    readout: Option<&ContentReadout>,
) -> Result<(ContentReadoutReading, Vec<ContentPromptReading>)> {
    if let Some(readout) = readout {
        validate_content_readout(readout)?;
    }
    let mut reading = ContentReadoutReading {
        prompts: samples.len() as u64,
        ..ContentReadoutReading::default()
    };
    let evaluations = samples
        .par_iter()
        .enumerate()
        .map(|(sample_index, sample)| {
            generate_with_readout(artifact, sample, cap, memory_enabled, readout)
                .map(|(decisions, reply)| (sample_index, sample, decisions, reply))
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
        if !reply.tokens.is_empty() {
            distinct.insert(reply.tokens.clone());
        }
        reading.repeated_4gram_replies += u64::from(has_repeated_4gram(&reply.tokens));
        for decision in &decisions {
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
        });
    }
    reading.distinct_non_empty = distinct.len() as u64;
    Ok((reading, prompts))
}

struct Decision {
    offset: usize,
    chosen: u32,
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
    let stop = loop {
        let offset = decisions.len();
        let signature = pack_content_signature(&current.query_q12)?;
        let (chosen, nearest_distance, ambiguous_nearest, used_readout) = match readout {
            Some(readout) => select_content_target(readout, &signature, current.token)?,
            None => (current.token, None, false, false),
        };
        decisions.push(Decision {
            offset,
            chosen,
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
            .map_err(|error| invalid(format!("content readout generated step: {error}")))?;
    };
    Ok((decisions, Generated { tokens, stop }))
}

fn validate_content_readout(readout: &ContentReadout) -> Result<()> {
    if readout.rows.is_empty() {
        return Err(invalid("content readout has no rows"));
    }
    if readout.max_distance as usize > CONTENT_SIGNATURE_BITS {
        return Err(invalid("content readout threshold exceeds the signature"));
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
        let mut signature = [0u64; CONTENT_SIGNATURE_WORDS];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn row(bit: usize, target: u32) -> ContentExample {
        ContentExample {
            signature: if bit == CONTENT_SIGNATURE_BITS {
                [0; CONTENT_SIGNATURE_WORDS]
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
        let packed = pack_content_signature(&query).unwrap();
        assert_eq!(
            content_hamming(&packed, &[0; CONTENT_SIGNATURE_WORDS]),
            (0..CONTENT_SIGNATURE_BITS)
                .filter(|index| index % 3 != 0)
                .count() as u32
        );
        assert!(pack_content_signature(&query[..255]).is_err());
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
        let training = fit_content_readout(rows, 2).unwrap();
        assert_eq!(training.covered_rows, 90);
        assert_eq!(training.rows, 100);
        assert_eq!(training.readout.max_distance, 0);
        assert!(training.covered_rows * 10 >= training.rows * 9);
        assert!(fit_content_readout(Vec::new(), 0).is_err());
    }

    #[test]
    fn shuffle_is_deterministic_changes_assignments_and_refuses_empty_rows() {
        let readout = ContentReadout {
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
                .map(|row| row.signature)
                .collect::<Vec<_>>(),
            readout
                .rows
                .iter()
                .map(|row| row.signature)
                .collect::<Vec<_>>()
        );
        assert!(shuffle_content_targets(
            &ContentReadout {
                rows: vec![row(0, 1)],
                max_distance: 0,
            },
            1
        )
        .is_err());
        let repeated = ContentReadout {
            rows: vec![row(0, 9), row(0, 7), row(0, 9), row(0, 7)],
            max_distance: 0,
        };
        assert!(shuffle_content_targets(&repeated, 2).is_err());
    }

    #[test]
    fn readout_uses_a_matching_row_and_falls_back_beyond_the_threshold() {
        let readout = ContentReadout {
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
        let (writer, _) = generate_with_readout(&artifact, &samples[0], 4, false, None).unwrap();
        let readout = ContentReadout {
            rows: vec![ContentExample {
                signature: writer[0].signature,
                target: writer[0]
                    .reference_target
                    .expect("the first generated row has a reference target"),
            }],
            max_distance: 0,
        };
        let (first_decisions, first_reply) =
            generate_with_readout(&artifact, &samples[0], 4, false, Some(&readout)).unwrap();
        let (second_decisions, second_reply) =
            generate_with_readout(&artifact, &samples[0], 4, false, Some(&readout)).unwrap();
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
                evaluate_content_readout(&artifact, &samples, 4, false, Some(&readout)).unwrap()
            })
        };
        assert_eq!(evaluate(1), evaluate(32));
    }
}
